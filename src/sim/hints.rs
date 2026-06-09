use crate::model::PeopleKind;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HintTracker {
    #[serde(default)]
    hints_shown: Vec<String>,
}

impl HintTracker {
    pub fn should_show(&self, hint_id: &str) -> bool {
        !self.hints_shown.iter().any(|h| h == hint_id)
    }

    pub fn mark_shown(&mut self, hint_id: &str) {
        if self.should_show(hint_id) {
            self.hints_shown.push(hint_id.to_string());
        }
    }
}

pub const HINT_FIRST_GATHER: &str = "first_gather";
pub const HINT_FIRST_REST: &str = "first_rest";
pub const HINT_FIRST_TRADE: &str = "first_trade";
pub const HINT_FIRST_ENCOUNTER: &str = "first_encounter";
pub const HINT_FIRST_COLLAPSE: &str = "first_collapse";
pub const HINT_FIRST_STRUCTURE: &str = "first_structure";

pub fn hint_text(hint_id: &str) -> Option<&'static str> {
    match hint_id {
        HINT_FIRST_GATHER => {
            Some("I pick through what the land offers. Some places give more than others.")
        }
        HINT_FIRST_REST => {
            Some("I find shelter and close my eyes. The deep world waits for no one.")
        }
        HINT_FIRST_TRADE => Some("Coin changes hands. Trust is harder to trade."),
        HINT_FIRST_ENCOUNTER => Some("The world has teeth. I should be careful where I walk."),
        HINT_FIRST_COLLAPSE => {
            Some("I stumble. The ground rises to meet me. I must rest more, eat more.")
        }
        HINT_FIRST_STRUCTURE => Some("Shelter takes shape. The land rewards patience."),
        _ => None,
    }
}

pub fn people_flavor(people: PeopleKind) -> &'static str {
    match people {
        PeopleKind::Metsik => "Forest people, known for hospitality",
        PeopleKind::Arkit => "Keepers of the archive, bound to memory",
        PeopleKind::Vayla => "River-folk, steady as the current",
        PeopleKind::Laakso => "Vale-dwellers, quiet and enduring",
        PeopleKind::Sepat => "Iron-people, forge-workers of the mountain",
        PeopleKind::Ahjo => "Sacred-forge folk, born of flame and steel",
        PeopleKind::Varhaiset => "The earliest ones, highland headwaters folk",
        PeopleKind::Metsareunat => "Forest-edge people, where clearing meets wood",
        PeopleKind::Porokansa => "Reindeer-following people, herd and horizon",
        PeopleKind::Koskimetsa => "Rapid-forest people, between white water and green shade",
        PeopleKind::Muistikansa => "Memory-keepers, oral tradition carriers",
        PeopleKind::Taulukansa => "Tablet-people, independent scribes",
        PeopleKind::Kirjakansa => "Book-people, river-valley scribes",
        PeopleKind::Takovaki => "Surface-copper forge folk",
        PeopleKind::Rantavaki => "Shore-folk, tidepool cultivators",
        PeopleKind::Saarivaki => "Island-folk, inland sea traders",
        PeopleKind::Hiekkakavelijat => "Sand-walkers, beach-ridge lagoon folk",
        PeopleKind::Haramaki => "Steep-valley people, dry-terrace farmers",
        PeopleKind::Jamavaki => "Hidden-valley people, hermit traditions",
        PeopleKind::Pohjavaki => "Deep-bottom people, silence keepers",
        PeopleKind::Tzakhar => "Deep-cave people, voices in the stone",
        PeopleKind::Merak => "Sea-people, salt and storm",
        PeopleKind::Shear => "Desert people, walkers of the waste",
        PeopleKind::Hal => "Canopy people, dwelling among the branches",
        PeopleKind::Khor => "Tundra people, wind and frost",
    }
}

pub fn profession_flavor(profession: &str) -> &'static str {
    match profession {
        "herbalist" => "I work with what grows. The land provides.",
        "builder" => "I build what lasts. Shelter for those who follow.",
        "trader" => "The road is my market. Coin follows trust.",
        "farmer" => "I tend what feeds. Patience is my craft.",
        "forester" => "The forest is my charge. I know every path.",
        "healer" => "I ease what ails. Herbs and quiet hands.",
        "scholar" => "I preserve what others forget. Words are my tools.",
        "smith" => "Fire and hammer. I shape what the forge gives.",
        "hunter" => "I track what moves. The wild provides.",
        "weaver" => "Thread and pattern. I make what clothes.",
        "fisher" => "Water provides. I take only what the current offers.",
        "guard" => "I watch. Danger comes; I stand.",
        "miner" => "Below the surface, I find what the earth hides.",
        "cook" => "Fire and flavor. I feed what sustains.",
        "shepherd" => "I follow the flock. The herd is my life.",
        "mason" => "Stone and mortar. I raise what endures.",
        "bard" => "I carry the songs. Memory set to melody.",
        "priest" => "I tend the sacred. The gods listen.",
        "carpenter" => "Wood and saw. I shape shelter from the forest.",
        _ => "I do what must be done.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_show_returns_true_for_unseen() {
        let tracker = HintTracker::default();
        assert!(tracker.should_show("first_gather"));
    }

    #[test]
    fn should_show_returns_false_after_mark() {
        let mut tracker = HintTracker::default();
        tracker.mark_shown("first_gather");
        assert!(!tracker.should_show("first_gather"));
    }

    #[test]
    fn mark_shown_idempotent() {
        let mut tracker = HintTracker::default();
        tracker.mark_shown("first_rest");
        tracker.mark_shown("first_rest");
        assert_eq!(tracker.hints_shown.len(), 1);
    }

    #[test]
    fn multiple_hints_independent() {
        let mut tracker = HintTracker::default();
        tracker.mark_shown("first_gather");
        assert!(tracker.should_show("first_rest"));
        assert!(!tracker.should_show("first_gather"));
    }

    #[test]
    fn hint_text_known_ids() {
        assert!(hint_text("first_gather").is_some());
        assert!(hint_text("first_rest").is_some());
        assert!(hint_text("first_trade").is_some());
        assert!(hint_text("first_encounter").is_some());
        assert!(hint_text("first_collapse").is_some());
        assert!(hint_text("first_structure").is_some());
    }

    #[test]
    fn hint_text_unknown_returns_none() {
        assert!(hint_text("nonexistent").is_none());
    }

    #[test]
    fn hint_text_no_forbidden_you() {
        let ids = [
            "first_gather",
            "first_rest",
            "first_trade",
            "first_encounter",
            "first_collapse",
            "first_structure",
        ];
        for id in &ids {
            if let Some(text) = hint_text(id) {
                let lower = text.to_lowercase();
                assert!(
                    !lower.contains(" you "),
                    "Hint '{}' contains forbidden 'you': {}",
                    id,
                    text
                );
                assert!(
                    !lower.starts_with("you "),
                    "Hint '{}' starts with 'you': {}",
                    id,
                    text
                );
            }
        }
    }

    #[test]
    fn tracker_serde_roundtrip() {
        let mut tracker = HintTracker::default();
        tracker.mark_shown("first_gather");
        tracker.mark_shown("first_rest");
        let ron_str = ron::to_string(&tracker).unwrap();
        let loaded: HintTracker = ron::from_str(&ron_str).unwrap();
        assert!(!loaded.should_show("first_gather"));
        assert!(!loaded.should_show("first_rest"));
        assert!(loaded.should_show("first_trade"));
    }
}
