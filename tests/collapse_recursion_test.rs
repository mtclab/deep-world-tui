// Regression: a collapse advances the clock for its unconscious hours, and that
// nested advance used to re-trigger check_collapse → unbounded mutual recursion →
// stack overflow once vitals stayed at zero (≈2 weeks of resting). Guarded by an
// `in_collapse` re-entrancy flag. This drives many rests (which drain hunger to
// zero and trigger repeated collapses) and asserts the sim survives.

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

#[test]
fn many_rests_through_collapse_do_not_overflow() {
    let charts = load_charts().expect("charts load");
    let mut app = App::new(9001, charts);
    app.sim_pop_cap = Some(300);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    // ~300 rests ≈ 100 in-game days — well past the old ~13-day crash point.
    for _ in 0..300 {
        app.rest();
    }

    // If we got here without aborting, the recursion is bounded. Sanity-check the
    // clock actually advanced a long way.
    assert!(
        app.clock.day > 50,
        "clock should advance across many rests, got day {}",
        app.clock.day
    );
}
