use crate::model::{CollapseOutcome, GodName, PlayerVitals};
use serde::{Deserialize, Serialize};

/// A collapse event recorded for playtest analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseEvent {
    pub tick: u64,
    pub vitals_before: PlayerVitals,
    pub region: String,
    pub weather: String,
    pub outcome: CollapseOutcome,
    pub died: bool,
    pub rescued_by: Option<GodName>,
}

/// A recovery event from a collapse (hours regained).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEvent {
    pub tick: u64,
    pub outcome: CollapseOutcome,
    pub hours_recovered: u32,
}

/// A death event — terminal collapse outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathEvent {
    pub tick: u64,
    pub cause: String,
    pub outcome: CollapseOutcome,
}

/// Aggregated statistics across multiple playthroughs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollapseStats {
    pub total_collapses: u64,
    pub total_deaths: u64,
    pub total_ticks: u64,
    pub outcomes: std::collections::HashMap<String, u64>,
    pub death_causes: std::collections::HashMap<String, u64>,
    pub total_hours_recovered: u64,
}

impl CollapseStats {
    pub fn death_rate(&self) -> f64 {
        if self.total_collapses == 0 {
            0.0
        } else {
            self.total_deaths as f64 / self.total_collapses as f64
        }
    }

    pub fn average_hours_recovered(&self) -> f64 {
        if self.total_collapses == 0 {
            0.0
        } else {
            self.total_hours_recovered as f64 / self.total_collapses as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_event_fields_match() {
        let event = CollapseEvent {
            tick: 42,
            vitals_before: PlayerVitals::new(),
            region: "Metsikwood".to_string(),
            weather: "clear".to_string(),
            outcome: CollapseOutcome::Ditch,
            died: false,
            rescued_by: None,
        };
        assert_eq!(event.tick, 42);
        assert_eq!(event.outcome, CollapseOutcome::Ditch);
        assert!(!event.died);
        assert_eq!(event.region, "Metsikwood");
        assert_eq!(event.weather, "clear");
    }

    #[test]
    fn death_rate_calculation() {
        let stats = CollapseStats {
            total_collapses: 100,
            total_deaths: 1,
            ..Default::default()
        };
        let rate = stats.death_rate();
        assert!((rate - 0.01).abs() < f64::EPSILON);
    }
}
