use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct World {
    pub seed: u64,
    pub tick: u64,
    pub regions: Vec<Region>,
    pub charts_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub region_type: String,
    pub description: String,
    pub settlements: Vec<Settlement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settlement {
    pub id: String,
    pub name: String,
    pub size: String,
    pub region: String,
    pub population: u32,
    pub description: String,
    pub people: Vec<Person>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Needs {
    pub food: f64,
    pub safety: f64,
    pub belonging: f64,
    pub esteem: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub social_class: String,
    pub craft_affinity: String,
    pub personality: Vec<String>,
    pub bias: String,
    pub needs: Needs,
    pub region: String,
    pub settlement: String,
    pub has_spouse: bool,
    pub children_count: u32,
    pub has_debt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Player {
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub craft_affinity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Household {
    pub head: Person,
    pub spouse: Option<Person>,
    pub children: Vec<Person>,
    pub has_debt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profession {
    pub name: String,
    pub people: String,
    pub base_weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Craft {
    pub name: String,
    pub base_weight: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
    >(
        value: &T,
    ) {
        let ser = ron::ser::to_string(value).unwrap();
        let de: T = ron::from_str(&ser).unwrap();
        assert_eq!(*value, de);
    }

    #[test]
    fn roundtrip_person() {
        let person = Person {
            id: "abc-123".into(),
            name: "Testi".into(),
            people: "metsik".into(),
            sex: "f".into(),
            age_band: "adult".into(),
            profession: "farmer".into(),
            social_class: "low".into(),
            craft_affinity: "none".into(),
            personality: vec!["stoic".into(), "curious".into()],
            bias: "metsik".into(),
            needs: Needs {
                food: 0.8,
                safety: 0.5,
                belonging: 0.6,
                esteem: 0.3,
            },
            region: "river_valley".into(),
            settlement: "hamlet-1".into(),
            has_spouse: true,
            children_count: 2,
            has_debt: false,
        };
        roundtrip(&person);
    }

    #[test]
    fn roundtrip_settlement() {
        let s = Settlement {
            id: "set-1".into(),
            name: "Test Village".into(),
            size: "village".into(),
            region: "river_valley".into(),
            population: 120,
            description: "A test village".into(),
            people: vec![],
        };
        roundtrip(&s);
    }

    #[test]
    fn roundtrip_region() {
        let r = Region {
            id: "reg-1".into(),
            name: "River Valley".into(),
            region_type: "river_valley".into(),
            description: "Fertile lowlands".into(),
            settlements: vec![],
        };
        roundtrip(&r);
    }

    #[test]
    fn roundtrip_world() {
        let w = World {
            seed: 42,
            tick: 0,
            regions: vec![],
            charts_version: "0.1.0".into(),
        };
        roundtrip(&w);
    }

    #[test]
    fn person_default_no_panic() {
        let p = Person::default();
        assert!(p.name.is_empty());
        assert!(p.personality.is_empty());
    }

    #[test]
    fn world_holds_many_persons() {
        let mut world = World::default();
        let region = Region {
            id: "r1".into(),
            name: "Test".into(),
            region_type: "river_valley".into(),
            description: "desc".into(),
            settlements: vec![Settlement {
                id: "s1".into(),
                name: "V".into(),
                size: "village".into(),
                region: "river_valley".into(),
                population: 10_000,
                description: "desc".into(),
                people: (0..10_000)
                    .map(|i| Person {
                        id: format!("p{}", i),
                        ..Default::default()
                    })
                    .collect(),
            }],
        };
        world.regions.push(region);
        let total: usize = world
            .regions
            .iter()
            .flat_map(|r| r.settlements.iter())
            .map(|s| s.people.len())
            .sum();
        assert_eq!(total, 10_000);
    }
}
