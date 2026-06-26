// Per-agent economy (deep-world-godot#54), slice 4: emergent price. A meal costs
// what scarcity makes it cost — cheap where the granary is full, dear as it
// empties — so a famine bites the poor hardest, dearest exactly when they can
// least pay.
use deep_world_tui::sim::agency::town_context;
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn price_at(food_per_capita: f64) -> u32 {
    let mut sim = SimState::new_capped(42, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = food_per_capita * s.population.max(1) as f64;
    town_context(s, 1.0, false, None, 0.15, 0).food_price
}

#[test]
fn plenty_is_cheap_famine_is_dear() {
    let plenty = price_at(2.0); // a full granary
    let lean = price_at(0.6); // running short
    let famine = price_at(0.0); // empty
    assert_eq!(plenty, 1, "where there is plenty, a meal is cheap");
    assert!(
        lean > plenty,
        "as the granary empties the price climbs ({lean} > {plenty})"
    );
    assert!(
        famine >= lean,
        "and a true famine is dearest ({famine} >= {lean})"
    );
    assert_eq!(famine, 3, "an empty granary asks the most");
}
