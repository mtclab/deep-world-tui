// Entity-first slice 3 (deep-world-godot#50): the hunger ladder. Each soul, when
// hungry, climbs eat -> buy -> work -> go hungry on its own coin and the town's
// stores. Slice 4: the coin moves through the town treasury, so trade is
// conserved. These are the divergences the aggregate per-head feed could never
// show: in a famine the moneyed buy through while the coinless poor go without,
// and the able-but-broke take work to earn for next time.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::Need;
use deep_world_tui::sim::SimState;

fn a_settlement(seed: u64) -> SimState {
    SimState::new_capped(seed, load_charts().expect("charts"), Some(40))
}

#[test]
fn in_a_famine_coin_buys_a_meal_from_outside() {
    let mut sim = a_settlement(42);
    let s = &mut sim.world.regions[0].settlements[0];
    // A famine: empty granary. Only the first soul is hungry, and it holds coin.
    s.food_stock = 0.0;
    s.treasury = 0;
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.9); // sated — they don't act
    }
    s.people[0].needs.set(Need::Food, 0.3);
    s.people[0].coins = 5;
    let food0 = s.people[0].needs.get(Need::Food);

    s.feed_people_ladder(0.15, 1, 1);

    assert!(
        s.people[0].needs.get(Need::Food) > food0,
        "coin buys through"
    );
    assert_eq!(s.people[0].coins, 4, "the meal cost a coin");
    assert_eq!(s.treasury, 1, "the price went into the town treasury");
}

#[test]
fn a_coinless_town_in_famine_starves() {
    let mut sim = a_settlement(555);
    let s = &mut sim.world.regions[0].settlements[0];
    // Empty granary, empty treasury, every purse empty — nothing to eat, nothing
    // to buy with, no coin to pay any wage. The whole town goes hungry.
    s.food_stock = 0.0;
    s.treasury = 0;
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.3);
        p.coins = 0;
    }
    let before: Vec<f64> = s.people.iter().map(|p| p.needs.get(Need::Food)).collect();

    s.feed_people_ladder(0.15, 1, 1);

    for (p, b) in s.people.iter().zip(before) {
        assert!(p.needs.get(Need::Food) < b, "a coinless famine starves all");
        assert_eq!(p.coins, 0, "no coin appears from nowhere");
    }
    assert_eq!(s.treasury, 0);
}

#[test]
fn the_hungry_and_able_take_work_paid_from_the_treasury() {
    let mut sim = a_settlement(7);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0; // nothing in the granary...
    s.treasury = 100; // ...but the town can pay a wage
    let p = &mut s.people[0];
    p.needs.set(Need::Food, 0.3);
    p.coins = 0;
    let money_before = p.needs.get(Need::Money);

    s.feed_people_ladder(0.15, 1, 2);

    assert_eq!(s.people[0].coins, 2, "took work, earned the wage");
    assert_eq!(s.treasury, 98, "the wage came out of the treasury");
    assert!(
        s.people[0].needs.get(Need::Money) > money_before,
        "work eases the money need"
    );
}

#[test]
fn a_full_granary_feeds_from_the_stores_not_the_purse() {
    let mut sim = a_settlement(99);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 100.0; // plenty
    let p = &mut s.people[0];
    p.needs.set(Need::Food, 0.3);
    p.coins = 5;
    let stock_before = s.food_stock;
    let treasury_before = s.treasury;

    s.feed_people_ladder(0.15, 1, 1);

    assert_eq!(
        s.people[0].coins, 5,
        "no coin spent when the granary is full"
    );
    assert_eq!(s.treasury, treasury_before, "and none changed hands");
    assert!(
        s.food_stock < stock_before,
        "the meal came out of the stores"
    );
    assert!(s.people[0].needs.get(Need::Food) > 0.3, "and the soul ate");
}

#[test]
fn coin_is_conserved_across_the_ladder() {
    // Slice 4 invariant: buying and working only MOVE coin between purses and the
    // treasury — they never mint or burn it. The town's total coin holds steady.
    let mut sim = a_settlement(123);
    let s = &mut sim.world.regions[0].settlements[0];
    // A mixed lot: an empty granary so everyone must buy or work, a treasury that
    // can pay some wages, and a spread of purses.
    s.food_stock = 0.0;
    s.treasury = 30;
    for (i, p) in s.people.iter_mut().enumerate() {
        p.needs.set(Need::Food, 0.2);
        p.coins = (i as u32) % 4; // 0,1,2,3,0,1,...
    }
    let total_before: u64 =
        s.treasury as u64 + s.people.iter().map(|p| p.coins as u64).sum::<u64>();

    for _ in 0..10 {
        s.feed_people_ladder(0.15, 1, 1);
    }

    let total_after: u64 = s.treasury as u64 + s.people.iter().map(|p| p.coins as u64).sum::<u64>();
    assert_eq!(
        total_before, total_after,
        "the ladder moves coin, it never mints or burns it"
    );
}
