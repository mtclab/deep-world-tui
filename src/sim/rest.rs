use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestQuality {
    OutInCold,
    Campfire,
    LeanTo,
    SettlementFloor,
    Inn,
}

impl RestQuality {
    pub fn stamina_per_hour(self) -> f64 {
        match self {
            RestQuality::OutInCold => 0.04,
            RestQuality::Campfire => 0.08,
            RestQuality::LeanTo => 0.12,
            RestQuality::SettlementFloor => 0.10,
            RestQuality::Inn => 0.18,
        }
    }

    pub fn morale_per_hour(self) -> f64 {
        match self {
            RestQuality::OutInCold => 0.02,
            RestQuality::Campfire => 0.04,
            RestQuality::LeanTo => 0.06,
            RestQuality::SettlementFloor => 0.04,
            RestQuality::Inn => 0.10,
        }
    }

    pub fn encounter_risk_per_hour(self) -> f64 {
        match self {
            RestQuality::OutInCold => 0.18,
            RestQuality::Campfire => 0.08,
            RestQuality::LeanTo => 0.04,
            RestQuality::SettlementFloor => 0.02,
            RestQuality::Inn => 0.005,
        }
    }

    pub fn journal_flavor(self) -> &'static str {
        match self {
            RestQuality::OutInCold => "Cold. I do not sleep.",
            RestQuality::Campfire => "The fire is small. I sleep.",
            RestQuality::LeanTo => "The walls hold the wind.",
            RestQuality::SettlementFloor => "A borrowed floor.",
            RestQuality::Inn => "I pay for warmth.",
        }
    }
}

impl fmt::Display for RestQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestQuality::OutInCold => write!(f, "out in the cold"),
            RestQuality::Campfire => write!(f, "by campfire"),
            RestQuality::LeanTo => write!(f, "in a lean-to"),
            RestQuality::SettlementFloor => write!(f, "on a settlement floor"),
            RestQuality::Inn => write!(f, "at an inn"),
        }
    }
}

pub fn tile_rest_quality(
    on_settlement: bool,
    inn_paid: bool,
    has_campfire: bool,
    has_lean_to: bool,
) -> RestQuality {
    if inn_paid {
        RestQuality::Inn
    } else if on_settlement {
        RestQuality::SettlementFloor
    } else if has_lean_to {
        RestQuality::LeanTo
    } else if has_campfire {
        RestQuality::Campfire
    } else {
        RestQuality::OutInCold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_quality_recovery_monotonic() {
        let tiers = [
            RestQuality::OutInCold,
            RestQuality::Campfire,
            RestQuality::LeanTo,
            RestQuality::SettlementFloor,
            RestQuality::Inn,
        ];
        for window in tiers.windows(2) {
            assert!(
                window[1].stamina_per_hour() > window[0].stamina_per_hour()
                    || (window[0] == RestQuality::LeanTo
                        && window[1] == RestQuality::SettlementFloor),
                "stamina not monotonic: {:?} stamina={:?} vs {:?} stamina={:?}",
                window[0],
                window[0].stamina_per_hour(),
                window[1],
                window[1].stamina_per_hour(),
            );
        }
        assert!(RestQuality::Inn.stamina_per_hour() > RestQuality::OutInCold.stamina_per_hour());
        assert!(RestQuality::Inn.morale_per_hour() > RestQuality::OutInCold.morale_per_hour());
    }

    #[test]
    fn rest_quality_risk_monotonic_decreasing() {
        let tiers = [
            RestQuality::OutInCold,
            RestQuality::Campfire,
            RestQuality::LeanTo,
            RestQuality::SettlementFloor,
            RestQuality::Inn,
        ];
        for window in tiers.windows(2) {
            assert!(
                window[1].encounter_risk_per_hour() < window[0].encounter_risk_per_hour(),
                "risk not decreasing: {:?} risk={:?} vs {:?} risk={:?}",
                window[0],
                window[0].encounter_risk_per_hour(),
                window[1],
                window[1].encounter_risk_per_hour(),
            );
        }
    }

    #[test]
    fn rest_quality_inn_best() {
        assert_eq!(
            tile_rest_quality(true, true, false, false),
            RestQuality::Inn
        );
    }

    #[test]
    fn rest_quality_settlement_floor() {
        assert_eq!(
            tile_rest_quality(true, false, false, false),
            RestQuality::SettlementFloor
        );
    }

    #[test]
    fn rest_quality_campfire() {
        assert_eq!(
            tile_rest_quality(false, false, true, false),
            RestQuality::Campfire
        );
    }

    #[test]
    fn rest_quality_lean_to() {
        assert_eq!(
            tile_rest_quality(false, false, false, true),
            RestQuality::LeanTo
        );
    }

    #[test]
    fn rest_quality_out_in_cold() {
        assert_eq!(
            tile_rest_quality(false, false, false, false),
            RestQuality::OutInCold
        );
    }

    #[test]
    fn rest_quality_journal_no_numbers() {
        for rq in [
            RestQuality::OutInCold,
            RestQuality::Campfire,
            RestQuality::LeanTo,
            RestQuality::SettlementFloor,
            RestQuality::Inn,
        ] {
            let text = rq.journal_flavor();
            assert!(!text.contains(|c: char| c.is_ascii_digit()));
            assert!(!text.contains('%'));
            assert!(!text.contains("bar"));
        }
    }
}
