// Balance: an elder's regard grows by the day, not by the deed. The trickle
// used to fire on every advance_clock call, so an afternoon of small actions
// saturated the town's esteem.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::app::App;

#[test]
fn elder_esteem_grows_by_the_day_not_the_deed() {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 6;
    a.elder = true;
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(deep_world_tui::model::ItemType::Food, 50);
        ps.inventory.add(deep_world_tui::model::ItemType::Water, 50);
        ps.companions.clear();
    }
    let pid = a.player_start.as_ref().unwrap().person.id.clone();
    let sid = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .id
        .clone();
    let before = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
    // Twelve one-hour deeds inside a single day: at most one day boundary
    // can be crossed, so the trickle fires at most once.
    for _ in 0..12 {
        a.advance_clock(1);
    }
    let after = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
    assert!(
        after - before <= 0.005 + 1e-9,
        "a busy afternoon must not saturate esteem ({before} -> {after})"
    );
}
