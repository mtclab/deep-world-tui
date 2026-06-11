// Infrastructure (#348): scale honesty — a TRAIL and a FOOTBRIDGE, laid by
// one pair of hands (Karsath was god-era engineering). A trail cuts the walk
// on its tile; a footbridge makes its water crossable while it stands. And
// the non-negotiable: player infrastructure decays from day one — an
// untended footbridge is a private Velkarmoss.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
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

fn put(a: &mut App, kind: BuildKind, ri: usize, x: u32, y: u32) {
    let tick = a.sim.as_ref().unwrap().world.tick;
    a.sim.as_mut().unwrap().world.regions[ri]
        .structures
        .push(Structure {
            kind,
            region_idx: ri,
            x,
            y,
            built_tick: 0,
            last_maintenance_tick: tick,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
}

/// A slow tile whose east neighbor is also slow (forest: 2h ground on the
/// rebalanced grid — open grass already walks at the 1h floor, so only slow
/// ground can show a trail's cut). Searches every region.
fn slow_pair(a: &App) -> Option<(usize, usize, usize)> {
    for (ri, region) in a.sim.as_ref().unwrap().world.regions.iter().enumerate() {
        let terr = &region.terrain;
        for y in 0..terr.height {
            for x in 0..terr.width.saturating_sub(1) {
                if terr.get(x, y) == Some(Terrain::Forest)
                    && terr.get(x + 1, y) == Some(Terrain::Forest)
                {
                    return Some((ri, x, y));
                }
            }
        }
    }
    None
}

/// A water tile with a passable neighbor to stand on.
fn water_with_bank(a: &App) -> Option<((usize, usize), (usize, usize))> {
    let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
    for y in 1..terr.height.saturating_sub(1) {
        for x in 1..terr.width.saturating_sub(1) {
            if terr.get(x, y) != Some(Terrain::Water) {
                continue;
            }
            for (dx, dy) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
                let (bx, by) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                if terr.get(bx, by).map(|t| t.passable()) == Some(true) {
                    return Some(((x, y), (bx, by)));
                }
            }
        }
    }
    None
}

#[test]
fn a_trail_cuts_the_walk() {
    let mut plain = app();
    let mut trailed = app();
    let Some((ri, x, y)) = slow_pair(&plain) else {
        return; // forestless seed: vacuous
    };
    {
        let tick = trailed.sim.as_ref().unwrap().world.tick;
        trailed.sim.as_mut().unwrap().world.regions[ri]
            .structures
            .push(Structure {
                kind: BuildKind::Trail,
                region_idx: ri,
                x: (x + 1) as u32,
                y: y as u32,
                built_tick: 0,
                last_maintenance_tick: tick,
                name: None,
                is_npc_built: false,
                stash: Default::default(),
            });
    }
    for a in [&mut plain, &mut trailed] {
        // Pin the sky: this is about the path, not the weather.
        a.sim.as_mut().unwrap().world.regions[ri].weather = deep_world_tui::model::Weather::Clear;
        a.player_pos = Some(PlayerPos {
            region_idx: ri,
            px: x,
            py: y,
        });
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Food, 20);
        ps.inventory.add(ItemType::Water, 20);
    }
    let h0 = plain.clock.day * 24 + plain.clock.hour;
    plain.move_player(1, 0);
    let plain_hours = plain.clock.day * 24 + plain.clock.hour - h0;
    let h1 = trailed.clock.day * 24 + trailed.clock.hour;
    trailed.move_player(1, 0);
    let trail_hours = trailed.clock.day * 24 + trailed.clock.hour - h1;
    assert!(
        trail_hours <= plain_hours,
        "a laid trail never walks slower ({trail_hours} vs {plain_hours})"
    );
    assert!(
        trail_hours < plain_hours,
        "on slow ground the trail must actually cut the walk ({trail_hours} vs {plain_hours})"
    );
}

#[test]
fn a_footbridge_opens_the_water_and_rots_shut_again() {
    let mut a = app();
    let Some(((wx, wy), (bx, by))) = water_with_bank(&a) else {
        return; // waterless region on this seed: vacuous
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: bx,
        py: by,
    });
    // Water without a bridge refuses the step.
    let (dx, dy) = (wx as i32 - bx as i32, wy as i32 - by as i32);
    a.move_player(dx, dy);
    assert_eq!(
        a.player_pos.unwrap().px,
        bx,
        "open water must block the walker"
    );
    // The planks change that.
    put(&mut a, BuildKind::Footbridge, 0, wx as u32, wy as u32);
    a.move_player(dx, dy);
    assert_eq!(
        (a.player_pos.unwrap().px, a.player_pos.unwrap().py),
        (wx, wy),
        "a standing footbridge carries you over: {:?}",
        a.status_msg
    );
    // Step back to the bank, then let the bridge rot: decay from day one.
    a.move_player(-dx, -dy);
    {
        let sim = a.sim.as_mut().unwrap();
        // 5 decay-years at 3 days/year and 24 ticks/day = 360 ticks of neglect.
        sim.world.tick += 400;
        sim.step();
    }
    assert!(
        !a.sim.as_ref().unwrap().world.regions[0]
            .structures
            .iter()
            .any(|s| s.kind == BuildKind::Footbridge),
        "an untended footbridge is a private Velkarmoss — it falls"
    );
    a.move_player(dx, dy);
    assert_eq!(
        a.player_pos.unwrap().px,
        bx,
        "with the planks gone, the water is water again"
    );
}

#[test]
fn the_bridge_is_raised_from_the_bank_and_worked_from_it() {
    let mut a = app();
    let Some(((wx, wy), (bx, by))) = water_with_bank(&a) else {
        return;
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: bx,
        py: by,
    });
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Wood, 24);
        ps.inventory.add(ItemType::Nails, 12);
        ps.inventory.add(ItemType::Cordage, 4);
        ps.inventory.add(ItemType::Tool, 1);
        ps.inventory.add(ItemType::Food, 60);
        ps.inventory.add(ItemType::Water, 60);
    }
    a.start_build_kind(Some(BuildKind::Footbridge));
    let site = a
        .sim
        .as_ref()
        .unwrap()
        .build_sites
        .first()
        .cloned()
        .expect("bridge site opens on the water");
    assert_eq!(
        (site.x, site.y),
        (wx as u32, wy as u32),
        "the site sits on the water, not the bank"
    );
    // Worked from the bank: three day-shifts of labor finish 24h of planks.
    for _ in 0..3 {
        a.work_site();
        a.rest_hours(10);
    }
    a.advance_clock(1);
    assert!(
        a.sim.as_ref().unwrap().world.regions[0]
            .structures
            .iter()
            .any(|s| s.kind == BuildKind::Footbridge),
        "worked from the bank, the bridge finishes"
    );
}

#[test]
fn infrastructure_decays_from_day_one() {
    // The §8 non-negotiable, stated as type-level truth.
    assert!(
        BuildKind::Trail.decay_years().is_some(),
        "a trail must decay"
    );
    assert!(
        BuildKind::Footbridge.decay_years().is_some(),
        "a footbridge must decay"
    );
    // Humble scale: both are quick-fading work, not god-era stone.
    assert!(BuildKind::Trail.decay_years().unwrap() <= 5.0);
    assert!(BuildKind::Footbridge.decay_years().unwrap() <= 10.0);
}
