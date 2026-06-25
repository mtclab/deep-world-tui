// Every action ticks the world, not only movement (#action-tick): spend_action
// owes time to the clock and pays it in whole hours, like travel.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

#[test]
fn actions_advance_the_world_clock() {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.sim_pop_cap = Some(300);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let hour0 = a.clock.hour as i64 + a.clock.day as i64 * 24;
    // ten discrete actions of ~6 minutes each = ~1 hour of world time
    for _ in 0..20 {
        a.spend_action(0.1);
    }
    let hour1 = a.clock.hour as i64 + a.clock.day as i64 * 24;
    assert!(
        hour1 > hour0,
        "twenty actions should pass time: {hour0} -> {hour1}"
    );
}

#[test]
fn a_single_short_action_does_not_freeze_the_world() {
    // The fraction is owed even when it doesn't yet reach a whole hour.
    let charts = load_charts().expect("charts");
    let mut a = App::new(7, charts);
    a.sim_pop_cap = Some(300);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let before = a.travel_debt;
    a.spend_action(0.1);
    assert!(a.travel_debt > before || a.clock.hour as i64 + a.clock.day as i64 * 24 > 0);
}
