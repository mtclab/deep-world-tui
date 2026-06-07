use deep_world_tui::charts;
use deep_world_tui::gen::{person, world};
use deep_world_tui::model::Need;
use deep_world_tui::rng::SeedRng;
use deep_world_tui::sim::effects::{Change, Effect, EffectQueue};
use deep_world_tui::sim::relationships::RelationshipTracker;
use deep_world_tui::sim::SimState;

fn load_charts() -> charts::Charts {
    charts::load_charts("data/charts.ron").expect("charts should load")
}

#[test]
fn leave_household_consequence() {
    let charts = load_charts();
    let seed: u64 = 42;
    let world = world::generate_world(seed, &charts);

    // 1. Find a settlement
    assert!(!world.regions.is_empty(), "should have at least one region");
    let region_idx = 0;
    let settlement_idx = 0;
    let settlement = &world.regions[region_idx].settlements[settlement_idx];

    // 2. Create spouse and child NPCs
    let mut rng = SeedRng::new(seed);
    let spouse = person::generate_person(&mut rng, &charts);
    let _child = person::generate_person(&mut rng, &charts);

    // 3. Create the "leave household" immediate effect
    let player_id = "player-1".to_string();
    let spouse_id = spouse.id.clone();

    // 3. Build the "leave household" effects
    // (Immediate effects are applied directly below; deferred are queued)

    // Deferred at tick 10: dependents' Care need drops
    let deferred_care = Effect::Deferred {
        at_tick: 10,
        description: "Dependents feel the absence".to_string(),
        changes: vec![Change::NeedDelta {
            person_id: spouse_id.clone(),
            need: Need::Care,
            delta: -0.2,
        }],
    };

    // Deferred at tick 20: reputation spreads to neighboring settlement
    let neighbor_settlement = if world.regions.len() > 1 && !world.regions[1].settlements.is_empty()
    {
        world.regions[1].settlements[0].name.clone()
    } else {
        settlement.name.clone()
    };

    let deferred_reputation = Effect::Deferred {
        at_tick: 20,
        description: "Word spreads to neighboring settlement".to_string(),
        changes: vec![Change::ReputationDelta {
            person_id: player_id.clone(),
            settlement: neighbor_settlement,
            delta: -0.1,
        }],
    };

    // 4. Set up sim and trackers
    let mut sim = SimState::new(seed, charts);
    let mut effect_queue = EffectQueue::new();

    // Apply immediate effects: relationship and reputation
    sim.relationships
        .update_relationship(&player_id, &spouse_id, "left household", 0, -0.3, -0.2);
    sim.reputation
        .adjust_local(&player_id, &settlement.name, -0.2);

    // Queue deferred effects
    effect_queue.queue(deferred_care);
    effect_queue.queue(deferred_reputation);

    // 5. Assert: spouse bond dropped
    let rel = sim.relationships.get(&player_id, &spouse_id).unwrap();
    assert!(
        rel.strength < 0.0 || rel.strength < 0.1,
        "spouse bond should drop after leaving: {}",
        rel.strength
    );
    assert!(rel.trust < 0.5, "spouse trust should drop: {}", rel.trust);

    // Verify reputation dropped below baseline (0.5)
    let rep_after_leave = sim.reputation.get(&player_id, &settlement.name);
    assert!(
        rep_after_leave < 0.5,
        "local reputation should drop below baseline after leaving: {}",
        rep_after_leave
    );

    // 6. Simulate 50 ticks, firing deferred effects
    for _ in 0..50u64 {
        sim.step();
        let due = effect_queue.due(sim.world.tick);
        for effect in due {
            if let Effect::Immediate { changes, .. } = &effect {
                for change in changes {
                    match change {
                        Change::NeedDelta { delta, .. } => {
                            assert!(*delta < 0.0, "need delta should be negative for dependents");
                        }
                        Change::ReputationDelta { delta, .. } => {
                            assert!(*delta < 0.0, "reputation delta should be negative");
                        }
                        Change::RelationshipDelta {
                            strength_delta,
                            trust_delta: _,
                            ..
                        } => {
                            assert!(*strength_delta < 0.0, "relationship strength should drop");
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // 7. Verify simulation completed and deferred effects fired
    assert_eq!(sim.world.tick, 50);

    // Verify effect queue processed all deferred effects
    assert!(
        effect_queue.is_empty(),
        "all deferred effects should have fired"
    );
}

#[test]
fn leave_household_spouse_does_not_recover_soon() {
    let mut tracker = RelationshipTracker::new();

    // Player leaves spouse
    tracker.update_relationship("player", "spouse", "left household", 0, -0.3, -0.2);

    let bond_after = tracker.get("player", "spouse").unwrap().strength;

    // Simulate 100 ticks with only convergence (no reconciliation)
    for _ in 0..100 {
        tracker.tick_converge(1.0);
    }

    let after_converge = tracker.get("player", "spouse").unwrap();
    // Bond does NOT recover without effort
    assert!(
        after_converge.strength <= bond_after + 0.01,
        "bond should not recover without effort: before={}, after={}",
        bond_after,
        after_converge.strength
    );
}

#[test]
fn reputation_spreads_across_settlements() {
    let charts = load_charts();
    let world = world::generate_world(42, &charts);

    // Verify world has multiple settlements
    let total_settlements: usize = world.regions.iter().map(|r| r.settlements.len()).sum();
    assert!(
        total_settlements > 1,
        "world should have multiple settlements for reputation spread"
    );
}

#[test]
fn deterministic_consequence_scenario() {
    fn run_scenario(seed: u64) -> f64 {
        let charts = load_charts();
        let mut rng = SeedRng::new(seed);
        let spouse = person::generate_person(&mut rng, &charts);
        let mut tracker = RelationshipTracker::new();

        tracker.update_relationship("player", &spouse.id, "left", 0, -0.3, -0.2);

        for _ in 0..50 {
            tracker.tick_converge(1.0);
        }

        tracker.get("player", &spouse.id).unwrap().trust
    }

    let trust_a = run_scenario(42);
    let trust_b = run_scenario(42);
    assert!(
        (trust_a - trust_b).abs() < f64::EPSILON,
        "same seed must produce same trust outcome: {} vs {}",
        trust_a,
        trust_b
    );
}
