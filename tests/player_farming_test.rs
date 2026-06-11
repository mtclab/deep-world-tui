// Player farming (#343): fields need a homestead, seed costs Food (nothing
// from nothing — NPC farms pay from the stores too), the forest watches you
// clear its edge, crops grow with the days, and the frost takes what stands.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{GodName, ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::sim::SimState;
use deep_world_tui::ui::app::App;

fn app_on_grass() -> App {
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
    a.clock.hour = 10;
    a
}

fn give_cabin_here(a: &mut App) {
    let pos = a.player_pos.unwrap();
    a.sim.as_mut().unwrap().world.regions[pos.region_idx]
        .structures
        .push(Structure {
            kind: BuildKind::Cabin,
            region_idx: pos.region_idx,
            x: pos.px as u32,
            y: pos.py as u32,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
        });
}

#[test]
fn fields_need_a_homestead_and_seed() {
    let mut a = app_on_grass();
    a.plant();
    assert!(
        a.status_msg
            .clone()
            .unwrap_or_default()
            .contains("homestead"),
        "no cabin, no farm"
    );
    give_cabin_here(&mut a);
    {
        let ps = a.player_start.as_mut().unwrap();
        let f = ps.inventory.get(ItemType::Food);
        ps.inventory.remove(ItemType::Food, f);
    }
    a.plant();
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("seed"),
        "no seed, no planting"
    );
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 2);
    a.plant();
    assert_eq!(a.player_farms.len(), 1, "planted");
    // Cabin allows ONE plot.
    a.player_pos = a.player_pos.map(|mut p| {
        p.px += 1;
        p
    });
    a.plant();
    assert_eq!(a.player_farms.len(), 1, "cabin works one field only");
}

#[test]
fn crops_grow_and_harvest_pays() {
    let mut a = app_on_grass();
    give_cabin_here(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 30);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Water, 30);
    a.plant();
    assert_eq!(a.player_farms.len(), 1);
    let food_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    // Grow it: rest day-cycles until ready (well under one season).
    for _ in 0..16 {
        a.rest_hours(12);
        if a.player_farms.first().map(|f| f.farm.is_ready()) == Some(true) {
            break;
        }
    }
    assert_eq!(
        a.player_farms.first().map(|f| f.farm.is_ready()),
        Some(true),
        "the crop should ripen within days"
    );
    a.harvest();
    let food_after = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    assert!(
        food_after > food_before,
        "harvest should out-pay the days ({food_before} -> {food_after})"
    );
    assert!(a.player_farms.is_empty(), "the field wants seed again");
}

#[test]
fn the_forest_watches_the_clearing() {
    let mut a = app_on_grass();
    // Find grass WITH adjacent forest.
    let spot = {
        let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
        let mut found = None;
        'o: for y in 1..terr.height - 1 {
            for x in 1..terr.width - 1 {
                if terr.get(x, y) == Some(Terrain::Grass) {
                    let near_forest = (-1i32..=1).any(|dy| {
                        (-1i32..=1).any(|dx| {
                            terr.get((x as i32 + dx) as usize, (y as i32 + dy) as usize)
                                == Some(Terrain::Forest)
                        })
                    });
                    if near_forest {
                        found = Some((x, y));
                        break 'o;
                    }
                }
            }
        }
        found
    };
    let Some((px, py)) = spot else { return }; // no such tile on this seed: vacuous
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });
    give_cabin_here(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 2);
    let keuru_before = a.god_affinity.get(GodName::Keuru);
    a.plant();
    assert!(
        a.god_affinity.get(GodName::Keuru) < keuru_before,
        "clearing against the wood costs Keuru's regard"
    );
}

#[test]
fn npc_farms_pay_seed_from_the_stores() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    {
        let s = &mut sim.world.regions[0].settlements[0];
        s.food_stock = 0.5; // not enough for seed
        for p in s.people.iter_mut().take(3) {
            p.profession = "farmer".into();
        }
        s.farms.clear();
    }
    sim.world.tick = 23;
    sim.step(); // day boundary
    assert!(
        sim.world.regions[0].settlements[0].farms.is_empty(),
        "an empty store cannot seed a field"
    );
}
