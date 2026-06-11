use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeOfDay {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    Dusk,
    Night,
    DeepNight,
}

impl TimeOfDay {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            5..=6 => TimeOfDay::Dawn,
            7..=10 => TimeOfDay::Morning,
            11..=13 => TimeOfDay::Noon,
            14..=17 => TimeOfDay::Afternoon,
            18..=20 => TimeOfDay::Dusk,
            21..=23 => TimeOfDay::Night,
            _ => TimeOfDay::DeepNight,
        }
    }

    pub fn glyph(self) -> char {
        match self {
            TimeOfDay::Dawn => '☼',
            TimeOfDay::Morning => '☼',
            TimeOfDay::Noon => '☀',
            TimeOfDay::Afternoon => '☀',
            TimeOfDay::Dusk => '◐',
            TimeOfDay::Night => '◑',
            TimeOfDay::DeepNight => '●',
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, TimeOfDay::Night | TimeOfDay::DeepNight)
    }

    pub fn blocks_services(self) -> bool {
        matches!(self, TimeOfDay::Night | TimeOfDay::DeepNight)
    }

    pub fn one_word(self) -> &'static str {
        match self {
            TimeOfDay::Dawn => "Dawn.",
            TimeOfDay::Morning => "Morning.",
            TimeOfDay::Noon => "Noon.",
            TimeOfDay::Afternoon => "Afternoon.",
            TimeOfDay::Dusk => "Dusk.",
            TimeOfDay::Night => "Night.",
            TimeOfDay::DeepNight => "Deep night.",
        }
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.one_word().trim_end_matches('.'))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Thaw,
    Green,
    Frost,
}

impl Season {
    pub fn from_day(day: u32) -> Self {
        if day == 0 {
            return Season::Thaw;
        }
        let day_in_year = (day - 1) % Self::YEAR_DAYS;
        match day_in_year {
            0..=29 => Season::Thaw,
            30..=59 => Season::Green,
            _ => Season::Frost,
        }
    }

    pub fn gather_multiplier(self) -> f64 {
        match self {
            Season::Thaw => 1.0,
            Season::Green => 1.2,
            Season::Frost => 0.3,
        }
    }

    pub fn need_decay_multiplier(self) -> f64 {
        match self {
            Season::Frost => 1.3,
            _ => 1.0,
        }
    }

    pub fn bias_modifier(self) -> f64 {
        match self {
            Season::Green => 0.05,
            Season::Frost => -0.05,
            _ => 0.0,
        }
    }

    pub fn glyph(self) -> char {
        match self {
            Season::Thaw => '❀',
            Season::Green => '✿',
            Season::Frost => '❄',
        }
    }

    pub fn festival_chance(self) -> u32 {
        match self {
            Season::Green => 30,
            Season::Thaw => 10,
            Season::Frost => 0,
        }
    }

    pub const YEAR_DAYS: u32 = 90;

    /// The canonical present: one year after the Quiet Chronicles close
    /// (trilogy runs to 182 AF; ethnographic surveys 155 AF; Archive index
    /// compiled 175 AF). The game year is a compressed abstraction of the
    /// canon 365-day year — seasons pass faster than the books count them.
    pub const PRESENT_YEAR_AF: u32 = 183;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
    Heatwave,
    Whiteout,
    Thunderhead,
    DryLightning,
    SeaSquall,
}

impl Weather {
    pub fn generate(seed: u64, tick: u64, terrain: Terrain) -> Self {
        let mut rng = crate::rng::SeedRng::new(seed.wrapping_add(tick));
        let roll = rng.gen_range(1000);

        // Regional bias
        let (
            clear_w,
            cloudy_w,
            rain_w,
            storm_w,
            snow_w,
            fog_w,
            heat_w,
            whiteout_w,
            thunder_w,
            dryltn_w,
            sqall_w,
        ) = match terrain {
            Terrain::Coast => (120, 200, 180, 80, 30, 150, 40, 10, 40, 20, 130),
            Terrain::Mountain => (160, 180, 120, 120, 120, 80, 30, 60, 60, 30, 10),
            Terrain::Forest => (120, 240, 220, 80, 40, 80, 40, 10, 40, 20, 10),
            Terrain::Swamp => (80, 180, 220, 80, 30, 200, 30, 5, 30, 15, 10),
            Terrain::DeepDesert | Terrain::Sand => (250, 150, 30, 30, 0, 30, 300, 0, 10, 80, 20),
            Terrain::Tundra => (120, 160, 80, 80, 280, 40, 30, 120, 40, 10, 10),
            _ => (160, 200, 160, 80, 80, 80, 40, 20, 30, 20, 20),
        };

        let total = clear_w
            + cloudy_w
            + rain_w
            + storm_w
            + snow_w
            + fog_w
            + heat_w
            + whiteout_w
            + thunder_w
            + dryltn_w
            + sqall_w;
        let roll = roll % total;

        if roll < clear_w {
            Weather::Clear
        } else if roll < clear_w + cloudy_w {
            Weather::Cloudy
        } else if roll < clear_w + cloudy_w + rain_w {
            Weather::Rain
        } else if roll < clear_w + cloudy_w + rain_w + storm_w {
            Weather::Storm
        } else if roll < clear_w + cloudy_w + rain_w + storm_w + snow_w {
            Weather::Snow
        } else if roll < clear_w + cloudy_w + rain_w + storm_w + snow_w + fog_w {
            Weather::Fog
        } else if roll < clear_w + cloudy_w + rain_w + storm_w + snow_w + fog_w + heat_w {
            Weather::Heatwave
        } else if roll
            < clear_w + cloudy_w + rain_w + storm_w + snow_w + fog_w + heat_w + whiteout_w
        {
            Weather::Whiteout
        } else if roll
            < clear_w
                + cloudy_w
                + rain_w
                + storm_w
                + snow_w
                + fog_w
                + heat_w
                + whiteout_w
                + thunder_w
        {
            Weather::Thunderhead
        } else if roll
            < clear_w
                + cloudy_w
                + rain_w
                + storm_w
                + snow_w
                + fog_w
                + heat_w
                + whiteout_w
                + thunder_w
                + dryltn_w
        {
            Weather::DryLightning
        } else {
            Weather::SeaSquall
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Weather::Clear => "clear",
            Weather::Cloudy => "cloudy",
            Weather::Rain => "rain",
            Weather::Storm => "storm",
            Weather::Snow => "snow",
            Weather::Fog => "fog",
            Weather::Heatwave => "heatwave",
            Weather::Whiteout => "whiteout",
            Weather::Thunderhead => "thunderhead",
            Weather::DryLightning => "dry_lightning",
            Weather::SeaSquall => "sea_squall",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            Weather::Clear => '☀',
            Weather::Cloudy => '☁',
            Weather::Rain => '🌧',
            Weather::Storm => '⛈',
            Weather::Snow => '❄',
            Weather::Fog => '🌫',
            Weather::Heatwave => '🔥',
            Weather::Whiteout => '⚪',
            Weather::Thunderhead => '🌩',
            Weather::DryLightning => '⚡',
            Weather::SeaSquall => '🌬',
        }
    }

    pub fn gather_modifier(self) -> f64 {
        match self {
            Weather::Clear => 1.0,
            Weather::Cloudy => 0.95,
            Weather::Rain => 0.8,
            Weather::Storm => 0.5,
            Weather::Snow => 0.6,
            Weather::Fog => 0.85,
            Weather::Heatwave => 0.7,
            Weather::Whiteout => 0.2,
            Weather::Thunderhead => 0.55,
            Weather::DryLightning => 0.65,
            Weather::SeaSquall => 0.4,
        }
    }

    pub fn need_decay_modifier(self) -> f64 {
        match self {
            Weather::Clear => 1.0,
            Weather::Cloudy => 1.0,
            Weather::Rain => 1.05,
            Weather::Storm => 1.15,
            Weather::Snow => 1.2,
            Weather::Fog => 1.05,
            Weather::Heatwave => 1.25,
            Weather::Whiteout => 1.3,
            Weather::Thunderhead => 1.15,
            Weather::DryLightning => 1.1,
            Weather::SeaSquall => 1.2,
        }
    }

    pub fn npc_mood_modifier(self) -> f64 {
        match self {
            Weather::Clear => 0.02,
            Weather::Cloudy => 0.0,
            Weather::Rain => -0.02,
            Weather::Storm => -0.05,
            Weather::Snow => -0.03,
            Weather::Fog => -0.01,
            Weather::Heatwave => -0.04,
            Weather::Whiteout => -0.06,
            Weather::Thunderhead => -0.05,
            Weather::DryLightning => -0.03,
            Weather::SeaSquall => -0.05,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessLevel {
    Seen,
    Unseen,
    Rumored,
}

impl WitnessLevel {
    pub fn roll(seed: u64, terrain: Terrain) -> Self {
        let hash = seed.wrapping_mul(2654435761) ^ (terrain as u64).wrapping_mul(40503);
        let val = hash % 100;
        match terrain {
            Terrain::Settlement => WitnessLevel::Seen,
            Terrain::Road => {
                if val < 70 {
                    WitnessLevel::Seen
                } else if val < 90 {
                    WitnessLevel::Rumored
                } else {
                    WitnessLevel::Unseen
                }
            }
            _ => {
                if val < 20 {
                    WitnessLevel::Seen
                } else if val < 50 {
                    WitnessLevel::Rumored
                } else {
                    WitnessLevel::Unseen
                }
            }
        }
    }

    pub fn reputation_multiplier(self) -> f64 {
        match self {
            WitnessLevel::Seen => 1.0,
            WitnessLevel::Rumored => 0.3,
            WitnessLevel::Unseen => 0.0,
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            WitnessLevel::Seen => "Word of your deed spreads quickly.",
            WitnessLevel::Rumored => "Whispers follow your footsteps.",
            WitnessLevel::Unseen => "No one saw. The silence holds.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensionEvent {
    BorderDispute,
    TradeSanction,
    LandClaim,
    BloodFeud,
    AllianceSigned,
}

impl TensionEvent {
    pub fn label(self) -> &'static str {
        match self {
            TensionEvent::BorderDispute => "border dispute",
            TensionEvent::TradeSanction => "trade sanction",
            TensionEvent::LandClaim => "land claim",
            TensionEvent::BloodFeud => "blood-feud",
            TensionEvent::AllianceSigned => "alliance",
        }
    }

    pub fn flavor(self, a: PeopleKind, b: PeopleKind) -> String {
        match self {
            TensionEvent::BorderDispute => format!(
                "Word spreads of a {} between {} and {} settlements. Tempers run short.",
                self.label(),
                a.label(),
                b.label()
            ),
            TensionEvent::TradeSanction => format!(
                "Traders whisper: {} merchants refuse {} goods. Prices shift.",
                a.label(),
                b.label()
            ),
            TensionEvent::LandClaim => format!(
                "A {} elder claims {} ground as ancestral. Guards double at the gate.",
                a.label(),
                b.label()
            ),
            TensionEvent::BloodFeud => format!(
                "Bad blood boils between {} and {}. Old grievances, fresh wounds.",
                a.label(),
                b.label()
            ),
            TensionEvent::AllianceSigned => format!(
                "An alliance is signed between {} and {} councils. Handshakes and hope.",
                a.label(),
                b.label()
            ),
        }
    }

    pub fn bias_shift(self) -> f64 {
        match self {
            TensionEvent::BorderDispute => -0.01,
            TensionEvent::TradeSanction => -0.005,
            TensionEvent::LandClaim => -0.01,
            TensionEvent::BloodFeud => -0.02,
            TensionEvent::AllianceSigned => 0.01,
        }
    }

    pub fn roll(seed: u64, day: u32) -> Option<(Self, PeopleKind, PeopleKind)> {
        let hash = seed.wrapping_mul(2654435769) ^ (day as u64).wrapping_mul(7919);
        let val = hash % 1000;
        if val > 8 {
            return None;
        }
        let all = [
            PeopleKind::Metsik,
            PeopleKind::Ahjo,
            PeopleKind::Sepat,
            PeopleKind::Vayla,
            PeopleKind::Arkit,
            PeopleKind::Laakso,
            PeopleKind::Varhaiset,
            PeopleKind::Metsareunat,
            PeopleKind::Porokansa,
            PeopleKind::Koskimetsa,
            PeopleKind::Muistikansa,
            PeopleKind::Taulukansa,
            PeopleKind::Kirjakansa,
            PeopleKind::Takovaki,
            PeopleKind::Rantavaki,
            PeopleKind::Saarivaki,
            PeopleKind::Hiekkakavelijat,
            PeopleKind::Haramaki,
            PeopleKind::Jamavaki,
            PeopleKind::Pohjavaki,
            PeopleKind::Tzakhar,
            PeopleKind::Merak,
            PeopleKind::Shear,
            PeopleKind::Hal,
            PeopleKind::Khor,
        ];
        let a = all[(hash / 7) as usize % all.len()];
        let b = all[(hash / 13) as usize % all.len()];
        if a == b {
            return None;
        }
        let event = match val % 5 {
            0 => TensionEvent::BorderDispute,
            1 => TensionEvent::TradeSanction,
            2 => TensionEvent::LandClaim,
            3 => TensionEvent::BloodFeud,
            _ => TensionEvent::AllianceSigned,
        };
        Some((event, a, b))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FestivalKind {
    HarvestFeast,
    ForgeDay,
    ForestRite,
    RiverGathering,
    MidsummerBonfire,
    AncestorVigil,
    /// Sampsa's festival: star-charts read aloud, the rolls of names recited.
    StarReckoning,
}

impl FestivalKind {
    pub fn label(self) -> &'static str {
        match self {
            FestivalKind::HarvestFeast => "Harvest Feast",
            FestivalKind::ForgeDay => "Forge-Day",
            FestivalKind::ForestRite => "Forest Rite",
            FestivalKind::RiverGathering => "River Gathering",
            FestivalKind::MidsummerBonfire => "Midsummer Bonfire",
            FestivalKind::AncestorVigil => "Ancestor Vigil",
            FestivalKind::StarReckoning => "Star-Reckoning",
        }
    }

    pub fn for_people(people: PeopleKind) -> Self {
        match people {
            PeopleKind::Ahjo => FestivalKind::HarvestFeast,
            PeopleKind::Sepat => FestivalKind::ForgeDay,
            PeopleKind::Metsik => FestivalKind::ForestRite,
            PeopleKind::Vayla => FestivalKind::RiverGathering,
            PeopleKind::Arkit => FestivalKind::StarReckoning,
            PeopleKind::Laakso => FestivalKind::AncestorVigil,
            PeopleKind::Varhaiset => FestivalKind::MidsummerBonfire,
            PeopleKind::Metsareunat => FestivalKind::ForestRite,
            PeopleKind::Porokansa => FestivalKind::ForestRite,
            PeopleKind::Koskimetsa => FestivalKind::ForestRite,
            PeopleKind::Muistikansa => FestivalKind::StarReckoning,
            PeopleKind::Taulukansa => FestivalKind::StarReckoning,
            PeopleKind::Kirjakansa => FestivalKind::StarReckoning,
            PeopleKind::Takovaki => FestivalKind::ForgeDay,
            PeopleKind::Rantavaki => FestivalKind::RiverGathering,
            PeopleKind::Saarivaki => FestivalKind::RiverGathering,
            PeopleKind::Hiekkakavelijat => FestivalKind::RiverGathering,
            PeopleKind::Haramaki => FestivalKind::AncestorVigil,
            PeopleKind::Jamavaki => FestivalKind::AncestorVigil,
            PeopleKind::Pohjavaki => FestivalKind::AncestorVigil,
            PeopleKind::Tzakhar => FestivalKind::AncestorVigil,
            PeopleKind::Merak => FestivalKind::RiverGathering,
            PeopleKind::Shear => FestivalKind::AncestorVigil,
            PeopleKind::Hal => FestivalKind::ForestRite,
            PeopleKind::Khor => FestivalKind::AncestorVigil,
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            FestivalKind::HarvestFeast => "Long tables line the square. Kettles steam. Someone presses a bowl into your hands.",
            FestivalKind::ForgeDay => "The ring of hammers fills the air. Sparks cascade like fallen stars. Iron songs echo off the walls.",
            FestivalKind::ForestRite => "Drumbeats pulse from the treeline. Firelight flickers between the boles. The forest speaks tonight.",
            FestivalKind::RiverGathering => "Boats crowd the dock. Lanterns float on the water. Voices carry across the current.",
            FestivalKind::MidsummerBonfire => "The bonfire roars taller than the rooftops. Shadows dance wild. The night is brief and ancient.",
            FestivalKind::AncestorVigil => "Candles burn in every window. Voices murmur old names. The past walks among the living tonight.",
            FestivalKind::StarReckoning => "Lanterns ring the square. Star-charts are read aloud, and the rolls of names — every birth, every death, every arrival — recited to the sky.",
        }
    }

    pub fn patron_god(self) -> GodName {
        match self {
            FestivalKind::HarvestFeast => GodName::Oltzed,
            FestivalKind::ForgeDay => GodName::Oltzed,
            FestivalKind::ForestRite => GodName::Keuru,
            FestivalKind::RiverGathering => GodName::Masa,
            FestivalKind::MidsummerBonfire => GodName::Keuru,
            FestivalKind::AncestorVigil => GodName::Kukri,
            FestivalKind::StarReckoning => GodName::Sampsa,
        }
    }
}

impl GameClock {
    /// The current year in After-Fall reckoning (game years are compressed).
    pub fn year_af(&self) -> u32 {
        Season::PRESENT_YEAR_AF + self.day / Season::YEAR_DAYS
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Season::Thaw => write!(f, "Thaw"),
            Season::Green => write!(f, "Green"),
            Season::Frost => write!(f, "Frost"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GameClock {
    pub day: u32,
    pub hour: u32,
}

impl Default for GameClock {
    fn default() -> Self {
        GameClock { day: 1, hour: 8 }
    }
}

impl GameClock {
    pub fn new(day: u32, hour: u32) -> Self {
        GameClock {
            day,
            hour: hour % 24,
        }
    }

    pub fn time_of_day(self) -> TimeOfDay {
        TimeOfDay::from_hour(self.hour)
    }

    pub fn season(self) -> Season {
        Season::from_day(self.day)
    }

    pub fn advance(&mut self, hours: u32) {
        self.hour += hours;
        self.day += self.hour / 24;
        self.hour %= 24;
    }

    pub fn advance_hour(&mut self) {
        self.advance(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
    >(
        value: &T,
    ) {
        let ser = ron::ser::to_string(value).unwrap();
        let de: T = ron::from_str(&ser).unwrap();
        assert_eq!(*value, de);
    }

    #[test]

    fn time_of_day_from_hour() {
        assert_eq!(TimeOfDay::from_hour(0), TimeOfDay::DeepNight);

        assert_eq!(TimeOfDay::from_hour(4), TimeOfDay::DeepNight);

        assert_eq!(TimeOfDay::from_hour(5), TimeOfDay::Dawn);

        assert_eq!(TimeOfDay::from_hour(6), TimeOfDay::Dawn);

        assert_eq!(TimeOfDay::from_hour(7), TimeOfDay::Morning);

        assert_eq!(TimeOfDay::from_hour(10), TimeOfDay::Morning);

        assert_eq!(TimeOfDay::from_hour(11), TimeOfDay::Noon);

        assert_eq!(TimeOfDay::from_hour(13), TimeOfDay::Noon);

        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);

        assert_eq!(TimeOfDay::from_hour(17), TimeOfDay::Afternoon);

        assert_eq!(TimeOfDay::from_hour(18), TimeOfDay::Dusk);

        assert_eq!(TimeOfDay::from_hour(20), TimeOfDay::Dusk);

        assert_eq!(TimeOfDay::from_hour(21), TimeOfDay::Night);

        assert_eq!(TimeOfDay::from_hour(23), TimeOfDay::Night);
    }

    #[test]

    fn time_of_day_is_dark() {
        assert!(TimeOfDay::Night.is_dark());

        assert!(TimeOfDay::DeepNight.is_dark());

        assert!(!TimeOfDay::Dawn.is_dark());

        assert!(!TimeOfDay::Morning.is_dark());

        assert!(!TimeOfDay::Noon.is_dark());

        assert!(!TimeOfDay::Afternoon.is_dark());

        assert!(!TimeOfDay::Dusk.is_dark());
    }

    #[test]

    fn time_of_day_blocks_services() {
        assert!(TimeOfDay::Night.blocks_services());

        assert!(TimeOfDay::DeepNight.blocks_services());

        assert!(!TimeOfDay::Dawn.blocks_services());

        assert!(!TimeOfDay::Morning.blocks_services());

        assert!(!TimeOfDay::Noon.blocks_services());

        assert!(!TimeOfDay::Afternoon.blocks_services());

        assert!(!TimeOfDay::Dusk.blocks_services());
    }

    #[test]

    fn time_of_day_one_word_ends_with_period_or_phrase() {
        for tod in [
            TimeOfDay::Dawn,
            TimeOfDay::Morning,
            TimeOfDay::Noon,
            TimeOfDay::Afternoon,
            TimeOfDay::Dusk,
            TimeOfDay::Night,
            TimeOfDay::DeepNight,
        ] {
            let w = tod.one_word();

            assert!(!w.is_empty(), "one_word() must not be empty for {tod:?}");

            assert!(w.ends_with('.'), "one_word() must end with period: {w}");
        }
    }

    #[test]

    fn time_of_day_display_strips_period() {
        assert_eq!(format!("{}", TimeOfDay::Dawn), "Dawn");

        assert_eq!(format!("{}", TimeOfDay::DeepNight), "Deep night");

        assert_eq!(format!("{}", TimeOfDay::Night), "Night");
    }

    #[test]

    fn time_of_day_seven_phases() {
        let mut seen = std::collections::HashSet::new();

        for h in 0..24 {
            seen.insert(TimeOfDay::from_hour(h));
        }

        assert_eq!(seen.len(), 7, "expected 7 distinct phases across 24h");
    }

    #[test]

    fn game_clock_advance() {
        let mut clock = GameClock::new(1, 22);

        assert_eq!(clock.time_of_day(), TimeOfDay::Night);

        clock.advance_hour();

        assert_eq!(clock.day, 1);

        assert_eq!(clock.hour, 23);

        clock.advance_hour();

        assert_eq!(clock.day, 2);

        assert_eq!(clock.hour, 0);
    }

    #[test]

    fn game_clock_advance_multi() {
        let mut clock = GameClock::new(1, 20);

        clock.advance(6);

        assert_eq!(clock.day, 2);

        assert_eq!(clock.hour, 2);
    }

    #[test]

    fn game_clock_serialization() {
        let clock = GameClock::new(3, 15);

        roundtrip(&clock);
    }

    #[test]

    fn night_is_dark() {
        assert!(TimeOfDay::Night.is_dark());

        assert!(TimeOfDay::DeepNight.is_dark());

        assert!(!TimeOfDay::Morning.is_dark());

        assert!(!TimeOfDay::Noon.is_dark());

        assert!(!TimeOfDay::Dawn.is_dark());

        assert!(!TimeOfDay::Dusk.is_dark());
    }

    #[test]

    fn season_cycle() {
        assert_eq!(Season::from_day(0), Season::Thaw);

        assert_eq!(Season::from_day(1), Season::Thaw);

        assert_eq!(Season::from_day(30), Season::Thaw);

        assert_eq!(Season::from_day(31), Season::Green);

        assert_eq!(Season::from_day(60), Season::Green);

        assert_eq!(Season::from_day(61), Season::Frost);

        assert_eq!(Season::from_day(90), Season::Frost);

        assert_eq!(Season::from_day(91), Season::Thaw);

        assert_eq!(Season::from_day(180), Season::Frost);

        assert_eq!(Season::from_day(181), Season::Thaw);
    }

    #[test]

    fn season_gather_multiplier() {
        assert!((Season::Green.gather_multiplier() - 1.2).abs() < f64::EPSILON);

        assert!((Season::Frost.gather_multiplier() - 0.3).abs() < f64::EPSILON);

        assert!((Season::Thaw.gather_multiplier() - 1.0).abs() < f64::EPSILON);
    }

    #[test]

    fn frost_faster_hunger_decay() {
        assert!((Season::Frost.need_decay_multiplier() - 1.3).abs() < f64::EPSILON);

        assert!((Season::Thaw.need_decay_multiplier() - 1.0).abs() < f64::EPSILON);

        assert!((Season::Green.need_decay_multiplier() - 1.0).abs() < f64::EPSILON);
    }

    #[test]

    fn clock_season() {
        let clock = GameClock::new(31, 12);

        assert_eq!(clock.season(), Season::Green);
    }

    #[test]

    fn weather_deterministic() {
        let w1 = Weather::generate(42, 100, Terrain::Forest);

        let w2 = Weather::generate(42, 100, Terrain::Forest);

        assert_eq!(w1, w2);
    }

    #[test]

    fn weather_varies_by_tick() {
        let w1 = Weather::generate(42, 100, Terrain::Forest);

        let _w2 = Weather::generate(42, 200, Terrain::Forest);

        // Different ticks should usually produce different weather

        // (not guaranteed, but highly likely)

        let mut different = false;

        for tick in 0..20 {
            if Weather::generate(42, tick, Terrain::Forest) != w1 {
                different = true;

                break;
            }
        }

        assert!(different, "weather should vary by tick");
    }

    #[test]

    fn weather_names_and_glyphs() {
        assert_eq!(Weather::Clear.name(), "clear");

        assert_eq!(Weather::Clear.glyph(), '☀');

        assert_eq!(Weather::Rain.name(), "rain");

        assert_eq!(Weather::Rain.glyph(), '🌧');
    }

    #[test]

    fn weather_modifiers_in_range() {
        let weathers = [
            Weather::Clear,
            Weather::Cloudy,
            Weather::Rain,
            Weather::Storm,
            Weather::Snow,
            Weather::Fog,
            Weather::Heatwave,
            Weather::Whiteout,
            Weather::Thunderhead,
            Weather::DryLightning,
            Weather::SeaSquall,
        ];

        for w in weathers {
            assert!(w.gather_modifier() > 0.0 && w.gather_modifier() <= 1.0);

            assert!(w.need_decay_modifier() >= 1.0 && w.need_decay_modifier() <= 1.5);

            assert!(w.npc_mood_modifier() >= -0.1 && w.npc_mood_modifier() <= 0.1);
        }
    }

    #[test]

    fn weather_regional_bias() {
        // Coast should have more fog

        let mut fog_count = 0;

        for tick in 0..100 {
            if Weather::generate(42, tick, Terrain::Coast) == Weather::Fog {
                fog_count += 1;
            }
        }

        assert!(
            fog_count > 10,
            "coast should have significant fog: {}",
            fog_count
        );

        // Desert should have more heatwave

        let mut heat_count = 0;

        for tick in 0..100 {
            if Weather::generate(42, tick, Terrain::DeepDesert) == Weather::Heatwave {
                heat_count += 1;
            }
        }

        assert!(
            heat_count > 20,
            "desert should have significant heatwave: {}",
            heat_count
        );
    }

    #[test]

    fn weather_generate_produces_all_variants() {
        use std::collections::HashSet;

        let terrains = [
            Terrain::Grass,
            Terrain::Coast,
            Terrain::Mountain,
            Terrain::Forest,
            Terrain::Swamp,
            Terrain::DeepDesert,
            Terrain::Sand,
            Terrain::Tundra,
            Terrain::Road,
        ];

        let mut seen: HashSet<Weather> = HashSet::new();

        for seed in 0..50u64 {
            for tick in 0..200u64 {
                for &terrain in &terrains {
                    let w = Weather::generate(seed, tick, terrain);

                    seen.insert(w);

                    if seen.len() == 11 {
                        return;
                    }
                }
            }
        }

        let missing: Vec<_> = [
            Weather::Clear,
            Weather::Cloudy,
            Weather::Rain,
            Weather::Storm,
            Weather::Snow,
            Weather::Fog,
            Weather::Heatwave,
            Weather::Whiteout,
            Weather::Thunderhead,
            Weather::DryLightning,
            Weather::SeaSquall,
        ]
        .into_iter()
        .filter(|w| !seen.contains(w))
        .collect();

        panic!("Weather::generate never produced: {:?}", missing);
    }

    #[test]

    fn farm_weather_effect() {
        let mut farm_clear = Farm::new(42, CropType::Grain, 100, Terrain::Farmland);

        let mut farm_storm = Farm::new(42, CropType::Grain, 100, Terrain::Farmland);

        farm_clear.update_growth(150, Weather::Clear);

        farm_storm.update_growth(150, Weather::Storm);

        assert!(farm_clear.growth_progress > farm_storm.growth_progress);
    }
}
