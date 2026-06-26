// Per-agent economy (deep-world-godot#54), slice 5: a producer owns the goods it
// makes. It keeps a little of what it crafts as its own ware, sells a piece to the
// market to eat when the granary fails (a smith's coin from a tool it made), and
// leaves the rest to its heirs.
use deep_world_tui::model::economy::ItemType;
use deep_world_tui::model::relation::{InterNpcRelation, RelationKind};
use deep_world_tui::model::Need;
use deep_world_tui::sim::agency::{step_agents, town_context};
use deep_world_tui::sim::lifecycle::bequeath;
use deep_world_tui::sim::{pay_makers, SimState};

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn town(seed: u64) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    for p in s.people.iter_mut() {
        p.profession = "labourer".into();
        p.coins = 0;
        p.wares = 0;
        p.age_band = "adult".into();
    }
    sim
}

#[test]
fn a_paid_maker_keeps_a_ware() {
    let mut sim = town(42);
    let s = &mut sim.world.regions[0].settlements[0];
    s.treasury = 100;
    s.people[0].profession = "smith".into();
    pay_makers(s);
    assert!(s.people[0].coins > 0, "the maker was paid");
    assert_eq!(s.people[0].wares, 1, "and kept a piece of its craft");
}

#[test]
fn a_producer_sells_its_craft_to_eat() {
    let mut sim = town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0; // famine: nothing in the granary
    s.treasury = 10;
    s.goods_stock.clear();
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.9); // sated bystanders
    }
    // A starving, penniless smith holding three tools it made.
    s.people[0].profession = "smith".into();
    s.people[0].needs.set(Need::Food, 0.3);
    s.people[0].coins = 0;
    s.people[0].wares = 3;
    step_agents(s, &town_context(s, 1.0, false, None, 0.15, 0));
    assert_eq!(s.people[0].wares, 2, "it sold one of its tools");
    assert!(s.people[0].coins > 0, "for coin");
    assert!(s.treasury < 10, "the market's purse paid for it");
    assert!(
        s.good(ItemType::Tool) >= 1.0,
        "and the tool joined the town's shelf"
    );
}

#[test]
fn goods_pass_to_the_heir() {
    let mut sim = town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    let mut dead = s.people[0].clone();
    dead.coins = 0;
    dead.wares = 5;
    let heir_id = s.people[1].id.clone();
    dead.relations = vec![InterNpcRelation {
        kind: RelationKind::Child,
        target_person_id: heir_id,
        intensity: 0.8,
        formed_at_tick: 0,
        reason: "kin".into(),
    }];
    let heir_wares_before = s.people[1].wares;
    let mut treasury = s.treasury;
    bequeath(&dead, &mut s.people, &mut treasury);
    assert_eq!(
        s.people[1].wares,
        heir_wares_before + 5,
        "the heir inherits the goods of the trade, not only the coin"
    );
}

#[test]
fn a_kinless_hoard_is_sold_into_the_commons() {
    let mut sim = town(123);
    let s = &mut sim.world.regions[0].settlements[0];
    let mut dead = s.people[0].clone();
    dead.coins = 3;
    dead.wares = 4;
    dead.relations.clear();
    let mut treasury = 0u32;
    bequeath(&dead, &mut s.people, &mut treasury);
    assert_eq!(
        treasury, 7,
        "coin escheats and the goods are sold into the commons"
    );
}
