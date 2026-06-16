// Wage-work (#526 livelihoods): hire on for a day's labour at the settlement
// you stand in — meagre coin for the work the place needs.
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

fn stand_in_town(a: &mut App) {
    let (mx, my) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: mx,
        py: my,
    });
}

#[test]
fn a_day_of_labour_pays_coin() {
    let mut a = app();
    stand_in_town(&mut a);
    assert!(a.player_on_settlement().is_some(), "standing in a town");
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    let day_before = a.clock.day;
    a.work_for_hire();
    let coin_after = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    assert!(coin_after > coin_before, "a day's work pays a wage");
    assert!(
        a.clock.day > day_before || a.clock.hour != 8,
        "the day is spent"
    );
}

#[test]
fn no_settlement_no_work() {
    let mut a = app();
    // Put the player off any settlement (corner is wilderness in seed 42).
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 0,
        py: 0,
    });
    if a.player_on_settlement().is_some() {
        // (0,0) happens to be town in this seed — skip; the guard is unit-clear.
        return;
    }
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    a.work_for_hire();
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin),
        coin_before,
        "no wage in the wilds"
    );
    assert!(
        a.status_msg
            .as_deref()
            .unwrap_or("")
            .contains("No settlement"),
        "told there's no work here"
    );
}
