# AGENTS.md — build guide for coding agents

You are implementing **Deep World (TUI)**, a Rust + ratatui procedural life-RPG.
Read this fully before coding. Tasks are GitHub issues; this is the ground truth.

## Read first (design)

- `docs/DESIGN.md` — the pitch + pillars (organic procedural life-RPG; freedom + consequence).
- `docs/GENERATION.md` — the possibility-chart engine (the heart). **Determinism is law.**
- `docs/CONSEQUENCES.md` — the choice→consequence spine.
- `docs/ARCHITECTURE.md` — crate layout, modules, crates, test strategy, build order.
- `data/charts.ron` — starter lore-grounded charts (tune vs the lore bible).

Do **not** invent lore. Ground charts/names/professions in
`../deep-world-history/src/docs/` (cited in GENERATION.md §5). Canon: SAST
people-names (Metsik, Arkit, Väylä, Laakso, Sepät, Ahjo), the **Kingdom of
Ahjorath**, magic is bounded *enhancement* (no spells). Don't edit
`../deep-world-history` from here.

## ⚠️ CANON RULES (HARD CONSTRAINTS)

The deep-world-history lore bible is ground truth. Violations break the game world.

### The Five Gods (NON-NEGOTIABLE)

1. **Oltzed** — Labor, invention, engineering, machinery, construction. Metal sky-chariot. Travels. (Patron of Sepät, Ahjo; also revered by Arkit)
2. **Keuru** — Forests, hospitality, celebration, travel, social life. Charismatic extrovert. (Patron of Metsik; also revered by Väylä)
3. **Sampsa** — Knowledge, memory, archives, astronomy, forgotten tech. Tireless scholar. (Patron of Arkit)
4. **Masa** — Trade, perseverance, loyalty, common people. Dependable merchant. (Patron of Väylä)
5. **Kukri** — Solitude, old wisdom, nostalgia, quiet kindness. Melancholic hermit. (Patron of Laakso)

**GOD NAMES ARE NEVER PEOPLE NAMES.** Metsik is a people, not a god. Ahjo is a people, not a god. Väylä is a people, not a god. The gods are Oltzed, Keuru, Sampsa, Masa, Kukri. Period.

People→God patron mapping:
- Metsik → Keuru
- Sepät → Oltzed  
- Ahjo → Oltzed
- Arkit → Sampsa
- Väylä → Masa
- Laakso → Kukri
- Tzäkhar → Kukri (deep solitude)
- Mëräk → Masa (sea trade)
- She'ar → none / Kukri (desert hermits)
- Häl → Keuru (forest communion)
- Khör → Sampsa (oral tradition keepers)

### The Six Human Peoples
Metsik, Arkit, Väylä, Laakso, Sepät, Ahjo — these are ENDONYMS (self-names). God-derived names like Keurimä, Sampsari, Sepät (forge-born), Masari, Kukreva are EXONYMS used by others.

### The Five Non-Human Peoples
Tzäkhar (deep/cave), Mëräk (sea/coastal), She'ar (desert), Häl (canopy/forest), Khör (tundra/steppe). They exist alongside humans. They are minorities in most regions but majorities in their home biomes. Playable but rare (~5-15% of encounters in their biomes).

### Terrain→God Mapping (CANON)
- Forest → Keuru (forests, hospitality)
- Grass/Farmland/Settlement → Oltzed (construction, settlement) or Masa (trade)
- Mountain → Oltzed (forge, engineering)
- Road → Masa (trade, travel) or Keuru (hospitality)
- Water/Coast → Masa (trade, rivers)
- Swamp → Kukri (solitude, wisdom)
- Sand/DeepDesert → Kukri (hermits, solitude) or She'ar gods
- Cave → Kukri or Sampsa (deep knowledge)
- Tundra/Steppe → Kukri (endurance, solitude)

### What Was Wrong (Fixed 2026-06-07)
The GodName enum previously had Metsik/Ahjo/Vayla — PEOPLE names used as GOD names. This was a fundamental category error. All code has been refactored to use the canonical five gods.

## Toolchain

No Rust on the base image — install once (user-local, no root):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
rustc --version && cargo --version
```

## Build & verify (REQUIRED before every commit)

```bash
. "$HOME/.cargo/env"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test                # determinism + distribution + sim + chart-integrity tests
```
All must pass. If your feature has logic, **add tests** (see ARCHITECTURE.md test
strategy). The core (`rng`/`charts`/`gen`/`model`/`sim`) must stay testable
**without the TUI** — never make core logic depend on ratatui.

## Hard rules

- **Determinism:** all generation + sim randomness flows from the **seed** via
  `rng.rs` sub-seeds. **No `thread_rng`, no time/entropy** in `gen/`/`sim/`. A test
  must assert same-seed → identical output.
- **No "99% soldiers":** profession/etc. are *conditional* weighted distributions
  (base rates + per-people/region/class modifiers). A distribution test must assert
  sane spread (see GENERATION.md §6).
- **The LLM is optional + off by default:** templated `voice.rs` always works;
  `llm.rs` is feature-gated (`llm`) + player-toggled + falls back to templates. The
  game must build + play with `--no-default-features` (no LLM).
- **Charts are data:** weights live in `data/*.ron`, tunable without recompiling.

## Workflow

1. Branch `feat/<issue>-<slug>` off `master`.
2. Implement; run the verify block; add tests for logic.
3. Commit (`Closes #N`), open a PR to `master`. In the PR: what you did, the
   `cargo test` output, and anything to check interactively.
4. One issue per PR. Keep the core UI-independent.

## Done =

`fmt`+`clippy`+`build`+`test` green · determinism asserted · distributions sane ·
LLM optional/off-by-default still builds · canon-grounded · PR explains testing.
