# Architecture (Rust)

> Rust + ratatui TUI, single static binary, deterministic. Build the
> **headless-testable core first** (RNG + charts + generators + sim), then the TUI
> on top, then the optional LLM layer. Mirrors the sibling games' "tested core,
> thin UI" discipline so parallel work coheres.

## Crate layout

```
Cargo.toml
src/
  main.rs            // entry: parse args (--seed), init, run TUI app
  lib.rs             // re-exports the core modules (so tests + bins share them)
  rng.rs             // seedable RNG: ChaCha8Rng, splitmix64 sub-seed derivation
  charts/            // the possibility-chart engine (GENERATION.md)
    mod.rs           //   WeightedTable, ConditionalTable, Condition, sampling
    load.rs          //   serde load of data/*.ron
  gen/               // generators (pure: (seed, charts) -> data; deterministic)
    world.rs         //   regions + settlements (river-corridor weights)
    person.rs        //   NPC sampling (people→region→class→profession→… )
    player.rs        //   player start (sample + reroll/point-buy)
    name.rs          //   per-people name grammar
  model/             // plain data types (serde): World, Settlement, Person,
                     //   Household, Relationship, Profession, Craft, …
  sim/               // the consequence engine (CONSEQUENCES.md)
    mod.rs           //   time tick, needs decay, reputation spread, scheduled events
    effects.rs       //   Effect { Immediate | Deferred{at} }, apply/queue
  ui/                // ratatui: app state, screens, input loop, rendering
    app.rs · screens/*.rs (create_character, world, location, npc, journal, …)
  llm.rs             // optional /v1 narrator (reqwest); toggled; templated fallback
  voice.rs           // deterministic templated dialogue from a Person's traits
  save.rs            // serde save/load (seed + diffs, or full state) to a file
data/
  charts.ron         // the possibility charts (lore-grounded; tunable at runtime)
tests/               // cargo tests (headless): determinism, distributions, sim math
```

## Crates (suggested)

`ratatui` + `crossterm` (TUI) · `rand` + `rand_chacha` (seedable RNG) · `serde` +
`ron` (data + save) · `anyhow`/`thiserror` (errors) · `clap` (args) · `reqwest`
(blocking or tokio) **optional**, feature-gated `llm`, for the /v1 layer.

## Determinism (hard rule)

- All generation flows from the **seed** via `rng.rs` sub-seed derivation. No
  `thread_rng`, no time/entropy in `gen/` or `sim/` scheduling.
- The sim is seeded too (deferred-event jitter, reputation spread). Same seed +
  same choices → same outcomes.

## NPC voice

- `voice.rs`: deterministic — assemble a line from the Person's people/profession/
  personality/bias + the situation (template/grammar). **Always available.**
- `llm.rs` (feature `llm`, **player-toggle in settings**): build a persona prompt
  from the same traits, POST OpenAI `/v1/chat/completions` (ollama/llama.cpp/vLLM),
  strip `<think>`, **fall back to `voice.rs` on any error or if disabled.** The
  game is fully playable with the LLM off (default).

## Save

`save.rs`: prefer **seed + the player's choices/diffs** (compact, and re-derivable)
or a full serde snapshot. Single-player, local file. Deterministic regen from seed
means saves can be small.

## Test strategy (gate — see AGENTS.md)

- `cargo build` clean, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- `cargo test` — must cover:
  - **determinism**: `gen` twice with the same seed → identical output.
  - **distributions**: 10k sampled people → sane spread (no profession > cap;
    per-people/region modifiers shift outcomes; names fit people).
  - **sim math**: needs decay, reputation spread/decay, deferred-event scheduling.
  - **chart integrity**: every outcome/condition id is defined.
- Core (rng/charts/gen/model/sim) is testable **without the TUI** — keep it that way.

## Build order (issues)

1. Scaffold Cargo project + toolchain + CI-ish `make check`.
2. `rng` + `charts` (weighted/conditional sampler) + tests.
3. `model` types + `data/charts.ron` starter (lore-grounded).
4. `gen` (person + player + name + world) + determinism/distribution tests.
5. `sim` consequence engine (needs/reputation/events) + tests.
6. `ui` (ratatui): character creation → world → location → NPC → journal.
7. `voice` (templated) then `llm` (optional, toggled).
8. `save`/load. Polish, balance, content.
