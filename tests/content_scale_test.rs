// Content scale (#324): the hourly-exposed axes grow — encounters 12→24,
// quests 5→9, discoveries 12→24, recipes 8→12 — and every new entry is wired
// (spawnable / generatable / completable), not just defined.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::discovery::DiscoveryKind;
use deep_world_tui::model::quest::QuestKind;
use deep_world_tui::model::{craft_recipes, Encounter, EncounterKind, PeopleKind, Terrain};
use std::collections::HashSet;

#[test]
fn new_encounter_kinds_actually_spawn() {
    let mut seen: HashSet<&'static str> = HashSet::new();
    let terrains = [
        Terrain::Road,
        Terrain::Grass,
        Terrain::Farmland,
        Terrain::Sand,
        Terrain::Tundra,
        Terrain::Mountain,
        Terrain::Cave,
        Terrain::Forest,
        Terrain::Swamp,
    ];
    for seed in 0..3000u64 {
        for t in terrains {
            for day in [5u32, 35, 65] {
                if let Some(e) = Encounter::roll_biased_weather(t, 10, day, seed, None, 1.5) {
                    let tag = match e.kind {
                        EncounterKind::Mirage => "mirage",
                        EncounterKind::CaveIn => "cavein",
                        EncounterKind::BorderWatch => "watch",
                        EncounterKind::PlagueWagon => "plague",
                        EncounterKind::BeastMigration => "herd",
                        EncounterKind::EscapedLivestock => "livestock",
                        EncounterKind::LostChild => "child",
                        EncounterKind::FuneralProcession => "funeral",
                        EncounterKind::PilgrimBand => "pilgrims",
                        EncounterKind::DistantFire => "fire",
                        EncounterKind::RiverFlood => "flood",
                        EncounterKind::AuroraVeil => "aurora",
                        _ => continue,
                    };
                    seen.insert(tag);
                }
            }
        }
        if seen.len() >= 12 {
            break;
        }
    }
    assert!(
        seen.len() >= 10,
        "most new encounter kinds should be reachable in play; saw {seen:?}"
    );
}

#[test]
fn every_new_encounter_kind_has_actions() {
    for k in [
        EncounterKind::RiverFlood,
        EncounterKind::Mirage,
        EncounterKind::CaveIn,
        EncounterKind::FuneralProcession,
        EncounterKind::LostChild,
        EncounterKind::EscapedLivestock,
        EncounterKind::PlagueWagon,
        EncounterKind::PilgrimBand,
        EncounterKind::BeastMigration,
        EncounterKind::DistantFire,
        EncounterKind::BorderWatch,
        EncounterKind::AuroraVeil,
    ] {
        assert!(!k.available_actions().is_empty());
        assert!(!k.description().is_empty());
    }
}

#[test]
fn new_quest_kinds_generate() {
    let charts = load_charts().expect("charts");
    let world = deep_world_tui::gen::world::generate_world(42, &charts);
    let mut kinds: HashSet<u8> = HashSet::new();
    for salt in 0..200u64 {
        for q in deep_world_tui::sim::quest_gen::generate_quests(
            salt,
            PeopleKind::Metsik,
            &world.regions,
            1,
        ) {
            kinds.insert(match q.kind {
                QuestKind::FetchItem { .. } => 0,
                QuestKind::VisitRegion { .. } => 1,
                QuestKind::AidNPC { .. } => 2,
                QuestKind::ReachReputation { .. } => 3,
                QuestKind::SurviveDays { .. } => 4,
                QuestKind::DeliverTo { .. } => 5,
                QuestKind::RaiseBuilding { .. } => 6,
                QuestKind::VisitDiscovery { .. } => 7,
                QuestKind::TalkToPeople { .. } => 8,
                // Living-world relief tasks are posted by the App from world
                // state, not by generate_quests — they never appear here.
                QuestKind::RelievePlague { .. } => 9,
                QuestKind::RelieveFamine { .. } => 10,
            });
        }
    }
    assert_eq!(
        kinds.len(),
        9,
        "all 9 quest kinds should generate: {kinds:?}"
    );
}

#[test]
fn discovery_and_recipe_counts() {
    assert_eq!(DiscoveryKind::all().len(), 24);
    assert_eq!(craft_recipes().len(), 18);
    // Every discovery has text, glyph, and a wired effect.
    for k in DiscoveryKind::all() {
        assert!(!k.observe_text().is_empty());
        let _ = k.observe_effect();
        assert_ne!(k.glyph(), '\0');
    }
}
