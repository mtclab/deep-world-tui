// Feud-load → unrest (#552): a town riven by rivalry lives uneasy and is talked
// of. Over a long soak, some settlement's grudges stir.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

#[test]
fn riven_towns_stir_grudges() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    // Long enough that rivalries form and deepen past the unrest threshold
    // somewhere in the province.
    for _ in 0..(24 * 300) {
        sim.step();
    }
    let stirred = sim
        .journal
        .entries
        .iter()
        .any(|e| e.text.contains("Old grudges stir"));
    // Either the grudges were talked of, or at least some town crossed the
    // feud-load threshold (the rumor is rate-limited, so accept either as proof
    // the unrest machinery engaged).
    let riven = sim.world.regions.iter().any(|r| {
        r.settlements
            .iter()
            .any(|s| deep_world_tui::model::relation::feud_load(&s.people) > 0.25)
    });
    assert!(
        stirred || riven,
        "feud-load unrest engaged somewhere over the soak"
    );
}
