// Entity-first slice 8 (deep-world-godot#50): the utility-driven needs selector.
// A soul acts on its MOST pressing unmet drive, not only hunger — and only on a
// drive it has the means to serve. Survival (Food, Safety) outweighs care and
// company; Money is instrumental (served inside the Food column), never a
// terminal drive.
use deep_world_tui::model::economy::SettlementService;
use deep_world_tui::model::Need;
use deep_world_tui::sim::agency::{step_agents, town_context};
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

/// A one-resident, fully-stocked town so we can isolate a single drive.
fn solo_town(seed: u64) -> SimState {
    let mut sim = SimState::new_capped(seed, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 1000.0; // never hungry
    s.treasury = 1000;
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.9);
        p.needs.set(Need::Care, 0.9);
        p.needs.set(Need::Presence, 0.9);
        p.needs.set(Need::Safety, 0.9);
    }
    sim
}

#[test]
fn the_sick_are_eased_where_a_healer_lives() {
    let mut sim = solo_town(42);
    let s = &mut sim.world.regions[0].settlements[0];
    s.people[0].profession = "healer".into(); // a healer lives here
    s.people[0].needs.set(Need::Care, 0.3); // and someone ails
    let before = s.people[0].needs.get(Need::Care);
    let ctx = town_context(s, 1.0, false, None, 0.15, 0);
    step_agents(s, &ctx);
    assert!(
        s.people[0].needs.get(Need::Care) > before,
        "a healer eases the ailing"
    );
}

#[test]
fn without_a_healer_the_ailing_decline() {
    let mut sim = solo_town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    for p in s.people.iter_mut() {
        p.profession = "farmer".into(); // no healer
        p.needs.set(Need::Care, 0.3);
    }
    let before = s.people[0].needs.get(Need::Care);
    let ctx = town_context(s, 1.0, false, None, 0.15, 0);
    step_agents(s, &ctx);
    assert!(
        s.people[0].needs.get(Need::Care) < before,
        "with no healer, care goes unmet"
    );
}

#[test]
fn the_lonely_find_company_at_the_tavern() {
    let mut sim = solo_town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    s.services = vec![SettlementService::Tavern];
    for p in s.people.iter_mut() {
        p.has_spouse = false;
        p.needs.set(Need::Presence, 0.3);
    }
    let before = s.people[0].needs.get(Need::Presence);
    let ctx = town_context(s, 1.0, false, None, 0.15, 0);
    step_agents(s, &ctx);
    assert!(
        s.people[0].needs.get(Need::Presence) > before,
        "a tavern answers loneliness"
    );
}

#[test]
fn survival_outranks_company() {
    // A soul both hungry and lonely tends to its hunger first.
    let mut sim = solo_town(123);
    let s = &mut sim.world.regions[0].settlements[0];
    s.services.clear(); // no company to be had anyway
    for p in s.people.iter_mut() {
        p.has_spouse = false;
        p.needs.set(Need::Food, 0.3); // hungry
        p.needs.set(Need::Presence, 0.2); // and lonelier still
    }
    let food_before = s.people[0].needs.get(Need::Food);
    let ctx = town_context(s, 1.0, false, None, 0.15, 0);
    step_agents(s, &ctx); // granary is full, so the hungry eat
    assert!(
        s.people[0].needs.get(Need::Food) > food_before,
        "the hungry-and-lonely eat first — survival before company"
    );
}

// ---- slice 9: need-satisfaction runs through the social fabric, health is real ----
use deep_world_tui::model::economy::{ActiveDisease, Disease};
use deep_world_tui::model::relation::{InterNpcRelation, RelationKind};

#[test]
fn a_healer_actually_tends_the_sick() {
    let mut sim = solo_town(2024);
    let s = &mut sim.world.regions[0].settlements[0];
    s.people[0].profession = "healer".into();
    s.people[0].needs.set(Need::Care, 0.3);
    s.people[0].illnesses = vec![ActiveDisease {
        disease: Disease::Fever,
        contracted_tick: 1000,
        severity: 1.4, // a worsened case
    }];
    let ctx = town_context(s, 1.0, false, None, 0.15, 1000);
    step_agents(s, &ctx);
    assert!(
        s.people[0].illnesses[0].severity < 1.4,
        "the healer eased the illness ({})",
        s.people[0].illnesses[0].severity
    );
}

#[test]
fn the_starving_beg_of_kin_before_robbing_a_stranger() {
    let mut sim = solo_town(2025);
    let s = &mut sim.world.regions[0].settlements[0];
    // Famine: no granary, no work-fund.
    s.food_stock = 0.0;
    s.treasury = 0;
    assert!(s.people.len() >= 3, "need a beggar, a kin, and a stranger");
    // Everyone sated (so only the beggar acts) and penniless to start.
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.9);
        p.coins = 0;
    }
    // person[0] alone is starving and would steal if it had to; person[1] is a
    // sworn friend with coin; person[2] is a richer stranger (no tie).
    s.people[0].needs.set(Need::Food, 0.05);
    let friend_id = s.people[1].id.clone();
    s.people[1].coins = 3;
    s.people[2].coins = 50;
    s.people[0].personality = vec!["bitter".into()];
    s.people[0].relations = vec![InterNpcRelation {
        kind: RelationKind::SwornFriend,
        target_person_id: friend_id.clone(),
        intensity: 0.5,
        formed_at_tick: 0,
        reason: "old friends".into(),
    }];
    let ctx = town_context(s, 1.0, false, None, 0.15, 10);
    step_agents(s, &ctx);
    // The friend gave (their coin fell); the rich stranger was not robbed.
    assert!(s.people[1].coins < 3, "the kin shared their coin");
    assert_eq!(
        s.people[2].coins, 50,
        "the stranger was not robbed — kin came first"
    );
    // Gratitude deepened the bond.
    assert!(
        s.people[0].relations[0].intensity > 0.5,
        "charity is remembered ({})",
        s.people[0].relations[0].intensity
    );
}

#[test]
fn robbing_a_stranger_sows_a_feud() {
    let mut sim = solo_town(2026);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0;
    s.treasury = 0;
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.05);
        p.coins = 0;
        p.relations.clear();
    }
    // A hard-hearted starving thief and a rich stranger with no tie to anyone.
    s.people[0].personality = vec!["devious".into(), "ruthless".into()];
    let rich = s
        .people
        .iter()
        .enumerate()
        .max_by_key(|(_, p)| p.coins)
        .map(|(i, _)| i)
        .unwrap();
    s.people[rich].coins = 0;
    s.people[1].coins = 40; // the mark
    let thief_id = s.people[0].id.clone();
    let ctx = town_context(s, 1.0, false, None, 0.15, 20);
    step_agents(s, &ctx);
    // The robbed neighbour now feuds with the thief.
    assert!(
        s.people[1]
            .relations
            .iter()
            .any(|r| r.kind == RelationKind::Feud && r.target_person_id == thief_id),
        "the robbed remember the thief"
    );
    assert!(s.people[1].coins < 40, "and they are the poorer for it");
}
