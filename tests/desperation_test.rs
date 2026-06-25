// Entity-first slice 5+8 (deep-world-godot#50): desperation and the fork at the
// bottom of the needs ladder. A soul that exhausts every lawful option — can't
// eat, buy, work, or beg — LEAVES. Where it goes is its character: a soul with a
// fed town to flee to and no taste for crime MIGRATES there; one with nowhere
// better, or a hard disposition, takes to the road as a brigand.
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

/// Strip a settlement of every way to feed itself and make its people starving,
/// penniless, and (optionally) of a given disposition.
fn make_destitute(s: &mut deep_world_tui::model::Settlement, disposition: &str) {
    s.food_stock = 0.0;
    s.treasury = 0;
    s.farms.clear();
    for p in s.people.iter_mut() {
        p.profession = "labourer".into();
        // Only Food is pressing; keep the other drives comfortable so the soul
        // acts on hunger, not on company or safety.
        p.needs.set(Need::Food, 0.05);
        p.needs.set(Need::Care, 0.9);
        p.needs.set(Need::Presence, 0.9);
        p.needs.set(Need::Safety, 0.9);
        p.coins = 0;
        if !disposition.is_empty() {
            p.personality = vec![disposition.into()];
        }
    }
}

#[test]
fn with_nowhere_better_the_destitute_take_to_the_road() {
    let mut sim = small_sim(42);
    // Whole region destitute — no fed town to flee to, so leaving means banditry.
    sim.world.regions[0].game_richness = 0.0;
    for s in sim.world.regions[0].settlements.iter_mut() {
        make_destitute(s, "bitter"); // hard disposition, and nowhere to go
    }
    let pop_before: usize = sim.world.regions[0]
        .settlements
        .iter()
        .map(|s| s.people.len())
        .sum();
    let wanderers_before = sim.frontier.wanderers;

    run_one_day(&mut sim);

    let pop_after: usize = sim.world.regions[0]
        .settlements
        .iter()
        .map(|s| s.people.len())
        .sum();
    assert!(
        pop_after < pop_before,
        "the destitute leave ({pop_before} -> {pop_after})"
    );
    assert!(
        sim.frontier.wanderers > wanderers_before,
        "with nowhere better, they take to the road ({wanderers_before} -> {})",
        sim.frontier.wanderers
    );
}

#[test]
fn a_fed_neighbour_draws_lawful_migrants_not_bandits() {
    let mut sim = small_sim(7);
    let region = &mut sim.world.regions[0];
    assert!(region.settlements.len() >= 2, "need a neighbour town");
    // Town 0 is destitute, its people loyal (no taste for crime); town 1 is fed.
    make_destitute(&mut region.settlements[0], "loyal");
    let dest_pop_before = {
        let s1 = &mut region.settlements[1];
        s1.food_stock = (s1.people.len() as f64) * 5.0; // plainly well-fed
        s1.people.len()
    };
    let wanderers_before = sim.frontier.wanderers;

    run_one_day(&mut sim);

    let region = &sim.world.regions[0];
    assert!(
        region.settlements[1].people.len() > dest_pop_before,
        "the loyal destitute migrate to the fed neighbour ({} -> {})",
        dest_pop_before,
        region.settlements[1].people.len()
    );
    assert_eq!(
        sim.frontier.wanderers, wanderers_before,
        "they sought a kinder town, not the road"
    );
}
