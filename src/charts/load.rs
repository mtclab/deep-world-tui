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
        let charts = load_charts("data/charts.ron").expect("charts.ron must parse");
        assert!(!charts.people.entries.is_empty());
        assert!(!charts.region.entries.is_empty());
        assert!(charts.people.entries.contains_key("metsik"));
        assert!(charts.people.entries.contains_key("ahjo"));
    }

    #[test]
    fn chart_integrity_people_keys() {
        let charts = load_charts("data/charts.ron").unwrap();
        // All profession modifier People() keys exist in people table
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
}
