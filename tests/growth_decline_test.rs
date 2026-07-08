// Settlement growth & decline (#319): head-count promotions (a new village
// raises a Temple), famine exodus, and abandonment — ghost towns the player
// can watch happen, or avert by delivering food.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::SettlementService;
use deep_world_tui::sim::SimState;

fn run_days(sim: &mut SimState, days: u64) {
    for _ in 0..days {
        sim.world.tick = ((sim.world.tick / 24) + 1) * 24 - 1;
        sim.step();
    }
}

#[test]
fn a_swelling_hamlet_becomes_a_village_with_a_temple() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    {
        let s = &mut sim.world.regions[0].settlements[0];
        s.size = "hamlet".into();
        s.services.retain(|sv| *sv != SettlementService::Temple);
        s.population = 600; // over the canon village threshold (500)
        s.food_stock = 2_000.0;
    }
    run_days(&mut sim, 1);
    let s = &sim.world.regions[0].settlements[0];
    assert_eq!(s.size, "village");
    assert!(
        s.services.contains(&SettlementService::Temple),
        "a new village raises a Temple"
    );
}

#[test]
fn famine_empties_a_settlement() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    // Make the whole region helpless: singers neither produce food nor build
    // (a builder roster raised a food Trap in an earlier draft of this test —
    // the sim fed itself). No spouses: no children with chance professions.
    sim.world.regions[0].region_type = "steppe".into();
    for s in sim.world.regions[0].settlements.iter_mut() {
        for p in s.people.iter_mut() {
            p.profession = "singer".into();
            p.has_spouse = false;
            p.sex = "m".into();
        }
        s.farms.clear();
    }
    {
        let s = &mut sim.world.regions[0].settlements[0];
        // Population is the roster now (entity-first epic): a 12-soul town has 12
        // residents, not just the count set to 12.
        s.people.truncate(12);
        s.population = s.people.len() as u32;
        s.food_stock = 0.0;
    }
    run_days(&mut sim, 40);
    let s = &sim.world.regions[0].settlements[0];
    assert_eq!(
        s.population, 0,
        "a month of empty stores should empty the settlement (famine_days {})",
        s.famine_days
    );
    assert!(s.people.is_empty());
    assert!(s.services.is_empty(), "nothing left open");
}

// A well-fed town holds its people (#728). This was deferred under the
// entity-first epic: with every soul real, a settlement bled to out-migration
// even when fed, because the daily food economy could not actually feed the
// seeded population (worldgen sized towns to the LAND's carrying capacity, but
// the sim recomputed that capacity from terrain the town had since paved over —
// and, on growth, from an anchor it had slid off the water — so the food ceiling
// collapsed ~10x and the town starved and stampeded out). With the land ceiling
// held stable, the farms scaled to their real workforce, a frost-hardy sowing,
// and job-seeking made an organic trickle rather than a one-day stampede, a fed
// town keeps its people. Re-enabled.
#[test]
fn fed_settlements_do_not_decline() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    {
        let s = &mut sim.world.regions[0].settlements[0];
        s.food_stock = 10_000.0;
    }
    let pop = sim.world.regions[0].settlements[0].population;
    run_days(&mut sim, 20);
    let s = &sim.world.regions[0].settlements[0];
    assert!(
        s.population + 10 >= pop,
        "a fed settlement should hold its people (was {pop}, now {})",
        s.population
    );
}
