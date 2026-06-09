use crate::model::PeopleKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MilestoneKind {
    Survived30Days,
    Survived100Days,
    Survived300Days,
    Survived365Days,
    FirstStructureBuilt,
    FirstCompanionAdopted,
    PeopleEnding { people: PeopleKind },
    ElderAchieved,
    FirstSettlement,
    FirstQuestCompleted,
    FirstDeath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CampaignArc {
    Stranger,
    Settled,
    Known,
    Elder,
    Legend,
}

impl CampaignArc {
    pub fn from_milestones(tracker: &MilestoneTracker) -> Self {
        if tracker.has(MilestoneKind::ElderAchieved) && tracker.settlements_visited >= 5 {
            CampaignArc::Legend
        } else if tracker.has(MilestoneKind::ElderAchieved) {
            CampaignArc::Elder
        } else if tracker.has(MilestoneKind::FirstQuestCompleted) {
            CampaignArc::Known
        } else if tracker.has(MilestoneKind::FirstSettlement) {
            CampaignArc::Settled
        } else {
            CampaignArc::Stranger
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CampaignArc::Stranger => "Stranger",
            CampaignArc::Settled => "Settled",
            CampaignArc::Known => "Known",
            CampaignArc::Elder => "Elder",
            CampaignArc::Legend => "Legend",
        }
    }

    pub fn arc_text(self) -> &'static str {
        match self {
            CampaignArc::Stranger => "The world does not know you yet. Every path is untrodden.",
            CampaignArc::Settled => "You have found a place. Now it begins to matter what you do here.",
            CampaignArc::Known => "Your name carries weight. Choices have consequences beyond yourself.",
            CampaignArc::Elder => "Time has worn you wise. The young seek your counsel.",
            CampaignArc::Legend => "Your story is told around fires you will never see. You are legend.",
        }
    }
}

impl MilestoneKind {
    pub fn journal_text(self) -> String {
        match self {
            MilestoneKind::Survived30Days => {
                "A month behind me. The land has not swallowed me yet.".into()
            }
            MilestoneKind::Survived100Days => {
                "A hundred dawns. I have learned the rhythm of survival.".into()
            }
            MilestoneKind::Survived300Days => {
                "Three hundred days. I am no longer a stranger to this world.".into()
            }
            MilestoneKind::Survived365Days => {
                "A full turn of seasons. This land knows my name now.".into()
            }
            MilestoneKind::FirstStructureBuilt => {
                "I raised shelter. The wind no longer finds me unprepared.".into()
            }
            MilestoneKind::FirstCompanionAdopted => {
                "A companion joins me. I am not alone.".into()
            }
            MilestoneKind::PeopleEnding { people } => {
                let label = people.label();
                let desc = people.description();
                match people {
                    PeopleKind::Metsik => format!(
                        "The {} — {} — call me their own. The forest opens for me.",
                        label, desc
                    ),
                    PeopleKind::Arkit => format!(
                        "The {} — {} — have inscribed my name in their rolls. The archive remembers.",
                        label, desc
                    ),
                    PeopleKind::Vayla => format!(
                        "The {} — {} — welcome me at every crossing. I am known on the rivers.",
                        label, desc
                    ),
                    PeopleKind::Laakso => format!(
                        "The {} — {} — speak my name in the deep places. I am trusted in the vales.",
                        label, desc
                    ),
                    PeopleKind::Sepat => format!(
                        "The {} — {} — have forged a place for me. The iron honors my hands.",
                        label, desc
                    ),
                    PeopleKind::Ahjo => format!(
                        "The {} — {} — have admitted me to the sacred forge. I am bound to their fire.",
                        label, desc
                    ),
                    _ => format!(
                        "The {} — {} — have welcomed me as one of their own.",
                        label, desc
                    ),
                }
            }
            MilestoneKind::ElderAchieved => {
                "I have walked this world longer than most dare dream. The young ones seek my counsel. I am elder.".into()
            }
            MilestoneKind::FirstSettlement => {
                "I found settlement. Someone else's fire, someone else's walls. I step closer.".into()
            }
            MilestoneKind::FirstQuestCompleted => {
                "I fulfilled a promise. The world takes note — I am not just passing through.".into()
            }
            MilestoneKind::FirstDeath => {
                "The world did not end. It merely paused, and then continued without me. I remember still.".into()
            }
        }
    }

    pub fn voice(self) -> crate::sim::journal::Voice {
        match self {
            MilestoneKind::PeopleEnding { .. } => crate::sim::journal::Voice::Dream,
            _ => crate::sim::journal::Voice::Travel,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub kind: MilestoneKind,
    pub day: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MilestoneTracker {
    pub milestones: Vec<Milestone>,
    pub settlements_visited: u32,
    pub quests_completed: u32,
}

impl MilestoneTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, kind: MilestoneKind) -> bool {
        self.milestones.iter().any(|m| m.kind == kind)
    }

    pub fn record(&mut self, kind: MilestoneKind, day: u32) -> bool {
        if self.has(kind) {
            return false;
        }
        self.milestones.push(Milestone { kind, day });
        true
    }

    pub fn check_day_milestones(&mut self, day: u32) -> Vec<MilestoneKind> {
        let mut fired = Vec::new();
        let checks: &[(u32, MilestoneKind)] = &[
            (30, MilestoneKind::Survived30Days),
            (100, MilestoneKind::Survived100Days),
            (300, MilestoneKind::Survived300Days),
            (365, MilestoneKind::Survived365Days),
        ];
        for &(threshold, kind) in checks {
            if day >= threshold && !self.has(kind) {
                self.record(kind, day);
                fired.push(kind);
            }
        }
        fired
    }

    pub fn check_elder(&mut self, day: u32) -> bool {
        if day >= 300 && !self.has(MilestoneKind::ElderAchieved) {
            self.record(MilestoneKind::ElderAchieved, day);
            true
        } else {
            false
        }
    }

    pub fn check_first_settlement(&mut self, day: u32) -> bool {
        if self.settlements_visited >= 1 && !self.has(MilestoneKind::FirstSettlement) {
            self.record(MilestoneKind::FirstSettlement, day);
            true
        } else {
            false
        }
    }

    pub fn record_quest_completed(&mut self, day: u32) -> bool {
        self.quests_completed += 1;
        if self.quests_completed == 1 && !self.has(MilestoneKind::FirstQuestCompleted) {
            self.record(MilestoneKind::FirstQuestCompleted, day);
            true
        } else {
            false
        }
    }

    pub fn record_death(&mut self, day: u32) -> bool {
        if !self.has(MilestoneKind::FirstDeath) {
            self.record(MilestoneKind::FirstDeath, day);
            true
        } else {
            false
        }
    }
}

const SIX_CORE_PEOPLES: &[PeopleKind] = &[
    PeopleKind::Metsik,
    PeopleKind::Arkit,
    PeopleKind::Vayla,
    PeopleKind::Laakso,
    PeopleKind::Sepat,
    PeopleKind::Ahjo,
];

pub fn core_peoples() -> &'static [PeopleKind] {
    SIX_CORE_PEOPLES
}

pub fn faction_key(people: PeopleKind) -> &'static str {
    match people {
        PeopleKind::Metsik => "metsik",
        PeopleKind::Arkit => "arkit",
        PeopleKind::Vayla => "vayla",
        PeopleKind::Laakso => "laakso",
        PeopleKind::Sepat => "sepat",
        PeopleKind::Ahjo => "ahjo",
        _ => "unknown",
    }
}

pub fn legacy_summary(
    milestones: &MilestoneTracker,
    structures_built: usize,
    has_companion: bool,
    faction_rep: impl Fn(PeopleKind) -> f64,
) -> Vec<String> {
    let mut lines = Vec::new();

    if milestones.has(MilestoneKind::ElderAchieved) {
        lines.push("Elder — walked longer than most dare dream.".into());
    }

    let revered: Vec<PeopleKind> = core_peoples()
        .iter()
        .copied()
        .filter(|&p| faction_rep(p) >= 0.9)
        .collect();
    if !revered.is_empty() {
        let names: Vec<&str> = revered.iter().map(|p| p.label()).collect();
        lines.push(format!("Revered by: {}", names.join(", ")));
    }

    if structures_built > 0 {
        lines.push(format!(
            "Built {} shelter{} for the road.",
            structures_built,
            if structures_built > 1 { "s" } else { "" }
        ));
    }

    if has_companion {
        lines.push("Traveled with a faithful companion.".into());
    }

    if milestones.has(MilestoneKind::Survived365Days) {
        lines.push("Survived a full turning of seasons.".into());
    } else if milestones.has(MilestoneKind::Survived100Days) {
        lines.push("Endured a hundred dawns.".into());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_milestones_fire_at_threshold() {
        let mut tracker = MilestoneTracker::new();
        let fired = tracker.check_day_milestones(30);
        assert!(fired.contains(&MilestoneKind::Survived30Days));
        assert!(!tracker
            .check_day_milestones(30)
            .contains(&MilestoneKind::Survived30Days));
    }

    #[test]
    fn day_milestones_no_dupes() {
        let mut tracker = MilestoneTracker::new();
        tracker.check_day_milestones(100);
        tracker.check_day_milestones(100);
        let count30 = tracker
            .milestones
            .iter()
            .filter(|m| m.kind == MilestoneKind::Survived30Days)
            .count();
        let count100 = tracker
            .milestones
            .iter()
            .filter(|m| m.kind == MilestoneKind::Survived100Days)
            .count();
        assert_eq!(count30, 1);
        assert_eq!(count100, 1);
    }

    #[test]
    fn elder_at_day_300() {
        let mut tracker = MilestoneTracker::new();
        assert!(tracker.check_elder(300));
        assert!(!tracker.check_elder(300));
        assert!(tracker.has(MilestoneKind::ElderAchieved));
    }

    #[test]
    fn elder_not_before_300() {
        let mut tracker = MilestoneTracker::new();
        assert!(!tracker.check_elder(299));
    }

    #[test]
    fn people_ending_milestone() {
        let mut tracker = MilestoneTracker::new();
        tracker.record(
            MilestoneKind::PeopleEnding {
                people: PeopleKind::Metsik,
            },
            50,
        );
        assert!(tracker.has(MilestoneKind::PeopleEnding {
            people: PeopleKind::Metsik
        }));
        assert!(!tracker.has(MilestoneKind::PeopleEnding {
            people: PeopleKind::Arkit
        }));
    }

    #[test]
    fn legacy_summary_elder() {
        let mut tracker = MilestoneTracker::new();
        tracker.record(MilestoneKind::ElderAchieved, 300);
        let lines = legacy_summary(&tracker, 2, true, |_| 0.5);
        assert!(lines.iter().any(|l| l.contains("Elder")));
    }

    #[test]
    fn legacy_summary_revered() {
        let tracker = MilestoneTracker::new();
        let lines = legacy_summary(&tracker, 0, false, |p| {
            if p == PeopleKind::Metsik {
                0.95
            } else {
                0.5
            }
        });
        assert!(lines
            .iter()
            .any(|l| l.contains("Revered by") && l.contains("Metsik")));
    }

    #[test]
    fn milestone_voice_travel_for_survival() {
        assert_eq!(
            MilestoneKind::Survived30Days.voice(),
            crate::sim::journal::Voice::Travel
        );
    }

    #[test]
    fn milestone_voice_dream_for_people_ending() {
        assert_eq!(
            MilestoneKind::PeopleEnding {
                people: PeopleKind::Metsik
            }
            .voice(),
            crate::sim::journal::Voice::Dream
        );
    }

    #[test]
    fn first_settlement_milestone() {
        let mut tracker = MilestoneTracker::new();
        assert!(!tracker.has(MilestoneKind::FirstSettlement));
        tracker.settlements_visited = 1;
        assert!(tracker.check_first_settlement(5));
        assert!(tracker.has(MilestoneKind::FirstSettlement));
        assert!(!tracker.check_first_settlement(5));
    }

    #[test]
    fn first_quest_milestone() {
        let mut tracker = MilestoneTracker::new();
        assert!(tracker.record_quest_completed(10));
        assert!(tracker.has(MilestoneKind::FirstQuestCompleted));
        assert_eq!(tracker.quests_completed, 1);
        assert!(!tracker.record_quest_completed(20));
        assert_eq!(tracker.quests_completed, 2);
    }

    #[test]
    fn first_death_milestone() {
        let mut tracker = MilestoneTracker::new();
        assert!(tracker.record_death(50));
        assert!(tracker.has(MilestoneKind::FirstDeath));
        assert!(!tracker.record_death(100));
    }

    #[test]
    fn campaign_arc_progression() {
        let mut tracker = MilestoneTracker::new();
        assert_eq!(CampaignArc::from_milestones(&tracker), CampaignArc::Stranger);

        tracker.settlements_visited = 1;
        tracker.check_first_settlement(5);
        assert_eq!(CampaignArc::from_milestones(&tracker), CampaignArc::Settled);

        tracker.record_quest_completed(10);
        assert_eq!(CampaignArc::from_milestones(&tracker), CampaignArc::Known);

        tracker.check_elder(300);
        assert_eq!(CampaignArc::from_milestones(&tracker), CampaignArc::Elder);

        tracker.settlements_visited = 5;
        assert_eq!(CampaignArc::from_milestones(&tracker), CampaignArc::Legend);
    }

    #[test]
    fn campaign_arc_labels() {
        assert_eq!(CampaignArc::Stranger.label(), "Stranger");
        assert_eq!(CampaignArc::Legend.label(), "Legend");
    }
}
