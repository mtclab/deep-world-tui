//! The polity: the territorial power the playable province answers to. The map
//! is one province of a continent of millions (see SCALE.md); the polity is the
//! authority that taxes, judges, and treats over it. Canon source:
//! deep-world-history nations/the_37_polities.md — the Cross-Code 81000 polity
//! criteria are territory, revenue, law, and treaty capacity. The one the
//! player feels is **revenue**: a resident owes the hearth-tax each season.
//!
//! Which polity holds a province follows its land — a river basin answers to
//! the League that works the grain barges, a coast to the Free Cities, the
//! steppe to the clan-alliance that gathers there. Derived from the region
//! type at worldgen, deterministic, and never authored by hand.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Polity {
    /// River Sampa Basin Leagues — twelve league-cities, one toll-signature.
    /// Revenue is the toll, and the toll is not gentle.
    SampaLeagues,
    /// Tähti Free Cities — the Compact Council's five survivors. Light hand,
    /// merchant's reckoning.
    TahtiFreeCities,
    /// Kelta Delta League — eight delta harbours under the Tidal Exchange.
    KeltaDelta,
    /// The Velkari Remnant — imperial continuation, a grain-ration and a
    /// debased coin, taxing as of ancient right. The default master of any
    /// province whose land does not clearly answer to another.
    #[default]
    VelkariRemnant,
    /// The Northern Forest Principalities (Keurimä reach) — hospitality law,
    /// the lightest levy in the Known World.
    ForestPrincipalities,
    /// The Steppe Clan-Alliance — exists at the seasonal gatherings; its due
    /// is a herd-tithe, reckoned when the clans meet.
    SteppeAlliance,
}

impl Polity {
    /// The polity that holds a province of this dominant region type. The land
    /// decides the master.
    pub fn for_region_type(region_type: &str) -> Polity {
        match region_type {
            "river_valley" => Polity::SampaLeagues,
            "coast" => Polity::TahtiFreeCities,
            "delta" => Polity::KeltaDelta,
            "forest" => Polity::ForestPrincipalities,
            "steppe" => Polity::SteppeAlliance,
            // Uplands and anything unclassed answer to the old imperial centre.
            _ => Polity::VelkariRemnant,
        }
    }

    /// The name the roads and registers use.
    pub fn name(self) -> &'static str {
        match self {
            Polity::SampaLeagues => "the River Sampa Leagues",
            Polity::TahtiFreeCities => "the Tähti Free Cities",
            Polity::KeltaDelta => "the Kelta Delta League",
            Polity::VelkariRemnant => "the Velkari Remnant",
            Polity::ForestPrincipalities => "the Forest Principalities",
            Polity::SteppeAlliance => "the Steppe Clan-Alliance",
        }
    }

    /// What the polity calls the levy it takes from a resident hearth.
    pub fn levy_name(self) -> &'static str {
        match self {
            Polity::SampaLeagues => "barge-toll",
            Polity::TahtiFreeCities => "city reckoning",
            Polity::KeltaDelta => "tide-due",
            Polity::VelkariRemnant => "imperial hearth-tax",
            Polity::ForestPrincipalities => "hospitality tithe",
            Polity::SteppeAlliance => "herd-tithe",
        }
    }

    /// How hard the polity taxes, as a multiplier on the base per-structure
    /// rate. The Leagues toll heavy; the forest barely asks; the Remnant taxes
    /// by ancient right whether it can collect or not.
    pub fn levy_multiplier(self) -> f64 {
        match self {
            Polity::SampaLeagues => 1.4,
            Polity::KeltaDelta => 1.2,
            Polity::VelkariRemnant => 1.1,
            Polity::TahtiFreeCities => 1.0,
            Polity::SteppeAlliance => 0.8,
            Polity::ForestPrincipalities => 0.6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_land_decides_the_master() {
        assert_eq!(
            Polity::for_region_type("river_valley"),
            Polity::SampaLeagues
        );
        assert_eq!(Polity::for_region_type("coast"), Polity::TahtiFreeCities);
        assert_eq!(Polity::for_region_type("steppe"), Polity::SteppeAlliance);
        // The unclassed upland answers to the old centre.
        assert_eq!(Polity::for_region_type("upland"), Polity::VelkariRemnant);
        assert_eq!(Polity::for_region_type("whatever"), Polity::VelkariRemnant);
    }

    #[test]
    fn every_polity_names_itself_and_its_levy() {
        for p in [
            Polity::SampaLeagues,
            Polity::TahtiFreeCities,
            Polity::KeltaDelta,
            Polity::VelkariRemnant,
            Polity::ForestPrincipalities,
            Polity::SteppeAlliance,
        ] {
            assert!(p.name().starts_with("the "));
            assert!(!p.levy_name().is_empty());
            assert!(p.levy_multiplier() > 0.0);
        }
    }
}
