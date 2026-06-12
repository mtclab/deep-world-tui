# Changelog

## v0.6.0 (2026-06-12)

- **Canon scale epic (#378, PRs #379–#381)** — carrying-capacity population model (water/arable/trade factors), canon tiers to 15k+ cities, hinterland grain imports + famine on road failure, 160×80 sectors with half-hour walking, CANON_CITIES wired into caravans/rumors, 72-tile city sprawl. SCALE.md binding.
- **Walkable towns (#372, PRs #373–#377)** — town streets/houses on the one map, walk-in doors (tavern serves on step-in), street life, gate on the map (menu retired), 80×40 sectors.
- **Lore epics (#363–#367)** — household children + blood-before-friendship, marriage, wild species, settlement footprints, infrastructure II (well, waymarker, palisade), elder-esteem balance fix (#371).
- **Building arc (#343–#358)** — player farming, stash + residency, homestead→hamlet growth, world re-population + land-taking, structure world-effects (hearth, waystation, shrine), infrastructure tier (trail, footbridge), penance-as-restitution tone fix.
- **Living world (#312–#341)** — settlement food economy with real farms, NPC construction, profession depth; real Tool/Bandage/Trap items, disease severity, discovery effects; NPC lifecycle (births, aging, deaths, inheritance); weather fronts; festivals + rumors; growth/decline (promotions, famine, ghost towns); crime witnesses + inter-people escalation; bonds, grief, chosen heirs; voice banks +80%; canon naming baseline; recorded-choice AI-play API.
- **Refactor (#382)** — ui/app.rs (5,669 lines) split into 11 per-domain modules; public API unchanged.

## v0.5.x

- v0.5.0 — roguelike playability: movement, fog, quit safety, wait key (#240–#245)
- v0.5.1 — glyph collision fix, weather indicator, expanded footer (#247, #248)
- v0.5.2 — settlement matching uses actual terrain positions (#249)

## v0.4.0

- World screen IS the roguelike map — @ walks terrain with hjkl (#239)

## v0.3.x

- v0.3.0 — campaign arc milestones (#213), trade routes + merchant caravans (#214), settlement politics (#215), crafting quality tiers + tool degradation (#216), companion AI depth (#217), fog of war (#218), perf profiling (#219)
- v0.3.1 — fmt fix, logo, README run instructions, CI workflow, audio events wired into gameplay (#201)
- v0.3.2 — charts.ron embedded via include_str!; no external data files

## v0.2.3

- **#222 Death scene enrichment** — DeathCause enum (Starvation, Exposure, Exhaustion, Wounds, Unknown) with cause-specific flavor text. Elder death ceremony header. Memorial stats now show settlements visited and quests completed. `settlements_visited`/`quests_completed` counters in MilestoneTracker.
- **#223 Encounter variety** — 4 rare encounter types (GodShrine, AncientRuin, HermitCamp, TravelingBard) at 3% base chance. 3 seasonal encounters (SpringBloom, HarvestMarket, WinterSurvivor). Settlements excluded from random encounters. `Encounter::roll` now takes `day` param for season.

## v0.2.2

- **#211 Audio feature-gate stub** — Split `audio` (hound, pure Rust WAV synthesis) from `audio-playback` (rodio, needs ALSA). `cargo build --features audio` works without ALSA.
- **#220 Accessibility audit** — Stance/reputation labels now show symbol+text (`++ ally`, `~ neutral`, `-- hostile`). Focus cursor uses `▸`. Help screen documents accessibility. Color never sole signal.
- **#221 UI animations** — Pulse on low vitals (<30%), flash border on encounter. `reduced_motion` setting + [p] toggle. `tick_count`/`flash_frames` in App, `pre_draw()` per-frame.
- **#212 Save migration tests** — v1 migration test, v2 RON roundtrip, data preservation test, migrate_v2_to_v3 stub. Save versioning policy documented in ARCHITECTURE.md.
- **#224 Config file support** — `~/.config/deep-world-tui/config.toml` (TOML) for display options (monochrome, high_contrast, reduced_motion). Zero-config: missing file = defaults. Invalid = fallback. OR-merged with settings.ron.

## v0.2.1

- High-contrast white-on-black theme + [h] toggle
- Balance harness v2 (3 tests, no SimState dependency)
- Remove dead code (`App.first_run`)
- Cargo.toml cleanup: reqwest default-features=false + rustls-tls; `PeopleKind::name()` endonyms
- Determinism audit (5 tests)