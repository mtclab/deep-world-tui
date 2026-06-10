// Variable-duration rest: a short spurt costs fewer hours and restores less than
// a full night; duration is clamped to the max.

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

fn fresh_app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut app = App::new(seed, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app
}

#[test]
fn short_rest_advances_fewer_hours_than_long() {
    let mut a = fresh_app(7);
    let d0 = a.clock.day as u64 * 24 + a.clock.hour as u64;
    a.rest_hours(2);
    let short = (a.clock.day as u64 * 24 + a.clock.hour as u64) - d0;

    let mut b = fresh_app(7);
    let e0 = b.clock.day as u64 * 24 + b.clock.hour as u64;
    b.rest_hours(8);
    let long = (b.clock.day as u64 * 24 + b.clock.hour as u64) - e0;

    assert_eq!(short, 2, "2h rest advances 2 hours");
    assert_eq!(long, 8, "8h rest advances 8 hours");
}

#[test]
fn short_rest_restores_less_energy() {
    let mut a = fresh_app(7);
    a.vitals.energy = 0.0;
    a.rest_hours(1);
    let after_short = a.vitals.energy;

    let mut b = fresh_app(7);
    b.vitals.energy = 0.0;
    b.rest_hours(8);
    let after_long = b.vitals.energy;

    assert!(
        after_long > after_short,
        "8h should restore more energy than 1h: {after_short} vs {after_long}"
    );
}

#[test]
fn rest_duration_is_clamped() {
    let mut a = fresh_app(7);
    let d0 = a.clock.day as u64 * 24 + a.clock.hour as u64;
    a.rest_hours(999); // clamps to MAX_REST_HOURS
    let advanced = (a.clock.day as u64 * 24 + a.clock.hour as u64) - d0;
    assert_eq!(advanced, App::MAX_REST_HOURS as u64);
}
