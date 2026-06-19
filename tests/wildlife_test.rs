// Wild species (#361): the anonymous "wild creature" now has a face. (The
// uncanny-sighting-journal tests went with the encounter screen's retirement
// (#649) — the deniable line itself is still checked here.)
use deep_world_tui::model::{Season, Terrain, WildSpecies};

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
fn uncanny_lines_stay_deniable() {
    // The strange always has a sober out — no god named, no certainty.
    let line = WildSpecies::HollowStag.line();
    assert!(
        line.contains("you tell yourself") || line.contains("surely") || line.contains("sensible"),
        "uncanny lines carry their own out: {line}"
    );
}
