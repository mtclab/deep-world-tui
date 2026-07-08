// World stability guard (playtest finding, 2026-06-26). The entity-first epic
// made every inhabitant a real, aging soul; the NPC life-clock was still the
// player's compressed 3-days-per-year, so the whole province aged ~30 life-years
// a calendar year and emptied itself (8.5k -> <300 in a game-year). Fixed by
// aging the world's people at the world's own calendar pace
// (`TICKS_PER_LIFE_YEAR` = one calendar year), plus gentler economy drains and a
// rare-rather-than-instant flight from famine. This guards the regression: a
// materialised province must remain a living society across a year, not collapse.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

fn pop(sim: &SimState) -> usize {
    sim.world
        .regions
        .iter()
        .flat_map(|r| &r.settlements)
        .map(|s| s.people.len())
        .sum()
}

#[test]
fn a_materialised_province_does_not_collapse_over_a_year() {
    for seed in [42u64, 7, 999] {
        let mut sim = SimState::new_capped(seed, load_charts().expect("charts"), Some(150));
        let before = pop(&sim);
        // A full calendar year of the living world.
        for _ in 0..(365 * 24) {
            sim.step();
        }
        let after = pop(&sim);
        // The old fast-aging bug emptied a province to a few percent in a year;
        // a stable world holds the great bulk of its people. A small capped
        // roster sits far below its land's ceiling, so a healthy world in fact
        // GROWS toward capacity here — this is only the no-collapse floor; the
        // near-capacity ±25% property is asserted uncapped below (#728).
        assert!(
            after * 4 >= before * 3,
            "the province collapsed over a year (seed {seed}: {before} -> {after})"
        );
    }
}

// The shipped config is UNCAPPED — every settlement seeded to its land's full
// carrying capacity (#728). That is the config that was ship-blocking: worldgen
// sized towns to a land capacity the daily sim then recomputed from terrain the
// town had paved over (and, on growth, from an anchor slid off its founding
// water), collapsing the food ceiling ~10x. Seed 42 crashed 8,540 -> 583 in a
// year (-93%), the player's home village 1,246 -> 46 with its stores emptied by
// day 30. This guards the shipped config directly: a province seeded near its
// carrying capacity must stay THERE across a year — a living, roughly stable
// society, not a collapse and not a runaway. Uncapped seed 42 is ~8.5k souls;
// one game-year steps in a few seconds.
#[test]
fn an_uncapped_province_holds_within_a_quarter_over_a_year() {
    let mut sim = SimState::new(42, load_charts().expect("charts"));
    let before = pop(&sim);
    // The player's home settlement (region 0, settlement 0) — a coastal village
    // that used to crater to a hamlet with empty stores.
    let home_before = sim.world.regions[0].settlements[0].population;
    assert!(
        home_before >= 500,
        "precondition: the home starts a village ({home_before})"
    );
    for _ in 0..(365 * 24) {
        sim.step();
    }
    let after = pop(&sim);
    // Within ±25% of the seeded population: neither collapse nor runaway.
    assert!(
        after * 4 >= before * 3 && after * 4 <= before * 5,
        "the province drifted past a quarter over a year (seed 42: {before} -> {after})"
    );
    // The home holds its size class (village, >= 500) and its stores never empty.
    let home = &sim.world.regions[0].settlements[0];
    assert!(
        home.population >= 500,
        "the home village collapsed a size class (was {home_before}, now {})",
        home.population
    );
    assert!(
        home.food_stock > 0.0,
        "the home's stores emptied to famine (food_stock {:.1})",
        home.food_stock
    );
}

// The same ±25% property across several seeds — the repo's multi-seed aggregate
// mandate (#728). Heavy: an uncapped province can be 100k+ souls, so a full
// game-year across three seeds runs on the order of minutes. Kept `#[ignore]`d
// like the death-rate soak, run by hand or in a nightly pass; the always-on
// guard above covers the fast seed.
#[ignore = "heavy: uncapped 100k-soul provinces, ~minutes; run manually/nightly"]
#[test]
fn uncapped_provinces_hold_within_a_quarter_multiseed() {
    for seed in [42u64, 7, 999] {
        let mut sim = SimState::new(seed, load_charts().expect("charts"));
        let before = pop(&sim);
        for _ in 0..(365 * 24) {
            sim.step();
        }
        let after = pop(&sim);
        assert!(
            after * 4 >= before * 3 && after * 4 <= before * 5,
            "province drifted past a quarter over a year (seed {seed}: {before} -> {after})"
        );
    }
}
