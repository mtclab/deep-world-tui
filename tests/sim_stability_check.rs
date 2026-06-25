use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

fn pop_of(sim: &SimState) -> usize {
    sim.world
        .regions
        .iter()
        .flat_map(|r| &r.settlements)
        .map(|s| s.people.len())
        .sum()
}

#[test]
fn long_sim_keeps_population_and_people_alive() {
    // A smoke test that a province neither depopulates nor explodes over a long
    // run. Summed across several seeds so it reads the system's behaviour, not
    // one seed's RNG basin — a single seed sits too close to the band's edge and
    // any worldgen change (a new region weight, a new profession) reshuffles the
    // whole sample and tips it. The aggregate is the honest measure.
    let charts = load_charts().expect("charts");
    let mut total0 = 0usize;
    let mut total1 = 0usize;
    for seed in 40..46u64 {
        let mut sim = SimState::new_capped(seed, charts.clone(), Some(300));
        total0 += pop_of(&sim);
        for _ in 0..2000 {
            sim.step();
        }
        let p1 = pop_of(&sim);
        assert!(
            p1 > 0,
            "no single world should fully depopulate (seed {seed})"
        );
        total1 += p1;
    }
    // The settled world holds: it keeps well over a third of its souls and does
    // not balloon. (A real collapse or explosion moves the aggregate far past
    // these bounds; ordinary seed-to-seed variance does not.)
    assert!(
        total1 * 5 >= total0 * 2 && total1 <= total0 * 3,
        "population stable-ish across seeds: {total0} -> {total1}"
    );
}
