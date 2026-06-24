//! Slice 0 of the entity-first epic (deep-world-godot#50): prove the per-agent
//! tick ceiling on this hardware BEFORE any refactor, so the coarse cadence
//! (daily vs seasonal) and the live active-region size are chosen from real
//! numbers, not guesses.
//!
//! Measures two costs the epic's two-rate design depends on:
//!   (a) one LIVE hourly tick of N real agents (active region cost), and
//!   (b) one BATCHED day-step of N real agents (inactive-region coarse cost).
//!
//! The per-agent decision here is a stand-in for the slice-3 needs ladder
//! (hungry -> eat -> buy -> work -> ...): pick the lowest need, branch, act.
//! It is deliberately representative of that cost, not the final logic.
//!
//! Run: `cargo run --release --bin agent_bench`

use std::time::Instant;

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::person::generate_person;
use deep_world_tui::model::person::{Need, Person};
use deep_world_tui::rng::SeedRng;

/// One agent's per-tick decision — a representative stand-in for the slice-3
/// needs ladder. Reads all needs, finds the most pressing, acts on it (cheap
/// arithmetic + the `Needs` HashMap touches a real `Person` already pays).
#[inline]
fn agent_decide_and_act(p: &mut Person, dt: f64) {
    const NEEDS: [Need; 5] = [
        Need::Food,
        Need::Money,
        Need::Care,
        Need::Presence,
        Need::Safety,
    ];
    // Decay every need (what tick_needs already does per person).
    for n in NEEDS {
        p.needs.decay(n, 0.01 * dt);
    }
    // Find the most pressing need.
    let mut worst = Need::Food;
    let mut worst_val = f64::MAX;
    for n in NEEDS {
        let v = p.needs.get(n);
        if v < worst_val {
            worst_val = v;
            worst = n;
        }
    }
    // Act on it — the ladder's shape: if it's dire, "spend" to fix it; the
    // fallback consumes a different resource (eat costs money, etc.).
    if worst_val < 0.3 {
        p.needs.satisfy(worst, 0.5);
        if worst == Need::Food {
            p.needs.decay(Need::Money, 0.05); // bought food
        }
    }
}

fn make_agents(n: usize, charts: &deep_world_tui::charts::Charts) -> Vec<Person> {
    let mut rng = SeedRng::new(0xDEEDu64).fork_for("agent-bench");
    (0..n).map(|_| generate_person(&mut rng, charts)).collect()
}

fn main() {
    let charts = load_charts().expect("load data/charts.ron");

    println!("=== Entity-first slice 0: agent tick ceiling ===");
    println!(
        "size_of::<Person>() = {} bytes (stack; heap extra for strings/vecs)",
        std::mem::size_of::<Person>()
    );
    println!();
    println!(
        "{:>9} | {:>12} | {:>12} | {:>11} | {:>11}",
        "agents", "live tick", "day-step", "live/1k", "day/1k"
    );
    println!(
        "{:-<9}-+-{:-<12}-+-{:-<12}-+-{:-<11}-+-{:-<11}",
        "", "", "", "", ""
    );

    for &n in &[10_000usize, 50_000, 100_000, 250_000] {
        let mut agents = make_agents(n, &charts);

        // (a) one live hourly tick.
        let t0 = Instant::now();
        for p in agents.iter_mut() {
            agent_decide_and_act(p, 1.0);
        }
        let live = t0.elapsed();

        // (b) one batched day-step: the day's 24 hours resolved in one pass per
        // agent (amortizes the iteration; the inactive-region coarse cost).
        let t1 = Instant::now();
        for p in agents.iter_mut() {
            agent_decide_and_act(p, 24.0);
        }
        let day = t1.elapsed();

        let per_k = |d: std::time::Duration| d.as_secs_f64() * 1000.0 / (n as f64 / 1000.0);
        println!(
            "{:>9} | {:>9.2} ms | {:>9.2} ms | {:>8.3} ms | {:>8.3} ms",
            n,
            live.as_secs_f64() * 1000.0,
            day.as_secs_f64() * 1000.0,
            per_k(live),
            per_k(day),
        );
        // keep agents alive past timing so the work isn't optimized away
        std::hint::black_box(&agents);
    }

    println!();
    println!("Read: 'live tick' = cost to tick the player's active region each game-hour.");
    println!("      'day-step'  = cost to advance ONE inactive region by a day (coarse cadence).");
    println!("Budget: turn-based, ~100ms/turn is invisible. Pick the coarsest cadence that stays believable.");
    println!("Note: Needs is [f64;5] as of slice 1a — ~10x faster than the old HashMap<Need,f64>.");
}
