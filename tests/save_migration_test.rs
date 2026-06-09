use deep_world_tui::sim::hints::HintTracker;

use deep_world_tui::charts;
use deep_world_tui::model::{
    GameClock, GodAffinity, InterPeopleBias, PeopleKind, PlayerPos, PlayerVitals,
};
use deep_world_tui::save::{self, SaveData};
use deep_world_tui::save_migrations::CURRENT_SAVE_VERSION;
use deep_world_tui::sim::SimState;
use deep_world_tui::sim::milestones::MilestoneTracker;

fn load_charts() -> charts::Charts {
    charts::load_charts("data/charts.ron").expect("charts should load")
}

fn make_save(version: u32) -> SaveData {
    let charts = load_charts();
    SaveData {
        sim: SimState::new(42, charts),
        player_start: None,
        clock: GameClock::default(),
        vitals: PlayerVitals::default(),
        player_pos: None,
        god_affinity: GodAffinity::new(),
        inter_people_bias: InterPeopleBias::new(PeopleKind::Metsik),
        encounters_had: 0,
        collapses_had: 0,
        collapse_log: Vec::new(),
        lineage: Vec::new(),
        milestones: MilestoneTracker::new(),
        version,
        first_run: true,
        hint_tracker: HintTracker::default(),
    }
}

fn cleanup(name: &str) {
    let _ = std::fs::remove_file(format!("saves/{}", name));
}

#[test]
fn current_version_save_load_roundtrip() {
    let data = make_save(CURRENT_SAVE_VERSION);
    save::save_game(&data, "test_current_v.ron").expect("save should succeed");
    let loaded = save::load_game("test_current_v.ron").expect("load should succeed");
    assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    assert_eq!(data.sim.world.tick, loaded.sim.world.tick);
    cleanup("test_current_v.ron");
}

#[test]
fn v0_migrates_to_current_on_load() {
    let data = make_save(0);
    save::save_game(&data, "test_v0_migrate.ron").expect("save should succeed");
    let loaded = save::load_game("test_v0_migrate.ron").expect("load should succeed");
    assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    assert!(loaded.collapse_log.is_empty());
    assert!(loaded.lineage.is_empty());
    cleanup("test_v0_migrate.ron");
}

#[test]
fn future_version_returns_error() {
    let data = make_save(999);
    save::save_game(&data, "test_future_v.ron").expect("save should succeed");
    let result = save::load_game("test_future_v.ron");
    assert!(result.is_err(), "loading future version should fail");
    let err = result.unwrap_err();
    assert!(!err.contains("dummy"));
    cleanup("test_future_v.ron");
}

#[test]
fn roundtrip_preserves_all_fields() {
    let charts = load_charts();
    let data = SaveData {
        sim: SimState::new(42, charts),
        player_start: None,
        clock: GameClock::default(),
        vitals: PlayerVitals::default(),
        player_pos: Some(PlayerPos {
            region_idx: 0,
            px: 20,
            py: 10,
        }),
        god_affinity: GodAffinity::new(),
        inter_people_bias: InterPeopleBias::new(PeopleKind::Metsik),
        encounters_had: 5,
        collapses_had: 2,
        collapse_log: Vec::new(),
        lineage: Vec::new(),
        milestones: MilestoneTracker::new(),
        version: CURRENT_SAVE_VERSION,
        first_run: true,
        hint_tracker: HintTracker::default(),
    };

    save::save_game(&data, "test_roundtrip.ron").expect("save should succeed");
    let loaded = save::load_game("test_roundtrip.ron").expect("load should succeed");
    assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    assert_eq!(loaded.encounters_had, 5);
    assert_eq!(loaded.collapses_had, 2);
    assert!(loaded.player_pos.is_some());
    assert_eq!(data.sim.world.tick, loaded.sim.world.tick);
    cleanup("test_roundtrip.ron");
}
