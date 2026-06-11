// Structure world-effects (#347): the kota hearth cooks better than a
// traveler's pot, a longhouse by the road quietly restarts one node of the
// dying waystation network, and a player-raised shrine is devotional
// practice — a slow, small pull, never a summons.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{craft_recipes, GodName, ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::ui::app::App;

fn app_on(terrain_want: Terrain) -> Option<App> {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
    let mut found = None;
    'o: for y in 0..terr.height {
        for x in 0..terr.width {
            if terr.get(x, y) == Some(terrain_want) {
                found = Some((x, y));
                break 'o;
            }
        }
    }
    let (px, py) = found?;
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });
    a.clock.hour = 10;
    Some(a)
}

fn put_structure(a: &mut App, kind: BuildKind, x: u32, y: u32, name: Option<String>) {
    a.sim.as_mut().unwrap().world.regions[0]
        .structures
        .push(Structure {
            kind,
            region_idx: 0,
            x,
            y,
            built_tick: 0,
            last_maintenance_tick: 0,
            name,
            is_npc_built: false,
            stash: Default::default(),
        });
}

fn meal_recipe_idx() -> usize {
    craft_recipes()
        .iter()
        .filter(|r| r.people.is_none())
        .position(|r| r.name == "Meal")
        .expect("Meal recipe")
}

fn food(a: &App) -> u32 {
    a.player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food)
}

#[test]
fn the_hearth_cooks_better_than_a_travelers_pot() {
    // Same craft, same materials — the only difference is whose fire.
    let mut road = app_on(Terrain::Grass).expect("grass");
    let mut hearth = app_on(Terrain::Grass).expect("grass");
    let pos = hearth.player_pos.unwrap();
    put_structure(
        &mut hearth,
        BuildKind::Kota,
        pos.px as u32,
        pos.py as u32,
        None,
    );
    for a in [&mut road, &mut hearth] {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Herb, 2);
        let have = ps.inventory.get(ItemType::Food);
        if have == 0 {
            ps.inventory.add(ItemType::Food, 1);
        }
    }
    let idx = meal_recipe_idx();
    let (r0, h0) = (food(&road), food(&hearth));
    road.craft_recipe(idx);
    hearth.craft_recipe(idx);
    let road_gain = food(&road) as i64 - r0 as i64;
    let hearth_gain = food(&hearth) as i64 - h0 as i64;
    assert_eq!(
        hearth_gain,
        road_gain + 1,
        "a real fire beats a traveler's pot by one"
    );
}

#[test]
fn a_shrine_is_raised_to_a_god_and_pulls_slowly() {
    let mut a = app_on(Terrain::Grass).expect("grass");
    a.god_affinity.adjust(GodName::Keuru, 0.5); // the god the player carries
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Stone, 6);
        ps.inventory.add(ItemType::Cloth, 3);
        ps.inventory.add(ItemType::Food, 40);
        ps.inventory.add(ItemType::Water, 40);
    }
    a.start_build_kind(Some(BuildKind::Shrine));
    assert!(
        !a.sim.as_ref().unwrap().build_sites.is_empty(),
        "shrine site opens: {:?}",
        a.status_msg
    );
    a.advance_clock(14); // 12h build, no labor needed — stone patience
    let shrine = a.sim.as_ref().unwrap().world.regions[0]
        .structures
        .iter()
        .find(|s| s.kind == BuildKind::Shrine && !s.is_npc_built)
        .cloned()
        .expect("shrine completes");
    assert_eq!(
        shrine.name.as_deref(),
        Some("Keuru"),
        "the shrine keeps the name of the god it was raised to"
    );
    // Rest beside it: a pull that is real but small — practice, not miracle.
    let before = a.god_affinity.get(GodName::Keuru);
    a.rest_hours(8);
    let delta = a.god_affinity.get(GodName::Keuru) - before;
    assert!(delta > 0.0, "rest at the shrine turns the heart a little");
    assert!(
        delta <= 0.02,
        "slow and small — no miracle sites (got {delta})"
    );
}

#[test]
fn a_longhouse_by_the_road_earns_quiet_standing() {
    let mut a = app_on(Terrain::Grass).expect("grass");
    // Find a road tile to neighbor.
    let road = {
        let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
        let mut found = None;
        'o: for y in 0..terr.height {
            for x in 0..terr.width {
                if terr.get(x, y) == Some(Terrain::Road) {
                    found = Some((x, y));
                    break 'o;
                }
            }
        }
        found
    };
    let Some((rx, ry)) = road else { return }; // roadless seed: vacuous
    put_structure(
        &mut a,
        BuildKind::Longhouse,
        rx as u32,
        (ry as u32).saturating_add(1),
        None,
    );
    let (player_id, settlement_id) = {
        let sim = a.sim.as_ref().unwrap();
        (
            a.player_start.as_ref().unwrap().person.id.clone(),
            sim.world.regions[0].settlements[0].id.clone(),
        )
    };
    let before = a
        .sim
        .as_ref()
        .unwrap()
        .reputation
        .get(&player_id, &settlement_id);
    a.tick_waystations();
    let after = a
        .sim
        .as_ref()
        .unwrap()
        .reputation
        .get(&player_id, &settlement_id);
    assert!(
        after > before,
        "sheltering travelers is remembered ({before} -> {after})"
    );
    assert!(
        after - before < 0.05,
        "a trickle, not a flood ({})",
        after - before
    );
}
