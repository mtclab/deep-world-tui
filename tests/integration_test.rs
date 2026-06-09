use deep_world_tui::charts;
use deep_world_tui::gen::{person, player, world};
use deep_world_tui::model::{
    GameClock, GodAffinity, InterPeopleBias, Need, PeopleKind, PlayerVitals,
};
use deep_world_tui::rng::SeedRng;
use deep_world_tui::save::{self, SaveData};
use deep_world_tui::save_migrations::CURRENT_SAVE_VERSION;
use deep_world_tui::sim::SimState;
use deep_world_tui::voice::Situation;

/// Full pipeline integration test:
/// seed → charts → world → player → NPC → sim tick → voice → save/load round-trip
fn load_charts() -> charts::Charts {
    charts::load_charts("data/charts.ron").expect("charts should load from data/charts.ron")
}

#[test]
fn full_pipeline_seed_generate_enter_talk() {
    let charts = load_charts();
    let seed: u64 = 42;

    // 1. Generate world from seed
    let world = world::generate_world(seed, &charts);
    assert!(!world.regions.is_empty(), "world should have regions");
    let region = &world.regions[0];
    assert!(
        !region.settlements.is_empty() || world.regions.len() > 1,
        "world should have settlements"
    );

    // 2. Create a player
    let mut rng = SeedRng::new(seed);
    let player_start = player::generate_player_start(&mut rng, &charts);
    assert!(
        !player_start.person.name.is_empty(),
        "player should have a name"
    );
    assert!(
        !player_start.person.people.is_empty(),
        "player should have a people"
    );

    // 3. Create an NPC
    let mut npc_rng = SeedRng::new(seed.wrapping_add(1));
    let npc = person::generate_person(&mut npc_rng, &charts);
    assert!(!npc.name.is_empty(), "NPC should have a name");
    assert!(!npc.people.is_empty(), "NPC should have a people");

    // 4. Verify player and NPC have valid needs
    assert!(
        player_start.person.needs.get(Need::Food) >= 0.0,
        "player food should be >= 0"
    );
    assert!(npc.needs.get(Need::Food) >= 0.0, "NPC food should be >= 0");
    assert!(
        npc.needs.get(Need::Money) >= 0.0,
        "NPC money should be >= 0"
    );

    // 5. Run 10 sim ticks
    let mut sim = SimState::new(seed, charts.clone());
    for i in 0..10 {
        sim.step();
        assert!(
            sim.world.tick == i as u64 + 1,
            "tick should increment: expected {}, got {}",
            i + 1,
            sim.world.tick
        );
    }

    // 6. Generate dialogue via voice.rs
    for sit in [
        Situation::Greeting,
        Situation::Trade,
        Situation::NeedDire,
        Situation::NeedFine,
        Situation::Farewell,
        Situation::Gossip,
    ] {
        let line = deep_world_tui::voice::voice_line_situation(&npc, sit);
        assert!(
            !line.is_empty(),
            "voice line for {:?} should not be empty",
            sit
        );
    }

    // 7. Verify determinism: same seed → same NPC
    let mut rng2 = SeedRng::new(seed.wrapping_add(1));
    let npc2 = person::generate_person(&mut rng2, &charts);
    assert_eq!(
        npc.name, npc2.name,
        "same seed should produce same NPC name"
    );
    assert_eq!(
        npc.people, npc2.people,
        "same seed should produce same NPC people"
    );

    // 8. Save/load round-trip
    let data = SaveData {
        sim: SimState::new(seed, charts.clone()),
        player_start: Some(player_start.clone()),
        clock: GameClock::default(),
        vitals: PlayerVitals::default(),
        player_pos: None,
        god_affinity: GodAffinity::new(),
        inter_people_bias: InterPeopleBias::new(PeopleKind::Metsik),
        encounters_had: 0,
        collapses_had: 0,
        collapse_log: Vec::new(),
        lineage: Vec::new(),
        version: CURRENT_SAVE_VERSION,
    };

    save::save_game(&data, "integration_save.ron").expect("save should succeed");
    let loaded = save::load_game("integration_save.ron").expect("load should succeed");

    assert_eq!(
        data.sim.world.tick, loaded.sim.world.tick,
        "tick should match after round-trip"
    );
    assert_eq!(
        data.sim.world.regions.len(),
        loaded.sim.world.regions.len(),
        "region count should match after round-trip"
    );
    assert_eq!(
        data.god_affinity.oltzed, loaded.god_affinity.oltzed,
        "god affinity should match after round-trip"
    );
}

#[test]
fn pipeline_time_budget() {
    use std::time::Instant;
    let start = Instant::now();

    let charts = load_charts();
    let seed: u64 = 42;
    let mut sim = SimState::new(seed, charts);
    for _ in 0..100 {
        sim.step();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 5,
        "100 sim ticks should complete in <5s, took {:?}",
        elapsed
    );
}
