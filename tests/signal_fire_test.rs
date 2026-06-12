// Infrastructure III: a signal fire is seen from far, and the lit dark stays
// quieter near it — while someone keeps it fed, because it rots like the rest.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::ui::app::App;

fn app_on_grass(seed: u64) -> (App, usize, usize) {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
    let mut found = None;
    'o: for y in 0..terr.height {
        for x in 0..terr.width {
            if terr.get(x, y) == Some(Terrain::Grass) {
                found = Some((x, y));
                break 'o;
            }
        }
    }
    let (px, py) = found.expect("grass");
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });
    (a, px, py)
}

fn put(a: &mut App, kind: BuildKind, x: u32, y: u32) {
    let tick = a.sim.as_ref().unwrap().world.tick;
    a.sim.as_mut().unwrap().world.regions[0]
        .structures
        .push(Structure {
            kind,
            region_idx: 0,
            x,
            y,
            built_tick: 0,
            last_maintenance_tick: tick,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
}

#[test]
fn the_fire_is_seen_from_far() {
    let (mut dark, px, py) = app_on_grass(42);
    let (mut lit, _, _) = app_on_grass(42);
    let terr_w = lit.sim.as_ref().unwrap().world.regions[0].terrain.width;
    let terr_h = lit.sim.as_ref().unwrap().world.regions[0].terrain.height;
    // A beacon far across the region — well outside any reveal the walker
    // earns on their own step.
    let fx = (terr_w - 5).max(px + 20) as u32;
    let fy = (terr_h - 5).max(py) as u32;
    if fx as usize >= terr_w || fy as usize >= terr_h {
        return; // tiny-map seed: vacuous
    }
    put(&mut lit, BuildKind::Beacon, fx, fy);
    for a in [&mut dark, &mut lit] {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Food, 20);
        ps.inventory.add(ItemType::Water, 20);
        a.clock.hour = 10;
    }
    dark.move_player(1, 0);
    lit.move_player(1, 0);
    assert!(
        lit.explored[0].is_explored(fx as usize, fy as usize),
        "the fire's own ground is known from across the region"
    );
    assert!(
        !dark.explored[0].is_explored(fx as usize, fy as usize),
        "without the fire that ground stays dark"
    );
}

#[test]
fn the_lit_dark_is_quieter_near_the_fire() {
    // Deterministic sweep: across many night-hours and seeds, the walker
    // beside their own beacon meets fewer encounters than the one without.
    let mut plain_count = 0u32;
    let mut lit_count = 0u32;
    for seed in [42u64, 555, 777, 1009, 31337] {
        let (mut plain, px, py) = app_on_grass(seed);
        let (mut lit, _, _) = app_on_grass(seed);
        put(&mut lit, BuildKind::Beacon, px as u32, py as u32);
        for a in [&mut plain, &mut lit] {
            let ps = a.player_start.as_mut().unwrap();
            ps.inventory.add(ItemType::Food, 50);
            ps.inventory.add(ItemType::Water, 50);
        }
        for day in 1..30u32 {
            for hour in [20u32, 22, 23] {
                for a in [&mut plain, &mut lit] {
                    a.clock.day = day;
                    a.clock.hour = hour;
                    a.encounter = None;
                    a.check_encounter(Terrain::Grass);
                }
                if plain.encounter.is_some() {
                    plain_count += 1;
                }
                if lit.encounter.is_some() {
                    lit_count += 1;
                }
            }
        }
    }
    assert!(
        lit_count < plain_count,
        "the fire holds the dark off ({lit_count} lit vs {plain_count} plain)"
    );
}

#[test]
fn the_fire_rots_like_everything() {
    assert!(
        BuildKind::Beacon.decay_years().is_some(),
        "an unfed fire-stack slumps — the lesson never stops applying"
    );
    // And it is not shelter: a watch-fire is not a roof.
    assert_eq!(BuildKind::Beacon.rest_quality(), "beacon");
}
