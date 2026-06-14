// The spectral elk (#455, Mythic Phase 2): a sending at the forest tree-line.
// Follow it (PushThrough) and the wood takes the day from you — chased to
// exhaustion, sometimes run down to nothing. Flee and you break clean.
use deep_world_tui::charts::load::load_charts;
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
fn the_sending_walks_the_tree_line() {
    assert!(
        kinds_on(Terrain::Forest).contains(&EncounterKind::SpectralElk),
        "the spectral elk should appear in the forest"
    );
    // Not out on the open steppe or the sand.
    for elsewhere in [Terrain::Grass, Terrain::Sand, Terrain::Coast] {
        assert!(
            !kinds_on(elsewhere).contains(&EncounterKind::SpectralElk),
            "the sending keeps to the forest, not {elsewhere:?}"
        );
    }
}

fn app_with_elk() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(3, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    a.vitals.energy = 1.0;
    a.vitals.hunger = 1.0;
    a.encounter = Some(Encounter {
        kind: EncounterKind::SpectralElk,
        terrain: Terrain::Forest,
        species: None,
    });
    a
}

#[test]
fn following_the_elk_exhausts_you() {
    let mut a = app_with_elk();
    a.resolve_encounter(EncounterAction::PushThrough);
    assert!(
        a.vitals.energy <= 0.45,
        "chasing the sending should drain the day, energy now {}",
        a.vitals.energy
    );
}

#[test]
fn fleeing_the_elk_breaks_clean() {
    let mut a = app_with_elk();
    a.resolve_encounter(EncounterAction::Flee);
    // Only the ordinary flee cost — nothing like the chase's drain.
    assert!(
        a.vitals.energy > 0.6,
        "breaking away should keep most of your strength, energy now {}",
        a.vitals.energy
    );
}
