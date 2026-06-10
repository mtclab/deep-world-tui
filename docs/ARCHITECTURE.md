# Architecture (Rust)

> Rust + ratatui TUI, single static binary, deterministic. Build the
> **headless-testable core first** (RNG + charts + generators + sim), then the TUI
> on top, then the optional LLM layer. Mirrors the sibling games' "tested core,
> thin UI" discipline so parallel work coheres.

## Crate layout

```
Cargo.toml
src/
  main.rs            // entry: parse args (--seed, --charts), init, run TUI app
  lib.rs             // re-exports the core modules (so tests + bins share them)
  rng.rs             // seedable RNG: ChaCha8Rng, splitmix64 sub-seed derivation
  charts/            // the possibility-chart engine (GENERATION.md)
    mod.rs           //   WeightedTable, ConditionalTable, Condition, sampling
    load.rs          //   serde load: --charts flag > DEEP_WORLD_CHARTS env var
                     //   > ~/.config/deep-world-tui/charts.ron > embedded fallback
  gen/               // generators (pure: (seed, charts) -> data; deterministic)
    world.rs         //   regions + settlements (river-corridor weights)
    person.rs        //   NPC sampling (people→region→class→profession→… )
    player.rs        //   player start (sample + reroll/point-buy)
    name.rs          //   per-people name grammar
    companion.rs     //   companion generation per settlement
  model/             // plain data types (serde), split into submodules:
    mod.rs           //   World, Region, module declarations + re-exports
    terrain.rs       //   Terrain, TerrainMap, PlayerPos, ExploredMap
    person.rs        //   GodName, PeopleKind, Person, Player, PlayerStart, Need
    vitals.rs         //   PlayerVitals, hunger/energy/needs
    weather.rs       //   TimeOfDay, Season, Weather, GameClock, FestivalKind
    encounter.rs     //   Encounter, EncounterKind, EncounterAction
    economy.rs       //   ItemType, Inventory, Settlement, SettlementPolitics
    companion.rs     //   Animal, Companion, CompanionMood
    memorial.rs      //   Memorial
    discovery.rs     //   DiscoveryStore
    relation.rs      //   InterNpcRelation
  sim/               // the consequence engine (CONSEQUENCES.md)
    mod.rs           //   SimState, time tick, needs decay, reputation, events
    effects.rs       //   Effect { Immediate | Deferred{at} }, apply/queue
    structures.rs    //   Structure generation, BuildSite, progress tracking
    weather.rs       //   Weather generation from seed + terrain + tick
    migration.rs     //   NPC migration between settlements
    wants.rs         //   NpcWant system
    journal.rs       //   Journal logging
    quests.rs        //   Quest generation and tracking
  ui/                // ratatui: app state, screens, input loop, rendering
    app.rs           //   App struct, state transitions, dispatch
    render.rs        //   draw() dispatcher → screen modules
    event.rs         //   crossterm event polling
    theme.rs         //   colour palette (monochrome / high-contrast support)
    screens/         //   per-screen draw functions (one file per screen)
      mod.rs          //   screen submodule declarations
      common.rs       //   shared helpers, MapViewport, build_npc_map
      map.rs          //   draw_map_screen (the roguelike map: terrain, @, NPCs)
      location.rs     //   draw_location_screen
      talk.rs         //   draw_talk_screen
      market.rs       //   draw_market_screen
      craft.rs        //   draw_craft_screen
      npc.rs          //   draw_npc_screen
      inventory.rs    //   draw_inventory_screen
      overmap.rs      //   draw_overmap_screen
      journal.rs      //   draw_journal_screen
      title.rs        //   draw_title_screen
      encounter_screen.rs  //   draw_encounter_screen
      encounter_log.rs    //   draw_encounter_log_screen
      game_over.rs    //   draw_game_over_screen
      collapse.rs     //   draw_collapse_screen
      help.rs         //   draw_help_screen
      settings.rs     //   draw_settings_screen
      save_browser.rs //   draw_save_browser_screen
      character_creation.rs //   draw_character_creation
      minimap.rs      //   render_minimap
      status_bar.rs   //   draw_status_bar
    input/           //   per-screen key handlers
      mod.rs          //   input submodule declarations
      world.rs        //   World screen: hjkl movement, Enter settlement, weather
      location.rs     //   Location screen input
      talk.rs         //   Talk screen input
      market.rs       //   Market screen input
      craft.rs        //   Craft screen input
      ... (one per screen)
  llm.rs             // optional /v1 narrator (reqwest); toggled; templated fallback
  voice.rs           // deterministic templated dialogue from a Person's traits
  save.rs            // serde save/load (seed + diffs, or full state) to a file
  config.rs          // user config file loading
data/
  charts.ron         // the possibility charts (lore-grounded; tunable at runtime)
tests/               // cargo tests (headless): determinism, distributions, sim math
```

## The Screen enum and render/input split

The `Screen` enum (in `ui/app.rs`) drives both rendering and input:

```rust
pub enum Screen {
    TitleScreen,
    CharacterCreation,
    World { region_idx: usize },  // the roguelike map — primary screen
    Location { region_idx: usize, settlement_idx: usize },
    Npc { region_idx: usize, settlement_idx: usize, person_idx: usize },
    Talk { ... },
    Market { ... },
    Craft { ... },
    Inventory,
    Overmap,
    Journal,
    EncounterLog,
    EncounterScreen { ... },
    Collapse { ... },
    GameOver,
    Help,
    Settings,
    SaveBrowser,
}
```

- **Rendering**: `draw()` in `render.rs` matches on `Screen` and calls the
  corresponding `screens::*::draw_*_screen()` function.
- **Input**: `handle_event()` in `app.rs` matches on `Screen` and calls the
  corresponding `input::*::handle_*_input()` function.

## The World screen (the roguelike map)

This is the primary game screen — not a menu. The player `@` walks on terrain
glyphs with hjkl/arrows. Key features:

- **Terrain**: Each tile has a glyph (`,` grass, `▓` forest, `≈` water, `▲` mountain,
  `:` deep desert, etc.) and a travel cost.
- **Fog of war**: Unexplored tiles show `·`. `reveal_around()` reveals adjacent tiles
  when the player moves.
- **Camera**: Viewport centered on player, clamped to map edges.
- **NPCs on map**: Settlement NPCs rendered as Greek-letter glyphs (yellow) at
  deterministic positions derived from seed+tick. Companions (green) adjacent to `@`.
- **Weather**: Glyph in header; storms/whiteouts force shelter (wait key `w`/`.`).
- **Structures/build sites/memorials**: Pre-built HashMap lookups per frame (O(1)
  per cell).
- **Region edges**: Walking off the edge transitions to the adjacent region.

## Charts override chain

Charts are embedded at build time (`include_str!`) but can be overridden at
runtime (AGENTS.md hard rule: "tunable without recompiling"):

1. `--charts <path>` CLI flag
2. `DEEP_WORLD_CHARTS` env var
3. `~/.config/deep-world-tui/charts.ron` if present
4. Embedded `include_str!` fallback

Invalid override falls back to embedded with eprintln warning.

## Crates

`ratatui` + `crossterm` (TUI) · `rand` + `rand_chacha` (seedable RNG) · `serde` +
`ron` (data + save) · `anyhow`/`thiserror` (errors) · `clap` (args) · `dirs`
(user config) · `reqwest` (blocking or tokio) **optional**, feature-gated `llm`,
for the /v1 layer.

## Determinism (hard rule)

- All generation flows from the **seed** via `rng.rs` sub-seed derivation. No
  `thread_rng`, no time/entropy in `gen/` or `sim/` scheduling.
- The sim is seeded too (deferred-event jitter, reputation spread). Same seed +
  same choices → same outcomes.
- NPC positions on the map are deterministic: `SeedRng::fork_for()` derives
  positions from (seed, region_idx, settlement_idx, person_idx), offset by tick.

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

### Save Versioning Policy

- `CURRENT_SAVE_VERSION` in `save_migrations.rs` tracks the format version.
- Every field addition to `SaveData` **must** use `#[serde(default)]` so old
  saves deserialize without error.
- Add a `migrate_vN_to_vN+1()` function for each version bump; register it in
  the `match` in `migrate()`.
- Saves from newer versions are rejected at load time with a clear error message.
- Test every migration step: v0→v1, v1→v2, … and vN round-trip through RON.

## Test strategy (gate — see AGENTS.md)

- `cargo build` clean, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- `cargo test` — must cover:
  - **determinism**: `gen` twice with the same seed → identical output.
  - **distributions**: 10k sampled people → sane spread (no profession > cap;
    per-people/region modifiers shift outcomes; names fit people).
  - **sim math**: needs decay, reputation spread/decay, deferred-event scheduling.
  - **chart integrity**: every outcome/condition id is defined.
  - **people glyphs**: distinct glyph per PeopleKind, deterministic.
- Core (rng/charts/gen/model/sim) is testable **without the TUI** — keep it that way.

## Build order (issues)

1. Scaffold Cargo project + toolchain + CI-ish `make check`.
2. `rng` + `charts` (weighted/conditional sampler) + tests.
3. `model` types + `data/charts.ron` starter (lore-grounded).
4. `gen` (person + player + name + world) + determinism/distribution tests.
5. `sim` consequence engine (needs/reputation/events) + tests.
6. `ui` (ratatui): roguelike map (World screen) → location → NPC → journal.
7. `voice` (templated) then `llm` (optional, toggled).
8. `save`/load. Polish, balance, content.