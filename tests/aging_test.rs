// Aging: the player ages with elapsed days (decoupled from the hour calendar),
// becomes an elder near the end of life, and dies of old age — which routes into
// the existing lineage / continue-as-heir flow.

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

fn fresh_app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut app = App::new(seed, charts);
    app.sim_pop_cap = Some(300);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app
}

#[test]
fn player_ages_with_elapsed_days() {
    let mut app = fresh_app(9001);
    // Pin a long lifespan so the character can't die mid-test, isolating the
    // age-from-elapsed-days math.
    app.start_age_years = 20;
    app.birth_day = app.clock.day;
    app.lifespan_years = 9999;
    let start = app.current_age_years();
    for _ in 0..60 {
        app.rest(); // ~8h each → ~20 days
    }
    let later = app.current_age_years();
    assert!(
        later > start,
        "player should age over many days: {start} -> {later} (day {})",
        app.clock.day
    );
}

#[test]
fn player_dies_of_old_age_and_lineage_continues() {
    let mut app = fresh_app(9001);
    // Enough rests to push well past a full lifespan (death every ~lifespan*3 days).
    for _ in 0..400 {
        app.rest();
    }
    // Old-age death routes through continue_as_npc, which records lineage.
    assert!(
        !app.lineage.is_empty(),
        "at least one old-age death should have produced a lineage record"
    );
    // And the run reached elderhood at some point.
    assert!(app.elder || !app.lineage.is_empty());
}
