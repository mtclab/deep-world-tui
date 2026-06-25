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
    let ctx = town_context(s, 1.0, false, None, 0.15);
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
    let ctx = town_context(s, 1.0, false, None, 0.15);
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
    let ctx = town_context(s, 1.0, false, None, 0.15);
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
    let ctx = town_context(s, 1.0, false, None, 0.15);
    step_agents(s, &ctx); // granary is full, so the hungry eat
    assert!(
        s.people[0].needs.get(Need::Food) > food_before,
        "the hungry-and-lonely eat first — survival before company"
    );
}
