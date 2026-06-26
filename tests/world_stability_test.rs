// World stability guard (playtest finding, 2026-06-26). The entity-first epic
// made every inhabitant a real, aging soul; the NPC life-clock was still the
// player's compressed 3-days-per-year, so the whole province aged ~30 life-years
// a calendar year and emptied itself (8.5k -> <300 in a game-year). Fixed by
// aging the world's people at the world's own calendar pace
// (`TICKS_PER_LIFE_YEAR` = one calendar year), plus gentler economy drains and a
// rare-rather-than-instant flight from famine. This guards the regression: a
// materialised province must remain a living society across a year, not collapse.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

fn pop(sim: &SimState) -> usize {
    sim.world
        .regions
        .iter()
        .flat_map(|r| &r.settlements)
        .map(|s| s.people.len())
        .sum()
}

#[test]
fn a_materialised_province_does_not_collapse_over_a_year() {
    for seed in [42u64, 7, 999] {
        let mut sim = SimState::new_capped(seed, load_charts().expect("charts"), Some(150));
        let before = pop(&sim);
        // A full calendar year of the living world.
        for _ in 0..(365 * 24) {
            sim.step();
        }
        let after = pop(&sim);
        // The old fast-aging bug emptied a province to a few percent in a year;
        // a stable world holds the great bulk of its people (and often grows
        // toward its land's carrying capacity).
        assert!(
            after * 2 >= before,
            "the province collapsed over a year (seed {seed}: {before} -> {after})"
        );
    }
}
