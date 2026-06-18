use crate::model::quest::{Quest, QuestKind, QuestReward};
use crate::model::{ItemType, PeopleKind, Region};
use crate::rng::SeedRng;

/// The aid one of the Five asks of a stranger who comes to their enclave
/// (#454): a fetch of what they lack from the surface, repaid in the good only
/// they make. `None` for the human regions. Each is true to its monograph —
/// the deep-smiths want surface food, the tideline-folk cloth, and so on.
pub fn enclave_quest(people: PeopleKind, current_day: u32) -> Option<Quest> {
    use ItemType as I;
    let (want, n, reward, rn, desc): (ItemType, u32, ItemType, u32, &str) = match people {
        PeopleKind::Tzakhar => (
            I::Food,
            4,
            I::Iron,
            2,
            "The Tzäkhar deep-forges burn day and night; they ask for surface food, and will repay it in worked iron.",
        ),
        PeopleKind::Merak => (
            I::Cloth,
            3,
            I::Glass,
            2,
            "The Mëräk have no looms beneath the tide; bring them cloth and they will trade deep-glass for it.",
        ),
        PeopleKind::Shear => (
            I::Tool,
            1,
            I::Herb,
            3,
            "The She'ar want good tools to work the dry country; bring one and they will give the succulent-physic only they keep.",
        ),
        PeopleKind::Hal => (
            I::Cloth,
            3,
            I::Herb,
            2,
            "The Häl weave nothing but the living green; bring them cloth and they will repay it with their true salve.",
        ),
        PeopleKind::Khor => (
            I::Tool,
            1,
            I::Hide,
            3,
            "The Khör need iron tools the steppe cannot forge; bring one and they will give härkä-hide in kind.",
        ),
        _ => return None,
    };
    Some(Quest {
        kind: QuestKind::FetchItem {
            item: want,
            count: n,
        },
        description: desc.into(),
        reward: QuestReward::Items {
            item: reward,
            count: rn,
        },
        progress: 0,
        target: n,
        // A long, soft deadline — the road to an enclave is not run in a week.
        deadline_day: current_day + 60,
        assigned_day: current_day,
    })
}

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

const DELIVER_DESCRIPTIONS: &[&str] = &[
    "Another market waits for what we have in plenty.",
    "Goods are owed across the border. Someone must carry them.",
    "What is common here is dear elsewhere. Take it there.",
];

const BUILD_DESCRIPTIONS: &[&str] = &[
    "Raise something that outlasts the season.",
    "The land needs a roof on it. Put one up.",
    "Build, and the road will remember you.",
];

const DISCOVERY_DESCRIPTIONS: &[&str] = &[
    "There are places no map names. Find one.",
    "Walk until the land surprises you.",
    "Something old waits off the path. Look for it.",
];

const TALK_DESCRIPTIONS: &[&str] = &[
    "Word matters more than coin here. Go and be known.",
    "Deal with people — talk, trade, be remembered.",
    "A stranger is a danger. Stop being one.",
];

pub fn generate_initial_quests(
    seed: u64,
    player_people: PeopleKind,
    regions: &[Region],
) -> Vec<Quest> {
    generate_quests(seed, player_people, regions, 1)
}

/// Generate a batch of quests dated from `current_day` (deadlines are relative
/// to it). Used both for the initial batch and for periodic regeneration so the
/// quest board never runs dry over a long, multi-generational game.
pub fn generate_quests(
    seed: u64,
    player_people: PeopleKind,
    regions: &[Region],
    current_day: u32,
) -> Vec<Quest> {
    let mut rng = SeedRng::new(seed).fork_for("quests");
    let count = 1 + rng.gen_range(2) as usize;
    let mut quests = Vec::new();

    let used_kinds = &mut [false; 9];

    for _ in 0..count {
        let order = rng.gen_range(9) as usize;
        let kind_idx = if used_kinds[order] {
            (0..9).find(|i| !used_kinds[*i]).unwrap_or(order)
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
            5 => {
                let region_idx = if regions.len() > 1 {
                    1 + rng.gen_range((regions.len() - 1) as u32) as usize
                } else {
                    0
                };
                let region_idx = region_idx.min(regions.len().saturating_sub(1));
                let item = match player_people {
                    PeopleKind::Sepat | PeopleKind::Ahjo => ItemType::Iron,
                    PeopleKind::Metsik | PeopleKind::Hal => ItemType::Wood,
                    _ => ItemType::Food,
                };
                let count = 2 + rng.gen_range(2);
                let deadline = current_day + 16 + rng.gen_range(8);
                Quest {
                    kind: QuestKind::DeliverTo {
                        region_idx,
                        item,
                        count,
                    },
                    description: pick(&mut rng, DELIVER_DESCRIPTIONS).into(),
                    reward: QuestReward::Items {
                        item: ItemType::Coin,
                        count: 8,
                    },
                    progress: 0,
                    target: 1,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
            6 => {
                let deadline = current_day + 20 + rng.gen_range(10);
                Quest {
                    kind: QuestKind::RaiseBuilding { region_idx: 0 },
                    description: pick(&mut rng, BUILD_DESCRIPTIONS).into(),
                    reward: QuestReward::Reputation { amount: 0.15 },
                    progress: 0,
                    target: 1,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
            7 => {
                let deadline = current_day + 15 + rng.gen_range(10);
                Quest {
                    // baseline is filled in by the caller (current observed count).
                    kind: QuestKind::VisitDiscovery { baseline: u32::MAX },
                    description: pick(&mut rng, DISCOVERY_DESCRIPTIONS).into(),
                    reward: QuestReward::Items {
                        item: ItemType::Herb,
                        count: 4,
                    },
                    progress: 0,
                    target: 1,
                    deadline_day: deadline,
                    assigned_day: current_day,
                }
            }
            8 => {
                let count = 3 + rng.gen_range(3);
                let deadline = current_day + 12 + rng.gen_range(8);
                Quest {
                    kind: QuestKind::TalkToPeople { count },
                    description: pick(&mut rng, TALK_DESCRIPTIONS).into(),
                    reward: QuestReward::Reputation { amount: 0.12 },
                    progress: 0,
                    target: count,
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

/// Everything quest progress is judged against.
pub struct QuestContext<'a> {
    pub inventory: &'a crate::model::Inventory,
    pub current_region_idx: usize,
    pub aided_npcs: &'a [String],
    pub local_reputation: f64,
    pub current_day: u32,
    pub observed_discoveries: u32,
    pub dealings: u32,
    pub player_structure_regions: &'a [usize],
}

pub fn check_quests(quests: &mut [Quest], ctx: &QuestContext) -> QuestCheckResult {
    let inventory = ctx.inventory;
    let current_region_idx = ctx.current_region_idx;
    let aided_npcs = ctx.aided_npcs;
    let local_reputation = ctx.local_reputation;
    let current_day = ctx.current_day;
    let observed_discoveries = ctx.observed_discoveries;
    let dealings = ctx.dealings;
    let player_structure_regions = ctx.player_structure_regions;
    let mut completed = Vec::new();
    let mut expired = Vec::new();

    for (i, quest) in quests.iter_mut().enumerate() {
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
            QuestKind::DeliverTo {
                region_idx,
                item,
                count,
            } => {
                if *region_idx == current_region_idx && inventory.get(*item) >= *count {
                    1
                } else {
                    0
                }
            }
            QuestKind::RaiseBuilding { region_idx } => {
                if player_structure_regions.contains(region_idx) {
                    1
                } else {
                    0
                }
            }
            QuestKind::VisitDiscovery { baseline } => {
                if *baseline != u32::MAX && observed_discoveries > *baseline {
                    1
                } else {
                    0
                }
            }
            QuestKind::TalkToPeople { count: _ } => dealings.min(quest.target),
            // Living-world relief quests are not recomputed here: the act of
            // tending the plague or provisioning the famine sets their progress
            // directly (see the App). Carry it through, so they complete when
            // the deed is done and expire on the generic deadline if it is not.
            QuestKind::RelievePlague { .. }
            | QuestKind::RelieveFamine { .. }
            | QuestKind::BrokerTruce { .. }
            | QuestKind::SteadyFaith { .. }
            | QuestKind::SupplyGoods { .. } => quest.progress,
        };

        quest.progress = progress;

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
    relationships: &mut crate::sim::relationships::RelationshipTracker,
    tick: u64,
) {
    match reward {
        QuestReward::Reputation { amount } => {
            reputation.adjust_settlement(player_id, current_settlement, *amount);
        }
        QuestReward::Items { item, count } => {
            inventory.add(*item, *count);
        }
        QuestReward::Relationship { npc_id, delta } => {
            relationships.update_relationship(
                player_id,
                npc_id,
                "quest fulfilled",
                tick,
                *delta,
                *delta * 0.5,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Inventory;

    #[test]
    fn each_of_the_five_asks_an_aid_humans_do_not() {
        for p in [
            PeopleKind::Tzakhar,
            PeopleKind::Merak,
            PeopleKind::Shear,
            PeopleKind::Hal,
            PeopleKind::Khor,
        ] {
            let q = enclave_quest(p, 10).expect("the Five each ask a thing");
            // A real fetch, repaid in a good, with a soft deadline.
            assert!(matches!(q.kind, QuestKind::FetchItem { .. }));
            assert!(matches!(q.reward, QuestReward::Items { count, .. } if count > 0));
            assert!(q.target > 0 && q.deadline_day > 10);
        }
        assert!(enclave_quest(PeopleKind::Metsik, 10).is_none());
    }

    fn make_regions() -> Vec<Region> {
        crate::gen::world::generate_world(42, &crate::charts::load_charts().unwrap()).regions
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
        let mut quests = vec![Quest {
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
        let result = check_quests(
            &mut quests,
            &QuestContext {
                inventory: &inv,
                current_region_idx: 0,
                aided_npcs: &[],
                local_reputation: 0.5,
                current_day: 5,
                observed_discoveries: 0,
                dealings: 0,
                player_structure_regions: &[],
            },
        );
        assert_eq!(result.completed.len(), 1);
        assert_eq!(quests[0].progress, 3);
    }

    #[test]
    fn quest_expires_past_deadline() {
        let mut quests = vec![Quest {
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
        let result = check_quests(
            &mut quests,
            &QuestContext {
                inventory: &inv,
                current_region_idx: 0,
                aided_npcs: &[],
                local_reputation: 0.5,
                current_day: 15,
                observed_discoveries: 0,
                dealings: 0,
                player_structure_regions: &[],
            },
        );
        assert_eq!(result.expired.len(), 1);
        assert!(result.completed.is_empty());
    }

    #[test]
    fn survive_days_tracks_progress() {
        let mut quests = vec![Quest {
            kind: QuestKind::SurviveDays { days: 5 },
            description: "test".into(),
            reward: QuestReward::Reputation { amount: 0.2 },
            progress: 0,
            target: 5,
            deadline_day: 20,
            assigned_day: 1,
        }];
        let inv = Inventory::default();
        let result = check_quests(
            &mut quests,
            &QuestContext {
                inventory: &inv,
                current_region_idx: 0,
                aided_npcs: &[],
                local_reputation: 0.5,
                current_day: 6,
                observed_discoveries: 0,
                dealings: 0,
                player_structure_regions: &[],
            },
        );
        assert_eq!(
            result.completed.len(),
            1,
            "after 5 days (day 6 from start), survive 5 days should complete"
        );
        assert_eq!(quests[0].progress, 5);
    }

    #[test]
    fn progress_updates_on_partial_fetch() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Herb, 1);
        let mut quests = vec![Quest {
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
        let result = check_quests(
            &mut quests,
            &QuestContext {
                inventory: &inv,
                current_region_idx: 0,
                aided_npcs: &[],
                local_reputation: 0.5,
                current_day: 5,
                observed_discoveries: 0,
                dealings: 0,
                player_structure_regions: &[],
            },
        );
        assert!(result.completed.is_empty());
        assert_eq!(quests[0].progress, 1);
        assert_eq!(
            quests[0].progress_hint(),
            "I have begun, but there is far to go."
        );
    }
}
