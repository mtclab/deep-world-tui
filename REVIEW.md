# Mechanic Review Protocol

A deep-dive, per-mechanic scrutiny pass. Each mechanic/module has a tracking
issue (see the **Mechanic Audit** epic). A mechanic is "reviewed" only when
every box below is checked **and** its regression tests are merged.

This exists because the journal `voice` save bug (a serialize/deserialize
asymmetry) shipped through a code-read-only review with no played-state
roundtrip test. Reviews are now behavior-verified, not just read.

## Per-mechanic checklist

For each module under review:

- [ ] **Wiring** — the system is actually invoked in gameplay (traced from a
      `playtest` session or a test), not just defined. List the call site(s).
- [ ] **Serde roundtrip** — every persisted type survives the production
      compact-RON path (`save_to_slot` → `load_game_file`). Watch for
      serialize/deserialize **asymmetry** (e.g. a field serialized as a bare
      enum but read into an `Option`). Compact RON has **no implicit-some**.
- [ ] **Legacy migration** — saves missing newly-added fields still load
      (`#[serde(default)]` / migration in `save_migrations.rs`).
- [ ] **Boundary values** — zero, max, empty collections, day/season/year
      rollover, age-band edges, negative/overflow guards.
- [ ] **Determinism** — same seed + same inputs ⇒ same state (no `HashMap`
      iteration leaking into RNG/order-sensitive output; use `IndexMap`).
- [ ] **Docs cross-check** — verify crate behavior against current docs via
      **context7** (`ron`, `serde`, `ratatui`, `indexmap`) before asserting how
      an API behaves. Don't trust memory for serialization edge cases.
- [ ] **Regression test** — a test that fails on the bug and passes on the fix,
      committed alongside.

## Pre-merge gates (enforced by CI — `.github/workflows/ci.yml`)

No PR merges until all of these are green:

1. `cargo fmt --check --all`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --all` — including:
   - `save_played_state_roundtrip_test` (played state, multiple seeds)
   - `journal_roundtrip_test` (every enum variant + legacy entry)
4. **playtest smoke** — scripted session on seeds 42/555/777 must show no
   `Load failed` / `panicked` / `Save failed`.
5. Three-target release build (linux-gnu, linux-musl, windows-gnu).

## When adding a new persisted field

1. Add it with `#[serde(default)]` (or a `default = "fn"`).
2. Keep serialize/deserialize **symmetric** — if the type is `T`, read `T`, not
   `Option<T>` (use a default fn for migration instead).
3. Add the new state to a `playtest` smoke action so the gate exercises it.
