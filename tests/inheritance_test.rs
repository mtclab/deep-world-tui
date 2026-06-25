// Households as economic units (deep-world-godot#56-F): a life's purse does not
// vanish into the ground. On death it passes to the nearest of kin still in town —
// spouse first, then child/sibling/parent — so a family's wealth, and its chance
// to rise, outlives any one of them. With no kin, it escheats to the commons.
use deep_world_tui::model::relation::{InterNpcRelation, RelationKind};
use deep_world_tui::sim::lifecycle::bequeath;
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn rel(kind: RelationKind, target: &str) -> InterNpcRelation {
    InterNpcRelation {
        kind,
        target_person_id: target.to_string(),
        intensity: 0.8,
        formed_at_tick: 0,
        reason: "kin".into(),
    }
}

#[test]
fn a_spouse_inherits_the_purse() {
    let mut sim = SimState::new_capped(42, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    assert!(s.people.len() >= 2);
    let mut dead = s.people[0].clone();
    dead.coins = 100;
    let spouse_id = s.people[1].id.clone();
    dead.relations = vec![rel(RelationKind::Spouse, &spouse_id)];
    let heir_before = s.people[1].coins;
    let mut treasury = s.treasury;
    bequeath(&dead, &mut s.people, &mut treasury);
    assert_eq!(
        s.people[1].coins,
        heir_before + 100,
        "the widow(er) inherits the purse"
    );
    assert_eq!(
        treasury, s.treasury,
        "the commons take nothing while kin live"
    );
}

#[test]
fn a_child_inherits_when_there_is_no_spouse() {
    let mut sim = SimState::new_capped(7, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    let mut dead = s.people[0].clone();
    dead.coins = 40;
    let child_id = s.people[1].id.clone();
    dead.relations = vec![rel(RelationKind::Child, &child_id)];
    let child_before = s.people[1].coins;
    let mut treasury = s.treasury;
    bequeath(&dead, &mut s.people, &mut treasury);
    assert_eq!(s.people[1].coins, child_before + 40, "the child inherits");
}

#[test]
fn a_kinless_purse_escheats_to_the_commons() {
    let mut sim = SimState::new_capped(99, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    let mut dead = s.people[0].clone();
    dead.coins = 25;
    dead.relations.clear(); // no kin to take it
    let coins_before: u32 = s.people.iter().map(|p| p.coins).sum();
    let mut treasury = 10u32;
    bequeath(&dead, &mut s.people, &mut treasury);
    assert_eq!(
        treasury, 35,
        "with no kin, the purse escheats to the common purse"
    );
    assert_eq!(
        s.people.iter().map(|p| p.coins).sum::<u32>(),
        coins_before,
        "and no living soul gained from it"
    );
}
