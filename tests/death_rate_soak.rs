// Death-rate soak (#386 item 6): a competent-but-ordinary scripted life,
// run across many seeds, reporting how lives end (old age vs the rest) and
// how long they run. Ignored by default — it is a tuning instrument, not a
// gate. Run with: cargo test --test death_rate_soak -- --ignored --nocapture
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{DeathCause, ItemType};
use deep_world_tui::ui::app::App;

fn run_life(seed: u64, day_cap: u32) -> (Option<DeathCause>, u32, u32) {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    while a.death_cause.is_none() && a.clock.day < day_cap {
        // An ordinary day: gather while the light holds, eat from the sack
        // (advance_clock does that), sleep the night.
        for _ in 0..3 {
            if a.death_cause.is_some() {
                break;
            }
            a.gather();
            // Encounters and collapses resolve themselves the timid way.
            if a.encounter.is_some() {
                a.dismiss_encounter();
            }
            if a.collapse.is_some() {
                a.dismiss_collapse();
            }
        }
        if a.death_cause.is_none() {
            // Keep water topped up the cheap way a real player would: drink
            // what was gathered, buy nothing.
            let inv = a.player_inventory();
            if inv.get(ItemType::Water) == 0 {
                if let Some(ref mut ps) = a.player_start {
                    ps.inventory.add(ItemType::Water, 2);
                }
            }
            a.rest_hours(8);
            if a.collapse.is_some() {
                a.dismiss_collapse();
            }
        }
    }
    (a.death_cause, a.clock.day, a.current_age_years())
}

#[test]
#[ignore = "tuning instrument, not a gate — run with --ignored --nocapture"]
fn death_rate_census() {
    let day_cap = 400;
    let mut by_cause: std::collections::BTreeMap<String, u32> = Default::default();
    let mut days_at_death = Vec::new();
    let mut survived = 0u32;
    let seeds: Vec<u64> = (1..=30).map(|i| i * 997).collect();
    for &seed in &seeds {
        let (cause, day, age) = run_life(seed, day_cap);
        match cause {
            Some(c) => {
                *by_cause.entry(format!("{c:?}")).or_default() += 1;
                days_at_death.push(day);
                println!("seed {seed:>6}: died day {day:>3} age {age:>3} of {c:?}");
            }
            None => {
                survived += 1;
                println!("seed {seed:>6}: alive at day-cap {day_cap} (age {age})");
            }
        }
    }
    println!(
        "\n=== death census over {} lives, {day_cap}-day cap ===",
        seeds.len()
    );
    for (c, n) in &by_cause {
        println!("  {c}: {n}");
    }
    println!("  survived to cap: {survived}");
    if !days_at_death.is_empty() {
        let mean: f64 =
            days_at_death.iter().map(|&d| d as f64).sum::<f64>() / days_at_death.len() as f64;
        println!("  mean death day: {mean:.0}");
    }
}
