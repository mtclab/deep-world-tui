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
    "Grey light, wet grass, a road that asked nothing. I gave it my feet.",
    "The birds started before I did, but not by much.",
    "Dawn is the road's honest hour. Nothing has had time to lie yet.",
    "I left while the smoke of last night's fire still hung in the air.",
    "Cold hands, warm blood, first light. Walking.",
];

static TRAVEL_MORNING: &[&str] = &[
    "The morning was bright and the road stretched ahead.",
    "I walked through the early hours. The light was kind.",
    "The day opened before me. I walked into it.",
    "The dew burned off and the miles came easy.",
    "A good morning for walking. There are not so many; I notice them.",
    "The road was busy with small lives this morning — beetles, larks, one fox.",
    "I ate as I walked. The morning did not mind.",
    "Mist in the hollows, sun on the rises. I walked through both.",
];

static TRAVEL_NOON: &[&str] = &[
    "The sun beat down. I walked the midpoint of the day.",
    "No rest at noon. The road does not pause.",
    "High sun, long shadow beneath me. I kept walking.",
    "I measured the day by my shadow and found it short.",
    "Noon is when the road is longest. I walked it anyway.",
    "I rested in the shade of a stone for a hundred breaths, then went on.",
    "The heat sat on my shoulders like a pack. I carried both.",
];

static TRAVEL_AFTERNOON: &[&str] = &[
    "The afternoon wore on. I walked the long hours.",
    "Shadows lengthened ahead of me. I followed them.",
    "The day turned. I did not turn with it.",
    "The light went gold and the dust with it.",
    "Long hours. The kind that pass through me rather than by me.",
    "I counted milestones until I stopped counting. Then I just walked.",
    "The afternoon asked nothing and gave little. Fair trade.",
];

static TRAVEL_DUSK: &[&str] = &[
    "The light was failing when I finally stopped walking.",
    "Dusk caught me between places. I pressed on.",
    "The road grew dark. I did not stop.",
    "Shadows lengthened across my path. I followed them.",
    "Evening found me still on the road. I accept that.",
    "The last light caught the tops of the trees and let go.",
    "Day's end. My legs knew it before the sky did.",
    "I walked the seam between day and night and felt at home in it.",
];

static TRAVEL_NIGHT: &[&str] = &[
    "I walked through the dark. The stars were my only company.",
    "Night on the road is its own kind of quiet. I moved through it.",
    "The world was dark and I was a shadow crossing it.",
    "Only fools travel at night. I kept walking anyway.",
    "The dark road held no comfort. I walked it regardless.",
    "Something kept pace with me in the dark for a while. Then it didn't.",
    "Night walking: half listening, half praying, all forward.",
    "The moon did what it could. I did the rest.",
];

static TRAVEL_DEEP_NIGHT: &[&str] = &[
    "The world slept. I did not.",
    "Deepest night. I moved through it like a ghost.",
    "The dark was absolute. My steps were the only sound.",
    "Even the wind had gone to bed. I trespassed in the stillness.",
    "The hour when the world forgets itself. I walked through its forgetting.",
    "No light, no sound, no company but my own heartbeat. It was enough.",
];

static TRAVEL_STORM: &[&str] = &[
    "The storm found me on the road. I did not stop.",
    "Rain hammered the path. I walked through it.",
    "Wind tried to turn me back. The wind does not decide.",
    "The sky opened up and I kept walking. What else is there to do?",
    "The thunder counted my steps. I lost count of the thunder.",
    "Every tree wanted me under it. I trusted none of them.",
    "Soaked through by the first mile. After that, the rain was just weather.",
    "The storm and I went the same way for a while. Then it lost interest.",
];

static TRAVEL_SNOW: &[&str] = &[
    "Snow covered the road. I made my own path.",
    "The path was white and still. I broke the silence with my steps.",
    "Each step left a mark in the snow. I kept stepping.",
    "The snow asked silence and I gave it. We got along.",
    "White road, grey sky, black trees. The world in three inks.",
    "My breath went ahead of me in little clouds. I followed it.",
];

static TRAVEL_FOG: &[&str] = &[
    "Fog swallowed the road ahead. I walked into it.",
    "The mist was thick. I trusted my feet.",
    "I could barely see. I walked anyway.",
    "Shapes came and went in the mist. I greeted none of them.",
    "The fog made the familiar strange. I made my legs ignore it.",
    "Ten paces of road at a time. The world rationed itself today.",
];

static TRAVEL_CLEAR_DAWN: &[&str] = &[
    "A clear dawn. The road stretched out before me like a promise.",
    "The morning was clean and bright. I took it as a sign.",
    "Sky like washed glass. Days like this are owed to someone.",
    "The horizon was sharp enough to cut. I walked toward the wound.",
    "Not a cloud. The land lay open like a ledger.",
];

static ENCOUNTER_TEMPLATES: &[&str] = &[
    "Another face on the road. Words were exchanged. That is all.",
    "Someone crossed my path. We spoke. The world is full of such moments.",
    "I met a stranger. The meeting was brief. It stays with me.",
    "An encounter. Brief, but not without weight.",
    "The road provides company, sometimes. Today it did — for a moment.",
    "The road put someone in my way. We managed it, both of us.",
    "Faces on the road: each one a country I will never visit twice.",
    "We traded a few words and kept our distances. Civilization, of a kind.",
    "They went their way, I went mine. The road held us both a moment.",
    "Some meetings weigh nothing. This one had a little weight to it.",
    "I keep a count of kindnesses met on the road. It moved today — one way or the other.",
];

static REST_CAMPFIRE: &[&str] = &[
    "The fire is small. I am tired. I sleep.",
    "I built a fire and watched it until my eyes closed.",
    "The flames are low. The night is long. I rest.",
    "A small fire. A small comfort. I sleep.",
    "The fire crackles. I let myself close my eyes.",
    "Fed the fire twigs until it trusted me. Slept in its good opinion.",
    "The fire talked all night, soft. I fell asleep mid-sentence.",
    "One ember watched me till morning. I let it.",
];

static REST_OUTSIDE: &[&str] = &[
    "No fire tonight. I sleep with one eye open.",
    "The ground is hard. The sky is wide. I lie down anyway.",
    "I sleep in the open. The cold is an old companion.",
    "No shelter. I rest against the earth and hope for dawn.",
    "The stars are my roof. It is not enough. I sleep anyway.",
    "I slept like the stones around me: badly, and watching.",
    "Every sound was a wolf until it wasn't. Then I slept.",
    "The earth is an honest bed. It promises nothing and keeps its word.",
    "Curled up small, the way the old road-wisdom says. Still cold.",
];

static REST_SETTLEMENT: &[&str] = &[
    "A roof overhead. The floor is hard but it is warm.",
    "I sleep under a proper roof. The walls hold back the world.",
    "The settlement takes me in. I rest on solid ground.",
    "Shelter. The simple kind. I am grateful.",
    "Four walls and a bed. It is enough.",
    "Other people's sounds through the wall: someone's cough, someone's child. I slept among lives.",
    "The floor smelled of woodsmoke and dogs. Better than most nights.",
    "They let me sleep by the wall. The wall and I are grateful.",
];

static REST_INN: &[&str] = &[
    "A proper bed. Soft linen. I sleep the sleep of the fortunate.",
    "The inn is warm. The mattress yields. I let myself rest deeply.",
    "For tonight, I am not on the road. I sleep without fear.",
    "Heat from the hearth. A blanket. I close my eyes and do not dream.",
    "The innkeeper asked no questions. I gave no answers. I slept well.",
    "I paid for the bed and the lock on the door. The lock slept better than either of us.",
    "Clean straw, near-clean blanket. Luxury keeps low standards on the road, thankfully.",
    "Somebody sang in the common room until late. I fell asleep inside the song.",
];

static REST_LEAN_TO: &[&str] = &[
    "A lean-to shields me from the worst of it. I sleep lightly.",
    "The wind is quieter here. I close my eyes against the rough wood.",
    "Rough shelter, but shelter. I rest.",
    "The lean-to creaks. I sleep anyway.",
    "My own work between me and the sky. It held.",
    "The lean-to took the rain's first argument and most of its second.",
    "Knees to chest under slanted wood. Sleep came sideways but it came.",
];

static DREAM_TEMPLATES: &[&str] = &[
    "I dreamed of a vast archive. The shelves went on forever.",
    "In my sleep, a voice spoke from the deep. I did not understand it.",
    "I dreamed of walking a road that had no end. I was not tired.",
    "The dream was of water — cold, clear, rising. I did not drown.",
    "A figure stood at the crossroads in my dream. It pointed. I did not follow.",
    "I dreamed the mountain was hollow and full of slow bells.",
    "In the dream I owed someone a name. I woke before I paid.",
    "I dreamed of the sea swallowing a library, shelf by shelf, patient as tide.",
    "A dream of hands building a wall, my hands, a wall I have never seen. It needed one more stone.",
    "I dreamed I was the road. Everyone I have ever met walked over me, gently.",
];

static SCAR_TEMPLATES: &[&str] = &[
    "I will not forget the teeth.",
    "The wound is closed. The memory is open.",
    "Pain teaches. This lesson is carved in skin.",
    "Another mark. Another story I do not tell.",
    "The body heals. The body remembers.",
    "Some debts are written on the skin so the heart can stop holding them.",
    "It aches before storms now. I own a small oracle.",
    "I tell people it was a beast. Some days that is even true.",
    "The mark has faded to the color of memory. Both still pull.",
];

static RUMOR_TEMPLATES: &[&str] = &[
    "I heard something. Whether it is true, I cannot say.",
    "A whisper reached me. I carry it now.",
    "They spoke of something. I listened. That is all.",
    "Rumor travels faster than people. It found me.",
    "Words from a stranger's mouth. I file them away.",
    "Talk is the one crop that grows in every season.",
    "I bought a drink and was paid in hearsay. Fair rate.",
    "Half of what I heard tonight is false. The trick is which half.",
    "News wears out its boots fast on these roads. This piece still had sole left.",
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
