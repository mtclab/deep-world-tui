# Slice 0 — agent tick ceiling (entity-first epic, deep-world-godot#50)

Measured before any refactor, to choose the two-rate cadence from real numbers.
Bench: `cargo run --release --bin agent_bench` (`src/bin/agent_bench.rs`).
Hardware: this dev host. `size_of::<Person>()` = **432 B** stack (+ heap for strings/vecs).

| agents | live tick | day-step | per 1k |
|-------:|----------:|---------:|-------:|
| 10k | 4.2 ms | 3.0 ms | ~0.42 ms |
| 50k | 23 ms | 22 ms | ~0.45 ms |
| 100k | 45 ms | 45 ms | ~0.45 ms |
| 250k | 110 ms | 110 ms | ~0.44 ms |

Linear: **~0.45 ms per 1,000 agents** for the needs-ladder decision pass.

## What it means

- **Active region is free.** A single settlement is hundreds to a few thousand
  agents → ≤4 ms/hour-tick. Live full-resolution is never a problem.
- **Daily cadence is affordable province-wide.** Advancing *every* inactive
  agent at province scale (~100k souls) one day-step ≈ **45 ms** — inside the
  ~100 ms turn budget, run once/day not once/hour. So option-3's coarse layer
  can be **daily**, not forced to seasonal. Seasonal is a cheaper fallback if
  the real per-agent step (full slice-3 ladder + economy) lands heavier than
  this stand-in.
- **The tax was `Needs: HashMap<Need,f64>`** — 5 hash ops/agent dominated the
  432 B and the time.

## Slice 1a update — Needs → `[f64; 5]`

Replacing the HashMap with a flat array (API and save wire-format unchanged)
cut the per-agent cost **~10×**:

| agents | live tick (before → after) |
|-------:|---------------------------:|
| 100k | 45 ms → **4.5 ms** |
| 250k | 110 ms → **11 ms** |

~0.045 ms / 1,000 agents. The HashMap was the entire tax. Province-scale
daily-cadence advance is now a few ms, not tens — wide headroom for the real
slice-3 ladder + per-agent economy to grow into.

## Caveats

- This measures the per-agent **decision** only; the full tick has ~20 other
  system passes. Re-bench after slice 3 with the real ladder + per-agent economy.
- `day-step ≈ live-tick` here because the stand-in resolves a day in one pass
  (dt=24) rather than looping 24×. Real amortization (skipping 23 hourly
  decisions) makes the coarse layer relatively cheaper than shown.

## Verdict for the epic

Per-agent cost is **not** the wall — memory + save size are (as expected).
Coarse cadence = **daily**, ceiling for active region effectively unbounded at
realistic settlement sizes. Proceed to slice 1 (slim `Person`, kill the
`Needs` HashMap first).
