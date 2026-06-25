// Per-agent economy (deep-world-godot#54), slice 1: work is plying your trade.
// What a soul's labour earns reflects the worth of its craft, and a craftsperson's
// hands put their good on the town's shelf — the first step toward an economy that
// is the sum of individual trades.
use deep_world_tui::model::economy::{trade_good, trade_wage, ItemType};
use deep_world_tui::model::Need;
use deep_world_tui::sim::agency::{step_agents, town_context};
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

/// A town where the only way to eat is to work: empty granary, full treasury.
fn work_town(seed: u64, profession: &str) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0;
    s.treasury = 100;
    s.goods_stock.clear();
    for p in s.people.iter_mut() {
        p.age_band = "adult".into();
        p.profession = profession.into();
        p.needs.set(Need::Food, 0.9); // sated...
        p.coins = 0;
    }
    s.people[0].needs.set(Need::Food, 0.3); // ...except the one who must work
    sim
}

#[test]
fn trade_good_and_wage_map_the_crafts() {
    assert_eq!(trade_good("smith"), Some(ItemType::Tool));
    assert_eq!(trade_good("weaver"), Some(ItemType::Cloth));
    assert_eq!(trade_good("labourer"), None);
    assert_eq!(trade_good("farmer"), None);
    assert!(trade_wage("smith") > trade_wage("labourer"));
}

#[test]
fn a_smith_at_work_earns_a_skilled_wage_and_makes_a_tool() {
    let mut sim = work_town(42, "smith");
    let s = &mut sim.world.regions[0].settlements[0];
    let treasury_before = s.treasury;
    step_agents(s, &town_context(s, 1.0, false, None, 0.15, 0));
    assert_eq!(
        s.people[0].coins, 2,
        "a smith's labour earns the skilled wage"
    );
    assert_eq!(
        s.treasury,
        treasury_before - 2,
        "paid out of the town purse"
    );
    assert!(
        s.good(ItemType::Tool) >= 0.1,
        "and the smith's hands put a tool on the shelf ({})",
        s.good(ItemType::Tool)
    );
}

#[test]
fn a_labourer_earns_common_wage_and_makes_no_good() {
    let mut sim = work_town(7, "labourer");
    let s = &mut sim.world.regions[0].settlements[0];
    step_agents(s, &town_context(s, 1.0, false, None, 0.15, 0));
    assert_eq!(s.people[0].coins, 1, "common labour earns the base wage");
    // No tradeable good was made (labour yields no craft of its own).
    assert!(s.good(ItemType::Tool) <= 0.0);
}
