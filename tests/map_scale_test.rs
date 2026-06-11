// One map, walkable towns (#372, PR 1): sectors are 80x40 now. Old saves
// carried 40x20 — they upscale 2x (each tile becomes a 2x2 block, the world
// keeps its exact shape) and every coordinate that lives on the map follows.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::world::upscale_world_2x;
use deep_world_tui::model::{Terrain, TerrainMap};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::sim::SimState;

#[test]
fn new_worlds_are_born_at_full_size() {
    let charts = load_charts().expect("charts");
    let sim = SimState::new(42, charts);
    for region in &sim.world.regions {
        assert_eq!(region.terrain.width, 80);
        assert_eq!(region.terrain.height, 40);
    }
}

#[test]
fn old_saves_upscale_cleanly_and_only_once() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    // Forge an old-save region: 40x20, one settlement tile, one cabin.
    {
        let r = &mut sim.world.regions[0];
        let mut tiles = vec![Terrain::Grass; 40 * 20];
        tiles[5 * 40 + 7] = Terrain::Settlement;
        r.terrain = TerrainMap {
            width: 40,
            height: 20,
            tiles,
        };
        r.settlements.truncate(1);
        r.settlements[0].map_x = 7;
        r.settlements[0].map_y = 5;
        r.settlements[0].size = "hamlet".into();
        r.structures.clear();
        r.structures.push(Structure {
            kind: BuildKind::Cabin,
            region_idx: 0,
            x: 9,
            y: 5,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
    }
    assert!(upscale_world_2x(&mut sim), "an old sector gets scaled");
    let r = &sim.world.regions[0];
    assert_eq!(r.terrain.width, 80);
    assert_eq!(r.terrain.height, 40);
    // The settlement tile became a 2x2 block at doubled coordinates.
    for (x, y) in [(14, 10), (15, 10), (14, 11), (15, 11)] {
        assert_eq!(
            r.terrain.get(x, y),
            Some(Terrain::Settlement),
            "tile ({x},{y}) keeps the town"
        );
    }
    assert_eq!((r.settlements[0].map_x, r.settlements[0].map_y), (14, 10));
    assert_eq!((r.structures[0].x, r.structures[0].y), (18, 10));
    // Idempotent: a second pass does nothing.
    assert!(
        !upscale_world_2x(&mut sim),
        "already at full size — no double scaling"
    );
    assert_eq!(sim.world.regions[0].structures[0].x, 18);
}

#[test]
fn travel_stays_honest_on_the_finer_grid() {
    // Tiles are half the ground: open land walks in 1h, hard land in 2h —
    // crossing a region costs the same days it did at the old scale.
    assert_eq!(Terrain::Grass.travel_hours(), 1);
    assert_eq!(Terrain::Forest.travel_hours(), 2);
    assert_eq!(Terrain::Road.travel_hours(), 1);
}
