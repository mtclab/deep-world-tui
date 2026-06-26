// Per-agent economy (deep-world-godot#54), slice 2: a trade's output is the sum
// of its producers' worth, not a flat head-count. A craftsperson whose innate
// craft-gift (#441) matches the trade makes half again as much — so a town's
// goods read from WHO keeps its trades, not only how many. An ungifted town is
// exactly its head-count (parity).
use deep_world_tui::model::gift::{CraftSense, Gift};
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn smith_town(seed: u64) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    // Three smiths, all ungifted to start.
    for (k, p) in s.people.iter_mut().enumerate() {
        p.profession = if k < 3 {
            "smith".into()
        } else {
            "labourer".into()
        };
        p.gift = Gift::default();
    }
    sim
}

#[test]
fn an_ungifted_trade_is_exactly_its_head_count() {
    let sim = smith_town(42);
    let s = &sim.world.regions[0].settlements[0];
    assert_eq!(s.profession_count("smith"), 3);
    assert!(
        (s.trade_power("smith") - 3.0).abs() < 1e-9,
        "parity with the count"
    );
}

#[test]
fn a_matching_gift_makes_a_trade_worth_more() {
    let mut sim = smith_town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    // Give one smith the forge-sense (iron-ear).
    s.people[0].gift = Gift::of(CraftSense::IronEar);
    assert!(
        (s.trade_power("smith") - 3.5).abs() < 1e-9,
        "a master smith is worth half again as much ({})",
        s.trade_power("smith")
    );
    assert_eq!(
        s.profession_count("smith"),
        3,
        "but it is still three heads"
    );
}

#[test]
fn a_gift_of_the_wrong_sense_does_not_help() {
    let mut sim = smith_town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    // A water-sense gift is no use at the forge.
    s.people[0].gift = Gift::of(CraftSense::ScaleHand);
    assert!(
        (s.trade_power("smith") - 3.0).abs() < 1e-9,
        "a gift of the wrong craft is no smith's boon"
    );
}
