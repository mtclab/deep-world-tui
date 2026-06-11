use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamePattern {
    Root,
    RootSuffix,
    PrefixRoot,
    PrefixRootSuffix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameGrammar {
    pub prefixes: Vec<String>,
    pub roots: Vec<String>,
    pub suffixes: Vec<String>,
    pub patterns: Vec<NamePattern>,
}

impl NameGrammar {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let grammar: Self = ron::from_str(&content)?;
        Ok(grammar)
    }

    pub fn generate(&self, rng: &mut crate::rng::SeedRng) -> String {
        if self.patterns.is_empty() || self.roots.is_empty() {
            return "Unnamed".into();
        }
        let pattern_idx = rng.gen_range(self.patterns.len() as u32) as usize;
        let pattern = &self.patterns[pattern_idx];
        let raw = match pattern {
            NamePattern::Root => {
                let ri = rng.gen_range(self.roots.len() as u32) as usize;
                self.roots[ri].clone()
            }
            NamePattern::RootSuffix => {
                let ri = rng.gen_range(self.roots.len() as u32) as usize;
                let si = rng.gen_range(self.suffixes.len() as u32) as usize;
                format!("{}{}", self.roots[ri], self.suffixes[si])
            }
            NamePattern::PrefixRoot => {
                let pi = rng.gen_range(self.prefixes.len() as u32) as usize;
                let ri = rng.gen_range(self.roots.len() as u32) as usize;
                format!("{}{}", self.prefixes[pi], self.roots[ri])
            }
            NamePattern::PrefixRootSuffix => {
                let pi = rng.gen_range(self.prefixes.len() as u32) as usize;
                let ri = rng.gen_range(self.roots.len() as u32) as usize;
                let si = rng.gen_range(self.suffixes.len() as u32) as usize;
                format!(
                    "{}{}{}",
                    self.prefixes[pi], self.roots[ri], self.suffixes[si]
                )
            }
        };
        // Compound segments join as one word: capitalize only the first
        // letter ("Poro"+"Sarvi" -> "Porosarvi", not the CamelCase mash that
        // produced names like "HiljPilkku").
        let lowered = raw.to_lowercase();
        let mut chars = lowered.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => raw,
        }
    }
}

pub fn load_grammar(people: &str, charts: &crate::charts::Charts) -> anyhow::Result<NameGrammar> {
    let rel_path = charts
        .name_grammars
        .get(people)
        .ok_or_else(|| anyhow::anyhow!("no name_grammar for people '{}'", people))?;
    let path = if std::path::Path::new(rel_path).is_absolute() {
        rel_path.clone()
    } else {
        format!("data/{}", rel_path)
    };
    NameGrammar::load(&path)
}

pub fn generate_name(
    rng: &mut crate::rng::SeedRng,
    people: &str,
    _sex: &str,
    charts: &crate::charts::Charts,
) -> anyhow::Result<String> {
    let grammar = load_grammar(people, charts)?;
    Ok(grammar.generate(rng))
}

/// A short stem for place names: a bare root (or root+suffix one time in
/// three), so settlement names don't bloat once the regional final is added.
pub fn generate_place_stem(
    rng: &mut crate::rng::SeedRng,
    people: &str,
    charts: &crate::charts::Charts,
) -> anyhow::Result<String> {
    let grammar = load_grammar(people, charts)?;
    if grammar.roots.is_empty() {
        return Ok("Unnamed".into());
    }
    let ri = rng.gen_range(grammar.roots.len() as u32) as usize;
    let mut raw = grammar.roots[ri].clone();
    if !grammar.suffixes.is_empty() && rng.gen_range(3) == 0 {
        let si = rng.gen_range(grammar.suffixes.len() as u32) as usize;
        raw.push_str(&grammar.suffixes[si]);
    }
    let lowered = raw.to_lowercase();
    let mut chars = lowered.chars();
    Ok(match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SeedRng;

    #[test]
    fn load_all_six_grammar_files() {
        let charts = crate::charts::load_charts().unwrap();
        for people in &["metsik", "arkit", "vayla", "laakso", "sepat", "ahjo"] {
            let grammar = load_grammar(people, &charts);
            assert!(
                grammar.is_ok(),
                "failed to load grammar for '{}': {:?}",
                people,
                grammar.err()
            );
        }
    }

    #[test]
    fn each_grammar_has_min_roots_and_suffixes() {
        let charts = crate::charts::load_charts().unwrap();
        for people in &["metsik", "arkit", "vayla", "laakso", "sepat", "ahjo"] {
            let grammar = load_grammar(people, &charts).unwrap();
            assert!(
                grammar.roots.len() >= 5,
                "{} has {} roots, need ≥5",
                people,
                grammar.roots.len()
            );
            assert!(
                grammar.suffixes.len() >= 3,
                "{} has {} suffixes, need ≥3",
                people,
                grammar.suffixes.len()
            );
        }
    }

    #[test]
    fn generate_deterministic() {
        let charts = crate::charts::load_charts().unwrap();
        let mut a = SeedRng::new(42);
        let mut b = SeedRng::new(42);
        for _ in 0..20 {
            let na = generate_name(&mut a, "metsik", "f", &charts).unwrap();
            let nb = generate_name(&mut b, "metsik", "f", &charts).unwrap();
            assert_eq!(na, nb);
        }
    }

    #[test]
    fn generate_produces_nontrivial_names() {
        let charts = crate::charts::load_charts().unwrap();
        let mut rng = SeedRng::new(77);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            let name = generate_name(&mut rng, "metsik", "f", &charts).unwrap();
            assert!(!name.is_empty());
            assert_ne!(name, "Unnamed");
            seen.insert(name);
        }
        assert!(
            seen.len() > 5,
            "only {} unique names in 100 draws",
            seen.len()
        );
    }

    #[test]
    fn grammar_roundtrip() {
        let charts = crate::charts::load_charts().unwrap();
        let grammar = load_grammar("metsik", &charts).unwrap();
        let ser = ron::ser::to_string(&grammar).unwrap();
        let de: NameGrammar = ron::from_str(&ser).unwrap();
        assert_eq!(grammar.roots, de.roots);
        assert_eq!(grammar.suffixes, de.suffixes);
        assert_eq!(grammar.prefixes, de.prefixes);
    }

    #[test]
    fn generated_names_are_capitalized() {
        let charts = crate::charts::load_charts().unwrap();
        let mut rng = SeedRng::new(42);
        for _ in 0..50 {
            let name = generate_name(&mut rng, "metsik", "f", &charts).unwrap();
            let first = name.chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "name '{}' starts with lowercase",
                name
            );
        }
    }

    #[test]
    fn within_people_uniqueness() {
        let charts = crate::charts::load_charts().unwrap();
        for people in &["metsik", "arkit", "vayla", "laakso", "sepat", "ahjo"] {
            let mut rng = SeedRng::new(42);
            let mut seen = std::collections::HashSet::new();
            for _ in 0..1000 {
                seen.insert(generate_name(&mut rng, people, "f", &charts).unwrap());
            }
            assert!(
                seen.len() > 300,
                "{} only produced {} unique names in 1000 draws",
                people,
                seen.len()
            );
        }
    }

    #[test]
    fn different_peoples_produce_distinct_name_sets() {
        let charts = crate::charts::load_charts().unwrap();
        let mut metsik_names = std::collections::HashSet::new();
        let mut rng = SeedRng::new(42);
        for _ in 0..200 {
            metsik_names.insert(generate_name(&mut rng, "metsik", "f", &charts).unwrap());
        }
        let mut sepat_names = std::collections::HashSet::new();
        let mut rng2 = SeedRng::new(42);
        for _ in 0..200 {
            sepat_names.insert(generate_name(&mut rng2, "sepat", "m", &charts).unwrap());
        }
        let overlap: usize = metsik_names.intersection(&sepat_names).count();
        assert!(
            overlap < 20,
            "metsik/sepat name overlap {} too high (expect distinct grammars)",
            overlap
        );
    }
}
