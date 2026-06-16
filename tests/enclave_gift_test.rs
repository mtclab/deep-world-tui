// Standing with a people (#454): a gift of worth lifts your regard with the
// folk of a settlement — goodwill is a kind of coin, and the Five remember it.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos};
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
fn a_gift_lifts_standing_and_is_given() {
    let mut a = app();
    stand_in_town(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Iron, 2);
    let people = a.current_settlement_people().expect("standing in a town");
    let before = a.inter_people_bias.effective_bias(people);

    a.give_gift();

    assert!(
        a.inter_people_bias.effective_bias(people) > before,
        "a gift lifts standing with the people"
    );
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Iron),
        1,
        "the gift is given from the pack"
    );
}

#[test]
fn no_gift_without_a_good_to_give() {
    let mut a = app();
    stand_in_town(&mut a);
    // Strip every tradeable good (keep food/water — those are never gifted).
    {
        let inv = &mut a.player_start.as_mut().unwrap().inventory;
        for item in ItemType::tradeable_items() {
            while inv.remove(item, 1) {}
        }
    }
    a.give_gift();
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("nothing worth giving"),
        "with nothing to give, no gift: {msg}"
    );
}
