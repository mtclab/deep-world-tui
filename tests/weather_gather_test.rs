// Weather affects player gathering: harsh skies thin the harvest. Reads the
// regional weather-front state (set directly here; fronts move it in play).
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
    let tile = {
        let sim = app.sim.as_ref().unwrap();
        let terr = &sim.world.regions[0].terrain;
        let mut found = None;
        'scan: for y in 0..terr.height {
            for x in 0..terr.width {
                if let Some(t) = terr.get(x, y) {
                    if let Some(item) = ItemType::gather_from(t) {
                        found = Some((x, y, item));
                        break 'scan;
                    }
                }
            }
        }
        found.expect("a gatherable tile exists")
    };
    let (px, py, item) = tile;
    app.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });

    let gather_yield = |app: &mut App, w: Weather| -> u32 {
        app.sim.as_mut().unwrap().world.regions[0].weather = w;
        let before = app.player_start.as_ref().unwrap().inventory.get(item);
        app.gather();
        app.player_start.as_ref().unwrap().inventory.get(item) - before
    };

    let clear_yield = gather_yield(&mut app, Weather::Clear);
    let harsh_yield = gather_yield(&mut app, Weather::Whiteout);

    assert!(
        harsh_yield < clear_yield,
        "harsh weather (yield {harsh_yield}) should gather less than clear (yield {clear_yield})"
    );
}
