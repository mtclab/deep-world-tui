use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

fn collect_npc_names(sim: &SimState) -> Vec<String> {
    let mut names = vec![];
    for region in &sim.world.regions {
        for settlement in &region.settlements {
            for person in &settlement.people {
                names.push(person.name.clone());
            }
        }
    }
    names.sort();
    names
}

fn collect_region_names(sim: &SimState) -> Vec<String> {
    sim.world.regions.iter().map(|r| r.name.clone()).collect()
}

/// A full fingerprint of the living-world state the daily sim drives — the
/// fields the seasons/economy/faith/plague/province systems mutate — so a
/// determinism check sees divergence in any of them, not only in names. All
/// map-backed fields are sorted, and floats are fixed-precision, so the string
/// is a stable, order-independent witness of the whole simulated state.
fn living_world_fingerprint(sim: &SimState) -> String {
    let mut out = String::new();
    for region in &sim.world.regions {
        out.push_str(&format!("[{}:march{}]", region.id, region.is_march));
        for s in &region.settlements {
            out.push_str(&format!(
                "{}|pop{}|food{:.3}|fam{}|plag{}|",
                s.id, s.population, s.food_stock, s.famine_days, s.plague_days
            ));
            let mut goods: Vec<String> = s
                .goods_stock
                .iter()
                .map(|(it, v)| format!("{it:?}={v:.3}"))
                .collect();
            goods.sort();
            out.push_str(&goods.join(","));
            out.push('|');
            let mut faith: Vec<String> = s
                .faith
                .devotion
                .iter()
                .map(|(g, v)| format!("{g:?}={v:.3}"))
                .collect();
            faith.sort();
            out.push_str(&faith.join(","));
            out.push_str(";\n");
        }
    }
    let mut bonds: Vec<String> = sim
        .province_ties
        .bonds
        .iter()
        .map(|((a, b), v)| format!("{a}-{b}={v:.3}"))
        .collect();
    bonds.sort();
    out.push_str(&bonds.join(","));
    out.push_str(&format!("|wanderers{}", sim.frontier.wanderers));
    let mut bands: Vec<String> = sim
        .frontier
        .bands
        .iter()
        .map(|b| format!("{}:{}@{}", b.id, b.size, b.region_idx))
        .collect();
    bands.sort();
    out.push_str("|bands:");
    out.push_str(&bands.join(","));
    let mut beasts: Vec<String> = sim
        .beasts
        .iter()
        .map(|b| format!("{}:{:?}@{},{}:{}", b.id, b.species, b.px, b.py, b.hp))
        .collect();
    beasts.sort();
    out.push_str("|beasts:");
    out.push_str(&beasts.join(","));
    out
}

/// Long full-state determinism: the slow living-world systems (migration @30t,
/// lifecycle @72t, faith upheavals, plague) only fire on their own cadences, so
/// the short name-only replay above cannot see a nondeterminism bug in them.
/// Run two worlds on one seed past all those cadences and compare the entire
/// living-world fingerprint.
#[test]
fn long_run_full_state_deterministic() {
    let charts = load_charts().expect("charts should load");
    let seed = 424242u64;
    let mut a = SimState::new_capped(seed, charts.clone(), Some(400));
    let mut b = SimState::new_capped(seed, charts.clone(), Some(400));
    for _ in 0..900 {
        a.step();
        b.step();
    }
    assert_eq!(a.world.tick, b.world.tick);
    assert_eq!(
        living_world_fingerprint(&a),
        living_world_fingerprint(&b),
        "same seed must drive the whole living world identically over a long run"
    );
}

#[test]
fn same_seed_produces_identical_world() {
    let charts = load_charts().expect("charts should load");
    let seed = 12345u64;
    let mut a = SimState::new_capped(seed, charts.clone(), Some(400));
    let mut b = SimState::new_capped(seed, charts.clone(), Some(400));
    for _ in 0..100 {
        a.step();
        b.step();
    }
    assert_eq!(a.world.regions.len(), b.world.regions.len());
    assert_eq!(collect_region_names(&a), collect_region_names(&b));
    assert_eq!(collect_npc_names(&a), collect_npc_names(&b));
}

#[test]
fn different_seeds_produce_different_worlds() {
    let charts = load_charts().expect("charts should load");
    let a = SimState::new(11111, charts.clone());
    let b = SimState::new(99999, charts.clone());
    assert_ne!(collect_region_names(&a), collect_region_names(&b));
}

#[test]
fn rng_fork_for_produces_different_subsequences() {
    use deep_world_tui::rng::SeedRng;
    let base = SeedRng::new(42);
    let mut ga = base.fork_for("gather");
    let mut gb = base.fork_for("encounter");
    let mut ga_vals = vec![];
    let mut gb_vals = vec![];
    for _ in 0..10 {
        ga_vals.push(ga.gen_f64());
        gb_vals.push(gb.gen_f64());
    }
    assert_ne!(ga_vals, gb_vals);
}

#[test]
fn rng_fork_for_same_domain_same_sequence() {
    use deep_world_tui::rng::SeedRng;
    let base = SeedRng::new(42);
    let mut ga = base.fork_for("gather");
    let mut gb = base.fork_for("gather");
    let mut ga_vals = vec![];
    let mut gb_vals = vec![];
    for _ in 0..10 {
        ga_vals.push(ga.gen_f64());
        gb_vals.push(gb.gen_f64());
    }
    assert_eq!(ga_vals, gb_vals);
}

#[test]
fn replay_100_ticks_deterministic() {
    let charts = load_charts().expect("charts should load");
    let seed = 77777u64;
    let mut run1 = SimState::new_capped(seed, charts.clone(), Some(400));
    let mut run2 = SimState::new_capped(seed, charts.clone(), Some(400));
    for _ in 0..100 {
        run1.step();
    }
    for _ in 0..100 {
        run2.step();
    }
    assert_eq!(run1.world.tick, run2.world.tick);
    assert_eq!(collect_region_names(&run1), collect_region_names(&run2));
    assert_eq!(collect_npc_names(&run1), collect_npc_names(&run2));
}
