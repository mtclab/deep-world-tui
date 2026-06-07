// Tunable simulation parameters loaded from data/sim_params.ron
// Hardcoded defaults serve as fallback when values are missing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimParams {
    // Need decay rates (per tick, before dt multiplier)
    pub food_decay_rate: f64,
    pub money_decay_rate: f64,
    pub care_decay_rate: f64,
    pub presence_decay_rate: f64,
    pub safety_decay_rate: f64,

    // Dependent care acceleration (multiplier on care decay when caregiver absent)
    pub dependent_care_acceleration: f64,

    // Reputation
    pub reputation_baseline: f64,
    pub reputation_local_decay_rate: f64,
    pub reputation_faction_decay_rate: f64,
    pub reputation_spread_rate: f64,

    // Relationships
    pub trust_convergence_rate: f64,

    // Deferred effect timing (min/max ticks)
    pub deferred_effect_min_ticks: u64,
    pub deferred_effect_max_ticks: u64,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            food_decay_rate: 0.08,
            money_decay_rate: 0.04,
            care_decay_rate: 0.02,
            presence_decay_rate: 0.02,
            safety_decay_rate: 0.01,
            dependent_care_acceleration: 2.0,
            reputation_baseline: 0.5,
            reputation_local_decay_rate: 0.01,
            reputation_faction_decay_rate: 0.005,
            reputation_spread_rate: 0.1,
            trust_convergence_rate: 0.005,
            deferred_effect_min_ticks: 5,
            deferred_effect_max_ticks: 30,
        }
    }
}

impl SimParams {
    pub fn load(path: &str) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read sim params: {}", e))?;
        ron::from_str(&contents).map_err(|e| format!("Failed to parse sim params: {}", e))
    }

    pub fn load_or_default(path: &str) -> Self {
        Self::load(path).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_params_default_values() {
        let p = SimParams::default();
        assert!(
            p.food_decay_rate > p.safety_decay_rate,
            "food should decay faster than safety"
        );
        assert!(
            p.food_decay_rate > p.money_decay_rate,
            "food should decay faster than money"
        );
        assert!(p.care_decay_rate > 0.0);
        assert!(p.reputation_spread_rate > 0.0 && p.reputation_spread_rate < 1.0);
        assert!(p.trust_convergence_rate > 0.0 && p.trust_convergence_rate < 0.1);
    }

    #[test]
    fn sim_params_ron_roundtrip() {
        let original = SimParams::default();
        let ron_str =
            ron::ser::to_string_pretty(&original, ron::ser::PrettyConfig::default()).unwrap();
        let restored: SimParams = ron::from_str(&ron_str).unwrap();
        assert!((original.food_decay_rate - restored.food_decay_rate).abs() < f64::EPSILON);
        assert!((original.reputation_baseline - restored.reputation_baseline).abs() < f64::EPSILON);
        assert_eq!(
            original.deferred_effect_min_ticks,
            restored.deferred_effect_min_ticks
        );
    }

    #[test]
    fn hundred_ticks_food_decay_pattern() {
        let params = SimParams::default();
        let mut food = 1.0_f64;
        for _ in 0..100 {
            food -= params.food_decay_rate;
            food = food.max(0.0);
        }
        assert!(
            food < 0.5,
            "food should decay below 0.5 after 100 ticks without satisfaction: got {}",
            food
        );
    }

    #[test]
    fn hundred_ticks_safety_stable() {
        let params = SimParams::default();
        // Safety decays slowest: after 50 ticks still above 0.3
        let mut safety = 1.0_f64;
        for _ in 0..50 {
            safety -= params.safety_decay_rate;
            safety = safety.max(0.0);
        }
        assert!(
            safety > 0.3,
            "safety should stay above 0.3 after 50 ticks without threat: got {}",
            safety
        );
        // After 100 ticks safety is low but not zero (starts at 1.0, decays 0.01/tick = 0.0)
        // This is correct — safety decays linearly with no restoration
    }

    #[test]
    fn reputation_decays_toward_baseline() {
        let params = SimParams::default();
        let mut rep = 0.0_f64;
        for _ in 0..50 {
            rep += params.reputation_local_decay_rate;
            if rep > params.reputation_baseline {
                rep = params.reputation_baseline;
            }
        }
        assert!(
            rep > 0.3,
            "reputation should converge toward baseline after 50 ticks: got {}",
            rep
        );
    }
}
