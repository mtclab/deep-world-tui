// Regression for weather-depth wiring:
// - need_decay_modifier wears the player down faster in harsh weather
// (The encounter-rate and MerchantCaravan-spawn tests went with the encounter
// screen's retirement (#649); caravans are grid actors now.)
use deep_world_tui::model::weather::Weather;

#[test]
fn harsh_weather_speeds_vitals_decay() {
    use deep_world_tui::model::{Inventory, PlayerVitals, Season};
    let mut calm = PlayerVitals::new();
    let mut harsh = PlayerVitals::new();
    let mut inv = Inventory::default();
    // Same hours, harsher multiplier (e.g. Whiteout need_decay 1.3).
    calm.tick_with_illness(5, &mut inv, Season::Thaw, 1.0);
    let mut inv2 = Inventory::default();
    harsh.tick_with_illness(
        5,
        &mut inv2,
        Season::Thaw,
        Weather::Whiteout.need_decay_modifier(),
    );
    assert!(
        harsh.hunger < calm.hunger,
        "harsh weather should drain hunger faster ({} vs {})",
        harsh.hunger,
        calm.hunger
    );
}
