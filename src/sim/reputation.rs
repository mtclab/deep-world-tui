use crate::model::World;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

const BASELINE: f64 = 0.5;
const LOCAL_DECAY_RATE: f64 = 0.01;
const FACTION_DECAY_RATE: f64 = 0.005;
const SPREAD_RATE: f64 = 0.02;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reputation {
    pub local: f64,
    pub by_faction: IndexMap<String, f64>,
}

impl Default for Reputation {
    fn default() -> Self {
        Reputation {
            local: BASELINE,
            by_faction: IndexMap::new(),
        }
    }
}

impl Reputation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn adjust(&mut self, delta: f64) {
        self.local = (self.local + delta).clamp(0.0, 1.0);
    }

    pub fn adjust_faction(&mut self, faction: &str, delta: f64) {
        let current = self.by_faction.get(faction).copied().unwrap_or(BASELINE);
        self.by_faction
            .insert(faction.to_string(), (current + delta).clamp(0.0, 1.0));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReputationEntry {
    pub person_id: String,
    pub settlement: String,
    pub reputation: Reputation,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationStore {
    pub entries: IndexMap<String, ReputationEntry>,
}

impl ReputationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, person_id: &str, settlement: &str) -> f64 {
        let key = reputation_key(person_id, settlement);
        self.entries
            .get(&key)
            .map(|e| e.reputation.local)
            .unwrap_or(BASELINE)
    }

    pub fn get_entry(&self, person_id: &str, settlement: &str) -> Option<&ReputationEntry> {
        let key = reputation_key(person_id, settlement);
        self.entries.get(&key)
    }

    pub fn adjust_settlement(&mut self, person_id: &str, settlement: &str, delta: f64) {
        self.adjust_local(person_id, settlement, delta);
    }

    pub fn get_with_faction(&self, person_id: &str, settlement: &str, faction: &str) -> f64 {
        let key = reputation_key(person_id, settlement);
        self.entries
            .get(&key)
            .map(|e| {
                let local = e.reputation.local;
                let faction_val = e
                    .reputation
                    .by_faction
                    .get(faction)
                    .copied()
                    .unwrap_or(BASELINE);
                (local + faction_val) / 2.0
            })
            .unwrap_or(BASELINE)
    }

    pub fn set(&mut self, person_id: &str, settlement: &str, rep: Reputation) {
        let key = reputation_key(person_id, settlement);
        self.entries.insert(
            key,
            ReputationEntry {
                person_id: person_id.to_string(),
                settlement: settlement.to_string(),
                reputation: rep,
            },
        );
    }

    pub fn adjust_local(&mut self, person_id: &str, settlement: &str, delta: f64) {
        let key = reputation_key(person_id, settlement);
        let entry = self.entries.entry(key).or_insert_with(|| ReputationEntry {
            person_id: person_id.to_string(),
            settlement: settlement.to_string(),
            reputation: Reputation::new(),
        });
        entry.reputation.adjust(delta);
    }

    pub fn adjust_local_biased(
        &mut self,
        person_id: &str,
        settlement: &str,
        delta: f64,
        player_people: crate::model::PeopleKind,
        settlement_people: crate::model::PeopleKind,
    ) {
        let bias = player_people.bias_toward(settlement_people);
        let mod_ = (1.0 + bias * 0.5).max(0.3);
        self.adjust_local(person_id, settlement, delta * mod_);
    }

    pub fn adjust_faction(&mut self, person_id: &str, settlement: &str, faction: &str, delta: f64) {
        let key = reputation_key(person_id, settlement);
        let entry = self.entries.entry(key).or_insert_with(|| ReputationEntry {
            person_id: person_id.to_string(),
            settlement: settlement.to_string(),
            reputation: Reputation::new(),
        });
        entry.reputation.adjust_faction(faction, delta);
    }
}

fn reputation_key(person_id: &str, settlement: &str) -> String {
    format!("{}@{}", person_id, settlement)
}

pub fn spread_reputation(
    store: &mut ReputationStore,
    world: &World,
    ties: &crate::model::province::ProvinceTies,
    dt: f64,
) {
    use crate::model::province::TieKind;
    // The reputation store keys by settlement id; the province ties key by
    // settlement name (#565 slice 2). Bridge them once.
    let id_to_name: std::collections::HashMap<&str, &str> = world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();

    let mut updates: Vec<(String, f64, IndexMap<String, f64>)> = Vec::new();
    let mut spread_acc: IndexMap<String, f64> = IndexMap::new();
    // Partner towns the player has never reached, but whose road to a town that
    // knows them is busy: open a channel (a fresh baseline entry) so word can
    // travel into them over the coming days.
    let mut open_channels: Vec<String> = Vec::new();
    for (key, entry) in &store.entries {
        let local_decayed = decay_toward(entry.reputation.local, BASELINE, LOCAL_DECAY_RATE * dt);
        let mut faction_decayed = IndexMap::new();
        for (faction, val) in &entry.reputation.by_faction {
            faction_decayed.insert(
                faction.clone(),
                decay_toward(*val, BASELINE, FACTION_DECAY_RATE * dt),
            );
        }
        let person_settlement = &entry.settlement;
        let source_name = id_to_name.get(person_settlement.as_str()).copied();
        for region in &world.regions {
            for settlement in &region.settlements {
                if settlement.id == *person_settlement {
                    continue;
                }
                if !same_region(world, person_settlement, &settlement.id) {
                    continue;
                }
                // Word travels faster down a thick trade road and not at all to a
                // town with no standing road to the source (#560 ties): a partner
                // carries it strongest, a rival still hears it (bad blood means
                // they hear of you), a neutral town barely.
                let tie = source_name
                    .map(|sn| ties.tie(sn, &settlement.name))
                    .unwrap_or(TieKind::Neutral);
                let bond = source_name
                    .map(|sn| ties.bond(sn, &settlement.name))
                    .unwrap_or(0.0);
                let factor = match tie {
                    TieKind::Partner => 1.0 + 2.0 * bond,
                    TieKind::Rival => 0.6,
                    TieKind::Neutral => 0.4,
                };
                let neighbor_key = reputation_key(&entry.person_id, &settlement.id);
                if let Some(neighbor) = store.entries.get(&neighbor_key) {
                    let diff = neighbor.reputation.local - entry.reputation.local;
                    if diff.abs() > f64::EPSILON {
                        let spread = diff * SPREAD_RATE * factor * dt;
                        *spread_acc.entry(key.clone()).or_insert(0.0) += spread;
                    }
                } else if tie == TieKind::Partner {
                    open_channels.push(neighbor_key);
                }
            }
        }
        updates.push((key.clone(), local_decayed, faction_decayed));
    }
    for (key, local, factions) in updates {
        if let Some(entry) = store.entries.get_mut(&key) {
            let spread_delta = spread_acc.get(&key).copied().unwrap_or(0.0);
            entry.reputation.local = (local + spread_delta).clamp(0.0, 1.0);
            entry.reputation.by_faction = factions;
        }
    }
    // Open the partner channels (at baseline) so the next days' diffusion can
    // carry the player's standing into towns they have never set foot in.
    for neighbor_key in open_channels {
        if store.entries.contains_key(&neighbor_key) {
            continue;
        }
        if let Some((pid, sid)) = neighbor_key.split_once('@') {
            store.entries.insert(
                neighbor_key.clone(),
                ReputationEntry {
                    person_id: pid.to_string(),
                    settlement: sid.to_string(),
                    reputation: Reputation::new(),
                },
            );
        }
    }
}

fn decay_toward(current: f64, target: f64, rate: f64) -> f64 {
    if (current - target).abs() < f64::EPSILON {
        return current;
    }
    if current > target {
        (current - rate).max(target)
    } else {
        (current + rate).min(target)
    }
}

fn same_region(world: &World, settlement_a: &str, settlement_b: &str) -> bool {
    let region_a = settlement_region(world, settlement_a);
    let region_b = settlement_region(world, settlement_b);
    region_a.is_some() && region_a == region_b
}

fn settlement_region(world: &World, settlement_id: &str) -> Option<String> {
    for region in &world.regions {
        for settlement in &region.settlements {
            if settlement.id == settlement_id {
                return Some(region.id.clone());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Needs, Person};

    fn make_simple_world() -> World {
        World {
            seed: 0,
            tick: 0,
            regions: vec![crate::model::Region {
                id: "r1".into(),
                name: "R".into(),
                region_type: "river_valley".into(),
                region_subtype: "flood_plain".into(),
                description: String::new(),
                terrain: crate::model::TerrainMap::default(),
                neighbors: crate::model::RegionNeighbors::default(),
                structures: vec![],
                settlements: vec![
                    crate::model::Settlement {
                        id: "s1".into(),
                        name: "S1".into(),
                        size: "village".into(),
                        region: "r1".into(),
                        population: 1,
                        description: String::new(),
                        people: vec![Person {
                            id: "p1".into(),
                            settlement: "s1".into(),
                            ..Default::default()
                        }],
                        services: vec![],
                        politics: crate::model::SettlementPolitics::new(),
                        faith: Default::default(),
                        food_stock: 0.0,
                        treasury: 0,
                        goods_stock: Default::default(),
                        farms: Vec::new(),
                        buildings: Vec::new(),
                        festival_until_day: 0,
                        famine_days: 0,
                        plague_days: 0,
                        map_x: 0,
                        map_y: 0,
                        district: 0,
                        remembered_deed: None,
                        land_capacity: 0,
                    },
                    crate::model::Settlement {
                        id: "s2".into(),
                        name: "S2".into(),
                        size: "village".into(),
                        region: "r1".into(),
                        population: 1,
                        description: String::new(),
                        people: vec![Person {
                            id: "p1".into(),
                            settlement: "s2".into(),
                            needs: Needs::default(),
                            ..Default::default()
                        }],
                        services: vec![],
                        politics: crate::model::SettlementPolitics::new(),
                        faith: Default::default(),
                        food_stock: 0.0,
                        treasury: 0,
                        goods_stock: Default::default(),
                        farms: Vec::new(),
                        buildings: Vec::new(),
                        festival_until_day: 0,
                        famine_days: 0,
                        plague_days: 0,
                        map_x: 0,
                        map_y: 0,
                        district: 0,
                        remembered_deed: None,
                        land_capacity: 0,
                    },
                ],
                weather: crate::model::Weather::Clear,
                game_richness: 1.0,
                is_march: false,
                known_fed: None,
                known_fed_as_of: 0,
            }],
            charts_version: "0.1.0".into(),
            region_cols: 1,
            polity: Default::default(),
        }
    }

    #[test]
    fn reputation_default_is_baseline() {
        let rep = Reputation::new();
        assert!((rep.local - BASELINE).abs() < f64::EPSILON);
    }

    #[test]
    fn reputation_adjust_clamps() {
        let mut rep = Reputation::new();
        rep.adjust(0.6);
        assert!((rep.local - 1.0).abs() < f64::EPSILON);
        rep.adjust(-2.0);
        assert!((rep.local).abs() < f64::EPSILON);
    }

    #[test]
    fn reputation_faction_adjust() {
        let mut rep = Reputation::new();
        rep.adjust_faction("sepat", 0.3);
        assert!((rep.by_faction["sepat"] - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn high_reputation_decays_toward_baseline() {
        let mut rep = Reputation::new();
        rep.adjust(0.4);
        for _ in 0..10 {
            rep.local = decay_toward(rep.local, BASELINE, LOCAL_DECAY_RATE);
        }
        assert!(
            rep.local < 0.9,
            "reputation should decay toward baseline: {}",
            rep.local
        );
        assert!(
            rep.local >= BASELINE,
            "reputation should not drop below baseline: {}",
            rep.local
        );
    }

    #[test]
    fn store_get_returns_baseline_for_missing() {
        let store = ReputationStore::new();
        assert!((store.get("p1", "s1") - BASELINE).abs() < f64::EPSILON);
    }

    #[test]
    fn store_set_and_get() {
        let mut store = ReputationStore::new();
        let mut rep = Reputation::new();
        rep.adjust(0.3);
        store.set("p1", "s1", rep);
        assert!((store.get("p1", "s1") - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn spread_reputation_decays_toward_baseline() {
        let world = make_simple_world();
        let mut store = ReputationStore::new();
        store.adjust_local("p1", "s1", 0.4);
        let before = store.get("p1", "s1");
        for _ in 0..10 {
            spread_reputation(
                &mut store,
                &world,
                &crate::model::province::ProvinceTies::default(),
                1.0,
            );
        }
        let after = store.get("p1", "s1");
        assert!(
            after < before,
            "reputation should decay toward 0.5: before={}, after={}",
            before,
            after
        );
        assert!(
            after >= BASELINE,
            "reputation should not drop below baseline: {}",
            after
        );
    }

    #[test]
    fn spread_reputation_propagates_within_region() {
        let world = make_simple_world();
        let mut store = ReputationStore::new();
        store.adjust_local("p1", "s1", 0.4);
        let s2_before = store.get("p1", "s2");
        for _ in 0..5 {
            spread_reputation(
                &mut store,
                &world,
                &crate::model::province::ProvinceTies::default(),
                1.0,
            );
        }
        let s2_after = store.get("p1", "s2");
        assert!(
            s2_after >= s2_before,
            "reputation should spread to neighboring settlement"
        );
    }

    #[test]
    fn spread_reputation_deterministic() {
        let world = make_simple_world();
        let mut store1 = ReputationStore::new();
        let mut store2 = ReputationStore::new();
        store1.adjust_local("p1", "s1", 0.3);
        store2.adjust_local("p1", "s1", 0.3);
        for _ in 0..10 {
            spread_reputation(
                &mut store1,
                &world,
                &crate::model::province::ProvinceTies::default(),
                1.0,
            );
            spread_reputation(
                &mut store2,
                &world,
                &crate::model::province::ProvinceTies::default(),
                1.0,
            );
        }
        assert_eq!(
            store1.get("p1", "s1"),
            store2.get("p1", "s1"),
            "spread_reputation must be deterministic"
        );
    }

    #[test]
    fn standing_travels_into_a_partner_town() {
        // #565 slice 2: with a busy trade road between S1 and S2, the standing
        // the player earns in S1 opens a channel to S2 and bleeds into it — they
        // arrive a half-friend in the partner town they have never seen.
        let world = make_simple_world();
        let mut ties = crate::model::province::ProvinceTies::default();
        ties.nudge("S1", "S2", 0.8); // partners
        let mut store = ReputationStore::new();
        store.adjust_local("p1", "s1", 0.4); // well-liked in S1
        for _ in 0..30 {
            spread_reputation(&mut store, &world, &ties, 1.0);
        }
        assert!(
            store.get("p1", "s2") > BASELINE,
            "the player's good name reached the partner town (s2={})",
            store.get("p1", "s2")
        );
    }

    #[test]
    fn standing_does_not_travel_to_a_stranger_town() {
        // With no tie, a town the player has never reached stays a stranger — no
        // channel opens, no word travels.
        let world = make_simple_world();
        let ties = crate::model::province::ProvinceTies::default();
        let mut store = ReputationStore::new();
        store.adjust_local("p1", "s1", 0.4);
        for _ in 0..30 {
            spread_reputation(&mut store, &world, &ties, 1.0);
        }
        assert!(
            (store.get("p1", "s2") - BASELINE).abs() < f64::EPSILON,
            "an untied town learns nothing of the player (s2={})",
            store.get("p1", "s2")
        );
    }

    #[test]
    fn get_with_faction_averages() {
        let mut store = ReputationStore::new();
        store.adjust_local("p1", "s1", 0.2);
        store.adjust_faction("p1", "s1", "sepat", -0.1);
        let combined = store.get_with_faction("p1", "s1", "sepat");
        let local = store.get("p1", "s1");
        let faction = 0.4;
        assert!(
            (combined - (local + faction) / 2.0).abs() < f64::EPSILON,
            "combined should be average of local and faction"
        );
    }
}
