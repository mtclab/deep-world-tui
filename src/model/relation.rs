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

/// One day's motion of a settlement's social web (#548 living relationships): a
/// sampled pair of residents either **deepens** an existing bond or **forms** a
/// new one, so the web grows and shifts rather than only fading. **Shared
/// hardship** (`famine`) works a bond the harder — the close drawn closer, the
/// soured frayed further. Returns a rumor when a notable new bond strikes.
/// Pure and deterministic from `seed`/`tick`; cheap (one pair). Bounded at eight
/// relations a person.
pub fn evolve_settlement_relations(
    people: &mut [crate::model::Person],
    settlement_name: &str,
    famine: bool,
    seed: u64,
    tick: u64,
) -> Option<String> {
    let n = people.len();
    if n < 2 {
        return None;
    }
    let mut rng = SeedRng::new(seed ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let i = rng.gen_range(n as u32) as usize;
    let mut j = rng.gen_range(n as u32) as usize;
    if i == j {
        j = (j + 1) % n;
    }
    let jid = people[j].id.clone();
    let jname = people[j].name.clone();
    let jprof = people[j].profession.clone();
    let iprof = people[i].profession.clone();
    let iname = people[i].name.clone();

    if let Some(rel) = people[i]
        .relations
        .iter_mut()
        .find(|r| r.target_person_id == jid)
    {
        let warm = matches!(
            rel.kind,
            RelationKind::SwornFriend
                | RelationKind::Spouse
                | RelationKind::Sibling
                | RelationKind::Parent
                | RelationKind::Child
                | RelationKind::Apprentice
        );
        let hardship = if famine { 0.03 } else { 0.0 };
        let base = if warm { 0.03 } else { 0.02 };
        rel.intensity = (rel.intensity + base + hardship).min(1.0);
        return None;
    }

    if people[i].relations.len() < 8 && rng.gen_range(100) < 8 {
        let same_trade = !iprof.is_empty() && iprof == jprof;
        let kind = if same_trade {
            RelationKind::Rival
        } else {
            RelationKind::SwornFriend
        };
        let reason = if same_trade {
            "two of one trade in one town"
        } else {
            "the long nearness of neighbours"
        };
        people[i].relations.push(InterNpcRelation {
            kind,
            target_person_id: jid,
            intensity: 0.3,
            formed_at_tick: tick,
            reason: reason.to_string(),
        });
        if rng.gen_range(100) < 35 {
            return Some(if same_trade {
                format!(
                    "A rivalry has sharpened between {iname} and {jname} in {settlement_name} — two of one trade, one town."
                )
            } else {
                format!("{iname} and {jname} have grown thick as thieves in {settlement_name}.")
            });
        }
    }
    None
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

    fn two_residents() -> Vec<crate::model::Person> {
        vec![
            crate::model::Person {
                id: "a".into(),
                name: "Ana".into(),
                profession: "smith".into(),
                relations: vec![InterNpcRelation {
                    kind: RelationKind::SwornFriend,
                    target_person_id: "b".into(),
                    intensity: 0.3,
                    formed_at_tick: 0,
                    reason: "x".into(),
                }],
                ..Default::default()
            },
            crate::model::Person {
                id: "b".into(),
                name: "Bo".into(),
                profession: "weaver".into(),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn shared_hardship_deepens_a_bond_harder() {
        // On the first day the sampled pair touches Ana's bond, famine must
        // deepen it more than plenty does from the same start.
        for tick in 0..400u64 {
            let mut fed = two_residents();
            let mut hungry = two_residents();
            evolve_settlement_relations(&mut fed, "Z", false, 99, tick);
            evolve_settlement_relations(&mut hungry, "Z", true, 99, tick);
            let f = fed[0].relations[0].intensity;
            let h = hungry[0].relations[0].intensity;
            if f > 0.3 {
                assert!(h > f, "famine works the bond harder ({h} > {f})");
                return;
            }
        }
        panic!("no tick deepened the bond across the window");
    }

    #[test]
    fn a_new_bond_can_form_and_be_spoken_of() {
        // Two residents with no bond from Bo's side: over many days, Bo forms a
        // bond toward Ana, and at least one is talked of.
        let mut spoke = false;
        let mut formed = false;
        for tick in 0..2000u64 {
            let mut people = vec![
                crate::model::Person {
                    id: "a".into(),
                    name: "Ana".into(),
                    profession: "smith".into(),
                    ..Default::default()
                },
                crate::model::Person {
                    id: "b".into(),
                    name: "Bo".into(),
                    profession: "weaver".into(),
                    ..Default::default()
                },
            ];
            let msg = evolve_settlement_relations(&mut people, "Z", false, 7, tick);
            if people.iter().any(|p| !p.relations.is_empty()) {
                formed = true;
            }
            if msg.is_some() {
                spoke = true;
            }
            if formed && spoke {
                break;
            }
        }
        assert!(formed, "a new bond forms over time");
        assert!(spoke, "a new bond is sometimes spoken of");
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
