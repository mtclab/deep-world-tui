// Regression: a companion neglected until its needs max out must leave. Nothing
// enforced is_alive(), so starved companions lingered forever at full need.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::companion::{Animal, Companion};
use deep_world_tui::ui::app::App;

#[test]
fn neglected_companion_departs() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    {
        let ps = app.player_start.as_mut().expect("player");
        ps.companions.clear();
        let mut c = Companion::new(Animal::Dog, "Scrap".into(), 0);
        c.food_need = 100.0; // maxed -> is_alive() == false
        ps.companions.push(c);
    }
    assert_eq!(app.player_start.as_ref().unwrap().companions.len(), 1);

    app.advance_clock(1);

    assert!(
        app.player_start.as_ref().unwrap().companions.is_empty(),
        "a maxed-need companion should depart on the next tick"
    );
}

#[test]
fn content_companion_stays() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    {
        let ps = app.player_start.as_mut().expect("player");
        ps.companions.clear();
        ps.companions
            .push(Companion::new(Animal::Dog, "Scrap".into(), 0));
    }
    app.advance_clock(1);
    assert_eq!(
        app.player_start.as_ref().unwrap().companions.len(),
        1,
        "a well-kept companion should stay"
    );
}
