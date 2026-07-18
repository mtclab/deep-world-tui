use crate::charts::Charts;
use anyhow::{Context, Result};
use std::path::Path;

const CHARTS_RON: &str = include_str!("../../data/charts.ron");

pub fn load_charts() -> Result<Charts> {
    load_charts_from_override().unwrap_or_else(load_charts_embedded)
}

pub fn load_charts_from_path(path: &Path) -> Result<Charts> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read charts from {}", path.display()))?;
    let charts: Charts = ron::from_str(&contents)
        .with_context(|| format!("Failed to parse charts from {}", path.display()))?;
    Ok(charts)
}

fn load_charts_from_override() -> Option<Result<Charts>> {
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(path) = std::env::var("DEEP_WORLD_CHARTS") {
            if !path.is_empty() {
                return Some(load_charts_from_path(Path::new(&path)));
            }
        }
        if let Some(config_dir) = dirs::config_dir() {
            let user_path = config_dir.join("deep-world-tui").join("charts.ron");
            if user_path.exists() {
                return Some(load_charts_from_path(&user_path));
            }
        }
        None
    }
}

fn load_charts_embedded() -> Result<Charts> {
    let charts: Charts = ron::from_str(CHARTS_RON)?;
    Ok(charts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_starter_charts() {
        let charts = load_charts().expect("charts.ron must parse");
        assert!(!charts.people.entries.is_empty());
        assert!(!charts.region.entries.is_empty());
        assert!(charts.people.entries.contains_key("metsik"));
        assert!(charts.people.entries.contains_key("ahjo"));
    }

    #[test]
    fn chart_integrity_people_keys() {
        let charts = load_charts().unwrap();
        for modifier in &charts.profession.modifiers {
            if let crate::charts::Condition::People(p) = &modifier.when {
                assert!(
                    charts.people.entries.contains_key(p),
                    "People modifier references unknown people '{}'",
                    p
                );
            }
        }
    }

    #[test]
    fn roundtrip_serde() {
        let original = load_charts().unwrap();
        let ser = ron::ser::to_string_pretty(&original, ron::ser::PrettyConfig::default()).unwrap();
        let roundtripped: Charts = ron::from_str(&ser).unwrap();
        assert_eq!(original.people.entries, roundtripped.people.entries);
        assert_eq!(original.region.entries, roundtripped.region.entries);
        assert_eq!(
            original.settlement_size.base.entries,
            roundtripped.settlement_size.base.entries
        );
        assert_eq!(
            original.social_class.entries,
            roundtripped.social_class.entries
        );
        assert_eq!(
            original.profession.base.entries,
            roundtripped.profession.base.entries
        );
        assert_eq!(
            original.profession.modifiers.len(),
            roundtripped.profession.modifiers.len()
        );
        assert_eq!(
            original.craft_affinity.base.entries,
            roundtripped.craft_affinity.base.entries
        );
        assert_eq!(
            original.personality_traits.entries,
            roundtripped.personality_traits.entries
        );
        assert_eq!(original.has_spouse.entries, roundtripped.has_spouse.entries);
        assert_eq!(
            original.children_count.entries,
            roundtripped.children_count.entries
        );
        assert_eq!(original.has_debt.entries, roundtripped.has_debt.entries);
        assert_eq!(original.age_band.entries, roundtripped.age_band.entries);
        assert_eq!(original.sex.entries, roundtripped.sex.entries);
        assert_eq!(original.name_grammars, roundtripped.name_grammars);
    }

    #[test]
    fn all_fields_have_min_two_entries() {
        let charts = load_charts().unwrap();
        assert!(
            charts.people.entries.len() >= 2,
            "people has {} entries, need ≥2",
            charts.people.entries.len()
        );
        assert!(
            charts.region.entries.len() >= 2,
            "region has {} entries, need ≥2",
            charts.region.entries.len()
        );
        assert!(
            charts.settlement_size.base.entries.len() >= 2,
            "settlement_size has {} entries, need ≥2",
            charts.settlement_size.base.entries.len()
        );
        assert!(
            charts.social_class.entries.len() >= 2,
            "social_class has {} entries, need ≥2",
            charts.social_class.entries.len()
        );
        assert!(
            charts.profession.base.entries.len() >= 2,
            "profession base has {} entries, need ≥2",
            charts.profession.base.entries.len()
        );
        assert!(
            charts.craft_affinity.base.entries.len() >= 2,
            "craft_affinity base has {} entries, need ≥2",
            charts.craft_affinity.base.entries.len()
        );
        assert!(
            charts.personality_traits.entries.len() >= 2,
            "personality_traits has {} entries, need ≥2",
            charts.personality_traits.entries.len()
        );
    }
}
