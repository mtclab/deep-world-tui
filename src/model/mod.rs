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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Need {
    Food,
    Safety,
    Belonging,
    Esteem,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Needs {
    pub food: f64,
    pub safety: f64,
    pub belonging: f64,
    pub esteem: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftAffinity {
    #[default]
    None,
    Word,
    Current,
    Still,
    Forge,
    Root,
}

impl CraftAffinity {
    pub fn from_chart_key(key: &str) -> Option<Self> {
        match key {
            "none" => Some(Self::None),
            "word" => Some(Self::Word),
            "current" => Some(Self::Current),
            "still" => Some(Self::Still),
            "forge" => Some(Self::Forge),
            "root" => Some(Self::Root),
            _ => None,
        }
    }

    pub fn to_chart_key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Word => "word",
            Self::Current => "current",
            Self::Still => "still",
            Self::Forge => "forge",
            Self::Root => "root",
        }
    }
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
    pub id: String,
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub social_class: String,
    pub craft_affinity: CraftAffinity,
    pub personality: Vec<String>,
    pub region: String,
    pub settlement: String,
    pub perks: Vec<Perk>,
    pub household_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Perk {
    pub name: String,
    pub description: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerStart {
    pub person: Person,
    pub reroll_count: u32,
    pub point_buy_adjustments: Vec<Adjustment>,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Adjustment {
    SwapProfession(String),
    SetCraft(CraftAffinity),
    AddPerk(Perk),
    CutHouseholdTie,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Debt {
    pub creditor_id: String,
    pub amount: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Household {
    pub id: String,
    pub head_id: String,
    pub spouse_id: Option<String>,
    pub children_ids: Vec<String>,
    pub location_settlement_id: String,
    pub has_debt: bool,
    pub debts: Vec<Debt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipKind {
    Spouse,
    Parent,
    Child,
    Sibling,
    Kin,
    Friend,
    Rival,
    Patron,
    Apprentice,
    Guildmate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipEvent {
    pub tick: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    pub from_id: String,
    pub to_id: String,
    pub kind: RelationshipKind,
    pub strength: f64,
    pub trust: f64,
    pub history: Vec<RelationshipEvent>,
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

    #[test]
    fn roundtrip_household() {
        let h = Household {
            id: "hh-1".into(),
            head_id: "p1".into(),
            spouse_id: Some("p2".into()),
            children_ids: vec!["p3".into(), "p4".into()],
            location_settlement_id: "set-1".into(),
            has_debt: true,
            debts: vec![Debt {
                creditor_id: "p5".into(),
                amount: 50.0,
                description: "seed loan".into(),
            }],
        };
        roundtrip(&h);
    }

    #[test]
    fn roundtrip_relationship() {
        let r = Relationship {
            from_id: "p1".into(),
            to_id: "p2".into(),
            kind: RelationshipKind::Friend,
            strength: 0.7,
            trust: 0.5,
            history: vec![RelationshipEvent {
                tick: 10,
                description: "shared a meal".into(),
            }],
        };
        roundtrip(&r);
    }

    #[test]
    fn craft_affinity_roundtrip_chart_keys() {
        for key in &["none", "word", "current", "still", "forge", "root"] {
            let affinity = CraftAffinity::from_chart_key(key).unwrap();
            assert_eq!(affinity.to_chart_key(), *key);
        }
        roundtrip(&CraftAffinity::Forge);
    }

    #[test]
    fn relationship_kind_all_variants() {
        let variants = [
            RelationshipKind::Spouse,
            RelationshipKind::Parent,
            RelationshipKind::Child,
            RelationshipKind::Sibling,
            RelationshipKind::Kin,
            RelationshipKind::Friend,
            RelationshipKind::Rival,
            RelationshipKind::Patron,
            RelationshipKind::Apprentice,
            RelationshipKind::Guildmate,
        ];
        for v in &variants {
            roundtrip(v);
        }
    }

    #[test]
    fn need_enum_roundtrip() {
        roundtrip(&Need::Food);
        roundtrip(&Need::Esteem);
    }

    #[test]
    fn roundtrip_player() {
        let p = Player {
            id: "player-1".into(),
            name: "Hero".into(),
            people: "metsik".into(),
            sex: "m".into(),
            age_band: "youth".into(),
            profession: "forester".into(),
            social_class: "low".into(),
            craft_affinity: CraftAffinity::Root,
            personality: vec!["curious".into()],
            region: "forest".into(),
            settlement: "set-1".into(),
            perks: vec![Perk {
                name: "Keen Eye".into(),
                description: "Spot details others miss".into(),
                source: "personality_traits".into(),
            }],
            household_id: Some("hh-1".into()),
        };
        roundtrip(&p);
    }

    #[test]
    fn player_default_valid() {
        let p = Player::default();
        assert!(p.name.is_empty());
        assert!(p.perks.is_empty());
        assert!(p.household_id.is_none());
    }

    #[test]
    fn roundtrip_player_start() {
        let ps = PlayerStart {
            person: Person::default(),
            reroll_count: 2,
            point_buy_adjustments: vec![
                Adjustment::SwapProfession("trader".into()),
                Adjustment::SetCraft(CraftAffinity::Current),
                Adjustment::AddPerk(Perk {
                    name: "Silver Tongue".into(),
                    description: "Better trade deals".into(),
                    source: "profession".into(),
                }),
                Adjustment::CutHouseholdTie,
            ],
            accepted: false,
        };
        roundtrip(&ps);
    }

    #[test]
    fn adjustment_all_variants_roundtrip() {
        roundtrip(&Adjustment::SwapProfession("smith".into()));
        roundtrip(&Adjustment::SetCraft(CraftAffinity::Forge));
        roundtrip(&Adjustment::AddPerk(Perk {
            name: "test".into(),
            description: "test perk".into(),
            source: "test".into(),
        }));
        roundtrip(&Adjustment::CutHouseholdTie);
    }
}
