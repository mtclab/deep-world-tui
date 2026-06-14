// Walk-in interactions (#458): no menus — the tavern serves when you step
// through its door, a plain home answers the knock, and the town is wherever
// you stand. Buildings are real walled structures with a single doorway.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::town::town_buildings;
use deep_world_tui::model::{ItemType, PlayerPos, Terrain};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 10;
    a
}

/// The doorway of settlement 0's k-th building, and the walkable street tile
/// adjacent to it (where the player stands to walk in). Returns
/// (street_x, street_y, door_x, door_y).
fn door_and_street(a: &App, region: usize, si: usize, k: usize) -> Option<(usize, usize, usize, usize)> {
    let region_ref = &a.sim.as_ref().unwrap().world.regions[region];
    let s = &region_ref.settlements[si];
    let b = town_buildings(s).into_iter().nth(k)?;
    let (dx, dy) = b.door;
    for (ox, oy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
        let (nx, ny) = (dx as i32 + ox, dy as i32 + oy);
        if nx < 0 || ny < 0 {
            continue;
        }
        if matches!(
            region_ref.terrain.get(nx as usize, ny as usize),
            Some(Terrain::Settlement | Terrain::Farmland)
        ) {
            return Some((nx as usize, ny as usize, dx, dy));
        }
    }
    None
}

#[test]
fn the_town_is_wherever_you_stand() {
    let mut a = app();
    let (sx, sy, _, _) = door_and_street(&a, 0, 0, 0).expect("a building with a reachable door");
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: sx,
        py: sy,
    });
    assert!(
        a.current_settlement().is_some(),
        "streets resolve to their town without any menu"
    );
}

#[test]
fn the_first_door_serves_its_service() {
    let mut a = app();
    let first_service = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .services
        .first()
        .copied();
    let Some(svc) = first_service else { return };
    let (sx, sy, dx, dy) = door_and_street(&a, 0, 0, 0).expect("a building with a reachable door");
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: sx,
        py: sy,
    });
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 30);
    // Step from the street into the doorway.
    let (mvx, mvy) = (dx as i32 - sx as i32, dy as i32 - sy as i32);
    let before = a.player_pos.unwrap();
    a.move_player(mvx, mvy);
    let after = a.player_pos.unwrap();
    assert_eq!(
        (before.px, before.py),
        (after.px, after.py),
        "the walker stays on the doorstep — roofs stay solid"
    );
    let msg = a.status_msg.clone().unwrap_or_default().to_lowercase();
    assert!(
        msg.contains(&svc.label().to_lowercase())
            || msg.contains("rested")
            || msg.contains("blessed")
            || msg.contains("closed")
            || msg.contains("shut")
            || msg.contains("knock"),
        "stepping into the {} door uses it (or finds it honestly shut): {msg}",
        svc.label()
    );
}

#[test]
fn a_plain_home_answers_the_knock() {
    let mut a = app();
    // Find any settlement with a building past its services list (a home).
    let spot = {
        let sim = a.sim.as_ref().unwrap();
        let mut found = None;
        'o: for (ri, region) in sim.world.regions.iter().enumerate() {
            for (si, s) in region.settlements.iter().enumerate() {
                let n = town_buildings(s).len();
                if n > s.services.len() {
                    if let Some(spot) = door_and_street(&a, ri, si, s.services.len()) {
                        found = Some((ri, spot));
                        break 'o;
                    }
                }
            }
        }
        found
    };
    let Some((ri, (sx, sy, dx, dy))) = spot else {
        panic!("some town somewhere must have a plain home");
    };
    a.player_pos = Some(PlayerPos {
        region_idx: ri,
        px: sx,
        py: sy,
    });
    let (mvx, mvy) = (dx as i32 - sx as i32, dy as i32 - sy as i32);
    a.move_player(mvx, mvy);
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("knock"),
        "a home answers the knock, not a menu: {msg}"
    );
}
