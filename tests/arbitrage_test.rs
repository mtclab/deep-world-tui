// Per-agent economy (deep-world-godot#54), slice 6: merchant arbitrage. A trader
// profits by carrying its town's goods SURPLUS to the wider world. Only where a
// town holds more than it keeps for itself do its traders earn a commission for
// brokering it — a gifted weigh-sense trader more — paid out of the town's share
// of the sale. It mints nothing.
use deep_world_tui::model::economy::ItemType;
use deep_world_tui::model::gift::{CraftSense, Gift};
use deep_world_tui::sim::pay_traders;
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn town(seed: u64) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    for p in s.people.iter_mut() {
        p.profession = "labourer".into();
        p.coins = 0;
        p.gift = Gift::default();
    }
    s.goods_stock.clear();
    s.treasury = 100;
    sim
}

fn total_coin(s: &deep_world_tui::model::economy::Settlement) -> u64 {
    s.treasury as u64 + s.people.iter().map(|p| p.coins as u64).sum::<u64>()
}

#[test]
fn a_trader_profits_from_a_towns_surplus() {
    let mut sim = town(42);
    let s = &mut sim.world.regions[0].settlements[0];
    s.people[0].profession = "trader".into();
    // A real surplus of goods to broker.
    s.goods_stock.insert(ItemType::Tool, 200.0);
    let before = total_coin(s);
    pay_traders(s);
    assert!(s.people[0].coins > 0, "the trader earned a commission");
    assert!(s.treasury < 100, "out of the town's share of the sale");
    assert_eq!(total_coin(s), before, "and nothing was minted — conserved");
}

#[test]
fn no_surplus_no_commission() {
    let mut sim = town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    s.people[0].profession = "trader".into();
    s.goods_stock.clear(); // the town has nothing spare to trade
    pay_traders(s);
    assert_eq!(
        s.people[0].coins, 0,
        "with no surplus, the trader brokers nothing"
    );
    assert_eq!(s.treasury, 100, "and the purse is untouched");
}

#[test]
fn a_gifted_trader_earns_more() {
    let mut sim = town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    s.goods_stock.insert(ItemType::Tool, 200.0);
    s.people[0].profession = "trader".into();
    s.people[0].gift = Gift::of(CraftSense::ScaleHand); // the weigh-sense
    s.people[1].profession = "trader".into(); // a plain trader
    pay_traders(s);
    assert!(
        s.people[0].coins > s.people[1].coins,
        "the weigh-sense trader earns more ({} vs {})",
        s.people[0].coins,
        s.people[1].coins
    );
}
