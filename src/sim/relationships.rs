use crate::model::{Relationship, RelationshipEvent, RelationshipKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const TRUST_BASELINE: f64 = 0.5;
const TRUST_CONVERGENCE_RATE: f64 = 0.005;
const MAX_HISTORY: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondCategory {
    Stranger,
    Acquaintance,
    Friend,
    Kin,
    Bonded,
}

impl BondCategory {
    pub fn from_strength(strength: f64) -> Self {
        if strength >= 0.9 {
            BondCategory::Bonded
        } else if strength >= 0.7 {
            BondCategory::Kin
        } else if strength >= 0.4 {
            BondCategory::Friend
        } else if strength >= 0.2 {
            BondCategory::Acquaintance
        } else {
            BondCategory::Stranger
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipTracker {
    pub relationships: HashMap<(String, String), Relationship>,
}

impl RelationshipTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, from: &str, to: &str) -> Option<&Relationship> {
        self.relationships.get(&(from.to_string(), to.to_string()))
    }

    pub fn get_mut(&mut self, from: &str, to: &str) -> Option<&mut Relationship> {
        self.relationships
            .get_mut(&(from.to_string(), to.to_string()))
    }

    pub fn get_or_create(&mut self, from: &str, to: &str) -> &mut Relationship {
        let key = (from.to_string(), to.to_string());
        self.relationships
            .entry(key)
            .or_insert_with(|| Relationship {
                from_id: from.to_string(),
                to_id: to.to_string(),
                kind: RelationshipKind::Friend,
                strength: 0.0,
                trust: TRUST_BASELINE,
                history: Vec::new(),
            })
    }

    pub fn update_relationship(
        &mut self,
        from: &str,
        to: &str,
        event: &str,
        tick: u64,
        strength_delta: f64,
        trust_delta: f64,
    ) {
        let rel = self.get_or_create(from, to);
        rel.strength = (rel.strength + strength_delta).clamp(0.0, 1.0);
        rel.trust = (rel.trust + trust_delta).clamp(0.0, 1.0);
        if rel.history.len() >= MAX_HISTORY {
            rel.history.remove(0);
        }
        rel.history.push(RelationshipEvent {
            tick,
            description: event.to_string(),
        });
    }

    pub fn tick_converge(&mut self, dt: f64) {
        for rel in self.relationships.values_mut() {
            if rel.trust > TRUST_BASELINE {
                rel.trust = (rel.trust - TRUST_CONVERGENCE_RATE * dt).max(TRUST_BASELINE);
            } else if rel.trust < TRUST_BASELINE {
                rel.trust = (rel.trust + TRUST_CONVERGENCE_RATE * dt).min(TRUST_BASELINE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bond_category_stranger() {
        assert_eq!(BondCategory::from_strength(0.1), BondCategory::Stranger);
    }

    #[test]
    fn bond_category_acquaintance() {
        assert_eq!(BondCategory::from_strength(0.3), BondCategory::Acquaintance);
    }

    #[test]
    fn bond_category_friend() {
        assert_eq!(BondCategory::from_strength(0.5), BondCategory::Friend);
    }

    #[test]
    fn bond_category_kin() {
        assert_eq!(BondCategory::from_strength(0.8), BondCategory::Kin);
    }

    #[test]
    fn bond_category_bonded() {
        assert_eq!(BondCategory::from_strength(0.95), BondCategory::Bonded);
    }

    #[test]
    fn tracker_update_helped_event() {
        let mut tracker = RelationshipTracker::new();
        tracker.update_relationship("p1", "p2", "helped with harvest", 10, 0.1, 0.05);
        let rel = tracker.get("p1", "p2").unwrap();
        assert!(
            rel.strength > 0.0,
            "bond should increase after helping: {}",
            rel.strength
        );
        assert!(
            rel.trust > TRUST_BASELINE,
            "trust should increase: {}",
            rel.trust
        );
        assert_eq!(rel.history.len(), 1);
        assert_eq!(rel.history[0].description, "helped with harvest");
    }

    #[test]
    fn tracker_update_betrayed_event() {
        let mut tracker = RelationshipTracker::new();
        tracker.update_relationship("p1", "p2", "stole grain", 5, -0.3, -0.2);
        let rel = tracker.get("p1", "p2").unwrap();
        assert!(
            rel.strength < 0.01,
            "bond should be near zero after betrayal"
        );
        assert!(
            rel.trust < TRUST_BASELINE,
            "trust should decrease: {}",
            rel.trust
        );
    }

    #[test]
    fn tracker_history_capped_at_20() {
        let mut tracker = RelationshipTracker::new();
        for i in 0..25u64 {
            tracker.update_relationship("p1", "p2", "event", i, 0.01, 0.01);
        }
        let rel = tracker.get("p1", "p2").unwrap();
        assert_eq!(rel.history.len(), MAX_HISTORY);
        assert_eq!(rel.history[0].tick, 5);
    }

    #[test]
    fn tracker_trust_converges_to_baseline() {
        let mut tracker = RelationshipTracker::new();
        tracker.update_relationship("p1", "p2", "trusted", 0, 0.0, 0.4);
        let before = tracker.get("p1", "p2").unwrap().trust;
        for _ in 0..100 {
            tracker.tick_converge(1.0);
        }
        let after = tracker.get("p1", "p2").unwrap().trust;
        assert!(
            (after - TRUST_BASELINE).abs() < (before - TRUST_BASELINE).abs(),
            "trust should converge toward baseline"
        );
    }

    #[test]
    fn tracker_deterministic() {
        let mut a = RelationshipTracker::new();
        let mut b = RelationshipTracker::new();
        for i in 0..10u64 {
            a.update_relationship("p1", "p2", "event", i, 0.05, 0.02);
            b.update_relationship("p1", "p2", "event", i, 0.05, 0.02);
        }
        assert_eq!(
            a.get("p1", "p2"),
            b.get("p1", "p2"),
            "tracker must be deterministic"
        );
    }

    #[test]
    fn tracker_get_or_create_new_relationship() {
        let mut tracker = RelationshipTracker::new();
        let rel = tracker.get_or_create("p1", "p2");
        assert_eq!(rel.from_id, "p1");
        assert_eq!(rel.to_id, "p2");
        assert!((rel.trust - TRUST_BASELINE).abs() < f64::EPSILON);
    }

    #[test]
    fn tracker_bidirectional_independent() {
        let mut tracker = RelationshipTracker::new();
        tracker.update_relationship("p1", "p2", "helped", 1, 0.2, 0.1);
        tracker.update_relationship("p2", "p1", "betrayed", 2, -0.1, -0.2);
        let ab = tracker.get("p1", "p2").unwrap();
        let ba = tracker.get("p2", "p1").unwrap();
        assert!(
            ab.strength > ba.strength,
            "p1->p2 bond should differ from p2->p1"
        );
    }
}
