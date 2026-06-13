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

    /// The polity this one contests, in the canon rivalries: the Remnant's
    /// claim against the breadbasket Leagues; two merchant powers over the
    /// sea-lanes; the forest and the steppe over the contested edge. Paired and
    /// symmetric — a's rival names a.
    pub fn rival(self) -> Polity {
        match self {
            Polity::VelkariRemnant => Polity::SampaLeagues,
            Polity::SampaLeagues => Polity::VelkariRemnant,
            Polity::TahtiFreeCities => Polity::KeltaDelta,
            Polity::KeltaDelta => Polity::TahtiFreeCities,
            Polity::ForestPrincipalities => Polity::SteppeAlliance,
            Polity::SteppeAlliance => Polity::ForestPrincipalities,
        }
    }

    /// Whether this season finds the polity and its rival at open tension —
    /// deterministic from the world seed, the canonical pair, and the season,
    /// so the same world always runs the same wars. Roughly one season in four.
    pub fn in_tension(self, seed: u64, season_ord: u32, year: u32) -> bool {
        let a = self as u32;
        let b = self.rival() as u32;
        let (lo, hi) = (a.min(b), a.max(b));
        let mut h = seed ^ crate::rng::fnv1a_hash("polity-tension");
        h = crate::rng::mix_u64(h ^ (((lo as u64) << 8) | hi as u64));
        h = crate::rng::mix_u64(h ^ (((season_ord as u64) << 16) | year as u64));
        crate::rng::unit_from_hash(h) < 0.25
    }

    /// The war-levy multiplier on the hearth-tax while the polity is at tension:
    /// a province at war asks more of its hearths.
    pub fn war_levy_multiplier(self) -> f64 {
        1.35
    }

    /// How well a generic coin trades in this polity's markets. There is no
    /// universal currency in the Known World: the coastal merchant leagues run
    /// on coin and notes (full value); the Remnant still stamps imperial coin,
    /// increasingly debased (a verification discount); the grain and in-kind
    /// economies — Basin, forest, steppe — tolerate coin but would rather have
    /// goods, so coin buys a little less there.
    pub fn coin_value_modifier(self) -> f64 {
        match self {
            Polity::TahtiFreeCities | Polity::KeltaDelta => 1.0,
            Polity::VelkariRemnant => 0.92,
            Polity::SampaLeagues | Polity::SteppeAlliance | Polity::ForestPrincipalities => 0.85,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Polity; 6] = [
        Polity::SampaLeagues,
        Polity::TahtiFreeCities,
        Polity::KeltaDelta,
        Polity::VelkariRemnant,
        Polity::ForestPrincipalities,
        Polity::SteppeAlliance,
    ];

    #[test]
    fn rivalry_is_symmetric_and_not_self() {
        for p in ALL {
            assert_ne!(p.rival(), p, "{p:?} cannot rival itself");
            assert_eq!(p.rival().rival(), p, "{p:?} rivalry must be mutual");
        }
    }

    #[test]
    fn tension_is_deterministic_and_paired_and_occasional() {
        let seed = 12345u64;
        // Same inputs -> same answer; a pair agrees with itself.
        for p in ALL {
            assert_eq!(
                p.in_tension(seed, 2, 3),
                p.in_tension(seed, 2, 3),
                "deterministic"
            );
            assert_eq!(
                p.in_tension(seed, 1, 0),
                p.rival().in_tension(seed, 1, 0),
                "both sides see the same war"
            );
        }
        // Roughly a quarter of seasons, not always/never.
        let mut hits = 0;
        for year in 0..200u32 {
            for s in 0..4u32 {
                if Polity::VelkariRemnant.in_tension(seed, s, year) {
                    hits += 1;
                }
            }
        }
        assert!(
            hits > 100 && hits < 350,
            "tension frequency off: {hits}/800"
        );
    }

    #[test]
    fn coin_trades_best_in_coin_economies() {
        // Merchant leagues full value; Remnant debased; grain economies a
        // foreign convenience — strict ordering.
        let leagues = Polity::TahtiFreeCities.coin_value_modifier();
        let remnant = Polity::VelkariRemnant.coin_value_modifier();
        let grain = Polity::SampaLeagues.coin_value_modifier();
        assert!(leagues > remnant && remnant > grain);
        assert!(grain > 0.0 && leagues <= 1.0);
        // War always asks more than peace.
        for p in ALL {
            assert!(p.war_levy_multiplier() > 1.0);
        }
    }

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
