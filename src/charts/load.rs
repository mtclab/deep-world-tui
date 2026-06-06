use crate::charts::Charts;
use anyhow::Result;
use std::fs;

/// Load charts from a RON file.
pub fn load_charts(path: &str) -> Result<Charts> {
    let content = fs::read_to_string(path)?;
    let charts: Charts = ron::from_str(&content)?;
    Ok(charts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_starter_charts() {
        let charts = load_charts("data/charts.ron").expect("charts.rons must parse");
        assert!(!charts.people.entries.is_empty());
        assert!(!charts.region.entries.is_empty());
        assert!(charts.people.entries.contains_key("metsik"));
        assert!(charts.people.entries.contains_key("ahjo"));
    }

    #[test]
    fn chart_integrity_people_keys() {
        let charts = load_charts("data/charts.ron").unwrap();
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
        let original = load_charts("data/charts.ron").unwrap();
        let ser = ron::ser::to_string_pretty(&original, ron::ser::PrettyConfig::default()).unwrap();
        let roundtripped: Charts = ron::from_str(&ser).unwrap();
        assert_eq!(original.people.entries, roundtripped.people.entries);
        assert_eq!(original.region.entries, roundtripped.region.entries);
        assert_eq!(
            original.settlement_size.entries,
            roundtripped.settlement_size.entries
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
        let charts = load_charts("data/charts.ron").unwrap();
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
            charts.settlement_size.entries.len() >= 2,
            "settlement_size has {} entries, need ≥2",
            charts.settlement_size.entries.len()
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
