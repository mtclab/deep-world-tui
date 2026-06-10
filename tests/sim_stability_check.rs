use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

#[test]
fn long_sim_keeps_population_and_people_alive() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    let pop0: usize = sim
        .world
        .regions
        .iter()
        .flat_map(|r| &r.settlements)
        .map(|s| s.people.len())
        .sum();
    for _ in 0..2000 {
        sim.step();
    }
    let pop1: usize = sim
        .world
        .regions
        .iter()
        .flat_map(|r| &r.settlements)
        .map(|s| s.people.len())
        .sum();
    assert!(pop1 > 0, "world should not depopulate");
    // population shouldn't collapse or explode
    assert!(
        pop1 * 2 >= pop0 && pop1 <= pop0 * 3,
        "population stable-ish: {pop0} -> {pop1}"
    );
}
