// Craft quality (#547): a masterwork piece fetches more at market than a rough
// or worn one of the same kind.
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let (mx, my) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: mx,
        py: my,
    });
    a
}

#[test]
fn a_masterwork_sells_dearer_than_a_rough_piece() {
    let mut a = app();
    {
        let inv = &mut a.player_start.as_mut().unwrap().inventory;
        inv.add(ItemType::Coat, 1); // a high-base good so the spread is visible
    }
    // Masterwork durability.
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .durability
        .insert(ItemType::Coat, 0.98);
    let master = a.quote_sell_price(ItemType::Coat);
    // Rough/worn durability.
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .durability
        .insert(ItemType::Coat, 0.28);
    let rough = a.quote_sell_price(ItemType::Coat);

    assert!(
        master > rough,
        "a masterwork Coat sells dearer than a rough one ({master} > {rough})"
    );
}
