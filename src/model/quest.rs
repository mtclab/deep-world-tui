use serde::{Deserialize, Serialize};

use super::ItemType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestKind {
    FetchItem {
        item: ItemType,
        count: u32,
    },
    VisitRegion {
        region_idx: usize,
    },
    AidNPC {
        npc_id: String,
    },
    ReachReputation {
        threshold: f64,
    },
    SurviveDays {
        days: u32,
    },
    /// Carry goods to another region's market (consumed on arrival).
    DeliverTo {
        region_idx: usize,
        item: ItemType,
        count: u32,
    },
    /// Raise any structure of your own in the named region.
    RaiseBuilding {
        region_idx: usize,
    },
    /// Find and observe somewhere new.
    VisitDiscovery {
        baseline: u32,
    },
    /// Deal with people — talks, trades, dealings remembered.
    TalkToPeople {
        count: u32,
    },
    /// A town the living world has thrown into need calls for the player
    /// (#613-epic): relieve a plague by bringing medicine until the sickness
    /// lifts. Progress is set by the act of tending, not recomputed.
    RelievePlague {
        settlement: String,
    },
    /// Provision a town gone to famine until its stores recover.
    RelieveFamine {
        settlement: String,
    },
    /// Two towns at deep, raiding rivalry call for a peacemaker (#614): carry
    /// goods between them until the bad blood eases out of rivalry. Towns stored
    /// name-ordered so the pair is the same either way.
    BrokerTruce {
        a: String,
        b: String,
    },
    /// A town whose faith has split near-even calls for a devotee to come and
    /// steady it (#614): make an offering there to tip the balance and quiet the
    /// looming schism. Resolved by the act of devotion, not recomputed.
    SteadyFaith {
        settlement: String,
    },
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
    fn quest_kind_serde_roundtrip() {
        let kinds = vec![
            QuestKind::FetchItem {
                item: ItemType::Herb,
                count: 3,
            },
            QuestKind::VisitRegion { region_idx: 2 },
            QuestKind::AidNPC {
                npc_id: "abc".into(),
            },
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
