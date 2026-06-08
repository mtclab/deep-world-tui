use crate::charts::{Charts, Condition};

const GOD_NAMES: &[&str] = &["Oltzed", "Keuru", "Sampsa", "Masa", "Kukri"];

fn contains_god_name(text: &str) -> bool {
    let lower = text.to_lowercase();
    GOD_NAMES.iter().any(|g| lower.contains(&g.to_lowercase()))
}

pub fn validate_charts(charts: &Charts) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();

    for modifier in &charts.profession.modifiers {
        match &modifier.when {
            Condition::People(p) => {
                if !charts.people.entries.contains_key(p) {
                    errors.push(format!(
                        "profession modifier references unknown people '{}'",
                        p
                    ));
                }
            }
            Condition::Region(r) => {
                if !charts.region.entries.contains_key(r) {
                    errors.push(format!(
                        "profession modifier references unknown region '{}'",
                        r
                    ));
                }
            }
            Condition::Class(c) => {
                if !charts.social_class.entries.contains_key(c) {
                    errors.push(format!(
                        "profession modifier references unknown class '{}'",
                        c
                    ));
                }
            }
            Condition::Settlement(s) => {
                if !charts.settlement_size.base.entries.contains_key(s) {
                    errors.push(format!(
                        "profession modifier references unknown settlement '{}'",
                        s
                    ));
                }
            }
        }
        for key in modifier.mult.keys() {
            if !charts.profession.base.entries.contains_key(key) {
                errors.push(format!(
                    "profession modifier for {:?} references unknown outcome key '{}'",
                    modifier.when, key
                ));
            }
        }
    }

    for modifier in &charts.craft_affinity.modifiers {
        match &modifier.when {
            Condition::People(p) => {
                if !charts.people.entries.contains_key(p) {
                    errors.push(format!(
                        "craft_affinity modifier references unknown people '{}'",
                        p
                    ));
                }
            }
            Condition::Region(r) => {
                if !charts.region.entries.contains_key(r) {
                    errors.push(format!(
                        "craft_affinity modifier references unknown region '{}'",
                        r
                    ));
                }
            }
            Condition::Class(c) => {
                if !charts.social_class.entries.contains_key(c) {
                    errors.push(format!(
                        "craft_affinity modifier references unknown class '{}'",
                        c
                    ));
                }
            }
            Condition::Settlement(s) => {
                if !charts.settlement_size.base.entries.contains_key(s) {
                    errors.push(format!(
                        "craft_affinity modifier references unknown settlement '{}'",
                        s
                    ));
                }
            }
        }
        for key in modifier.mult.keys() {
            if !charts.craft_affinity.base.entries.contains_key(key) {
                errors.push(format!(
                    "craft_affinity modifier for {:?} references unknown outcome key '{}'",
                    modifier.when, key
                ));
            }
        }
    }

    for (key, w) in &charts.people.entries {
        if *w == 0 {
            errors.push(format!("people '{}' has zero weight", key));
        }
    }
    for (key, w) in &charts.region.entries {
        if *w == 0 {
            errors.push(format!("region '{}' has zero weight", key));
        }
    }

    for people_key in charts.people.entries.keys() {
        if !charts.name_grammars.contains_key(people_key) {
            errors.push(format!("people '{}' has no name_grammar entry", people_key));
        }
    }

    for key in charts.people.entries.keys() {
        if contains_god_name(key) {
            errors.push(format!(
                "people '{}' contains a god name — god names must never be people names",
                key
            ));
        }
    }
    for key in charts.profession.base.entries.keys() {
        if contains_god_name(key) {
            errors.push(format!("profession '{}' contains a god name", key));
        }
    }
    for key in charts.personality_traits.entries.keys() {
        if contains_god_name(key) {
            errors.push(format!("personality '{}' contains a god name", key));
        }
    }

    if errors.is_empty() {
        Ok(vec![])
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::load::load_charts;
    use std::collections::HashMap;

    #[test]
    fn validate_starter_charts_passes() {
        let charts = load_charts("data/charts.ron").unwrap();
        assert!(validate_charts(&charts).is_ok());
    }

    #[test]
    fn validate_catches_unknown_people_condition() {
        let mut charts = load_charts("data/charts.ron").unwrap();
        charts.profession.modifiers.push(crate::charts::Modifier {
            when: Condition::People("nonexistent_people".into()),
            mult: HashMap::new(),
        });
        let errors = validate_charts(&charts).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("unknown people 'nonexistent_people'")));
    }

    #[test]
    fn validate_catches_unknown_modifier_outcome() {
        let mut charts = load_charts("data/charts.ron").unwrap();
        let mut mult = HashMap::new();
        mult.insert("nonexistent_prof".into(), 2.0);
        charts.profession.modifiers.push(crate::charts::Modifier {
            when: Condition::People("metsik".into()),
            mult,
        });
        let errors = validate_charts(&charts).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("unknown outcome key 'nonexistent_prof'")));
    }

    #[test]
    fn validate_catches_zero_weight() {
        let mut charts = load_charts("data/charts.ron").unwrap();
        charts.region.entries.insert("zero_region".into(), 0);
        let errors = validate_charts(&charts).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("zero_region") && e.contains("zero weight")));
    }

    #[test]
    fn validate_catches_missing_name_grammar() {
        let mut charts = load_charts("data/charts.ron").unwrap();
        charts.people.entries.insert("new_people".into(), 10);
        let errors = validate_charts(&charts).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("new_people") && e.contains("name_grammar")));
    }

    #[test]
    fn validate_starter_charts_no_god_names() {
        let charts = load_charts("data/charts.ron").unwrap();
        let result = validate_charts(&charts);
        if let Err(errors) = &result {
            for e in errors {
                assert!(!e.contains("god name"), "Found god name in charts: {}", e);
            }
        }
    }

    #[test]
    fn validate_catches_god_name_in_people() {
        let mut charts = load_charts("data/charts.ron").unwrap();
        charts.people.entries.insert("Oltzed".into(), 10);
        charts
            .name_grammars
            .insert("Oltzed".into(), "names/metsik.ron".into());
        let errors = validate_charts(&charts).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("god name")));
    }
}
