// Imperfect knowledge (deep-world-godot#55): folk act on what they have heard, not
// on live truth. Word of where grain is to be had travels slowly, so the believed
// best-fed town lags reality — a town can starve while still reputed to be fed,
// and migrants keep making for it.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

fn run_one_day(sim: &mut SimState, onto_tick: u64) {
    sim.world.tick = onto_tick - 1;
    sim.step();
}

#[test]
fn word_of_a_fed_town_lags_the_truth() {
    let mut sim = SimState::new_capped(42, load_charts().expect("charts"), Some(60));
    // First word gets around — the region learns which town is best-fed.
    run_one_day(&mut sim, 24);
    let believed = sim.world.regions[0]
        .known_fed
        .expect("the region should have formed a belief about where grain is");

    // That town is struck by famine, and a different one comes into plenty — but
    // no time has passed for the news to travel (well inside the 5-day interval).
    {
        let r = &mut sim.world.regions[0];
        let other = (0..r.settlements.len())
            .find(|&i| i != believed)
            .expect("need a second town");
        r.settlements[believed].food_stock = 0.0;
        r.settlements[believed].farms.clear();
        r.settlements[other].food_stock = r.settlements[other].people.len() as f64 * 10.0;
    }

    run_one_day(&mut sim, 48); // one more day — still inside the news interval

    assert_eq!(
        sim.world.regions[0].known_fed,
        Some(believed),
        "the province still reckons the now-starved town is where the grain is — \
         knowledge lags the world"
    );
}

#[test]
fn word_eventually_catches_up() {
    // After the news interval passes, the belief refreshes to the truth.
    let mut sim = SimState::new_capped(7, load_charts().expect("charts"), Some(60));
    run_one_day(&mut sim, 24);
    let as_of_first = sim.world.regions[0].known_fed_as_of;
    // Step past the 5-day interval; the belief is allowed to update again.
    run_one_day(&mut sim, 24 * 7);
    assert!(
        sim.world.regions[0].known_fed_as_of > as_of_first,
        "word got around again after the interval ({} -> {})",
        as_of_first,
        sim.world.regions[0].known_fed_as_of
    );
}
