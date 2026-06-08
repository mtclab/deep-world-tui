use crate::charts::Charts;
use crate::gen::world::generate_world;
use crate::model::{Need, World};
use crate::rng::SeedRng;

pub mod collapse_log;
pub mod effects;
pub mod god;
pub mod illness;
pub mod migration;
pub mod needs_dependent;
pub mod params;
pub mod relationships;
pub mod reputation;
pub mod rest;
pub mod signals;
pub mod weather;

use effects::{EffectContext, EffectQueue};
pub use params::SimParams;
use relationships::RelationshipTracker;
use reputation::ReputationStore;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JournalEntry {
    pub tick: u64,
    pub text: String,
}

const MAX_JOURNAL: usize = 200;

pub fn tick_needs_with_params(world: &mut World, dt: f64, params: &SimParams) {
    let rates: [(Need, f64); 5] = [
        (Need::Food, params.food_decay_rate),
        (Need::Money, params.money_decay_rate),
        (Need::Care, params.care_decay_rate),
        (Need::Presence, params.presence_decay_rate),
        (Need::Safety, params.safety_decay_rate),
    ];
    for region in &mut world.regions {
        for settlement in &mut region.settlements {
            for person in &mut settlement.people {
                for (need, rate) in &rates {
                    person.needs.decay(*need, rate * dt);
                }
            }
        }
    }
}

pub fn tick_needs(world: &mut World, dt: f64) {
    tick_needs_with_params(world, dt, &SimParams::default());
}

pub fn tick(world: &mut World) {
    tick_needs(world, 1.0);
    world.tick += 1;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimState {
    pub world: World,
    pub effect_queue: EffectQueue,
    pub relationships: RelationshipTracker,
    pub reputation: ReputationStore,
    pub obligations: Vec<needs_dependent::Obligation>,
    pub charts: Charts,
    pub journal: Vec<JournalEntry>,
    #[serde(default = "SimParams::default")]
    pub params: SimParams,
    #[serde(default)]
    pub npc_memories: std::collections::HashMap<String, crate::model::NpcMemory>,
    #[serde(default)]
    pub quests: Vec<crate::model::Quest>,
    #[serde(default)]
    pub discoveries: crate::model::DiscoveryStore,
}

impl SimState {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let world = generate_world(seed, &charts);
        let mut discoveries = crate::model::DiscoveryStore::new();
        {
            let mut rng = SeedRng::new(seed).fork_for("discoveries");
            for (ri, region) in world.regions.iter().enumerate() {
                let w = region.terrain.width.max(1);
                let h = region.terrain.height.max(1);
                let region_discs =
                    crate::model::discovery::generate_region_discoveries(&mut rng, ri, w, h);
                discoveries.entries.extend(region_discs);
            }
        }
        SimState {
            world,
            effect_queue: EffectQueue::new(),
            relationships: RelationshipTracker::new(),
            reputation: ReputationStore::new(),
            obligations: Vec::new(),
            charts,
            journal: Vec::new(),
            params: SimParams::default(),
            npc_memories: std::collections::HashMap::new(),
            quests: Vec::new(),
            discoveries,
        }
    }

    pub fn step(&mut self) {
        sim_tick(self);
    }

    pub fn log_journal(&mut self, tick: u64, text: String) {
        if self.journal.len() >= MAX_JOURNAL {
            self.journal.remove(0);
        }
        self.journal.push(JournalEntry { tick, text });
    }
}

pub fn sim_tick(sim: &mut SimState) {
    sim.world.tick += 1;
    let current_tick = sim.world.tick;
    let due = sim.effect_queue.due(current_tick);
    let descs: Vec<String> = due
        .iter()
        .map(|e| match e {
            effects::Effect::Immediate { description, .. } => description.clone(),
            effects::Effect::Deferred { description, .. } => description.clone(),
        })
        .filter(|d| !d.is_empty())
        .collect();
    {
        let mut ctx = EffectContext {
            world: &mut sim.world,
            relationships: &mut sim.relationships,
            reputation: &mut sim.reputation,
            current_tick,
        };
        for effect in &due {
            effects::apply_effect(&mut ctx, effect);
        }
    }
    for desc in descs {
        sim.log_journal(current_tick, desc);
    }
    tick_needs_with_params(&mut sim.world, 1.0, &sim.params);
    needs_dependent::propagate_dependent_needs(&mut sim.world, &sim.obligations);
    reputation::spread_reputation(&mut sim.reputation, &sim.world, 1.0);
    sim.relationships.tick_converge(1.0);
    tick_npc_illness(sim, current_tick);
    migration::tick_migration(sim, current_tick);
}

fn tick_npc_illness(sim: &mut SimState, current_tick: u64) {
    use crate::sim::illness;

    let person_info: Vec<(usize, usize)> = sim
        .world
        .regions
        .iter()
        .enumerate()
        .flat_map(|(ri, region)| {
            region
                .settlements
                .iter()
                .enumerate()
                .map(move |(si, _)| (ri, si))
        })
        .collect();

    for (ri, si) in person_info {
        let has_healer = sim
            .world
            .regions
            .get(ri)
            .and_then(|r| r.settlements.get(si))
            .map(illness::settlement_has_healer)
            .unwrap_or(false);
        let terrain = sim
            .world
            .regions
            .get(ri)
            .and_then(|r| r.terrain.get(0, 0))
            .unwrap_or(crate::model::Terrain::Grass);
        let settlement = match sim
            .world
            .regions
            .get_mut(ri)
            .and_then(|r| r.settlements.get_mut(si))
        {
            Some(s) => s,
            None => continue,
        };
        let person_count = settlement.people.len();
        let mut new_illnesses: Vec<(usize, crate::model::ActiveDisease)> = Vec::new();

        for i in 0..person_count {
            illness::apply_illness_effects(&mut settlement.people[i], current_tick);
        }

        let ill_count = settlement
            .people
            .iter()
            .filter(|p| !p.illnesses.is_empty())
            .count();
        let cap = (settlement.people.len().max(1) * 30 / 100).max(1);

        if ill_count >= cap {
            continue;
        }

        for i in 0..person_count {
            let person = &settlement.people[i];
            if person.illnesses.len() >= 2 {
                continue;
            }
            let person_id_bytes = person.id.as_bytes();
            let mut seed_val: u64 = sim.world.seed;
            for &b in person_id_bytes.iter().take(8) {
                seed_val = seed_val.wrapping_shl(8).wrapping_add(b as u64);
            }
            seed_val = seed_val.wrapping_add(current_tick);
            let existing = person.illnesses.len();
            if let Some(disease) = illness::tick_illness(
                seed_val,
                current_tick,
                terrain,
                &person.needs,
                0,
                has_healer,
                existing,
            ) {
                new_illnesses.push((i, disease));
            }
        }

        for (i, disease) in new_illnesses {
            if settlement
                .people
                .iter()
                .filter(|p| !p.illnesses.is_empty())
                .count()
                < cap
            {
                settlement.people[i].illnesses.push(disease);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::gen::world::generate_world;

    #[test]
    fn tick_needs_food_highest_decay() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        let food_before = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let safety_before = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Safety);
        tick_needs(&mut world, 1.0);
        let food_after = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let safety_after = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Safety);
        let food_drop = food_before - food_after;
        let safety_drop = safety_before - safety_after;
        assert!(
            food_drop > safety_drop,
            "food decay ({:.4}) should exceed safety ({:.4})",
            food_drop,
            safety_drop
        );
    }

    #[test]
    fn tick_needs_exact_values() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    person.needs = crate::model::Needs::default();
                }
            }
        }
        let params = SimParams::default();
        tick_needs_with_params(&mut world, 1.0, &params);
        let p = &world.regions[0].settlements[0].people[0];
        assert!(
            (p.needs.get(Need::Food) - (0.8 - params.food_decay_rate)).abs() < f64::EPSILON,
            "food after 1 tick: expected {}, got {}",
            0.8 - params.food_decay_rate,
            p.needs.get(Need::Food)
        );
        assert!(
            (p.needs.get(Need::Safety) - (0.8 - params.safety_decay_rate)).abs() < f64::EPSILON,
            "safety after 1 tick: expected {}, got {}",
            0.8 - params.safety_decay_rate,
            p.needs.get(Need::Safety)
        );
    }

    #[test]
    fn tick_needs_clamped_at_zero() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        for _ in 0..200 {
            tick_needs(&mut world, 1.0);
        }
        for region in &world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    for need in &[
                        Need::Food,
                        Need::Money,
                        Need::Care,
                        Need::Presence,
                        Need::Safety,
                    ] {
                        assert!(
                            person.needs.get(*need) >= 0.0,
                            "{} went negative: {}",
                            need,
                            person.needs.get(*need)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn tick_needs_10_ticks_food_below_half() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    person.needs = crate::model::Needs::default();
                }
            }
        }
        for _ in 0..10 {
            tick_needs(&mut world, 1.0);
        }
        for region in &world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    let food = person.needs.get(Need::Food);
                    let safety = person.needs.get(Need::Safety);
                    assert!(food < 0.5, "food after 10 ticks: {} (expect < 0.5)", food);
                    assert!(
                        safety > 0.5,
                        "safety after 10 ticks: {} (expect > 0.5)",
                        safety
                    );
                }
            }
        }
    }

    #[test]
    fn tick_deterministic() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut a = generate_world(42, &charts);
        let mut b = generate_world(42, &charts);
        for _ in 0..5 {
            tick_needs(&mut a, 1.0);
            tick_needs(&mut b, 1.0);
        }
        let pa = &a.regions[0].settlements[0].people[0];
        let pb = &b.regions[0].settlements[0].people[0];
        assert_eq!(pa.needs, pb.needs, "tick_needs must be deterministic");
    }

    #[test]
    fn tick_increments_tick_counter() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        assert_eq!(world.tick, 0);
        tick(&mut world);
        assert_eq!(world.tick, 1);
        tick(&mut world);
        assert_eq!(world.tick, 2);
    }

    fn make_sim(seed: u64) -> SimState {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        SimState::new(seed, charts)
    }

    #[test]
    fn sim_tick_100_deterministic() {
        let mut a = make_sim(42);
        let mut b = make_sim(42);
        for _ in 0..100 {
            a.step();
            b.step();
        }
        assert_eq!(a.world.tick, b.world.tick);
        let pa = &a.world.regions[0].settlements[0].people[0];
        let pb = &b.world.regions[0].settlements[0].people[0];
        assert_eq!(
            pa.needs, pb.needs,
            "sim needs must be deterministic after 100 ticks"
        );
        assert_eq!(a.world.regions.len(), b.world.regions.len());
    }

    #[test]
    fn sim_tick_advances_time() {
        let mut sim = make_sim(42);
        assert_eq!(sim.world.tick, 0);
        sim.step();
        assert_eq!(sim.world.tick, 1);
        for _ in 0..99 {
            sim.step();
        }
        assert_eq!(sim.world.tick, 100);
    }

    #[test]
    fn sim_tick_needs_decay_over_time() {
        let mut sim = make_sim(42);
        let food_before = sim.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        for _ in 0..10 {
            sim.step();
        }
        let food_after = sim.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            food_after < food_before,
            "food should decay over 10 sim ticks: before={}, after={}",
            food_before,
            food_after
        );
    }

    #[test]
    fn sim_tick_empty_queue_no_panic() {
        let mut sim = make_sim(42);
        for _ in 0..5 {
            sim.step();
        }
    }

    #[test]
    fn sim_tick_fire_scheduled_effect() {
        let mut sim_with = make_sim(42);
        let mut sim_without = make_sim(42);
        sim_with.effect_queue.queue(effects::Effect::deferred(
            "feast",
            3,
            vec![effects::Change::NeedDelta {
                person_id: sim_with.world.regions[0].settlements[0].people[0]
                    .id
                    .clone(),
                need: Need::Food,
                delta: 0.5,
            }],
        ));
        for _ in 0..5 {
            sim_with.step();
            sim_without.step();
        }
        let food_with = sim_with.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let food_without = sim_without.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            food_with > food_without,
            "feast effect should result in higher food: with={}, without={}",
            food_with,
            food_without
        );
    }

    #[test]
    fn sim_state_new_generates_world() {
        let sim = make_sim(42);
        assert_eq!(sim.world.seed, 42);
        assert!(!sim.world.regions.is_empty());
        assert!(sim.effect_queue.is_empty());
    }
}

#[cfg(test)]
mod determinism_tests {
    use super::*;
    use crate::charts;
    use crate::sim::effects::{Change, Effect};

    fn make_sim(seed: u64) -> SimState {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        SimState::new(seed, charts)
    }

    #[test]
    fn same_seed_same_choices_100_ticks_identical() {
        let mut a = make_sim(42);
        let mut b = make_sim(42);
        let person_id = a.world.regions[0].settlements[0].people[0].id.clone();
        for tick in [10u64, 30, 50, 70] {
            let effect = Effect::immediate(
                "fed",
                vec![Change::NeedDelta {
                    person_id: person_id.clone(),
                    need: Need::Food,
                    delta: 0.1,
                }],
            );
            let mut ctx_a = effects::EffectContext {
                world: &mut a.world,
                relationships: &mut a.relationships,
                reputation: &mut a.reputation,
                current_tick: tick,
            };
            let mut ctx_b = effects::EffectContext {
                world: &mut b.world,
                relationships: &mut b.relationships,
                reputation: &mut b.reputation,
                current_tick: tick,
            };
            effects::apply_immediate(&mut ctx_a, &effect);
            effects::apply_immediate(&mut ctx_b, &effect);
            a.step();
            b.step();
        }
        for _ in 0..96 {
            a.step();
            b.step();
        }
        for (ra, rb) in a.world.regions.iter().zip(b.world.regions.iter()) {
            assert_eq!(ra.id, rb.id);
            assert_eq!(ra.settlements.len(), rb.settlements.len());
            for (sa, sb) in ra.settlements.iter().zip(rb.settlements.iter()) {
                assert_eq!(sa.people.len(), sb.people.len());
                for (pa, pb) in sa.people.iter().zip(sb.people.iter()) {
                    assert_eq!(
                        pa.needs, pb.needs,
                        "needs must be identical for person {}",
                        pa.id
                    );
                }
            }
        }
    }

    #[test]
    fn same_seed_deferred_effects_identical_order() {
        let mut a = make_sim(42);
        let mut b = make_sim(42);
        let person_id_a = a.world.regions[0].settlements[0].people[0].id.clone();
        let person_id_b = b.world.regions[0].settlements[0].people[0].id.clone();
        for (tick, delta) in [(5u64, 0.1), (5, 0.05), (10, 0.08), (15, 0.12)] {
            a.effect_queue.queue(Effect::deferred(
                "event",
                tick,
                vec![Change::NeedDelta {
                    person_id: person_id_a.clone(),
                    need: Need::Food,
                    delta,
                }],
            ));
            b.effect_queue.queue(Effect::deferred(
                "event",
                tick,
                vec![Change::NeedDelta {
                    person_id: person_id_b.clone(),
                    need: Need::Food,
                    delta,
                }],
            ));
        }
        for _ in 0..20 {
            a.step();
            b.step();
        }
        let food_a = a.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let food_b = b.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            (food_a - food_b).abs() < f64::EPSILON,
            "deferred effect order must be deterministic: a={}, b={}",
            food_a,
            food_b
        );
    }

    #[test]
    fn different_seed_different_world() {
        let mut a = make_sim(42);
        let mut b = make_sim(99);
        for _ in 0..10 {
            a.step();
            b.step();
        }
        let names_a: Vec<&str> = a.world.regions.iter().map(|r| r.name.as_str()).collect();
        let names_b: Vec<&str> = b.world.regions.iter().map(|r| r.name.as_str()).collect();
        assert_ne!(
            names_a, names_b,
            "different seeds should produce different region names"
        );
    }

    #[test]
    fn same_seed_same_choices_full_sim_identical() {
        let mut a = make_sim(77);
        let mut b = make_sim(77);
        let person_id = a.world.regions[0].settlements[0].people[0].id.clone();
        a.effect_queue.queue(Effect::deferred(
            "late feast",
            50,
            vec![Change::NeedDelta {
                person_id: person_id.clone(),
                need: Need::Food,
                delta: 0.3,
            }],
        ));
        b.effect_queue.queue(Effect::deferred(
            "late feast",
            50,
            vec![Change::NeedDelta {
                person_id: person_id.clone(),
                need: Need::Food,
                delta: 0.3,
            }],
        ));
        for _ in 0..100 {
            a.step();
            b.step();
        }
        assert_eq!(a.world.tick, b.world.tick);
        assert_eq!(a.world.regions.len(), b.world.regions.len());
        let food_a = a.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let food_b = b.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            (food_a - food_b).abs() < f64::EPSILON,
            "full sim determinism: a={}, b={}",
            food_a,
            food_b
        );
    }
}
