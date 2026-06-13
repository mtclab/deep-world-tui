// The deep and the tide earn their acts (#439): scale-hand reads true value
// in a trade (better prices, costs the body); still-sense settles a beast.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::ItemType;
use deep_world_tui::model::{
    CraftSense, Encounter, EncounterAction, EncounterKind, Fortune, Gift, PlayerPos, Terrain,
};
use deep_world_tui::ui::app::App;

fn app(seed: u64, gift: Option<CraftSense>) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    a.fortune = Fortune::from_value(0.0);
    a.gift = match gift {
        Some(s) => Gift::of(s),
        None => Gift::NONE,
    };
    if let Some(ps) = a.player_start.as_mut() {
        ps.inventory.add(ItemType::Coin, 200);
        ps.inventory.add(ItemType::Herb, 40);
        ps.companions.clear();
    }
    a
}

#[test]
fn the_scale_hand_buys_under_and_sells_over() {
    let plain = app(7, None);
    let scale = app(7, Some(CraftSense::ScaleHand));
    // A higher-base good so the 10% shows past integer rounding.
    let it = ItemType::Coat;
    let buy_plain = plain.quote_buy_price(it);
    let buy_scale = scale.quote_buy_price(it);
    let sell_plain = plain.quote_sell_price(it);
    let sell_scale = scale.quote_sell_price(it);
    assert!(buy_scale <= buy_plain, "scale-hand should not buy dearer");
    assert!(
        sell_scale >= sell_plain,
        "scale-hand should not sell cheaper"
    );
    // And at least one direction is strictly better (the gift is real).
    assert!(buy_scale < buy_plain || sell_scale > sell_plain);
}

#[test]
fn trading_reveals_and_taxes_the_scale_hand() {
    let mut a = app(7, Some(CraftSense::ScaleHand));
    assert!(!a.gift_revealed);
    a.buy_item(ItemType::Herb);
    assert!(a.gift_revealed, "a trade surfaces the scale-hand");
    assert!(a.gift_strain > 0.0, "the trade taxed the body");
}

#[test]
fn the_craftless_trade_costs_nothing() {
    let mut a = app(7, None);
    a.buy_item(ItemType::Herb);
    assert!(!a.gift_revealed && a.gift_strain == 0.0);
}

#[test]
fn the_still_sense_settles_a_beast_and_pays() {
    let mut a = app(7, Some(CraftSense::StillSense));
    a.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain: Terrain::Forest,
        species: None,
    });
    a.resolve_encounter(EncounterAction::Calm);
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("stills") || msg.contains("quiet"),
        "still-sense calm: {msg}"
    );
    assert!(
        a.gift_revealed && a.gift_strain > 0.0,
        "the calm taxed the body"
    );
}
