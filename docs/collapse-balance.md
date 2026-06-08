# Collapse Balance Design

## Death Probability

The collapse system uses a deterministic weighted distribution for outcomes
and a flat threshold for death probability.

### Current Threshold

The death check in `Collapse::roll()` uses:

```rust
let val = (hash % 1000) as u32;
let died = val < 12;  // 1.2% base death rate
```

The `val < 12` threshold means 12 out of 1000, or **1.2% base probability**.
This is before god affinity, reputation, and people-bias modifiers which
can shift outcome weights (but do NOT change the death probability directly).

### Target Range

Playtest harness (`tests/collapse_balance_test.rs`) asserts:

- **Death rate: 0.5%–2.0%** across 10 deterministic playthroughs of 2000 ticks each
- **Variance proof**: at least one run has 0 deaths, at least one has ≥2 deaths
- **Determinism**: same seed must produce identical results on re-run

### Outcome Weight Table (Base)

| Outcome          | Weight | Notes                        |
|------------------|-------:|------------------------------|
| GodCampsite      |      3 | Rarest; divine intervention  |
| BeastGuarded     |      5 | Beast protects you           |
| FestivalBench    |      5 | Community warmth             |
| SettlementBed    |     10 | Safe recovery                |
| HostileBeast     |     30 | Dangerous encounter           |
| Riverbank        |     30 | Washed up safe               |
| WaysideShrine    |     50 | Quiet recovery               |
| StrangerHut      |    100 | Anonymous aid                |
| BeastNest        |    120 | Unpleasant but survived      |
| Ditch            |    150 | Worst common outcome          |

### Modifiers

- **God affinity**: Strongest ally shifts weights toward favorable outcomes;
  strongest grudge shifts toward hostile outcomes
- **Local reputation** ≥ 0.7: +30 to Ditch (reduced), +60 to SettlementBed
- **Local reputation** ≤ 0.15: +40 to Ditch, -20 to StrangerHut
- **Keuru affinity** > 0.5: +60 to BeastGuarded, -30 to HostileBeast
- **Masa affinity** > 0.4: +40 to Riverbank
- **Bias** < -0.15 (inter-people): Degrades StrangerHut→Ditch, SettlementBed→Ditch,
  FestivalBench→Ditch
- **Bias** > 0.05: Upgrades Ditch→StrangerHut

### Adjustments

If playtest data shows death rate outside 0.5–2.0%, the threshold in
`src/model/mod.rs` at line ~1895 should be adjusted:

- Increase `val < N` → higher death rate
- Decrease `val < N` → lower death rate

The threshold is a single integer; current value is 12 (yielding 1.2%).

### Collapse Event Logging

All collapse events are recorded in `App::collapse_log` as `CollapseEvent` structs,
containing tick, vitals before collapse, region, weather, outcome, death flag,
and rescuing god. This data is available for playtest analysis and persists
through save/load.