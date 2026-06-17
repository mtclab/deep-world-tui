// Leadership events (#556): the council turns over now and then, and it reaches
// the town's talk.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

#[test]
fn the_council_turns_over_and_is_talked_of() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    // The journal is capped (MAX_JOURNAL), so a leadership rumor logged early in
    // the soak can be evicted by later talk. Watch for it as it lands instead of
    // only inspecting the journal at the end.
    let mut heard = false;
    for _ in 0..(24 * 200) {
        sim.step();
        if sim.journal.entries.iter().any(|e| {
            e.text.contains("take the council")
                || e.text.contains("dispute splits the council")
                || e.text.contains("council-festival")
        }) {
            heard = true;
            break;
        }
    }
    assert!(
        heard,
        "a leadership event reached the town's talk over the soak"
    );
}
