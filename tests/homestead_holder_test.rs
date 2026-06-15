// A rural holding's hospitality (#458): knock at a country holding's door and,
// at the right hour, a holder is in — bread and water for the road and the news
// of the valley; otherwise the folk are out in the fields. No menu, no roster.
use deep_world_tui::charts::load::load_charts;
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

/// A homestead door (a `Door` belonging to no settlement) and an adjacent
/// walkable tile to stand on. Searches regions for one.
fn a_holding_door(a: &App) -> Option<(usize, usize, usize, usize, usize)> {
    for ri in 0..a.sim.as_ref().unwrap().world.regions.len() {
        let r = &a.sim.as_ref().unwrap().world.regions[ri];
        for y in 1..r.terrain.height - 1 {
            for x in 1..r.terrain.width - 1 {
                if r.terrain.get(x, y) == Some(Terrain::Door)
                    && !r.settlements.iter().any(|s| s.contains_tile(x, y))
                {
                    for (ox, oy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                        let (nx, ny) = ((x as i32 + ox) as usize, (y as i32 + oy) as usize);
                        if matches!(
                            r.terrain.get(nx, ny),
                            Some(Terrain::Settlement | Terrain::Farmland | Terrain::Grass)
                        ) {
                            return Some((ri, x, y, nx, ny));
                        }
                    }
                }
            }
        }
    }
    None
}

#[test]
fn a_holding_sometimes_takes_you_in() {
    let mut a = app();
    let (ri, dx, dy, nx, ny) = match a_holding_door(&a) {
        Some(t) => t,
        None => return, // a homestead-less seed: nothing to test
    };
    let mut welcomed = 0;
    let mut empty = 0;
    let mut gained_food_once = false;
    for day in 0..24u32 {
        a.clock.day = day;
        let food_before = a
            .player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Food);
        a.player_pos = Some(PlayerPos {
            region_idx: ri,
            px: nx,
            py: ny,
        });
        let (mvx, mvy) = (dx as i32 - nx as i32, dy as i32 - ny as i32);
        a.move_player(mvx, mvy);
        let msg = a.status_msg.clone().unwrap_or_default();
        if msg.contains("waves you in") {
            welcomed += 1;
            if a.player_start
                .as_ref()
                .unwrap()
                .inventory
                .get(ItemType::Food)
                > food_before
            {
                gained_food_once = true;
            }
        } else if msg.contains("out in the fields") {
            empty += 1;
        }
    }
    assert!(welcomed > 0, "some days a holder is home");
    assert!(empty > 0, "some days the folk are out");
    assert!(gained_food_once, "a welcome gives bread for the road");
}
