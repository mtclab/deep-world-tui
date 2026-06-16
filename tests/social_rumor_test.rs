// Surfacing the living web (#548): notable bonds reach the town's talk, so the
// social web is felt, not just simulated.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

#[test]
fn the_town_talks_of_new_bonds() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    for _ in 0..(24 * 150) {
        sim.step();
    }
    let heard =
        sim.journal.entries.iter().any(|e| {
            e.text.contains("thick as thieves") || e.text.contains("rivalry has sharpened")
        });
    assert!(heard, "the living social web reached the town's talk");
}
