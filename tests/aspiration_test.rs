// Purposeful agents (deep-world-godot#53): a settled soul works toward a standing
// aspiration over many days — to master a trade, to marry — resolving into a real
// life event. A soul in survival crisis tends to that first and dreams later.
use deep_world_tui::model::relation::RelationKind;
use deep_world_tui::model::Need;
use deep_world_tui::sim::aspiration::{tick_settlement_aspirations, Aspiration};
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn town(seed: u64) -> SimState {
    SimState::new_capped(seed, charts(), Some(40))
}

fn settle(p: &mut deep_world_tui::model::Person) {
    p.needs.set(Need::Food, 0.9);
    p.needs.set(Need::Safety, 0.9);
    p.needs.set(Need::Care, 0.9);
    p.needs.set(Need::Presence, 0.9);
}

#[test]
fn a_settled_youth_learns_a_trade_present_in_town() {
    let mut sim = town(42);
    let s = &mut sim.world.regions[0].settlements[0];
    assert!(s.people.len() >= 2);
    for p in s.people.iter_mut() {
        settle(p);
        p.profession = "labourer".into(); // unskilled all round...
        p.has_spouse = true; // ...and wed, so no one chases marriage
        p.aspiration = None;
    }
    // ...except one smith to learn from, and one trade-less youth to learn.
    s.people[1].profession = "smith".into();
    s.people[0].age_band = "youth".into();
    s.people[0].profession = "labourer".into();

    for day in 0..30u64 {
        tick_settlement_aspirations(s, 42, day * 24);
    }
    assert_eq!(
        s.people[0].profession, "smith",
        "the youth came into the smith's trade"
    );
}

#[test]
fn two_settled_unwed_adults_marry() {
    let mut sim = town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    assert!(s.people.len() >= 2);
    for p in s.people.iter_mut() {
        settle(p);
        p.has_spouse = true; // most already wed, so they don't compete
        p.aspiration = None;
    }
    // Two free souls of marrying years.
    for k in 0..2 {
        s.people[k].age_band = "adult".into();
        s.people[k].has_spouse = false;
    }
    let a_id = s.people[0].id.clone();
    let b_id = s.people[1].id.clone();

    for day in 0..30u64 {
        tick_settlement_aspirations(s, 7, day * 24);
    }
    assert!(s.people[0].has_spouse && s.people[1].has_spouse, "both wed");
    assert!(
        s.people[0]
            .relations
            .iter()
            .any(|r| r.kind == RelationKind::Spouse && r.target_person_id == b_id),
        "and hold a spouse bond to each other"
    );
    assert!(s.people[1]
        .relations
        .iter()
        .any(|r| r.kind == RelationKind::Spouse && r.target_person_id == a_id));
}

#[test]
fn a_soul_in_crisis_does_not_chase_ambitions() {
    let mut sim = town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    let p = &mut s.people[0];
    p.age_band = "youth".into();
    p.profession = "labourer".into();
    p.has_spouse = false;
    p.aspiration = None;
    p.needs.set(Need::Food, 0.1); // starving — no time for dreams
    p.needs.set(Need::Safety, 0.2);

    for day in 0..10u64 {
        tick_settlement_aspirations(s, 99, day * 24);
    }
    assert!(
        s.people[0].aspiration.is_none(),
        "a soul in survival crisis takes up no aspiration"
    );
}

#[test]
fn aspiration_label_reads() {
    let a = Aspiration::LearnTrade {
        trade: "smith".into(),
        progress: 0.3,
    };
    assert!(a.label().contains("smith"));
    assert!(Aspiration::Marry { progress: 0.0 }
        .label()
        .contains("marry"));
}
