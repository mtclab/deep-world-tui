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

#[test]
fn a_working_town_holds_a_broad_goods_shelf() {
    // #671: the living economy reads the whole working town, not four trades.
    // After a fortnight a province should hold a goods shelf wider than the old
    // {Iron, Tool, Cloth, Wood} — foresters, herders, beast-handlers,
    // herbalists, and healers all stocking their wares.
    use std::collections::BTreeSet;
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(7, charts);
    for _ in 0..(24 * 14) {
        sim.step();
    }
    let mut shelf: BTreeSet<String> = BTreeSet::new();
    for region in &sim.world.regions {
        for s in &region.settlements {
            for (item, qty) in &s.goods_stock {
                if *qty > 0.0 {
                    shelf.insert(format!("{item:?}"));
                }
            }
        }
    }
    // The old economy could make at most four kinds; the working town makes
    // more. Bar set below the full roster so it is not seed-brittle.
    assert!(
        shelf.len() >= 6,
        "a working province holds a broad goods shelf, got {}: {:?}",
        shelf.len(),
        shelf
    );
    // And at least one good from beyond the old four turns up.
    let beyond_the_old_four = [
        "Hide", "Leather", "Herb", "Branches", "Stone", "Salve", "Bandage", "Nails",
    ];
    assert!(
        beyond_the_old_four.iter().any(|g| shelf.contains(*g)),
        "the new trades stock new goods, got: {shelf:?}"
    );
}

#[test]
fn the_settled_crafts_stock_the_deeper_shelf() {
    // #671 slice 2: potters, brewers, and charcoal-burning foresters stock the
    // new goods (Pottery, Ale, Charcoal) autonomously — somewhere across a few
    // seeds a working province holds at least one of them.
    use std::collections::BTreeSet;
    let charts = load_charts().expect("charts");
    let deeper = ["Pottery", "Ale", "Charcoal"];
    let found = (0..6u64).any(|seed| {
        let mut sim = SimState::new(seed, charts.clone());
        for _ in 0..(24 * 14) {
            sim.step();
        }
        let mut shelf: BTreeSet<String> = BTreeSet::new();
        for region in &sim.world.regions {
            for s in &region.settlements {
                for (item, qty) in &s.goods_stock {
                    if *qty > 0.0 {
                        shelf.insert(format!("{item:?}"));
                    }
                }
            }
        }
        deeper.iter().any(|g| shelf.contains(*g))
    });
    assert!(
        found,
        "the deeper shelf (Pottery/Ale/Charcoal) is stocked by the settled crafts"
    );
}

#[test]
fn the_wider_roster_fills_the_old_producer_gaps() {
    // #674: Glass and Cordage had NO trade producing them in the living economy
    // until the glass-worker and rope-maker joined the roster. Across a few
    // seeds a working province now stocks at least one of them on its own.
    use std::collections::BTreeSet;
    let charts = load_charts().expect("charts");
    let found = (0..8u64).any(|seed| {
        let mut sim = SimState::new(seed, charts.clone());
        for _ in 0..(24 * 14) {
            sim.step();
        }
        let mut shelf: BTreeSet<String> = BTreeSet::new();
        for region in &sim.world.regions {
            for s in &region.settlements {
                for (item, qty) in &s.goods_stock {
                    if *qty > 0.0 {
                        shelf.insert(format!("{item:?}"));
                    }
                }
            }
        }
        shelf.contains("Glass") || shelf.contains("Cordage")
    });
    assert!(
        found,
        "the glass-workers and rope-makers stock Glass/Cordage the old roster never made"
    );
}

#[test]
fn registry_goods_flow_through_the_living_economy() {
    // #678 slice 2: data-defined trade goods (ItemType::Good) are produced by
    // trades and drift across the region — a working province holds some on its
    // own, with no player.
    let charts = load_charts().expect("charts");
    let found = (0..6u64).any(|seed| {
        let mut sim = SimState::new(seed, charts.clone());
        for _ in 0..(24 * 14) {
            sim.step();
        }
        sim.world.regions.iter().any(|r| {
            r.settlements.iter().any(|s| {
                s.goods_stock
                    .iter()
                    .any(|(it, q)| matches!(it, ItemType::Good(_)) && *q > 0.0)
            })
        })
    });
    assert!(
        found,
        "settlements stock registry trade goods (salt/bread/copper/...) on their own"
    );
}
