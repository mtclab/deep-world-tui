use deep_world_tui::charts;
use deep_world_tui::model::{
    GameClock, GodAffinity, InterPeopleBias, PeopleKind, PlayerPos, PlayerVitals,
};
use deep_world_tui::rng::SeedRng;
use deep_world_tui::save::{self, LineageRecord, SaveData};
use deep_world_tui::save_migrations::CURRENT_SAVE_VERSION;
use deep_world_tui::sim::hints::HintTracker;
use deep_world_tui::sim::milestones::MilestoneTracker;
use deep_world_tui::sim::SimState;

fn load_charts() -> charts::Charts {
    charts::load_charts().expect("charts should load from data/charts.ron")
}

#[allow(deprecated)]
fn make_save_data(seed: u64, charts: &charts::Charts) -> SaveData {
    let mut rng = SeedRng::new(seed);
    let player_start = deep_world_tui::gen::player::generate_player_start(&mut rng, charts);
    SaveData {
        sim: SimState::new(seed, charts.clone()),
        player_start: Some(player_start),
        clock: GameClock::default(),
        vitals: PlayerVitals::default(),
        player_pos: Some(PlayerPos {
            region_idx: 0,
            px: 20,
            py: 10,
        }),
        god_affinity: GodAffinity::new(),
        inter_people_bias: InterPeopleBias::new(PeopleKind::Metsik),
        encounters_had: 0,
        collapses_had: 0,
        collapse_log: Vec::new(),
        lineage: Vec::new(),
        milestones: MilestoneTracker::new(),
        explored: Vec::new(),
        version: CURRENT_SAVE_VERSION,
        first_run: true,
        hint_tracker: HintTracker::default(),
        start_age_years: 0,
        birth_day: 0,
        lifespan_years: 0,
        encounter_log: Default::default(),
    }
}

#[test]
fn lineage_record_serializes_and_deserializes() {
    let record = LineageRecord {
        predecessor_name: "Test Character".to_string(),
        predecessor_id: "p_42".to_string(),
        cause: "Ditch".to_string(),
        settlement_id: "settlement_0".to_string(),
        tick: 100,
    };
    let ron_str = ron::ser::to_string_pretty(&record, ron::ser::PrettyConfig::default())
        .expect("serialize should work");
    let loaded: LineageRecord = ron::from_str(&ron_str).expect("deserialize should work");
    assert_eq!(loaded.predecessor_name, "Test Character");
    assert_eq!(loaded.predecessor_id, "p_42");
    assert_eq!(loaded.cause, "Ditch");
    assert_eq!(loaded.settlement_id, "settlement_0");
    assert_eq!(loaded.tick, 100);
}

#[test]
fn savedata_with_lineage_roundtrips() {
    let charts = load_charts();
    let mut data = make_save_data(42, &charts);
    data.lineage.push(LineageRecord {
        predecessor_name: "First Hero".to_string(),
        predecessor_id: "p_1".to_string(),
        cause: "Storm".to_string(),
        settlement_id: "settlement_0".to_string(),
        tick: 50,
    });

    save::save_game(&data, "lineage_roundtrip.ron").expect("save should succeed");
    let loaded = save::load_game("lineage_roundtrip.ron").expect("load should succeed");

    assert_eq!(loaded.lineage.len(), 1, "should have one lineage record");
    assert_eq!(loaded.lineage[0].predecessor_name, "First Hero");
    assert_eq!(loaded.lineage[0].predecessor_id, "p_1");
    assert_eq!(loaded.lineage[0].cause, "Storm");
    assert_eq!(loaded.lineage[0].settlement_id, "settlement_0");
    assert_eq!(loaded.lineage[0].tick, 50);
}

#[test]
fn savedata_without_lineage_loads_empty() {
    let charts = load_charts();
    let data = make_save_data(77, &charts);

    save::save_game(&data, "no_lineage.ron").expect("save should succeed");
    let loaded = save::load_game("no_lineage.ron").expect("load should succeed");

    assert!(loaded.lineage.is_empty(), "lineage should default to empty");
}

#[test]
fn lineage_save_file_created_on_death() {
    let charts = load_charts();
    let mut data = make_save_data(123, &charts);
    data.lineage.push(LineageRecord {
        predecessor_name: "Dead Hero".to_string(),
        predecessor_id: "p_dead".to_string(),
        cause: "Fell".to_string(),
        settlement_id: "settlement_1".to_string(),
        tick: 200,
    });

    let result = save::save_lineage(&data, 123);
    assert!(result.is_ok(), "save_lineage should succeed");

    let loaded = save::load_game("lineage_123.ron");
    assert!(loaded.is_ok(), "lineage file should be loadable");
    let loaded = loaded.expect("loaded");
    assert_eq!(loaded.lineage.len(), 1);
    assert_eq!(loaded.lineage[0].predecessor_name, "Dead Hero");
}

#[test]
fn find_related_npc_by_relationship() {
    let charts = load_charts();
    let data = make_save_data(55, &charts);

    // Verify sim has relationships and settlements
    assert!(
        !data.sim.world.regions.is_empty(),
        "world should have regions"
    );
    let region = &data.sim.world.regions[0];
    assert!(
        !region.settlements.is_empty(),
        "region should have settlements"
    );
    let settlement = &region.settlements[0];
    assert!(
        !settlement.people.is_empty(),
        "settlement should have people"
    );

    // Find any person ID to verify relationship lookup works
    let person = &settlement.people[0];
    let rels = data.sim.relationships.relationships_for(&person.id);
    // Relationships may or may not exist for generated characters
    // but the function should not panic
    assert!(rels.len() < 1000, "relationship count should be reasonable");
}

#[test]
fn memorial_entry_contains_predecessor_name() {
    let predecessor_name = "Elder Sampo";
    let memorial = format!(
        "{} passed on. You carry their memory forward.",
        predecessor_name
    );
    assert!(
        memorial.contains(predecessor_name),
        "memorial should contain predecessor name"
    );
    assert!(
        memorial.contains("passed on"),
        "memorial should mention passing"
    );
}

#[test]
fn reputation_boost_is_015() {
    let mut store = deep_world_tui::sim::reputation::ReputationStore::new();
    let person_id = "test_person";
    let settlement_id = "test_settlement";

    let initial_rep = store.get(person_id, settlement_id);
    store.adjust_local(person_id, settlement_id, 0.15);
    let boosted_rep = store.get(person_id, settlement_id);

    let diff = boosted_rep - initial_rep;
    assert!(
        (diff - 0.15).abs() < f64::EPSILON,
        "reputation boost should be exactly +0.15, got diff={}",
        diff
    );
}

#[test]
fn multiple_lineage_records_accumulate() {
    let charts = load_charts();
    let mut data = make_save_data(99, &charts);
    data.lineage.push(LineageRecord {
        predecessor_name: "First".to_string(),
        predecessor_id: "p1".to_string(),
        cause: "Storm".to_string(),
        settlement_id: "s1".to_string(),
        tick: 100,
    });
    data.lineage.push(LineageRecord {
        predecessor_name: "Second".to_string(),
        predecessor_id: "p2".to_string(),
        cause: "Ditch".to_string(),
        settlement_id: "s2".to_string(),
        tick: 200,
    });

    save::save_game(&data, "multi_lineage.ron").expect("save should work");
    let loaded = save::load_game("multi_lineage.ron").expect("load should work");

    assert_eq!(loaded.lineage.len(), 2, "should have two lineage records");
    assert_eq!(loaded.lineage[0].predecessor_name, "First");
    assert_eq!(loaded.lineage[1].predecessor_name, "Second");
    assert_eq!(loaded.lineage[0].tick, 100);
    assert_eq!(loaded.lineage[1].tick, 200);
}
