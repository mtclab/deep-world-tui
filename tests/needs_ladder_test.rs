// Entity-first slice 3 (deep-world-godot#50): the hunger ladder. Each soul, when
// hungry, climbs eat -> buy -> work -> go hungry on its own coin and the town's
// stores. These are the divergences the aggregate per-head feed could never show:
// in a famine the moneyed buy through while the coinless poor go without, and the
// able-but-broke take work to earn for next time.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::Need;
use deep_world_tui::sim::SimState;

fn a_settlement_with_two(seed: u64) -> SimState {
    SimState::new_capped(seed, load_charts().expect("charts"), Some(40))
}

#[test]
fn in_a_famine_coin_buys_through_and_the_penniless_go_hungry() {
    let mut sim = a_settlement_with_two(42);
    let s = &mut sim.world.regions[0].settlements[0];
    assert!(s.people.len() >= 2, "need at least two residents");
    // A famine: empty granary, no work to be had.
    s.food_stock = 0.0;
    // Make the first two souls hungry; one holds coin, one is penniless.
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.3);
    }
    s.people[0].coins = 5;
    s.people[1].coins = 0;
    let food0_before = s.people[0].needs.get(Need::Food);
    let food1_before = s.people[1].needs.get(Need::Food);

    s.feed_people_ladder(0.15, 1, 1, /*work*/ false);

    // The moneyed soul bought a meal from outside: fed, a coin poorer.
    assert!(
        s.people[0].needs.get(Need::Food) > food0_before,
        "a soul with coin should buy through the famine"
    );
    assert_eq!(s.people[0].coins, 4, "the meal cost a coin");
    // The penniless soul, with no work to be had, went hungry.
    assert!(
        s.people[1].needs.get(Need::Food) < food1_before,
        "a penniless soul in a workless famine should go hungry"
    );
    assert_eq!(s.people[1].coins, 0);
}

#[test]
fn the_hungry_and_able_take_work_for_coin() {
    let mut sim = a_settlement_with_two(7);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0; // nothing in the granary to eat...
    let p = &mut s.people[0];
    p.needs.set(Need::Food, 0.3);
    p.coins = 0;
    let money_before = p.needs.get(Need::Money);

    s.feed_people_ladder(0.15, 1, 2, /*work*/ true);

    let p = &s.people[0];
    assert_eq!(
        p.coins, 2,
        "a hungry, broke soul takes work and earns a wage"
    );
    assert!(
        p.needs.get(Need::Money) > money_before,
        "working eases the money need"
    );
}

#[test]
fn a_full_granary_feeds_from_the_stores_not_the_purse() {
    let mut sim = a_settlement_with_two(99);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 100.0; // plenty
    let p = &mut s.people[0];
    p.needs.set(Need::Food, 0.3);
    p.coins = 5;
    let stock_before = s.food_stock;

    s.feed_people_ladder(0.15, 1, 1, true);

    assert_eq!(
        s.people[0].coins, 5,
        "no coin spent when the granary is full"
    );
    assert!(
        s.food_stock < stock_before,
        "the meal came out of the stores"
    );
    assert!(s.people[0].needs.get(Need::Food) > 0.3, "and the soul ate");
}
