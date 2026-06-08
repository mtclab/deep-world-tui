use crate::rng::SeedRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationKind {
    OwesDebt,
    SwornFriend,
    Feud,
    Estranged,
    Spouse,
    Sibling,
    Parent,
    Child,
    Apprentice,
    Rival,
}

impl RelationKind {
    pub fn label(self) -> &'static str {
        match self {
            RelationKind::OwesDebt => "owes a debt",
            RelationKind::SwornFriend => "sworn friend",
            RelationKind::Feud => "feud",
            RelationKind::Estranged => "estranged",
            RelationKind::Spouse => "spouse",
            RelationKind::Sibling => "sibling",
            RelationKind::Parent => "parent",
            RelationKind::Child => "child",
            RelationKind::Apprentice => "apprentice",
            RelationKind::Rival => "rival",
        }
    }

    fn is_exclusive(self) -> bool {
        matches!(self, RelationKind::Spouse)
    }

    fn conflicts_with(self, other: RelationKind) -> bool {
        matches!(
            (self, other),
            (RelationKind::Spouse, RelationKind::Rival)
                | (RelationKind::Rival, RelationKind::Spouse)
                | (RelationKind::Spouse, RelationKind::Feud)
                | (RelationKind::Feud, RelationKind::Spouse)
        )
    }
}

static RELATION_WEIGHTS: &[(RelationKind, u32)] = &[
    (RelationKind::OwesDebt, 20),
    (RelationKind::SwornFriend, 15),
    (RelationKind::Feud, 10),
    (RelationKind::Estranged, 10),
    (RelationKind::Rival, 15),
    (RelationKind::Apprentice, 10),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterNpcRelation {
    pub kind: RelationKind,
    pub target_person_id: String,
    pub intensity: f64,
    pub formed_at_tick: u64,
    pub reason: String,
}

const DECAY_RATE: f64 = 0.001;
const PRUNE_THRESHOLD: f64 = 0.05;

pub fn decay_relations(relations: &mut Vec<InterNpcRelation>) {
    for r in relations.iter_mut() {
        r.intensity = (r.intensity - DECAY_RATE).max(0.0);
    }
    relations.retain(|r| r.intensity >= PRUNE_THRESHOLD);
}

fn pick_weighted(rng: &mut SeedRng) -> RelationKind {
    let total: u32 = RELATION_WEIGHTS.iter().map(|(_, w)| *w).sum();
    let roll = rng.gen_range(total);
    let mut acc = 0u32;
    for &(kind, weight) in RELATION_WEIGHTS {
        acc += weight;
        if roll < acc {
            return kind;
        }
    }
    RelationKind::OwesDebt
}

static RELATION_FLAVORS: &[&str] = &[
    "a promise kept long ago",
    "a debt unpaid",
    "shared hardship",
    "a slight never forgiven",
    "old words spoken in anger",
    "a bond forged in youth",
    "trade gone sour",
    "a favour returned twice",
    "blood remembered",
    "a craft passed down",
];

pub fn generate_npc_relations(
    rng: &mut SeedRng,
    person_id: &str,
    settlement_ids: &[String],
    tick: u64,
    has_spouse: bool,
    children_count: u32,
) -> Vec<InterNpcRelation> {
    let mut relations = Vec::new();
    if settlement_ids.len() < 2 {
        return relations;
    }
    let count = 1 + rng.gen_range(2) as usize;
    let count = count.min(settlement_ids.len() - 1);

    for _ in 0..count {
        let kind = pick_weighted(rng);
        let target_idx = rng.gen_range(settlement_ids.len() as u32) as usize;
        let target_id = &settlement_ids[target_idx];
        if target_id == person_id {
            continue;
        }
        let has_conflicting = relations.iter().any(|r| {
            r.target_person_id == *target_id
                && (kind.conflicts_with(r.kind) || (kind.is_exclusive() && r.kind.is_exclusive()))
        });
        if has_conflicting {
            continue;
        }
        let reason_idx = rng.gen_range(RELATION_FLAVORS.len() as u32) as usize;
        relations.push(InterNpcRelation {
            kind,
            target_person_id: target_id.clone(),
            intensity: (0.3 + rng.gen_f64() * 0.7),
            formed_at_tick: tick,
            reason: RELATION_FLAVORS[reason_idx].into(),
        });
    }

    if has_spouse && !settlement_ids.is_empty() {
        let spouse_idx = rng.gen_range(settlement_ids.len() as u32) as usize;
        let spouse_id = &settlement_ids[spouse_idx];
        if spouse_id != person_id {
            let has_spouse_already = relations.iter().any(|r| r.kind == RelationKind::Spouse);
            if !has_spouse_already {
                relations.push(InterNpcRelation {
                    kind: RelationKind::Spouse,
                    target_person_id: spouse_id.clone(),
                    intensity: 0.8,
                    formed_at_tick: tick,
                    reason: "marriage".into(),
                });
            }
        }
    }

    if children_count > 0 && !settlement_ids.is_empty() {
        let n_child_rels = (children_count).min(settlement_ids.len() as u32);
        for _i in 0..n_child_rels {
            let idx = rng.gen_range(settlement_ids.len() as u32) as usize;
            let child_id = &settlement_ids[idx];
            if child_id == person_id {
                continue;
            }
            relations.push(InterNpcRelation {
                kind: RelationKind::Child,
                target_person_id: child_id.clone(),
                intensity: 0.7,
                formed_at_tick: tick,
                reason: "parenthood".into(),
            });
        }
    }

    relations
}

static CONFLICT_FLAVORS: &[&str] = &[
    "glare past each other in the market.",
    "exchange sharp words by the well.",
    "walk a wide circle around one another.",
    "argue over the price of grain.",
    "one turns away, jaw tight. The other watches.",
    "a door slams. Neither speaks.",
];

pub fn conflict_flavor(rng: &mut SeedRng) -> &'static str {
    let idx = rng.gen_range(CONFLICT_FLAVORS.len() as u32) as usize;
    CONFLICT_FLAVORS[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_kind_labels() {
        assert_eq!(RelationKind::OwesDebt.label(), "owes a debt");
        assert_eq!(RelationKind::SwornFriend.label(), "sworn friend");
        assert_eq!(RelationKind::Feud.label(), "feud");
        assert_eq!(RelationKind::Spouse.label(), "spouse");
        assert_eq!(RelationKind::Rival.label(), "rival");
    }

    #[test]
    fn spouse_rival_conflict() {
        assert!(RelationKind::Spouse.conflicts_with(RelationKind::Rival));
        assert!(RelationKind::Rival.conflicts_with(RelationKind::Spouse));
        assert!(!RelationKind::OwesDebt.conflicts_with(RelationKind::SwornFriend));
    }

    #[test]
    fn decay_monotonic() {
        let mut relations = vec![InterNpcRelation {
            kind: RelationKind::OwesDebt,
            target_person_id: "p2".into(),
            intensity: 0.5,
            formed_at_tick: 100,
            reason: "test".into(),
        }];
        decay_relations(&mut relations);
        assert!(relations[0].intensity < 0.5);
    }

    #[test]
    fn decay_prunes_below_threshold() {
        let mut relations = vec![
            InterNpcRelation {
                kind: RelationKind::Estranged,
                target_person_id: "p2".into(),
                intensity: 0.03,
                formed_at_tick: 100,
                reason: "test".into(),
            },
            InterNpcRelation {
                kind: RelationKind::OwesDebt,
                target_person_id: "p3".into(),
                intensity: 0.2,
                formed_at_tick: 100,
                reason: "test".into(),
            },
        ];
        decay_relations(&mut relations);
        assert!(relations.len() == 1);
        assert_eq!(relations[0].target_person_id, "p3");
    }

    #[test]
    fn generate_0_5_to_2_per_person() {
        let mut rng = SeedRng::new(42);
        let ids: Vec<String> = (0..10).map(|i| format!("p{}", i)).collect();
        for _ in 0..20 {
            let rels = generate_npc_relations(&mut rng, "p0", &ids, 0, false, 0);
            assert!(rels.len() <= 4, "too many relations: {:?}", rels.len());
        }
    }

    #[test]
    fn no_self_relation() {
        let mut rng = SeedRng::new(99);
        let ids: Vec<String> = (0..10).map(|i| format!("p{}", i)).collect();
        let rels = generate_npc_relations(&mut rng, "p0", &ids, 0, true, 2);
        for r in &rels {
            assert_ne!(r.target_person_id, "p0", "no self-relations");
        }
    }

    #[test]
    fn conflict_flavor_deterministic() {
        let mut rng = SeedRng::new(7);
        let a = conflict_flavor(&mut rng);
        let mut rng2 = SeedRng::new(7);
        let b = conflict_flavor(&mut rng2);
        assert_eq!(a, b);
    }

    #[test]
    fn exclusive_spouse_not_duplicated() {
        let mut rng = SeedRng::new(123);
        let ids: Vec<String> = (0..10).map(|i| format!("p{}", i)).collect();
        let rels = generate_npc_relations(&mut rng, "p0", &ids, 0, true, 0);
        let spouse_count = rels
            .iter()
            .filter(|r| r.kind == RelationKind::Spouse)
            .count();
        assert!(spouse_count <= 1, "at most one spouse relation");
    }
}
