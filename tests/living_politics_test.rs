// Living politics (#556): faction standings shift toward each town's character
// in the daily sim — so a real dominant faction emerges, not the frozen 0.5s.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::Faction;
use deep_world_tui::sim::SimState;

// Regression guard (#556 fix, 2026-06-25): at materialized scale the pulls were
// raw profession head-counts, so a big town's makers swamped the rarer traders
// and keepers and every council froze to Crafters. The pulls are now per-capita
// densities (scale-invariant) — a town's character, not its size, decides its
// council. Multi-seed so it does not tip on a single worldgen RNG draw (the
// single-seed brittleness landmine in this codebase).
#[test]
fn councils_diverge_from_the_frozen_default() {
    let mut any_noncrafter = false;
    let mut any_moved = false;
    for seed in [42u64, 7, 99] {
        let charts = load_charts().expect("charts");
        let mut sim = SimState::new_capped(seed, charts, Some(300));
        // Long enough that standings drift to reflect each town's trades/stores.
        for _ in 0..(24 * 120) {
            sim.step();
        }
        for region in &sim.world.regions {
            for s in &region.settlements {
                if s.population == 0 {
                    continue;
                }
                let f = s.politics.dominant_faction();
                if f == Faction::Traders || f == Faction::Elders {
                    any_noncrafter = true;
                }
                if (s.politics.trader_standing - 0.5).abs() > 0.05
                    || (s.politics.elder_standing - 0.5).abs() > 0.05
                {
                    any_moved = true;
                }
            }
        }
    }
    assert!(
        any_noncrafter,
        "across seeds some council should be Traders or Elders, not the frozen-default Crafters"
    );
    assert!(
        any_moved,
        "faction standings should drift off the frozen 0.5"
    );
}
