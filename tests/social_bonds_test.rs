// Social bonds (#318): warmth accumulated through dealings (NPC memory) makes
// friends — friends vouch for cheaper services, a friend's death is grieved
// by name, and your closest friend continues the story when you die.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{EncounterAction, SettlementService};
use deep_world_tui::sim::SimState;
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

fn befriend(a: &mut App, person_idx: usize, trust: f64) {
    let id = a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[person_idx]
        .id
        .clone();
    a.sim
        .as_mut()
        .unwrap()
        .npc_memories
        .entry(id)
        .or_default()
        .add(EncounterAction::Trade, 0, "here".into(), trust);
}

#[test]
fn a_friend_in_town_lowers_service_costs() {
    let mut plain = app(42);
    let mut befriended = app(42);
    for a in [&mut plain, &mut befriended] {
        a.clock.hour = 10;
        a.player_start
            .as_mut()
            .unwrap()
            .inventory
            .add(deep_world_tui::model::ItemType::Coin, 30);
        a.enter_settlement(0, 0);
    }
    befriend(&mut befriended, 0, 0.3);
    let coins = |a: &App| {
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(deep_world_tui::model::ItemType::Coin)
    };
    let b0 = coins(&plain);
    plain.use_service(SettlementService::Temple);
    let plain_cost = b0 - coins(&plain);
    let b1 = coins(&befriended);
    befriended.use_service(SettlementService::Temple);
    let friend_cost = b1 - coins(&befriended);
    assert!(
        friend_cost < plain_cost,
        "a friend should vouch a coin off ({friend_cost} vs {plain_cost})"
    );
}

#[test]
fn a_friends_death_is_grieved_by_name() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    let (id, name) = {
        let p = &mut sim.world.regions[0].settlements[0].people[0];
        p.age_band = "aged".into();
        p.age_years = 90;
        (p.id.clone(), p.name.clone())
    };
    sim.npc_memories
        .entry(id)
        .or_default()
        .add(EncounterAction::Talk, 0, "here".into(), 0.3);
    // Run years until they die (aged mortality is high).
    for _ in 0..10 {
        sim.world.tick = ((sim.world.tick / deep_world_tui::sim::lifecycle::TICKS_PER_LIFE_YEAR)
            + 1)
            * deep_world_tui::sim::lifecycle::TICKS_PER_LIFE_YEAR
            - 1;
        sim.step();
    }
    let grieved = sim
        .journal
        .iter()
        .any(|e| e.voice == deep_world_tui::sim::journal::Voice::Scar && e.text.contains(&name));
    assert!(
        grieved,
        "a friend's death should be grieved by name ({name})"
    );
}

#[test]
fn the_closest_friend_inherits_the_story() {
    let mut a = app(42);
    befriend(&mut a, 2, 0.5);
    let friend_id = a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[2]
        .id
        .clone();
    // Force an old-age death -> heir handoff.
    a.start_age_years = 80;
    a.birth_day = a.clock.day;
    a.lifespan_years = 80;
    a.vitals.hunger = 1.0;
    a.vitals.thirst = 1.0;
    a.vitals.energy = 1.0;
    a.advance_clock(1);
    let heir = a.player_start.as_ref().expect("heir");
    assert_eq!(
        heir.person.id, friend_id,
        "the heir should be the player's closest friend"
    );
}
