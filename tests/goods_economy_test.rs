// A living goods economy (#540): settlements produce trade goods by their
// trades in the daily sim — with no player present at all.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::SimState;

/// Find a (region, settlement) whose trades should produce a good, across seeds.
/// Returns the indices and the good its trades make.
fn find_producer(sim: &SimState) -> Option<(usize, usize, ItemType)> {
    for (ri, region) in sim.world.regions.iter().enumerate() {
        for (si, s) in region.settlements.iter().enumerate() {
            if s.population == 0 {
                continue;
            }
            if s.profession_count("smith") > 0 {
                return Some((ri, si, ItemType::Tool));
            }
            if s.profession_count("weaver") > 0 {
                return Some((ri, si, ItemType::Cloth));
            }
            if s.profession_count("miner") > 0 {
                return Some((ri, si, ItemType::Iron));
            }
            if s.profession_count("carpenter") > 0 {
                return Some((ri, si, ItemType::Wood));
            }
        }
    }
    None
}

#[test]
fn settlements_make_goods_without_the_player() {
    let charts = load_charts().expect("charts");
    let (mut sim, ri, si, good) = (0..40u64)
        .find_map(|seed| {
            let sim = SimState::new(seed, charts.clone());
            find_producer(&sim).map(|(ri, si, good)| (sim, ri, si, good))
        })
        .expect("a settlement with a producing trade somewhere across seeds");

    let before = sim.world.regions[ri].settlements[si].good(good);
    // Run a fortnight of pure simulation — no player acts.
    for _ in 0..(24 * 14) {
        sim.step();
    }
    let after = sim.world.regions[ri].settlements[si].good(good);
    assert!(
        after > before,
        "the trade's good grew on its own ({good:?}: {before} -> {after})"
    );
}

#[test]
fn a_good_is_capped_not_unbounded() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(7, charts);
    // Pick the most populous settlement and run a long stretch; no good should
    // exceed the per-settlement cap (population * 0.5).
    for _ in 0..(24 * 120) {
        sim.step();
    }
    for region in &sim.world.regions {
        for s in &region.settlements {
            let cap = s.population as f64 * 0.5 + 0.001;
            for it in [
                ItemType::Iron,
                ItemType::Tool,
                ItemType::Cloth,
                ItemType::Wood,
            ] {
                assert!(
                    s.good(it) <= cap,
                    "{} holds {:?} {} over cap {}",
                    s.name,
                    it,
                    s.good(it),
                    cap
                );
            }
        }
    }
}
