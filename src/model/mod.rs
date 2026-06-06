use serde::{Deserialize, Serialize};

/// Plain data types for the game world. All are serde-serialisable.
/// Stubs for issue #3; fleshed out in later issues.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct World {
    pub seed: u64,
    pub regions: Vec<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub name: String,
    pub region_type: String,
    pub settlements: Vec<Settlement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub name: String,
    pub size: String,
    pub region: String,
    pub people: Vec<Person>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Person {
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub social_class: String,
    pub craft_affinity: String,
    pub personality: Vec<String>,
    pub has_spouse: bool,
    pub children_count: u32,
    pub has_debt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Player {
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub craft_affinity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Household {
    pub head: Person,
    pub spouse: Option<Person>,
    pub children: Vec<Person>,
    pub has_debt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profession {
    pub name: String,
    pub people: String,
    pub base_weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Craft {
    pub name: String,
    pub base_weight: u32,
}
