mod load;

use serde::Deserialize;
use std::collections::HashMap;

/// A simple weighted table: pick a key by relative weight.
#[derive(Debug, Clone, Deserialize)]
pub struct WeightedTable {
    pub entries: HashMap<String, u32>,
}

impl WeightedTable {
    pub fn total(&self) -> u32 {
        self.entries.values().sum()
    }

    /// Pick a key from the table using the provided RNG.
    /// Returns None if the table is empty or all weights are zero.
    /// Zero-weight entries are skipped. Iterates in sorted key order for determinism.
    pub fn sample(&self, rng: &mut crate::rng::SeedRng) -> Option<String> {
        let mut sorted: Vec<(&String, u32)> = self
            .entries
            .iter()
            .filter(|(_, w)| **w > 0)
            .map(|(k, w)| (k, *w))
            .collect();
        sorted.sort_by_key(|(k, _)| *k);
        let total: u32 = sorted.iter().map(|(_, w)| *w).sum();
        if total == 0 {
            return None;
        }
        let mut roll = rng.gen_range(total);
        for (key, weight) in &sorted {
            if roll < *weight {
                return Some((*key).clone());
            }
            roll -= weight;
        }
        sorted.last().map(|(k, _)| (*k).clone())
    }
}

/// A condition for the conditional table modifiers.
#[derive(Debug, Clone, Deserialize)]
pub enum Condition {
    People(String),
    Region(String),
    Class(String),
    Settlement(String),
}

/// A modifier entry: when condition matches, multiply the listed keys' weights.
#[derive(Debug, Clone, Deserialize)]
pub struct Modifier {
    pub when: Condition,
    pub mult: HashMap<String, f64>,
}

/// A conditional weighted table: base distribution plus context-dependent modifiers.
#[derive(Debug, Clone, Deserialize)]
pub struct ConditionalTable {
    pub base: WeightedTable,
    pub modifiers: Vec<Modifier>,
}

impl ConditionalTable {
    /// Resolve the effective weights given a context (people, region, class, settlement).
    pub fn resolve(
        &self,
        people: &str,
        region: &str,
        class: &str,
        settlement_size: &str,
    ) -> HashMap<String, u32> {
        let mut result = self.base.entries.clone();
        for modifier in &self.modifiers {
            let matches = match &modifier.when {
                Condition::People(p) => p == people,
                Condition::Region(r) => r == region,
                Condition::Class(c) => c == class,
                Condition::Settlement(s) => s == settlement_size,
            };
            if matches {
                for (key, mult) in &modifier.mult {
                    if let Some(weight) = result.get(key) {
                        let new_weight = (*weight as f64 * mult).round() as u32;
                        result.insert(key.clone(), new_weight.max(1));
                    }
                }
            }
        }
        result
    }
}

/// The top-level charts loaded from data/charts.ron.
#[derive(Debug, Clone, Deserialize)]
pub struct Charts {
    pub people: WeightedTable,
    pub region: WeightedTable,
    pub settlement_size: WeightedTable,
    pub social_class: WeightedTable,
    pub profession: ConditionalTable,
    pub craft_affinity: ConditionalTable,
    pub personality_traits: WeightedTable,
    pub has_spouse: WeightedTable,
    pub children_count: WeightedTable,
    pub has_debt: WeightedTable,
    pub age_band: WeightedTable,
    pub sex: WeightedTable,
    pub name_grammars: HashMap<String, String>,
}

pub use load::load_charts;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SeedRng;

    #[test]
    fn sample_empty_table_returns_none() {
        let table = WeightedTable {
            entries: HashMap::new(),
        };
        let mut rng = SeedRng::new(42);
        assert!(table.sample(&mut rng).is_none());
    }

    #[test]
    fn sample_single_entry_always_returns_it() {
        let mut entries = HashMap::new();
        entries.insert("only".into(), 10u32);
        let table = WeightedTable { entries };
        let mut rng = SeedRng::new(42);
        for _ in 0..100 {
            assert_eq!(table.sample(&mut rng).as_deref(), Some("only"));
        }
    }

    #[test]
    fn sample_all_zero_weights_returns_none() {
        let mut entries = HashMap::new();
        entries.insert("a".into(), 0u32);
        entries.insert("b".into(), 0u32);
        let table = WeightedTable { entries };
        let mut rng = SeedRng::new(42);
        assert!(table.sample(&mut rng).is_none());
    }

    #[test]
    fn sample_deterministic_same_seed() {
        let charts = load::load_charts("data/charts.ron").unwrap();
        let mut a = SeedRng::new(42);
        let mut b = SeedRng::new(42);
        for _ in 0..50 {
            assert_eq!(charts.people.sample(&mut a), charts.people.sample(&mut b));
        }
    }

    #[test]
    fn sample_people_distribution_covers_all_keys() {
        let charts = load::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(77);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..10_000 {
            if let Some(k) = charts.people.sample(&mut rng) {
                seen.insert(k);
            }
        }
        for key in charts.people.entries.keys() {
            if charts.people.entries[key] > 0 {
                assert!(
                    seen.contains(key),
                    "people key '{}' never sampled in 10k draws",
                    key
                );
            }
        }
    }

    #[test]
    fn sample_people_distribution_sane_ratios() {
        let charts = load::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(99);
        let mut counts = HashMap::new();
        let n = 10_000usize;
        for _ in 0..n {
            if let Some(k) = charts.people.sample(&mut rng) {
                *counts.entry(k).or_insert(0usize) += 1;
            }
        }
        let total_weight = charts.people.total() as f64;
        for (key, weight) in &charts.people.entries {
            if *weight == 0 {
                continue;
            }
            let expected = (*weight as f64 / total_weight) * n as f64;
            let actual = counts.get(key).copied().unwrap_or(0) as f64;
            let ratio = actual / expected;
            assert!(
                ratio > 0.33 && ratio < 3.0,
                "people '{}' ratio {:.2} out of range [0.33, 3.0] (actual={}, expected={:.0})",
                key,
                ratio,
                actual as usize,
                expected
            );
        }
    }

    #[test]
    fn sample_skips_zero_weight() {
        let mut entries = HashMap::new();
        entries.insert("zero".into(), 0u32);
        entries.insert("one".into(), 1u32);
        let table = WeightedTable { entries };
        let mut rng = SeedRng::new(42);
        for _ in 0..50 {
            assert_eq!(table.sample(&mut rng).as_deref(), Some("one"));
        }
    }
}
