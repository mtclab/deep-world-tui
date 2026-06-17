// Living politics (#556): faction standings shift toward each town's character
// in the daily sim — so a real dominant faction emerges, not the frozen 0.5s.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::Faction;
use deep_world_tui::sim::SimState;

#[test]
fn councils_diverge_from_the_frozen_default() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);

    // Long enough that standings drift to reflect each town's trades and stores.
    for _ in 0..(24 * 120) {
        sim.step();
    }

    // The councils are no longer uniformly Crafters (the frozen-tie default):
    // some town's dominant faction is now Traders or Elders, driven by its own
    // economy and stability.
    let mut seen = std::collections::HashSet::new();
    for region in &sim.world.regions {
        for s in &region.settlements {
            if s.population > 0 {
                seen.insert(s.politics.dominant_faction());
            }
        }
    }
    assert!(
        seen.contains(&Faction::Traders) || seen.contains(&Faction::Elders),
        "some council is no longer the frozen-default Crafters (saw {seen:?})"
    );
    // And the standings actually moved off 0.5.
    let moved = sim.world.regions.iter().any(|r| {
        r.settlements
            .iter()
            .any(|s| (s.politics.trader_standing - 0.5).abs() > 0.05)
    });
    assert!(moved, "faction standings drifted off the frozen 0.5");
}
