// Regression: weather now affects player gathering (gather_modifier), not just
// NPC farm growth. Harsh weather thins the harvest.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::weather::Weather;
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

#[test]
fn harsh_weather_reduces_player_gather_yield() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app.clock.hour = 12; // daylight so gather isn't blocked

    // Find a gatherable tile in region 0 and stand on it.
    let (seed, tile) = {
        let sim = app.sim.as_ref().unwrap();
        let seed = sim.world.seed;
        let terr = &sim.world.regions[0].terrain;
        let mut found = None;
        'scan: for y in 0..terr.height {
            for x in 0..terr.width {
                if let Some(t) = terr.get(x, y) {
                    if let Some(item) = ItemType::gather_from(t) {
                        found = Some((x, y, t, item));
                        break 'scan;
                    }
                }
            }
        }
        (seed, found.expect("a gatherable tile exists"))
    };
    let (px, py, terrain, item) = tile;
    app.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });

    // Find a clear-weather tick and a harsh-weather tick at this tile.
    let mut clear_tick = None;
    let mut harsh_tick = None;
    for tick in 0..4000u64 {
        let m = Weather::generate(seed, tick, terrain).gather_modifier();
        if clear_tick.is_none() && m >= 0.99 {
            clear_tick = Some(tick);
        }
        if harsh_tick.is_none() && m <= 0.4 {
            harsh_tick = Some(tick);
        }
        if clear_tick.is_some() && harsh_tick.is_some() {
            break;
        }
    }
    let clear_tick = clear_tick.expect("a clear-weather tick");
    let harsh_tick = harsh_tick.expect("a harsh-weather tick");

    let gather_yield = |app: &mut App, tick: u64| -> u32 {
        app.sim.as_mut().unwrap().world.tick = tick;
        let before = app.player_start.as_ref().unwrap().inventory.get(item);
        app.gather();
        app.player_start.as_ref().unwrap().inventory.get(item) - before
    };

    let clear_yield = gather_yield(&mut app, clear_tick);
    let harsh_yield = gather_yield(&mut app, harsh_tick);

    assert!(
        harsh_yield < clear_yield,
        "harsh weather (yield {harsh_yield}) should gather less than clear (yield {clear_yield})"
    );
}
