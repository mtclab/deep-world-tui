use crate::model::{Need, World};
use crate::sim::relationships::RelationshipTracker;
use crate::sim::reputation::ReputationStore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Change {
    NeedDelta {
        person_id: String,
        need: Need,
        delta: f64,
    },
    ReputationDelta {
        person_id: String,
        settlement: String,
        delta: f64,
    },
    RelationshipDelta {
        from: String,
        to: String,
        strength_delta: f64,
        trust_delta: f64,
    },
    MovePerson {
        person_id: String,
        to_settlement: String,
    },
    AddObligation {
        debtor: String,
        creditor: String,
        amount: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Effect {
    Immediate {
        description: String,
        changes: Vec<Change>,
    },
    Deferred {
        at_tick: u64,
        description: String,
        changes: Vec<Change>,
    },
}

impl Effect {
    pub fn immediate(desc: &str, changes: Vec<Change>) -> Self {
        Effect::Immediate {
            description: desc.to_string(),
            changes,
        }
    }

    pub fn deferred(desc: &str, at_tick: u64, changes: Vec<Change>) -> Self {
        Effect::Deferred {
            at_tick,
            description: desc.to_string(),
            changes,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EffectQueue {
    scheduled: BTreeMap<u64, Vec<Effect>>,
}

impl EffectQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue(&mut self, effect: Effect) {
        if let Effect::Deferred { at_tick, .. } = &effect {
            self.scheduled.entry(*at_tick).or_default().push(effect);
        }
    }

    pub fn due(&mut self, tick: u64) -> Vec<Effect> {
        let mut due_effects = Vec::new();
        let ticks: Vec<u64> = self
            .scheduled
            .keys()
            .copied()
            .filter(|&t| t <= tick)
            .collect();
        for t in ticks {
            if let Some(effects) = self.scheduled.remove(&t) {
                due_effects.extend(effects);
            }
        }
        due_effects
    }

    pub fn is_empty(&self) -> bool {
        self.scheduled.is_empty()
    }

    pub fn len(&self) -> usize {
        self.scheduled.values().map(|v| v.len()).sum()
    }
}

pub struct EffectContext<'a> {
    pub world: &'a mut World,
    pub relationships: &'a mut RelationshipTracker,
    pub reputation: &'a mut ReputationStore,
    pub current_tick: u64,
}

pub fn apply_effect(ctx: &mut EffectContext, effect: &Effect) {
    let changes = match effect {
        Effect::Immediate { changes, .. } => changes,
        Effect::Deferred { .. } => return,
    };
    apply_changes(ctx, changes);
}

pub fn apply_changes(ctx: &mut EffectContext, changes: &[Change]) {
    for change in changes {
        apply_change(ctx, change);
    }
}

fn apply_change(ctx: &mut EffectContext, change: &Change) {
    match change {
        Change::NeedDelta {
            person_id,
            need,
            delta,
        } => {
            apply_need_delta(ctx.world, person_id, *need, *delta);
        }
        Change::ReputationDelta {
            person_id,
            settlement,
            delta,
        } => {
            ctx.reputation.adjust_local(person_id, settlement, *delta);
        }
        Change::RelationshipDelta {
            from,
            to,
            strength_delta,
            trust_delta,
        } => {
            ctx.relationships.update_relationship(
                from,
                to,
                "effect",
                ctx.current_tick,
                *strength_delta,
                *trust_delta,
            );
        }
        Change::MovePerson {
            person_id,
            to_settlement,
        } => {
            apply_move_person(ctx.world, person_id, to_settlement);
        }
        Change::AddObligation {
            debtor,
            creditor,
            amount,
        } => {
            apply_add_obligation(ctx.world, debtor, creditor, *amount);
        }
    }
}

fn apply_need_delta(world: &mut World, person_id: &str, need: Need, delta: f64) {
    for region in &mut world.regions {
        for settlement in &mut region.settlements {
            for person in &mut settlement.people {
                if person.id == person_id {
                    person.needs.satisfy(need, delta);
                    return;
                }
            }
        }
    }
}

fn apply_move_person(world: &mut World, person_id: &str, to_settlement: &str) {
    let mut person_opt: Option<crate::model::Person> = None;
    for region in &world.regions {
        for settlement in &region.settlements {
            for p in &settlement.people {
                if p.id == person_id {
                    person_opt = Some(p.clone());
                    break;
                }
            }
            if person_opt.is_some() {
                break;
            }
        }
        if person_opt.is_some() {
            break;
        }
    }
    if let Some(mut person) = person_opt {
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                settlement.people.retain(|p| p.id != person_id);
            }
        }
        person.settlement = to_settlement.to_string();
        if let Some(target_region_id) = find_region_for_settlement(world, to_settlement) {
            for region in &mut world.regions {
                if region.id == target_region_id {
                    for settlement in &mut region.settlements {
                        if settlement.id == to_settlement {
                            settlement.people.push(person);
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn find_region_for_settlement(world: &World, settlement_id: &str) -> Option<String> {
    for region in &world.regions {
        for settlement in &region.settlements {
            if settlement.id == settlement_id {
                return Some(region.id.clone());
            }
        }
    }
    None
}

fn apply_add_obligation(world: &mut World, debtor: &str, _creditor: &str, _amount: f64) {
    for region in &mut world.regions {
        for settlement in &mut region.settlements {
            for person in &mut settlement.people {
                if person.id == debtor {
                    person.has_debt = true;
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Needs, Person};

    fn make_simple_world() -> World {
        World {
            seed: 0,
            tick: 0,
            regions: vec![crate::model::Region {
                id: "r1".into(),
                name: "R".into(),
                region_type: "river_valley".into(),
                description: String::new(),
                settlements: vec![crate::model::Settlement {
                    id: "s1".into(),
                    name: "S1".into(),
                    size: "village".into(),
                    region: "r1".into(),
                    population: 2,
                    description: String::new(),
                    people: vec![
                        Person {
                            id: "p1".into(),
                            settlement: "s1".into(),
                            needs: Needs::default(),
                            ..Default::default()
                        },
                        Person {
                            id: "p2".into(),
                            settlement: "s1".into(),
                            needs: Needs::default(),
                            ..Default::default()
                        },
                    ],
                }],
            }],
            charts_version: "0.1.0".into(),
        }
    }

    fn make_context<'a>(
        world: &'a mut World,
        relationships: &'a mut RelationshipTracker,
        reputation: &'a mut ReputationStore,
    ) -> EffectContext<'a> {
        EffectContext {
            world,
            relationships,
            reputation,
            current_tick: 0,
        }
    }

    #[test]
    fn immediate_need_delta_updates_person() {
        let mut world = make_simple_world();
        let mut rels = RelationshipTracker::new();
        let mut rep = ReputationStore::new();
        let mut ctx = make_context(&mut world, &mut rels, &mut rep);
        let before = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        apply_effect(
            &mut ctx,
            &Effect::immediate(
                "fed",
                vec![Change::NeedDelta {
                    person_id: "p1".into(),
                    need: Need::Food,
                    delta: 0.1,
                }],
            ),
        );
        let after = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            after > before,
            "food should increase: before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn immediate_reputation_delta() {
        let mut world = make_simple_world();
        let mut rels = RelationshipTracker::new();
        let mut rep = ReputationStore::new();
        let mut ctx = make_context(&mut world, &mut rels, &mut rep);
        apply_effect(
            &mut ctx,
            &Effect::immediate(
                "good deed",
                vec![Change::ReputationDelta {
                    person_id: "p1".into(),
                    settlement: "s1".into(),
                    delta: 0.2,
                }],
            ),
        );
        let rep_val = ctx.reputation.get("p1", "s1");
        assert!(rep_val > 0.5, "reputation should increase: {}", rep_val);
    }

    #[test]
    fn immediate_relationship_delta() {
        let mut world = make_simple_world();
        let mut rels = RelationshipTracker::new();
        let mut rep = ReputationStore::new();
        let mut ctx = make_context(&mut world, &mut rels, &mut rep);
        apply_effect(
            &mut ctx,
            &Effect::immediate(
                "helped",
                vec![Change::RelationshipDelta {
                    from: "p1".into(),
                    to: "p2".into(),
                    strength_delta: 0.3,
                    trust_delta: 0.1,
                }],
            ),
        );
        let rel = ctx.relationships.get("p1", "p2").unwrap();
        assert!(rel.strength > 0.0, "relationship strength should increase");
        assert!(rel.trust > 0.5, "trust should increase above baseline");
    }

    #[test]
    fn immediate_move_person() {
        let mut world = World {
            seed: 0,
            tick: 0,
            regions: vec![
                crate::model::Region {
                    id: "r1".into(),
                    name: "R1".into(),
                    region_type: "river_valley".into(),
                    description: String::new(),
                    settlements: vec![crate::model::Settlement {
                        id: "s1".into(),
                        name: "S1".into(),
                        size: "village".into(),
                        region: "r1".into(),
                        population: 1,
                        description: String::new(),
                        people: vec![Person {
                            id: "p1".into(),
                            settlement: "s1".into(),
                            ..Default::default()
                        }],
                    }],
                },
                crate::model::Region {
                    id: "r2".into(),
                    name: "R2".into(),
                    region_type: "forest".into(),
                    description: String::new(),
                    settlements: vec![crate::model::Settlement {
                        id: "s2".into(),
                        name: "S2".into(),
                        size: "hamlet".into(),
                        region: "r2".into(),
                        population: 0,
                        description: String::new(),
                        people: vec![],
                    }],
                },
            ],
            charts_version: "0.1.0".into(),
        };
        let mut rels = RelationshipTracker::new();
        let mut rep = ReputationStore::new();
        let mut ctx = make_context(&mut world, &mut rels, &mut rep);
        apply_effect(
            &mut ctx,
            &Effect::immediate(
                "moved",
                vec![Change::MovePerson {
                    person_id: "p1".into(),
                    to_settlement: "s2".into(),
                }],
            ),
        );
        assert!(
            ctx.world.regions[0].settlements[0]
                .people
                .iter()
                .all(|p| p.id != "p1"),
            "p1 should be removed from s1"
        );
        let s2_people: Vec<&Person> = ctx.world.regions[1].settlements[0]
            .people
            .iter()
            .filter(|p| p.id == "p1")
            .collect();
        assert_eq!(s2_people.len(), 1, "p1 should appear in s2");
        assert_eq!(s2_people[0].settlement, "s2");
    }

    #[test]
    fn immediate_add_obligation() {
        let mut world = make_simple_world();
        let mut rels = RelationshipTracker::new();
        let mut rep = ReputationStore::new();
        let mut ctx = make_context(&mut world, &mut rels, &mut rep);
        assert!(!ctx.world.regions[0].settlements[0].people[0].has_debt);
        apply_effect(
            &mut ctx,
            &Effect::immediate(
                "loan",
                vec![Change::AddObligation {
                    debtor: "p1".into(),
                    creditor: "p2".into(),
                    amount: 50.0,
                }],
            ),
        );
        assert!(
            ctx.world.regions[0].settlements[0].people[0].has_debt,
            "debtor should have debt flag"
        );
    }

    #[test]
    fn deferred_effect_not_applied_immediately() {
        let mut world = make_simple_world();
        let mut rels = RelationshipTracker::new();
        let mut rep = ReputationStore::new();
        let mut ctx = make_context(&mut world, &mut rels, &mut rep);
        let effect = Effect::deferred(
            "harvest",
            5,
            vec![Change::NeedDelta {
                person_id: "p1".into(),
                need: Need::Food,
                delta: 0.2,
            }],
        );
        let before = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        apply_effect(&mut ctx, &effect);
        let after = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            (before - after).abs() < f64::EPSILON,
            "deferred effect should not apply immediately"
        );
    }

    #[test]
    fn effect_queue_stores_deferred() {
        let mut queue = EffectQueue::new();
        let effect = Effect::deferred(
            "delayed",
            10,
            vec![Change::NeedDelta {
                person_id: "p1".into(),
                need: Need::Food,
                delta: 0.1,
            }],
        );
        queue.queue(effect);
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[test]
    fn effect_queue_due_returns_at_tick() {
        let mut queue = EffectQueue::new();
        queue.queue(Effect::deferred(
            "a",
            5,
            vec![Change::NeedDelta {
                person_id: "p1".into(),
                need: Need::Food,
                delta: 0.1,
            }],
        ));
        queue.queue(Effect::deferred(
            "b",
            10,
            vec![Change::NeedDelta {
                person_id: "p2".into(),
                need: Need::Money,
                delta: 0.1,
            }],
        ));
        let due_at_5 = queue.due(5);
        assert_eq!(due_at_5.len(), 1);
        assert_eq!(queue.len(), 1);
        let due_at_10 = queue.due(10);
        assert_eq!(due_at_10.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn effect_queue_due_catches_earlier_ticks() {
        let mut queue = EffectQueue::new();
        queue.queue(Effect::deferred(
            "a",
            3,
            vec![Change::NeedDelta {
                person_id: "p1".into(),
                need: Need::Food,
                delta: 0.1,
            }],
        ));
        queue.queue(Effect::deferred(
            "b",
            5,
            vec![Change::NeedDelta {
                person_id: "p2".into(),
                need: Need::Money,
                delta: 0.1,
            }],
        ));
        let due_at_7 = queue.due(7);
        assert_eq!(due_at_7.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn effect_queue_multiple_at_same_tick() {
        let mut queue = EffectQueue::new();
        queue.queue(Effect::deferred(
            "a",
            5,
            vec![Change::NeedDelta {
                person_id: "p1".into(),
                need: Need::Food,
                delta: 0.1,
            }],
        ));
        queue.queue(Effect::deferred(
            "b",
            5,
            vec![Change::NeedDelta {
                person_id: "p2".into(),
                need: Need::Money,
                delta: 0.1,
            }],
        ));
        let due = queue.due(5);
        assert_eq!(due.len(), 2);
    }

    #[test]
    fn effect_deterministic() {
        let mut world1 = make_simple_world();
        let mut world2 = make_simple_world();
        let mut rels1 = RelationshipTracker::new();
        let mut rels2 = RelationshipTracker::new();
        let mut rep1 = ReputationStore::new();
        let mut rep2 = ReputationStore::new();
        let mut ctx1 = make_context(&mut world1, &mut rels1, &mut rep1);
        let mut ctx2 = make_context(&mut world2, &mut rels2, &mut rep2);
        let effect = Effect::immediate(
            "test",
            vec![
                Change::NeedDelta {
                    person_id: "p1".into(),
                    need: Need::Food,
                    delta: 0.15,
                },
                Change::RelationshipDelta {
                    from: "p1".into(),
                    to: "p2".into(),
                    strength_delta: 0.1,
                    trust_delta: 0.05,
                },
                Change::ReputationDelta {
                    person_id: "p1".into(),
                    settlement: "s1".into(),
                    delta: 0.2,
                },
            ],
        );
        apply_effect(&mut ctx1, &effect);
        apply_effect(&mut ctx2, &effect);
        let food1 = ctx1.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let food2 = ctx2.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            (food1 - food2).abs() < f64::EPSILON,
            "deterministic: food1={}, food2={}",
            food1,
            food2
        );
        let rel1 = ctx1.relationships.get("p1", "p2").unwrap().clone();
        let rel2 = ctx2.relationships.get("p1", "p2").unwrap().clone();
        assert_eq!(rel1, rel2, "relationship must be deterministic");
        assert_eq!(
            ctx1.reputation.get("p1", "s1"),
            ctx2.reputation.get("p1", "s1"),
            "reputation must be deterministic"
        );
    }

    #[test]
    fn change_roundtrip_serde() {
        let changes = vec![
            Change::NeedDelta {
                person_id: "p1".into(),
                need: Need::Food,
                delta: 0.2,
            },
            Change::ReputationDelta {
                person_id: "p1".into(),
                settlement: "s1".into(),
                delta: -0.1,
            },
            Change::RelationshipDelta {
                from: "p1".into(),
                to: "p2".into(),
                strength_delta: 0.3,
                trust_delta: 0.1,
            },
            Change::MovePerson {
                person_id: "p1".into(),
                to_settlement: "s2".into(),
            },
            Change::AddObligation {
                debtor: "p1".into(),
                creditor: "p2".into(),
                amount: 50.0,
            },
        ];
        let effect = Effect::immediate("test", changes);
        let ser = ron::ser::to_string(&effect).unwrap();
        let de: Effect = ron::from_str(&ser).unwrap();
        assert_eq!(effect, de);
    }

    #[test]
    fn effect_queue_roundtrip_serde() {
        let mut queue = EffectQueue::new();
        queue.queue(Effect::deferred(
            "delayed",
            5,
            vec![Change::NeedDelta {
                person_id: "p1".into(),
                need: Need::Food,
                delta: 0.1,
            }],
        ));
        let ser = ron::ser::to_string(&queue).unwrap();
        let de: EffectQueue = ron::from_str(&ser).unwrap();
        assert_eq!(queue, de);
    }

    #[test]
    fn multiple_changes_in_one_effect() {
        let mut world = make_simple_world();
        let mut rels = RelationshipTracker::new();
        let mut rep = ReputationStore::new();
        let mut ctx = make_context(&mut world, &mut rels, &mut rep);
        let food_before = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let money_before = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Money);
        apply_effect(
            &mut ctx,
            &Effect::immediate(
                "feast",
                vec![
                    Change::NeedDelta {
                        person_id: "p1".into(),
                        need: Need::Food,
                        delta: 0.2,
                    },
                    Change::NeedDelta {
                        person_id: "p1".into(),
                        need: Need::Money,
                        delta: -0.1,
                    },
                ],
            ),
        );
        let food_after = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let money_after = ctx.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Money);
        assert!(food_after > food_before, "food should increase");
        assert!(money_after < money_before, "money should decrease");
    }
}
