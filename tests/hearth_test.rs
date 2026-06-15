// The hearth (#458): every building has a fire at its heart, and resting by it
// is the warmest rest in the world — a roof, walls, and a fire — even in the
// deep of a cold night, where the open country gives only a shivering doze.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{PlayerPos, Terrain};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

/// First hearth tile found on region 0's map.
fn a_hearth(a: &App) -> Option<(usize, usize)> {
    let r = &a.sim.as_ref().unwrap().world.regions[0];
    for y in 0..r.terrain.height {
        for x in 0..r.terrain.width {
            if r.terrain.get(x, y) == Some(Terrain::Hearth) {
                return Some((x, y));
            }
        }
    }
    None
}

/// A wild Grass tile well clear of any settlement on region 0.
fn a_wild_tile(a: &App) -> Option<(usize, usize)> {
    let r = &a.sim.as_ref().unwrap().world.regions[0];
    for y in 0..r.terrain.height {
        for x in 0..r.terrain.width {
            if r.terrain.get(x, y) == Some(Terrain::Grass)
                && !r.settlements.iter().any(|s| {
                    let n = s.footprint() as usize + 6;
                    x + 6 >= s.map_x as usize
                        && x <= s.map_x as usize + n
                        && y + 6 >= s.map_y as usize
                        && y <= s.map_y as usize + n
                })
            {
                return Some((x, y));
            }
        }
    }
    None
}

#[test]
fn at_night_the_household_gathers_at_the_hearths() {
    let a = app();
    let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
    let buildings = deep_world_tui::gen::town::town_buildings(s);
    let hearths: std::collections::HashSet<(usize, usize)> = buildings
        .iter()
        .map(|b| (b.x + b.w / 2, b.y + b.h / 2))
        .collect();
    // Deep night: people are home and gathered by the fire — the warmest
    // tiles (the hearths) fill before any far corner.
    let night = deep_world_tui::gen::town::npc_street_positions(s, 5, 23);
    let on_hearth = night
        .iter()
        .filter(|&&(_, x, y)| hearths.contains(&(x, y)))
        .count();
    let expect = buildings.len().min(s.people.len());
    assert_eq!(
        on_hearth, expect,
        "every hearth that has someone to fill it is occupied at night"
    );
}

#[test]
fn the_town_has_hearths() {
    let a = app();
    assert!(
        a_hearth(&a).is_some(),
        "a settlement laid on the map has hearths in its buildings"
    );
}

#[test]
fn resting_by_the_hearth_is_warmer_than_the_cold_wild() {
    let (hx, hy) = a_hearth(&app()).expect("a hearth");
    let (wx, wy) = a_wild_tile(&app()).expect("open country");

    // Deep night, energy run down, the same eight-hour rest in each place.
    let mut warm = app();
    warm.clock.hour = 2;
    warm.vitals.energy = 0.1;
    warm.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: hx,
        py: hy,
    });
    warm.rest_hours(8);
    let warm_gain = warm.vitals.energy - 0.1;

    let mut cold = app();
    cold.clock.hour = 2;
    cold.vitals.energy = 0.1;
    cold.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: wx,
        py: wy,
    });
    cold.rest_hours(8);
    let cold_gain = cold.vitals.energy - 0.1;

    assert!(
        warm_gain > cold_gain,
        "the hearth ({warm_gain}) rests better than the cold open ground ({cold_gain})"
    );
}
