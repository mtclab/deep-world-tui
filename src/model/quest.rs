use serde::{Deserialize, Serialize};

use super::ItemType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestKind {
    FetchItem { item: ItemType, count: u32 },
    VisitRegion { region_idx: usize },
    AidNPC { npc_id: String },
    ReachReputation { threshold: f64 },
    SurviveDays { days: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestReward {
    Reputation { amount: f64 },
    Items { item: ItemType, count: u32 },
    Relationship { npc_id: String, delta: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quest {
    pub kind: QuestKind,
    pub description: String,
    pub reward: QuestReward,
    pub progress: u32,
    pub target: u32,
    pub deadline_day: u32,
    pub assigned_day: u32,
}

impl Quest {
    pub fn is_complete(&self) -> bool {
        self.progress >= self.target
    }

    pub fn is_expired(&self, current_day: u32) -> bool {
        current_day > self.deadline_day
    }

    pub fn progress_hint(&self) -> &'static str {
        let ratio = if self.target == 0 {
            1.0
        } else {
            self.progress as f64 / self.target as f64
        };
        if ratio >= 1.0 {
            "I have done what was asked."
        } else if ratio >= 0.75 {
            "I have nearly enough."
        } else if ratio >= 0.5 {
            "I am halfway there."
        } else if ratio >= 0.25 {
            "I have begun, but there is far to go."
        } else {
            "The settlement needs more."
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestType {
    DeliverItem,
    GatherResource,
    EscortNpc,
    FindLocation,
    ResolveDispute,
}

impl QuestType {
    pub fn name(self) -> &'static str {
        match self {
            QuestType::DeliverItem => "deliver item",
            QuestType::GatherResource => "gather resource",
            QuestType::EscortNpc => "escort NPC",
            QuestType::FindLocation => "find location",
            QuestType::ResolveDispute => "resolve dispute",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyQuest {
    pub id: String,
    pub quest_type: QuestType,
    pub description: String,
    pub issuer_id: String,
    pub issuer_name: String,
    pub target_item: Option<ItemType>,
    pub target_count: u32,
    pub target_location: Option<String>,
    pub reward_coins: u32,
    pub reward_reputation: f64,
    pub deadline_tick: u64,
    pub accepted: bool,
    pub completed: bool,
    pub progress: u32,
}

impl LegacyQuest {
    pub fn generate(seed: u64, issuer_id: String, issuer_name: String, current_tick: u64) -> Self {
        let mut rng = crate::rng::SeedRng::new(seed);
        let quest_type = match rng.gen_range(5) {
            0 => QuestType::DeliverItem,
            1 => QuestType::GatherResource,
            2 => QuestType::EscortNpc,
            3 => QuestType::FindLocation,
            _ => QuestType::ResolveDispute,
        };

        let (description, target_item, target_count, target_location) = match quest_type {
            QuestType::DeliverItem => {
                let items = [
                    ItemType::Herb,
                    ItemType::Food,
                    ItemType::Cloth,
                    ItemType::Iron,
                ];
                let item = items[rng.gen_range(items.len() as u32) as usize];
                let count = 2 + rng.gen_range(4);
                (
                    format!("Deliver {} {} to a contact", count, item.name()),
                    Some(item),
                    count,
                    None,
                )
            }
            QuestType::GatherResource => {
                let items = [ItemType::Wood, ItemType::Stone, ItemType::Herb];
                let item = items[rng.gen_range(items.len() as u32) as usize];
                let count = 3 + rng.gen_range(5);
                (
                    format!("Gather {} {}", count, item.name()),
                    Some(item),
                    count,
                    None,
                )
            }
            QuestType::EscortNpc => (
                "Escort a traveler safely to their destination".to_string(),
                None,
                1,
                Some("nearby settlement".to_string()),
            ),
            QuestType::FindLocation => {
                let locations = [
                    "ancient ruins",
                    "hidden cave",
                    "forgotten shrine",
                    "old camp",
                ];
                let loc = locations[rng.gen_range(locations.len() as u32) as usize];
                (format!("Find the {}", loc), None, 1, Some(loc.to_string()))
            }
            QuestType::ResolveDispute => (
                "Mediate a dispute between two parties".to_string(),
                None,
                1,
                None,
            ),
        };

        let reward_coins = 3 + rng.gen_range(8);
        let reward_reputation = 0.05 + rng.gen_range(10) as f64 / 100.0;
        let deadline_tick = current_tick + 48 + rng.gen_range(72) as u64;

        LegacyQuest {
            id: format!("quest-{:016x}", seed),
            quest_type,
            description,
            issuer_id,
            issuer_name,
            target_item,
            target_count,
            target_location,
            reward_coins,
            reward_reputation,
            deadline_tick,
            accepted: false,
            completed: false,
            progress: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= self.target_count
    }

    pub fn advance_progress(&mut self, amount: u32) {
        self.progress = (self.progress + amount).min(self.target_count);
        if self.is_complete() {
            self.completed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_progress_hint_near_complete() {
        let q = Quest {
            kind: QuestKind::FetchItem {
                item: ItemType::Herb,
                count: 4,
            },
            description: "The settlement needs what the land sometimes gives.".into(),
            reward: QuestReward::Reputation { amount: 0.1 },
            progress: 3,
            target: 4,
            deadline_day: 30,
            assigned_day: 1,
        };
        assert_eq!(q.progress_hint(), "I have nearly enough.");
    }

    #[test]
    fn quest_progress_hint_just_started() {
        let q = Quest {
            kind: QuestKind::SurviveDays { days: 10 },
            description: "The deep world will test me.".into(),
            reward: QuestReward::Reputation { amount: 0.2 },
            progress: 0,
            target: 10,
            deadline_day: 30,
            assigned_day: 1,
        };
        assert_eq!(q.progress_hint(), "The settlement needs more.");
    }

    #[test]
    fn quest_is_complete() {
        let q = Quest {
            kind: QuestKind::FetchItem {
                item: ItemType::Food,
                count: 3,
            },
            description: "test".into(),
            reward: QuestReward::Items {
                item: ItemType::Food,
                count: 5,
            },
            progress: 3,
            target: 3,
            deadline_day: 30,
            assigned_day: 1,
        };
        assert!(q.is_complete());
    }

    #[test]
    fn quest_is_expired() {
        let q = Quest {
            kind: QuestKind::VisitRegion { region_idx: 0 },
            description: "test".into(),
            reward: QuestReward::Items {
                item: ItemType::Herb,
                count: 3,
            },
            progress: 0,
            target: 1,
            deadline_day: 5,
            assigned_day: 1,
        };
        assert!(q.is_expired(6));
        assert!(!q.is_expired(5));
        assert!(!q.is_expired(4));
    }

    #[test]
    fn legacy_quest_generate_deterministic() {
        let q1 = LegacyQuest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        let q2 = LegacyQuest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        assert_eq!(q1.id, q2.id);
        assert_eq!(q1.quest_type, q2.quest_type);
    }

    #[test]
    fn legacy_quest_types_variety() {
        let mut types = std::collections::HashSet::new();
        for seed in 0..20 {
            let q = LegacyQuest::generate(seed, "npc-1".into(), "Test NPC".into(), 100);
            types.insert(q.quest_type);
        }
        assert!(types.len() >= 3, "should generate at least 3 different quest types");
    }

    #[test]
    fn legacy_quest_progress_tracking() {
        let mut q = LegacyQuest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        assert!(!q.is_complete());
        q.advance_progress(1);
        if q.target_count > 1 {
            assert!(!q.is_complete());
        }
        q.advance_progress(q.target_count);
        assert!(q.is_complete());
        assert!(q.completed);
    }

    #[test]
    fn legacy_quest_deadline_in_future() {
        let q = LegacyQuest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        assert!(q.deadline_tick > 100);
        assert!(q.deadline_tick <= 100 + 48 + 72);
    }

    #[test]
    fn legacy_quest_roundtrip() {
        let q = LegacyQuest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        let ser = ron::ser::to_string(&q).unwrap();
        let de: LegacyQuest = ron::from_str(&ser).unwrap();
        assert_eq!(q, de);
    }

    #[test]
    fn quest_kind_serde_roundtrip() {
        let kinds = vec![
            QuestKind::FetchItem { item: ItemType::Herb, count: 3 },
            QuestKind::VisitRegion { region_idx: 2 },
            QuestKind::AidNPC { npc_id: "abc".into() },
            QuestKind::ReachReputation { threshold: 0.7 },
            QuestKind::SurviveDays { days: 5 },
        ];
        for kind in kinds {
            let ser = ron::ser::to_string(&kind).unwrap();
            let de: QuestKind = ron::from_str(&ser).unwrap();
            assert_eq!(kind, de);
        }
    }
}