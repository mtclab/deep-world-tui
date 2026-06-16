use crate::gen::companion::settlement_companions;
use crate::model::{EncounterAction, GodName, ItemType, Need, PeopleKind};
use crate::rng::SeedRng;

use super::*;

/// A plain compass bearing for a tile offset (north = up / −y), as a traveller
/// would name it (#528). Diagonals when both axes carry; a single direction
/// when one clearly dominates (offset > 2× the other).
fn compass_bearing(dx: i64, dy: i64) -> &'static str {
    if dx == 0 && dy == 0 {
        return "right here";
    }
    let ns = if dy < 0 { "north" } else { "south" };
    let ew = if dx < 0 { "west" } else { "east" };
    let (adx, ady) = (dx.abs(), dy.abs());
    if dy == 0 || adx > ady * 2 {
        return if dx < 0 { "to the west" } else { "to the east" };
    }
    if dx == 0 || ady > adx * 2 {
        return if dy < 0 {
            "to the north"
        } else {
            "to the south"
        };
    }
    match (ns, ew) {
        ("north", "west") => "to the northwest",
        ("north", "east") => "to the northeast",
        ("south", "west") => "to the southwest",
        _ => "to the southeast",
    }
}

impl App {
    pub fn enter_npc(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_npc(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    /// Best travel-speed multiplier among the player's companions (a horse
    /// quickens the road); 1.0 with none.
    pub(super) fn companion_travel_mult(&self) -> f64 {
        self.player_start
            .as_ref()
            .map(|ps| {
                ps.companions
                    .iter()
                    .map(|c| c.animal.travel_speed_multiplier())
                    .fold(1.0, f64::min)
            })
            .unwrap_or(1.0)
    }

    pub fn enter_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if region.region_type == "forest" && self.god_affinity.get(GodName::Keuru) > 0.2 {
                    self.god_affinity.adjust(GodName::Keuru, 0.01);
                }
                if region.region_type == "river_valley"
                    && self.god_affinity.get(GodName::Masa) > 0.2
                {
                    self.god_affinity.adjust(GodName::Masa, 0.01);
                }
            }
        }
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.01);
        self.screen = Screen::Talk {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn give_food(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Food, 1) {
                self.status_msg = Some("No food to give".into());
                return;
            }
        }
        let npc_people = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| PeopleKind::from_name(&p.people))
        });
        if let Some(npc_pk) = npc_people {
            let mut bias = self.inter_people_bias.effective_bias(npc_pk);
            if let Some(god) = npc_pk.patron_god() {
                if self.god_affinity.get(god) > 0.4 {
                    bias += 0.05;
                }
            }
            if bias < -0.20 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Food, 1);
                }
                self.status_msg = Some(format!(
                    "'Keep your food, {}.' They push it back. 'We don't take from clearing-sympathizers.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Food, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    let mut trust_bonus = 0.05;
                    let mut rep_bonus = 0.02;
                    if self.god_affinity.get(GodName::Oltzed) > 0.3 {
                        trust_bonus += 0.02;
                        rep_bonus += 0.01;
                    }
                    if self.god_affinity.get(GodName::Masa) > 0.3 {
                        trust_bonus += 0.01;
                    }
                    let npc_people_pk = PeopleKind::from_name(&person.people);
                    sim.relationships.update_relationship_biased_full(
                        pid,
                        &person_id,
                        "gave food",
                        sim.world.tick,
                        trust_bonus,
                        0.03,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                        &person.personality,
                    );
                    sim.reputation.adjust_local_biased(
                        pid,
                        sid,
                        rep_bonus,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                    );
                }
                self.status_msg = Some(format!("Gave food to {}", person.name));
                self.god_affinity.adjust(GodName::Oltzed, 0.02);
                self.god_affinity.adjust(GodName::Masa, 0.01);
                if let Some(god) = PeopleKind::from_name(&person.people).patron_god() {
                    self.god_affinity.adjust(god, 0.01);
                }
                self.check_quests_on_aid(&person_id);
            }
        }
    }

    /// Treat a sick villager with what you carry — the herbalist's work (#454).
    /// The same remedies that tend your own sickness (#451) ease theirs; the
    /// root-eye heals true. A healer is remembered: standing rises, the people
    /// warm, and the river-keeper Masa marks the kindness. Refused only by those
    /// who will not take help from your kind.
    pub fn heal_npc(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        use crate::model::Disease;
        // Is there anything to treat?
        let sick = self
            .sim
            .as_ref()
            .and_then(|sim| {
                sim.world
                    .regions
                    .get(region_idx)?
                    .settlements
                    .get(settlement_idx)?
                    .people
                    .get(person_idx)
            })
            .map(|p| !p.illnesses.is_empty())
            .unwrap_or(false);
        if !sick {
            self.status_msg = Some("They are hale enough — no sickness to tend.".into());
            return;
        }
        // Those set hard against your kind will not take healing from your hand.
        let npc_pk = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| PeopleKind::from_name(&p.people))
        });
        if let Some(pk) = npc_pk {
            let mut bias = self.inter_people_bias.effective_bias(pk);
            if let Some(god) = pk.patron_god() {
                if self.god_affinity.get(god) > 0.4 {
                    bias += 0.05;
                }
            }
            if bias < -0.20 {
                self.status_msg = Some(
                    "'I'd sooner keep the fever.' They will not take healing from your hand."
                        .into(),
                );
                return;
            }
        }
        let root_eye = self.gift.sense() == Some(crate::model::CraftSense::RootEye);
        let is_wound =
            |d: Disease| matches!(d, Disease::Infection | Disease::Venom | Disease::Sprain);
        // Does the patient carry a wound-illness a salve answers best?
        let has_wound = self
            .sim
            .as_ref()
            .and_then(|sim| {
                sim.world
                    .regions
                    .get(region_idx)?
                    .settlements
                    .get(settlement_idx)?
                    .people
                    .get(person_idx)
            })
            .map(|p| p.illnesses.iter().any(|d| is_wound(d.disease)))
            .unwrap_or(false);
        // Spend a remedy from the player's own stores.
        let used: Option<&'static str> = if let Some(ref mut ps) = self.player_start {
            if has_wound && ps.inventory.remove(ItemType::Salve, 1) {
                Some("salve")
            } else if ps.inventory.remove(ItemType::Herb, 1) {
                Some("herb")
            } else if ps.inventory.remove(ItemType::Bandage, 1) {
                Some("bandage")
            } else if root_eye {
                Some("hands") // the root-eye can ease a little with nothing but skill
            } else {
                None
            }
        } else {
            None
        };
        let Some(remedy) = used else {
            self.status_msg =
                Some("You have nothing to treat them with — no herb, no salve, no bandage.".into());
            return;
        };

        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        let strong = remedy == "salve";
        let mut name = String::new();
        let mut patient_disease = None;
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                name = person.name.clone();
                patient_disease = person.illnesses.first().map(|d| d.disease);
                let person_id = person.id.clone();
                for d in person.illnesses.iter_mut() {
                    if strong && is_wound(d.disease) {
                        d.tend_strong();
                    } else {
                        d.tend();
                    }
                    if root_eye {
                        d.tend();
                    }
                }
                // The root-eye can break a mild fever outright.
                if root_eye {
                    person.illnesses.retain(|d| {
                        !matches!(
                            d.disease,
                            Disease::Fever | Disease::WinterCough | Disease::MarshFever
                        )
                    });
                }
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    let npc_people_pk = PeopleKind::from_name(&person.people);
                    sim.relationships.update_relationship_biased_full(
                        pid,
                        &person_id,
                        "tended their sickness",
                        sim.world.tick,
                        0.07,
                        0.04,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                        &person.personality,
                    );
                    sim.reputation.adjust_local_biased(
                        pid,
                        sid,
                        0.04,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                    );
                }
                self.check_quests_on_aid(&person_id);
            }
        }
        // The river-keeper marks the mercy; the patient's own god too.
        self.god_affinity.adjust(GodName::Masa, 0.02);
        if let Some(pk) = npc_pk {
            if let Some(god) = pk.patron_god() {
                self.god_affinity.adjust(god, 0.01);
            }
        }
        let mut msg = format!("You tend {name}'s sickness with what you carry.");
        if root_eye {
            msg = format!("You lay hands on {name} — the root-eye reads what the body needs.");
            if let Some(note) = self.use_gift() {
                msg.push_str(&note);
            }
        }
        // The healer's hazard (#457): in a plague year, tending the sick is how
        // the plague finds the healer. Fortune-leaned; a real price for the
        // standing it earns. (Outside a plague year, ordinary tending is safe.)
        let plague = self.current_world_event() == Some(crate::model::WorldEvent::PlagueYear);
        if plague {
            if let Some(d) = patient_disease
                .filter(|d| !matches!(d, Disease::Sprain | Disease::FlameFever | Disease::IronAche))
            {
                let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
                let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x4EA1_E12E));
                if crate::rng::unit_from_hash(h) < self.fortune.tilt_bad(0.18)
                    && self.afflict(
                        d,
                        "I tended the sick through the plague, and it found me too.",
                    )
                {
                    msg.push_str(" The plague is on the wind — and now it is on you.");
                }
            }
        }
        self.advance_clock(1);
        self.status_msg = Some(msg);
    }

    pub fn give_coin(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Coin, 1) {
                self.status_msg = Some("No coin to give".into());
                return;
            }
        }
        let npc_people = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| PeopleKind::from_name(&p.people))
        });
        if let Some(npc_pk) = npc_people {
            let mut bias = self.inter_people_bias.effective_bias(npc_pk);
            if let Some(god) = npc_pk.patron_god() {
                if self.god_affinity.get(god) > 0.4 {
                    bias += 0.05;
                }
            }
            if bias < -0.20 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Coin, 1);
                }
                self.status_msg = Some(format!(
                    "The coin is set back on the table. 'We don't take {} coin here.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Money, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    let mut trust_bonus = 0.03;
                    let mut rep_bonus = 0.01;
                    if self.god_affinity.get(GodName::Oltzed) > 0.3 {
                        trust_bonus += 0.01;
                        rep_bonus += 0.01;
                    }
                    if self.god_affinity.get(GodName::Masa) > 0.3 {
                        trust_bonus += 0.01;
                    }
                    let npc_people_pk = PeopleKind::from_name(&person.people);
                    sim.relationships.update_relationship_biased_full(
                        pid,
                        &person_id,
                        "gave coin",
                        sim.world.tick,
                        trust_bonus,
                        rep_bonus,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                        &person.personality,
                    );
                    sim.reputation.adjust_local_biased(
                        pid,
                        sid,
                        0.01,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                    );
                }
                self.status_msg = Some(format!("Gave coin to {}", person.name));
                self.god_affinity.adjust(GodName::Oltzed, 0.02);
                self.god_affinity.adjust(GodName::Masa, 0.01);
                if let Some(god) = PeopleKind::from_name(&person.people).patron_god() {
                    self.god_affinity.adjust(god, 0.01);
                }
                self.check_quests_on_aid(&person_id);
            }
        }
    }

    pub fn current_settlement_people(&self) -> Option<PeopleKind> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let settlement = region.settlements.first()?;
        let dominant = settlement.people.first()?;
        Some(PeopleKind::from_name(&dominant.people))
    }

    /// Give the people of this settlement a gift — the most valued good you
    /// carry — to lift your **standing** with them (#454). Goodwill is a kind
    /// of coin: a gift mends the bias that gates entry and trade, plateauing
    /// like all standing, the richer the gift the more it mends. The Five
    /// especially do not forget a gift freely given.
    pub fn give_gift(&mut self) {
        use crate::model::ItemType;
        let Some(people) = self.current_settlement_people() else {
            self.status_msg = Some("There is no one here to receive a gift.".into());
            return;
        };
        // The most valued tradeable good you carry — never your food or water.
        let best = self.player_start.as_ref().and_then(|ps| {
            ItemType::tradeable_items()
                .into_iter()
                .filter(|&i| !matches!(i, ItemType::Food | ItemType::Water | ItemType::Coin))
                .filter(|&i| ps.inventory.get(i) > 0)
                .max_by_key(|&i| i.base_price())
        });
        let Some(item) = best else {
            self.status_msg = Some("You have nothing worth giving as a gift.".into());
            return;
        };
        if !self
            .player_start
            .as_mut()
            .is_some_and(|ps| ps.inventory.remove(item, 1))
        {
            return;
        }
        // The richer the gift, the more it lifts your standing — plateauing.
        let delta = (item.base_price() as f64 / 40.0).clamp(0.03, 0.12);
        self.inter_people_bias.mod_toward(people, delta);
        self.advance_clock(1);
        let of_five = people.is_of_the_five();
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            let tail = if of_five {
                "The Five do not forget a gift freely given."
            } else {
                "Goodwill is a kind of coin."
            };
            sim.log(
                tick,
                crate::sim::journal::Voice::Encounter,
                format!(
                    "I gave the {} a gift of {}. {tail}",
                    people.label(),
                    item.name()
                ),
            );
        }
        self.status_msg = Some(format!(
            "You give {} as a gift to the {}. Your standing with them rises. (1h)",
            item.name(),
            people.label()
        ));
    }

    pub fn npc_memory(&self, person_id: &str) -> Option<&crate::model::NpcMemory> {
        self.sim
            .as_ref()
            .and_then(|sim| sim.npc_memories.get(person_id))
    }

    pub fn has_met_npc(&self, person_id: &str) -> bool {
        self.npc_memory(person_id).is_some_and(|m| m.count() > 0)
    }

    pub fn npc_trust_bonus(&self, person_id: &str) -> f64 {
        self.npc_memory(person_id)
            .map_or(0.0, |m| m.cumulative_trust().clamp(-0.3, 0.3))
    }

    pub fn record_npc_memory(
        &mut self,
        settlement_idx: usize,
        person_idx: usize,
        action: EncounterAction,
        trust_delta: f64,
    ) {
        let (person_id, settlement_name, _region_idx) = if let Some(ref sim) = self.sim {
            let pos = match self.player_pos {
                Some(p) => p,
                None => return,
            };
            let region = sim.world.regions.get(pos.region_idx);
            let settlement = region.and_then(|r| r.settlements.get(settlement_idx));
            let person = settlement.and_then(|s| s.people.get(person_idx));
            match (person, settlement) {
                (Some(p), Some(s)) => (p.id.clone(), s.name.clone(), pos.region_idx),
                _ => return,
            }
        } else {
            return;
        };
        let tick = (self.clock.day * 24 + self.clock.hour) as u64;
        if let Some(ref mut sim) = self.sim {
            sim.npc_memories.entry(person_id).or_default().add(
                action,
                tick,
                settlement_name,
                trust_delta,
            );
        }
    }

    pub fn npc_will_engage(
        &self,
        npc_people_name: &str,
        npc_id: &str,
    ) -> crate::sim::signals::EngagementLevel {
        let bias = crate::model::PeopleKind::from_name(npc_people_name);
        let inter_bias = self.inter_people_bias.effective_bias(bias);
        let rep_drag = (inter_bias * -0.5).clamp(-0.2, 0.2);
        let effective_rep = (self.reputation_in_current_settlement() + rep_drag).clamp(0.0, 1.0);
        let _ = npc_id;
        crate::sim::signals::engagement_for(effective_rep)
    }

    /// Ask an NPC for news (#528 conversations): word of the world, surfaced
    /// through a person rather than only overheard at the tavern. What they
    /// share turns on how they regard you — a cold welcome gives nothing; a
    /// well-travelled trade carries word of the wider world too. The answer is
    /// stable per person per day (ask twice, hear the same).
    pub fn ask_news(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        let Some((pid, people, profession, pname)) = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| {
                    (
                        p.id.clone(),
                        PeopleKind::from_name(&p.people),
                        p.profession.clone(),
                        p.name.clone(),
                    )
                })
        }) else {
            return;
        };
        // How they regard you: cross-people standing + remembered trust, a
        // shared god warming it a little.
        let mut regard = self.inter_people_bias.effective_bias(people) + self.npc_trust_bonus(&pid);
        if let Some(god) = people.patron_god() {
            if self.god_affinity.get(god) > 0.4 {
                regard += 0.05;
            }
        }
        if regard < -0.15 {
            self.status_msg = Some(format!("{pname} has no word for the likes of you."));
            return;
        }
        // A stable answer per person per day.
        let day = self.clock.day;
        let salt = crate::rng::mix_u64(
            pid.bytes()
                .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(b as u64))
                ^ (day as u64).wrapping_shl(32),
        );
        // The well-travelled carry word of the wider world; most know only the
        // local state of things.
        let well_travelled = matches!(
            profession.as_str(),
            "trader" | "sailor" | "path-finder" | "singer" | "scribe"
        );
        let local = self
            .sim
            .as_ref()
            .and_then(|sim| crate::sim::rumors::informed_rumor(sim, day, salt));
        let line = match local {
            Some(r) => r,
            None if well_travelled => {
                let mut rng =
                    crate::rng::SeedRng::new(self.seed).fork_for(&format!("ask-news-{pid}-{day}"));
                crate::sim::journal::rumor_text(&mut rng)
            }
            None => {
                self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.01);
                self.status_msg =
                    Some(format!("{pname}: \"Quiet times. Nothing worth carrying.\""));
                return;
            }
        };
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.01);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(tick, crate::sim::journal::Voice::Rumor, line.clone());
        }
        self.status_msg = Some(format!("{pname} tells you: \"{line}\""));
    }

    /// Ask an NPC the way (#528 conversations): a local points you toward the
    /// nearest neighbouring settlement and any notable place in the region they
    /// know of — directions you would otherwise wander to find. Gated by regard
    /// like asking for news; a cold welcome points you nowhere.
    pub fn ask_directions(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        let Some((people, pname)) = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| (PeopleKind::from_name(&p.people), p.name.clone()))
        }) else {
            return;
        };
        let mut regard = self.inter_people_bias.effective_bias(people);
        if let Some(god) = people.patron_god() {
            if self.god_affinity.get(god) > 0.4 {
                regard += 0.05;
            }
        }
        if regard < -0.15 {
            self.status_msg = Some(format!("{pname} only shrugs — no road they'd set you on."));
            return;
        }
        let Some(pos) = self.player_pos else {
            return;
        };
        let (px, py) = (pos.px as i64, pos.py as i64);
        let mut parts: Vec<String> = Vec::new();
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                // The nearest other settlement, named with a compass bearing.
                let nearest = region
                    .settlements
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != settlement_idx)
                    .filter(|(_, s)| s.population > 0)
                    .min_by_key(|(_, s)| {
                        let dx = s.map_x as i64 - px;
                        let dy = s.map_y as i64 - py;
                        dx * dx + dy * dy
                    });
                if let Some((_, s)) = nearest {
                    let dir = compass_bearing(s.map_x as i64 - px, s.map_y as i64 - py);
                    parts.push(format!("{} lies {}", s.name, dir));
                }
                // A notable place in the region the player has not yet found.
                let place = sim
                    .discoveries
                    .entries
                    .iter()
                    .find(|d| d.region_idx == region_idx && !d.observed);
                if let Some(d) = place {
                    let dir = compass_bearing(d.x as i64 - px, d.y as i64 - py);
                    parts.push(format!("there's {} {}", d.label.to_lowercase(), dir));
                }
            }
        }
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.01);
        if parts.is_empty() {
            self.status_msg = Some(format!(
                "{pname}: \"You're as far as the road goes hereabouts.\""
            ));
            return;
        }
        let told = parts.join(", and ");
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                format!("Was told the way: {told}."),
            );
        }
        self.status_msg = Some(format!("{pname} points the way: {told}."));
    }

    /// Ask an NPC who is worth knowing here (#528 conversations): a local names
    /// the folk a stranger would want to find — a gifted crafter, a healer, a
    /// smith, a scribe, a trader — so the tradespeople you can deal with are
    /// findable by asking, not only by knocking on every door. Regard-gated.
    pub fn ask_folk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        let Some((people, pname)) = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| (PeopleKind::from_name(&p.people), p.name.clone()))
        }) else {
            return;
        };
        let mut regard = self.inter_people_bias.effective_bias(people);
        if let Some(god) = people.patron_god() {
            if self.god_affinity.get(god) > 0.4 {
                regard += 0.05;
            }
        }
        if regard < -0.15 {
            self.status_msg = Some(format!(
                "{pname} names no one — your kind learn nothing here."
            ));
            return;
        }
        let mut notable: Vec<String> = Vec::new();
        if let Some(ref sim) = self.sim {
            if let Some(s) = sim
                .world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
            {
                // A gifted crafter is the first name on anyone's lips.
                if let Some(g) = s.people.iter().find(|p| p.gift.has()) {
                    if let Some(sense) = g.gift.sense() {
                        notable.push(format!("{}, who has the {} gift", g.name, sense.name()));
                    }
                }
                // The tradespeople a stranger would seek — one of each, named by
                // their trade (the dealings of #527 hang on finding them).
                let trades = [
                    ("smith", "the smith"),
                    ("healer", "the healer"),
                    ("herbalist", "the herbalist"),
                    ("scribe", "the scribe"),
                    ("trader", "the trader"),
                    ("path-finder", "the path-finder"),
                ];
                for (prof, title) in trades {
                    if notable.len() >= 4 {
                        break;
                    }
                    if let Some(p) = s.people.iter().find(|p| {
                        p.profession == prof && !notable.iter().any(|n| n.contains(&p.name))
                    }) {
                        notable.push(format!("{title} {}", p.name));
                    }
                }
            }
        }
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.01);
        if notable.is_empty() {
            self.status_msg = Some(format!(
                "{pname}: \"Plain folk here — no one you'd cross a field to meet.\""
            ));
            return;
        }
        let told = notable.join("; ");
        self.status_msg = Some(format!("{pname} tells you who to seek: {told}."));
    }

    /// Commission a Tool from a settlement's smith (#527 tradespeople): bring
    /// the Iron and the fee, and the smith does the rest — no botch, no wasted
    /// stock, the way the player's own bench can fail. A gifted smith (iron-ear)
    /// makes it truer, with an offcut of Nails for the giving. Refused by a smith
    /// set against your kind. Gated to an NPC who actually keeps the forge.
    pub fn commission_smith(
        &mut self,
        region_idx: usize,
        settlement_idx: usize,
        person_idx: usize,
    ) {
        const FEE: u32 = 5;
        let Some((people, pname, is_smith, gifted)) = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| {
                    (
                        PeopleKind::from_name(&p.people),
                        p.name.clone(),
                        p.profession == "smith",
                        p.gift.sense() == Some(crate::model::CraftSense::IronEar),
                    )
                })
        }) else {
            return;
        };
        if !is_smith {
            self.status_msg = Some(format!(
                "{pname} is no smith — the forge is not their trade."
            ));
            return;
        }
        let mut regard = self.inter_people_bias.effective_bias(people);
        if let Some(god) = people.patron_god() {
            if self.god_affinity.get(god) > 0.4 {
                regard += 0.05;
            }
        }
        if regard < -0.10 {
            self.status_msg = Some(format!("{pname} will not strike iron for your kind."));
            return;
        }
        // The player brings the Iron and the fee; the smith brings the skill.
        let has_stock = self.player_start.as_ref().is_some_and(|ps| {
            ps.inventory.has(ItemType::Iron) && ps.inventory.get(ItemType::Coin) >= FEE
        });
        if !has_stock {
            self.status_msg = Some(format!(
                "A commission wants Iron and {FEE} coin in hand; the smith finds the rest."
            ));
            return;
        }
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(ItemType::Iron, 1);
            ps.inventory.remove(ItemType::Coin, FEE);
            ps.inventory.add(ItemType::Tool, 1);
            if gifted {
                ps.inventory.add(ItemType::Nails, 1);
            }
        }
        self.advance_clock(3);
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.02);
        self.god_affinity.adjust(GodName::Oltzed, 0.02);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                if gifted {
                    "The smith's gift showed in the work — a Tool that will not turn in the hand."
                        .to_string()
                } else {
                    "I had a Tool struck at the forge, and kept my own hands clean of the botch."
                        .to_string()
                },
            );
        }
        self.status_msg = Some(if gifted {
            format!("{pname} strikes you a fine Tool (−Iron −{FEE} coin, +Tool +Nails, 3h)")
        } else {
            format!("{pname} strikes you a Tool (−Iron −{FEE} coin, +Tool, 3h)")
        });
    }

    /// Consult a settlement's healer or herbalist for your own sickness (#527
    /// tradespeople): the temple's cure, but out in the villages where there is
    /// no temple — a healer cures what ails you outright, a herbalist eases it
    /// strongly. A root-eye gift mends deeper, leaving you the steadier. Costs a
    /// fee; refused by one set against your kind. Gated to a healer/herbalist.
    pub fn consult_healer(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        let Some((people, pname, is_healer, is_herbalist, root_eye)) =
            self.sim.as_ref().and_then(|sim| {
                sim.world
                    .regions
                    .get(region_idx)
                    .and_then(|r| r.settlements.get(settlement_idx))
                    .and_then(|s| s.people.get(person_idx))
                    .map(|p| {
                        (
                            PeopleKind::from_name(&p.people),
                            p.name.clone(),
                            p.profession == "healer",
                            p.profession == "herbalist",
                            p.gift.sense() == Some(crate::model::CraftSense::RootEye),
                        )
                    })
            })
        else {
            return;
        };
        if !is_healer && !is_herbalist {
            self.status_msg = Some(format!(
                "{pname} is no healer — tending is not their trade."
            ));
            return;
        }
        let mut regard = self.inter_people_bias.effective_bias(people);
        if let Some(god) = people.patron_god() {
            if self.god_affinity.get(god) > 0.4 {
                regard += 0.05;
            }
        }
        if regard < -0.10 {
            self.status_msg = Some(format!("{pname} will not lay hands on your kind."));
            return;
        }
        let sick = self
            .player_start
            .as_ref()
            .is_some_and(|ps| !ps.person.illnesses.is_empty());
        if !sick {
            self.status_msg = Some("You are hale enough — nothing for a healer to tend.".into());
            return;
        }
        // A healer cures outright; a herbalist eases strongly. The healer's work
        // costs the more.
        let fee: u32 = if is_healer { 10 } else { 5 };
        let can_pay = self
            .player_start
            .as_ref()
            .is_some_and(|ps| ps.inventory.get(ItemType::Coin) >= fee);
        if !can_pay {
            self.status_msg = Some(format!(
                "The healer's fee is {fee} coin; you cannot meet it."
            ));
            return;
        }
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(ItemType::Coin, fee);
            if is_healer {
                ps.person.illnesses.clear();
            } else {
                for d in ps.person.illnesses.iter_mut() {
                    d.tend_strong();
                }
            }
            if root_eye {
                self.vitals.energy = (self.vitals.energy + 0.2).min(1.0);
            }
        }
        self.advance_clock(2);
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.02);
        self.god_affinity.adjust(GodName::Masa, 0.01);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                if is_healer {
                    "A village healer tended me, and the sickness let go its grip.".to_string()
                } else {
                    "A herbalist eased what ailed me with the country's own physic.".to_string()
                },
            );
        }
        self.status_msg = Some(if is_healer {
            format!("{pname} cures what ails you (−{fee} coin, 2h)")
        } else {
            format!("{pname} eases your sickness with herb-lore (−{fee} coin, 2h)")
        });
    }

    /// Hire a settlement's path-finder or forester to learn the country (#527
    /// tradespeople): for a fee, they draw you the lay of the whole region —
    /// the map opens — and name the bearings to the nearest settlement and to
    /// every notable place you have not yet found. Regard-gated; the trade of
    /// those who walk the land for a living.
    pub fn hire_guide(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        const FEE: u32 = 6;
        let Some((people, pname, is_guide)) = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| {
                    (
                        PeopleKind::from_name(&p.people),
                        p.name.clone(),
                        p.profession == "path-finder" || p.profession == "forester",
                    )
                })
        }) else {
            return;
        };
        if !is_guide {
            self.status_msg = Some(format!(
                "{pname} does not walk the country for a trade — find a path-finder."
            ));
            return;
        }
        let mut regard = self.inter_people_bias.effective_bias(people);
        if let Some(god) = people.patron_god() {
            if self.god_affinity.get(god) > 0.4 {
                regard += 0.05;
            }
        }
        if regard < -0.10 {
            self.status_msg = Some(format!("{pname} will not set your kind on their paths."));
            return;
        }
        if self
            .player_start
            .as_ref()
            .is_none_or(|ps| ps.inventory.get(ItemType::Coin) < FEE)
        {
            self.status_msg = Some(format!("A guide's fee is {FEE} coin; you cannot meet it."));
            return;
        }
        let Some(pos) = self.player_pos else {
            return;
        };
        let (px, py) = (pos.px as i64, pos.py as i64);
        // The whole region opens: reveal it end to end from its middle.
        let (cx, cy, span) = self
            .sim
            .as_ref()
            .and_then(|sim| sim.world.regions.get(region_idx))
            .map(|r| {
                (
                    r.terrain.width / 2,
                    r.terrain.height / 2,
                    r.terrain.width + r.terrain.height,
                )
            })
            .unwrap_or((0, 0, 0));
        if region_idx < self.explored.len() {
            self.explored[region_idx].reveal(cx, cy, span);
        }
        // Bearings to the nearest settlement and every unfound place.
        let mut parts: Vec<String> = Vec::new();
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if let Some((_, s)) = region
                    .settlements
                    .iter()
                    .enumerate()
                    .filter(|(i, s)| *i != settlement_idx && s.population > 0)
                    .min_by_key(|(_, s)| {
                        let (dx, dy) = (s.map_x as i64 - px, s.map_y as i64 - py);
                        dx * dx + dy * dy
                    })
                {
                    parts.push(format!(
                        "{} {}",
                        s.name,
                        compass_bearing(s.map_x as i64 - px, s.map_y as i64 - py)
                    ));
                }
                for d in sim
                    .discoveries
                    .entries
                    .iter()
                    .filter(|d| d.region_idx == region_idx && !d.observed)
                {
                    parts.push(format!(
                        "{} {}",
                        d.label.to_lowercase(),
                        compass_bearing(d.x as i64 - px, d.y as i64 - py)
                    ));
                }
            }
        }
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(ItemType::Coin, FEE);
        }
        self.advance_clock(2);
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.02);
        self.god_affinity.adjust(GodName::Sampsa, 0.01);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                "A guide drew me the whole lay of the land, and the region opened on the map."
                    .to_string(),
            );
        }
        self.status_msg = Some(if parts.is_empty() {
            format!("{pname} draws you the region — the map opens. (−{FEE} coin, 2h)")
        } else {
            format!(
                "{pname} draws you the region (map opens; {}). (−{FEE} coin, 2h)",
                parts.join(", ")
            )
        });
    }

    /// Consult a settlement's scribe for a piece of the wider world (#527
    /// tradespeople): Sampsa's folk deal in knowledge — for a small fee, a
    /// scribe reads you something true of the continent beyond the province (a
    /// canon fact of the far cities, the old roads, the Five), and it is kept in
    /// the journal as learned. The lore-keeper's trade; regard-gated. Stable per
    /// scribe per day.
    pub fn consult_scribe(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        const FEE: u32 = 4;
        let Some((people, pname, pid, is_scribe)) = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| {
                    (
                        PeopleKind::from_name(&p.people),
                        p.name.clone(),
                        p.id.clone(),
                        p.profession == "scribe",
                    )
                })
        }) else {
            return;
        };
        if !is_scribe {
            self.status_msg = Some(format!(
                "{pname} keeps no records — find a scribe for lore."
            ));
            return;
        }
        let mut regard = self.inter_people_bias.effective_bias(people);
        if let Some(god) = people.patron_god() {
            if self.god_affinity.get(god) > 0.4 {
                regard += 0.05;
            }
        }
        if regard < -0.10 {
            self.status_msg = Some(format!("{pname} closes the book — not for your kind."));
            return;
        }
        if self
            .player_start
            .as_ref()
            .is_none_or(|ps| ps.inventory.get(ItemType::Coin) < FEE)
        {
            self.status_msg = Some(format!(
                "The scribe's fee is {FEE} coin; you cannot meet it."
            ));
            return;
        }
        let lore = {
            let bank = crate::banks::bank("CANON_RUMORS");
            if bank.is_empty() {
                self.status_msg = Some(format!("{pname} finds nothing new in the records today."));
                return;
            }
            let day = self.clock.day;
            let salt = crate::rng::mix_u64(
                pid.bytes()
                    .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(b as u64))
                    ^ (day as u64).wrapping_shl(32),
            );
            bank[(salt % bank.len() as u64) as usize].clone()
        };
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(ItemType::Coin, FEE);
        }
        self.advance_clock(1);
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.02);
        self.god_affinity.adjust(GodName::Sampsa, 0.02);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(tick, crate::sim::journal::Voice::Dream, lore.clone());
        }
        self.status_msg = Some(format!(
            "{pname} reads you from the records: \"{lore}\" (−{FEE} coin, 1h)"
        ));
    }

    pub fn adopt_companion(&mut self, region_idx: usize, settlement_idx: usize) {
        let ps = match self.player_start {
            Some(ref mut ps) => ps,
            None => {
                self.status_msg = Some("No character yet.".into());
                return;
            }
        };
        if ps.companions.len() >= 3 {
            self.status_msg = Some("I cannot take another companion. My hands are full.".into());
            return;
        }
        let (capacity, settlement_name) = if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if let Some(settlement) = region.settlements.get(settlement_idx) {
                    if !settlement.allows_companions() {
                        self.status_msg = Some(
                            "This place is too small for stable animals. No companions here."
                                .into(),
                        );
                        return;
                    }
                    (settlement.companion_capacity(), settlement.name.clone())
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        };
        let animal_rng_seed = self
            .seed
            .wrapping_add(region_idx as u64 * 997)
            .wrapping_add(settlement_idx as u64 * 31);
        let mut rng = SeedRng::new(animal_rng_seed);
        let available = settlement_companions(&mut rng, capacity);
        if available.is_empty() {
            self.status_msg = Some("No companions available.".into());
            return;
        }
        let companion = available.into_iter().next().unwrap();
        if self
            .player_start
            .as_ref()
            .is_some_and(|ps| ps.has_companion_kind(companion.animal))
        {
            self.status_msg = Some(format!(
                "I already travel with a {}. No need for another.",
                companion.animal.name()
            ));
            return;
        }
        let cost = companion.animal.cost();
        if self
            .player_start
            .as_ref()
            .map_or(0, |ps| ps.inventory.get(ItemType::Coin))
            < cost
        {
            self.status_msg = Some(format!(
                "The {} in {} costs {} coin. I lack that much.",
                companion.animal.name(),
                settlement_name,
                cost
            ));
            return;
        }
        let _ = self
            .player_start
            .as_mut()
            .unwrap()
            .inventory
            .remove(ItemType::Coin, cost);
        let name = companion.name.clone();
        let animal = companion.animal;
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        self.player_start
            .as_mut()
            .unwrap()
            .adopt_companion(animal, name.clone(), tick);
        if let Some(ref mut sim) = self.sim {
            sim.log_journal(
                tick,
                format!(
                    "A {} called {} joins me from the yards of {}. My road grows less lonely.",
                    animal.name(),
                    name,
                    settlement_name
                ),
            );
        }
        self.status_msg = Some(format!("{} the {} joins me.", name, animal.name()));
    }

    /// Court a person of the settlement the player stands in (#362). The
    /// oath is ordinary — no chosen ones: it asks trust earned over time, a
    /// family that doesn't bar the door, a standing the town can live with,
    /// a roof, and a feast the hall will remember.
    pub fn court(&mut self, person_idx: usize) {
        let Some((ri, si)) = self.player_on_settlement() else {
            self.status_msg = Some("Courting is done where people live.".into());
            return;
        };
        if self.spouse_id.is_some() {
            self.status_msg = Some("You are wed already.".into());
            return;
        }
        if self.widowed_day > 0 && self.clock.day < self.widowed_day + 30 {
            self.status_msg = Some("The grief is too new. The house is not ready.".into());
            return;
        }
        let Some(person) = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(ri))
            .and_then(|r| r.settlements.get(si))
            .and_then(|s| s.people.get(person_idx))
            .cloned()
        else {
            self.status_msg = Some("No such person here.".into());
            return;
        };
        if person.has_spouse {
            self.status_msg = Some(format!("{} is wed.", person.name));
            return;
        }
        let np = crate::model::PeopleKind::from_name(&person.people);
        if self.inter_people_bias.effective_bias(np) < -0.15 {
            self.status_msg = Some(format!(
                "{}'s family turns you from the door. Mend things between your peoples first.",
                person.name
            ));
            return;
        }
        let trust = self
            .sim
            .as_ref()
            .and_then(|s| s.npc_memories.get(&person.id))
            .map(|m| m.cumulative_trust())
            .unwrap_or(0.0);
        if trust < 0.15 {
            self.status_msg = Some(format!(
                "{} knows you only from the road. Time, gifts, and help come first.",
                person.name
            ));
            return;
        }
        if self.reputation_in_current_settlement() < 0.45 {
            self.status_msg =
                Some("The town would not stand witness for you. Earn its regard.".into());
            return;
        }
        if !self.owns_a_home() {
            self.status_msg =
                Some("An oath needs a roof to live under — raise a cabin first.".into());
            return;
        }
        let feast_paid = self
            .player_start
            .as_mut()
            .map(|ps| {
                if ps.inventory.get(ItemType::Food) >= 20 && ps.inventory.get(ItemType::Coin) >= 10
                {
                    ps.inventory.remove(ItemType::Food, 20);
                    ps.inventory.remove(ItemType::Coin, 10);
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false);
        if !feast_paid {
            self.status_msg =
                Some("A wedding wants a feast: 20 food and 10 coins for the hall.".into());
            return;
        }
        // The oath is spoken.
        self.spouse_id = Some(person.id.clone());
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let (settlement_id, settlement_name) = {
            let s = &self.sim.as_ref().unwrap().world.regions[ri].settlements[si];
            (s.id.clone(), s.name.clone())
        };
        if let Some(ref mut sim) = self.sim {
            if let Some(p) = sim
                .world
                .regions
                .get_mut(ri)
                .and_then(|r| r.settlements.get_mut(si))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                p.has_spouse = true;
            }
            sim.reputation
                .adjust_settlement(&player_id, &settlement_id, 0.10);
            let t = sim.world.tick;
            sim.log(
                t,
                crate::sim::journal::Voice::Scar,
                format!(
                    "We spoke the oath at {} with the town for witness, and the hall ate                      well. {} keeps my house with me now.",
                    settlement_name, person.name
                ),
            );
        }
        // The oath is Masa's (contract kept); the feast is Keuru's table.
        self.god_affinity.adjust(GodName::Masa, 0.05);
        self.god_affinity.adjust(GodName::Keuru, 0.05);
        // Marriage is the oldest diplomacy between peoples.
        if np != self.inter_people_bias.player_people {
            self.inter_people_bias.mod_toward(np, 0.10);
        }
        if let Some(ref mut ps) = self.player_start {
            ps.person.has_spouse = true;
        }
        self.advance_clock(6);
        self.status_msg = Some(format!(
            "Wed to {} — the oath spoken, the hall fed (6h).",
            person.name
        ));
    }

    /// Once a day, look in on the marriage: if the spouse has gone from the
    /// world (lifecycle deaths reach spouses too), the player is widowed —
    /// grief in the journal and a closed door for a season.
    pub(super) fn check_spouse(&mut self) {
        let Some(spouse_id) = self.spouse_id.clone() else {
            return;
        };
        let alive = self
            .sim
            .as_ref()
            .map(|s| {
                s.world
                    .regions
                    .iter()
                    .flat_map(|r| r.settlements.iter())
                    .flat_map(|s| s.people.iter())
                    .any(|p| p.id == spouse_id)
            })
            .unwrap_or(true);
        if !alive {
            self.spouse_id = None;
            self.widowed_day = self.clock.day.max(1);
            if let Some(ref mut ps) = self.player_start {
                ps.person.has_spouse = false;
            }
            if let Some(ref mut sim) = self.sim {
                let t = sim.world.tick;
                sim.log(
                    t,
                    crate::sim::journal::Voice::Scar,
                    "The house is quiet in a way it was not. I keep setting two bowls.".into(),
                );
            }
            self.status_msg = Some("Word comes that your spouse has died.".into());
        }
    }

    /// A child's age in life-years on the compressed aging calendar.
    pub(super) fn child_age_years(&self, child: &crate::model::HouseholdChild) -> u32 {
        self.clock.day.saturating_sub(child.born_day) / AGING_DAYS_PER_LIFE_YEAR
    }

    pub(super) fn find_related_npc(&self, dead_person: &crate::model::Person) -> Option<usize> {
        let sim = self.sim.as_ref()?;
        let pos = self.player_pos?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let settlement = region.settlements.first()?;
        let dead_id = &dead_person.id;

        // 0. The player's own bonds come first: the heir is whoever the player
        // dealt with most warmly (NPC memory), if anyone qualifies.
        if let Some(ref sim) = self.sim {
            let mut best: Option<(usize, f64)> = None;
            for (idx, person) in settlement.people.iter().enumerate() {
                if person.id == *dead_id {
                    continue;
                }
                let trust = sim
                    .npc_memories
                    .get(&person.id)
                    .map(|m| m.cumulative_trust())
                    .unwrap_or(0.0);
                if trust >= 0.15 && best.map(|(_, t)| trust > t).unwrap_or(true) {
                    best = Some((idx, trust));
                }
            }
            if let Some((idx, _)) = best {
                return Some(idx);
            }
        }

        // 1. Find person with highest bond to dead character
        let mut best_idx: Option<usize> = None;
        let mut best_strength: f64 = -1.0;
        for (idx, person) in settlement.people.iter().enumerate() {
            if person.id == *dead_id {
                continue;
            }
            if let Some(rel) = sim.relationships.get(dead_id, &person.id) {
                if rel.strength > best_strength {
                    best_strength = rel.strength;
                    best_idx = Some(idx);
                }
            }
            if let Some(rel) = sim.relationships.get(&person.id, dead_id) {
                if rel.strength > best_strength {
                    best_strength = rel.strength;
                    best_idx = Some(idx);
                }
            }
        }
        if best_idx.is_some() {
            return best_idx;
        }

        // 2. Prefer spouse
        if dead_person.has_spouse {
            for (idx, person) in settlement.people.iter().enumerate() {
                if person.id == *dead_id {
                    continue;
                }
                if let Some(rel) = sim.relationships.get(dead_id, &person.id) {
                    if rel.kind == crate::model::RelationshipKind::Spouse {
                        return Some(idx);
                    }
                }
                if let Some(rel) = sim.relationships.get(&person.id, dead_id) {
                    if rel.kind == crate::model::RelationshipKind::Spouse {
                        return Some(idx);
                    }
                }
            }
        }

        // 3. Same people kind
        let dead_people_kind = dead_person.people.as_str();
        for (idx, person) in settlement.people.iter().enumerate() {
            if person.id == *dead_id {
                continue;
            }
            if person.people == dead_people_kind {
                return Some(idx);
            }
        }

        // 4. First adult in settlement (age_band != "child")
        for (idx, person) in settlement.people.iter().enumerate() {
            if person.id == *dead_id {
                continue;
            }
            if person.age_band != "child" {
                return Some(idx);
            }
        }

        // 5. Any person
        settlement.people.iter().position(|p| p.id != *dead_id)
    }

    pub fn critical_need_people(&self) -> Vec<(String, String, String, Need, f64)> {
        let mut out = Vec::new();
        if let Some(ref sim) = self.sim {
            let needs = [
                Need::Food,
                Need::Money,
                Need::Care,
                Need::Presence,
                Need::Safety,
            ];
            for region in &sim.world.regions {
                for settlement in &region.settlements {
                    for person in &settlement.people {
                        for need in &needs {
                            let val = person.needs.get(*need);
                            if val < 0.2 {
                                out.push((
                                    person.name.clone(),
                                    person.settlement.clone(),
                                    person.profession.clone(),
                                    *need,
                                    val,
                                ));
                            }
                        }
                    }
                }
            }
        }
        out.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::compass_bearing;

    #[test]
    fn compass_bearing_names_the_cardinals() {
        assert_eq!(compass_bearing(0, -5), "to the north");
        assert_eq!(compass_bearing(0, 5), "to the south");
        assert_eq!(compass_bearing(5, 0), "to the east");
        assert_eq!(compass_bearing(-5, 0), "to the west");
    }

    #[test]
    fn compass_bearing_names_the_diagonals() {
        assert_eq!(compass_bearing(4, -4), "to the northeast");
        assert_eq!(compass_bearing(-4, -4), "to the northwest");
        assert_eq!(compass_bearing(4, 4), "to the southeast");
        assert_eq!(compass_bearing(-4, 4), "to the southwest");
    }

    #[test]
    fn a_dominant_axis_reads_cardinal_not_diagonal() {
        // Far east, barely north → just "east".
        assert_eq!(compass_bearing(10, -1), "to the east");
        assert_eq!(compass_bearing(0, 0), "right here");
    }
}
