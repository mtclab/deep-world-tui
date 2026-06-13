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

// A bold traveler: one who crosses real country rather than nesting in a
// gathering spot. They walk most of the day (which rolls encounters), flee what
// they meet (the flee gamble), eat and drink only when the body demands it, and
// rest only at the edge of exhaustion — so they spend their days *worn*, which
// is exactly where mischance and a cursed star bite. This is the realistic
// "moderately skilled" life BALANCE.md sizes for, against which the cautious
// census reads as a best case.
fn run_bold_life(seed: u64, day_cap: u32) -> (Option<DeathCause>, u32, u32) {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let dirs = [(1i32, 0i32), (0, 1), (-1, 0), (0, -1)];
    let mut di = 0usize;
    while a.death_cause.is_none() && a.clock.day < day_cap {
        let day_start = a.clock.day;
        // Walk the day away: several steps across country, each a chance to
        // meet something. Flee it — and live with the gamble.
        let mut steps = 0;
        while steps < 6 && a.death_cause.is_none() && a.clock.day == day_start {
            a.move_player(dirs[di].0, dirs[di].1);
            // Cycle direction when a step is refused (edge, water, wall).
            di = (di + 1) % dirs.len();
            if a.encounter.is_some() {
                a.dismiss_encounter(); // flee
            }
            if a.collapse.is_some() {
                a.dismiss_collapse();
            }
            steps += 1;
        }
        if a.death_cause.is_some() {
            break;
        }
        // Resupply only when the body demands it — never travel rich.
        let inv = a.player_inventory();
        if inv.get(ItemType::Food) == 0 || inv.get(ItemType::Water) == 0 {
            a.gather();
            if a.encounter.is_some() {
                a.dismiss_encounter();
            }
            if a.collapse.is_some() {
                a.dismiss_collapse();
            }
            if let Some(ref mut ps) = a.player_start {
                if ps.inventory.get(ItemType::Water) == 0 {
                    ps.inventory.add(ItemType::Water, 1);
                }
            }
        }
        // Rest only at the edge of collapse, and only briefly — stay worn.
        if a.vitals.energy < 0.2 {
            a.rest_hours(5);
            if a.collapse.is_some() {
                a.dismiss_collapse();
            }
        }
        // Guarantee forward time even if every step was refused this day.
        if a.clock.day == day_start && a.death_cause.is_none() {
            a.advance_clock(6);
            if a.collapse.is_some() {
                a.dismiss_collapse();
            }
        }
    }
    (a.death_cause, a.clock.day, a.current_age_years())
}

#[test]
#[ignore = "tuning instrument, not a gate — run with --ignored --nocapture"]
fn bold_death_rate_census() {
    let day_cap = 400;
    let mut by_cause: std::collections::BTreeMap<String, u32> = Default::default();
    let mut days_at_death = Vec::new();
    let mut survived = 0u32;
    let seeds: Vec<u64> = (1..=30).map(|i| i * 997).collect();
    for &seed in &seeds {
        let (cause, day, age) = run_bold_life(seed, day_cap);
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
        "\n=== BOLD death census over {} lives, {day_cap}-day cap ===",
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
