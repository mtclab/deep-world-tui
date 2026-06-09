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
        #[derive(serde::Deserialize)]
        struct OldEntry {
            tick: u64,
            #[serde(default)]
            voice: Option<Voice>,
            text: String,
        }
        let old = OldEntry::deserialize(deserializer)?;
        Ok(JournalEntry {
            tick: old.tick,
            voice: old.voice.unwrap_or(Voice::Encounter),
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
            self.entries.remove(0);
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

fn pick_template(rng: &mut SeedRng, templates: &[&'static str]) -> &'static str {
    let idx = rng.gen_range(templates.len() as u32) as usize;
    templates[idx % templates.len()]
}

static TRAVEL_DAWN: &[&str] = &[
    "I rose before the light and set out. The road was mine alone.",
    "The sky turned pale as I walked. Another step, another mile.",
    "Dawn broke cold on the path. I kept moving.",
    "First light found me on the road. I did not wait for the sun.",
    "The world woke around me, but I was already walking.",
];

static TRAVEL_MORNING: &[&str] = &[
    "The morning was bright and the road stretched ahead.",
    "I walked through the early hours. The light was kind.",
    "The day opened before me. I walked into it.",
];

static TRAVEL_NOON: &[&str] = &[
    "The sun beat down. I walked the midpoint of the day.",
    "No rest at noon. The road does not pause.",
    "High sun, long shadow beneath me. I kept walking.",
];

static TRAVEL_AFTERNOON: &[&str] = &[
    "The afternoon wore on. I walked the long hours.",
    "Shadows lengthened ahead of me. I followed them.",
    "The day turned. I did not turn with it.",
];

static TRAVEL_DUSK: &[&str] = &[
    "The light was failing when I finally stopped walking.",
    "Dusk caught me between places. I pressed on.",
    "The road grew dark. I did not stop.",
    "Shadows lengthened across my path. I followed them.",
    "Evening found me still on the road. I accept that.",
];

static TRAVEL_NIGHT: &[&str] = &[
    "I walked through the dark. The stars were my only company.",
    "Night on the road is its own kind of quiet. I moved through it.",
    "The world was dark and I was a shadow crossing it.",
    "Only fools travel at night. I kept walking anyway.",
    "The dark road held no comfort. I walked it regardless.",
];

static TRAVEL_DEEP_NIGHT: &[&str] = &[
    "The world slept. I did not.",
    "Deepest night. I moved through it like a ghost.",
    "The dark was absolute. My steps were the only sound.",
];

static TRAVEL_STORM: &[&str] = &[
    "The storm found me on the road. I did not stop.",
    "Rain hammered the path. I walked through it.",
    "Wind tried to turn me back. The wind does not decide.",
    "The sky opened up and I kept walking. What else is there to do?",
];

static TRAVEL_SNOW: &[&str] = &[
    "Snow covered the road. I made my own path.",
    "The path was white and still. I broke the silence with my steps.",
    "Each step left a mark in the snow. I kept stepping.",
];

static TRAVEL_FOG: &[&str] = &[
    "Fog swallowed the road ahead. I walked into it.",
    "The mist was thick. I trusted my feet.",
    "I could barely see. I walked anyway.",
];

static TRAVEL_CLEAR_DAWN: &[&str] = &[
    "A clear dawn. The road stretched out before me like a promise.",
    "The morning was clean and bright. I took it as a sign.",
];

static ENCOUNTER_TEMPLATES: &[&str] = &[
    "Another face on the road. Words were exchanged. That is all.",
    "Someone crossed my path. We spoke. The world is full of such moments.",
    "I met a stranger. The meeting was brief. It stays with me.",
    "An encounter. Brief, but not without weight.",
    "The road provides company, sometimes. Today it did — for a moment.",
];

static REST_CAMPFIRE: &[&str] = &[
    "The fire is small. I am tired. I sleep.",
    "I built a fire and watched it until my eyes closed.",
    "The flames are low. The night is long. I rest.",
    "A small fire. A small comfort. I sleep.",
    "The fire crackles. I let myself close my eyes.",
];

static REST_OUTSIDE: &[&str] = &[
    "No fire tonight. I sleep with one eye open.",
    "The ground is hard. The sky is wide. I lie down anyway.",
    "I sleep in the open. The cold is an old companion.",
    "No shelter. I rest against the earth and hope for dawn.",
    "The stars are my roof. It is not enough. I sleep anyway.",
];

static REST_SETTLEMENT: &[&str] = &[
    "A roof overhead. The floor is hard but it is warm.",
    "I sleep under a proper roof. The walls hold back the world.",
    "The settlement takes me in. I rest on solid ground.",
    "Shelter. The simple kind. I am grateful.",
    "Four walls and a bed. It is enough.",
];

static REST_INN: &[&str] = &[
    "A proper bed. Soft linen. I sleep the sleep of the fortunate.",
    "The inn is warm. The mattress yields. I let myself rest deeply.",
    "For tonight, I am not on the road. I sleep without fear.",
    "Heat from the hearth. A blanket. I close my eyes and do not dream.",
    "The innkeeper asked no questions. I gave no answers. I slept well.",
];

static REST_LEAN_TO: &[&str] = &[
    "A lean-to shields me from the worst of it. I sleep lightly.",
    "The wind is quieter here. I close my eyes against the rough wood.",
    "Rough shelter, but shelter. I rest.",
    "The lean-to creaks. I sleep anyway.",
];

static DREAM_TEMPLATES: &[&str] = &[
    "I dreamed of a vast archive. The shelves went on forever.",
    "In my sleep, a voice spoke from the deep. I did not understand it.",
    "I dreamed of walking a road that had no end. I was not tired.",
    "The dream was of water — cold, clear, rising. I did not drown.",
    "A figure stood at the crossroads in my dream. It pointed. I did not follow.",
];

static SCAR_TEMPLATES: &[&str] = &[
    "I will not forget the teeth.",
    "The wound is closed. The memory is open.",
    "Pain teaches. This lesson is carved in skin.",
    "Another mark. Another story I do not tell.",
    "The body heals. The body remembers.",
];

static RUMOR_TEMPLATES: &[&str] = &[
    "I heard something. Whether it is true, I cannot say.",
    "A whisper reached me. I carry it now.",
    "They spoke of something. I listened. That is all.",
    "Rumor travels faster than people. It found me.",
    "Words from a stranger's mouth. I file them away.",
];

pub fn travel_text(rng: &mut SeedRng, tod: TimeOfDay, weather: Weather) -> String {
    let templates = match (&weather, &tod) {
        (Weather::Storm | Weather::Thunderhead | Weather::SeaSquall, _) => TRAVEL_STORM,
        (Weather::Snow | Weather::Whiteout, _) => TRAVEL_SNOW,
        (Weather::Fog, _) => TRAVEL_FOG,
        (Weather::Clear, TimeOfDay::Dawn) => TRAVEL_CLEAR_DAWN,
        _ => match tod {
            TimeOfDay::Dawn => TRAVEL_DAWN,
            TimeOfDay::Morning => TRAVEL_MORNING,
            TimeOfDay::Noon => TRAVEL_NOON,
            TimeOfDay::Afternoon => TRAVEL_AFTERNOON,
            TimeOfDay::Dusk => TRAVEL_DUSK,
            TimeOfDay::Night => TRAVEL_NIGHT,
            TimeOfDay::DeepNight => TRAVEL_DEEP_NIGHT,
        },
    };
    pick_template(rng, templates).into()
}

pub fn encounter_text(rng: &mut SeedRng) -> String {
    pick_template(rng, ENCOUNTER_TEMPLATES).into()
}

pub fn rest_text(rng: &mut SeedRng, quality: &str) -> String {
    let templates = match quality {
        "inn" => REST_INN,
        "settlement" => REST_SETTLEMENT,
        "campfire" => REST_CAMPFIRE,
        "lean_to" => REST_LEAN_TO,
        _ => REST_OUTSIDE,
    };
    pick_template(rng, templates).into()
}

pub fn dream_text(rng: &mut SeedRng) -> String {
    pick_template(rng, DREAM_TEMPLATES).into()
}

pub fn scar_text(rng: &mut SeedRng) -> String {
    pick_template(rng, SCAR_TEMPLATES).into()
}

pub fn rumor_text(rng: &mut SeedRng) -> String {
    pick_template(rng, RUMOR_TEMPLATES).into()
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
        let mut j = Journal::default();
        for i in 0..205 {
            j.log(i, Voice::Travel, format!("entry {i}"));
        }
        assert_eq!(j.entries.len(), 200);
        assert_eq!(j.entries[0].tick, 5);
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
        let all_templates: Vec<&[&str]> = vec![
            TRAVEL_DAWN,
            TRAVEL_MORNING,
            TRAVEL_NOON,
            TRAVEL_AFTERNOON,
            TRAVEL_DUSK,
            TRAVEL_NIGHT,
            TRAVEL_DEEP_NIGHT,
            TRAVEL_STORM,
            TRAVEL_SNOW,
            TRAVEL_FOG,
            TRAVEL_CLEAR_DAWN,
            ENCOUNTER_TEMPLATES,
            REST_CAMPFIRE,
            REST_OUTSIDE,
            REST_SETTLEMENT,
            REST_INN,
            REST_LEAN_TO,
            DREAM_TEMPLATES,
            SCAR_TEMPLATES,
            RUMOR_TEMPLATES,
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
        assert!(TRAVEL_DAWN.len() >= 3);
        assert!(TRAVEL_MORNING.len() >= 3);
        assert!(TRAVEL_NOON.len() >= 3);
        assert!(TRAVEL_AFTERNOON.len() >= 3);
        assert!(TRAVEL_DUSK.len() >= 3);
        assert!(TRAVEL_NIGHT.len() >= 3);
        assert!(TRAVEL_DEEP_NIGHT.len() >= 3);
        assert!(ENCOUNTER_TEMPLATES.len() >= 3);
        assert!(REST_CAMPFIRE.len() >= 3);
        assert!(REST_OUTSIDE.len() >= 3);
        assert!(REST_SETTLEMENT.len() >= 3);
        assert!(REST_INN.len() >= 3);
        assert!(REST_LEAN_TO.len() >= 3);
        assert!(DREAM_TEMPLATES.len() >= 3);
        assert!(SCAR_TEMPLATES.len() >= 3);
        assert!(RUMOR_TEMPLATES.len() >= 3);
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
