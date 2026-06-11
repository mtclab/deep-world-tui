// Structures weather and fall to ruin over time (on the compressed aging scale);
// a sturdy home outlasts a flimsy tarp-tent.

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::sim::SimState;

fn put(sim: &mut SimState, kind: BuildKind) {
    let s = Structure {
        kind,
        region_idx: 0,
        x: 1,
        y: 1,
        built_tick: 0,
        last_maintenance_tick: 0,
        name: None,
        is_npc_built: false,
        stash: Default::default(),
    };
    sim.world.regions[0].structures.push(s.clone());
    sim.structures.push(s);
}

fn count(sim: &SimState, kind: BuildKind) -> usize {
    sim.world.regions[0]
        .structures
        .iter()
        .filter(|s| s.kind == kind)
        .count()
}

#[test]
fn flimsy_structure_decays_sturdy_survives() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    put(&mut sim, BuildKind::TarpTent); // 3y -> ~216 ticks
    put(&mut sim, BuildKind::Home); // 30y -> ~2160 ticks

    assert_eq!(count(&sim, BuildKind::TarpTent), 1);
    // Tick well past the tarp-tent's life but within the home's.
    for _ in 0..300 {
        sim.step();
    }
    assert_eq!(
        count(&sim, BuildKind::TarpTent),
        0,
        "tarp-tent should have weathered away"
    );
    assert_eq!(
        count(&sim, BuildKind::Home),
        1,
        "a home should still stand after 300 ticks"
    );
}
