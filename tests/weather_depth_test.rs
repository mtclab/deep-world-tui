// Regression for weather-depth wiring:
// - encounter_rate_modifier now scales encounter spawn chance
// - MerchantCaravan actually spawns on roads (was defined, never spawned)
// - need_decay_modifier wears the player down faster in harsh weather
use deep_world_tui::model::weather::Weather;
use deep_world_tui::model::{Encounter, EncounterKind, Terrain};

#[test]
fn weather_multiplier_scales_encounter_chance() {
    let mut low = 0u32;
    let mut high = 0u32;
    for seed in 0..400u64 {
        if Encounter::roll_biased_weather(Terrain::Forest, 10, 1, seed, None, 0.5).is_some() {
            low += 1;
        }
        if Encounter::roll_biased_weather(Terrain::Forest, 10, 1, seed, None, 2.0).is_some() {
            high += 1;
        }
    }
    assert!(
        high > low,
        "storm weather (x2.0: {high}) should spawn more encounters than calm (x0.5: {low})"
    );
}

#[test]
fn merchant_caravans_appear_on_roads() {
    let found = (0..600u64).any(|seed| {
        matches!(
            Encounter::roll_biased_weather(Terrain::Road, 10, 1, seed, None, 1.0),
            Some(Encounter {
                kind: EncounterKind::MerchantCaravan,
                ..
            })
        )
    });
    assert!(found, "MerchantCaravan should spawn on roads");
}

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
