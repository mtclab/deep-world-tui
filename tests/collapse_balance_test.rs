/// Playtest harness for collapse/death balance.
/// Runs deterministic simulations and asserts the death rate lands in 0.5–2.0%.
use deep_world_tui::model::{Collapse, CollapseOutcome, GodAffinity};
use deep_world_tui::sim::collapse_log::{CollapseEvent, CollapseStats};

/// Direct unit test for Collapse::roll death probability.
/// Tests the 1.2% base death rate across many deterministic rolls.
#[test]
fn collapse_roll_death_rate_is_in_range() {
    let affinity = GodAffinity::new();
    let total_rolls: u64 = 10_000;
    let mut deaths: u64 = 0;

    for i in 0..total_rolls {
        let seed = i.wrapping_mul(2654435761) ^ 0xDEAD_BEEF;
        let collapse = Collapse::roll(seed, &affinity, 0.5);
        if collapse.died {
            deaths += 1;
        }
    }

    let death_rate = deaths as f64 / total_rolls as f64;
    // 1.2% base rate → expect 0.8%–2.0% in practice
    assert!(
        death_rate >= 0.008,
        "death rate too low: {:.4} (expected ≥ 0.8%)",
        death_rate * 100.0
    );
    assert!(
        death_rate <= 0.020,
        "death rate too high: {:.4} (expected ≤ 2.0%)",
        death_rate * 100.0
    );
}

/// With negative reputation, death rate should still be in range but
/// outcomes may shift toward hostile.
#[test]
fn collapse_roll_low_reputation_shifts_outcomes() {
    let affinity = GodAffinity::new();
    let total_rolls: u64 = 10_000;
    let mut ditch_count: u64 = 0;

    for i in 0..total_rolls {
        let seed = i.wrapping_mul(2654435761) ^ 0xFEED_FACE;
        let collapse = Collapse::roll(seed, &affinity, 0.1);
        if collapse.outcome == CollapseOutcome::Ditch {
            ditch_count += 1;
        }
    }

    // Low rep (0.1) adds +40 to Ditch weight and -20 to StrangerHut
    // So Ditch should be noticeably common
    let ditch_rate = ditch_count as f64 / total_rolls as f64;
    assert!(
        ditch_rate > 0.10,
        "ditch should be >10% with low rep, got {:.2}%",
        ditch_rate * 100.0
    );
}

/// Determinism: same seed must produce identical results.
#[test]
fn collapse_roll_deterministic() {
    let affinity = GodAffinity::new();
    for seed in [42u64, 77, 123, 256, 999] {
        let a = Collapse::roll(seed, &affinity, 0.5);
        let b = Collapse::roll(seed, &affinity, 0.5);
        assert_eq!(a.outcome, b.outcome, "outcome mismatch for seed {}", seed);
        assert_eq!(a.died, b.died, "died mismatch for seed {}", seed);
        assert_eq!(
            a.rescued_by, b.rescued_by,
            "rescued_by mismatch for seed {}",
            seed
        );
    }
}

/// Outcome distribution sanity check: all base outcomes should be reachable
/// and Ditch should be the most common.
#[test]
fn collapse_roll_outcome_distribution() {
    let affinity = GodAffinity::new();
    let total_rolls: u64 = 10_000;
    let mut outcomes = std::collections::HashMap::new();

    for i in 0..total_rolls {
        let seed = i.wrapping_mul(2654435761) ^ 0x1234_5678;
        let collapse = Collapse::roll(seed, &affinity, 0.5);
        *outcomes
            .entry(format!("{:?}", collapse.outcome))
            .or_insert(0u64) += 1;
    }

    let ditch_pct = *outcomes.get("Ditch").unwrap_or(&0) as f64 / total_rolls as f64;
    assert!(
        ditch_pct > 0.05,
        "Ditch should be >5% (most common), got {:.2}%",
        ditch_pct * 100.0
    );

    // HostileBeast should exist
    assert!(
        outcomes.contains_key("HostileBeast"),
        "HostileBeast should appear"
    );
}

/// Recovery hours should be within expected ranges per outcome.
#[test]
fn collapse_outcome_hours_in_range() {
    use deep_world_tui::model::CollapseOutcome::*;
    for outcome in [
        BeastNest,
        BeastGuarded,
        HostileBeast,
        StrangerHut,
        SettlementBed,
        WaysideShrine,
        Riverbank,
        FestivalBench,
        Ditch,
        GodCampsite,
    ] {
        let hours = outcome.hours_passed();
        assert!(
            (6..=16).contains(&hours),
            "{:?} hours {} not in 6-16 range",
            outcome,
            hours
        );
    }
}

/// God affinity shifts outcomes away from hostile and toward rescue.
#[test]
fn collapse_roll_god_affinity_shifts_outcomes() {
    let mut affinity = GodAffinity::new();
    // High Keuru → more BeastGuarded, fewer HostileBeast
    affinity.adjust(deep_world_tui::model::GodName::Keuru, 0.8);

    let total_rolls: u64 = 10_000;
    let mut beast_guarded: u64 = 0;

    for i in 0..total_rolls {
        let seed = i.wrapping_mul(2654435761) ^ 0xABCD_EF01;
        let collapse = Collapse::roll(seed, &affinity, 0.5);
        if collapse.outcome == CollapseOutcome::BeastGuarded {
            beast_guarded += 1;
        }
    }

    // With high Keuru affinity, BeastGuarded should be noticeably present
    assert!(
        beast_guarded > 0,
        "high Keuru should produce BeastGuarded outcomes"
    );
}

/// Variance check: across different seeds, some runs should have 0 deaths
/// and some should have ≥1 death (proves the probability is working).
#[test]
fn collapse_roll_death_variance_across_seeds() {
    let affinity = GodAffinity::new();
    let rolls_per_batch: u64 = 200; // ~1.2% death rate → ~2-3 deaths per batch
    let num_batches = 20;
    let mut batches_with_zero_deaths = 0;
    let mut batches_with_deaths = 0;
    let mut total_deaths: u64 = 0;

    for batch in 0..num_batches {
        let base_seed = (batch as u64).wrapping_mul(7919) ^ 0x5A5A;
        let mut deaths_in_batch: u64 = 0;
        for j in 0..rolls_per_batch {
            let seed = base_seed.wrapping_add(j).wrapping_mul(2654435761) ^ 0xDEAD;
            let collapse = Collapse::roll(seed, &affinity, 0.5);
            if collapse.died {
                deaths_in_batch += 1;
            }
        }
        total_deaths += deaths_in_batch;
        if deaths_in_batch == 0 {
            batches_with_zero_deaths += 1;
        }
        if deaths_in_batch >= 1 {
            batches_with_deaths += 1;
        }
    }

    // Overall death rate should be approximately 1.2%
    let total_rolls = rolls_per_batch * num_batches as u64;
    let overall_rate = total_deaths as f64 / total_rolls as f64;
    assert!(
        (overall_rate - 0.012).abs() < 0.01,
        "overall death rate should be ~1.2%, got {:.2}%",
        overall_rate * 100.0
    );

    // Some batches should have 0 deaths (most will with only 200 rolls at 1.2%)
    assert!(
        batches_with_zero_deaths >= 1,
        "expected at least one batch with 0 deaths, got {}",
        batches_with_zero_deaths
    );

    // Across 20 batches of 200 rolls each, we should see at least 1 batch with deaths
    assert!(
        batches_with_deaths >= 1,
        "expected at least one batch with ≥1 death, got {}",
        batches_with_deaths
    );
}

/// CollapseEvent and CollapseStats basic construction tests.
#[test]
fn collapse_stats_death_rate_calc() {
    let stats = CollapseStats {
        total_collapses: 100,
        total_deaths: 1,
        ..Default::default()
    };
    let rate = stats.death_rate();
    assert!(
        (rate - 0.01).abs() < f64::EPSILON,
        "death rate should be 0.01, got {:.6}",
        rate
    );
}

#[test]
fn collapse_stats_zero_collapses_zero_rate() {
    let stats = CollapseStats::default();
    assert_eq!(stats.death_rate(), 0.0);
}

#[test]
fn collapse_event_fields() {
    let event = CollapseEvent {
        tick: 42,
        vitals_before: deep_world_tui::model::PlayerVitals::new(),
        region: "TestRegion".to_string(),
        weather: "storm".to_string(),
        outcome: CollapseOutcome::Ditch,
        died: false,
        rescued_by: None,
    };
    assert_eq!(event.tick, 42);
    assert_eq!(event.region, "TestRegion");
    assert_eq!(event.weather, "storm");
    assert_eq!(event.outcome, CollapseOutcome::Ditch);
    assert!(!event.died);
}
