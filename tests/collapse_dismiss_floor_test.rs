// Regression: dismiss_collapse hardcoded vitals to 0.4/0.5, clobbering the
// outcome-specific restores check_collapse had already applied (a settlement
// bed or divine rescue recovers far more than a ditch). It must be a floor, not
// an assignment.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

#[test]
fn dismiss_preserves_good_restore() {
    let mut a = app();
    a.vitals.hunger = 0.8;
    a.vitals.energy = 0.9;
    a.dismiss_collapse();
    assert!(
        (a.vitals.hunger - 0.8).abs() < 1e-9,
        "good restore must not be reduced, got {}",
        a.vitals.hunger
    );
    assert!(
        (a.vitals.energy - 0.9).abs() < 1e-9,
        "good restore must not be reduced, got {}",
        a.vitals.energy
    );
}

#[test]
fn dismiss_floors_low_vitals() {
    let mut a = app();
    a.vitals.hunger = 0.1;
    a.vitals.energy = 0.1;
    a.dismiss_collapse();
    assert!(a.vitals.hunger >= 0.4, "hunger floored to 0.4");
    assert!(a.vitals.energy >= 0.5, "energy floored to 0.5");
}
