// The mountain stirs (#455, Mythic Phase 2): a vast slow presence on the high
// slope, far too large for a rockfall, that goes still when you look. Wait it
// out (Calm) and pass safe; cross beneath it (PushThrough) and risk the loose
// scree. Deniable as the mountain settling.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::Disease;
use deep_world_tui::model::{Encounter, EncounterAction, EncounterKind, PlayerPos, Terrain};
use deep_world_tui::ui::app::App;

fn kinds_on(terrain: Terrain) -> Vec<EncounterKind> {
    (0..4000u64)
        .filter_map(|seed| {
            Encounter::roll_biased_weather(terrain, 10, 1, seed, None, 1.0).map(|e| e.kind)
        })
        .collect()
}

#[test]
fn the_mountain_stirs_only_on_the_high_rock() {
    assert!(
        kinds_on(Terrain::Mountain).contains(&EncounterKind::MountainStir),
        "the mountain stir should appear on the mountain"
    );
    assert!(
        !kinds_on(Terrain::Grass).contains(&EncounterKind::MountainStir),
        "not on the open grass"
    );
}

fn app_on_slope(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.vitals.energy = 1.0;
    a.vitals.hunger = 1.0;
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    if let Some(ref mut ps) = a.player_start {
        ps.person.illnesses.clear();
    }
    a.encounter = Some(Encounter {
        kind: EncounterKind::MountainStir,
        terrain: Terrain::Mountain,
        species: None,
    });
    a
}

#[test]
fn waiting_it_out_is_safe() {
    for seed in 0..120u64 {
        let mut a = app_on_slope(seed);
        a.resolve_encounter(EncounterAction::Calm);
        assert!(
            a.player_start
                .as_ref()
                .unwrap()
                .person
                .illnesses
                .iter()
                .all(|d| d.disease != Disease::Sprain),
            "waiting it out should never sprain you (seed {seed})"
        );
        assert!(a.vitals.energy > 0.8, "waiting it out keeps your strength");
    }
}

#[test]
fn crossing_beneath_it_costs_and_sometimes_sprains() {
    let mut sprains = 0;
    for seed in 0..200u64 {
        let mut a = app_on_slope(seed);
        a.resolve_encounter(EncounterAction::PushThrough);
        assert!(a.vitals.energy <= 0.72, "crossing beneath spends the day");
        if a.player_start
            .as_ref()
            .unwrap()
            .person
            .illnesses
            .iter()
            .any(|d| d.disease == Disease::Sprain)
        {
            sprains += 1;
        }
    }
    assert!(
        sprains > 0,
        "the loose scree should turn an ankle now and then, got {sprains}/200"
    );
}
