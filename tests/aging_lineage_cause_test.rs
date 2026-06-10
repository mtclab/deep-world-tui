// Regression: an old-age death must be recorded in the lineage with the
// correct cause. continue_as_npc used to read self.collapse.outcome (stale /
// None for an old-age death), so OldAge deaths were mislabeled or "unknown".
// The cause now comes from death_cause.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

#[test]
fn old_age_death_is_labeled_old_age_in_lineage() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    // Force the aging clock past the lifespan so the next tick triggers
    // die_of_old_age. Keep vitals healthy so a collapse death doesn't preempt
    // it within the single advanced hour.
    app.start_age_years = 80;
    app.birth_day = app.clock.day;
    app.lifespan_years = 80; // current_age (80) >= lifespan triggers old-age death
    app.vitals.hunger = 1.0;
    app.vitals.thirst = 1.0;
    app.vitals.energy = 1.0;

    let before = app.lineage.len();
    app.advance_clock(1);

    assert!(
        app.lineage.len() > before,
        "old-age death should hand off to an heir and push a lineage record"
    );
    let rec = app.lineage.last().unwrap();
    assert_eq!(
        rec.cause, "old age",
        "lineage cause should reflect death_cause (OldAge), got {:?}",
        rec.cause
    );
}
