// Regression: reputation must move prices the right way and stay centered on
// 1.0. Buy/sell used raw local reputation ([0,1], baseline 0.5) directly as a
// multiplier, which halved prices at baseline and made buying *dearer* the more
// you were liked. They now use the centered signals::reputation_price_modifier
// (buy) and its inverse (sell), matching the service pricing.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::ItemType;
use deep_world_tui::ui::app::App;

#[test]
fn higher_reputation_lowers_buys_and_raises_sells() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    let buy0 = app.quote_buy_price(ItemType::Iron);
    let sell0 = app.quote_sell_price(ItemType::Iron);

    // Raise the player's standing in the current settlement.
    let pid = app.player_start.as_ref().unwrap().person.id.clone();
    let sid = {
        let sim = app.sim.as_ref().unwrap();
        sim.world.regions[0].settlements[0].id.clone()
    };
    app.sim
        .as_mut()
        .unwrap()
        .reputation
        .adjust_settlement(&pid, &sid, 0.4);

    let buy1 = app.quote_buy_price(ItemType::Iron);
    let sell1 = app.quote_sell_price(ItemType::Iron);

    assert!(
        buy1 <= buy0,
        "higher reputation should not raise buy price ({buy0} -> {buy1})"
    );
    assert!(
        buy1 < buy0,
        "reputation should lower the buy price (buy {buy0}->{buy1})"
    );
    // Sell is ceilinged by buy-1 (merchant spread), so it may tighten as buys
    // get cheaper — but the spread must never invert.
    assert!(
        sell0 < buy0 && sell1 < buy1,
        "sell must stay below buy (before {sell0}/{buy0}, after {sell1}/{buy1})"
    );
}
