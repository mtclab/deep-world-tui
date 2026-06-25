// Belief colours the person (deep-world-godot#56-H): where a temple or shrine
// stands, the devout give of what they have — a coin of alms to the commons.
// Culture shaping the economy, soul by soul.
use deep_world_tui::model::economy::SettlementService;
use deep_world_tui::model::Need;
use deep_world_tui::sim::aspiration::tick_settlement_aspirations;
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn settled_town(seed: u64) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.9);
        p.needs.set(Need::Safety, 0.9);
        p.has_spouse = true; // settled — no one chasing marriage/trade
        p.coins = 0;
        p.personality.clear();
    }
    sim
}

#[test]
fn the_devout_tithe_where_a_temple_stands() {
    let mut sim = settled_town(42);
    let s = &mut sim.world.regions[0].settlements[0];
    s.services = vec![SettlementService::Temple];
    s.treasury = 0;
    s.people[0].personality = vec!["devout".into()];
    s.people[0].coins = 5;
    tick_settlement_aspirations(s, 42, 24);
    assert_eq!(s.people[0].coins, 4, "the devout soul gave a coin of alms");
    assert!(s.treasury >= 1, "and the temple's common purse swelled");
}

#[test]
fn the_worldly_do_not_tithe() {
    let mut sim = settled_town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    s.services = vec![SettlementService::Temple];
    s.treasury = 0;
    s.people[0].personality = vec!["proud".into()]; // not devout
    s.people[0].coins = 5;
    tick_settlement_aspirations(s, 7, 24);
    assert_eq!(s.people[0].coins, 5, "a worldly soul keeps its coin");
}

#[test]
fn no_temple_no_tithe() {
    let mut sim = settled_town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    s.services.clear(); // no temple or shrine
    s.treasury = 0;
    s.people[0].personality = vec!["devout".into()];
    s.people[0].coins = 5;
    tick_settlement_aspirations(s, 99, 24);
    assert_eq!(
        s.people[0].coins, 5,
        "with no temple to give to, even the devout keep their coin"
    );
}
