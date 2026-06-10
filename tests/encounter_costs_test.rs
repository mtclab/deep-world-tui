// Regression: EncounterAction's data-driven costs (energy, hunger, hours) and
// god-affinity shift were defined but never applied — resolve_encounter
// hardcoded a flat 1h + 0.08 energy and ignored hunger and god affinity. These
// assert the action methods now actually drive resolution.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{Encounter, EncounterAction, EncounterKind, GodName, Terrain};
use deep_world_tui::ui::app::App;

fn fresh_app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut app = App::new(seed, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app
}

#[test]
fn push_through_spends_energy_and_hunger() {
    let mut app = fresh_app(42);
    app.vitals.energy = 1.0;
    app.vitals.hunger = 1.0;
    app.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain: Terrain::Grass,
    });
    app.resolve_encounter(EncounterAction::PushThrough);
    // PushThrough: energy_cost 0.2, hunger_cost 0.1.
    assert!(
        (app.vitals.energy - 0.8).abs() < 1e-6,
        "energy should drop by 0.2, got {}",
        app.vitals.energy
    );
    assert!(
        (app.vitals.hunger - 0.9).abs() < 1e-6,
        "hunger should drop by 0.1, got {}",
        app.vitals.hunger
    );
}

#[test]
fn calm_raises_keuru_affinity() {
    let mut app = fresh_app(42);
    let before = app.god_affinity.get(GodName::Keuru);
    app.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain: Terrain::Grass,
    });
    app.resolve_encounter(EncounterAction::Calm);
    // Calm -> Keuru +0.05.
    assert!(
        app.god_affinity.get(GodName::Keuru) > before,
        "Calm should raise Keuru affinity ({} -> {})",
        before,
        app.god_affinity.get(GodName::Keuru)
    );
}
