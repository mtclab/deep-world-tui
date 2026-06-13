use crate::model::{GodName, ItemType, TensionEvent, Terrain, Weather};
use crate::sim::hints;

use super::*;

impl App {
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

    pub fn advance_clock(&mut self, hours: u32) {
        let day_before = self.clock.day;
        let season = self.clock.season();
        self.clock.advance(hours);
        // Harsh weather wears the body down faster too (need_decay_modifier
        // was defined per-weather but never applied to the player). The life's
        // hidden star leans the harshness — only the penalty over fair weather,
        // so a blessed soul bears the cold a little better and the cursed a
        // little worse; clear skies fall on everyone the same. A worn coat
        // halves the harsh-weather penalty (#414) — the hunt-to-warmth loop.
        let has_coat = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.has(ItemType::Coat) && !ps.inventory.is_broken(ItemType::Coat))
            .unwrap_or(false);
        let coat_factor = if has_coat { 0.5 } else { 1.0 };
        let mut weather_harsh = false;
        let weather_mult = self
            .player_pos
            .map(|pos| {
                let raw = self.region_weather(pos.region_idx).need_decay_modifier();
                let harsh_excess = (raw - 1.0).max(0.0);
                weather_harsh = harsh_excess > 0.0;
                1.0 + harsh_excess * coat_factor * self.fortune.bad_multiplier()
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
            // The coat earns its keep by wearing out: harsh weather frays it.
            if has_coat && weather_harsh {
                ps.inventory.decay(ItemType::Coat, 0.02 * hours as f64);
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

    pub(super) fn check_quests_on_tick(&mut self) {
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
            if sim.quests.len() < 2 {
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
        let sheltered = structure_tier.is_some() || on_settlement;
        let base_quality = if was_deep_night && !sheltered {
            RestQuality::OutInCold
        } else {
            tile_rest_quality(on_settlement, false, false, false)
        };
        let quality = match structure_tier {
            Some(t) if t.stamina_per_hour() > base_quality.stamina_per_hour() => t,
            _ => base_quality,
        };
        let stamina_gain = quality.stamina_per_hour() * h;
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
