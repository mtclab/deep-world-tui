# Changelog

All notable changes to **Deep World TUI** are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 2 — God-prayer mini encounter (#141)

- **#141** God-prayer mini-encounter. When the player rests in a settlement
  whose dominant people has a patron (Metsik→Keuru, Sepät/Ahjo→Oltzed,
  Arkit→Sampsa, Väylä/Mëräk→Masa, Laakso/Tzäkhar/She'ar→Kukri, Häl→Keuru,
  Khör→Sampsa), a sensory first-person dream line is deterministically
  appended to the journal on roughly one rest in four. The line is pure
  flavor: no god name, no people name, no number. Implementation lives in
  `src/sim/god.rs` and is wired into `App::rest()`.

### Phase 2 — Indirect bond descriptors (#137)

- **#137** NPC bond % no longer leaks to the player in inter-NPC relationship lines. The `→/← other {:?} str=XX% trust=YY%` line is replaced with `→/← other — bond_descriptor. regard_descriptor.` using the existing `bond_descriptor()` function from `sim::relationships`. The `BondCategory` import is removed from `render.rs`.
  - 387 tests pass.

### Phase 2 — Indirect reputation signals (#135)

- **#135** Player deduces reputation from world's reactions; game never prints a number, label, or bar.
  - `sim::signals` module: `ReputationBand` (Hostile / Cold / Neutral / Warm / Revered), `EngagementLevel` (Refuses / Reluctant / Neutral / Willing / Eager), body-language strings keyed off FNV-1a hash of person id.
  - `reputation_price_modifier` layers multiplicatively on top of `inter_people_bias`; rep 0.0 → 1.5× (worst price), rep 1.0 → 0.6× (best price).
  - `App::reputation_in_current_settlement`, `quote_buy_price`, `quote_sell_price`, `npc_will_engage` added; `buy_item` / `sell_item` / `use_service` use the quote helpers.
  - `EncounterKind::can_have_outside_help()` introduced; deterministic 1-2% seed-rolled intervention: at rep ≥ 0.7 a passing trader intervenes in `Bandit` / `Wildlife` encounters; at rep ≤ 0.25 the attacker cuts and runs. Single `Voice::Travel` journal entry per intervention.
  - 17 unit tests including 4 leak-guard tests asserting no `Refuses` / `Warm` / `reputation` / `level=` token ever leaks from any band across the full rep range.
  - 383 tests pass (366 baseline + 17 signals).

### Added
- #127 Illness/disease balance pass
- `Person.illnesses: Vec<ActiveDisease>` field (serde default)
- `sim/illness.rs`: illness contraction per tick based on health, shelter, healer presence
- Low health (<0.5 Food need) increases illness probability
- Low shelter (Safety <0.3) → 1.5× illness rate
- Missing healer → 1.5× illness rate
- Cap: max 30% of settlement ill, max 2 diseases per person
- `illness::apply_illness_effects()` removes recovered diseases each tick
- `illness::illness_productivity_modifier()` returns 0.7× for ill NPCs
- `illness::settlement_has_healer()` checks for Temple/Shrine services or healer/herbalist profession
- Wired into `sim_tick` via `tick_npc_illness()`

## [0.1.0] - 2026-06-07

First public release. Terminal adventure TUI built on ratatui, deterministically
generated from a seed via a chart engine, with optional audio (`hound`) and LLM
narrator (`reqwest`) feature gates.

### Phase 1 — Foundation (issues 1–16)

- **#1** Scaffold Cargo project with crate layout and `make check` target.
- **#2** Seedable RNG: splitmix64 with sub-seed derivation and `fork_for`.
- **#11** Sub-seed derivation + deterministic forking helpers.
- **#12** `WeightedTable` sampling.
- **#13** `ConditionalTable` resolve + sample (modifier system).
- **#14** Chart data loading from RON + round-trip serde test.
- **#15** Charts RON integrity validator (every outcome references defined ids).
- **#16** Model types: `Person`, `Settlement`, `Region`, `World` (serde round-trip).

### Phase 2 — Generators (issues 17–28)

- **#17** Model types: `Household`, `Relationship`, `Craft`, `Need`.
- **#18** Model types: `Player`, `PlayerStart` (reroll / point-buy stubs).
- **#19** Name generator: per-peoples name grammars + test data.
- **#20** Name generator: sampling + determinism test.
- **#21** Person generator: full pipeline (seed → charts → `Person` with all fields).
- **#22** Distribution test: 10k sample, profession caps, per-peoples shifts.
- **#23** Player generator: sample from charts + reroll stub.
- **#24** World generator: regions → settlements → population.
- **#25** River-corridor density principle (region weights).
- **#26** World generator determinism test (same seed → identical world).
- **#27** Need enum: `food`, `money`, `care`, `presence`, `safety`.
- **#28** Need decay tick (needs degrade over time without attention).
- **#29** Dependent needs (child needs → parent obligation).

### Phase 3 — Reputation, voice, simulation (issues 30–55)

- **#30** Reputation: local + by-faction, spread + decay.
- **#31** Per-NPC relationship tracking (bond + trust + history).
- **#36** Leave-household screen flow.
- **#37** TUI terminal harness (ratatui + crossterm).
- **#38** Character creation screen.
- **#39** Location detail screen.
- **#40** NPC detail screen.
- **#41** Journal screen.
- **#46** `voice.rs`: craft-specific dialogue hooks.
- **#49** LLM narrator: persona prompt assembly from `Person` traits.
- **#50** Save/load: serde snapshot (full state to file).
- **#51** Save/load: seed + choice-diff format (compact, re-derivable).
- **#52** Save/load: load → regenerate world from seed + apply diffs.
- **#53** Charts lore-tune pass (starter weights vs deep-world-history docs).
- **#54** Charts: non-human peoples (`Vaskarii`, `Merak`, `Shear`, `Hal`, `Khor`) as rare options.
- **#55** Charts: regional sub-types and settlement naming.

### Phase 4 — Polish + integration (issues 56–60)

- **#56** Key-bindings help screen + settings menu.
- **#57** Color/theming for different peoples' regions in TUI.
- **#58** Integration test: full pipeline (seed → generate world → enter location → talk to NPC).
- **#59** Balance: needs decay rates, reputation spread rates, deferred-effect timing.
- **#60** `cargo xtask check` target (replaces Makefile).

### Phase 5 — Mechanics (issues 101–106)

- **#101** Equipment durability & repair.
- **#102** NPC memory (past interactions).
- **#103** NPC scheduling (time-of-day).
- **#104** Combat/dueling system.
- **#105** Weather effects.
- **#106** Settlement & farm management.

### Phase 6 — Late mechanics + framing (issues 107–115)

- **#107** Quests & journal hooks.
- **#108** Caravan & animal companions.
- **#109** Illness & medicine.
- **#110** Reputation reputation-spread tuning pass.
- **#111** Encounter-resolution polish.
- **#112** Obligation tracking.
- **#113** Minimap rendering.
- **#114** Audio & sound effects (optional feature, `hound`).
- **#115** i18n localization framework (en + fi RON locales).

### Statistics

- **115 issues** closed.
- **357 unit + integration tests**, all green.
- **`cargo xtask check`** (fmt + clippy `--all-targets -D warnings` + build + test) green
  on default, `--no-default-features`, and `--features audio`.

### Features (Cargo)

```toml
[features]
default  = []              # vanilla, no native deps
audio    = ["hound"]       # synthesize + play SoundEvents
llm      = ["reqwest"]     # optional LLM narrator (off by default)
```

### Audio

`SoundEvent` variants: `UiClick`, `UiCancel`, `Gather`, `Combat`, `Trade`,
`Weather`, `Ambient`. `AppSettings.audio_enabled` + `audio_volume` exposed in the
Settings screen; bound to `[a]` toggle and `[+]/[-]` volume keys. `App::play_sound()`
hooked into gather and Intimidate/Trade encounter actions.

### i18n

`Locale { code, strings: HashMap<String, String> }` loaded from
`data/locales/{en,fi}.ron`. `t(key)` + `t_fmt(key, args)` accessors. `t!` macro
for inline string lookup. `AppSettings.language` (default `"en"`); Settings screen
`[g]` cycles en ↔ fi.

### Determinism Law

All randomness flows from a single seed via `ChaCha8Rng` and splitmix64-derived
sub-seeds. No `thread_rng` / `time` / `entropy` in `gen/` or `sim/`. Same seed →
byte-identical world + NPCs + schedules.

### Canon Compliance

`docs/DEEP_WORLD_LORE.md` + the chart data are the single source of truth for
people, gods, geography, and naming. Hard rules enforced:

- Five Gods are Oltzed, Keuru, Sampsa, Masa, Kukri — never used as people names.
- People → God mapping: Metsik→Keuru, Sepät/Ahjo→Oltzed, Arkit→Sampsa, Väylä→Masa, Laakso→Kukri.
- God-peoples (SAST 6) are minorities; the "peoples who stayed" are the majority.
- Five non-human peoples are rare and never dominate a region.
- No affinity / bias / opinion numbers shown to the player — only flavor text,
  stance labels, and downstream consequences.

### Build

```bash
cargo xtask check              # fmt + clippy + build + test
cargo test --features audio    # with synthesized audio tests
cargo test --no-default-features
```

[Unreleased]: https://github.com/mtclab/deep-world-tui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mtclab/deep-world-tui/releases/tag/v0.1.0
