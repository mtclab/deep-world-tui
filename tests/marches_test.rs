// The marches (#630 slice 1): the province keeps an ungoverned wilderness edge
// — a region with no town holding it, where the frontier's bands and holds and
// the deep wild's dreads will concentrate.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::world::generate_world_capped;

#[test]
fn a_large_province_keeps_a_march_at_its_edge() {
    let charts = load_charts().expect("charts");
    let world = generate_world_capped(7, &charts, Some(300));
    if world.regions.len() >= 4 {
        // The last region is the march.
        let march = world.regions.last().unwrap();
        assert!(march.is_march, "the edge region is a march");
        assert!(
            march.settlements.is_empty(),
            "a march is ungoverned — no town holds it"
        );
        // The core stays settled: region 0 has its towns, and the province as a
        // whole keeps more than one settlement.
        assert!(
            !world.regions[0].settlements.is_empty(),
            "the settled core is unchanged"
        );
        let total: usize = world.regions.iter().map(|r| r.settlements.len()).sum();
        assert!(total > 1, "the province still has its settlements");
        // The frontier scales with the province (#669): at least one march, but
        // never more than the settled core, and the marches are a contiguous
        // block at the far (highest-index) edge — the interior stays settled.
        let n = world.regions.len();
        let marches = world.regions.iter().filter(|r| r.is_march).count();
        assert!(marches >= 1, "a large province keeps a march");
        assert!(
            marches <= n - 2,
            "the settled core stays the majority: {marches} of {n}"
        );
        let first_march = world.regions.iter().position(|r| r.is_march).unwrap();
        assert!(
            world.regions[first_march..].iter().all(|r| r.is_march),
            "the marches are the far edge, one contiguous wild block"
        );
        assert!(
            world.regions[..first_march].iter().all(|r| !r.is_march),
            "the interior is settled, not wild"
        );
    }
}

#[test]
fn marches_are_deterministic() {
    let charts = load_charts().expect("charts");
    let a = generate_world_capped(31, &charts, Some(300));
    let b = generate_world_capped(31, &charts, Some(300));
    let am: Vec<bool> = a.regions.iter().map(|r| r.is_march).collect();
    let bm: Vec<bool> = b.regions.iter().map(|r| r.is_march).collect();
    assert_eq!(am, bm, "the same seed marks the same marches");
}

#[test]
fn a_march_with_a_grown_hold_is_tamed() {
    use deep_world_tui::sim::SimState;
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new_capped(7, charts, Some(300));
    // Take the region holding the world's most-populous town — a real town that
    // sits above the tame bar and will not starve back down — and call it a
    // march with a grown hold. The tide should tame it back into the province.
    let ri = sim
        .world
        .regions
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.settlements
                .iter()
                .map(|s| s.population)
                .max()
                .map(|p| (i, p))
        })
        .max_by_key(|&(_, p)| p)
        .map(|(i, _)| i)
        .expect("a settled region");
    sim.world.regions[ri].is_march = true;
    // Drive the clock to a seasonal tide-turn and run the tide directly, so the
    // check is deterministic and not at the mercy of a long soak's migrations.
    sim.world.tick = 30 * 24;
    deep_world_tui::sim::frontier::march_tide(&mut sim);
    assert!(
        !sim.world.regions[ri].is_march,
        "a march whose hold grew into a town is tamed back into the province"
    );
}

#[test]
fn a_settled_region_emptied_falls_back_to_march() {
    use deep_world_tui::sim::SimState;
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new_capped(7, charts, Some(300));
    // A settled region whose every town dies returns to the wild.
    let ri = sim
        .world
        .regions
        .iter()
        .position(|r| !r.is_march && r.settlements.iter().any(|s| s.population > 0))
        .expect("a settled region");
    for s in sim.world.regions[ri].settlements.iter_mut() {
        s.population = 0;
        s.people.clear();
    }
    sim.world.tick = 30 * 24;
    deep_world_tui::sim::frontier::march_tide(&mut sim);
    assert!(
        sim.world.regions[ri].is_march,
        "a settled region emptied of its towns falls back to march"
    );
}

#[test]
fn a_march_reads_as_wilderness_not_a_settled_region() {
    let charts = load_charts().expect("charts");
    // Find a seed whose world has a march, and confirm its description frames it
    // as ungoverned wild — not a settled region's blurb.
    for seed in 0..12u64 {
        let world = generate_world_capped(seed, &charts, Some(300));
        if let Some(march) = world.regions.iter().find(|r| r.is_march) {
            let d = march.description.to_lowercase();
            assert!(
                d.contains("wild") || d.contains("ungoverned") || d.contains("march"),
                "a march should read as wilderness, got: {}",
                march.description
            );
            return;
        }
    }
}

#[test]
fn the_frontier_scales_with_the_province() {
    let charts = load_charts().expect("charts");
    // Across many seeds, larger provinces carry more march-sectors on average
    // than small ones — the wild edge grows with the land (#669).
    let mut small_marches = 0usize; // provinces of 4-6 sectors
    let mut small_n = 0usize;
    let mut big_marches = 0usize; // provinces of 10+ sectors
    let mut big_n = 0usize;
    for seed in 0..24u64 {
        let w = generate_world_capped(seed, &charts, Some(300));
        let n = w.regions.len();
        let m = w.regions.iter().filter(|r| r.is_march).count();
        if (4..=6).contains(&n) {
            small_marches += m;
            small_n += 1;
        } else if n >= 10 {
            big_marches += m;
            big_n += 1;
        }
    }
    if small_n == 0 || big_n == 0 {
        return;
    }
    let small_avg = small_marches as f64 / small_n as f64;
    let big_avg = big_marches as f64 / big_n as f64;
    assert!(
        big_avg > small_avg,
        "a wider province keeps a wider frontier: big {big_avg:.2} vs small {small_avg:.2}"
    );
}
