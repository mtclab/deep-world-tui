// Trade caravans actually run: they spawn between settlements over time and are
// retired once their goods disperse (bounded, not accumulating forever).

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

#[test]
fn caravans_spawn_and_stay_bounded() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    let mut ever_seen = 0usize;
    let mut max_at_once = 0usize;
    for _ in 0..600 {
        sim.step();
        ever_seen = ever_seen.max(sim.caravans.len());
        max_at_once = max_at_once.max(sim.caravans.len());
    }
    assert!(ever_seen > 0, "caravans should spawn over time");
    assert!(
        max_at_once < 20,
        "caravans should be retired, not pile up: peak {max_at_once}"
    );
}
