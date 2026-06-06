# deep-world-tui

A terminal **procedural life-RPG** in the Deep World (Rust + ratatui). Nothing is
hand-placed: the world, its people, and *you* are sampled from lore-grounded
weighted charts. Live an organic life — a people, a town, a family, a trade — or
walk away from it to wander, and live with what that costs. Same seed → same world.

Siblings: [`deep-world-archive`](https://github.com/mtclab/deep-world-archive) (web
CRPG), [`deep-world-godot`](https://github.com/mtclab/deep-world-godot) (open-world).
Source world: [`deep-world-history`](https://github.com/mtclab/deep-world-history).

## Status

**Design + handoff stage.** No code yet — the design is specced and the build is
broken into issues for implementation. See `docs/` and the issues.

- `docs/DESIGN.md` — vision + pillars
- `docs/GENERATION.md` — the possibility-chart engine (the heart)
- `docs/CONSEQUENCES.md` — freedom + consequence (the spine)
- `docs/ARCHITECTURE.md` — Rust crate layout + build order + tests
- `data/charts.ron` — starter lore-grounded charts
- `AGENTS.md` — build/verify/canon rules for contributors

## Build (once scaffolded — see issue #1)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && . "$HOME/.cargo/env"
cargo run -- --seed 1234        # generate + play a world
cargo test                      # determinism + distribution + sim tests
cargo run --no-default-features  # no LLM (default play is LLM-off anyway)
```

## License

MIT — see [LICENSE](LICENSE).
