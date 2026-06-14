// Biome herblore (#456): foraging for medicine is biome-true. The deep wood and
// the mire give physic freely; the cold heights, the sand, and the bare stone
// give almost nothing. The supply half of herbalism — range to where the herbs
// grow.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos, Season, Terrain};
use deep_world_tui::ui::app::App;

fn app_on(terrain: Terrain) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(7, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    // Daylight, growing season, clear sky — isolate the biome.
    a.clock.hour = 10;
    a.clock.day = 45; // Green
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    if let Some(ref mut sim) = a.sim {
        sim.world.regions[0].weather = deep_world_tui::model::Weather::Clear;
        sim.world.regions[0].terrain.set(5, 5, terrain);
    }
    a
}

fn herb(a: &App) -> u32 {
    a.player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Herb)
}

#[test]
fn the_deep_wood_gives_physic() {
    let mut a = app_on(Terrain::Forest);
    assert_eq!(a.clock.season(), Season::Green);
    let before = herb(&a);
    a.forage_herbs();
    assert!(
        herb(&a) > before,
        "the forest should yield medicinal herbs, got {} -> {}",
        before,
        herb(&a)
    );
}

#[test]
fn the_mire_gives_physic_too() {
    let mut a = app_on(Terrain::Swamp);
    let before = herb(&a);
    a.forage_herbs();
    assert!(herb(&a) > before, "the mire should yield herbs");
}

#[test]
fn the_sand_is_barren_for_an_herbalist() {
    let mut a = app_on(Terrain::DeepDesert);
    let before = herb(&a);
    a.forage_herbs();
    assert_eq!(herb(&a), before, "no physic grows in the deep desert");
    assert!(a.status_msg.clone().unwrap_or_default().contains("barren"));
}

#[test]
fn the_dark_hides_the_herbs() {
    let mut a = app_on(Terrain::Forest);
    a.clock.hour = 2; // deep night
    let before = herb(&a);
    a.forage_herbs();
    assert_eq!(herb(&a), before, "cannot forage in the dark");
    assert!(a
        .status_msg
        .clone()
        .unwrap_or_default()
        .to_lowercase()
        .contains("dark"));
}
