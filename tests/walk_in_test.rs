// Walk-in interactions (#372 PR 3): no menus — the tavern serves when you
// step through its door, a plain home answers the knock, and the town is
// wherever you stand.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::town::house_cells;
use deep_world_tui::model::{ItemType, PlayerPos};
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

/// Stand on the street just east of the k-th house of settlement 0.
fn stand_at_door(a: &mut App, k: usize) -> (usize, usize) {
    let (cells, _) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (
            house_cells(s.map_x as usize, s.map_y as usize, s.footprint() as usize),
            (),
        )
    };
    let (hx, hy) = cells[k];
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: hx + 1,
        py: hy,
    });
    (hx, hy)
}

#[test]
fn the_town_is_wherever_you_stand() {
    let mut a = app();
    stand_at_door(&mut a, 0);
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
    let (hx, hy) = stand_at_door(&mut a, 0);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 30);
    let before = a.player_pos.unwrap();
    a.move_player(-1, 0); // step into the doorway
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
            || msg.contains("shut"),
        "stepping into the {} door uses it (or finds it honestly shut): {msg} at ({hx},{hy})",
        svc.label()
    );
}

#[test]
fn a_plain_home_answers_the_knock() {
    let mut a = app();
    // Find any settlement with a roof past its services list (a home).
    let spot = {
        let sim = a.sim.as_ref().unwrap();
        let mut found = None;
        'o: for (ri, region) in sim.world.regions.iter().enumerate() {
            for s in &region.settlements {
                let cells = house_cells(s.map_x as usize, s.map_y as usize, s.footprint() as usize);
                if cells.len() > s.services.len() {
                    let (hx, hy) = cells[s.services.len()];
                    found = Some((ri, hx, hy));
                    break 'o;
                }
            }
        }
        found
    };
    let Some((ri, hx, hy)) = spot else {
        panic!("some town somewhere must have a plain home");
    };
    a.player_pos = Some(PlayerPos {
        region_idx: ri,
        px: hx + 1,
        py: hy,
    });
    a.move_player(-1, 0);
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("knock"),
        "a home answers the knock, not a menu: {msg}"
    );
}
