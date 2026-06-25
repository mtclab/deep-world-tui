// The quest board must not run dry: after the initial quests expire (deadlines
// are ~2-3 weeks), the world keeps posting new needs over a long game.

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
fn quest_board_does_not_run_dry() {
    let mut app = fresh_app(7);
    assert!(
        !app.sim.as_ref().unwrap().quests.is_empty(),
        "should start with quests"
    );
    // ~100 in-game days — well past the initial quests' deadlines.
    for _ in 0..300 {
        app.rest();
    }
    assert!(
        !app.sim.as_ref().unwrap().quests.is_empty(),
        "the quest board should have regenerated, not emptied"
    );
}
