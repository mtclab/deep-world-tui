use crate::model::{Need, World};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Obligation {
    pub caregiver_id: String,
    pub dependent_id: String,
    pub need: Need,
    pub strength: f64,
}

pub fn propagate_dependent_needs(world: &mut World, obligations: &[Obligation]) {
    let mut extra_decay: indexmap::IndexMap<String, Vec<(Need, f64)>> = indexmap::IndexMap::new();
    for obl in obligations {
        let caregiver = find_person(world, &obl.caregiver_id);
        let caregiver_val = caregiver
            .as_ref()
            .map(|p| p.needs.get(obl.need))
            .unwrap_or(0.0);
        let deficit = (0.8 - caregiver_val).max(0.0);
        let absent = caregiver.as_ref().is_none_or(|p| {
            let dep = find_person(world, &obl.dependent_id);
            dep.as_ref().is_none_or(|d| p.settlement != d.settlement)
        });
        let mut rate = deficit * obl.strength * 0.05;
        if absent && obl.need == Need::Presence {
            rate += 0.04;
        }
        if rate > 0.0 {
            extra_decay
                .entry(obl.dependent_id.clone())
                .or_default()
                .push((obl.need, rate));
        }
    }
    for (dep_id, decays) in &extra_decay {
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    if person.id == *dep_id {
                        for (need, rate) in decays {
                            person.needs.decay(*need, *rate);
                        }
                    }
                }
            }
        }
    }
}

fn find_person(world: &World, id: &str) -> Option<crate::model::Person> {
    for region in &world.regions {
        for settlement in &region.settlements {
            for person in &settlement.people {
                if person.id == id {
                    return Some(person.clone());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Needs, Person};

    fn make_world_with(parent: Person, child: Person) -> World {
        World {
            seed: 0,
            tick: 0,
            regions: vec![crate::model::Region {
                id: "r1".into(),
                name: "R".into(),
                region_type: "river_valley".into(),
                region_subtype: "flood_plain".into(),
                description: String::new(),
                terrain: crate::model::TerrainMap::default(),
                neighbors: crate::model::RegionNeighbors::default(),
                structures: vec![],
                settlements: vec![crate::model::Settlement {
                    id: "s1".into(),
                    name: "S".into(),
                    size: "village".into(),
                    region: "r1".into(),
                    population: 2,
                    description: String::new(),
                    people: vec![parent, child],
                    services: vec![],
                    politics: crate::model::SettlementPolitics::new(),
                    food_stock: 0.0,
                    farms: Vec::new(),
                    buildings: Vec::new(),
                    festival_until_day: 0,
                }],
                weather: crate::model::Weather::Clear,
            }],
            charts_version: "0.1.0".into(),
            region_cols: 1,
        }
    }

    #[test]
    fn low_money_caregiver_child_money_decays_faster() {
        let parent = Person {
            id: "parent-1".into(),
            settlement: "s1".into(),
            needs: {
                let mut n = Needs::default();
                n.decay(Need::Money, 0.7);
                n
            },
            ..Default::default()
        };
        let child = Person {
            id: "child-1".into(),
            settlement: "s1".into(),
            needs: Needs::default(),
            ..Default::default()
        };
        let obligations = vec![Obligation {
            caregiver_id: "parent-1".into(),
            dependent_id: "child-1".into(),
            need: Need::Money,
            strength: 1.0,
        }];
        let mut world = make_world_with(parent.clone(), child.clone());
        let child_money_before = world.regions[0].settlements[0].people[1]
            .needs
            .get(Need::Money);
        propagate_dependent_needs(&mut world, &obligations);
        let child_money_after = world.regions[0].settlements[0].people[1]
            .needs
            .get(Need::Money);
        assert!(
            child_money_after < child_money_before,
            "child money should decay when parent money is low: before={}, after={}",
            child_money_before,
            child_money_after
        );
    }

    #[test]
    fn absent_caregiver_presence_low() {
        let parent = Person {
            id: "parent-1".into(),
            settlement: "s-other".into(),
            ..Default::default()
        };
        let child = Person {
            id: "child-1".into(),
            settlement: "s1".into(),
            needs: Needs::default(),
            ..Default::default()
        };
        let obligations = vec![Obligation {
            caregiver_id: "parent-1".into(),
            dependent_id: "child-1".into(),
            need: Need::Presence,
            strength: 1.0,
        }];
        let mut world = make_world_with(parent, child);
        let before = world.regions[0].settlements[0].people[1]
            .needs
            .get(Need::Presence);
        propagate_dependent_needs(&mut world, &obligations);
        let after = world.regions[0].settlements[0].people[1]
            .needs
            .get(Need::Presence);
        assert!(
            after < before,
            "child presence should decay when parent is absent: before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn satisfied_caregiver_no_extra_decay() {
        let parent = Person {
            id: "parent-1".into(),
            settlement: "s1".into(),
            needs: Needs::default(),
            ..Default::default()
        };
        let child = Person {
            id: "child-1".into(),
            settlement: "s1".into(),
            needs: Needs::default(),
            ..Default::default()
        };
        let obligations = vec![Obligation {
            caregiver_id: "parent-1".into(),
            dependent_id: "child-1".into(),
            need: Need::Money,
            strength: 1.0,
        }];
        let mut world = make_world_with(parent, child);
        let before = world.regions[0].settlements[0].people[1]
            .needs
            .get(Need::Money);
        propagate_dependent_needs(&mut world, &obligations);
        let after = world.regions[0].settlements[0].people[1]
            .needs
            .get(Need::Money);
        assert!(
            (before - after).abs() < f64::EPSILON,
            "satisfied caregiver should cause no extra decay: before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn propagation_deterministic() {
        let parent = Person {
            id: "parent-1".into(),
            settlement: "s1".into(),
            needs: {
                let mut n = Needs::default();
                n.decay(Need::Money, 0.5);
                n
            },
            ..Default::default()
        };
        let child = Person {
            id: "child-1".into(),
            settlement: "s1".into(),
            ..Default::default()
        };
        let obligations = vec![Obligation {
            caregiver_id: "parent-1".into(),
            dependent_id: "child-1".into(),
            need: Need::Money,
            strength: 1.0,
        }];
        let mut w1 = make_world_with(parent.clone(), child.clone());
        let mut w2 = make_world_with(parent, child);
        propagate_dependent_needs(&mut w1, &obligations);
        propagate_dependent_needs(&mut w2, &obligations);
        assert_eq!(
            w1.regions[0].settlements[0].people[1].needs,
            w2.regions[0].settlements[0].people[1].needs
        );
    }
}
