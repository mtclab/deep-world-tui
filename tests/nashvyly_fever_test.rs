// The näšvyly's miasma (#455, Mythic Phase 2): the forest-fever shape leaves a
// wood-fever behind it more often than not — the myth creature (#450) tied into
// the disease/mortality system (#448). Deniable, fortune-leaned. A mundane beast
// of the same wood leaves no such thing.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::Disease;
use deep_world_tui::model::wildlife::WildSpecies;
use deep_world_tui::model::{Encounter, EncounterAction, EncounterKind, PlayerPos, Terrain};
use deep_world_tui::ui::app::App;

fn met_brought_fever(seed: u64, species: WildSpecies) -> bool {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    if let Some(ref mut ps) = a.player_start {
        ps.person.illnesses.clear();
    }
    a.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain: Terrain::Forest,
        species: Some(species),
    });
    // Calm the beast — no flee gamble, no wound: any fever is the miasma alone.
    a.resolve_encounter(EncounterAction::Calm);
    a.player_start.as_ref().is_some_and(|ps| {
        ps.person
            .illnesses
            .iter()
            .any(|d| d.disease == Disease::Fever)
    })
}

#[test]
fn the_nashvyly_leaves_a_wood_fever_more_often_than_not() {
    let fevers = (0..240u64)
        .filter(|&s| met_brought_fever(s, WildSpecies::Nashvyly))
        .count();
    assert!(
        fevers > 40,
        "meeting the näšvyly should often bring the wood-fever, got {fevers}/240"
    );
}

#[test]
fn a_mundane_beast_brings_no_fever() {
    let fevers = (0..240u64)
        .filter(|&s| met_brought_fever(s, WildSpecies::PineMarten))
        .count();
    assert_eq!(
        fevers, 0,
        "an ordinary pine marten leaves no fever behind it, got {fevers}/240"
    );
}
