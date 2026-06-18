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
    /// Tend a plagued town with medicine (#604 slice 4): if the player stands in
    /// a stricken settlement and carries a Salve or Bandage, lay it down — easing
    /// the plague (cutting its days), paid a fair fee, and remembered for it the
    /// way a town remembers the stranger who kept it fed. Returns true when it
    /// handled the act (medicine given, or named the need so plain provisioning
    /// does not run its bread path over a town that wants physic). False when the
    /// town is not plagued, so ordinary provisioning proceeds.
    fn tend_plague_here(&mut self) -> bool {
        let Some((ri, si)) = self.player_on_settlement() else {
            return false;
        };
        let (plagued, pname) = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(ri)?.settlements.get(si))
            .map(|s| (s.plague_days > 0, s.name.clone()))
            .unwrap_or((false, String::new()));
        if !plagued {
            return false;
        }
        let med = [ItemType::Salve, ItemType::Bandage].into_iter().find(|m| {
            self.player_start
                .as_ref()
                .map(|ps| ps.inventory.get(*m) > 0)
                .unwrap_or(false)
        });
        let Some(med) = med else {
            // The town is stricken but the player has no physic; provisioning it
            // with bread is still allowed, so let the ordinary path run.
            return false;
        };
        let pay = (med.base_price() as f64 * 1.25).round() as u32;
        let player_name = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.name.clone())
            .unwrap_or_default();
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(med, 1);
            ps.inventory.add(ItemType::Coin, pay);
        }
        let pid = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let mut broke = false;
        if let Some(ref mut sim) = self.sim {
            if let Some(s) = sim
                .world
                .regions
                .get_mut(ri)
                .and_then(|r| r.settlements.get_mut(si))
            {
                s.plague_days = s.plague_days.saturating_sub(4);
                broke = s.plague_days == 0;
                if s.remembered_deed.is_none() && !player_name.is_empty() {
                    s.remembered_deed = Some(format!(
                        "{player_name}, who brought medicine through the plague"
                    ));
                }
                let sid = s.id.clone();
                if !pid.is_empty() {
                    sim.reputation.adjust_local(&pid, &sid, 0.03);
                }
            }
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Faith,
                format!("I carried {} into {pname} in its sickness.", med.name()),
            );
        }
        self.advance_clock_hour();
        // Answering a plague-relief task the living world posted (#613-epic).
        self.complete_world_task(true, &pname);
        let note = if broke {
            " The worst of the sickness breaks."
        } else {
            ""
        };
        self.status_msg = Some(format!(
            "You bring {} to {pname} in the grip of the plague (+{pay} coin, standing rises).{note} (1h)",
            med.name()
        ));
        true
    }

    pub fn provision_settlement(&mut self) {
        let Some((ri, si)) = self.player_on_settlement() else {
            self.status_msg = Some("No settlement here to provision.".into());
            return;
        };
        if self.market_barred() {
            self.status_msg = Some("They'll take nothing from your hand here.".into());
            return;
        }
        // A town in the grip of a plague needs medicine before bread (#604 slice
        // 4): if the player carries it, tending the sickness takes precedence.
        if self.tend_plague_here() {
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
        // In wartime the roads are raided and goods scarce (#579 slice 4): a
        // runner who gets supplies through the blockade is paid better still.
        let at_war = self.polity_at_war();
        let war_premium = if at_war { 1.4 } else { 1.0 };
        let pay = (deliver as f64 * want.base_price() as f64 * 1.25 * war_premium).round() as u32;
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(want, deliver);
            ps.inventory.add(ItemType::Coin, pay);
        }
        let player_name = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.name.clone())
            .unwrap_or_default();
        // The good actually lands in the town's stores.
        let mut brokered_truce: Option<(String, String)> = None;
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
                // A town fed through a famine remembers the stranger who did it
                // (#565 slice 4): a lasting mark, set once, that its talk and the
                // road will carry for as long as the town stands.
                if s.famine_days > 0 && s.remembered_deed.is_none() && !player_name.is_empty() {
                    s.remembered_deed = Some(format!(
                        "{player_name}, the stranger who kept us fed through the lean year"
                    ));
                } else if at_war && s.remembered_deed.is_none() && !player_name.is_empty() {
                    // Kept supplied while the roads were closed and raided: a war
                    // analogue of the lean-year deed (#579 slice 4).
                    s.remembered_deed = Some(format!(
                        "{player_name}, who ran supplies to us through the war"
                    ));
                }
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
                    // Crossing between two DEEP rivals — the raiding kind (#579
                    // slice 4) — is the braver run, and it cools the feud
                    // harder: the player is the cart that crosses where raids
                    // fly.
                    let deep = sim.province_ties.bond(&prev_name, &pname) <= -0.7;
                    sim.province_ties
                        .nudge(&prev_name, &pname, if deep { 0.10 } else { 0.06 });
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
                        // The feud is out of rivalry: a broker-truce task, if one
                        // stood for this pair, is answered (#614 slice 2).
                        brokered_truce = Some((prev_name.clone(), pname.clone()));
                    }
                }
            }
            sim.last_provisioned_town = Some(pname.clone());
        }
        if let Some((x, y)) = brokered_truce {
            self.complete_truce_task(&x, &y);
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
        // Answering a famine-relief task the living world posted (#613-epic);
        // a no-op if no such task stands for this town.
        self.complete_world_task(false, &pname);
        self.status_msg = Some(if at_war {
            format!(
                "You run {deliver} {} into {pname} past the war's closed roads (+{pay} coin, standing rises, 1h)",
                want.name()
            )
        } else {
            format!(
                "You provision {pname} with {deliver} {} (+{pay} coin, standing rises, 1h)",
                want.name()
            )
        });
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

    /// The season's plain pressure on the market (#570): goods come dear in
    /// Frost and cheap in high Green. Felt in every buy and sell.
    pub fn season_price_modifier(&self) -> f64 {
        self.clock.season().market_price_modifier()
    }

    /// Whether the province's polity is at open tension with its rival right now
    /// (#579): the war the player can run supplies through. Deterministic on the
    /// same season clock the war rumor, levy, and road-watch use.
    pub fn polity_at_war(&self) -> bool {
        let Some(sim) = self.sim.as_ref() else {
            return false;
        };
        let day = self.clock.day;
        let season_ord = (day / 30) % 4;
        let year = day / 120;
        sim.world.polity.in_tension(self.seed, season_ord, year)
    }

    /// Whether the town the player stands in is mid-festival (#570 slice 3):
    /// the doors are open, the drink flows, and the stalls sell a little under
    /// the odds.
    pub fn in_festival_here(&self) -> bool {
        self.current_settlement()
            .map(|s| s.in_festival(self.clock.day))
            .unwrap_or(false)
    }

    /// A festival in town is a boon the player feels at the market: goods go a
    /// little cheaper while the doors are open (#570 slice 3).
    fn festival_discount(&self) -> f64 {
        if self.in_festival_here() {
            0.90
        } else {
            1.0
        }
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
            * self.season_price_modifier()
            * self.festival_discount()
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
            * self.season_price_modifier()
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

#[cfg(test)]
mod festival_tests {
    use super::*;
    use crate::charts::load::load_charts;

    fn app_in_first_settlement() -> App {
        let mut app = App::new(7, load_charts().unwrap());
        app.generate_player();
        app.accept_player();
        app.screen = Screen::Location {
            region_idx: 0,
            settlement_idx: 0,
            scroll: 0,
        };
        app
    }

    #[test]
    fn polity_at_war_tracks_the_tension_seasons() {
        let mut app = app_in_first_settlement();
        app.enter_map(0);
        let polity = app.sim.as_ref().unwrap().world.polity;
        let seed = app.seed;
        let war = (0..600u32).find(|&d| polity.in_tension(seed, (d / 30) % 4, d / 120));
        let peace = (0..600u32).find(|&d| !polity.in_tension(seed, (d / 30) % 4, d / 120));
        let (war, peace) = (war.expect("a war season"), peace.expect("a peace season"));
        app.clock.day = peace;
        assert!(!app.polity_at_war(), "peace reads as peace");
        app.clock.day = war;
        assert!(app.polity_at_war(), "tension reads as war");
    }

    #[test]
    fn a_deep_rivalry_posts_a_broker_truce_task() {
        use crate::model::quest::QuestKind;
        let mut app = App::new(7, load_charts().unwrap());
        app.generate_player();
        app.accept_player();
        // Two named towns at deep rivalry.
        let (a, b) = {
            let regs = &app.sim.as_ref().unwrap().world.regions;
            let mut names = Vec::new();
            'o: for r in regs {
                for s in &r.settlements {
                    if s.population > 1 {
                        names.push(s.name.clone());
                        if names.len() == 2 {
                            break 'o;
                        }
                    }
                }
            }
            (names[0].clone(), names[1].clone())
        };
        app.sim.as_mut().unwrap().province_ties.nudge(&a, &b, -0.9);
        app.generate_world_task_quests();
        let posted = app.sim.as_ref().unwrap().quests.iter().any(|q| {
            matches!(&q.kind, QuestKind::BrokerTruce { a: qa, b: qb }
                if (qa == &a && qb == &b) || (qa == &b && qb == &a))
        });
        assert!(posted, "a deep rivalry posts a broker-truce task");
        // Answering it (the peace brokered) resolves it.
        app.complete_truce_task(&a, &b);
        let still = app
            .sim
            .as_ref()
            .unwrap()
            .quests
            .iter()
            .any(|q| matches!(&q.kind, QuestKind::BrokerTruce { .. }));
        assert!(!still, "brokering the peace clears the task");
        assert_eq!(app.milestones.quests_completed, 1);
    }

    #[test]
    fn a_plagued_town_posts_a_relief_task_the_player_can_answer() {
        use crate::model::quest::QuestKind;
        let mut app = App::new(7, load_charts().unwrap());
        app.generate_player();
        app.accept_player();
        let town = {
            let s = &mut app.sim.as_mut().unwrap().world.regions[0].settlements[0];
            s.plague_days = 8;
            s.name.clone()
        };
        // The living world posts the call.
        app.generate_world_task_quests();
        let posted = app.sim.as_ref().unwrap().quests.iter().any(
            |q| matches!(&q.kind, QuestKind::RelievePlague { settlement } if *settlement == town),
        );
        assert!(posted, "the plagued town posts a relief task");
        // Answering it (the act of tending) resolves it: reward paid, task
        // cleared, quest recorded.
        app.complete_world_task(true, &town);
        let still_open = app.sim.as_ref().unwrap().quests.iter().any(
            |q| matches!(&q.kind, QuestKind::RelievePlague { settlement } if *settlement == town),
        );
        assert!(!still_open, "answering the call clears the task");
        assert_eq!(
            app.milestones.quests_completed, 1,
            "the answered call counts as a quest done"
        );
    }

    #[test]
    fn bringing_medicine_eases_a_plagued_town() {
        use crate::model::economy::SettlementService;
        use crate::model::PlayerPos;
        let mut app = App::new(7, load_charts().unwrap());
        app.generate_player();
        app.accept_player();
        app.screen = Screen::Location {
            region_idx: 0,
            settlement_idx: 0,
            scroll: 0,
        };
        // Plague the town, stand the player on its ground, hand them a Salve.
        let (mx, my) = {
            let s = &mut app.sim.as_mut().unwrap().world.regions[0].settlements[0];
            s.plague_days = 10;
            s.services.push(SettlementService::Shrine);
            (s.map_x as usize + 1, s.map_y as usize + 1)
        };
        app.player_pos = Some(PlayerPos {
            region_idx: 0,
            px: mx,
            py: my,
        });
        app.player_start
            .as_mut()
            .unwrap()
            .inventory
            .add(ItemType::Salve, 1);
        // Only proceed if the player actually stands in town (footprint paint).
        if app.player_on_settlement().is_some() {
            app.provision_settlement();
            let s = &app.sim.as_ref().unwrap().world.regions[0].settlements[0];
            assert!(s.plague_days < 10, "the medicine eases the plague");
            assert!(
                s.remembered_deed
                    .as_deref()
                    .unwrap_or("")
                    .contains("medicine"),
                "the town remembers the medicine-bringer"
            );
            assert!(app
                .status_msg
                .as_deref()
                .unwrap_or("")
                .contains("grip of the plague"));
        }
    }

    #[test]
    fn a_festival_in_town_cheapens_the_stalls() {
        let mut app = app_in_first_settlement();
        // A coat is dear enough that the discount clears the rounding.
        let item = ItemType::Coat;
        // No festival in town: the plain price, no discount.
        if let Some(sim) = app.sim.as_mut() {
            sim.world.regions[0].settlements[0].festival_until_day = 0;
        }
        assert!(!app.in_festival_here());
        assert!((app.festival_discount() - 1.0).abs() < 1e-9);
        let plain = app.quote_buy_price(item);
        // The town turns to festival, running past today.
        let today = app.clock.day;
        if let Some(sim) = app.sim.as_mut() {
            sim.world.regions[0].settlements[0].festival_until_day = today + 3;
        }
        assert!(app.in_festival_here(), "the player's town is mid-festival");
        assert!((app.festival_discount() - 0.90).abs() < 1e-9);
        let feast = app.quote_buy_price(item);
        assert!(
            feast < plain,
            "festival goods are cheaper: {feast} should be under {plain}"
        );
    }
}
