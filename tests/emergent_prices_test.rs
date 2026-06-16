// Emergent prices (#540): a trade good's market price leans on the settlement's
// own stock — cheap where plentiful, dear where scarce. The basis for arbitrage.
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

fn set_iron(a: &mut App, si: usize, amount: f64) {
    let s = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[si];
    s.goods_stock.insert(ItemType::Iron, amount);
}

#[test]
fn a_good_is_dearer_where_scarce_than_where_plentiful() {
    let mut a = app();
    let (_ri, si) = a.player_on_settlement().expect("standing in town");
    let cap = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[si];
        s.population as f64 * 0.5
    };

    set_iron(&mut a, si, cap.max(1.0)); // plentiful
    let cheap = a.quote_buy_price(ItemType::Iron);
    set_iron(&mut a, si, 0.0); // scarce
    let dear = a.quote_buy_price(ItemType::Iron);

    assert!(
        dear > cheap,
        "Iron costs more where scarce ({dear}) than where plentiful ({cheap})"
    );
}

#[test]
fn an_untracked_good_is_unmoved_by_stock() {
    let mut a = app();
    let (_ri, si) = a.player_on_settlement().expect("town");
    // Food is governed by food_scarcity, not the goods-abundance lever; setting
    // an Iron stock must not move a Bandage's price.
    let before = a.quote_buy_price(ItemType::Bandage);
    set_iron(&mut a, si, 100.0);
    let after = a.quote_buy_price(ItemType::Bandage);
    assert_eq!(
        before, after,
        "an untracked good is unaffected by Iron stock"
    );
}

#[test]
fn selling_stays_below_buying_in_one_town() {
    // The spread holds for a tracked good at any stock level (no infinite-coin
    // loop within a single market).
    let mut a = app();
    let (_ri, si) = a.player_on_settlement().expect("town");
    for stock in [0.0, 1.0, 5.0, 50.0] {
        set_iron(&mut a, si, stock);
        let buy = a.quote_buy_price(ItemType::Iron);
        let sell = a.quote_sell_price(ItemType::Iron);
        assert!(
            sell < buy,
            "sell {sell} must stay below buy {buy} at stock {stock}"
        );
    }
}
