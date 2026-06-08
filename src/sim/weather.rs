use crate::model::{Terrain, Weather};

const BASE_MINUTES_PER_KM: u32 = 30;

/// Compute travel time in minutes for a given distance, weather, and terrain.
///
/// Returns 0 if forced shelter is triggered (no progress made).
/// Clear weather = 1.0× base speed. Worse weather multiplies time.
/// Snow has different effects on mountain vs non-mountain terrain.
pub fn travel_time_minutes(distance_km: u32, weather: Weather, terrain: Terrain) -> u32 {
    let base = distance_km as f64 * BASE_MINUTES_PER_KM as f64;

    let multiplier = match weather {
        Weather::Clear => 1.0,
        Weather::Cloudy => 1.0,
        Weather::Rain => 1.3,
        Weather::Storm => 1.8,
        Weather::Snow => match terrain {
            Terrain::Mountain => 1.2,
            _ => 1.6,
        },
        Weather::Fog => 1.1,
        Weather::Heatwave => 1.2,
        Weather::Whiteout => 2.5,
        Weather::Thunderhead => 1.5,
        Weather::DryLightning => 1.1,
        Weather::SeaSquall => 2.0,
    };

    (base * multiplier).round() as u32
}

/// Check if weather forces the traveller into shelter (no progress).
///
/// Returns true when the roll is within the forced-shelter probability for
/// the given weather type. Deterministic given the same roll value.
pub fn forced_shelter(weather: Weather, roll: f64) -> bool {
    let threshold = match weather {
        Weather::Storm => 0.25,
        Weather::Whiteout => 0.40,
        Weather::Thunderhead => 0.15,
        Weather::SeaSquall => 0.30,
        _ => return false,
    };
    roll < threshold
}

/// Compute travel time accounting for forced-shelter chance.
///
/// Uses the provided `SeedRng` to deterministically decide forced shelter.
/// Returns 0 if forced shelter triggers; otherwise returns the weather-adjusted
/// travel time.
pub fn travel_time_with_shelter(
    distance_km: u32,
    weather: Weather,
    terrain: Terrain,
    rng: &mut crate::rng::SeedRng,
) -> u32 {
    if forced_shelter(weather, rng.gen_f64()) {
        return 0;
    }
    travel_time_minutes(distance_km, weather, terrain)
}

/// Encounter rate modifier based on weather.
///
/// Storm conditions increase encounter probability (people and creatures
/// seek the same shelter). Clear weather reduces it.
/// Returns a multiplier (not a percentage).
pub fn encounter_rate_modifier(weather: Weather) -> f64 {
    match weather {
        Weather::Storm => 1.4,
        Weather::Clear => 0.8,
        Weather::Rain => 1.0,
        Weather::Fog => 1.0,
        Weather::Cloudy => 1.0,
        Weather::Snow => 1.0,
        Weather::Heatwave => 1.0,
        Weather::Whiteout => 1.0,
        Weather::Thunderhead => 1.3,
        Weather::DryLightning => 1.0,
        Weather::SeaSquall => 1.2,
    }
}

/// Flavor text for weather-triggered shelter events.
/// Returns descriptive text for forced-shelter situations, or
/// atmospheric text when the weather merely slows travel.
pub fn weather_travel_flavor(weather: Weather, shelter: bool) -> &'static str {
    if shelter {
        match weather {
            Weather::Storm => "The storm drove you into an old pilgrim's shelter. Wind howled at the door all night.",
            Weather::Whiteout => "White erased the world. You huddled in a snow-drift hollow until it passed.",
            Weather::Thunderhead => "Lightning split the sky. You took cover beneath an overhang, counting the beats between flash and thunder.",
            Weather::SeaSquall => "The sea rose and the wind screamed. You found a fisher's hut barely standing — inside, you waited.",
            _ => "The weather forced you to stop. You waited it out.",
        }
    } else {
        match weather {
            Weather::Rain => "The road stretched long under grey skies. Rain drummed steady on your hood.",
            Weather::Storm => "Thunder rolled across the hills. You pressed on, rain streaming down your face.",
            Weather::Snow => "Snow muffled the world. Each step crunched, each breath hung in the air.",
            Weather::Fog => "Through the fog, shapes moved just beyond sight. The path faded in and out of view.",
            Weather::Heatwave => "The heat pressed down like a hand. Sweat stung your eyes, and the horizon shimmered.",
            Weather::Whiteout => "Blinding white. You staggered forward, unable to tell ground from sky.",
            Weather::Thunderhead => "The air tasted of iron. Thunder rumbled somewhere beyond the next ridge.",
            Weather::DryLightning => "Lightning forked across a cloudless sky. No rain, no sound — just dry fire above.",
            Weather::SeaSquall => "Salt spray stung your lips. The wind tore at your clothes and bent the grass flat.",
            Weather::Clear => "Clear skies and open road. The kind of day that makes you forget the world has teeth.",
            Weather::Cloudy => "Grey light filtered through the clouds. The world felt quiet, waiting.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SeedRng;

    // --- travel_time_minutes table tests ---

    #[test]
    fn clear_is_base_rate() {
        let t = travel_time_minutes(10, Weather::Clear, Terrain::Grass);
        assert_eq!(t, 300, "10 km at 30 min/km clear = 300 min");
    }

    #[test]
    fn rain_is_1_3x() {
        let t = travel_time_minutes(10, Weather::Rain, Terrain::Grass);
        assert_eq!(t, (10.0_f64 * 30.0 * 1.3).round() as u32);
    }

    #[test]
    fn storm_is_1_8x() {
        let t = travel_time_minutes(10, Weather::Storm, Terrain::Grass);
        assert_eq!(t, (10.0_f64 * 30.0 * 1.8).round() as u32);
    }

    #[test]
    fn snow_mountain_is_1_2x() {
        let t = travel_time_minutes(10, Weather::Snow, Terrain::Mountain);
        assert_eq!(t, (10.0_f64 * 30.0 * 1.2).round() as u32);
    }

    #[test]
    fn snow_non_mountain_is_1_6x() {
        for terrain in [
            Terrain::Grass,
            Terrain::Forest,
            Terrain::Road,
            Terrain::Swamp,
            Terrain::Coast,
            Terrain::Sand,
        ] {
            let t = travel_time_minutes(10, Weather::Snow, terrain);
            let expected: u32 = (10.0_f64 * 30.0 * 1.6).round() as u32;
            assert_eq!(t, expected, "snow on {:?}", terrain);
        }
    }

    #[test]
    fn fog_is_1_1x() {
        let t = travel_time_minutes(10, Weather::Fog, Terrain::Grass);
        assert_eq!(t, (10.0_f64 * 30.0 * 1.1).round() as u32);
    }

    #[test]
    fn heatwave_is_1_2x() {
        let t = travel_time_minutes(10, Weather::Heatwave, Terrain::Grass);
        assert_eq!(t, (10.0_f64 * 30.0 * 1.2).round() as u32);
    }

    #[test]
    fn whiteout_is_2_5x() {
        let t = travel_time_minutes(10, Weather::Whiteout, Terrain::Tundra);
        assert_eq!(t, (10.0_f64 * 30.0 * 2.5).round() as u32);
    }

    #[test]
    fn thunderhead_is_1_5x() {
        let t = travel_time_minutes(10, Weather::Thunderhead, Terrain::Grass);
        assert_eq!(t, (10.0_f64 * 30.0 * 1.5).round() as u32);
    }

    #[test]
    fn dry_lightning_is_1_1x() {
        let t = travel_time_minutes(10, Weather::DryLightning, Terrain::Grass);
        assert_eq!(t, (10.0_f64 * 30.0 * 1.1).round() as u32);
    }

    #[test]
    fn sea_squall_is_2_0x() {
        let t = travel_time_minutes(10, Weather::SeaSquall, Terrain::Coast);
        assert_eq!(t, (10.0_f64 * 30.0 * 2.0).round() as u32);
    }

    #[test]
    fn cloudy_same_as_clear() {
        let t = travel_time_minutes(10, Weather::Cloudy, Terrain::Grass);
        assert_eq!(t, 300, "cloudy should match clear for travel");
    }

    // --- determiinism: same seed = same travel times ---

    #[test]
    fn deterministic_travel_with_shelter() {
        let dist = 20u32;
        let weather = Weather::Storm;
        let terrain = Terrain::Grass;
        let mut r1 = SeedRng::new(12345);
        let mut r2 = SeedRng::new(12345);
        let a = travel_time_with_shelter(dist, weather, terrain, &mut r1);
        let b = travel_time_with_shelter(dist, weather, terrain, &mut r2);
        assert_eq!(a, b, "same seed must produce same travel result");
    }

    // --- forced_shelter probability ---

    #[test]
    fn forced_shelter_storm_25_percent() {
        let mut shelter_count = 0u32;
        let trials = 10_000u32;
        let mut rng = SeedRng::new(42);
        for _ in 0..trials {
            if forced_shelter(Weather::Storm, rng.gen_f64()) {
                shelter_count += 1;
            }
        }
        let ratio = shelter_count as f64 / trials as f64;
        assert!(
            (ratio - 0.25).abs() < 0.05,
            "Storm forced shelter ratio: {:.3}, expected ~0.25",
            ratio
        );
    }

    #[test]
    fn forced_shelter_whiteout_40_percent() {
        let mut shelter_count = 0u32;
        let trials = 10_000u32;
        let mut rng = SeedRng::new(99);
        for _ in 0..trials {
            if forced_shelter(Weather::Whiteout, rng.gen_f64()) {
                shelter_count += 1;
            }
        }
        let ratio = shelter_count as f64 / trials as f64;
        assert!(
            (ratio - 0.40).abs() < 0.05,
            "Whiteout forced shelter ratio: {:.3}, expected ~0.40",
            ratio
        );
    }

    #[test]
    fn forced_shelter_thunderhead_15_percent() {
        let mut shelter_count = 0u32;
        let trials = 10_000u32;
        let mut rng = SeedRng::new(77);
        for _ in 0..trials {
            if forced_shelter(Weather::Thunderhead, rng.gen_f64()) {
                shelter_count += 1;
            }
        }
        let ratio = shelter_count as f64 / trials as f64;
        assert!(
            (ratio - 0.15).abs() < 0.05,
            "Thunderhead forced shelter ratio: {:.3}, expected ~0.15",
            ratio
        );
    }

    #[test]
    fn forced_shelter_sea_squall_30_percent() {
        let mut shelter_count = 0u32;
        let trials = 10_000u32;
        let mut rng = SeedRng::new(55);
        for _ in 0..trials {
            if forced_shelter(Weather::SeaSquall, rng.gen_f64()) {
                shelter_count += 1;
            }
        }
        let ratio = shelter_count as f64 / trials as f64;
        assert!(
            (ratio - 0.30).abs() < 0.05,
            "SeaSquall forced shelter ratio: {:.3}, expected ~0.30",
            ratio
        );
    }

    #[test]
    fn forced_shelter_never_for_non_shelter_weather() {
        let weathers = [
            Weather::Clear,
            Weather::Cloudy,
            Weather::Rain,
            Weather::Fog,
            Weather::Snow,
            Weather::Heatwave,
            Weather::DryLightning,
        ];
        for w in weathers {
            assert!(
                !forced_shelter(w, 0.0),
                "{:?} should never force shelter",
                w
            );
            assert!(
                !forced_shelter(w, 0.5),
                "{:?} should never force shelter at 0.5",
                w
            );
            assert!(
                !forced_shelter(w, 0.99),
                "{:?} should never force shelter at 0.99",
                w
            );
        }
    }

    // --- encounter_rate_modifier ---

    #[test]
    fn encounter_rate_modifier_all_variants() {
        assert!((encounter_rate_modifier(Weather::Storm) - 1.4).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Clear) - 0.8).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Rain) - 1.0).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Fog) - 1.0).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Cloudy) - 1.0).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Snow) - 1.0).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Heatwave) - 1.0).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Whiteout) - 1.0).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::Thunderhead) - 1.3).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::DryLightning) - 1.0).abs() < f64::EPSILON);
        assert!((encounter_rate_modifier(Weather::SeaSquall) - 1.2).abs() < f64::EPSILON);
    }

    #[test]
    fn storm_encounter_higher_than_clear() {
        assert!(
            encounter_rate_modifier(Weather::Storm) > encounter_rate_modifier(Weather::Clear),
            "storm should increase encounter rate over clear"
        );
    }

    // --- travel flavor text ---

    #[test]
    fn weather_flavor_shelter_storm() {
        let text = weather_travel_flavor(Weather::Storm, true);
        assert!(text.contains("shelter"), "storm shelter flavor: {}", text);
    }

    #[test]
    fn weather_flavor_shelter_whiteout() {
        let text = weather_travel_flavor(Weather::Whiteout, true);
        assert!(
            text.contains("snow") || text.contains("White") || text.contains("huddled"),
            "whiteout shelter flavor: {}",
            text
        );
    }

    #[test]
    fn weather_flavor_shelter_thunderhead() {
        let text = weather_travel_flavor(Weather::Thunderhead, true);
        assert!(
            text.contains("Lightning") || text.contains("thunder"),
            "thunderhead shelter flavor: {}",
            text
        );
    }

    #[test]
    fn weather_flavor_shelter_sea_squall() {
        let text = weather_travel_flavor(Weather::SeaSquall, true);
        assert!(
            text.contains("sea") || text.contains("wind") || text.contains("hut"),
            "sea squall shelter flavor: {}",
            text
        );
    }

    #[test]
    fn weather_flavor_no_shelter_rain() {
        let text = weather_travel_flavor(Weather::Rain, false);
        assert!(
            text.contains("rain") || text.contains("Rain") || text.contains("grey"),
            "rain travel flavor: {}",
            text
        );
    }

    #[test]
    fn weather_flavor_no_shelter_fog() {
        let text = weather_travel_flavor(Weather::Fog, false);
        assert!(
            text.contains("fog") || text.contains("Fog") || text.contains("sight"),
            "fog travel flavor: {}",
            text
        );
    }

    #[test]
    fn weather_flavor_no_shelter_heatwave() {
        let text = weather_travel_flavor(Weather::Heatwave, false);
        assert!(
            text.contains("heat") || text.contains("Heat") || text.contains("shimmered"),
            "heatwave flavor: {}",
            text
        );
    }

    // --- table test: (distance × weather × terrain) ---

    #[test]
    fn travel_time_cross_product() {
        let distances = [1u32, 5, 10, 50];
        let weathers = [
            Weather::Clear,
            Weather::Rain,
            Weather::Storm,
            Weather::Snow,
            Weather::Fog,
            Weather::Heatwave,
            Weather::Whiteout,
            Weather::Thunderhead,
            Weather::DryLightning,
            Weather::SeaSquall,
        ];
        let terrains = [
            Terrain::Grass,
            Terrain::Mountain,
            Terrain::Coast,
            Terrain::Forest,
        ];

        for &d in &distances {
            for &w in &weathers {
                for &t in &terrains {
                    let result = travel_time_minutes(d, w, t);
                    assert!(
                        result > 0,
                        "travel time must be positive for {}km {:?} {:?}",
                        d,
                        w,
                        t
                    );
                    // Base rate: 30 min/km, minimum multiplier is 1.0 (clear/cloudy)
                    let base = d * BASE_MINUTES_PER_KM;
                    // Result should be >= base (multipliers are >= 1.0 except clear/cloudy = 1.0)
                    assert!(
                        result >= base,
                        "travel time {} should be >= base {} for {:?}",
                        result,
                        base,
                        w
                    );
                }
            }
        }
    }

    // --- forced shelter determinism: roll < threshold always true ---

    #[test]
    fn forced_shelter_deterministic_bounds() {
        assert!(forced_shelter(Weather::Storm, 0.0));
        assert!(forced_shelter(Weather::Storm, 0.24));
        assert!(!forced_shelter(Weather::Storm, 0.25));
        assert!(!forced_shelter(Weather::Storm, 0.26));
        assert!(!forced_shelter(Weather::Storm, 1.0));

        assert!(forced_shelter(Weather::Whiteout, 0.0));
        assert!(forced_shelter(Weather::Whiteout, 0.39));
        assert!(!forced_shelter(Weather::Whiteout, 0.40));
        assert!(!forced_shelter(Weather::Whiteout, 0.41));

        assert!(forced_shelter(Weather::Thunderhead, 0.0));
        assert!(forced_shelter(Weather::Thunderhead, 0.14));
        assert!(!forced_shelter(Weather::Thunderhead, 0.15));
        assert!(!forced_shelter(Weather::Thunderhead, 0.16));

        assert!(forced_shelter(Weather::SeaSquall, 0.0));
        assert!(forced_shelter(Weather::SeaSquall, 0.29));
        assert!(!forced_shelter(Weather::SeaSquall, 0.30));
        assert!(!forced_shelter(Weather::SeaSquall, 0.31));
    }

    #[test]
    fn travel_time_with_shelter_returns_zero_on_shelter() {
        let mut rng = SeedRng::new(42);
        let mut forced_zero = 0u32;
        let trials = 1000u32;
        for _ in 0..trials {
            let result = travel_time_with_shelter(10, Weather::Storm, Terrain::Grass, &mut rng);
            if result == 0 {
                forced_zero += 1;
            }
        }
        // ~25% should be forced shelter (0 result)
        let ratio = forced_zero as f64 / trials as f64;
        assert!(
            (ratio - 0.25).abs() < 0.08,
            "expected ~25% forced shelter, got {:.1}%",
            ratio * 100.0
        );
    }
}
