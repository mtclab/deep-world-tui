// Luck against the weather. Harsh skies wear the body down faster — and the
// hidden star leans how hard. A cursed soul bears the cold and heat worse than
// a blessed one; clear weather falls on everyone the same. Only the penalty
// over fair weather is leaned, never the fair baseline.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{Fortune, PlayerPos, Weather};
use deep_world_tui::ui::app::App;

fn app(seed: u64, fortune: f64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 3,
        py: 3,
    });
    a.fortune = Fortune::from_value(fortune);
    a
}

// Thirst lost over `hours` under a fixed sky, with no drink to mask it.
fn thirst_lost(fortune: f64, weather: Weather, hours: u32) -> f64 {
    let mut a = app(7, fortune);
    for r in a.sim.as_mut().unwrap().world.regions.iter_mut() {
        r.weather = weather;
    }
    // Strip water so auto-drink cannot hide the decay; full vitals to start.
    if let Some(ps) = a.player_start.as_mut() {
        let w = ps.inventory.get(deep_world_tui::model::ItemType::Water);
        ps.inventory
            .remove(deep_world_tui::model::ItemType::Water, w);
    }
    a.vitals.thirst = 1.0;
    a.vitals.hunger = 1.0;
    a.vitals.energy = 1.0;
    a.clock.hour = 1; // a short step, no day rollover
    a.advance_clock(hours);
    1.0 - a.vitals.thirst
}

#[test]
fn the_cursed_bear_the_whiteout_worse() {
    let blessed = thirst_lost(1.0, Weather::Whiteout, 6);
    let cursed = thirst_lost(-1.0, Weather::Whiteout, 6);
    assert!(blessed > 0.0 && cursed > 0.0, "the cold thirsts everyone");
    assert!(
        cursed > blessed,
        "the cursed wear faster in the whiteout ({cursed:.4} vs {blessed:.4})"
    );
}

#[test]
fn clear_skies_fall_on_everyone_the_same() {
    let blessed = thirst_lost(1.0, Weather::Clear, 6);
    let cursed = thirst_lost(-1.0, Weather::Clear, 6);
    assert!(
        (blessed - cursed).abs() < 1e-9,
        "fair weather is no respecter of stars ({blessed:.4} vs {cursed:.4})"
    );
}

#[test]
fn the_star_only_leans_the_harshness_not_the_baseline() {
    // The blessed still wear in a whiteout — fortune softens the penalty, it
    // does not grant immunity — and they wear more there than under clear sky.
    let blessed_clear = thirst_lost(1.0, Weather::Clear, 6);
    let blessed_storm = thirst_lost(1.0, Weather::Whiteout, 6);
    assert!(
        blessed_storm > blessed_clear,
        "even the blessed feel the whiteout ({blessed_storm:.4} vs clear {blessed_clear:.4})"
    );
}
