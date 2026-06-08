use deep_world_tui::charts;
use deep_world_tui::model::{
    GameClock, GodAffinity, InterPeopleBias, PeopleKind, PlayerPos, PlayerVitals,
};
use deep_world_tui::save::{self, SaveData};
use deep_world_tui::save_migrations::CURRENT_SAVE_VERSION;
use deep_world_tui::sim::SimState;

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
        version,
    }
}

#[test]
fn current_version_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("current_v.ron");
    let path_str = path.to_str().unwrap();

    let data = make_save(CURRENT_SAVE_VERSION);
    save::save_game(&data, path_str).expect("save should succeed");
    let loaded = save::load_game(path_str).expect("load should succeed");
    assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    assert_eq!(data.sim.world.tick, loaded.sim.world.tick);
}

#[test]
fn v0_migrates_to_current_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v0_migrate.ron");
    let path_str = path.to_str().unwrap();

    let data = make_save(0);
    // Save with version 0
    save::save_game(&data, path_str).expect("save should succeed");
    let loaded = save::load_game(path_str).expect("load should succeed");
    assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    // Default fields should be populated
    assert!(loaded.collapse_log.is_empty());
    assert!(loaded.lineage.is_empty());
}

#[test]
fn future_version_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("future_v.ron");
    let path_str = path.to_str().unwrap();

    let data = make_save(999);
    save::save_game(&data, path_str).expect("save should succeed");
    let result = save::load_game(path_str);
    assert!(result.is_err(), "loading future version should fail");
    let err = result.unwrap_err();
    assert!(!err.contains("dummy"));
}

#[test]
fn roundtrip_preserves_all_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("roundtrip.ron");
    let path_str = path.to_str().unwrap();

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
        version: CURRENT_SAVE_VERSION,
    };

    save::save_game(&data, path_str).expect("save should succeed");
    let loaded = save::load_game(path_str).expect("load should succeed");
    assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    assert_eq!(loaded.encounters_had, 5);
    assert_eq!(loaded.collapses_had, 2);
    assert!(loaded.player_pos.is_some());
    assert_eq!(data.sim.world.tick, loaded.sim.world.tick);
}
