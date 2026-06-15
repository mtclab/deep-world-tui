// Faith (#457): the gods are withdrawn since the Fall, so prayer is devotion,
// not a summons — a quiet hour that deepens your bond with the god you keep,
// the practice plateauing the more you already keep it. No miracle, no world
// change; only the one who prays is changed.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 8;
    a
}

#[test]
fn prayer_deepens_a_bond_and_costs_an_hour() {
    let mut a = app();
    let day = a.clock.day;
    let hour = a.clock.hour;
    a.pray();
    // The god you most keep now exists — your devotion settled on one.
    let god = a
        .god_affinity
        .strongest_ally()
        .expect("prayer settles on a god");
    assert!(a.god_affinity.get(god) > 0.0, "the bond deepened");
    // An hour passed.
    let advanced = a.clock.day > day || a.clock.hour != hour;
    assert!(advanced, "prayer takes an hour");
    // It speaks in the prayer voice, deniably.
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("prayer"),
        "a quiet moment of prayer"
    );
}

#[test]
fn devotion_plateaus() {
    let mut a = app();
    a.pray();
    let god = a.god_affinity.strongest_ally().unwrap();
    let after_one = a.god_affinity.get(god);
    a.pray();
    let after_two = a.god_affinity.get(god);
    let first_gain = after_one; // from 0
    let second_gain = after_two - after_one;
    assert!(after_two > after_one, "devotion still deepens");
    assert!(
        second_gain < first_gain,
        "but the practice plateaus ({first_gain} then {second_gain})"
    );
    // Many hours of prayer never make a god of you.
    for _ in 0..200 {
        a.pray();
    }
    assert!(
        a.god_affinity.get(god) < 1.0,
        "faith is a long road, never finished"
    );
}
