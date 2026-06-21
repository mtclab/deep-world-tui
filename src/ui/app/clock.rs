use crate::model::{GodName, ItemType, TensionEvent, Terrain, Weather};
use crate::sim::hints;

use super::*;

/// A day's gift-strain at or above which the day counts as worked-to-the-bone.
const GIFT_OVERWORK_DAY: f64 = 1.0;
/// Consecutive worked-to-the-bone days that settle into the chronic iron-ache.
const GIFT_IRON_ACHE_DAYS: u32 = 3;

impl App {
    /// At the day's turn, reckon the gift's toll (#427): a day worked to the
    /// bone counts toward the chronic iron-ache (rauta-särky); three such days
    /// running settle it into the body. Then the day's strain resets.
    fn settle_gift_strain(&mut self) {
        if self.gift_strain >= GIFT_OVERWORK_DAY {
            self.gift_overworked_days = self.gift_overworked_days.saturating_add(1);
            if self.gift_overworked_days >= GIFT_IRON_ACHE_DAYS {
                let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
                if let Some(ps) = self.player_start.as_mut() {
                    let has = ps
                        .person
                        .illnesses
                        .iter()
                        .any(|d| d.disease == crate::model::Disease::IronAche);
                    if !has && ps.person.illnesses.len() < 2 {
                        ps.person.illnesses.push(crate::model::ActiveDisease::new(
                            crate::model::Disease::IronAche,
                            tick,
                        ));
                        self.status_msg = Some(
                            "Days of the gift have settled into your bones — iron-ache.".into(),
                        );
                    }
                }
                self.gift_overworked_days = 0;
            }
        } else {
            self.gift_overworked_days = 0;
        }
        self.gift_strain = 0.0;
    }

    /// The seasonal world-event gripping the world right now, if any (#417):
    /// a market fair, a hard winter, a plague year. Deterministic per
    /// seed + season + year (a 90-day, three-season year).
    pub fn current_world_event(&self) -> Option<crate::model::WorldEvent> {
        let year = self.clock.day / 90;
        crate::model::WorldEvent::current(self.seed, self.clock.season(), year)
    }

    /// Today's sky over a region (the weather-front system's state).
    pub fn region_weather(&self, region_idx: usize) -> Weather {
        self.sim
            .as_ref()
            .and_then(|s| s.world.regions.get(region_idx))
            .map(|r| r.weather)
            .unwrap_or(Weather::Clear)
    }

    /// Open the rest-duration picker.
    pub fn open_rest_prompt(&mut self) {
        self.screen = Screen::RestPrompt {
            hours: DEFAULT_REST_HOURS,
        };
    }

    pub const MAX_REST_HOURS: u32 = MAX_REST_HOURS;

    /// How well the player is dressed against harsh weather (#689): the factor
    /// the harsh-weather body-drain is scaled by — lower is warmer. A worn coat
    /// halves it (the old hunt-to-warmth loop); a fur cloak does better; humbler
    /// wool or felt, less. The best worn piece sets the floor, and a fur or coat
    /// layered over wool/felt does a little better still — so the cold-gear the
    /// economy produces actually matters in Frost and on the tundra. Returns the
    /// factor and the piece that earns it (to wear that piece down, not another).
    pub(super) fn warmth_factor(&self) -> (f64, Option<ItemType>) {
        let Some(ps) = self.player_start.as_ref() else {
            return (1.0, None);
        };
        let owned = |it: ItemType| ps.inventory.has(it) && !ps.inventory.is_broken(it);
        let good = |slug: &str| crate::model::good_id(slug).map(ItemType::Good);
        // (piece, factor) warmest first; only registry goods that exist resolve.
        let mut tiers: Vec<(ItemType, f64)> = vec![(ItemType::Coat, 0.5)];
        if let Some(fur) = good("fur") {
            tiers.push((fur, 0.4));
        }
        if let Some(felt) = good("felt") {
            tiers.push((felt, 0.72));
        }
        if let Some(wool) = good("wool") {
            tiers.push((wool, 0.8));
        }
        // The warmest single piece the player actually wears.
        let mut best: (f64, Option<ItemType>) = (1.0, None);
        for (it, f) in &tiers {
            if owned(*it) && *f < best.0 {
                best = (*f, Some(*it));
            }
        }
        // Layering: a coat or fur over a humbler wool/felt undergarment shaves a
        // little more off the bite.
        let has_outer = owned(ItemType::Coat) || good("fur").is_some_and(owned);
        let has_under = good("felt").is_some_and(owned) || good("wool").is_some_and(owned);
        if has_outer && has_under {
            best.0 *= 0.9;
        }
        best
    }

    pub fn advance_clock(&mut self, hours: u32) {
        let day_before = self.clock.day;
        let season = self.clock.season();
        self.clock.advance(hours);
        // The turning of the year is named for the player (#570 slice 4): when
        // the season changes, its word reaches them on the road and in the
        // journal — winter's bite, the green plenty, the thaw's opening.
        let season_now = self.clock.season();
        if season_now != season {
            let line = season_now.turn_announcement();
            self.status_msg = Some(line.to_string());
            if let Some(ref mut sim) = self.sim {
                sim.log(
                    sim.world.tick,
                    crate::sim::journal::Voice::Rumor,
                    line.to_string(),
                );
            }
        }
        // Harsh weather wears the body down faster too (need_decay_modifier
        // was defined per-weather but never applied to the player). The life's
        // hidden star leans the harshness — only the penalty over fair weather,
        // so a blessed soul bears the cold a little better and the cursed a
        // little worse; clear skies fall on everyone the same. A worn coat
        // halves the harsh-weather penalty (#414) — the hunt-to-warmth loop.
        // Graded cold-gear (#689): the warmest worn piece sets the harsh-weather
        // factor; the piece that earns it is the one that wears down.
        let (coat_factor, warm_piece) = self.warmth_factor();
        // A declared hard winter bites deeper than an ordinary Frost (#417).
        let event_weather = self
            .current_world_event()
            .map(|e| e.weather_decay_modifier())
            .unwrap_or(1.0);
        let mut weather_harsh = false;
        let weather_mult = self
            .player_pos
            .map(|pos| {
                let raw = self.region_weather(pos.region_idx).need_decay_modifier();
                let harsh_excess = (raw - 1.0).max(0.0);
                weather_harsh = harsh_excess > 0.0;
                // Kukri's vow: the patient cold cannot wear the sworn (#457).
                1.0 + harsh_excess
                    * coat_factor
                    * event_weather
                    * self.fortune.bad_multiplier()
                    * self.vow_weather_mult()
            })
            .unwrap_or(1.0);
        let mut departed: Vec<String> = Vec::new();
        if let Some(ref mut ps) = self.player_start {
            // A sick player wears down faster (worst active disease sets the
            // rate), and untreated illness slowly worsens.
            for d in ps.person.illnesses.iter_mut() {
                d.worsen(hours);
            }
            let illness_mult = ps
                .person
                .illnesses
                .iter()
                .map(|d| d.vitals_modifier())
                .fold(1.0_f64, f64::max);
            self.vitals.tick_with_illness(
                hours,
                &mut ps.inventory,
                season,
                illness_mult * weather_mult,
            );
            // Cold-gear earns its keep by wearing out: harsh weather frays the
            // piece that's keeping you warm (#689).
            if weather_harsh {
                if let Some(piece) = warm_piece {
                    ps.inventory.decay(piece, 0.02 * hours as f64);
                }
            }
            for companion in &mut ps.companions {
                companion.decay_needs(hours as u64);
            }
            // A companion neglected until its needs max out leaves (or dies).
            // Nothing enforced this, so is_alive() was dead and starved
            // companions lingered forever at full need, Unhappy and idle.
            ps.companions.retain(|c| {
                if c.is_alive() {
                    true
                } else {
                    departed.push(c.name.clone());
                    false
                }
            });
        }
        for name in departed {
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    format!(
                        "{} could bear it no longer and slipped away. I kept nothing.",
                        name
                    ),
                );
            }
            self.status_msg = Some(format!("{} left — too long neglected.", name));
        }
        if let Some(ref mut sim) = self.sim {
            for _ in 0..hours {
                sim.step();
            }
        }
        if let Some((event, a, b)) = TensionEvent::roll(self.seed, self.clock.day) {
            let shift = event.bias_shift();
            if self.inter_people_bias.player_people == a {
                self.inter_people_bias.mod_toward(b, shift);
            } else if self.inter_people_bias.player_people == b {
                self.inter_people_bias.mod_toward(a, -shift);
            }
            self.status_msg = Some(event.flavor(a, b));
        }
        self.check_milestones();
        // An elder's regard grows by the day, not by the deed — it used to
        // tick on every action, so a busy elder saturated the town's esteem
        // in an afternoon.
        if self.elder && self.clock.day != day_before {
            let elder_settlement_id = self
                .sim
                .as_ref()
                .and_then(|s| {
                    let pos = self.player_pos?;
                    let region = s.world.regions.get(pos.region_idx)?;
                    region.settlements.first().map(|s| s.id.clone())
                })
                .unwrap_or_default();
            let elder_player_id = self
                .player_start
                .as_ref()
                .map(|ps| ps.person.id.clone())
                .unwrap_or_default();
            if let Some(ref mut sim) = self.sim {
                if !elder_player_id.is_empty() && !elder_settlement_id.is_empty() {
                    sim.reputation
                        .adjust_settlement(&elder_player_id, &elder_settlement_id, 0.005);
                }
            }
        }
        self.check_quests_on_tick();
        self.check_collapse();
        self.check_aging();
        self.check_player_illness();
        self.tick_player_farms();
        if self.clock.day != day_before {
            self.check_spouse();
            self.maybe_omen();
            self.settle_gift_strain();
            // Disease is the great leveller of the post-Fall age (once a day).
            self.check_illness_mortality();
            // And the broken peace claims its own in the tension seasons.
            self.check_turmoil();
            // A vow kept too thinly breaks of itself (#457).
            self.check_vow();
        }
        // The season-turn reckoning: every thirty days the polity's assessor
        // comes for the hearth-tax (#396), once per season.
        let day = self.clock.day;
        if day > 0 && day.is_multiple_of(30) && day != self.last_tax_day {
            self.last_tax_day = day;
            self.assess_hearth_tax();
        }
        // The founding check asks the roads every ten days; the waystation
        // ledger keeps the same calendar.
        if self.clock.day / 10 > self.founding_check_day / 10 {
            self.founding_check_day = self.clock.day;
            self.tick_founding();
            self.tick_waystations();
            self.tick_household();
        }
    }

    pub(super) fn log_travel(&mut self, terrain: Terrain) {
        if let Some(ref mut sim) = self.sim {
            let tod = self.clock.time_of_day();
            let weather = self
                .player_pos
                .and_then(|pos| sim.world.regions.get(pos.region_idx))
                .map(|r| r.weather)
                .unwrap_or(Weather::Clear);
            let _ = terrain;
            let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                .fork_for(&format!("travel-journal-{}", sim.world.tick));
            let text = crate::sim::journal::travel_text(&mut rng, tod, weather);
            sim.log(sim.world.tick, crate::sim::journal::Voice::Travel, text);
        }
    }

    pub fn check_memorial(&mut self) {
        if let Some(pos) = self.player_pos {
            if let Some(ref sim) = self.sim {
                if let Some(memorial) = sim
                    .memorials
                    .iter()
                    .find(|m| m.at_position(pos.region_idx, pos.px as u32, pos.py as u32))
                {
                    self.status_msg = Some(memorial.text.clone());
                }
            }
        }
    }

    /// The living world calls for help (#613-epic): towns thrown into plague or
    /// famine by the daily sim post a relief task the player can take up, so the
    /// drama the systems make becomes something to *do*, not only to read. Capped
    /// so a plague-year does not flood the list, deduped by town, and surfaced as
    /// word on the road. Completed by the act of tending/provisioning (see those
    /// methods); expired on their deadline by the generic quest check.
    pub(super) fn generate_world_task_quests(&mut self) {
        use crate::model::quest::{Quest, QuestKind, QuestReward};
        const MAX_WORLD_TASKS: usize = 4;
        let day = self.clock.day;
        let Some(sim) = self.sim.as_mut() else {
            return;
        };
        let mut active = 0usize;
        let mut have_plague: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut have_famine: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut have_truce: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let mut have_faith: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut have_supply: std::collections::HashSet<String> = std::collections::HashSet::new();
        for q in &sim.quests {
            match &q.kind {
                QuestKind::RelievePlague { settlement } => {
                    active += 1;
                    have_plague.insert(settlement.clone());
                }
                QuestKind::RelieveFamine { settlement } => {
                    active += 1;
                    have_famine.insert(settlement.clone());
                }
                QuestKind::BrokerTruce { a, b } => {
                    active += 1;
                    have_truce.insert((a.clone(), b.clone()));
                }
                QuestKind::SteadyFaith { settlement } => {
                    active += 1;
                    have_faith.insert(settlement.clone());
                }
                QuestKind::SupplyGoods { settlement } => {
                    active += 1;
                    have_supply.insert(settlement.clone());
                }
                _ => {}
            }
        }
        let mut new_quests: Vec<Quest> = Vec::new();
        let mut msgs: Vec<String> = Vec::new();
        'scan: for region in &sim.world.regions {
            for s in &region.settlements {
                if active + new_quests.len() >= MAX_WORLD_TASKS {
                    break 'scan;
                }
                if s.population == 0 {
                    continue;
                }
                if s.plague_days > 0 && !have_plague.contains(&s.name) {
                    new_quests.push(Quest {
                        kind: QuestKind::RelievePlague {
                            settlement: s.name.clone(),
                        },
                        description: format!("Carry medicine to {} — a plague grips it.", s.name),
                        reward: QuestReward::Reputation { amount: 0.15 },
                        progress: 0,
                        target: 1,
                        deadline_day: day + 30,
                        assigned_day: day,
                    });
                    msgs.push(format!(
                        "Word on the road: a sickness has {} in its grip — they need medicine.",
                        s.name
                    ));
                } else if s.famine_days > 0 && !have_famine.contains(&s.name) {
                    new_quests.push(Quest {
                        kind: QuestKind::RelieveFamine {
                            settlement: s.name.clone(),
                        },
                        description: format!("Provision {} — its stores stand empty.", s.name),
                        reward: QuestReward::Reputation { amount: 0.12 },
                        progress: 0,
                        target: 1,
                        deadline_day: day + 30,
                        assigned_day: day,
                    });
                    msgs.push(format!(
                        "Word on the road: {} has gone hungry — they would pay well for grain.",
                        s.name
                    ));
                } else if s.faith.is_contested() && !have_faith.contains(&s.name) {
                    new_quests.push(Quest {
                        kind: QuestKind::SteadyFaith {
                            settlement: s.name.clone(),
                        },
                        description: format!(
                            "Steady the faith of {} — two gods contend, and a schism looms.",
                            s.name
                        ),
                        reward: QuestReward::Reputation { amount: 0.14 },
                        progress: 0,
                        target: 1,
                        deadline_day: day + 35,
                        assigned_day: day,
                    });
                    msgs.push(format!(
                        "Word on the road: {}'s faith is split two ways — a devotee could steady it.",
                        s.name
                    ));
                }
            }
        }
        // Deep, raiding rivalries call for a peacemaker (#614 slice 2): a pair
        // whose bad blood runs past the raiding mark posts a broker-truce task.
        // Pairs are stored name-ordered by the province ties, so the dedup key
        // matches either way.
        let mut deep_rivals: Vec<(String, String)> = sim
            .province_ties
            .bonds
            .iter()
            .filter(|(_, &v)| v <= -0.7)
            .map(|((a, b), _)| (a.clone(), b.clone()))
            .collect();
        // bonds is a HashMap, whose iteration order is not stable across runs;
        // sort so which truce tasks post (when more deep rivals exist than the
        // cap allows) is deterministic, not down to hash seeding.
        deep_rivals.sort();
        for (a, b) in deep_rivals {
            if active + new_quests.len() >= MAX_WORLD_TASKS {
                break;
            }
            if have_truce.contains(&(a.clone(), b.clone())) {
                continue;
            }
            new_quests.push(Quest {
                kind: QuestKind::BrokerTruce {
                    a: a.clone(),
                    b: b.clone(),
                },
                description: format!(
                    "Broker a truce between {a} and {b} — carry goods where no cart will cross."
                ),
                reward: QuestReward::Reputation { amount: 0.18 },
                progress: 0,
                target: 1,
                deadline_day: day + 40,
                assigned_day: day,
            });
            msgs.push(format!(
                "Word on the road: bad blood between {a} and {b} — a peacemaker could ease it."
            ));
        }
        // Goods-supply tasks are the lowest priority (#614 slice 4): a town cut
        // off from trade is a slow ache, not the emergency a plague, famine,
        // schism, or raiding feud is — and goods-starved towns are common, so
        // posting them last keeps them from crowding the urgent calls out of the
        // task cap. They fill only whatever slots are left.
        'supply: for region in &sim.world.regions {
            for s in &region.settlements {
                if active + new_quests.len() >= MAX_WORLD_TASKS {
                    break 'supply;
                }
                if s.population == 0 || have_supply.contains(&s.name) || !s.is_goods_starved() {
                    continue;
                }
                new_quests.push(Quest {
                    kind: QuestKind::SupplyGoods {
                        settlement: s.name.clone(),
                    },
                    description: format!(
                        "Supply {} — cut off from trade, it wants tools and cloth.",
                        s.name
                    ),
                    reward: QuestReward::Reputation { amount: 0.13 },
                    progress: 0,
                    target: 1,
                    deadline_day: day + 30,
                    assigned_day: day,
                });
                msgs.push(format!(
                    "Word on the road: {} has run out of tools and cloth — a trader could set it right.",
                    s.name
                ));
            }
        }
        let tick = sim.world.tick;
        for q in new_quests {
            sim.quests.push(q);
        }
        for m in msgs {
            sim.log(tick, crate::sim::journal::Voice::Rumor, m);
        }
    }

    /// Towns post **living, stale-able bounties** on the bands that prey them
    /// (#623 slice 5). A bounty goes up when a band raids a town and is left up
    /// for a long age — so a notice on the board may be years old and its band
    /// long scattered, settled, or roamed away: a real cold trail the reader
    /// only learns by checking. Posted separately from the relief calls (its own
    /// small cap), deduped by band, and left to expire on its own long deadline.
    pub(super) fn generate_band_bounties(&mut self) {
        use crate::model::quest::{Quest, QuestKind, QuestReward};
        const BOUNTY_CAP: usize = 3;
        let day = self.clock.day;
        let Some(sim) = self.sim.as_mut() else {
            return;
        };
        let mut active = 0usize;
        let mut have: std::collections::HashSet<String> = std::collections::HashSet::new();
        for q in &sim.quests {
            if let QuestKind::BountyOnBand { band_id, .. } = &q.kind {
                active += 1;
                have.insert(band_id.clone());
            }
        }
        let mut new_quests: Vec<Quest> = Vec::new();
        let mut msgs: Vec<String> = Vec::new();
        for band in &sim.frontier.bands {
            if active + new_quests.len() >= BOUNTY_CAP {
                break;
            }
            if have.contains(&band.id) {
                continue;
            }
            // The band must actually be preying on a town for that town to know
            // of it and put coin on its head — its own country's, or a
            // neighbour's if the band raids the settled edge from a march (#630).
            let Some(town) = crate::sim::frontier::band_prey_town(sim, band.region_idx) else {
                continue;
            };
            new_quests.push(Quest {
                kind: QuestKind::BountyOnBand {
                    band_id: band.id.clone(),
                    band_name: band.name.clone(),
                    settlement: town.clone(),
                },
                description: format!(
                    "{town} will pay to be rid of {} — the band that raids their country.",
                    band.name
                ),
                reward: QuestReward::Reputation { amount: 0.2 },
                progress: 0,
                target: 1,
                // A long age: the notice outlives the band, going cold on the
                // board for whoever reads it too late.
                deadline_day: day + 365,
                assigned_day: day,
            });
            msgs.push(format!(
                "Word on the road: {town} has put a bounty on {}, who raid their country.",
                band.name
            ));
        }
        let tick = sim.world.tick;
        for q in new_quests {
            sim.quests.push(q);
        }
        for m in msgs {
            sim.log(tick, crate::sim::journal::Voice::Rumor, m);
        }
    }

    /// Whether a band-bounty's trail has gone cold — the named band is no longer
    /// abroad in the world (scattered, settled, or simply gone). The board still
    /// shows the notice; only checking against the living world tells the reader
    /// the quarry is gone (#623 slice 5).
    pub fn band_bounty_is_cold(&self, band_id: &str) -> bool {
        self.sim
            .as_ref()
            .map(|sim| !sim.frontier.bands.iter().any(|b| b.id == band_id))
            .unwrap_or(true)
    }

    /// Pay out any standing `BountyOnBand` for a band the player has just
    /// scattered for good: reward into the player's local standing, quest
    /// cleared, milestone recorded. Returns whether a bounty was claimed.
    fn award_band_bounty(&mut self, band_id: &str) -> bool {
        use crate::model::quest::QuestKind;
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let here_id = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                sim.world
                    .regions
                    .get(pos.region_idx)?
                    .settlements
                    .first()
                    .map(|s| s.id.clone())
            })
            .unwrap_or_default();
        let bounty = self.sim.as_ref().and_then(|sim| {
            sim.quests.iter().enumerate().find_map(|(i, q)| {
                matches!(&q.kind, QuestKind::BountyOnBand { band_id: b, .. } if b == band_id)
                    .then(|| (i, q.reward.clone()))
            })
        });
        let Some((idx, reward)) = bounty else {
            return false;
        };
        if let (Some(ps), Some(sim)) = (self.player_start.as_mut(), self.sim.as_mut()) {
            crate::sim::quest_gen::apply_quest_reward(
                &reward,
                &mut ps.inventory,
                &mut sim.reputation,
                &player_id,
                &here_id,
                &mut sim.relationships,
                sim.world.tick,
            );
        }
        if let Some(ref mut sim) = self.sim {
            if idx < sim.quests.len() {
                sim.quests.remove(idx);
            }
        }
        self.milestones.record_quest_completed(self.clock.day);
        true
    }

    /// What a felled outlaw actually had on them (#685): outlaws are poor, and
    /// you find only what they carried — a few coins, perhaps the rags off their
    /// back or a worn tool, now and then a trinket they stole. Nothing summoned
    /// from nowhere; a big band yields more only because there are more bodies to
    /// search, each as meagre as the last. Looted gear comes worn — low
    /// durability, in want of mending — and ill-fitting (it was the dead's, not
    /// yours). Deterministic from the seed and a per-body salt. Returns the note
    /// for the status line (empty when there was nothing worth taking).
    fn loot_fallen_outlaw(&mut self, salt: u64) -> String {
        use crate::model::ItemType;
        let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(salt ^ 0x0107_1007));
        let r1 = crate::rng::unit_from_hash(h);
        let r2 = crate::rng::unit_from_hash(crate::rng::mix_u64(h ^ 0xBEEF));
        let r3 = crate::rng::unit_from_hash(crate::rng::mix_u64(h ^ 0x0005_1A52));
        // The purse: most carry a coin or three; a lucky body a little more.
        // An outlaw's purse is an outlaw's purse — not scaled to your wanting.
        let coins: u32 = match (r1 * 100.0) as u32 {
            0..=34 => 0,
            35..=69 => 1 + (h % 2) as u32,
            70..=89 => 2 + (h % 3) as u32,
            90..=98 => 4 + (h % 4) as u32,
            _ => 8 + (h % 5) as u32,
        };
        let mut taken: Vec<String> = Vec::new();
        if coins > 0 {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(ItemType::Coin, coins);
            }
            taken.push(format!("{coins} coin{}", if coins == 1 { "" } else { "s" }));
        }
        // The one thing on their back, if any — worn, and theirs not yours.
        let worn = 0.15 + r3 * 0.30; // 0.15..0.45 durability: in want of mending
        let item: Option<(ItemType, &str)> = if r2 < 0.30 {
            Some((ItemType::Cloth, "a bundle of rags"))
        } else if r2 < 0.45 {
            Some((ItemType::Tool, "a worn tool"))
        } else if r2 < 0.55 {
            Some((ItemType::Leather, "a scrap of cracked leather"))
        } else if r2 < 0.60 {
            // A coat cut to the dead's frame hangs ill on you until refitted —
            // carried as low condition.
            Some((
                ItemType::Coat,
                "an ill-fitting coat, much in want of mending",
            ))
        } else if r2 < 0.64 {
            // Rarer still: a trinket they stole, the one thing of worth.
            let slugs = ["amber", "silver", "garnet", "copper"];
            let pick = (h >> 8) as usize % slugs.len();
            crate::model::good_id(slugs[pick]).map(|g| (ItemType::Good(g), "a stolen trinket"))
        } else {
            None
        };
        if let Some((it, desc)) = item {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add_with_quality(it, 1, worn);
            }
            taken.push(desc.to_string());
        }
        if taken.is_empty() {
            String::new()
        } else {
            format!(" You search the fallen: {}.", taken.join(", "))
        }
    }

    /// The signal fire's worth in a fight (#663): a kept beacon of the player's
    /// standing near them in the dark hours steadies the hand — the lit dark is
    /// the safer dark. Returns the factor the combat press is scaled by: ×0.6 in
    /// the light of one's own beacon at night, ×1.0 otherwise (by day, or away
    /// from the fire). Was the beacon's `halve_night_encounters` worth before the
    /// encounter system was retired (#649); brought onto the grid combat it now
    /// feeds into.
    pub(super) fn beacon_press_factor(&self) -> f64 {
        if self.clock.time_of_day().is_dark()
            && self.own_structure_near(crate::sim::structures::BuildKind::Beacon, 3)
        {
            0.6
        } else {
            1.0
        }
    }

    /// One stand-up exchange with a band on the grid (#637): the player strikes,
    /// cutting the band's strength; the band fights back, and the press of it
    /// wears the player down (energy — the same exhaustion that funnels to a
    /// collapse, so a long fight against a big band can put you on the ground).
    /// Scatter the band and the bounty on it is yours. Bumping again strikes
    /// again — the roguelike turn loop, no menu. Sets the status line for the
    /// blow. Returns true while the band still stands (more bumps to come).
    pub(super) fn bump_attack_band(&mut self, band_id: &str) {
        // One strike fells the one outlaw you stepped into — a band is fought a
        // man at a time. A tool in hand wears you less than bare fists.
        let armed = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.get(crate::model::ItemType::Tool) > 0)
            .unwrap_or(false);
        let Some((name, scattered, remaining)) = self
            .sim
            .as_mut()
            .and_then(|sim| crate::sim::frontier::strike_band(sim, band_id, 1))
        else {
            return;
        };
        // Each cut fells one outlaw; you find only what was on that body
        // (#685). Salt by the band and the body's ordinal so each search is its
        // own, deterministic, meagre find.
        let loot_salt = crate::rng::fnv1a_hash(band_id) ^ remaining as u64;
        let loot = self.loot_fallen_outlaw(loot_salt);
        if scattered {
            let claimed = self.award_band_bounty(band_id);
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Travel,
                    format!("I cut down the last of {name} where they stood."),
                );
            }
            self.status_msg = Some(if claimed {
                format!("You cut down the last of {name} — the bounty on them is yours.{loot}")
            } else {
                format!("You cut down the last of {name}. The country is the quieter for it.{loot}")
            });
            return;
        }
        // The rest of the band fights back: the more still standing, the harder
        // they press. Energy is the wound here — run it out against a big band
        // and you go down through the existing collapse funnel. A tool eases it.
        // Balanced (#637 slice 5) so the toll is tense, not a death sentence:
        // clearing a band of N costs about 0.02·N + 0.005·N² energy — a band of
        // 6 wears you (~0.3, ~0.2 armed), a band of 12 nearly puts you down
        // (~0.96, ~0.67 armed) so the deep wild's big bands are to be feared and
        // picked off, not waded into. The per-blow press is capped so no single
        // exchange can swing you straight to the ground.
        let press = ((0.02 + 0.010 * remaining as f64) * if armed { 0.7 } else { 1.0 }).min(0.15)
            * self.beacon_press_factor();
        self.vitals.energy = (self.vitals.energy - press).max(0.0);
        self.status_msg = Some(format!(
            "You cut down one of {name} — {remaining} still standing. They press back. (energy {:.0}%){loot}",
            self.vitals.energy * 100.0
        ));
    }

    /// One stand-up exchange with a wild beast on the grid (#637): the player
    /// strikes, cutting the beast's toughness; a beast still standing strikes
    /// back, the harder the more danger it carries (energy — the collapse
    /// funnel, so a bear is a real risk). Down it and you take its hide and
    /// meat. Bump again to strike again. Sets the status line.
    pub(super) fn bump_attack_beast(&mut self, beast_id: &str) {
        let armed = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.get(crate::model::ItemType::Tool) > 0)
            .unwrap_or(false);
        let blow = if armed { 3 } else { 2 };
        let Some(idx) = self
            .sim
            .as_ref()
            .and_then(|sim| sim.beasts.iter().position(|b| b.id == beast_id))
        else {
            return;
        };
        let (species, hp) = {
            let b = &self.sim.as_ref().unwrap().beasts[idx];
            (b.species, b.hp)
        };
        let name = species.name();
        // The näšvyly's miasma (#455/#649): closing with the forest-fever shape
        // risks a wood-fever — the rot in the air gets into the body, deniable
        // (you were deep in cold wet forest, of course you took a fever).
        // Fortune leans whether it takes. Moved off the retired encounter screen
        // onto the grid creature you now strike.
        if species == crate::model::wildlife::WildSpecies::Nashvyly {
            let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
            let p = self.fortune.tilt_bad(0.35);
            let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x4EA5_7E11));
            if crate::rng::unit_from_hash(h) < p {
                self.afflict(
                    crate::model::Disease::Fever,
                    "The grey shape was upwind, and the rot in the air got into me. By dusk the \
                     fever has me — the wet wood and the cold, surely.",
                );
            }
        }
        // The uncanny that cannot be fought, only suffered (#455/#649): a strike
        // does not fell the spectral elk or the stir on the slope — closing with
        // them is the danger. The elk leads you astray and never tires; the
        // mountain's scree turns treacherous beneath you. Then they are gone —
        // old light through the trunks, the mountain settling, surely.
        use crate::model::wildlife::WildSpecies;
        if matches!(
            species,
            WildSpecies::SpectralElk | WildSpecies::MountainShade
        ) {
            let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
            if let Some(ref mut sim) = self.sim {
                sim.beasts.remove(idx);
            }
            match species {
                WildSpecies::SpectralElk => {
                    self.vitals.energy = (self.vitals.energy - 0.35).max(0.0);
                    self.vitals.hunger = (self.vitals.hunger - 0.15).max(0.0);
                    let astray = self.fortune.tilt_bad(0.30);
                    let h =
                        crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x735C_0FFE));
                    if crate::rng::unit_from_hash(h) < astray {
                        self.vitals.energy = 0.0;
                    }
                    if let Some(ref mut sim) = self.sim {
                        sim.log(
                            sim.world.tick,
                            crate::sim::journal::Voice::Scar,
                            "I followed the elk past where I meant to, and the trail simply \
                             stopped. Old light through the trunks, surely. I do not rightly \
                             know how I got back."
                                .into(),
                        );
                    }
                    self.status_msg = Some(
                        "You give chase, but the elk only draws you deeper and never tires. \
                         When you stop, it is gone. (energy spent)"
                            .into(),
                    );
                }
                _ => {
                    self.vitals.energy = (self.vitals.energy - 0.3).max(0.0);
                    self.vitals.hunger = (self.vitals.hunger - 0.1).max(0.0);
                    let turned = self.fortune.tilt_bad(0.25);
                    let h =
                        crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x5C2E_3311));
                    if crate::rng::unit_from_hash(h) < turned {
                        self.afflict(
                            crate::model::Disease::Sprain,
                            "The scree went out from under me on the shifting slope. A turned \
                             ankle — the mountain settling, surely.",
                        );
                    }
                    if let Some(ref mut sim) = self.sim {
                        sim.log(
                            sim.world.tick,
                            crate::sim::journal::Voice::Scar,
                            "The slope stirred, and I crossed beneath it on treacherous footing. \
                             It went still the moment I looked. The mountain does not say."
                                .into(),
                        );
                    }
                    self.status_msg = Some(
                        "You cross beneath the stirring slope on treacherous scree. It goes \
                         still the moment you look back."
                            .into(),
                    );
                }
            }
            return;
        }
        if blow >= hp {
            // Down. Take what it gives — the hunt's yield, where it has one.
            let (hide, meat) = species.hunt_yield();
            if let Some(ref mut ps) = self.player_start {
                if hide > 0 {
                    ps.inventory.add(crate::model::ItemType::Hide, hide);
                }
                if meat > 0 {
                    ps.inventory.add(crate::model::ItemType::Food, meat);
                }
            }
            if let Some(ref mut sim) = self.sim {
                sim.beasts.remove(idx);
            }
            // The belly's truth (#685): a big predator may have taken a person
            // before you took it — and only what a stomach cannot dissolve is
            // left, ruined by the acid and the days. No remains in a hare; none
            // in a beast that fed only on the wild; metal and stone alone
            // survive when there are any. Deterministic, fortune-leaned, rare.
            let mut belly = String::new();
            if species.danger() >= 2 && !species.uncanny() {
                let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
                let hh = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0xEA7E_6044));
                // ~12% that this predator had lately taken a person.
                if crate::rng::unit_from_hash(hh) < self.fortune.tilt_good(0.12) {
                    use crate::model::ItemType;
                    let coins = 1 + (hh % 6) as u32;
                    let mut got: Vec<String> =
                        vec![format!("{coins} coin{}", if coins == 1 { "" } else { "s" })];
                    if let Some(ref mut ps) = self.player_start {
                        ps.inventory.add(ItemType::Coin, coins);
                    }
                    // Sometimes a metal trinket, corroded near to ruin.
                    if crate::rng::unit_from_hash(crate::rng::mix_u64(hh ^ 0xC0DE)) < 0.4 {
                        if let Some(g) = crate::model::good_id("copper") {
                            if let Some(ref mut ps) = self.player_start {
                                ps.inventory.add_with_quality(ItemType::Good(g), 1, 0.1);
                            }
                            got.push("a corroded buckle".into());
                        }
                    }
                    belly = format!(
                        " In its belly, undissolved: {} — what the beast had eaten of someone, days gone.",
                        got.join(", ")
                    );
                }
            }
            self.status_msg = Some(if hide > 0 || meat > 0 {
                format!("You bring down the {name} (+{hide} hide, +{meat} meat).{belly}")
            } else {
                format!("You bring down the {name}.{belly}")
            });
            return;
        }
        // It lives, and fights: the press wears you by its danger.
        if let Some(ref mut sim) = self.sim {
            sim.beasts[idx].hp = hp - blow;
        }
        // The strange of the deep wild cuts deeper than its danger rating reads
        // (#637 slice 5): an uncanny thing presses harder than a like-rated
        // beast, so a march thick with dreads is its own kind of survival.
        let mut press = match species.danger() {
            0 => 0.02,
            1 => 0.06,
            _ => 0.13,
        };
        if species.uncanny() {
            press += 0.04;
        }
        let press = press * if armed { 0.8 } else { 1.0 } * self.beacon_press_factor();
        self.vitals.energy = (self.vitals.energy - press).max(0.0);
        // A wound is not only what it costs at the moment (#662): a beast that
        // presses can leave a festering hurt — a venomous bite turns to venom,
        // any other torn flesh to a dirty-ground infection. Fortune leans only
        // whether it takes; the uncanny leave no ordinary wound. Brought onto
        // grid combat from the retired encounter screen (#649). Deniable scar.
        if species.danger() >= 1 && !species.uncanny() {
            let venomous = species.venomous();
            let (disease, base, scar): (crate::model::Disease, f64, &str) = if venomous {
                (
                    crate::model::Disease::Venom,
                    0.45,
                    "The bite burns cold, then hot. Venom — it is in me now.",
                )
            } else {
                (
                    crate::model::Disease::Infection,
                    0.12,
                    "The wound will not close clean. By morning it is hot and angry — it has \
                     turned.",
                )
            };
            let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
            let p = self.fortune.tilt_bad(base);
            let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x00AF_F11C));
            if crate::rng::unit_from_hash(h) < p {
                self.afflict(disease, scar);
            }
        }
        self.status_msg = Some(format!(
            "You strike the {name} — it is still up and comes at you. (energy {:.0}%)",
            self.vitals.energy * 100.0
        ));
    }

    /// Greet a wanderer on the road (#649 slice 2): a traveller or pilgrim
    /// carries the road's news, a bard's song lifts a weary heart, a hermit
    /// offers a spare word. No menu — a word in passing, set on the status line
    /// and the journal. The player does not move onto the wanderer's tile.
    pub(super) fn greet_wayfarer(&mut self, kind: crate::sim::wayfarers::WayfarerKind) {
        use crate::sim::wayfarers::WayfarerKind;
        // A trader on the road lays out their barter — the trade panel, not a word.
        if let WayfarerKind::Trader(people) = kind {
            self.screen = Screen::RoadBarter { people };
            self.status_msg = Some(format!(
                "A {} trader stops to barter — they take no coin.",
                people.label()
            ));
            return;
        }
        let day = self.clock.day;
        let news = self.sim.as_ref().and_then(|sim| {
            crate::sim::rumors::informed_rumor(sim, day, self.seed ^ (day as u64).wrapping_mul(131))
        });
        let line = match kind {
            WayfarerKind::Traveler => news.unwrap_or_else(|| {
                "A traveller shares the road a while — quiet country ahead, they say.".into()
            }),
            WayfarerKind::Pilgrim => news.unwrap_or_else(|| {
                "A pilgrim bound for a far shrine gives you the day's blessing.".into()
            }),
            WayfarerKind::Bard => {
                // A song eases the road — a small comfort to a worn body.
                self.vitals.energy = (self.vitals.energy + 0.05).min(1.0);
                "A wandering bard plays you a tune — the road feels shorter for it.".into()
            }
            WayfarerKind::Hermit => news.unwrap_or_else(|| {
                "A hermit at the wild's edge spares you a few weathered words.".into()
            }),
            // Traders open the barter panel above and never reach this word.
            WayfarerKind::Trader(_) => return,
            // Someone in need — helping is a small kindness, the gods take note,
            // and the road remembers it in the journal.
            WayfarerKind::LostChild => {
                self.god_affinity.adjust(crate::model::GodName::Keuru, 0.03);
                "You walk a lost child back toward the nearest hearth. Keuru marks the mercy."
                    .into()
            }
            WayfarerKind::WinterSurvivor => {
                // A little food may keep them; give it if you can spare it.
                let gave = self
                    .player_start
                    .as_mut()
                    .map(|ps| ps.inventory.remove(crate::model::ItemType::Food, 1))
                    .unwrap_or(false);
                if gave {
                    self.god_affinity.adjust(crate::model::GodName::Keuru, 0.04);
                    "You share what food you can with a soul caught in the cold. They may yet live."
                        .into()
                } else {
                    "Someone is failing in the cold, and you have nothing to spare them. You walk on heavy.".into()
                }
            }
            WayfarerKind::FuneralProcession => {
                self.god_affinity.adjust(crate::model::GodName::Kukri, 0.03);
                "You stand aside and bow your head as the dead pass. Kukri keeps the reckoning."
                    .into()
            }
            WayfarerKind::EscapedLivestock => {
                // A hand turning strayed beasts — a grateful farmer spares a little.
                if let Some(ps) = self.player_start.as_mut() {
                    ps.inventory.add(crate::model::ItemType::Food, 1);
                }
                "You head off the strayed beasts and turn them home. The herder presses a little food on you in thanks.".into()
            }
            // The keeper at the marches' edge asks her price for the road past
            // (#455): bread or herb buys passage and her road-blessing; with
            // nothing to give, you take the long cold way round and it wears you.
            WayfarerKind::ThresholdKeeper => {
                let paid = self
                    .player_start
                    .as_mut()
                    .map(|ps| {
                        ps.inventory.remove(crate::model::ItemType::Food, 1)
                            || ps.inventory.remove(crate::model::ItemType::Herb, 1)
                    })
                    .unwrap_or(false);
                if paid {
                    self.god_affinity
                        .adjust(crate::model::GodName::Sampsa, 0.04);
                    if let Some(pos) = self.player_pos {
                        self.reveal_around(pos.region_idx, pos.px, pos.py);
                    }
                    "You lay down your toll — bread, or herb — and the keeper lets the road past open. The way ahead lies clear. A strange hermit, surely.".into()
                } else {
                    self.vitals.energy = (self.vitals.energy - 0.2).max(0.0);
                    self.vitals.hunger = (self.vitals.hunger - 0.1).max(0.0);
                    "You have nothing to lay down, and take the long cold way round the keeper's ground. Hours and strength lost. A hermit, surely.".into()
                }
            }
        };
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(tick, crate::sim::journal::Voice::Travel, line.clone());
        }
        self.status_msg = Some(line);
    }

    /// Meet a caravan on the road (#641 slice 2): what stepping into a member
    /// of the train does, by their part in it. The trader stops to deal — the
    /// merchant-caravan trade opens on the spot, no separate village needed. A
    /// guard waves you past; a drover or pack animal you simply give the road.
    /// The player does not move onto the tile in any case.
    pub(super) fn bump_caravan(&mut self, role: crate::sim::caravans::CaravanRole, raided: bool) {
        use crate::sim::caravans::CaravanRole;
        // A wrecked caravan has nothing to deal and no one to wave you on — only
        // the aftermath of the road's danger (#641 slice 4).
        if raided {
            self.status_msg = Some(
                "The caravan is a wreck — its goods carried off, nothing left to trade.".into(),
            );
            return;
        }
        match role {
            CaravanRole::Trader => {
                // Deal on the road, in place (#649): the caravan trader spares
                // you a little of what they carry for your news — the old
                // merchant-caravan trade, now resolved on the grid with no
                // cut-away to an encounter screen. A pair of herbs from the
                // packs, as the road has always gone.
                let herbs = 2u32;
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(crate::model::ItemType::Herb, herbs);
                }
                self.play_sound(crate::audio::SoundEvent::Trade);
                self.advance_clock_hour();
                self.status_msg = Some(format!(
                    "You fall in with the caravan a while and trade news for goods (+{herbs} herb). (1h)"
                ));
            }
            CaravanRole::Guard => {
                self.status_msg =
                    Some("A caravan guard waves you on — they want no trouble on the road.".into());
            }
            CaravanRole::Drover | CaravanRole::Pack => {
                self.status_msg = Some("You stand aside and let the caravan keep the road.".into());
            }
        }
    }

    /// Resolve a living-world relief task when the player has answered it
    /// (#613-epic): pays the task's reward into the player's standing in the town
    /// they stand in (having just relieved it), clears the task, records the
    /// quest, and notes it on the road. A no-op if no matching task stands.
    pub(super) fn complete_world_task(&mut self, kind_is_plague: bool, settlement: &str) {
        use crate::model::quest::QuestKind;
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let here_id = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                sim.world
                    .regions
                    .get(pos.region_idx)?
                    .settlements
                    .first()
                    .map(|s| s.id.clone())
            })
            .unwrap_or_default();
        let found = self.sim.as_ref().and_then(|sim| {
            sim.quests.iter().enumerate().find_map(|(i, q)| {
                let hit = match &q.kind {
                    QuestKind::RelievePlague { settlement: s } => kind_is_plague && s == settlement,
                    QuestKind::RelieveFamine { settlement: s } => {
                        !kind_is_plague && s == settlement
                    }
                    _ => false,
                };
                hit.then(|| (i, q.reward.clone()))
            })
        });
        let Some((idx, reward)) = found else {
            return;
        };
        if let (Some(ps), Some(sim)) = (self.player_start.as_mut(), self.sim.as_mut()) {
            crate::sim::quest_gen::apply_quest_reward(
                &reward,
                &mut ps.inventory,
                &mut sim.reputation,
                &player_id,
                &here_id,
                &mut sim.relationships,
                sim.world.tick,
            );
        }
        if let Some(ref mut sim) = self.sim {
            if idx < sim.quests.len() {
                sim.quests.remove(idx);
            }
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                "I answered the world's call. A town will remember it.".into(),
            );
        }
        self.milestones.record_quest_completed(self.clock.day);
    }

    /// Resolve a broker-truce task when the player has eased a deep rivalry
    /// (#614 slice 2): called by the broker act once the bad blood lifts out of
    /// rivalry. Matches the pair in either order, pays the reward into the
    /// player's current standing, clears the task, records the quest.
    pub(super) fn complete_truce_task(&mut self, x: &str, y: &str) {
        use crate::model::quest::QuestKind;
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let here_id = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                sim.world
                    .regions
                    .get(pos.region_idx)?
                    .settlements
                    .first()
                    .map(|s| s.id.clone())
            })
            .unwrap_or_default();
        let found = self.sim.as_ref().and_then(|sim| {
            sim.quests.iter().enumerate().find_map(|(i, q)| {
                let hit = match &q.kind {
                    QuestKind::BrokerTruce { a, b } => (a == x && b == y) || (a == y && b == x),
                    _ => false,
                };
                hit.then(|| (i, q.reward.clone()))
            })
        });
        let Some((idx, reward)) = found else {
            return;
        };
        if let (Some(ps), Some(sim)) = (self.player_start.as_mut(), self.sim.as_mut()) {
            crate::sim::quest_gen::apply_quest_reward(
                &reward,
                &mut ps.inventory,
                &mut sim.reputation,
                &player_id,
                &here_id,
                &mut sim.relationships,
                sim.world.tick,
            );
        }
        if let Some(ref mut sim) = self.sim {
            if idx < sim.quests.len() {
                sim.quests.remove(idx);
            }
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                "The peace I carried between them holds. They will not forget it.".into(),
            );
        }
        self.milestones.record_quest_completed(self.clock.day);
    }

    /// Resolve a steady-faith task when the player has made an offering in a
    /// contested town (#614 slice 3): the act of devotion answers the call. Pays
    /// the reward into the player's standing there, clears the task, records it.
    /// A no-op if no matching task stands for the town.
    pub(super) fn complete_faith_task(&mut self, settlement: &str) {
        use crate::model::quest::QuestKind;
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let here_id = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                sim.world
                    .regions
                    .get(pos.region_idx)?
                    .settlements
                    .first()
                    .map(|s| s.id.clone())
            })
            .unwrap_or_default();
        let found = self.sim.as_ref().and_then(|sim| {
            sim.quests.iter().enumerate().find_map(|(i, q)| {
                let hit =
                    matches!(&q.kind, QuestKind::SteadyFaith { settlement: s } if s == settlement);
                hit.then(|| (i, q.reward.clone()))
            })
        });
        let Some((idx, reward)) = found else {
            return;
        };
        if let (Some(ps), Some(sim)) = (self.player_start.as_mut(), self.sim.as_mut()) {
            crate::sim::quest_gen::apply_quest_reward(
                &reward,
                &mut ps.inventory,
                &mut sim.reputation,
                &player_id,
                &here_id,
                &mut sim.relationships,
                sim.world.tick,
            );
        }
        if let Some(ref mut sim) = self.sim {
            if idx < sim.quests.len() {
                sim.quests.remove(idx);
            }
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                "My offering steadied a town whose faith was tearing. They will remember it."
                    .into(),
            );
        }
        self.milestones.record_quest_completed(self.clock.day);
    }

    /// Resolve a supply-goods task when the player has provisioned a town the
    /// living economy left goods-starved (#614 slice 4): the act of supplying it
    /// answers the call. Pays the reward into local standing, clears the task,
    /// records it. A no-op if no matching task stands for the town.
    pub(super) fn complete_supply_task(&mut self, settlement: &str) {
        use crate::model::quest::QuestKind;
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let here_id = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                sim.world
                    .regions
                    .get(pos.region_idx)?
                    .settlements
                    .first()
                    .map(|s| s.id.clone())
            })
            .unwrap_or_default();
        let found = self.sim.as_ref().and_then(|sim| {
            sim.quests.iter().enumerate().find_map(|(i, q)| {
                let hit =
                    matches!(&q.kind, QuestKind::SupplyGoods { settlement: s } if s == settlement);
                hit.then(|| (i, q.reward.clone()))
            })
        });
        let Some((idx, reward)) = found else {
            return;
        };
        if let (Some(ps), Some(sim)) = (self.player_start.as_mut(), self.sim.as_mut()) {
            crate::sim::quest_gen::apply_quest_reward(
                &reward,
                &mut ps.inventory,
                &mut sim.reputation,
                &player_id,
                &here_id,
                &mut sim.relationships,
                sim.world.tick,
            );
        }
        if let Some(ref mut sim) = self.sim {
            if idx < sim.quests.len() {
                sim.quests.remove(idx);
            }
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                "I carried trade back to a town the roads had forgotten. It will thrive again."
                    .into(),
            );
        }
        self.milestones.record_quest_completed(self.clock.day);
    }

    pub(super) fn check_quests_on_tick(&mut self) {
        self.generate_world_task_quests();
        self.generate_band_bounties();
        let current_day = self.clock.day;
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);

        let inventory = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.clone())
            .unwrap_or_default();
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let current_settlement_id = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                let region = sim.world.regions.get(pos.region_idx)?;
                region.settlements.first().map(|s| s.id.clone())
            })
            .unwrap_or_default();
        let local_rep = self
            .sim
            .as_ref()
            .and_then(|sim| {
                sim.reputation
                    .get_entry(&player_id, &current_settlement_id)
                    .map(|e| e.reputation.local)
            })
            .unwrap_or(0.5);
        let aided_npcs = self
            .sim
            .as_ref()
            .map(|s| s.aided_npcs.clone())
            .unwrap_or_default();

        if self.sim.is_none() {
            return;
        }

        // Inputs for the newer quest kinds: discoveries seen, dealings had,
        // and where the player has raised structures.
        let (observed, dealings, structure_regions) = {
            let sim = self.sim.as_ref().unwrap();
            let observed = sim
                .discoveries
                .entries
                .iter()
                .filter(|d| d.observed)
                .count() as u32;
            let dealings: u32 = sim.npc_memories.values().map(|m| m.count() as u32).sum();
            let mut regions: Vec<usize> = sim
                .world
                .regions
                .iter()
                .enumerate()
                .filter(|(_, r)| r.structures.iter().any(|st| !st.is_npc_built))
                .map(|(i, _)| i)
                .collect();
            regions.dedup();
            (observed, dealings, regions)
        };

        let result = crate::sim::quest_gen::check_quests(
            &mut self.sim.as_mut().unwrap().quests,
            &crate::sim::quest_gen::QuestContext {
                inventory: &inventory,
                current_region_idx: region_idx,
                aided_npcs: &aided_npcs,
                local_reputation: local_rep,
                current_day,
                observed_discoveries: observed,
                dealings,
                player_structure_regions: &structure_regions,
            },
        );

        if result.completed.is_empty() && result.expired.is_empty() {
            return;
        }

        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);

        let completed_rewards: Vec<crate::model::quest::QuestReward> = result
            .completed
            .iter()
            .filter_map(|&idx| self.sim.as_ref()?.quests.get(idx).map(|q| q.reward.clone()))
            .collect();

        // FetchItem quests are deliveries ("needed within the walls") — the
        // goods are handed over on completion. Collect what to consume before
        // the quests are removed below; without this you keep the items and the
        // same stack can satisfy repeated fetch quests (reward farming).
        let fetch_deliveries: Vec<(ItemType, u32)> = result
            .completed
            .iter()
            .filter_map(|&idx| match &self.sim.as_ref()?.quests.get(idx)?.kind {
                crate::model::quest::QuestKind::FetchItem { item, count } => Some((*item, *count)),
                crate::model::quest::QuestKind::DeliverTo { item, count, .. } => {
                    Some((*item, *count))
                }
                _ => None,
            })
            .collect();

        if let Some(ref mut sim) = self.sim {
            for _ in &result.completed {
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Travel,
                    "I did what was asked. The world shifts, just a little.".into(),
                );
            }
            for _ in &result.expired {
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Travel,
                    "What was asked of me fades. The moment has passed.".into(),
                );
            }

            let mut to_remove: Vec<usize> = result.completed;
            to_remove.extend(&result.expired);
            to_remove.sort_unstable();
            to_remove.dedup();
            to_remove.sort_unstable_by(|a, b| b.cmp(a));
            for &idx in &to_remove {
                if idx < sim.quests.len() {
                    sim.quests.remove(idx);
                }
            }
        }

        // Hand over the delivered goods.
        if let Some(ref mut ps) = self.player_start {
            for (item, count) in &fetch_deliveries {
                ps.inventory.remove(*item, *count);
            }
        }

        for reward in completed_rewards {
            if let Some(ref mut ps) = self.player_start {
                if let Some(ref mut sim) = self.sim {
                    crate::sim::quest_gen::apply_quest_reward(
                        &reward,
                        &mut ps.inventory,
                        &mut sim.reputation,
                        &player_id,
                        &current_settlement_id,
                        &mut sim.relationships,
                        sim.world.tick,
                    );
                }
            }
        }

        // Keep the quest board alive: when active quests run low, the world posts
        // new needs. Salted by world tick (not day): a day-only salt regenerated
        // the IDENTICAL quest batch after completing one within the same day,
        // letting the same fetch quest be re-completed for repeat rewards.
        let player_people = self.inter_people_bias.player_people;
        let day = self.clock.day;
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let salt = self
            .seed
            .wrapping_add(tick.wrapping_mul(1009))
            .wrapping_add(7);
        if let Some(ref mut sim) = self.sim {
            // Count only ordinary quests: the living-world tasks (plague, famine,
            // supply, ...) are a separate board layer and are common enough that
            // counting them here would keep the board "full" and freeze the
            // ordinary quest stream (#614 slice 4).
            let regular = sim
                .quests
                .iter()
                .filter(|q| !q.kind.is_world_task())
                .count();
            if regular < 2 {
                let mut fresh = crate::sim::quest_gen::generate_quests(
                    salt,
                    player_people,
                    &sim.world.regions,
                    day,
                );
                let seen = sim
                    .discoveries
                    .entries
                    .iter()
                    .filter(|d| d.observed)
                    .count() as u32;
                for q in fresh.iter_mut() {
                    if let crate::model::quest::QuestKind::VisitDiscovery { baseline } = &mut q.kind
                    {
                        if *baseline == u32::MAX {
                            *baseline = seen;
                        }
                    }
                }
                sim.quests.extend(fresh);
            }
        }
    }

    pub(super) fn check_quests_on_travel(&mut self, region_idx: usize) {
        let _ = region_idx;
        self.check_quests_on_tick();
    }

    pub(super) fn check_quests_on_gather(&mut self) {
        self.check_quests_on_tick();
    }

    pub(super) fn check_quests_on_aid(&mut self, npc_id: &str) {
        if let Some(ref mut sim) = self.sim {
            if !sim.aided_npcs.contains(&npc_id.to_string()) {
                sim.aided_npcs.push(npc_id.to_string());
            }
        }
        self.check_quests_on_tick();
    }

    pub fn advance_clock_hour(&mut self) {
        self.advance_clock(1);
    }

    /// A full night's rest (8h). Kept for the legacy/default path.
    pub fn rest(&mut self) {
        self.rest_hours(8);
    }

    /// Rest for a chosen number of hours (a short spurt up to a full night),
    /// clamped to [1, MAX_REST_HOURS]. Effects and risk scale with the duration;
    /// quality still depends on where you rest (settlement, shelter, the cold).
    pub fn rest_hours(&mut self, hours: u32) {
        use crate::sim::rest::{tile_rest_quality, RestQuality};

        let hours = hours.clamp(1, MAX_REST_HOURS);
        let h = hours as f64;
        let tod = crate::model::TimeOfDay::from_hour(self.clock.hour);
        let was_deep_night = tod == crate::model::TimeOfDay::DeepNight;
        let on_settlement = self.player_on_settlement().is_some();
        // A structure on this tile raises the rest tier — your own walls count.
        // Tier drives stamina rate, encounter risk, and journal flavor; the
        // tile_rest_quality shelter flags were previously hardcoded false, so a
        // built Home still rested like open ground (and deep night forced
        // "out in the cold" even inside it).
        let structure_tier = self.structure_at_player().map(|s| match s.kind {
            crate::sim::structures::BuildKind::Tarp => RestQuality::Campfire,
            crate::sim::structures::BuildKind::LeanTo
            | crate::sim::structures::BuildKind::TarpTent => RestQuality::LeanTo,
            crate::sim::structures::BuildKind::Laavu | crate::sim::structures::BuildKind::Kota => {
                RestQuality::SettlementFloor
            }
            crate::sim::structures::BuildKind::Cabin
            | crate::sim::structures::BuildKind::Longhouse
            | crate::sim::structures::BuildKind::Home => RestQuality::Inn,
            // A shrine, a trail, a bridge, a well, a cairn, a fence line:
            // none of them is shelter.
            crate::sim::structures::BuildKind::Shrine
            | crate::sim::structures::BuildKind::Trail
            | crate::sim::structures::BuildKind::Footbridge
            | crate::sim::structures::BuildKind::Well
            | crate::sim::structures::BuildKind::Waymarker
            | crate::sim::structures::BuildKind::Palisade
            | crate::sim::structures::BuildKind::Beacon => RestQuality::Campfire,
        });
        // A hearth is the warmest rest in the world: a roof, walls, and a fire.
        // Resting on it is an inn-grade night even in the deep cold (#458).
        let on_hearth = self.player_pos.and_then(|p| {
            self.sim
                .as_ref()
                .and_then(|s| s.world.regions.get(p.region_idx))
                .and_then(|r| r.terrain.get(p.px, p.py))
        }) == Some(crate::model::Terrain::Hearth);
        let sheltered = structure_tier.is_some() || on_settlement || on_hearth;
        let base_quality = if was_deep_night && !sheltered {
            RestQuality::OutInCold
        } else {
            tile_rest_quality(on_settlement, false, false, false)
        };
        let mut quality = match structure_tier {
            Some(t) if t.stamina_per_hour() > base_quality.stamina_per_hour() => t,
            _ => base_quality,
        };
        if on_hearth && RestQuality::Inn.stamina_per_hour() > quality.stamina_per_hour() {
            quality = RestQuality::Inn;
        }
        // Keuru's vow: welcomed at every hearth, the rest restores the deeper.
        let stamina_gain = quality.stamina_per_hour() * h * self.vow_rest_bonus();
        let morale_gain = quality.morale_per_hour() * h;
        let mut encounter_risk = quality.encounter_risk_per_hour() * h;
        // A palisade line within two tiles quiets the night.
        if self.own_structure_near(crate::sim::structures::BuildKind::Palisade, 2) {
            encounter_risk *= 0.5;
        }

        let quality_label = crate::sim::journal::rest_quality_label(
            on_settlement,
            quality == RestQuality::Inn,
            false,
            false,
        );

        self.advance_clock(hours);
        self.vitals.rest(hours);
        let mut scouted = false;
        if let Some(ref mut ps) = self.player_start {
            for companion in &mut ps.companions {
                companion.rest(h / 8.0);
                let action_seed = self
                    .seed
                    .wrapping_add((self.clock.day as u64 * 24 + self.clock.hour as u64) * 137);
                if let Some(action) = companion.autonomous_action(action_seed) {
                    // The flavor promises a yield — deliver it to the player.
                    match action {
                        crate::model::CompanionAction::Hunt => {
                            ps.inventory.add(crate::model::ItemType::Food, 1)
                        }
                        crate::model::CompanionAction::Gather => {
                            ps.inventory.add(crate::model::ItemType::Herb, 1)
                        }
                        crate::model::CompanionAction::Scout => scouted = true,
                    }
                    let flavor = companion.apply_action(action);
                    self.status_msg = Some(format!("{} {}", companion.name, flavor));
                }
                // Companion eats from shared inventory when hungry
                if companion.food_need > 50.0 && ps.inventory.has(crate::model::ItemType::Food) {
                    ps.inventory.remove(crate::model::ItemType::Food, 1);
                    companion.feed(0.5);
                }
                // A goat is a walking larder: a proper rest stop yields milk.
                // milk_production existed on Animal but was never collected.
                let milk = companion.animal.milk_production();
                if milk > 0
                    && hours >= 4
                    && companion.mood() != crate::model::CompanionMood::Unhappy
                {
                    ps.inventory.add(crate::model::ItemType::Food, milk);
                }
            }
        }
        // A scouting companion reveals the ground around the player.
        if scouted {
            if let Some(pos) = self.player_pos {
                self.reveal_around(pos.region_idx, pos.px, pos.py);
            }
        }
        // Rest is when wounds get tended and snares get checked.
        let fortune = self.fortune;
        let snare_seed = self.seed;
        let snare_tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        if let Some(ref mut ps) = self.player_start {
            // Tending an illness with a bandage eases it and shortens its course.
            if !ps.person.illnesses.is_empty() && ps.inventory.remove(ItemType::Bandage, 1) {
                for d in ps.person.illnesses.iter_mut() {
                    d.tend();
                }
                self.status_msg = Some("You dress your sickness with a bandage.".into());
            }
            // A salve answers what a bandage can't: infection and venom run
            // their course faster under a poultice (#414, the luck-body loop).
            let has_treatable = ps.person.illnesses.iter().any(|d| {
                matches!(
                    d.disease,
                    crate::model::Disease::Infection | crate::model::Disease::Venom
                )
            });
            if has_treatable && ps.inventory.remove(ItemType::Salve, 1) {
                for d in ps.person.illnesses.iter_mut() {
                    if matches!(
                        d.disease,
                        crate::model::Disease::Infection | crate::model::Disease::Venom
                    ) {
                        d.tend_strong();
                    }
                }
                self.status_msg = Some("You work a salve into the wound — it answers.".into());
            }
            // A set trap yields food over a proper rest in the wild — if the
            // land still carries game. Trapping draws the valley down.
            let richness = self
                .player_pos
                .and_then(|pos| {
                    self.sim
                        .as_ref()
                        .and_then(|s| s.world.regions.get(pos.region_idx))
                })
                .map(|r| r.game_richness)
                .unwrap_or(1.0);
            if hours >= 4
                && !on_settlement
                && richness > 0.3
                && ps.inventory.has(ItemType::Trap)
                && !ps.inventory.is_broken(ItemType::Trap)
            {
                ps.inventory.add(ItemType::Food, 1);
                // Small game caught whole carries a hide too — sometimes. Luck
                // leans whether the snare takes something worth skinning.
                let luck = crate::rng::unit_from_hash(crate::rng::mix_u64(
                    snare_seed ^ crate::rng::mix_u64(snare_tick ^ 0x5AE5_70AD),
                ));
                let took_hide = luck < fortune.tilt_good(0.40);
                if took_hide {
                    ps.inventory.add(ItemType::Hide, 1);
                }
                ps.inventory.use_tool(ItemType::Trap);
                self.status_msg = Some(if took_hide {
                    "The snare took small game — meat and a hide.".into()
                } else {
                    "The snare took small game — a little meat.".into()
                });
                if let Some(pos) = self.player_pos {
                    if let Some(region) = self
                        .sim
                        .as_mut()
                        .and_then(|s| s.world.regions.get_mut(pos.region_idx))
                    {
                        region.game_richness = (region.game_richness - 0.02).max(0.0);
                    }
                }
            } else if hours >= 4
                && !on_settlement
                && richness <= 0.3
                && ps.inventory.has(ItemType::Trap)
            {
                self.status_msg = Some("The snare sits empty. This land is trapped out.".into());
            }
        }
        let structure_bonus = match self.structure_at_player() {
            Some(s) => match s.kind {
                crate::sim::structures::BuildKind::Tarp => 0.05,
                crate::sim::structures::BuildKind::LeanTo => 0.10,
                crate::sim::structures::BuildKind::TarpTent => 0.15,
                crate::sim::structures::BuildKind::Laavu => 0.20,
                crate::sim::structures::BuildKind::Kota => 0.30,
                crate::sim::structures::BuildKind::Cabin => 0.45,
                crate::sim::structures::BuildKind::Longhouse => 0.60,
                crate::sim::structures::BuildKind::Home => 0.80,
                crate::sim::structures::BuildKind::Shrine
                | crate::sim::structures::BuildKind::Trail
                | crate::sim::structures::BuildKind::Footbridge
                | crate::sim::structures::BuildKind::Well
                | crate::sim::structures::BuildKind::Waymarker
                | crate::sim::structures::BuildKind::Palisade
                | crate::sim::structures::BuildKind::Beacon => 0.0,
            },
            None => 0.0,
        };
        let dur_frac = h / 8.0;
        if structure_bonus > 0.0 {
            self.vitals.energy = (self.vitals.energy + structure_bonus * dur_frac).min(1.0);
            self.vitals.hunger = (self.vitals.hunger - structure_bonus * 0.3 * dur_frac).max(0.0);
        }
        self.vitals.energy = (self.vitals.energy + stamina_gain / 8.0).min(1.0);
        self.god_affinity
            .adjust(GodName::Kukri, 0.02 + morale_gain * 0.1);
        // A well within a tile slakes the rest for free — water where the
        // land gives none.
        if self.own_structure_near(crate::sim::structures::BuildKind::Well, 1) {
            self.vitals.thirst = (self.vitals.thirst + 0.5 * dur_frac).min(1.0);
        }
        // A shared roof rests better: at one's own Cabin+ with a living
        // marriage, the night gives a little more back.
        if self.spouse_id.is_some() && self.own_hearth_here() {
            self.vitals.energy = (self.vitals.energy + 0.05 * dur_frac).min(1.0);
        }
        // Rest beside your own shrine: a slow, small pull toward the god it
        // was raised to. Devotional practice, not a summons — the gods are
        // withdrawn, and a new shrine changes the rester, not the world.
        if let Some(god) = self.own_shrine_god_near() {
            self.god_affinity.adjust(god, 0.01 * dur_frac);
            let t = self.sim.as_ref().map_or(0, |s| s.world.tick);
            let mut prng =
                crate::rng::SeedRng::new(self.seed).fork_for(&format!("shrine-pilgrims-{}", t));
            // Pilgrims are ordinary people on ordinary roads; sometimes they
            // pass, nod, and walk on.
            if prng.gen_range(6) == 0 {
                if let Some(ref mut sim) = self.sim {
                    sim.log(
                        t,
                        crate::sim::journal::Voice::Encounter,
                        format!(
                            "Pilgrims in road-grey stopped at the {} shrine while I rested. \
                             They left a ribbon and walked on.",
                            god.label()
                        ),
                    );
                }
            }
        }
        if quality == RestQuality::Inn {
            self.vitals.energy = (self.vitals.energy + 0.05 * dur_frac).min(1.0);
        }

        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        if let Some(ref mut sim) = self.sim {
            sim.log_journal(tick, quality.journal_flavor().to_string());
            let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                .fork_for(&format!("rest-journal-{}", sim.world.tick));
            let text = crate::sim::journal::rest_text(&mut rng, quality_label);
            sim.log(sim.world.tick, crate::sim::journal::Voice::Rest, text);
        }

        if encounter_risk > 0.0 {
            let roll = {
                let mut rng = crate::rng::SeedRng::new(self.seed.wrapping_add(tick));
                rng.gen_f64()
            };
            if roll < encounter_risk {
                self.status_msg = Some(format!("Restless night. {}", quality.journal_flavor()));
            } else {
                self.status_msg = Some(quality.journal_flavor().to_string());
            }
        } else {
            self.status_msg = Some(quality.journal_flavor().to_string());
        }

        // God-prayer mini-encounter: a quiet dream from the patron of the
        // land we rest in. Logged to the journal so the world remembers.
        let dominant = self.sim.as_ref().and_then(|sim| {
            let pos = self.player_pos?;
            let region = sim.world.regions.get(pos.region_idx)?;
            let settlement = region.settlements.first()?;
            Some(settlement.people.first()?.people.clone())
        });
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_else(|| "player".to_string());
        if let Some(line) = crate::sim::god::maybe_prayer(&player_id, dominant.as_deref(), tick) {
            if let Some(ref mut sim) = self.sim {
                sim.log_journal(tick, line);
            }
        }

        // Dream journal entry when Kukri affinity is high
        if self.god_affinity.get(GodName::Kukri) > 0.5 {
            if let Some(ref mut sim) = self.sim {
                let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                    .fork_for(&format!("dream-journal-{}", sim.world.tick));
                let text = crate::sim::journal::dream_text(&mut rng);
                sim.log(sim.world.tick, crate::sim::journal::Voice::Dream, text);
            }
        }

        self.fire_hint(hints::HINT_FIRST_REST);
        self.play_sound(crate::audio::SoundEvent::Ambient);
    }

    pub fn clock_str(&self) -> String {
        let tod = self.clock.time_of_day();
        let season = self.clock.season();
        format!(
            "D{} {:02}:00 {} {}",
            self.clock.day,
            self.clock.hour,
            tod.glyph(),
            season.glyph()
        )
    }
}
