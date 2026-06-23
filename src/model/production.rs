//! Data-driven production for registry goods (#678 slice 3).
//!
//! The core `ItemType` wares (Iron, Tool, Cloth, …) are produced by hand-written
//! rules in the living economy — they carry code behaviour. The long tail of
//! data-defined trade goods (`data/goods.ron`) is produced by **data** rules
//! here (`data/production.ron`), so a town's shelf grows by editing data, not
//! code: add a good, add a line saying which trades make it, and it flows.
//!
//! Each rule yields an amount for one good from a settlement's trade counts:
//!
//! ```text
//! amount = (Σ weightᵢ·count(professionᵢ) + coastal_base·[coastal])
//!          · ([coastal] if coastal_only else 1)
//!          · (region_richness if richness else 1)
//! ```
//!
//! The caller then scales by population/season and applies the shared cap and
//! upkeep, exactly as for core goods.

use std::sync::OnceLock;

use serde::Deserialize;

/// One authored production rule, keyed by a registry good's slug.
#[derive(Debug, Clone, Deserialize)]
pub struct ProductionRule {
    /// The registry good this rule makes (a slug in `data/goods.ron`).
    pub good: String,
    /// `(profession, weight)` contributions — the hands that make it.
    #[serde(default)]
    pub by: Vec<(String, f64)>,
    /// A flat contribution added only in coastal settlements (salt pans, the
    /// shore's own bounty) — independent of any one trade.
    #[serde(default)]
    pub coastal_base: f64,
    /// If set, the whole rule yields nothing away from the coast.
    #[serde(default)]
    pub coastal_only: bool,
    /// If set, the yield scales with the region's wildness (hunted/gathered).
    #[serde(default)]
    pub richness: bool,
    /// If non-empty, the rule yields ONLY in these region_types — the biome
    /// gate. E.g. ["steppe"] for goat-cheese, ["delta","river_valley"] for
    /// wine, ["forest","tundra","upland"] for tar. Empty = any region. The
    /// wider world's other bands reach a town as imports, not local make.
    #[serde(default)]
    pub regions: Vec<String>,
    /// If non-empty, the rule yields ONLY in these seasons ("thaw"/"green"/
    /// "frost") — fresh produce in Green, the wild fur-take in Frost, birch-sap
    /// in Thaw. Empty = every season (the global season modifier still applies).
    #[serde(default)]
    pub seasons: Vec<String>,
}

impl ProductionRule {
    /// The raw amount this rule makes for a settlement, before the caller's
    /// population/season scaling. `count` returns a settlement's head-count of a
    /// profession.
    pub fn amount(
        &self,
        coastal: bool,
        region_richness: f64,
        region_type: &str,
        season: &str,
        count: &impl Fn(&str) -> f64,
    ) -> f64 {
        if self.coastal_only && !coastal {
            return 0.0;
        }
        if !self.regions.is_empty() && !self.regions.iter().any(|r| r == region_type) {
            return 0.0;
        }
        if !self.seasons.is_empty() && !self.seasons.iter().any(|s| s == season) {
            return 0.0;
        }
        let mut a: f64 = self.by.iter().map(|(p, w)| w * count(p)).sum();
        if coastal {
            a += self.coastal_base;
        }
        if self.coastal_only {
            // already gated above; the gate is the only coastal effect
        }
        if self.richness {
            a *= region_richness;
        }
        a
    }
}

const PRODUCTION_RON: &str = include_str!("../../data/production.ron");

/// All production rules, parsed once from the embedded data.
pub fn production_rules() -> &'static [ProductionRule] {
    static RULES: OnceLock<Vec<ProductionRule>> = OnceLock::new();
    RULES
        .get_or_init(|| {
            ron::from_str(PRODUCTION_RON)
                .expect("data/production.ron must parse as Vec<ProductionRule>")
        })
        .as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::good_id;

    #[test]
    fn rules_load_and_name_real_goods() {
        let rules = production_rules();
        assert!(
            rules.len() >= 12,
            "a real production table, got {}",
            rules.len()
        );
        for r in rules {
            assert!(
                good_id(&r.good).is_some(),
                "production rule names a good not in the registry: {}",
                r.good
            );
        }
    }

    #[test]
    fn amount_respects_coastal_gate_and_weights() {
        let rule = ProductionRule {
            good: "salt-fish".into(),
            by: vec![("fisher".into(), 0.5)],
            coastal_base: 0.0,
            coastal_only: true,
            richness: false,
            regions: vec![],
            seasons: vec![],
        };
        let two_fishers = |p: &str| if p == "fisher" { 2.0 } else { 0.0 };
        assert!((rule.amount(true, 1.0, "coast", "green", &two_fishers) - 1.0).abs() < 1e-9);
        assert_eq!(
            rule.amount(false, 1.0, "forest", "green", &two_fishers),
            0.0,
            "inland: none"
        );
    }

    #[test]
    fn region_gate_limits_to_listed_biomes() {
        let rule = ProductionRule {
            good: "goat-cheese".into(),
            by: vec![("herder".into(), 0.5)],
            coastal_base: 0.0,
            coastal_only: false,
            richness: false,
            regions: vec!["steppe".into()],
            seasons: vec![],
        };
        let herders = |p: &str| if p == "herder" { 2.0 } else { 0.0 };
        assert!((rule.amount(false, 1.0, "steppe", "green", &herders) - 1.0).abs() < 1e-9);
        assert_eq!(
            rule.amount(false, 1.0, "forest", "green", &herders),
            0.0,
            "off-biome: none"
        );
    }

    #[test]
    fn season_gate_limits_to_listed_seasons() {
        let rule = ProductionRule {
            good: "fox-pelt".into(),
            by: vec![("hunter".into(), 0.5)],
            coastal_base: 0.0,
            coastal_only: false,
            richness: false,
            regions: vec![],
            seasons: vec!["frost".into()],
        };
        let hunters = |p: &str| if p == "hunter" { 2.0 } else { 0.0 };
        assert!((rule.amount(false, 1.0, "forest", "frost", &hunters) - 1.0).abs() < 1e-9);
        assert_eq!(
            rule.amount(false, 1.0, "forest", "green", &hunters),
            0.0,
            "off-season: none"
        );
    }
}
