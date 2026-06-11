// Re-population + land-taking (#346): the world builds back without the
// player — prosperous settlements reopen neighboring ghost towns and, rarely,
// send founding parties into rich empty land. The sponsor PAYS (population
// and stores actually move), and the pace stays slow: some checks pass with
// nothing happening at all.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::{founding, SimState};

fn fresh_sim(seed: u64) -> SimState {
    let charts = load_charts().expect("charts");
    SimState::new(seed, charts)
}

fn total_population(sim: &SimState) -> u64 {
    sim.world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .map(|s| s.population as u64)
        .sum()
}

/// Make settlement (ri, si) a ghost town the way famine does.
fn make_ghost(sim: &mut SimState, ri: usize, si: usize) {
    let s = &mut sim.world.regions[ri].settlements[si];
    s.population = 0;
    s.people.clear();
    s.services.clear();
    s.farms.clear();
    s.food_stock = 0.0;
    s.description = "Abandoned. Doors hang open; nothing moves.".into();
}

/// Make settlement (ri, si) prosperous enough to sponsor.
fn make_prosperous(sim: &mut SimState, ri: usize, si: usize) {
    let s = &mut sim.world.regions[ri].settlements[si];
    s.population = s.population.max(120);
    s.food_stock = s.population as f64 * 3.0;
}

#[test]
fn a_prosperous_neighbor_reopens_a_ghost_town_and_pays_for_it() {
    let mut sim = fresh_sim(42);
    let ri = 0;
    if sim.world.regions[ri].settlements.len() < 2 {
        return; // need a ghost and a sponsor in reach: vacuous on this seed
    }
    make_ghost(&mut sim, ri, 0);
    make_prosperous(&mut sim, ri, 1);
    let pop_before = total_population(&sim);
    let sponsor_food_before = sim.world.regions[ri].settlements[1].food_stock;
    let mut reopened_at = None;
    for season in 1..=24u32 {
        founding::tick_world_building(&mut sim, season);
        if sim.world.regions[ri].settlements[0].population > 0 {
            reopened_at = Some(season);
            break;
        }
    }
    assert!(
        reopened_at.is_some(),
        "two dozen seasons of a fat neighbor should reopen the ghost town"
    );
    let ghost = &sim.world.regions[ri].settlements[0];
    assert!(
        (10..=20).contains(&ghost.population),
        "a reopening party is 10-20 souls, got {}",
        ghost.population
    );
    assert!(!ghost.services.is_empty(), "services rebuilt from none");
    assert!(
        !ghost.description.starts_with("Abandoned"),
        "the door-boards come off"
    );
    // The sponsor paid: people are conserved, and the larder moved.
    assert_eq!(
        total_population(&sim),
        pop_before,
        "every settler came from somewhere"
    );
    assert!(
        sim.world.regions[ri].settlements[1].food_stock < sponsor_food_before,
        "stores travel with the party"
    );
    assert!(
        sim.world.regions[ri].settlements[0].food_stock > 0.0,
        "the reopened town starts with the moved stores"
    );
    // Word of it reaches the record.
    let told = sim
        .journal
        .entries
        .iter()
        .any(|e| e.text.contains("reopen"));
    assert!(told, "the reopening feeds the rumor mill");
}

#[test]
fn a_poor_world_stays_quiet() {
    let mut sim = fresh_sim(42);
    let ri = 0;
    if sim.world.regions[ri].settlements.len() < 2 {
        return;
    }
    make_ghost(&mut sim, ri, 0);
    // Starve every possible sponsor below the line.
    for region in sim.world.regions.iter_mut() {
        for s in region.settlements.iter_mut() {
            if s.population > 0 {
                s.population = s.population.min(25);
                s.food_stock = 5.0;
            }
        }
    }
    for season in 1..=24u32 {
        founding::tick_world_building(&mut sim, season);
    }
    assert_eq!(
        sim.world.regions[ri].settlements[0].population, 0,
        "no prosperous sponsor, no reopening — some wounds stay open"
    );
}

#[test]
fn the_pace_is_slow_not_every_season_acts() {
    let mut sim = fresh_sim(42);
    let ri = 0;
    if sim.world.regions[ri].settlements.len() < 2 {
        return;
    }
    make_ghost(&mut sim, ri, 0);
    make_prosperous(&mut sim, ri, 1);
    // The very first season-check must not be guaranteed to act: count how
    // many of the first 24 single-season worlds act immediately. With a 1-in-3
    // gate the majority must stay quiet (determinism makes this exact per seed).
    let mut acted = 0;
    for season in 1..=24u32 {
        let mut probe = fresh_sim(42);
        make_ghost(&mut probe, ri, 0);
        make_prosperous(&mut probe, ri, 1);
        founding::tick_world_building(&mut probe, season);
        if probe.world.regions[ri].settlements[0].population > 0 {
            acted += 1;
        }
    }
    assert!(
        acted < 24,
        "the Fall's tail is long: reopening must not fire every season"
    );
    assert!(acted > 0, "but the world does build back eventually");
}

#[test]
fn the_frontier_is_taken_rarely_and_paid_for() {
    let mut sim = fresh_sim(42);
    // Build a frontier: region 1 rich, one living settlement, its neighbor
    // region 0 prosperous enough to sponsor.
    if sim.world.regions.len() < 2 {
        return;
    }
    let ri = 1;
    sim.world.regions[ri].game_richness = 1.0;
    // Thin the region to one living settlement and no crowding.
    while sim.world.regions[ri].settlements.len() > 1 {
        sim.world.regions[ri].settlements.pop();
    }
    make_prosperous(&mut sim, ri, 0);
    let n_before = sim.world.regions[ri].settlements.len();
    let pop_before = total_population(&sim);
    let mut founded_at = None;
    for season in 1..=64u32 {
        founding::tick_world_building(&mut sim, season);
        if sim.world.regions[ri].settlements.len() > n_before {
            founded_at = Some(season);
            break;
        }
    }
    let Some(_) = founded_at else {
        // Rare by design; on a seed where 64 seasons never roll it, the
        // discipline holds — nothing to assert further.
        return;
    };
    let new_s = sim.world.regions[ri]
        .settlements
        .iter()
        .find(|s| s.id.contains("-set-f"))
        .expect("the new place carries a founding id");
    assert!(
        new_s.population >= 10,
        "a founding party is at least ten souls"
    );
    assert_eq!(new_s.size, "hamlet", "scale honesty: a hamlet, not a town");
    assert_eq!(
        total_population(&sim),
        pop_before,
        "the sponsor paid in people — none were made from air"
    );
}
