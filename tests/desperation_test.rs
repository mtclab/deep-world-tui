// Entity-first slice 5 (deep-world-godot#50): desperation -> crime -> banditry,
// emergent. A soul that falls off the bottom of the hunger ladder (chronic
// hunger, empty purse, no work) steals from a wealthier neighbour; and when the
// whole town is as destitute as it is, it takes to the road — a real body fed
// into the frontier's bands, banditry born of provincial scarcity, not a roll.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::Need;
use deep_world_tui::sim::SimState;

fn small_sim(seed: u64) -> SimState {
    SimState::new_capped(seed, load_charts().expect("charts"), Some(40))
}

/// Step exactly onto the next daily boundary so the settlement pass runs once.
fn run_one_day(sim: &mut SimState) {
    sim.world.tick = 23;
    sim.step();
}

#[test]
fn a_destitute_starving_town_sheds_its_people_to_the_frontier() {
    let mut sim = small_sim(42);
    {
        // A barren region so nothing is hunted or trapped back into the stores.
        sim.world.regions[0].game_richness = 0.0;
        let s = &mut sim.world.regions[0].settlements[0];
        s.food_stock = 0.0;
        s.treasury = 0;
        s.farms.clear(); // no harvest to refill the granary
        for p in s.people.iter_mut() {
            p.profession = "labourer".into(); // no food-producers among them
            p.needs.set(Need::Food, 0.05);
            p.coins = 0;
        }
    }
    let pop_before = sim.world.regions[0].settlements[0].people.len();
    let wanderers_before = sim.frontier.wanderers;

    run_one_day(&mut sim);

    let pop_after = sim.world.regions[0].settlements[0].people.len();
    assert!(
        pop_after < pop_before,
        "a town with nothing — no food, no coin, no work — loses its people ({pop_before} -> {pop_after})"
    );
    assert!(
        sim.frontier.wanderers > wanderers_before,
        "and they swell the frontier's wanderers ({wanderers_before} -> {})",
        sim.frontier.wanderers
    );
}

#[test]
fn the_desperate_rob_a_rich_neighbour_before_taking_the_road() {
    let mut sim = small_sim(7);
    let rich_id;
    {
        sim.world.regions[0].game_richness = 0.0;
        let s = &mut sim.world.regions[0].settlements[0];
        assert!(s.people.len() >= 3, "need a few residents");
        s.food_stock = 0.0;
        s.treasury = 0;
        s.farms.clear();
        for p in s.people.iter_mut() {
            p.profession = "labourer".into();
            p.needs.set(Need::Food, 0.05); // starving
            p.coins = 0;
        }
        // One sated, wealthy neighbour — the others will rob them rather than
        // immediately taking to the road.
        s.people[0].needs.set(Need::Food, 0.9);
        s.people[0].coins = 100;
        rich_id = s.people[0].id.clone();
    }

    run_one_day(&mut sim);

    let s = &sim.world.regions[0].settlements[0];
    let rich_now = s
        .people
        .iter()
        .find(|p| p.id == rich_id)
        .expect("the wealthy neighbour did not leave (they were fed)");
    assert!(
        rich_now.coins < 100,
        "the desperate robbed the rich neighbour ({} left)",
        rich_now.coins
    );
}
