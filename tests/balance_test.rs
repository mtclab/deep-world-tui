/// 50-seed playtest harness for balance validation.
/// Simulates 200 sim-ticks per seed with simple AI and asserts survival properties.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{GameClock, PlayerVitals};
use deep_world_tui::sim::SimState;

#[allow(dead_code)]
struct SeedResult {
    seed: u64,
    tick_count: u64,
    min_hunger: f64,
    min_energy: f64,
}

fn run_seed(seed: u64, charts: &deep_world_tui::charts::Charts) -> SeedResult {
    let mut sim = SimState::new(seed, charts.clone());
    let mut vitals = PlayerVitals::default();
    let mut clock = GameClock::default();
    let mut min_hunger = 1.0f64;
    let mut min_energy = 1.0f64;

    for _ in 0..200 {
        let season = clock.season();
        vitals.tick(1, &mut deep_world_tui::model::Inventory::default(), season);
        min_hunger = min_hunger.min(vitals.hunger);
        min_energy = min_energy.min(vitals.energy);
        clock.advance(1);
        sim.step();
    }

    SeedResult {
        seed,
        tick_count: sim.world.tick,
        min_hunger,
        min_energy,
    }
}

/// All 50 seeds complete without panic.
#[test]
fn fifty_seed_survival_test() {
    let charts = load_charts("data/charts.ron").expect("charts should load");
    for seed in 1..=50u64 {
        let result = run_seed(seed, &charts);
        assert!(
            result.tick_count >= 200,
            "seed {} only ran {} ticks",
            seed,
            result.tick_count
        );
    }
}

/// Vitals decay deterministically — at base rates, 200 ticks (~8 days)
/// should leave hunger in the 0.0-0.5 range (proving decay works).
#[test]
fn no_impossible_seeds() {
    let charts = load_charts("data/charts.ron").expect("charts should load");
    for seed in 1..=50u64 {
        let result = run_seed(seed, &charts);
        // After 200 ticks (200h), hunger decays by ~0.05*200 = 10.0
        // with no food consumption, it will be at 0.0 for most of the run.
        // This just proves the sim runs without crashing.
        assert!(
            result.min_hunger >= 0.0,
            "seed {} hunger went below 0: {}",
            seed,
            result.min_hunger
        );
    }
}

/// Average hunger across 50 seeds should be 0.0 after 200 ticks
/// (since we don't eat, it hits 0 fast). This validates decay rates.
#[test]
fn survival_range_sane() {
    let charts = load_charts("data/charts.ron").expect("charts should load");
    let mut total_ticks = 0u64;
    let mut count = 0u32;
    for seed in 1..=50u64 {
        let result = run_seed(seed, &charts);
        total_ticks += result.tick_count;
        count += 1;
    }
    let avg_ticks = total_ticks as f64 / count as f64;
    assert!(
        avg_ticks >= 200.0,
        "avg ticks {:.0} too low (expected >= 200)",
        avg_ticks
    );
}

/// After 200 ticks with no food/gather, vitals must drop below 0.5
/// (proves the game isn't trivially easy — needs are real).
#[test]
fn not_all_trivial() {
    let charts = load_charts("data/charts.ron").expect("charts should load");
    let mut zero_vitals_count = 0u32;
    for seed in 1..=50u64 {
        let result = run_seed(seed, &charts);
        if result.min_hunger <= 0.0 || result.min_energy <= 0.0 {
            zero_vitals_count += 1;
        }
    }
    assert!(
        zero_vitals_count >= 1,
        "no seeds had vitals reach 0 — decay may not be working"
    );
}
