use serde::{Deserialize, Serialize};

/// Name grammars embedded at build time (like charts.ron), so name
/// generation works regardless of the process working directory — the
/// disk path in `load_grammar` only resolves when run from the crate root.
static EMBEDDED_GRAMMARS: &[(&str, &str)] = &[
    ("names/ahjo.ron", include_str!("../../data/names/ahjo.ron")),
    (
        "names/arkit.ron",
        include_str!("../../data/names/arkit.ron"),
    ),
    ("names/hal.ron", include_str!("../../data/names/hal.ron")),
    (
        "names/haramaki.ron",
        include_str!("../../data/names/haramaki.ron"),
    ),
    (
        "names/hiekkakavelijat.ron",
        include_str!("../../data/names/hiekkakavelijat.ron"),
    ),
    (
        "names/jamavaki.ron",
        include_str!("../../data/names/jamavaki.ron"),
    ),
    ("names/khor.ron", include_str!("../../data/names/khor.ron")),
    (
        "names/kirjakansa.ron",
        include_str!("../../data/names/kirjakansa.ron"),
    ),
    (
        "names/koskimetsa.ron",
        include_str!("../../data/names/koskimetsa.ron"),
    ),
    (
        "names/laakso.ron",
        include_str!("../../data/names/laakso.ron"),
    ),
    (
        "names/merak.ron",
        include_str!("../../data/names/merak.ron"),
    ),
    (
        "names/metsareunat.ron",
        include_str!("../../data/names/metsareunat.ron"),
    ),
    (
        "names/metsik.ron",
        include_str!("../../data/names/metsik.ron"),
    ),
    (
        "names/muistikansa.ron",
        include_str!("../../data/names/muistikansa.ron"),
    ),
    (
        "names/pohjavaki.ron",
        include_str!("../../data/names/pohjavaki.ron"),
    ),
    (
        "names/porokansa.ron",
        include_str!("../../data/names/porokansa.ron"),
    ),
    (
        "names/rantavaki.ron",
        include_str!("../../data/names/rantavaki.ron"),
    ),
    (
        "names/saarivaki.ron",
        include_str!("../../data/names/saarivaki.ron"),
    ),
    (
        "names/sepat.ron",
        include_str!("../../data/names/sepat.ron"),
    ),
    (
        "names/shear.ron",
        include_str!("../../data/names/shear.ron"),
    ),
    (
        "names/takovaki.ron",
        include_str!("../../data/names/takovaki.ron"),
    ),
    (
        "names/taulukansa.ron",
        include_str!("../../data/names/taulukansa.ron"),
    ),
    (
        "names/tzakhar.ron",
        include_str!("../../data/names/tzakhar.ron"),
    ),
    (
        "names/varhaiset.ron",
        include_str!("../../data/names/varhaiset.ron"),
    ),
    (
        "names/vayla.ron",
        include_str!("../../data/names/vayla.ron"),
    ),
];

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
    // Prefer the on-disk file (data stays editable without a rebuild), but fall
    // back to the embedded copy when the cwd isn't the crate root — e.g. the
    // Godot GDExtension binding runs from elsewhere, where the disk read failed
    // and every name silently became "Unnamed" / "Settlement {idx}".
    if let Ok(g) = NameGrammar::load(&path) {
        return Ok(g);
    }
    let embedded = EMBEDDED_GRAMMARS
        .iter()
        .find(|(p, _)| *p == rel_path)
        .ok_or_else(|| anyhow::anyhow!("no grammar on disk or embedded for '{}'", rel_path))?;
    Ok(ron::from_str::<NameGrammar>(embedded.1)?)
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
    fn every_grammar_is_embedded() {
        // Guards the cwd-independent path: each people's grammar must have an
        // embedded copy, or names silently fall back to "Unnamed" off-cwd.
        let charts = crate::charts::load_charts().unwrap();
        for (people, rel_path) in &charts.name_grammars {
            assert!(
                EMBEDDED_GRAMMARS.iter().any(|(p, _)| p == rel_path),
                "people '{}' grammar '{}' is not embedded",
                people,
                rel_path
            );
        }
    }

    #[test]
    fn embedded_grammars_parse() {
        for (path, content) in EMBEDDED_GRAMMARS {
            ron::from_str::<NameGrammar>(content)
                .unwrap_or_else(|e| panic!("embedded grammar {} failed to parse: {}", path, e));
        }
    }

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
