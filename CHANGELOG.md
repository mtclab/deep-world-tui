# Changelog

All notable changes to **Deep World TUI** are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-06-09

Patch release: 20 feature merges, 3 critical bug fixes, 2 hardening fixes.

### Fixed

- **Weather::generate** produced only 7 of 11 variants. Whiteout, Thunderhead,
  DryLightning, and SeaSquall were unreachable due to missing weights in the
  terrain match. All 11 variants now have terrain-specific probabilities.
- **Season::from_day(0)** panicked on u32 underflow `(0 - 1) % 90`. Day 0 now
  returns `Season::Thaw`.
- **SeedRng::fork_for** ignored the parent seed — `splitmix64(domain_hash)`
  produced identical sub-RNGs regardless of seed. Now mixes seed via
  `splitmix64(seed ^ domain_hash)`.
- **save.rs path traversal**: filenames containing `/`, `\`, or `..` are now
  rejected. All saves forced to the `saves/` directory.
- **HashMap → IndexMap** in sim/reputation, sim/relationships, sim/npc_memories,
  sim/needs_dependent, and model/Inventory for deterministic iteration order
  across process restarts.

### Added

- **#127** Illness/disease balance pass — `ActiveDisease`, illness contraction per
  tick based on health/shelter/healer, recovery tick, productivity modifier.
- **#128** Personality-driven gossip flavor — NPC trait adjectives flavor
  encounter dialogue.
- **#129** Expand charts — weather weights, animals, diseases, professions, crafts.
- **#124** Weather affects travel speed and encounter chance —
  `travel_time_minutes`, `forced_shelter`, `encounter_rate_modifier`.
- **#131** Death rebalance with playtest harness — `CollapseEvent`,
  `CollapseStats`, 10-seed playtest, death rate 0.5–2.0%.
- **#126** NPC migration between settlements — `tick_migration()`, job-seeking,
  marriage, flight (threshold 0.15).
- **#133** Death scene lineage save & continue-as-NPC — `LineageRecord`,
  `save_lineage()`, continues as a new NPC in the same settlement.
- **#123** Save format migration helper — `save_migrations.rs`, `version` field
  on `SaveData`, `CURRENT_SAVE_VERSION = 1`.
- **#130** Swedish locale removed — deleted `src/i18n.rs`, `data/locales/`,
  language cycling keybinding. `language` field kept as no-op.
- **#135** Indirect reputation signals — `ReputationBand`, `EngagementLevel`,
  body language, `reputation_price_modifier`, outside help at high rep.
- **#137** Bond descriptors — replaced % display with organic flavor in
  inter-NPC relationship lines.
- **#141** God-prayer mini encounter — sensory dream on rest in patron
  settlement, 5 canonical gods.
- **#145** Time-of-day rules — 7 `TimeOfDay` phases, `blocks_services()`,
  DeepNight forced `OutInCold` rest, dream flavor.
- **#146** Seasons — Thaw/Green/Frost, 90-day year, gather/decay/bias modifiers.
- **#143** Hunger & thirst core loop — `PlayerVitals.thirst`, `ItemType::Water`,
  auto-drink threshold, dehydration labels.
- **#138** Discoveries & landmarks — 12 `DiscoveryKind` variants, one-shot
  permanent features, journal + world-screen display.
- **#144** Rest quality by location — 5-tier `RestQuality`, recovery/encounter
  risk per tier, DeepNight override.
- **#139** Voice journal entries — `Voice` enum (Encounter/Travel/Rest/Dream/
  Scar/Rumor), per-voice color rendering.
- **#140** Inter-NPC relationships — `RelationKind`, `InterNpcRelation`,
  relationship decay in sim tick.
- **#142** Death memorials — `Memorial` struct, ⚰ glyph on map,
  recovery-region bonus.
- **#147** NPC goal-driven daily behavior — `WantKind`, `NpcWant`, wants tick.
- **#148** Companion adoption — Animal `Hound/Donkey/Crow`, `[a]` key, rest/decay.
- **#149** Camping structures — 8-tier `BuildKind`, `Structure`, `BuildSite`,
  `[b]` key, camp materials.

### Statistics

- **541 tests** (513 unit + 10 playtest + 4 consequence + 8 lineage + 4
  migration + 2 integration), all green.
- **24,300+ lines** of Rust across 46 source files.
- **`cargo clippy --all-targets -- -D warnings`**, `cargo build`, `cargo test`
  all green on default and `--no-default-features`.

### Breaking Changes

- **Save format bumped to v1**. Old saves (version 0) auto-migrate on load.
- **`SeedRng::fork_for` output changed** due to seed-mixing fix. Worlds
  generated before this release produce different sub-RNGs for the same domain.
  Existing save files will regenerate correctly from their own seed.

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

[0.1.1]: https://github.com/mtclab/deep-world-tui/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/mtclab/deep-world-tui/releases/tag/v0.1.0
