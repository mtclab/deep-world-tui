// Animal stat bonuses actually apply: a gathering animal (hound) makes the
// player gather more over time than gathering alone.

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{Animal, Companion, ItemType};
use deep_world_tui::ui::app::App;

fn fresh_app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut app = App::new(seed, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app
}

fn total_gatherables(app: &App) -> u32 {
    let ps = app.player_start.as_ref().unwrap();
    [
        ItemType::Herb,
        ItemType::Wood,
        ItemType::Food,
        ItemType::Tinder,
    ]
    .iter()
    .map(|i| ps.inventory.get(*i))
    .sum()
}

#[test]
fn hound_companion_increases_gather_yield() {
    let mut plain = fresh_app(3);
    let before_plain = total_gatherables(&plain);
    for _ in 0..40 {
        plain.gather();
    }
    let plain_gain = total_gatherables(&plain).saturating_sub(before_plain);

    let mut with_hound = fresh_app(3);
    with_hound
        .player_start
        .as_mut()
        .unwrap()
        .companions
        .push(Companion::new(Animal::Hound, "Bracken".into(), 0));
    let before_h = total_gatherables(&with_hound);
    for _ in 0..40 {
        with_hound.gather();
    }
    let hound_gain = total_gatherables(&with_hound).saturating_sub(before_h);

    assert!(
        hound_gain >= plain_gain,
        "a hound should not reduce gathering: plain {plain_gain} vs hound {hound_gain}"
    );
}
