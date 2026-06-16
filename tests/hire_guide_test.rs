// Hire a guide to learn the country (#527 tradespeople): a path-finder/forester
// opens the whole region on the map for a fee.
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

fn find_a_guide(a: &mut App) -> Option<(usize, usize, usize)> {
    let regions = a.sim.as_ref().unwrap().world.regions.len();
    for ri in 0..regions {
        let found = {
            let region = &a.sim.as_ref().unwrap().world.regions[ri];
            region.settlements.iter().enumerate().find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.profession == "path-finder" || p.profession == "forester")
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
fn a_guide_opens_the_region_for_a_fee() {
    let mut a = app(42);
    let mut deal = find_a_guide(&mut a);
    for seed in [7u64, 100, 555, 2024, 1, 9, 13, 21, 33, 77] {
        if deal.is_some() {
            break;
        }
        a = app(seed);
        deal = find_a_guide(&mut a);
    }
    let Some((ri, si, pi)) = deal else {
        panic!("no path-finder/forester found across seeds");
    };
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 20);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);

    // A far corner of the region is unexplored before hiring.
    let (w, h) = {
        let t = &a.sim.as_ref().unwrap().world.regions[ri].terrain;
        (t.width, t.height)
    };
    let (fx, fy) = (w - 1, h - 1);
    let before = a.explored[ri].is_explored(fx, fy);

    a.hire_guide(ri, si, pi);

    assert!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin)
            < coin_before,
        "the fee is paid"
    );
    assert!(
        a.explored[ri].is_explored(fx, fy),
        "the whole region is opened (corner {fx},{fy} now explored; was {before})"
    );
}

#[test]
fn only_a_path_walker_guides_you() {
    let mut a = app(42);
    // A non-guide person at settlement 0.
    let (ri, si, pi) = {
        let region = &a.sim.as_ref().unwrap().world.regions[0];
        let (si, pi) = region
            .settlements
            .iter()
            .enumerate()
            .find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.profession != "path-finder" && p.profession != "forester")
                    .map(|pi| (si, pi))
            })
            .expect("a non-guide");
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
        .add(ItemType::Coin, 20);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    a.hire_guide(ri, si, pi);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin),
        coin_before,
        "a non-guide takes no fee"
    );
}
