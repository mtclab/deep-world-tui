# Changelog

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