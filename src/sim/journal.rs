use crate::model::{TimeOfDay, Weather};
use crate::rng::SeedRng;
use serde::Deserializer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Voice {
    Encounter,
    Travel,
    Rest,
    Dream,
    Scar,
    Rumor,
}

impl Voice {
    pub fn label(self) -> &'static str {
        match self {
            Voice::Encounter => "encounter",
            Voice::Travel => "travel",
            Voice::Rest => "rest",
            Voice::Dream => "dream",
            Voice::Scar => "scar",
            Voice::Rumor => "rumor",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct JournalEntry {
    pub tick: u64,
    pub voice: Voice,
    pub text: String,
}

impl<'de> serde::Deserialize<'de> for JournalEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        fn default_voice() -> Voice {
            Voice::Encounter
        }
        #[derive(serde::Deserialize)]
        struct OldEntry {
            tick: u64,
            // Plain Voice (not Option): derived Serialize writes a bare
            // `voice:Scar`, and compact RON's reader has no implicit-some, so a
            // bare enum into Option<Voice> fails with "Expected option". The
            // default fn still migrates older saves that lacked the field.
            #[serde(default = "default_voice")]
            voice: Voice,
            text: String,
        }
        let old = OldEntry::deserialize(deserializer)?;
        Ok(JournalEntry {
            tick: old.tick,
            voice: old.voice,
            text: old.text,
        })
    }
}

const MAX_JOURNAL: usize = 200;

#[derive(Debug, Clone, Default)]
pub struct Journal {
    pub entries: Vec<JournalEntry>,
}

impl Journal {
    pub fn log(&mut self, tick: u64, voice: Voice, text: String) {
        if self.entries.len() >= MAX_JOURNAL {
            // Drop the oldest quarter in one shift instead of remove(0) per
            // entry — at the cap, every log used to pay an O(n) front-shift.
            self.entries.drain(0..MAX_JOURNAL / 4);
        }
        self.entries.push(JournalEntry { tick, voice, text });
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, JournalEntry> {
        self.entries.iter()
    }

    pub fn iter_rev(&self) -> std::iter::Rev<std::slice::Iter<'_, JournalEntry>> {
        self.entries.iter().rev()
    }
}

impl serde::Serialize for Journal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.entries.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Journal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries: Vec<JournalEntry> = Vec::deserialize(deserializer)?;
        Ok(Journal { entries })
    }
}

fn pick_template(rng: &mut SeedRng, templates: &'static [String]) -> &'static str {
    let idx = rng.gen_range(templates.len() as u32) as usize;
    &templates[idx % templates.len()]
}

pub fn travel_text(rng: &mut SeedRng, tod: TimeOfDay, weather: Weather) -> String {
    let templates = match (&weather, &tod) {
        (Weather::Storm | Weather::Thunderhead | Weather::SeaSquall, _) => {
            crate::banks::bank("TRAVEL_STORM")
        }
        (Weather::Snow | Weather::Whiteout, _) => crate::banks::bank("TRAVEL_SNOW"),
        (Weather::Fog, _) => crate::banks::bank("TRAVEL_FOG"),
        (Weather::Clear, TimeOfDay::Dawn) => crate::banks::bank("TRAVEL_CLEAR_DAWN"),
        _ => match tod {
            TimeOfDay::Dawn => crate::banks::bank("TRAVEL_DAWN"),
            TimeOfDay::Morning => crate::banks::bank("TRAVEL_MORNING"),
            TimeOfDay::Noon => crate::banks::bank("TRAVEL_NOON"),
            TimeOfDay::Afternoon => crate::banks::bank("TRAVEL_AFTERNOON"),
            TimeOfDay::Dusk => crate::banks::bank("TRAVEL_DUSK"),
            TimeOfDay::Night => crate::banks::bank("TRAVEL_NIGHT"),
            TimeOfDay::DeepNight => crate::banks::bank("TRAVEL_DEEP_NIGHT"),
        },
    };
    pick_template(rng, templates).into()
}

pub fn encounter_text(rng: &mut SeedRng) -> String {
    pick_template(rng, crate::banks::bank("ENCOUNTER_TEMPLATES")).into()
}

pub fn rest_text(rng: &mut SeedRng, quality: &str) -> String {
    let templates = match quality {
        "inn" => crate::banks::bank("REST_INN"),
        "settlement" => crate::banks::bank("REST_SETTLEMENT"),
        "campfire" => crate::banks::bank("REST_CAMPFIRE"),
        "lean_to" => crate::banks::bank("REST_LEAN_TO"),
        _ => crate::banks::bank("REST_OUTSIDE"),
    };
    pick_template(rng, templates).into()
}

pub fn dream_text(rng: &mut SeedRng) -> String {
    pick_template(rng, crate::banks::bank("DREAM_TEMPLATES")).into()
}

pub fn scar_text(rng: &mut SeedRng) -> String {
    pick_template(rng, crate::banks::bank("SCAR_TEMPLATES")).into()
}

pub fn rumor_text(rng: &mut SeedRng) -> String {
    // The talk: wider-world news, local color, and — now and then — a deniable
    // word of the uncanny (#455), the myth-creatures heard of before they are
    // ever met.
    match rng.gen_range(5) {
        0 | 1 => pick_template(rng, crate::banks::bank("CANON_RUMORS")).to_string(),
        2 => pick_template(rng, crate::banks::bank("UNCANNY_RUMORS")).to_string(),
        _ => pick_template(rng, crate::banks::bank("RUMOR_TEMPLATES")).to_string(),
    }
}

pub fn rest_quality_label(
    on_settlement: bool,
    inn_paid: bool,
    has_campfire: bool,
    has_lean_to: bool,
) -> &'static str {
    if inn_paid {
        "inn"
    } else if on_settlement {
        "settlement"
    } else if has_campfire {
        "campfire"
    } else if has_lean_to {
        "lean_to"
    } else {
        "outside"
    }
}

pub fn voice_color(voice: Voice) -> &'static str {
    match voice {
        Voice::Encounter => "ink",
        Voice::Travel => "warm_brown",
        Voice::Rest => "dark_ink",
        Voice::Dream => "archive_red",
        Voice::Scar => "archive_red",
        Voice::Rumor => "ink",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_labels() {
        assert_eq!(Voice::Encounter.label(), "encounter");
        assert_eq!(Voice::Travel.label(), "travel");
        assert_eq!(Voice::Rest.label(), "rest");
        assert_eq!(Voice::Dream.label(), "dream");
        assert_eq!(Voice::Scar.label(), "scar");
        assert_eq!(Voice::Rumor.label(), "rumor");
    }

    #[test]
    fn journal_log_and_trim() {
        // The cap drops the oldest quarter in one amortized drain rather than
        // an O(n) remove(0) per entry. Never exceeds MAX_JOURNAL; newest kept.
        let mut j = Journal::default();
        for i in 0..205 {
            j.log(i, Voice::Travel, format!("entry {i}"));
        }
        assert!(j.entries.len() <= 200);
        assert_eq!(j.entries.last().unwrap().tick, 204);
        assert!(j.entries[0].tick >= 5, "oldest entries dropped");
    }

    #[test]
    fn travel_text_deterministic() {
        let mut rng = SeedRng::new(42);
        let a = travel_text(&mut rng, TimeOfDay::Dawn, Weather::Clear);
        let mut rng2 = SeedRng::new(42);
        let b = travel_text(&mut rng2, TimeOfDay::Dawn, Weather::Clear);
        assert_eq!(a, b);
    }

    #[test]
    fn encounter_text_deterministic() {
        let mut rng = SeedRng::new(99);
        let a = encounter_text(&mut rng);
        let mut rng2 = SeedRng::new(99);
        let b = encounter_text(&mut rng2);
        assert_eq!(a, b);
    }

    #[test]
    fn rest_text_inn_deterministic() {
        let mut rng = SeedRng::new(7);
        let a = rest_text(&mut rng, "inn");
        let mut rng2 = SeedRng::new(7);
        let b = rest_text(&mut rng2, "inn");
        assert_eq!(a, b);
    }

    #[test]
    fn scar_text_deterministic() {
        let mut rng = SeedRng::new(13);
        let a = scar_text(&mut rng);
        let mut rng2 = SeedRng::new(13);
        let b = scar_text(&mut rng2);
        assert_eq!(a, b);
    }

    #[test]
    fn dream_text_deterministic() {
        let mut rng = SeedRng::new(21);
        let a = dream_text(&mut rng);
        let mut rng2 = SeedRng::new(21);
        let b = dream_text(&mut rng2);
        assert_eq!(a, b);
    }

    #[test]
    fn rumor_text_deterministic() {
        let mut rng = SeedRng::new(33);
        let a = rumor_text(&mut rng);
        let mut rng2 = SeedRng::new(33);
        let b = rumor_text(&mut rng2);
        assert_eq!(a, b);
    }

    #[test]
    fn no_you_in_templates() {
        let all_templates: Vec<&[String]> = vec![
            crate::banks::bank("TRAVEL_DAWN"),
            crate::banks::bank("TRAVEL_MORNING"),
            crate::banks::bank("TRAVEL_NOON"),
            crate::banks::bank("TRAVEL_AFTERNOON"),
            crate::banks::bank("TRAVEL_DUSK"),
            crate::banks::bank("TRAVEL_NIGHT"),
            crate::banks::bank("TRAVEL_DEEP_NIGHT"),
            crate::banks::bank("TRAVEL_STORM"),
            crate::banks::bank("TRAVEL_SNOW"),
            crate::banks::bank("TRAVEL_FOG"),
            crate::banks::bank("TRAVEL_CLEAR_DAWN"),
            crate::banks::bank("ENCOUNTER_TEMPLATES"),
            crate::banks::bank("REST_CAMPFIRE"),
            crate::banks::bank("REST_OUTSIDE"),
            crate::banks::bank("REST_SETTLEMENT"),
            crate::banks::bank("REST_INN"),
            crate::banks::bank("REST_LEAN_TO"),
            crate::banks::bank("DREAM_TEMPLATES"),
            crate::banks::bank("SCAR_TEMPLATES"),
            crate::banks::bank("RUMOR_TEMPLATES"),
        ];
        for group in &all_templates {
            for t in *group {
                let lower = t.to_lowercase();
                assert!(
                    !lower.contains(" you ")
                        && !lower.starts_with("you ")
                        && !lower.ends_with(" you"),
                    "Template contains forbidden 'you': {t}"
                );
            }
        }
    }

    #[test]
    fn minimum_templates_per_voice() {
        assert!(crate::banks::bank("TRAVEL_DAWN").len() >= 3);
        assert!(crate::banks::bank("TRAVEL_MORNING").len() >= 3);
        assert!(crate::banks::bank("TRAVEL_NOON").len() >= 3);
        assert!(crate::banks::bank("TRAVEL_AFTERNOON").len() >= 3);
        assert!(crate::banks::bank("TRAVEL_DUSK").len() >= 3);
        assert!(crate::banks::bank("TRAVEL_NIGHT").len() >= 3);
        assert!(crate::banks::bank("TRAVEL_DEEP_NIGHT").len() >= 3);
        assert!(crate::banks::bank("ENCOUNTER_TEMPLATES").len() >= 3);
        assert!(crate::banks::bank("REST_CAMPFIRE").len() >= 3);
        assert!(crate::banks::bank("REST_OUTSIDE").len() >= 3);
        assert!(crate::banks::bank("REST_SETTLEMENT").len() >= 3);
        assert!(crate::banks::bank("REST_INN").len() >= 3);
        assert!(crate::banks::bank("REST_LEAN_TO").len() >= 3);
        assert!(crate::banks::bank("DREAM_TEMPLATES").len() >= 3);
        assert!(crate::banks::bank("SCAR_TEMPLATES").len() >= 3);
        assert!(crate::banks::bank("RUMOR_TEMPLATES").len() >= 3);
        assert!(crate::banks::bank("UNCANNY_RUMORS").len() >= 6);
    }

    #[test]
    fn tavern_talk_carries_word_of_the_uncanny() {
        // Over enough tavern talk, the deniable rumours of the myth-creatures
        // surface — the uncanny is heard of before it is ever met (#455).
        let uncanny = crate::banks::bank("UNCANNY_RUMORS");
        let mut heard = false;
        for seed in 0..400u64 {
            let mut rng = SeedRng::new(seed).fork_for("tavern");
            let line = rumor_text(&mut rng);
            if uncanny.contains(&line) {
                heard = true;
                break;
            }
        }
        assert!(heard, "the uncanny should be heard of in the taverns");
    }

    #[test]
    fn voice_color_mapping() {
        assert_eq!(voice_color(Voice::Encounter), "ink");
        assert_eq!(voice_color(Voice::Travel), "warm_brown");
        assert_eq!(voice_color(Voice::Rest), "dark_ink");
        assert_eq!(voice_color(Voice::Dream), "archive_red");
        assert_eq!(voice_color(Voice::Scar), "archive_red");
        assert_eq!(voice_color(Voice::Rumor), "ink");
    }
}
