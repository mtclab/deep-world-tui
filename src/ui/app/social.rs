use crate::gen::companion::settlement_companions;
use crate::model::{EncounterAction, GodName, ItemType, Need, PeopleKind};
use crate::rng::SeedRng;

use super::*;

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
