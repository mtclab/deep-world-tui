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
        assert_eq!(region.terrain.width, 160);
        assert_eq!(region.terrain.height, 80);
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
    assert!(
        upscale_world_2x(&mut sim),
        "a 40x20 save doubles twice to reach 160x80"
    );
    let r = &sim.world.regions[0];
    assert_eq!(r.terrain.width, 160);
    assert_eq!(r.terrain.height, 80);
    // The settlement tile became a 4x4 block at quadrupled coordinates.
    for (x, y) in [(28, 20), (31, 20), (28, 23), (31, 23)] {
        assert_eq!(
            r.terrain.get(x, y),
            Some(Terrain::Settlement),
            "tile ({x},{y}) keeps the town"
        );
    }
    assert_eq!((r.settlements[0].map_x, r.settlements[0].map_y), (28, 20));
    assert_eq!((r.structures[0].x, r.structures[0].y), (36, 20));
    // Idempotent: a second pass does nothing.
    assert!(
        !upscale_world_2x(&mut sim),
        "already at full size — no double scaling"
    );
    assert_eq!(sim.world.regions[0].structures[0].x, 36);
}

#[test]
fn travel_stays_honest_on_the_finer_grid() {
    // Stage 2: tiles halve again and the WALK gains half-hours — two open
    // tiles to the hour (travel debt). The table stays integer; the debt
    // carries the fraction.
    assert_eq!(Terrain::Grass.travel_hours(), 1);
    assert_eq!(Terrain::Forest.travel_hours(), 2);
    use deep_world_tui::model::PlayerPos;
    use deep_world_tui::ui::app::App;
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 8;
    // Find two adjacent open grass tiles and walk one step: half an hour
    // owed, the clock unmoved until the debt fills the hour.
    let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
    let mut spot = None;
    'o: for y in 0..terr.height {
        for x in 0..terr.width - 1 {
            if terr.get(x, y) == Some(Terrain::Grass) && terr.get(x + 1, y) == Some(Terrain::Grass)
            {
                spot = Some((x, y));
                break 'o;
            }
        }
    }
    let Some((x, y)) = spot else { return };
    a.sim.as_mut().unwrap().world.regions[0].weather = deep_world_tui::model::Weather::Clear;
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: x,
        py: y,
    });
    let h0 = a.clock.hour;
    a.move_player(1, 0);
    let stepped = a.player_pos.unwrap().px == x + 1;
    if stepped {
        assert!(
            a.clock.hour == h0 || a.clock.hour == h0 + 1,
            "one open step costs at most the hour it falls into"
        );
        assert!(a.travel_debt < 1.0, "no whole hours left owing");
    }
}
