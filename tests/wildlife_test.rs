// Wild species (#361): the anonymous "wild creature" now has a face — and
// the strange ones get written down in the journal, deniable on the page.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{
    Encounter, EncounterAction, EncounterKind, Season, Terrain, WildSpecies,
};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

#[test]
fn the_roster_is_terrain_true() {
    // Forest rolls forest beasts; coast rolls coast beasts; never crossed.
    for seed in 0..300u64 {
        if let Some(s) = WildSpecies::roll(Terrain::Coast, Season::Green, seed) {
            assert!(
                s.habitats().contains(&Terrain::Coast),
                "{:?} does not live on the coast",
                s
            );
        }
    }
}

#[test]
fn an_uncanny_sighting_reaches_the_journal_deniable() {
    let mut a = app();
    a.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain: Terrain::Forest,
        species: Some(WildSpecies::HollowStag),
    });
    a.resolve_encounter(EncounterAction::Calm);
    let told = a
        .sim
        .as_ref()
        .unwrap()
        .journal
        .iter()
        .any(|e| e.text.contains("decided not to wonder"));
    assert!(told, "the strange gets written down");
    // And the line itself stays deniable — no god named, no certainty.
    let line = WildSpecies::HollowStag.line();
    assert!(
        line.contains("you tell yourself") || line.contains("surely") || line.contains("sensible"),
        "uncanny lines carry their own out: {line}"
    );
}

#[test]
fn mundane_beasts_do_not_clutter_the_scar_journal() {
    let mut a = app();
    a.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain: Terrain::Forest,
        species: Some(WildSpecies::Hare),
    });
    a.resolve_encounter(EncounterAction::Calm);
    let told = a
        .sim
        .as_ref()
        .unwrap()
        .journal
        .iter()
        .any(|e| e.text.contains("decided not to wonder"));
    assert!(!told, "a hare is just a hare");
}
