// Per-agent economy (deep-world-godot#54), slice 3: the common purse pays its
// makers. The coin a town takes in flows out to the hands that craft its goods —
// each producer earns from the treasury for its trade, a gifted maker more. It
// mints nothing (coin only moves from the purse to the makers), so the province's
// scarcity stands.
use deep_world_tui::model::gift::{CraftSense, Gift};
use deep_world_tui::sim::pay_makers;
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn town(seed: u64) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    for p in s.people.iter_mut() {
        p.profession = "labourer".into(); // no trade good — not a maker
        p.coins = 0;
        p.gift = Gift::default();
    }
    sim
}

fn total_coin(s: &deep_world_tui::model::economy::Settlement) -> u64 {
    s.treasury as u64 + s.people.iter().map(|p| p.coins as u64).sum::<u64>()
}

#[test]
fn the_purse_pays_makers_and_mints_nothing() {
    let mut sim = town(42);
    let s = &mut sim.world.regions[0].settlements[0];
    s.treasury = 100;
    s.people[0].profession = "smith".into();
    s.people[1].profession = "weaver".into();
    let before = total_coin(s);
    pay_makers(s);
    assert_eq!(total_coin(s), before, "coin is conserved — nothing minted");
    assert!(
        s.people[0].coins > 0 && s.people[1].coins > 0,
        "the makers were paid"
    );
    assert!(s.treasury < 100, "out of the common purse");
}

#[test]
fn a_gifted_maker_earns_more() {
    let mut sim = town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    s.treasury = 100; // ample, so both are paid their full worth
    s.people[0].profession = "smith".into();
    s.people[0].gift = Gift::of(CraftSense::IronEar); // a master smith
    s.people[1].profession = "smith".into(); // a journeyman
    pay_makers(s);
    assert!(
        s.people[0].coins > s.people[1].coins,
        "the master smith earns more than the journeyman ({} vs {})",
        s.people[0].coins,
        s.people[1].coins
    );
}

#[test]
fn the_payout_is_bounded_by_the_purse() {
    let mut sim = town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    s.treasury = 10; // a thin purse
    for p in s.people.iter_mut() {
        p.profession = "smith".into(); // many makers, far more owed than held
    }
    let before = s.treasury;
    pay_makers(s);
    // The maker-pay is gentle on the purse (an eighth at most per day), so a
    // famine-struck town keeps coin for the wages that feed its hungry — the
    // balance fix that kept towns from starving their work-rung dry.
    assert!(
        s.treasury >= before * 7 / 8,
        "no more than an eighth of the purse is paid out at once (was {before}, now {})",
        s.treasury
    );
    assert!(s.treasury < before, "but the makers were paid something");
}

#[test]
fn a_town_of_no_makers_pays_nothing() {
    let mut sim = town(123); // all labourers
    let s = &mut sim.world.regions[0].settlements[0];
    s.treasury = 100;
    pay_makers(s);
    assert_eq!(s.treasury, 100, "with no makers, the purse is untouched");
}
