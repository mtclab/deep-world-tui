use crate::model::quest::{Quest, QuestKind, QuestReward};
use crate::model::{ItemType, PeopleKind, Region};
use crate::rng::SeedRng;

const FETCH_DESCRIPTIONS: &[&str] = &[
    "The settlement needs what the land sometimes gives.",
    "There is something the earth provides. I must gather it.",
    "What grows wild is needed within the walls.",
];

const VISIT_DESCRIPTIONS: &[&str] = &[
    "There is a place I have not yet seen.",
    "A distant road calls. I should answer.",
    "Somewhere beyond the horizon waits. I must go.",
];

const AID_DESCRIPTIONS: &[&str] = &[
    "Someone near needs help I can give.",
    "A face in the settlement bears need. I can ease it.",
    "There is one who struggles. I have something to offer.",
];

const REPUTATION_DESCRIPTIONS: &[&str] = &[
    "I must earn trust in a new home.",
    "The people here do not know me yet. I must change that.",
    "Stranger is a word that closes doors. I must become something else.",
];

const SURVIVE_DESCRIPTIONS: &[&str] = &[
    "The deep world will test me.",
    "I must endure. That is the whole of it.",
    "Survival is its own quest. I will prove myself.",
];

fn pick(rng: &mut SeedRng, slots: &[&'static str]) -> &'static str {
    let idx = rng.gen_range(slots.len() as u32) as usize;
    slots[idx % slots.len()]
}

pub fn generate_initial_quests(
    seed: u64,
    player_people: PeopleKind,
    regions: &[Region],
) -> Vec<Quest> {
    let mut rng = SeedRng::new(seed).fork_for("initial-quests");
    let count = 1 + rng.gen_range(2) as usize;
    let mut quests = Vec::new();
    let current_day = 1;

    let used_kinds = &mut [false; 5];

    for _ in 0..count {
        let order = rng.gen_range(5) as usize;
        let kind_idx = if used_kinds[order] {
            (0..5).find(|i| !used_kinds[*i]).unwrap_or(order)
        } else {
            order
        };
        used_kinds[kind_idx] = true;

        let starting_region = regions.first();
        let quest = match kind_idx {
            0 => {
                let fetch_items = match player_people {
                    PeopleKind::Metsik | PeopleKind::Hal => ItemType::Wood,
                    PeopleKind::Sepat => ItemType::Iron,
                    PeopleKind::Ahjo => ItemType::Food,
                    PeopleKind::Merak => ItemType::Water,
                    PeopleKind::Tzakhar => ItemType::Stone,
                    _ => ItemType::Herb,
                };
                let count = 2 + rng.gen_range(3);
                let deadline = current_day + 14 + rng.gen_range(7);
                Quest {
                    kind: QuestKind::FetchItem {
                        item: fetch_items,
                        count,
                    },
                    description: pick(&mut rng, FETCH_DESCRIPTIONS).into(),
                    reward: QuestReward::Reputation { amount: 0.1 },
                    progress: 0,
                    target: count,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
            1 => {
                let region_idx = if regions.len() > 1 {
                    1 + rng.gen_range((regions.len() - 1) as u32) as usize
                } else {
                    0
                };
                let region_idx = region_idx.min(regions.len() - 1);
                let deadline = current_day + 10 + rng.gen_range(10);
                Quest {
                    kind: QuestKind::VisitRegion { region_idx },
                    description: pick(&mut rng, VISIT_DESCRIPTIONS).into(),
                    reward: QuestReward::Items {
                        item: ItemType::Herb,
                        count: 3,
                    },
                    progress: 0,
                    target: 1,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
            2 => {
                let npc_id = starting_region
                    .and_then(|r| r.settlements.first())
                    .and_then(|s| s.people.first())
                    .map(|p| p.id.clone())
                    .unwrap_or_else(|| "npc-0".into());
                let deadline = current_day + 12 + rng.gen_range(8);
                Quest {
                    kind: QuestKind::AidNPC {
                        npc_id: npc_id.clone(),
                    },
                    description: pick(&mut rng, AID_DESCRIPTIONS).into(),
                    reward: QuestReward::Relationship {
                        npc_id,
                        delta: 0.15,
                    },
                    progress: 0,
                    target: 1,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
            3 => {
                let threshold = 0.5 + rng.gen_range(3) as f64 / 10.0;
                let deadline = current_day + 15 + rng.gen_range(10);
                Quest {
                    kind: QuestKind::ReachReputation { threshold },
                    description: pick(&mut rng, REPUTATION_DESCRIPTIONS).into(),
                    reward: QuestReward::Items {
                        item: ItemType::Food,
                        count: 5,
                    },
                    progress: 0,
                    target: 1,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
            _ => {
                let days = 5 + rng.gen_range(6);
                let deadline = current_day + days + 3;
                Quest {
                    kind: QuestKind::SurviveDays { days },
                    description: pick(&mut rng, SURVIVE_DESCRIPTIONS).into(),
                    reward: QuestReward::Reputation { amount: 0.2 },
                    progress: 0,
                    target: days,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
        };
        quests.push(quest);
    }

    quests
}

pub struct QuestCheckResult {
    pub completed: Vec<usize>,
    pub expired: Vec<usize>,
}

pub fn check_quests(
    quests: &[Quest],
    inventory: &crate::model::Inventory,
    current_region_idx: usize,
    aided_npcs: &[String],
    local_reputation: f64,
    current_day: u32,
) -> QuestCheckResult {
    let mut completed = Vec::new();
    let mut expired = Vec::new();

    for (i, quest) in quests.iter().enumerate() {
        if quest.is_complete() {
            continue;
        }

        if quest.is_expired(current_day) {
            expired.push(i);
            continue;
        }

        let progress = match &quest.kind {
            QuestKind::FetchItem { item, count: _ } => inventory.get(*item).min(quest.target),
            QuestKind::VisitRegion { region_idx } => {
                if *region_idx == current_region_idx {
                    1
                } else {
                    0
                }
            }
            QuestKind::AidNPC { npc_id } => {
                if aided_npcs.contains(npc_id) {
                    1
                } else {
                    0
                }
            }
            QuestKind::ReachReputation { threshold } => {
                if local_reputation >= *threshold {
                    1
                } else {
                    0
                }
            }
            QuestKind::SurviveDays { days: _ } => {
                let elapsed = current_day.saturating_sub(quest.assigned_day);
                elapsed.min(quest.target)
            }
        };

        if progress >= quest.target {
            completed.push(i);
        }
    }

    QuestCheckResult { completed, expired }
}

pub fn apply_quest_reward(
    reward: &QuestReward,
    inventory: &mut crate::model::Inventory,
    reputation: &mut crate::sim::reputation::ReputationStore,
    player_id: &str,
    current_settlement: &str,
) {
    match reward {
        QuestReward::Reputation { amount } => {
            reputation.adjust_settlement(player_id, current_settlement, *amount);
        }
        QuestReward::Items { item, count } => {
            inventory.add(*item, *count);
        }
        QuestReward::Relationship {
            npc_id: _,
            delta: _,
        } => {
            // Relationship deltas are applied via the relationship tracker in the caller
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Inventory;

    fn make_regions() -> Vec<Region> {
        crate::gen::world::generate_world(
            42,
            &crate::charts::load_charts("data/charts.ron").unwrap(),
        )
        .regions
    }

    #[test]
    fn generate_produces_1_to_2_quests() {
        let regions = make_regions();
        let quests = generate_initial_quests(42, PeopleKind::Metsik, &regions);
        assert!(!quests.is_empty(), "should produce at least 1 quest");
        assert!(
            quests.len() <= 2,
            "should produce at most 2 quests, got {}",
            quests.len()
        );
    }

    #[test]
    fn generate_deterministic() {
        let regions = make_regions();
        let a = generate_initial_quests(42, PeopleKind::Metsik, &regions);
        let b = generate_initial_quests(42, PeopleKind::Metsik, &regions);
        assert_eq!(a.len(), b.len());
        for (qa, qb) in a.iter().zip(b.iter()) {
            assert_eq!(qa.description, qb.description);
            assert_eq!(qa.kind, qb.kind);
        }
    }

    #[test]
    fn different_seed_different_quests() {
        let regions = make_regions();
        let a = generate_initial_quests(42, PeopleKind::Metsik, &regions);
        let b = generate_initial_quests(99, PeopleKind::Metsik, &regions);
        assert_ne!(
            a.len(),
            b.len(),
            "different seeds should produce different quest counts or content"
        );
    }

    #[test]
    fn fetch_item_completes_when_inventory_sufficient() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Herb, 3);
        let quests = vec![Quest {
            kind: QuestKind::FetchItem {
                item: ItemType::Herb,
                count: 3,
            },
            description: "test".into(),
            reward: QuestReward::Reputation { amount: 0.1 },
            progress: 0,
            target: 3,
            deadline_day: 30,
            assigned_day: 1,
        }];
        let result = check_quests(&quests, &inv, 0, &[], 0.5, 5);
        assert_eq!(result.completed.len(), 1);
    }

    #[test]
    fn quest_expires_past_deadline() {
        let quests = vec![Quest {
            kind: QuestKind::VisitRegion { region_idx: 5 },
            description: "test".into(),
            reward: QuestReward::Items {
                item: ItemType::Herb,
                count: 3,
            },
            progress: 0,
            target: 1,
            deadline_day: 10,
            assigned_day: 1,
        }];
        let inv = Inventory::default();
        let result = check_quests(&quests, &inv, 0, &[], 0.5, 15);
        assert_eq!(result.expired.len(), 1);
        assert!(result.completed.is_empty());
    }

    #[test]
    fn survive_days_tracks_progress() {
        let quests = vec![Quest {
            kind: QuestKind::SurviveDays { days: 5 },
            description: "test".into(),
            reward: QuestReward::Reputation { amount: 0.2 },
            progress: 0,
            target: 5,
            deadline_day: 20,
            assigned_day: 1,
        }];
        let inv = Inventory::default();
        let result = check_quests(&quests, &inv, 0, &[], 0.5, 6);
        assert_eq!(
            result.completed.len(),
            1,
            "after 5 days (day 6 from start), survive 5 days should complete"
        );
    }
}
