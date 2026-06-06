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
    /// Returns None if the table is empty.
    pub fn sample(&self, rng: &mut crate::rng::SeedRng) -> Option<String> {
        let total = self.total();
        if total == 0 {
            return None;
        }
        let mut roll = rng.gen_range(total);
        for (key, weight) in &self.entries {
            if roll < *weight {
                return Some(key.clone());
            }
            roll -= *weight;
        }
        // Fallback to last entry (shouldn't reach here)
        self.entries.keys().last().cloned()
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
