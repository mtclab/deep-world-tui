use crate::model::{GodName, ItemType, PeopleKind};
use crate::sim::hints;

use super::*;

impl App {
    pub fn enter_market(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Market {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_market(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    /// Supply effect of arrived caravans on the current settlement's price for
    /// an item (more goods in town → lower price). 1.0 when no caravan applies.
    fn caravan_price_modifier(&self, item: ItemType) -> f64 {
        let Some(name) = self.current_settlement().map(|s| s.name.clone()) else {
            return 1.0;
        };
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let mut m = 1.0;
        if let Some(ref sim) = self.sim {
            for c in &sim.caravans {
                if c.destination == name && c.has_arrived(tick) {
                    m *= c.price_modifier(item, tick);
                }
            }
        }
        m
    }

    /// Escalation: at deep hostility the market simply closes to the player.
    fn market_barred(&self) -> bool {
        // Two seasons of unpaid hearth-tax and the polity shuts the stalls to
        // you (#396); a deep grudge does the same.
        if self.tax_unpaid_seasons >= 2 {
            return true;
        }
        self.current_settlement_people()
            .map(|p| self.inter_people_bias.effective_bias(p) < -0.25)
            .unwrap_or(false)
    }

    /// The people of an enclave the player is currently trading at, if any —
    /// the Five take no coin, so their floor barters in kind (#454).
    fn current_enclave_people(&self) -> Option<crate::model::PeopleKind> {
        self.current_settlement().and_then(|s| s.enclave_people())
    }

    pub fn buy_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot buy that".into());
            return;
        }
        if self.market_barred() {
            self.status_msg = Some("The market is closed to your kind here.".into());
            return;
        }
        if let Some(pk) = self.current_enclave_people() {
            self.status_msg = Some(format!(
                "The {} take no coin. Lay down a good and they trade in kind.",
                pk.label()
            ));
            return;
        }
        // Single source of truth with the displayed quote.
        let price = self.quote_buy_price(item);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(ItemType::Coin, price) {
                ps.inventory.add(item, 1);
                self.advance_clock_hour();
                self.fire_hint(hints::HINT_FIRST_TRADE);
                self.check_quests_on_tick();
                self.status_msg =
                    Some(format!("Bought 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Masa, 0.02);
                // Honest trade slowly mends what tension breaks.
                if let Some(np) = self.current_settlement_people() {
                    self.inter_people_bias.mod_toward(np, 0.005);
                }
                self.charge_gift_for_trade();
            } else {
                self.status_msg = Some(format!("Need {} coins", price));
            }
        }
    }

    pub fn sell_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot sell that".into());
            return;
        }
        if self.market_barred() {
            self.status_msg = Some("The market is closed to your kind here.".into());
            return;
        }
        // At an enclave there is no coin: laying down a good is bartering it for
        // theirs, at the people's own fixed rate (#454).
        if let Some(pk) = self.current_enclave_people() {
            self.barter_at_enclave(pk, item);
            return;
        }
        // Single source of truth with the displayed quote (spread-clamped so
        // selling never pays more than buying costs).
        let price = self.quote_sell_price(item);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(item, 1) {
                ps.inventory.add(ItemType::Coin, price);
                self.advance_clock_hour();
                self.fire_hint(hints::HINT_FIRST_TRADE);
                self.check_quests_on_tick();
                self.status_msg = Some(format!("Sold 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Masa, 0.01);
                if let Some(np) = self.current_settlement_people() {
                    self.inter_people_bias.mod_toward(np, 0.005);
                }
                self.charge_gift_for_trade();
            } else {
                self.status_msg = Some(format!("No {} to sell", item.name()));
            }
        }
    }

    /// Barter a good at an enclave of the Five (#454): no coin changes hands —
    /// you lay down a good and take theirs at a fixed rate. If they want none of
    /// what you offer, they say so and keep their own.
    fn barter_at_enclave(&mut self, people: crate::model::PeopleKind, offered: ItemType) {
        let Some((cost, gives)) = crate::model::enclave_barter(people, offered) else {
            self.status_msg = Some(format!(
                "The {} want none of your {} — and they will not take coin.",
                people.label(),
                offered.name()
            ));
            return;
        };
        let Some(ref mut ps) = self.player_start else {
            return;
        };
        if ps.inventory.get(offered) < cost {
            self.status_msg = Some(format!(
                "The {} ask {} {} for that trade.",
                people.label(),
                cost,
                offered.name()
            ));
            return;
        }
        ps.inventory.remove(offered, cost);
        let mut got: Vec<String> = Vec::new();
        for (item, qty) in &gives {
            ps.inventory.add(*item, *qty);
            got.push(format!("{} {}", qty, item.name()));
        }
        self.play_sound(crate::audio::SoundEvent::Trade);
        self.advance_clock_hour();
        self.check_quests_on_tick();
        if let Some(np) = self.current_settlement_people() {
            self.inter_people_bias.mod_toward(np, 0.005);
        }
        self.status_msg = Some(format!(
            "The {} take your {} and lay out {} in fair measure. No coin, no haggle. (1h)",
            people.label(),
            offered.name(),
            got.join(", ")
        ));
    }

    /// Try to palm an item off a market stall. The witness roll decides:
    /// unseen takes it clean, rumored takes it with a whisper attached, seen
    /// gets nothing and a name for thieving. Crime is a choice with weight.
    pub fn steal_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot steal that".into());
            return;
        }
        if self.current_settlement().is_none() {
            self.status_msg = Some("Nothing here to steal.".into());
            return;
        }
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let mut rng = crate::rng::SeedRng::new(self.seed.wrapping_add(tick)).fork_for("steal");
        let roll = rng.gen_range(100);
        let pid = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let sid = self
            .current_settlement()
            .map(|s| s.id.clone())
            .unwrap_or_default();
        let npc_people = self.current_settlement_people();
        // More eyes in a city: witness odds scale with the place. A hamlet's
        // lanes are empty half the day; a city street is never unwatched.
        let (clean_under, whisper_under) = match self
            .current_settlement()
            .map(|s| s.size.clone())
            .unwrap_or_default()
            .as_str()
        {
            "hamlet" => (60, 85),
            "town" => (40, 70),
            "city" => (25, 60),
            _ => (50, 80), // village — the old odds
        };
        self.advance_clock_hour();
        if roll < clean_under {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, 1);
            }
            self.god_affinity.adjust(GodName::Masa, -0.03);
            self.status_msg = Some(format!("No one saw. The {} is yours.", item.name()));
        } else if roll < whisper_under {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, 1);
            }
            self.god_affinity.adjust(GodName::Masa, -0.05);
            if let Some(ref mut sim) = self.sim {
                sim.reputation.adjust_local(&pid, &sid, -0.05);
            }
            self.status_msg = Some(format!(
                "The {} is yours — but someone is whispering already.",
                item.name()
            ));
        } else {
            self.god_affinity.adjust(GodName::Masa, -0.05);
            if let (Some(ref mut sim), Some(pos)) = (&mut self.sim, self.player_pos) {
                sim.reputation.adjust_local(&pid, &sid, -0.15);
                // Word of a theft runs the roads ahead of you (#8 crime): the
                // other settlements of the region hear of it and your welcome
                // cools across them — a reputation that follows, mended the
                // same slow way (gifts, time, the temple's penance).
                let others: Vec<String> = sim
                    .world
                    .regions
                    .get(pos.region_idx)
                    .map(|r| {
                        r.settlements
                            .iter()
                            .map(|s| s.id.clone())
                            .filter(|id| *id != sid)
                            .collect()
                    })
                    .unwrap_or_default();
                for oid in &others {
                    sim.reputation.adjust_local(&pid, oid, -0.06);
                }
                if !others.is_empty() {
                    let tick = sim.world.tick;
                    sim.log(
                        tick,
                        crate::sim::journal::Voice::Rumor,
                        "Word of the theft will run the roads ahead of me. I am not welcome in these parts now.".into(),
                    );
                }
            }
            if let Some(np) = npc_people {
                self.inter_people_bias.mod_toward(np, -0.03);
            }
            self.status_msg =
                Some("Caught with your hand out. Word of this will travel the region.".into());
        }
    }

    /// Hire on for a day's labour at the settlement you stand in (#526
    /// livelihoods): the meagre post-Fall wage, paid in coin, for the work the
    /// place actually needs — fields and stores when the larder runs lean, plain
    /// labour otherwise. Costs the day and the body; an Oltzed vow lightens it.
    /// A people set hard against you takes no day-hand. Earns a little standing,
    /// and a lean town's stores rise by the hands you lent.
    /// The trade good a settlement most wants — the tracked good it holds least,
    /// relative to what it can keep (#526/#540). `None` off a settlement.
    pub fn settlement_shortfall(&self) -> Option<ItemType> {
        let s = self.current_settlement()?;
        let cap = (s.population as f64 * 0.5).max(1.0);
        [
            ItemType::Iron,
            ItemType::Tool,
            ItemType::Cloth,
            ItemType::Wood,
        ]
        .into_iter()
        .min_by(|a, b| {
            (s.good(*a) / cap)
                .partial_cmp(&(s.good(*b) / cap))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Provision a settlement with a good it is short of (#526 livelihoods, into
    /// the living economy #540): carry in what the town lacks and they pay a
    /// fair premium for it and remember the hand that brought it. The town's own
    /// stock rises by what you delivered — you have actually supplied it, not
    /// just turned a coin. The most direct way to be useful to a community.
    pub fn provision_settlement(&mut self) {
        let Some((ri, si)) = self.player_on_settlement() else {
            self.status_msg = Some("No settlement here to provision.".into());
            return;
        };
        if self.market_barred() {
            self.status_msg = Some("They'll take nothing from your hand here.".into());
            return;
        }
        let Some(want) = self.settlement_shortfall() else {
            return;
        };
        let (cap, pname, sid) = self
            .sim
            .as_ref()
            .and_then(|sim| sim.world.regions.get(ri)?.settlements.get(si))
            .map(|s| {
                (
                    (s.population as f64 * 0.5).max(1.0),
                    s.name.clone(),
                    s.id.clone(),
                )
            })
            .unwrap_or((1.0, String::new(), String::new()));
        let have = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.get(want))
            .unwrap_or(0);
        if have == 0 {
            self.status_msg = Some(format!(
                "{pname} is short of {} — bring some and they'll pay well for it.",
                want.name()
            ));
            return;
        }
        // Deliver what they can hold, up to a few, up to what you carry.
        let room = (cap - self.current_settlement().map_or(0.0, |s| s.good(want))).max(0.0);
        let deliver = (have.min(3) as f64).min(room.ceil()).max(0.0) as u32;
        if deliver == 0 {
            self.status_msg = Some(format!("{pname} has all the {} it can hold.", want.name()));
            return;
        }
        // A fair provisioning premium over the bare base price — they need it.
        let pay = (deliver as f64 * want.base_price() as f64 * 1.25).round() as u32;
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(want, deliver);
            ps.inventory.add(ItemType::Coin, pay);
        }
        // The good actually lands in the town's stores.
        if let Some(ref mut sim) = self.sim {
            if let Some(s) = sim
                .world
                .regions
                .get_mut(ri)
                .and_then(|r| r.settlements.get_mut(si))
            {
                let cur = s.good(want);
                s.goods_stock.insert(want, cur + deliver as f64);
                // A trade-runner who keeps a town supplied lifts the hand of its
                // Traders — the faction of roads and coin (#565): provision a
                // place often enough and you shift who holds its council.
                s.politics
                    .adjust(crate::model::economy::Faction::Traders, 0.01);
            }
            // Carrying goods between two RIVAL towns thaws their feud a little
            // (#565 slice 3): the player is a cart crossing where no town's own
            // cart will, and trade is the solvent. Their bond eases toward
            // neutral, and a thaw the road notices is talked of.
            let prev = sim.last_provisioned_town.take();
            if let Some(prev_name) = prev {
                if prev_name != pname
                    && sim.province_ties.tie(&prev_name, &pname)
                        == crate::model::province::TieKind::Rival
                {
                    sim.province_ties.nudge(&prev_name, &pname, 0.06);
                    if sim.province_ties.tie(&prev_name, &pname)
                        != crate::model::province::TieKind::Rival
                    {
                        let tick = sim.world.tick;
                        sim.log(
                            tick,
                            crate::sim::journal::Voice::Rumor,
                            format!(
                                "The old bad blood between {prev_name} and {pname} is easing — a trader crosses between them again."
                            ),
                        );
                    }
                }
            }
            sim.last_provisioned_town = Some(pname.clone());
        }
        self.advance_clock_hour();
        self.god_affinity.adjust(GodName::Masa, 0.02);
        // A provisioner is remembered: standing rises more than a plain sale.
        let pid = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        if let Some(ref mut sim) = self.sim {
            if !pid.is_empty() && !sid.is_empty() {
                sim.reputation.adjust_local(&pid, &sid, 0.03);
            }
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                format!("I carried {deliver} {} into {pname}, who were short of it. They paid well, and will know my face.", want.name()),
            );
        }
        self.status_msg = Some(format!(
            "You provision {pname} with {deliver} {} (+{pay} coin, standing rises, 1h)",
            want.name()
        ));
    }

    pub fn work_for_hire(&mut self) {
        let Some((ri, si)) = self.player_on_settlement() else {
            self.status_msg = Some("No settlement here to take you on.".into());
            return;
        };
        let Some((people, lean, pname, sid)) = self.sim.as_ref().and_then(|sim| {
            let s = sim.world.regions.get(ri)?.settlements.get(si)?;
            Some((
                s.people.first().map(|p| PeopleKind::from_name(&p.people)),
                s.food_scarcity_modifier() > 1.0,
                s.name.clone(),
                s.id.clone(),
            ))
        }) else {
            return;
        };
        if let Some(pk) = people {
            if self.inter_people_bias.effective_bias(pk) < -0.15 {
                self.status_msg = Some(format!("No one in {pname} will set your kind to work."));
                return;
            }
        }
        // A meagre day-wage; a town short of hands (lean stores) pays the more.
        // Fortune leans it a touch; never to a fortune.
        let base = if lean { 4 } else { 3 };
        let wage = ((base as f64) * (1.0 + self.fortune.value() * 0.12))
            .round()
            .max(1.0) as u32;
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.add(ItemType::Coin, wage);
        }
        self.advance_clock(8);
        self.vitals.energy = (self.vitals.energy - 0.2 * self.vow_work_energy_mult()).max(0.0);
        self.vitals.hunger = (self.vitals.hunger - 0.1).max(0.0);
        // Honest work mends standing a little and pleases the trade-keeper Masa.
        self.god_affinity.adjust(GodName::Masa, 0.02);
        let pid = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        if let Some(ref mut sim) = self.sim {
            if !pid.is_empty() {
                sim.reputation.adjust_local(&pid, &sid, 0.01);
            }
            // Hands in the fields lift a hungry town's stores a little.
            if lean {
                if let Some(s) = sim
                    .world
                    .regions
                    .get_mut(ri)
                    .and_then(|r| r.settlements.get_mut(si))
                {
                    s.food_stock += 2.0;
                }
            }
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                if lean {
                    "I worked the fields a day for a town short of hands. Honest coin, honest ache."
                        .to_string()
                } else {
                    "I hired on for a day's labour. The wage was small; the work was real."
                        .to_string()
                },
            );
        }
        self.status_msg = Some(if lean {
            format!("You work the fields for {pname} (+{wage} coin, 8h)")
        } else {
            format!("You hire on for a day in {pname} (+{wage} coin, 8h)")
        });
    }

    pub fn reputation_in_current_settlement(&self) -> f64 {
        let mut rep = 0.5;
        if let (Some(ref ps), Some(ref sim), Some(pos)) =
            (&self.player_start, &self.sim, self.player_pos)
        {
            if let Some(region) = sim.world.regions.get(pos.region_idx) {
                if let Some(settlement) = region.settlements.first() {
                    rep = sim.reputation.get(&ps.person.id, &settlement.id);
                }
            }
        }
        rep
    }

    pub fn politics_price_modifier(&self) -> f64 {
        if let (Some(ref sim), Some(pos)) = (&self.sim, self.player_pos) {
            if let Some(region) = sim.world.regions.get(pos.region_idx) {
                if let Some(settlement) = region.settlements.first() {
                    return settlement.politics.price_modifier();
                }
            }
        }
        1.0
    }

    /// Whether the player carries the scale-hand — the Väylä weight-sense that
    /// reads true value in a trade (#439).
    fn scale_hand(&self) -> bool {
        self.gift.sense().map(|s| s.aids_trade()).unwrap_or(false)
    }

    /// A gifted crafter living in this settlement makes their craft-goods truer
    /// and cheaper here (#441): an iron-ear smith's tools, a root-eye's salves.
    /// Returns the price multiplier for the item (1.0 if no matching gift).
    fn settlement_gift_discount(&self, item: ItemType) -> f64 {
        let Some(s) = self.current_settlement() else {
            return 1.0;
        };
        for p in &s.people {
            if let Some(sense) = p.gift.sense() {
                let matches_goods = match sense {
                    crate::model::CraftSense::IronEar => item == ItemType::Tool,
                    crate::model::CraftSense::RootEye => {
                        matches!(item, ItemType::Bandage | ItemType::Salve)
                    }
                    _ => false,
                };
                if matches_goods {
                    return 0.85;
                }
            }
        }
        1.0
    }

    /// Reading the true price is the gift at work — and the gift costs the body.
    /// Only the scale-hand pays here (#439).
    fn charge_gift_for_trade(&mut self) {
        if self.scale_hand() {
            if let Some(note) = self.use_gift() {
                if let Some(m) = self.status_msg.as_mut() {
                    m.push_str(&note);
                }
            }
        }
    }

    /// How well coin trades in the ruling polity's markets (canon: no universal
    /// currency). 1.0 in coin economies; a discount where coin is debased
    /// (Remnant) or a foreign convenience (grain/in-kind economies).
    pub fn coin_value_here(&self) -> f64 {
        self.sim
            .as_ref()
            .map(|s| s.world.polity.coin_value_modifier())
            .unwrap_or(1.0)
    }

    /// What the market charges the player. Includes ALL the modifiers the
    /// transaction applies (politics + caravan supply too — the quotes used to
    /// omit those, so the displayed price could differ from the charged one).
    pub fn quote_buy_price(&self, item: ItemType) -> u32 {
        self.buy_price_inner(item, true)
    }

    /// The buy price, optionally without the scale-hand discount. The sell-side
    /// spread clamp uses the gift-free price, so a scale-hand's cheaper buying
    /// does not also drag down what it can sell for (#439).
    fn buy_price_inner(&self, item: ItemType, with_gift: bool) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|sp| self.inter_people_bias.price_modifier(sp))
            .unwrap_or(1.0);
        let rep_mod =
            crate::sim::signals::reputation_price_modifier(self.reputation_in_current_settlement());
        // The fortunate strike a slightly better bargain — a few coppers, not a
        // fortune, and bounded; the cursed pay a little over the odds.
        let luck = 1.0 - self.fortune.value() * 0.08;
        // Coin worth less here means it takes more of it to buy: divide the
        // price by how well coin trades in this polity.
        let coin = self.coin_value_here();
        // A market fair makes goods cheaper this season (#417).
        let event = self
            .current_world_event()
            .map(|e| e.buy_price_modifier())
            .unwrap_or(1.0);
        // The scale-hand reads the fair price and buys under it (#439).
        let gift = if with_gift && self.scale_hand() {
            0.90
        } else {
            1.0
        };
        // A gifted crafter in town makes their goods truer and cheaper (#441).
        let town_gift = self.settlement_gift_discount(item);
        let m = inter_mod
            * rep_mod
            * self.politics_price_modifier()
            * self.caravan_price_modifier(item)
            * self.food_scarcity_modifier(item)
            * luck
            * event
            * gift
            * town_gift
            * self.vow_buy_mult()
            * self.goods_abundance_modifier(item)
            / coin;
        ((base as f64 * m).ceil() as u32).max(1)
    }

    /// Hunger in the settlement moves the price of what can be eaten: lean
    /// stores raise Food/Herb prices, full ones lower them.
    fn food_scarcity_modifier(&self, item: ItemType) -> f64 {
        if !matches!(item, ItemType::Food | ItemType::Herb) {
            return 1.0;
        }
        self.current_settlement()
            .map(|s| s.food_scarcity_modifier())
            .unwrap_or(1.0)
    }

    /// What the quality of the player's own piece does to its sale price (#547):
    /// a masterwork fetches more, a worn or rough one less. Reads the durability
    /// the item carries (craft quality + wear). `1.0` for what you do not hold.
    fn craft_quality_modifier(&self, item: ItemType) -> f64 {
        self.player_start
            .as_ref()
            .filter(|ps| ps.inventory.has(item))
            .map(|ps| ps.inventory.quality(item).sell_multiplier())
            .unwrap_or(1.0)
    }

    /// A trade good's price leans on the settlement's **own stock** (#540 living
    /// economy): a town holding plenty of what its trades make sells it cheap; a
    /// town that lacks it pays dear. Drives inter-settlement arbitrage — buy
    /// where a good is plentiful, carry it where it is scarce. `1.0` for goods
    /// the living economy does not yet track, or when off a settlement.
    fn goods_abundance_modifier(&self, item: ItemType) -> f64 {
        if !matches!(
            item,
            ItemType::Iron | ItemType::Tool | ItemType::Cloth | ItemType::Wood
        ) {
            return 1.0;
        }
        self.current_settlement()
            .map(|s| {
                let cap = (s.population as f64 * 0.5).max(1.0);
                let abundance = (s.good(item) / cap).clamp(0.0, 1.0);
                // Scarce (empty) → 1.25 dearer; full → 0.80 cheaper.
                1.25 - 0.45 * abundance
            })
            .unwrap_or(1.0)
    }

    /// What the market pays the player — always below the buy price, merchants
    /// take a margin. Without the clamp, high reputation inverted the spread
    /// (buy at 0.6x base, sell at 1.4x) and buy->sell was an infinite-coin loop.
    pub fn quote_sell_price(&self, item: ItemType) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|bp| 2.0 - self.inter_people_bias.price_modifier(bp))
            .unwrap_or(1.0);
        let rep_mod = 2.0
            - crate::sim::signals::reputation_price_modifier(
                self.reputation_in_current_settlement(),
            );
        // The fortunate sell a little dearer — the mirror of their buying luck.
        let luck = 1.0 + self.fortune.value() * 0.08;
        // Coin worth less here means you receive fewer of it for your goods.
        let coin = self.coin_value_here();
        // The scale-hand will not be short-weighted: it sells a little dearer.
        let gift = if self.scale_hand() { 1.10 } else { 1.0 };
        let m = inter_mod
            * rep_mod
            * self.politics_price_modifier()
            * self.caravan_price_modifier(item)
            * self.food_scarcity_modifier(item)
            * self.goods_abundance_modifier(item)
            * self.craft_quality_modifier(item)
            * luck
            * coin
            * gift;
        let raw = ((base as f64 * m).floor() as u32).max(1);
        // Clamp against the gift-free buy price so the scale-hand's cheaper
        // buying does not also cap what it can sell for (#439).
        let buy = self.buy_price_inner(item, false);
        if buy > 1 {
            raw.clamp(1, buy - 1)
        } else {
            1
        }
    }
}
