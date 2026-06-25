// Crime & justice (deep-world-godot#56-E): theft breeds a feud (#713), and now
// the law answers it. Where there are guards a thief may be caught in the act —
// nothing is taken, the thief is marked — and a hardened repeat offender is driven
// out. A known thief is shunned: the neighbours who would help an honest soul turn
// it away.
use deep_world_tui::model::relation::{InterNpcRelation, RelationKind};
use deep_world_tui::model::Need;
use deep_world_tui::sim::agency::{step_agents, town_context};
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

/// A famine town full of would-be thieves and one rich victim. `guards` sets the
/// law. Returns the sim; settlement 0 is the scene.
fn thief_town(seed: u64, guards: usize) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0;
    s.treasury = 0;
    for p in s.people.iter_mut() {
        p.age_band = "adult".into();
        p.needs.set(Need::Food, 0.05);
        p.coins = 0;
        p.personality = vec!["devious".into(), "ruthless".into()]; // all would steal
        p.relations.clear();
        p.crimes = 0;
    }
    // One rich victim with no kin tie to anyone.
    s.people[0].coins = 500;
    s.people[0].personality = vec!["gentle".into()]; // the victim won't steal
    s.people[0].needs.set(Need::Food, 0.9); // and is sated, so it just stands there
    for g in 0..guards {
        s.people[1 + g].profession = "guard".into();
    }
    sim
}

#[test]
fn without_law_thieves_take_freely() {
    let mut sim = thief_town(42, 0);
    let s = &mut sim.world.regions[0].settlements[0];
    let victim_before = s.people[0].coins;
    step_agents(s, &town_context(s, 1.0, false, None, 0.15, 0));
    assert!(
        s.people[0].coins < victim_before,
        "with no law, the rich are robbed"
    );
    let crimes: u32 = s.people.iter().map(|p| p.crimes as u32).sum();
    assert_eq!(crimes, 0, "no one is caught where there are no guards");
}

#[test]
fn the_law_catches_thieves_and_spares_the_victim() {
    let mut no_law = thief_town(42, 0);
    let mut with_law = thief_town(42, 3); // a real watch
    let v_before = no_law.world.regions[0].settlements[0].people[0].coins;

    let s0 = &mut no_law.world.regions[0].settlements[0];
    step_agents(s0, &town_context(s0, 1.0, false, None, 0.15, 0));
    let s1 = &mut with_law.world.regions[0].settlements[0];
    step_agents(s1, &town_context(s1, 1.0, false, None, 0.15, 0));

    let lost_no_law = v_before - no_law.world.regions[0].settlements[0].people[0].coins;
    let lost_with_law = v_before - with_law.world.regions[0].settlements[0].people[0].coins;
    let crimes_marked: u32 = with_law.world.regions[0].settlements[0]
        .people
        .iter()
        .map(|p| p.crimes as u32)
        .sum();
    assert!(crimes_marked > 0, "the watch catches thieves in the act");
    assert!(
        lost_with_law < lost_no_law,
        "and what the law foils, the victim keeps ({lost_with_law} vs {lost_no_law})"
    );
}

#[test]
fn a_known_thief_is_shunned_from_charity() {
    let mut sim = SimState::new_capped(7, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0;
    s.treasury = 0;
    assert!(s.people.len() >= 2);
    for p in s.people.iter_mut() {
        p.age_band = "adult".into();
        p.needs.set(Need::Food, 0.9); // sated bystanders
        p.coins = 0;
    }
    // A known thief, starving and broke, with a sworn friend who has coin.
    let friend_id = s.people[1].id.clone();
    s.people[1].coins = 5;
    s.people[0].needs.set(Need::Food, 0.05);
    s.people[0].crimes = 2; // shunned
    s.people[0].personality = vec!["gentle".into()]; // would beg, not steal
    s.people[0].relations = vec![InterNpcRelation {
        kind: RelationKind::SwornFriend,
        target_person_id: friend_id,
        intensity: 0.6,
        formed_at_tick: 0,
        reason: "old friends".into(),
    }];
    step_agents(s, &town_context(s, 1.0, false, None, 0.15, 0));
    assert_eq!(
        s.people[1].coins, 5,
        "the friend gives the known thief nothing — it is shunned"
    );
    assert_eq!(s.people[0].coins, 0, "and the thief gets no charity");
}
