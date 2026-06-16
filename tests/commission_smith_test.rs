// Commission a Tool from a settlement's smith (#527 tradespeople): bring Iron
// and the fee, the smith does the rest — no botch, the way the player's bench
// can fail.
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

/// Find a settlement with a smith across a few seeds; stand the player there and
/// return (region, settlement, person) indices.
fn find_a_smith(a: &mut App) -> Option<(usize, usize, usize)> {
    let regions = a.sim.as_ref().unwrap().world.regions.len();
    for ri in 0..regions {
        let found = {
            let region = &a.sim.as_ref().unwrap().world.regions[ri];
            region.settlements.iter().enumerate().find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.profession == "smith")
                    .map(|pi| (si, pi, s.map_x as usize, s.map_y as usize))
            })
        };
        if let Some((si, pi, mx, my)) = found {
            a.player_pos = Some(PlayerPos {
                region_idx: ri,
                px: mx,
                py: my,
            });
            return Some((ri, si, pi));
        }
    }
    None
}

#[test]
fn the_smith_strikes_a_tool_for_iron_and_coin() {
    let mut a = app(42);
    let mut deal = find_a_smith(&mut a);
    for seed in [7u64, 100, 555, 2024, 1, 9, 13] {
        if deal.is_some() {
            break;
        }
        a = app(seed);
        deal = find_a_smith(&mut a);
    }
    let Some((ri, si, pi)) = deal else {
        panic!("no smith found across seeds — settlements should keep one somewhere");
    };

    {
        let inv = &mut a.player_start.as_mut().unwrap().inventory;
        inv.add(ItemType::Iron, 1);
        inv.add(ItemType::Coin, 20);
    }
    let tools_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Tool);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);

    a.commission_smith(ri, si, pi);

    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(
        inv.get(ItemType::Tool),
        tools_before + 1,
        "a Tool is struck"
    );
    assert_eq!(inv.get(ItemType::Iron), 0, "the Iron is given to the work");
    assert!(inv.get(ItemType::Coin) < coin_before, "the fee is paid");
}

#[test]
fn no_iron_no_commission() {
    let mut a = app(42);
    let mut deal = find_a_smith(&mut a);
    for seed in [7u64, 100, 555, 2024, 1, 9, 13] {
        if deal.is_some() {
            break;
        }
        a = app(seed);
        deal = find_a_smith(&mut a);
    }
    let (ri, si, pi) = deal.expect("a smith");
    // Coin but no Iron.
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 20);
    a.status_msg = None;
    a.commission_smith(ri, si, pi);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(inv.get(ItemType::Tool), 0, "no Tool without Iron");
    assert!(
        a.status_msg.as_deref().unwrap_or("").contains("Iron"),
        "told the commission wants Iron"
    );
}

#[test]
fn a_non_smith_will_not_take_the_commission() {
    let mut a = app(42);
    // Find any non-smith person at a settlement.
    let (ri, si, pi) = {
        let region = &a.sim.as_ref().unwrap().world.regions[0];
        let (si, pi) = region
            .settlements
            .iter()
            .enumerate()
            .find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.profession != "smith")
                    .map(|pi| (si, pi))
            })
            .expect("a non-smith");
        (0usize, si, pi)
    };
    let (mx, my) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[si];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(PlayerPos {
        region_idx: ri,
        px: mx,
        py: my,
    });
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Iron, 1);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 20);
    a.commission_smith(ri, si, pi);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Tool),
        0,
        "only a smith strikes the Tool"
    );
}
