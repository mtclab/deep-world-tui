use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Grass,
    Forest,
    Water,
    Mountain,
    Road,
    Settlement,
    Farmland,
    Sand,
    Swamp,
    Coast,
    Cave,
    Tundra,
    DeepDesert,
}

impl Terrain {
    pub fn glyph(self) -> char {
        match self {
            Terrain::Grass => '░',
            Terrain::Forest => '▓',
            Terrain::Water => '≈',
            Terrain::Mountain => '▲',
            Terrain::Road => '·',
            Terrain::Settlement => '█',
            Terrain::Farmland => '▒',
            Terrain::Sand => '·',
            Terrain::Swamp => '~',
            Terrain::Coast => '≋',
            Terrain::Cave => '◉',
            Terrain::Tundra => '▒',
            Terrain::DeepDesert => '░',
        }
    }

    pub fn passable(self) -> bool {
        !matches!(self, Terrain::Water | Terrain::Mountain)
    }

    pub fn travel_hours(self) -> u32 {
        match self {
            Terrain::Road | Terrain::Settlement => 1,
            Terrain::Grass | Terrain::Farmland | Terrain::Sand | Terrain::Coast => 2,
            Terrain::Forest | Terrain::Swamp | Terrain::Cave | Terrain::Tundra => 3,
            Terrain::DeepDesert => 4,
            Terrain::Water | Terrain::Mountain => 2,
        }
    }

    pub fn people_gather_bonus(people: PeopleKind, terrain: Terrain) -> u32 {
        match (people, terrain) {
            (PeopleKind::Metsik, Terrain::Forest) => 1,
            (PeopleKind::Sepat, Terrain::Mountain) => 1,
            (PeopleKind::Ahjo, Terrain::Grass | Terrain::Farmland) => 1,
            (PeopleKind::Hal, Terrain::Forest) => 1,
            (PeopleKind::Tzakhar, Terrain::Cave) => 1,
            (PeopleKind::Merak, Terrain::Coast) => 1,
            (PeopleKind::Khor, Terrain::Tundra) => 1,
            // Stayed peoples terrain bonuses
            (PeopleKind::Metsareunat, Terrain::Forest) => 1,
            (PeopleKind::Koskimetsa, Terrain::Forest) => 1,
            (PeopleKind::Porokansa, Terrain::Tundra) => 1,
            (PeopleKind::Rantavaki, Terrain::Coast) => 1,
            (PeopleKind::Saarivaki, Terrain::Coast) => 1,
            (PeopleKind::Hiekkakavelijat, Terrain::Coast) => 1,
            (PeopleKind::Haramaki, Terrain::Mountain) => 1,
            (PeopleKind::Pohjavaki, Terrain::Cave) => 1,
            _ => 0,
        }
    }

    pub fn patron_god(self) -> Option<GodName> {
        match self {
            Terrain::Forest => Some(GodName::Keuru),
            Terrain::Grass | Terrain::Farmland | Terrain::Settlement => Some(GodName::Oltzed),
            Terrain::Mountain => Some(GodName::Oltzed),
            Terrain::Road | Terrain::Water => Some(GodName::Masa),
            Terrain::Swamp => Some(GodName::Kukri),
            Terrain::Coast => Some(GodName::Masa),
            Terrain::Cave => Some(GodName::Kukri),
            Terrain::Tundra => Some(GodName::Kukri),
            Terrain::Sand | Terrain::DeepDesert => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PlayerPos {
    pub region_idx: usize,
    pub px: usize,
    pub py: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TerrainMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Terrain>,
}

impl TerrainMap {
    pub fn get(&self, x: usize, y: usize) -> Option<Terrain> {
        if x < self.width && y < self.height {
            self.tiles.get(y * self.width + x).copied()
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, terrain: Terrain) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = terrain;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Food,
    Coin,
    Herb,
    Wood,
    Stone,
    Cloth,
    Iron,
}

impl ItemType {
    pub fn name(self) -> &'static str {
        match self {
            ItemType::Food => "Food",
            ItemType::Coin => "Coin",
            ItemType::Herb => "Herb",
            ItemType::Wood => "Wood",
            ItemType::Stone => "Stone",
            ItemType::Cloth => "Cloth",
            ItemType::Iron => "Iron",
        }
    }

    pub fn base_price(self) -> u32 {
        match self {
            ItemType::Coin => 1,
            ItemType::Herb => 2,
            ItemType::Food => 3,
            ItemType::Wood => 2,
            ItemType::Stone => 3,
            ItemType::Cloth => 4,
            ItemType::Iron => 5,
        }
    }

    pub fn tradeable(self) -> bool {
        self != ItemType::Coin
    }

    pub fn tradeable_items() -> Vec<ItemType> {
        vec![
            ItemType::Herb,
            ItemType::Food,
            ItemType::Wood,
            ItemType::Stone,
            ItemType::Cloth,
            ItemType::Iron,
        ]
    }

    pub fn gather_from(terrain: Terrain) -> Option<ItemType> {
        match terrain {
            Terrain::Grass | Terrain::Farmland | Terrain::Tundra => Some(ItemType::Herb),
            Terrain::Forest => Some(ItemType::Wood),
            Terrain::Settlement => Some(ItemType::Coin),
            Terrain::Coast => Some(ItemType::Food),
            Terrain::Sand
            | Terrain::DeepDesert
            | Terrain::Cave
            | Terrain::Swamp
            | Terrain::Water
            | Terrain::Mountain
            | Terrain::Road => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Inventory {
    pub items: std::collections::HashMap<ItemType, u32>,
    #[serde(default = "default_durability")]
    pub durability: std::collections::HashMap<ItemType, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcInteraction {
    pub action: EncounterAction,
    pub tick: u64,
    pub settlement: String,
    pub trust_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct NpcMemory {
    pub interactions: Vec<NpcInteraction>,
}

impl NpcMemory {
    pub fn last(&self) -> Option<&NpcInteraction> {
        self.interactions.last()
    }

    pub fn count(&self) -> usize {
        self.interactions.len()
    }

    pub fn cumulative_trust(&self) -> f64 {
        self.interactions.iter().map(|i| i.trust_delta).sum()
    }

    pub fn add(
        &mut self,
        action: EncounterAction,
        tick: u64,
        settlement: String,
        trust_delta: f64,
    ) {
        if self.interactions.len() >= 10 {
            self.interactions.remove(0);
        }
        self.interactions.push(NpcInteraction {
            action,
            tick,
            settlement,
            trust_delta,
        });
    }
}

fn default_durability() -> std::collections::HashMap<ItemType, f64> {
    std::collections::HashMap::new()
}

impl Inventory {
    pub fn get(&self, item: ItemType) -> u32 {
        self.items.get(&item).copied().unwrap_or(0)
    }

    pub fn durability(&self, item: ItemType) -> f64 {
        self.durability.get(&item).copied().unwrap_or(1.0)
    }

    pub fn is_broken(&self, item: ItemType) -> bool {
        self.has(item) && self.durability(item) <= 0.0
    }

    pub fn has(&self, item: ItemType) -> bool {
        self.get(item) > 0
    }

    pub fn decay(&mut self, item: ItemType, amount: f64) {
        if let Some(d) = self.durability.get_mut(&item) {
            *d = (*d - amount).max(0.0);
        }
    }

    pub fn repair_cost(&self, item: ItemType) -> u32 {
        let d = self.durability(item);
        if d >= 1.0 {
            return 0;
        }
        let base = item.base_price();
        ((1.0 - d) * base as f64 * 2.0).ceil() as u32
    }

    pub fn repair(&mut self, item: ItemType) -> u32 {
        let cost = self.repair_cost(item);
        if cost > 0 && self.durability.contains_key(&item) {
            self.durability.insert(item, 1.0);
        }
        cost
    }

    pub fn add(&mut self, item: ItemType, count: u32) {
        *self.items.entry(item).or_insert(0) += count;
        self.durability.entry(item).or_insert(1.0);
    }

    pub fn remove(&mut self, item: ItemType, count: u32) -> bool {
        let current = self.get(item);
        if current >= count {
            if count == current {
                self.items.remove(&item);
            } else {
                *self.items.get_mut(&item).unwrap() -= count;
            }
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CraftRecipe {
    pub name: String,
    pub inputs: Vec<(ItemType, u32)>,
    pub output: ItemType,
    pub output_count: u32,
    pub people: Option<PeopleKind>,
}

pub fn craft_recipes() -> Vec<CraftRecipe> {
    vec![
        CraftRecipe {
            name: "Bandage".into(),
            inputs: vec![(ItemType::Herb, 3), (ItemType::Cloth, 1)],
            output: ItemType::Food,
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Tool".into(),
            inputs: vec![(ItemType::Wood, 2), (ItemType::Iron, 1)],
            output: ItemType::Iron,
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Meal".into(),
            inputs: vec![(ItemType::Herb, 2), (ItemType::Food, 1)],
            output: ItemType::Food,
            output_count: 3,
            people: None,
        },
        CraftRecipe {
            name: "Sepät Forge-Kit".into(),
            inputs: vec![(ItemType::Iron, 3), (ItemType::Wood, 1)],
            output: ItemType::Iron,
            output_count: 5,
            people: Some(PeopleKind::Sepat),
        },
        CraftRecipe {
            name: "Ahjo Hearth-Meal".into(),
            inputs: vec![(ItemType::Food, 2), (ItemType::Herb, 1)],
            output: ItemType::Food,
            output_count: 6,
            people: Some(PeopleKind::Ahjo),
        },
        CraftRecipe {
            name: "Metsik Trap".into(),
            inputs: vec![(ItemType::Wood, 3), (ItemType::Herb, 1)],
            output: ItemType::Herb,
            output_count: 4,
            people: Some(PeopleKind::Metsik),
        },
    ]
}

pub fn npc_combat_action(trust: f64, aggression: f64, seed: u64) -> CombatAction {
    let mut rng = crate::rng::SeedRng::new(seed);
    let roll = rng.gen_range(1000) as f64 / 1000.0;

    // High trust = more defensive, low trust = more aggressive
    // High aggression = more likely to attack
    let attack_threshold = 0.3 + aggression * 0.4 - trust * 0.2;
    let parry_threshold = attack_threshold + 0.3;
    let feint_threshold = parry_threshold + 0.2;

    if roll < attack_threshold {
        CombatAction::Attack
    } else if roll < parry_threshold {
        CombatAction::Parry
    } else if roll < feint_threshold {
        CombatAction::Feint
    } else {
        CombatAction::Yield
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Dawn,
    Day,
    Dusk,
    Night,
}

impl TimeOfDay {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            5..=7 => TimeOfDay::Dawn,
            8..=17 => TimeOfDay::Day,
            18..=20 => TimeOfDay::Dusk,
            _ => TimeOfDay::Night,
        }
    }

    pub fn glyph(self) -> char {
        match self {
            TimeOfDay::Dawn => '☼',
            TimeOfDay::Day => '☀',
            TimeOfDay::Dusk => '◐',
            TimeOfDay::Night => '●',
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, TimeOfDay::Night)
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeOfDay::Dawn => write!(f, "Dawn"),
            TimeOfDay::Day => write!(f, "Day"),
            TimeOfDay::Dusk => write!(f, "Dusk"),
            TimeOfDay::Night => write!(f, "Night"),
        }
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
        let day_in_year = (day - 1) % 90;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
    Heatwave,
}

impl Weather {
    pub fn generate(seed: u64, tick: u64, terrain: Terrain) -> Self {
        let mut rng = crate::rng::SeedRng::new(seed.wrapping_add(tick));
        let roll = rng.gen_range(1000);

        // Regional bias
        let (clear_w, cloudy_w, rain_w, storm_w, snow_w, fog_w, heat_w) = match terrain {
            Terrain::Coast => (150, 250, 200, 100, 50, 200, 50),
            Terrain::Mountain => (200, 200, 150, 150, 150, 100, 50),
            Terrain::Forest => (150, 300, 250, 100, 50, 100, 50),
            Terrain::Swamp => (100, 200, 250, 100, 50, 250, 50),
            Terrain::DeepDesert | Terrain::Sand => (300, 200, 50, 50, 0, 50, 350),
            Terrain::Tundra => (150, 200, 100, 100, 350, 50, 50),
            _ => (200, 250, 200, 100, 100, 100, 50),
        };

        let total = clear_w + cloudy_w + rain_w + storm_w + snow_w + fog_w + heat_w;
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
        } else {
            Weather::Heatwave
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
        }
    }

    pub fn travel_speed_modifier(self) -> f64 {
        match self {
            Weather::Clear => 1.0,
            Weather::Cloudy => 0.95,
            Weather::Rain => 0.85,
            Weather::Storm => 0.6,
            Weather::Snow => 0.7,
            Weather::Fog => 0.75,
            Weather::Heatwave => 0.8,
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
        }
    }

    pub fn for_people(people: PeopleKind) -> Self {
        match people {
            PeopleKind::Ahjo => FestivalKind::HarvestFeast,
            PeopleKind::Sepat => FestivalKind::ForgeDay,
            PeopleKind::Metsik => FestivalKind::ForestRite,
            PeopleKind::Vayla => FestivalKind::RiverGathering,
            PeopleKind::Arkit => FestivalKind::AncestorVigil,
            PeopleKind::Laakso => FestivalKind::MidsummerBonfire,
            PeopleKind::Varhaiset => FestivalKind::AncestorVigil,
            PeopleKind::Metsareunat => FestivalKind::ForestRite,
            PeopleKind::Porokansa => FestivalKind::ForestRite,
            PeopleKind::Koskimetsa => FestivalKind::ForestRite,
            PeopleKind::Muistikansa => FestivalKind::AncestorVigil,
            PeopleKind::Taulukansa => FestivalKind::AncestorVigil,
            PeopleKind::Kirjakansa => FestivalKind::AncestorVigil,
            PeopleKind::Takovaki => FestivalKind::ForgeDay,
            PeopleKind::Rantavaki => FestivalKind::RiverGathering,
            PeopleKind::Saarivaki => FestivalKind::RiverGathering,
            PeopleKind::Hiekkakavelijat => FestivalKind::RiverGathering,
            PeopleKind::Haramaki => FestivalKind::MidsummerBonfire,
            PeopleKind::Jamavaki => FestivalKind::MidsummerBonfire,
            PeopleKind::Pohjavaki => FestivalKind::MidsummerBonfire,
            PeopleKind::Tzakhar => FestivalKind::AncestorVigil,
            PeopleKind::Merak => FestivalKind::RiverGathering,
            PeopleKind::Shear => FestivalKind::MidsummerBonfire,
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
        }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PlayerVitals {
    pub hunger: f64,
    pub energy: f64,
}

impl Default for PlayerVitals {
    fn default() -> Self {
        PlayerVitals {
            hunger: 1.0,
            energy: 1.0,
        }
    }
}

impl PlayerVitals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self, hours: u32, inventory: &mut Inventory, season: Season) {
        let hunger_rate = 0.05 * season.need_decay_multiplier();
        let energy_rate = 0.02 * season.need_decay_multiplier();
        for _ in 0..hours {
            self.hunger -= hunger_rate;
            self.energy -= energy_rate;
            if self.hunger <= 0.3 && inventory.remove(ItemType::Food, 1) {
                self.hunger = (self.hunger + 0.3).min(1.0);
            }
        }
        self.hunger = self.hunger.max(0.0);
        self.energy = self.energy.max(0.0);
    }

    pub fn rest(&mut self) {
        self.energy = (self.energy + 0.6).min(1.0);
    }

    pub fn is_starving(self) -> bool {
        self.hunger <= 0.0
    }

    pub fn is_exhausted(self) -> bool {
        self.energy <= 0.1
    }

    pub fn hunger_label(self) -> &'static str {
        if self.hunger >= 0.7 {
            "full"
        } else if self.hunger >= 0.4 {
            "hungry"
        } else if self.hunger > 0.0 {
            "starving"
        } else {
            "famished"
        }
    }

    pub fn energy_label(self) -> &'static str {
        if self.energy >= 0.7 {
            "energized"
        } else if self.energy >= 0.4 {
            "tired"
        } else {
            "exhausted"
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GodName {
    Oltzed,
    Keuru,
    Sampsa,
    Masa,
    Kukri,
}

impl GodName {
    pub fn label(self) -> &'static str {
        match self {
            GodName::Oltzed => "Oltzed",
            GodName::Keuru => "Keuru",
            GodName::Sampsa => "Sampsa",
            GodName::Masa => "Masa",
            GodName::Kukri => "Kukri",
        }
    }

    pub fn domains(self) -> &'static str {
        match self {
            GodName::Oltzed => "labor, invention, engineering",
            GodName::Keuru => "forests, hospitality, celebration",
            GodName::Sampsa => "knowledge, memory, archives",
            GodName::Masa => "trade, perseverance, common people",
            GodName::Kukri => "solitude, old wisdom, nostalgia",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            GodName::Oltzed => '⚒',
            GodName::Keuru => '🌲',
            GodName::Sampsa => '📖',
            GodName::Masa => '⚖',
            GodName::Kukri => '🕯',
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GodAffinity {
    #[serde(default)]
    pub oltzed: f64,
    #[serde(default)]
    pub keuru: f64,
    #[serde(default)]
    pub sampsa: f64,
    #[serde(default)]
    pub masa: f64,
    #[serde(default)]
    pub kukri: f64,
}

impl Default for GodAffinity {
    fn default() -> Self {
        GodAffinity {
            oltzed: 0.0,
            keuru: 0.0,
            sampsa: 0.0,
            masa: 0.0,
            kukri: 0.0,
        }
    }
}

impl GodAffinity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, god: GodName) -> f64 {
        match god {
            GodName::Oltzed => self.oltzed,
            GodName::Keuru => self.keuru,
            GodName::Sampsa => self.sampsa,
            GodName::Masa => self.masa,
            GodName::Kukri => self.kukri,
        }
    }

    pub fn adjust(&mut self, god: GodName, delta: f64) {
        let val = match god {
            GodName::Oltzed => &mut self.oltzed,
            GodName::Keuru => &mut self.keuru,
            GodName::Sampsa => &mut self.sampsa,
            GodName::Masa => &mut self.masa,
            GodName::Kukri => &mut self.kukri,
        };
        *val = (*val + delta).clamp(-1.0, 1.0);
    }

    pub fn strongest_ally(&self) -> Option<GodName> {
        let gods = [
            (GodName::Oltzed, self.oltzed),
            (GodName::Keuru, self.keuru),
            (GodName::Sampsa, self.sampsa),
            (GodName::Masa, self.masa),
            (GodName::Kukri, self.kukri),
        ];
        let best = gods
            .iter()
            .filter(|(_, v)| *v > 0.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        best.map(|(g, _)| *g)
    }

    pub fn strongest_grudge(&self) -> Option<GodName> {
        let gods = [
            (GodName::Oltzed, self.oltzed),
            (GodName::Keuru, self.keuru),
            (GodName::Sampsa, self.sampsa),
            (GodName::Masa, self.masa),
            (GodName::Kukri, self.kukri),
        ];
        let worst = gods
            .iter()
            .filter(|(_, v)| *v < 0.0)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        worst.map(|(g, _)| *g)
    }

    pub fn people_title(self, people: PeopleKind) -> &'static str {
        let affinity = self.get(people.patron_god().unwrap_or(GodName::Oltzed));
        if affinity > 0.6 {
            match people {
                PeopleKind::Metsik => "Friend of the Forest",
                PeopleKind::Ahjo => "Hearth-kin",
                PeopleKind::Sepat => "Iron-Bound",
                PeopleKind::Vayla => "River-Walker",
                PeopleKind::Arkit => "Archive-Shadow",
                PeopleKind::Laakso => "Deep-Patient",
                PeopleKind::Varhaiset => "Ancient-Known",
                PeopleKind::Metsareunat => "Edge-Friend",
                PeopleKind::Porokansa => "Herd-Brother",
                PeopleKind::Koskimetsa => "Rapids-Kin",
                PeopleKind::Muistikansa => "Song-Keeper",
                PeopleKind::Taulukansa => "Tablet-Friend",
                PeopleKind::Kirjakansa => "Book-Brother",
                PeopleKind::Takovaki => "Copper-Bound",
                PeopleKind::Rantavaki => "Shore-Kin",
                PeopleKind::Saarivaki => "Island-Brother",
                PeopleKind::Hiekkakavelijat => "Sand-Kin",
                PeopleKind::Haramaki => "Terrace-Friend",
                PeopleKind::Jamavaki => "Hidden-Known",
                PeopleKind::Pohjavaki => "Deep-Root",
                PeopleKind::Tzakhar => "Cave-Dweller",
                PeopleKind::Merak => "Wave-Rider",
                PeopleKind::Shear => "Sand-Walker",
                PeopleKind::Hal => "Canopy-Friend",
                PeopleKind::Khor => "Frost-Enduring",
            }
        } else if affinity > 0.3 {
            match people {
                PeopleKind::Metsik => "Wood-Familiar",
                PeopleKind::Ahjo => "Settlement-Guest",
                PeopleKind::Sepat => "Forge-Acquainted",
                PeopleKind::Vayla => "Trade-Known",
                PeopleKind::Arkit => "Page-Turner",
                PeopleKind::Laakso => "Steady-Presence",
                PeopleKind::Varhaiset => "Old-Acquaintance",
                PeopleKind::Metsareunat => "Margin-Known",
                PeopleKind::Porokansa => "Herd-Aware",
                PeopleKind::Koskimetsa => "River-Aware",
                PeopleKind::Muistikansa => "Song-Heard",
                PeopleKind::Taulukansa => "Tablet-Aware",
                PeopleKind::Kirjakansa => "Book-Aware",
                PeopleKind::Takovaki => "Copper-Aware",
                PeopleKind::Rantavaki => "Shore-Acquainted",
                PeopleKind::Saarivaki => "Island-Acquainted",
                PeopleKind::Hiekkakavelijat => "Sand-Aware",
                PeopleKind::Haramaki => "Terrace-Acquainted",
                PeopleKind::Jamavaki => "Valley-Aware",
                PeopleKind::Pohjavaki => "Depth-Aware",
                PeopleKind::Tzakhar => "Depth-Curious",
                PeopleKind::Merak => "Shore-Acquainted",
                PeopleKind::Shear => "Heat-Tolerant",
                PeopleKind::Hal => "Branch-Greeter",
                PeopleKind::Khor => "Cold-Respected",
            }
        } else if affinity < -0.2 {
            match people {
                PeopleKind::Metsik => "Tree-Feller",
                PeopleKind::Ahjo => "Hearth-Stranger",
                PeopleKind::Sepat => "Ore-Taker",
                PeopleKind::Vayla => "Current-Breaker",
                PeopleKind::Arkit => "Page-Burner",
                PeopleKind::Laakso => "Impatient-One",
                PeopleKind::Varhaiset => "Ancient-Shadow",
                PeopleKind::Metsareunat => "Edge-Usurper",
                PeopleKind::Porokansa => "Herd-Thief",
                PeopleKind::Koskimetsa => "Rapids-Blocker",
                PeopleKind::Muistikansa => "Song-Silencer",
                PeopleKind::Taulukansa => "Tablet-Smasher",
                PeopleKind::Kirjakansa => "Book-Burner",
                PeopleKind::Takovaki => "Copper-Thief",
                PeopleKind::Rantavaki => "Shore-Taker",
                PeopleKind::Saarivaki => "Island-Invader",
                PeopleKind::Hiekkakavelijat => "Sand-Grabber",
                PeopleKind::Haramaki => "Terrace-Seizer",
                PeopleKind::Jamavaki => "Valley-Defiler",
                PeopleKind::Pohjavaki => "Depth-Robber",
                PeopleKind::Tzakhar => "Surface-Clasher",
                PeopleKind::Merak => "Land-Anchor",
                PeopleKind::Shear => "Oasis-Hoarder",
                PeopleKind::Hal => "Canopy-Skimmer",
                PeopleKind::Khor => "Warmth-Seeker",
            }
        } else {
            ""
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PeopleKind {
    #[default]
    Metsik,
    Arkit,
    Vayla,
    Laakso,
    Sepat,
    Ahjo,
    Varhaiset,
    Metsareunat,
    Porokansa,
    Koskimetsa,
    Muistikansa,
    Taulukansa,
    Kirjakansa,
    Takovaki,
    Rantavaki,
    Saarivaki,
    Hiekkakavelijat,
    Haramaki,
    Jamavaki,
    Pohjavaki,
    Tzakhar,
    Merak,
    Shear,
    Hal,
    Khor,
}

impl PeopleKind {
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "metsik" | "čyrvä" | "keurimä" => PeopleKind::Metsik,
            "arkit" | "märät" | "sampsari" => PeopleKind::Arkit,
            "vayla" | "väylä" | "vylti" | "masari" => PeopleKind::Vayla,
            "laakso" | "kiškam" | "kukreva" => PeopleKind::Laakso,
            "sepat" | "sepät" | "wosyt" => PeopleKind::Sepat,
            "ahjo" | "njumka" | "iltkari" => PeopleKind::Ahjo,
            "varhaiset" | "körvä" | "perikansan" => PeopleKind::Varhaiset,
            "metsareunat" | "metsäreunat" | "pyršä" | "keurunreunat" => PeopleKind::Metsareunat,
            "porokansa" | "tuorva" | "keuruporo" => PeopleKind::Porokansa,
            "koskimetsa" | "koskimetsä" | "jälky" | "keurukoski" => PeopleKind::Koskimetsa,
            "muistikansa" | "särät" | "sampsamuisti" => PeopleKind::Muistikansa,
            "taulukansa" | "velmät" | "sampsataulu" => PeopleKind::Taulukansa,
            "kirjakansa" | "tärent" | "sampsakirja" => PeopleKind::Kirjakansa,
            "takovaki" | "takoväki" | "wonśyt" | "oltkartako" => PeopleKind::Takovaki,
            "rantavaki" | "rantaväki" | "vylri" | "masaranta" => PeopleKind::Rantavaki,
            "saarivaki" | "saariväki" | "kylmi" | "masasaari" => PeopleKind::Saarivaki,
            "hiekkakavelijat" | "hiekkakävelijät" | "tyrväi" | "masahiekka" => {
                PeopleKind::Hiekkakavelijat
            }
            "haramaki" | "härämäki" | "kišmäs" | "kukriharma" => PeopleKind::Haramaki,
            "jamavaki" | "jämäväki" | "hoskam" | "kukrijämä" => PeopleKind::Jamavaki,
            "pohjavaki" | "pohjaväki" | "väškam" | "kukripohja" => PeopleKind::Pohjavaki,
            "tzakhar" | "tzäkhar" | "vaskiluuri" => PeopleKind::Tzakhar,
            "merak" | "mëräk" | "iltäkälä" => PeopleKind::Merak,
            "shear" | "she'ar" | "muraskala" => PeopleKind::Shear,
            "hal" | "häl" => PeopleKind::Hal,
            "khor" | "khör" | "khmört" => PeopleKind::Khor,
            _ => PeopleKind::Metsik,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Forest-people",
            PeopleKind::Arkit => "Archive-keepers",
            PeopleKind::Vayla => "River-folk",
            PeopleKind::Laakso => "Vale-dwellers",
            PeopleKind::Sepat => "Iron-people",
            PeopleKind::Ahjo => "Sacred-forge folk",
            PeopleKind::Varhaiset => "The Earliest Ones — highland headwaters folk",
            PeopleKind::Metsareunat => "Forest-edge people",
            PeopleKind::Porokansa => "Reindeer-following people",
            PeopleKind::Koskimetsa => "River-rapids forest people",
            PeopleKind::Muistikansa => "Memory-keepers — oral tradition folk",
            PeopleKind::Taulukansa => "Tablet-people — independent scribes",
            PeopleKind::Kirjakansa => "Book-people — river-valley scribes",
            PeopleKind::Takovaki => "Surface-copper forge folk",
            PeopleKind::Rantavaki => "Shore-folk — tidepool cultivators",
            PeopleKind::Saarivaki => "Island-folk — inland sea traders",
            PeopleKind::Hiekkakavelijat => "Sand-walkers — beach-ridge lagoon folk",
            PeopleKind::Haramaki => "Steep-valley people — dry-terrace farmers",
            PeopleKind::Jamavaki => "Hidden-valley people — hermit traditions",
            PeopleKind::Pohjavaki => "Deep-bottom people — depth-silence keepers",
            PeopleKind::Tzakhar => "Deep-cave people",
            PeopleKind::Merak => "Sea-people",
            PeopleKind::Shear => "Desert people",
            PeopleKind::Hal => "Canopy people",
            PeopleKind::Khor => "Tundra people",
        }
    }

    pub fn patron_god(self) -> Option<GodName> {
        match self {
            PeopleKind::Metsik => Some(GodName::Keuru),
            PeopleKind::Ahjo | PeopleKind::Sepat => Some(GodName::Oltzed),
            PeopleKind::Vayla => Some(GodName::Masa),
            PeopleKind::Arkit => Some(GodName::Sampsa),
            PeopleKind::Laakso => Some(GodName::Kukri),
            // Keurish family: Keuru
            PeopleKind::Varhaiset => None, // All Five (travelers saw all gods pass through)
            PeopleKind::Metsareunat => Some(GodName::Keuru),
            PeopleKind::Porokansa => Some(GodName::Keuru),
            PeopleKind::Koskimetsa => Some(GodName::Keuru),
            // Sampsaran family: Sampsa
            PeopleKind::Muistikansa => Some(GodName::Sampsa),
            PeopleKind::Taulukansa => Some(GodName::Sampsa),
            PeopleKind::Kirjakansa => Some(GodName::Sampsa),
            // Oltkar family: Oltzed
            PeopleKind::Takovaki => Some(GodName::Oltzed),
            // Masaran family: Masa
            PeopleKind::Rantavaki => Some(GodName::Masa),
            PeopleKind::Saarivaki => Some(GodName::Masa),
            PeopleKind::Hiekkakavelijat => Some(GodName::Masa),
            // Kukresh family: Kukri
            PeopleKind::Haramaki => Some(GodName::Kukri),
            PeopleKind::Jamavaki => Some(GodName::Kukri),
            PeopleKind::Pohjavaki => Some(GodName::Kukri),
            // Non-human
            PeopleKind::Tzakhar => Some(GodName::Kukri),
            PeopleKind::Merak => Some(GodName::Masa),
            PeopleKind::Shear => None,
            PeopleKind::Hal => Some(GodName::Keuru),
            PeopleKind::Khor => Some(GodName::Sampsa),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Metsik",
            PeopleKind::Arkit => "Arkit",
            PeopleKind::Vayla => "Väylä",
            PeopleKind::Laakso => "Laakso",
            PeopleKind::Sepat => "Sepät",
            PeopleKind::Ahjo => "Ahjo",
            PeopleKind::Varhaiset => "Varhaiset",
            PeopleKind::Metsareunat => "Metsäreunat",
            PeopleKind::Porokansa => "Porokansa",
            PeopleKind::Koskimetsa => "Koskimetsä",
            PeopleKind::Muistikansa => "Muistikansa",
            PeopleKind::Taulukansa => "Taulukansa",
            PeopleKind::Kirjakansa => "Kirjakansa",
            PeopleKind::Takovaki => "Takoväki",
            PeopleKind::Rantavaki => "Rantaväki",
            PeopleKind::Saarivaki => "Saariväki",
            PeopleKind::Hiekkakavelijat => "Hiekkakävelijät",
            PeopleKind::Haramaki => "Härämäki",
            PeopleKind::Jamavaki => "Jämäväki",
            PeopleKind::Pohjavaki => "Pohjaväki",
            PeopleKind::Tzakhar => "Tzäkhar",
            PeopleKind::Merak => "Mëräk",
            PeopleKind::Shear => "She'ar",
            PeopleKind::Hal => "Häl",
            PeopleKind::Khor => "Khör",
        }
    }

    pub fn bias_toward(self, other: PeopleKind) -> f64 {
        if self == other {
            return 0.15;
        }
        match (self, other) {
            // Human-human biases (preserved from original)
            (PeopleKind::Metsik, PeopleKind::Sepat) => -0.20,
            (PeopleKind::Metsik, PeopleKind::Ahjo) => -0.15,
            (PeopleKind::Sepat, PeopleKind::Metsik) => -0.15,
            (PeopleKind::Ahjo, PeopleKind::Metsik) => -0.12,
            (PeopleKind::Sepat, PeopleKind::Ahjo) => 0.10,
            (PeopleKind::Ahjo, PeopleKind::Sepat) => 0.10,
            (PeopleKind::Metsik, PeopleKind::Vayla) => -0.05,
            (PeopleKind::Vayla, PeopleKind::Metsik) => -0.05,
            (PeopleKind::Sepat, PeopleKind::Arkit) => 0.08,
            (PeopleKind::Arkit, PeopleKind::Sepat) => 0.05,
            (PeopleKind::Ahjo, PeopleKind::Vayla) => 0.05,
            (PeopleKind::Vayla, PeopleKind::Ahjo) => 0.05,
            (PeopleKind::Laakso, PeopleKind::Metsik) => 0.05,
            (PeopleKind::Metsik, PeopleKind::Laakso) => 0.05,
            (PeopleKind::Laakso, PeopleKind::Vayla) => -0.08,
            (PeopleKind::Vayla, PeopleKind::Laakso) => -0.05,
            // Non-human to human: generally neutral-wary
            (PeopleKind::Tzakhar, PeopleKind::Metsik) => 0.02,
            (PeopleKind::Tzakhar, PeopleKind::Laakso) => 0.05,
            (PeopleKind::Tzakhar, _) => -0.03,
            (PeopleKind::Merak, PeopleKind::Vayla) => 0.08,
            (PeopleKind::Merak, PeopleKind::Ahjo) => 0.03,
            (PeopleKind::Merak, _) => -0.02,
            (PeopleKind::Shear, _) => -0.05,
            (PeopleKind::Hal, PeopleKind::Metsik) => 0.08,
            (PeopleKind::Hal, _) => -0.02,
            (PeopleKind::Khor, PeopleKind::Arkit) => 0.03,
            (PeopleKind::Khor, _) => -0.04,
            // Human to non-human: generally curious but distant
            (PeopleKind::Metsik, PeopleKind::Hal) => 0.05,
            (PeopleKind::Vayla, PeopleKind::Merak) => 0.06,
            (PeopleKind::Laakso, PeopleKind::Tzakhar) => 0.04,
            (PeopleKind::Arkit, PeopleKind::Khor) => 0.02,
            (PeopleKind::Sepat, PeopleKind::Tzakhar) => -0.03,
            // Stayed peoples to SAST peoples
            (PeopleKind::Varhaiset, PeopleKind::Metsik) => 0.05,
            (PeopleKind::Varhaiset, PeopleKind::Arkit) => 0.02,
            (PeopleKind::Metsareunat, PeopleKind::Metsik) => 0.08,
            (PeopleKind::Metsareunat, PeopleKind::Arkit) => 0.02,
            (PeopleKind::Porokansa, PeopleKind::Metsik) => 0.06,
            (PeopleKind::Koskimetsa, PeopleKind::Metsik) => 0.07,
            (PeopleKind::Koskimetsa, PeopleKind::Vayla) => 0.04,
            (PeopleKind::Muistikansa, PeopleKind::Arkit) => 0.08,
            (PeopleKind::Taulukansa, PeopleKind::Arkit) => 0.06,
            (PeopleKind::Kirjakansa, PeopleKind::Arkit) => 0.05,
            (PeopleKind::Takovaki, PeopleKind::Sepat) => 0.06,
            (PeopleKind::Takovaki, PeopleKind::Ahjo) => 0.04,
            (PeopleKind::Rantavaki, PeopleKind::Vayla) => 0.07,
            (PeopleKind::Saarivaki, PeopleKind::Vayla) => 0.09,
            (PeopleKind::Saarivaki, PeopleKind::Merak) => 0.05,
            (PeopleKind::Hiekkakavelijat, PeopleKind::Vayla) => 0.06,
            (PeopleKind::Haramaki, PeopleKind::Laakso) => 0.08,
            (PeopleKind::Jamavaki, PeopleKind::Laakso) => 0.09,
            (PeopleKind::Pohjavaki, PeopleKind::Laakso) => 0.10,
            // SAST peoples to stayed peoples (reciprocal)
            (PeopleKind::Metsik, PeopleKind::Koskimetsa) => 0.06,
            (PeopleKind::Metsik, PeopleKind::Metsareunat) => 0.05,
            (PeopleKind::Arkit, PeopleKind::Muistikansa) => 0.07,
            (PeopleKind::Vayla, PeopleKind::Saarivaki) => 0.08,
            (PeopleKind::Laakso, PeopleKind::Haramaki) => 0.07,
            // Stayed to stayed (same family = positive)
            (PeopleKind::Koskimetsa, PeopleKind::Varhaiset) => 0.04,
            (PeopleKind::Koskimetsa, PeopleKind::Metsareunat) => 0.05,
            (PeopleKind::Muistikansa, PeopleKind::Taulukansa) => 0.06,
            (PeopleKind::Muistikansa, PeopleKind::Kirjakansa) => 0.06,
            (PeopleKind::Taulukansa, PeopleKind::Kirjakansa) => 0.07,
            (PeopleKind::Rantavaki, PeopleKind::Saarivaki) => 0.05,
            (PeopleKind::Haramaki, PeopleKind::Jamavaki) => 0.06,
            (PeopleKind::Haramaki, PeopleKind::Pohjavaki) => 0.07,
            (PeopleKind::Jamavaki, PeopleKind::Pohjavaki) => 0.08,
            // Stayed peoples to non-human
            (PeopleKind::Koskimetsa, PeopleKind::Hal) => 0.04,
            (PeopleKind::Rantavaki, PeopleKind::Merak) => 0.04,
            (PeopleKind::Haramaki, PeopleKind::Tzakhar) => 0.03,
            (PeopleKind::Pohjavaki, PeopleKind::Tzakhar) => 0.04,
            // Default
            (PeopleKind::Laakso, _) => -0.05,
            (PeopleKind::Arkit, _) => 0.0,
            (PeopleKind::Vayla, _) => 0.0,
            _ => 0.0,
        }
    }

    pub fn greeting_to(self, other: PeopleKind) -> &'static str {
        if self == other {
            return "You are among your own. Their eyes warm with recognition.";
        }
        match (self, other) {
            (PeopleKind::Metsik, PeopleKind::Sepat) | (PeopleKind::Metsik, PeopleKind::Ahjo) => {
                "They size you up. Forest-people are not loved here. 'Clearing-sympathizer,' someone mutters."
            }
            (PeopleKind::Sepat, PeopleKind::Metsik) => {
                "They eye your hands. 'Forest-dweller. Your kind took good iron-ore ground.'"
            }
            (PeopleKind::Ahjo, PeopleKind::Metsik) => {
                "A guarded look. 'Another one who thinks trees matter more than forge-heat.'"
            }
            (PeopleKind::Metsik, PeopleKind::Vayla) => {
                "Neutral, but watchful. 'Trader. You would sell the forest if the price was right.'"
            }
            (PeopleKind::Vayla, PeopleKind::Metsik) => {
                "Interested. 'Forest goods fetch fine prices. But we respect your... boundaries.'"
            }
            (PeopleKind::Laakso, PeopleKind::Vayla) => {
                "A long pause. 'You move too fast. Everything with you is a transaction.'"
            }
            (PeopleKind::Sepat, PeopleKind::Arkit) => {
                "A nod of respect. 'Keeper of knowledge. That is honest work.'"
            }
            (PeopleKind::Arkit, PeopleKind::Sepat) => {
                "Professional courtesy. 'Makers. You preserve things differently than we do.'"
            }
            (PeopleKind::Laakso, _) => "They watch you in silence. You must prove patience to earn their warmth.",
            (PeopleKind::Vayla, _) => "Open face, calculating eyes. 'Welcome, stranger. What do you trade?'",
            (PeopleKind::Arkit, _) => "Polite, measured. The archivist's neutral welcome.",
            (PeopleKind::Tzakhar, PeopleKind::Laakso) => "A faint nod from the shadows. 'Cave-kin. We know your ways.'",
            (PeopleKind::Tzakhar, _) => "Eyes that have seen too much dark. 'You are not of the deep. Walk carefully.'",
            (PeopleKind::Merak, PeopleKind::Vayla) => "A salty grin. 'River-folk! We share a love of water, at least.'",
            (PeopleKind::Merak, _) => "Sea-salt in the air. 'Land-walker. What brings you shoreward?'",
            (PeopleKind::Shear, _) => "Distant eyes. 'The sun is harsh. But we endure. As must you.'",
            (PeopleKind::Hal, PeopleKind::Metsik) => "A sway and a smile from above. 'Cousins of the canopy! Welcome.'",
            (PeopleKind::Hal, _) => "Bright eyes from the branches. 'Ground-walker. Interesting.'",
            (PeopleKind::Khor, PeopleKind::Arkit) => "A slow nod. 'You keep memory. We keep survival. Same work, different tool.'",
            (PeopleKind::Khor, _) => "A breath in the cold. 'The wind tests all equally. You are being tested now.'",
            // Stayed peoples greeting SAST
            (PeopleKind::Varhaiset, _) => "They look at you as if from a very long time ago. 'The source remembers.'",
            (PeopleKind::Metsareunat, PeopleKind::Metsik) => "A knowing look from the forest-edge. 'Cousin-who-went-deeper. The margin remembers.'",
            (PeopleKind::Porokansa, PeopleKind::Metsik) => "A reindeer-follower's measured gaze. 'The herd moved on. We follow still.'",
            (PeopleKind::Koskimetsa, PeopleKind::Metsik) => "Quiet eyes from the rapids. 'We fish where you hunt. The river runs between us.'",
            (PeopleKind::Koskimetsa, _) => "Quiet, appraising eyes. They offer river-fish without a word.",
            (PeopleKind::Muistikansa, PeopleKind::Arkit) => "An ancient stare. 'We sang before you wrote. The song is older than the tablet.'",
            (PeopleKind::Taulukansa, PeopleKind::Arkit) => "A wry smile. 'We had tablets too. We just chose not to stack them into an Archive.'",
            (PeopleKind::Kirjakansa, PeopleKind::Arkit) => "Careful courtesy. 'The river carried the books. The books carried us. We both serve memory, differently.'",
            (PeopleKind::Takovaki, PeopleKind::Sepat) => "A forge-worker's nod, but cautious. 'We worked copper before you found iron. The trailing sound, they called our god.'",
            (PeopleKind::Rantavaki, PeopleKind::Vayla) => "A shore-salt greeting. 'We cultivated the tide-pools while you sailed the deep. Both are water's work.'",
            (PeopleKind::Saarivaki, PeopleKind::Vayla) => "An islander's open welcome. 'Island-kin! The waves bring us news of you.'",
            (PeopleKind::Saarivaki, _) => "An islander's measured curiosity. 'Mainland-walker. The tides bring all kinds here.'",
            (PeopleKind::Hiekkakavelijat, PeopleKind::Vayla) => "A sand-walker's grin. 'We walk the beach-ridges. You sail past them. Same water, different legs.'",
            (PeopleKind::Haramaki, PeopleKind::Laakso) => "A rare, almost warm look. 'Härma valley recognizes Kyrö valley. The slopes remember.'",
            (PeopleKind::Jamavaki, PeopleKind::Laakso) => "Silence, then a slight nod. 'The hidden valley keeps its own counsel. As do we.'",
            (PeopleKind::Pohjavaki, PeopleKind::Laakso) => "Deep silence, then three words: 'The bottom speaks.'",
            (PeopleKind::Pohjavaki, _) => "Bottom-eyes. They watch you from the deepest place. 'You stand high. We stand in the root.'",
            _ => "Cautious eyes. A stranger from another people.",
        }
    }

    pub fn trade_modifier(self, seller: PeopleKind) -> f64 {
        1.0 - seller.bias_toward(self) * 0.3
    }

    pub fn true_endonym(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Čyrvä",
            PeopleKind::Arkit => "Märät",
            PeopleKind::Sepat => "Wosyt",
            PeopleKind::Ahjo => "Njumka",
            PeopleKind::Vayla => "Vylti",
            PeopleKind::Laakso => "Kiškam",
            // Stayed peoples: opaque roots per language family branch
            PeopleKind::Varhaiset => "Körvä",
            PeopleKind::Metsareunat => "Pyršä",
            PeopleKind::Porokansa => "Tuorva",
            PeopleKind::Koskimetsa => "Jälky",
            PeopleKind::Muistikansa => "Särät",
            PeopleKind::Taulukansa => "Velmät",
            PeopleKind::Kirjakansa => "Tärent",
            PeopleKind::Takovaki => "Wonśyt",
            PeopleKind::Rantavaki => "Vylri",
            PeopleKind::Saarivaki => "Kylmi",
            PeopleKind::Hiekkakavelijat => "Tyrväi",
            PeopleKind::Haramaki => "Kišmäs",
            PeopleKind::Jamavaki => "Hoskam",
            PeopleKind::Pohjavaki => "Väškam",
            // Non-humans: opaque native endonyms
            PeopleKind::Tzakhar => "Tzäkhar",
            PeopleKind::Merak => "Mëräk",
            PeopleKind::Shear => "She'ar",
            PeopleKind::Hal => "Häl",
            PeopleKind::Khor => "Khör",
        }
    }

    pub fn arkit_name(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Metsik",
            PeopleKind::Arkit => "Arkit",
            PeopleKind::Sepat => "Sepät",
            PeopleKind::Ahjo => "Ahjo",
            PeopleKind::Vayla => "Väylä",
            PeopleKind::Laakso => "Laakso",
            PeopleKind::Varhaiset => "Varhaiset",
            PeopleKind::Metsareunat => "Metsäreunat",
            PeopleKind::Porokansa => "Porokansa",
            PeopleKind::Koskimetsa => "Koskimetsä",
            PeopleKind::Muistikansa => "Muistikansa",
            PeopleKind::Taulukansa => "Taulukansa",
            PeopleKind::Kirjakansa => "Kirjakansa",
            PeopleKind::Takovaki => "Takoväki",
            PeopleKind::Rantavaki => "Rantaväki",
            PeopleKind::Saarivaki => "Saariväki",
            PeopleKind::Hiekkakavelijat => "Hiekkakävelijät",
            PeopleKind::Haramaki => "Härämäki",
            PeopleKind::Jamavaki => "Jämäväki",
            PeopleKind::Pohjavaki => "Pohjaväki",
            PeopleKind::Tzakhar => "Vaskiluuri",
            PeopleKind::Merak => "Iltäkälä",
            PeopleKind::Shear => "Muraskala",
            PeopleKind::Hal => "Khör",
            PeopleKind::Khor => "Khmört",
        }
    }

    pub fn pilgrimage_exonym(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Keurimä",
            PeopleKind::Arkit => "Sampsari",
            PeopleKind::Sepat => "Sepät",
            PeopleKind::Ahjo => "Iltkari",
            PeopleKind::Vayla => "Masari",
            PeopleKind::Laakso => "Kukreva",
            PeopleKind::Varhaiset => "Perikansan",
            PeopleKind::Metsareunat => "Keurunreunat",
            PeopleKind::Porokansa => "Keuruporo",
            PeopleKind::Koskimetsa => "Keurukoski",
            PeopleKind::Muistikansa => "Sampsamuisti",
            PeopleKind::Taulukansa => "Sampsataulu",
            PeopleKind::Kirjakansa => "Sampsakirja",
            PeopleKind::Takovaki => "Oltkartako",
            PeopleKind::Rantavaki => "Masaranta",
            PeopleKind::Saarivaki => "Masasaari",
            PeopleKind::Hiekkakavelijat => "Masahiekka",
            PeopleKind::Haramaki => "Kukriharma",
            PeopleKind::Jamavaki => "Kukrijämä",
            PeopleKind::Pohjavaki => "Kukripohja",
            PeopleKind::Tzakhar => "Vaskiluuri",
            PeopleKind::Merak => "Iltäkälä",
            PeopleKind::Shear => "Muraskala",
            PeopleKind::Hal => "Khör",
            PeopleKind::Khor => "Khmört",
        }
    }

    pub fn language_family(self) -> &'static str {
        match self {
            PeopleKind::Metsik
            | PeopleKind::Varhaiset
            | PeopleKind::Metsareunat
            | PeopleKind::Porokansa
            | PeopleKind::Koskimetsa => "Keurish",
            PeopleKind::Arkit
            | PeopleKind::Muistikansa
            | PeopleKind::Taulukansa
            | PeopleKind::Kirjakansa => "Sampsaran",
            PeopleKind::Sepat | PeopleKind::Ahjo | PeopleKind::Takovaki => "Oltkar",
            PeopleKind::Vayla
            | PeopleKind::Rantavaki
            | PeopleKind::Saarivaki
            | PeopleKind::Hiekkakavelijat => "Masaran",
            PeopleKind::Laakso
            | PeopleKind::Haramaki
            | PeopleKind::Jamavaki
            | PeopleKind::Pohjavaki => "Kukresh",
            PeopleKind::Tzakhar => "Deep-Isolate",
            PeopleKind::Merak => "Coastal-Isolate",
            PeopleKind::Shear => "Desert-Isolate",
            PeopleKind::Hal => "Canopy-Isolate",
            PeopleKind::Khor => "Steppe-Isolate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InterPeopleBias {
    pub player_people: PeopleKind,
    #[serde(default)]
    pub bias_modifiers: HashMap<String, f64>,
}

impl InterPeopleBias {
    pub fn new(player_people: PeopleKind) -> Self {
        InterPeopleBias {
            player_people,
            bias_modifiers: HashMap::new(),
        }
    }

    pub fn mod_toward(&mut self, other: PeopleKind, delta: f64) {
        let key = format!("{:?}", other);
        let entry = self.bias_modifiers.entry(key).or_insert(0.0);
        *entry = (*entry + delta).clamp(-0.15, 0.15);
    }

    pub fn effective_bias(&self, other: PeopleKind) -> f64 {
        let base = self.player_people.bias_toward(other);
        let key = format!("{:?}", other);
        let modifier = self.bias_modifiers.get(&key).copied().unwrap_or(0.0);
        base + modifier
    }

    pub fn trust_baseline(&self, npc_people: PeopleKind) -> f64 {
        let bias = self.effective_bias(npc_people);
        (0.5 + bias).clamp(0.1, 0.9)
    }

    pub fn npc_trust_baseline(&self, npc_people: PeopleKind) -> f64 {
        let bias = npc_people.bias_toward(self.player_people);
        (0.5 + bias).clamp(0.1, 0.9)
    }

    pub fn strength_modifier(&self, npc_people: PeopleKind) -> f64 {
        self.effective_bias(npc_people) * 0.5
    }

    pub fn price_modifier(&self, seller_people: PeopleKind) -> f64 {
        self.player_people.trade_modifier(seller_people)
    }

    pub fn personality_mod(personality: &[String]) -> f64 {
        let mut mod_val = 0.0;
        for trait_val in personality {
            match trait_val.as_str() {
                "hospitable" | "warm" | "generous" => mod_val += 0.08,
                "xenophobic" | "suspicious" | "insular" => mod_val -= 0.10,
                "cautious" | "guarded" => mod_val -= 0.04,
                "open" | "curious" => mod_val += 0.05,
                _ => {}
            }
        }
        mod_val
    }

    pub fn trade_price_modifier(personality: &[String]) -> f64 {
        let mut mod_val = 0.0;
        for trait_val in personality {
            match trait_val.as_str() {
                "hospitable" | "generous" => mod_val -= 0.05,
                "mercenary" | "greedy" => mod_val += 0.08,
                "miserly" => mod_val += 0.10,
                "cautious" | "guarded" => mod_val += 0.03,
                "open" | "curious" => mod_val -= 0.02,
                "bitter" | "shrewd" => mod_val += 0.04,
                "reckless" => mod_val -= 0.03,
                _ => {}
            }
        }
        mod_val
    }

    pub fn encounter_modifier(personality: &[String]) -> EncounterMod {
        let mut mod_val = EncounterMod::default();
        for trait_val in personality {
            match trait_val.as_str() {
                "hospitable" | "warm" | "generous" => mod_val.talk += 0.05,
                "xenophobic" | "suspicious" | "insular" => mod_val.flee += 0.08,
                "cautious" | "guarded" => mod_val.flee += 0.03,
                "open" | "curious" => mod_val.talk += 0.03,
                "mercenary" => mod_val.bribe_cost -= 0.10,
                "devout" => mod_val.calm += 0.05,
                "reckless" => mod_val.push_through += 0.05,
                "loyal" => mod_val.calm += 0.03,
                "bitter" => mod_val.intimidate += 0.04,
                "stoic" => mod_val.calm += 0.04,
                "withdrawn" => mod_val.talk -= 0.03,
                "shrewd" => mod_val.trade += 0.04,
                _ => {}
            }
        }
        mod_val
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncounterMod {
    pub talk: f64,
    pub flee: f64,
    pub calm: f64,
    pub push_through: f64,
    pub bribe_cost: f64,
    pub intimidate: f64,
    pub trade: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollapseOutcome {
    BeastNest,
    BeastGuarded,
    HostileBeast,
    StrangerHut,
    SettlementBed,
    WaysideShrine,
    Riverbank,
    FestivalBench,
    Ditch,
    GodCampsite,
}

impl CollapseOutcome {
    pub fn description(self) -> &'static str {
        match self {
            CollapseOutcome::BeastNest => "You wake tangled in foul straw, a beast's breath still warm beside you. It fled at your stirring.",
            CollapseOutcome::BeastGuarded => "You wake warm. A great beast lies beside you — it watched over you through the night. It rises, snorts, and vanishes into the trees. The forest remembers kindness.",
            CollapseOutcome::HostileBeast => "Pain. Something was chewing on you. You thrash and it bolts, but the wounds are real. The wild has no love for you.",
            CollapseOutcome::StrangerHut => "You wake on a rough pallet. A stranger props broth by your head. They shake their head at you — wordless, disappointed — and wave you out.",
            CollapseOutcome::SettlementBed => "A proper bed. Clean sheets. Someone carried you here — the townsfolk whisper outside the door. Your name means something here.",
            CollapseOutcome::WaysideShrine => "Cold stone under your back. A shrine's shadow. Someone left an offering of bread — you took it. The shrine feels... watchful.",
            CollapseOutcome::Riverbank => "Water on your face. You're lying on smooth stones by a river, soaked but alive. The current carried you somewhere gentler. A traveler's cairn marks the spot.",
            CollapseOutcome::FestivalBench => "Laughter. Firelight. You're slumped on a bench at some festival — someone draped a blanket over you. A child pokes your arm offering bread. The hearth-fires burn bright here.",
            CollapseOutcome::Ditch => "Mud. Cold mud. You're in a ditch. Something crawled over you in the night. Your coin pouch feels lighter.",
            CollapseOutcome::GodCampsite => "You wake by a fire that burns without smoke. Five figures sit around it, speaking in a language you almost understand. One glances at you, amused. 'Sleep well, little one?' They are gone by morning. You feel... changed.",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            CollapseOutcome::BeastNest => '🐺',
            CollapseOutcome::BeastGuarded => '🦌',
            CollapseOutcome::HostileBeast => '🩸',
            CollapseOutcome::StrangerHut => '🏠',
            CollapseOutcome::SettlementBed => '🛏',
            CollapseOutcome::WaysideShrine => '⛩',
            CollapseOutcome::Riverbank => '🌊',
            CollapseOutcome::FestivalBench => '🎉',
            CollapseOutcome::Ditch => '🕳',
            CollapseOutcome::GodCampsite => '✦',
        }
    }

    pub fn is_divine(self) -> bool {
        matches!(self, CollapseOutcome::GodCampsite)
    }

    pub fn is_beast_aided(self) -> bool {
        matches!(self, CollapseOutcome::BeastGuarded)
    }

    pub fn is_hostile(self) -> bool {
        matches!(self, CollapseOutcome::HostileBeast | CollapseOutcome::Ditch)
    }

    pub fn is_safe(self) -> bool {
        matches!(
            self,
            CollapseOutcome::SettlementBed
                | CollapseOutcome::FestivalBench
                | CollapseOutcome::BeastGuarded
                | CollapseOutcome::GodCampsite
        )
    }

    pub fn coin_loss(self) -> u32 {
        match self {
            CollapseOutcome::Ditch => 3,
            CollapseOutcome::HostileBeast => 4,
            CollapseOutcome::BeastNest => 1,
            CollapseOutcome::Riverbank => 1,
            _ => 0,
        }
    }

    pub fn item_loss(self) -> Option<ItemType> {
        match self {
            CollapseOutcome::HostileBeast => Some(ItemType::Food),
            CollapseOutcome::Ditch => Some(ItemType::Coin),
            _ => None,
        }
    }

    pub fn hunger_restore(self) -> f64 {
        match self {
            CollapseOutcome::SettlementBed => 0.6,
            CollapseOutcome::FestivalBench => 0.7,
            CollapseOutcome::StrangerHut => 0.5,
            CollapseOutcome::WaysideShrine => 0.3,
            CollapseOutcome::GodCampsite => 0.8,
            CollapseOutcome::BeastGuarded => 0.4,
            CollapseOutcome::Riverbank => 0.3,
            CollapseOutcome::HostileBeast => 0.15,
            CollapseOutcome::BeastNest => 0.2,
            CollapseOutcome::Ditch => 0.1,
        }
    }

    pub fn energy_restore(self) -> f64 {
        match self {
            CollapseOutcome::SettlementBed => 0.6,
            CollapseOutcome::FestivalBench => 0.5,
            CollapseOutcome::StrangerHut => 0.4,
            CollapseOutcome::WaysideShrine => 0.3,
            CollapseOutcome::GodCampsite => 1.0,
            CollapseOutcome::BeastGuarded => 0.5,
            CollapseOutcome::Riverbank => 0.3,
            CollapseOutcome::HostileBeast => 0.1,
            CollapseOutcome::BeastNest => 0.3,
            CollapseOutcome::Ditch => 0.2,
        }
    }

    pub fn hours_passed(self) -> u32 {
        match self {
            CollapseOutcome::SettlementBed => 14,
            CollapseOutcome::FestivalBench => 10,
            CollapseOutcome::GodCampsite => 16,
            CollapseOutcome::BeastGuarded => 12,
            CollapseOutcome::StrangerHut => 12,
            CollapseOutcome::WaysideShrine => 8,
            CollapseOutcome::Riverbank => 8,
            CollapseOutcome::HostileBeast => 6,
            CollapseOutcome::BeastNest => 10,
            CollapseOutcome::Ditch => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collapse {
    pub outcome: CollapseOutcome,
    pub died: bool,
    pub rescued_by: Option<GodName>,
}

impl Collapse {
    pub fn roll(seed: u64, affinity: &GodAffinity, local_rep: f64) -> Self {
        let hash = seed.wrapping_mul(2654435761) ^ 0xDEAD;
        let val = (hash % 1000) as u32;

        let died = val < 12;

        let mut weights: [(CollapseOutcome, u32); 10] = [
            (CollapseOutcome::GodCampsite, 3),
            (CollapseOutcome::BeastGuarded, 5),
            (CollapseOutcome::HostileBeast, 30),
            (CollapseOutcome::BeastNest, 120),
            (CollapseOutcome::StrangerHut, 100),
            (CollapseOutcome::SettlementBed, 10),
            (CollapseOutcome::WaysideShrine, 50),
            (CollapseOutcome::Riverbank, 30),
            (CollapseOutcome::FestivalBench, 5),
            (CollapseOutcome::Ditch, 150),
        ];

        if let Some(ally) = affinity.strongest_ally() {
            let bonus = (affinity.get(ally) * 80.0) as u32;
            match ally {
                GodName::Oltzed => {
                    weights[0].1 += bonus / 3;
                    weights[4].1 += bonus / 2;
                    weights[5].1 += bonus;
                    weights[8].1 += bonus / 2;
                    weights[9].1 = weights[9].1.saturating_sub(bonus / 2);
                }
                GodName::Keuru => {
                    weights[0].1 += bonus / 3;
                    weights[1].1 += bonus;
                    weights[2].1 = weights[2].1.saturating_sub(bonus);
                    weights[3].1 = weights[3].1.saturating_sub(bonus / 2);
                    weights[6].1 += bonus / 4;
                }
                GodName::Sampsa => {
                    weights[0].1 += bonus / 3;
                    weights[4].1 += bonus / 2;
                    weights[6].1 += bonus;
                    weights[2].1 = weights[2].1.saturating_sub(bonus / 3);
                }
                GodName::Masa => {
                    weights[0].1 += bonus / 3;
                    weights[7].1 += bonus;
                    weights[5].1 += bonus / 4;
                    weights[4].1 += bonus / 4;
                }
                GodName::Kukri => {
                    weights[0].1 += bonus / 3;
                    weights[6].1 += bonus / 2;
                    weights[7].1 += bonus;
                    weights[9].1 = weights[9].1.saturating_sub(bonus);
                }
            }
        }

        if let Some(grudge) = affinity.strongest_grudge() {
            let penalty = (affinity.get(grudge).abs() * 60.0) as u32;
            match grudge {
                GodName::Oltzed => {
                    weights[9].1 += penalty;
                    weights[5].1 = weights[5].1.saturating_sub(penalty / 2);
                    weights[8].1 = weights[8].1.saturating_sub(penalty / 3);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Keuru => {
                    weights[2].1 += penalty;
                    weights[3].1 += penalty / 2;
                    weights[9].1 += penalty / 3;
                    weights[1].1 = weights[1].1.saturating_sub(penalty);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Sampsa => {
                    weights[9].1 += penalty / 2;
                    weights[4].1 = weights[4].1.saturating_sub(penalty / 2);
                    weights[6].1 = weights[6].1.saturating_sub(penalty / 3);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Masa => {
                    weights[9].1 += penalty / 2;
                    weights[7].1 = weights[7].1.saturating_sub(penalty / 2);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Kukri => {
                    weights[2].1 += penalty;
                    weights[9].1 += penalty / 2;
                    weights[6].1 = weights[6].1.saturating_sub(penalty / 2);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
            }
        }

        if local_rep >= 0.7 {
            weights[5].1 += 60;
            weights[8].1 += 30;
            weights[9].1 = weights[9].1.saturating_sub(40);
        } else if local_rep >= 0.4 {
            weights[4].1 += 30;
            weights[5].1 += 20;
        } else if local_rep <= 0.15 {
            weights[9].1 += 40;
            weights[4].1 = weights[4].1.saturating_sub(20);
        }

        if affinity.get(GodName::Keuru) > 0.5 {
            weights[1].1 += 60;
            weights[2].1 = weights[2].1.saturating_sub(30);
        }

        if affinity.get(GodName::Masa) > 0.4 {
            weights[7].1 += 40;
        }

        let total: u32 = weights.iter().map(|(_, w)| *w).sum();
        let pick = if total > 0 { val % total } else { val % 1000 };
        let mut acc = 0u32;
        let mut chosen = CollapseOutcome::Ditch;
        for (outcome, weight) in &weights {
            acc += weight;
            if pick < acc {
                chosen = *outcome;
                break;
            }
        }

        let rescued_by = if chosen == CollapseOutcome::GodCampsite {
            affinity.strongest_ally()
        } else if chosen == CollapseOutcome::BeastGuarded {
            Some(GodName::Keuru)
        } else if chosen == CollapseOutcome::Riverbank {
            Some(GodName::Masa)
        } else if chosen == CollapseOutcome::FestivalBench
            || chosen == CollapseOutcome::SettlementBed
        {
            Some(GodName::Oltzed)
        } else if chosen == CollapseOutcome::WaysideShrine {
            Some(GodName::Kukri)
        } else {
            None
        };

        Collapse {
            outcome: chosen,
            died,
            rescued_by,
        }
    }

    pub fn roll_biased(
        seed: u64,
        affinity: &GodAffinity,
        local_rep: f64,
        effective_bias: f64,
    ) -> Self {
        let mut result = Self::roll(seed, affinity, local_rep);
        if effective_bias < -0.15 {
            result.outcome = match result.outcome {
                CollapseOutcome::StrangerHut => CollapseOutcome::Ditch,
                CollapseOutcome::SettlementBed => CollapseOutcome::Ditch,
                CollapseOutcome::FestivalBench => CollapseOutcome::Ditch,
                other => other,
            };
        } else if effective_bias > 0.05 {
            result.outcome = match result.outcome {
                CollapseOutcome::Ditch => CollapseOutcome::StrangerHut,
                other => other,
            };
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterKind {
    Wildlife,
    Bandit,
    Traveler,
    Storm,
}

impl EncounterKind {
    pub fn description(self) -> &'static str {
        match self {
            EncounterKind::Wildlife => "A wild creature blocks your path!",
            EncounterKind::Bandit => "A bandit demands your coin!",
            EncounterKind::Traveler => "A friendly traveler shares news.",
            EncounterKind::Storm => "A sudden storm forces you to take shelter!",
        }
    }

    pub fn is_hostile(self) -> bool {
        matches!(self, EncounterKind::Wildlife | EncounterKind::Bandit)
    }

    pub fn available_actions(self) -> Vec<EncounterAction> {
        match self {
            EncounterKind::Wildlife => vec![EncounterAction::Flee, EncounterAction::Calm],
            EncounterKind::Bandit => vec![
                EncounterAction::Flee,
                EncounterAction::Bribe,
                EncounterAction::Intimidate,
            ],
            EncounterKind::Traveler => vec![EncounterAction::Talk, EncounterAction::Trade],
            EncounterKind::Storm => vec![EncounterAction::Shelter, EncounterAction::PushThrough],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterAction {
    Flee,
    Bribe,
    Calm,
    Intimidate,
    Talk,
    Trade,
    Shelter,
    PushThrough,
}

impl EncounterAction {
    pub fn label(self) -> &'static str {
        match self {
            EncounterAction::Flee => "Flee",
            EncounterAction::Bribe => "Bribe (2 coins)",
            EncounterAction::Calm => "Calm the beast",
            EncounterAction::Intimidate => "Intimidate",
            EncounterAction::Talk => "Talk",
            EncounterAction::Trade => "Trade info",
            EncounterAction::Shelter => "Take shelter (1h)",
            EncounterAction::PushThrough => "Push through",
        }
    }

    pub fn key(self) -> char {
        match self {
            EncounterAction::Flee => 'f',
            EncounterAction::Bribe => 'b',
            EncounterAction::Calm => 'c',
            EncounterAction::Intimidate => 'i',
            EncounterAction::Talk => 't',
            EncounterAction::Trade => 'r',
            EncounterAction::Shelter => 's',
            EncounterAction::PushThrough => 'p',
        }
    }

    pub fn coin_cost(self) -> u32 {
        match self {
            EncounterAction::Bribe => 2,
            _ => 0,
        }
    }

    pub fn energy_cost(self) -> f64 {
        match self {
            EncounterAction::Flee => 0.15,
            EncounterAction::PushThrough => 0.2,
            EncounterAction::Intimidate => 0.1,
            _ => 0.0,
        }
    }

    pub fn hunger_cost(self) -> f64 {
        match self {
            EncounterAction::PushThrough => 0.1,
            EncounterAction::Flee => 0.05,
            _ => 0.0,
        }
    }

    pub fn hours(self) -> u32 {
        match self {
            EncounterAction::Shelter => 1,
            EncounterAction::Talk => 1,
            EncounterAction::Trade => 1,
            EncounterAction::Calm => 1,
            _ => 0,
        }
    }

    pub fn god_affinity_effect(self) -> Option<(GodName, f64)> {
        match self {
            EncounterAction::Calm => Some((GodName::Keuru, 0.05)),
            EncounterAction::Intimidate => Some((GodName::Oltzed, -0.02)),
            EncounterAction::Talk => Some((GodName::Masa, 0.03)),
            EncounterAction::Trade => Some((GodName::Masa, 0.04)),
            EncounterAction::Bribe => Some((GodName::Masa, -0.01)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    pub kind: EncounterKind,
    pub terrain: Terrain,
}

impl Encounter {
    pub fn roll(terrain: Terrain, hour: u32, seed: u64) -> Option<Self> {
        Self::roll_biased(terrain, hour, seed, None)
    }

    pub fn roll_biased(
        terrain: Terrain,
        hour: u32,
        seed: u64,
        player_people: Option<PeopleKind>,
    ) -> Option<Self> {
        let hash = seed.wrapping_mul(2654435761)
            ^ (terrain as u64).wrapping_mul(40503)
            ^ (hour as u64).wrapping_mul(92000);
        let val = hash % 100;
        let (mut threshold, mut kind): (u32, EncounterKind) = match terrain {
            Terrain::Forest => (
                25,
                if val.is_multiple_of(2) {
                    EncounterKind::Wildlife
                } else {
                    EncounterKind::Bandit
                },
            ),
            Terrain::Mountain => (15, EncounterKind::Storm),
            Terrain::Swamp => (20, EncounterKind::Wildlife),
            Terrain::Sand => (10, EncounterKind::Storm),
            Terrain::DeepDesert => (12, EncounterKind::Storm),
            Terrain::Road => (5, EncounterKind::Traveler),
            Terrain::Settlement => (0, EncounterKind::Traveler),
            Terrain::Cave => (18, EncounterKind::Wildlife),
            Terrain::Tundra => (15, EncounterKind::Storm),
            Terrain::Coast => (8, EncounterKind::Traveler),
            _ => (8, EncounterKind::Wildlife),
        };
        if let Some(pp) = player_people {
            match (pp, terrain) {
                (PeopleKind::Metsik, Terrain::Forest) => {
                    threshold = threshold.saturating_sub(5);
                    kind = EncounterKind::Wildlife;
                }
                (PeopleKind::Vayla, Terrain::Road) => {
                    threshold = threshold.saturating_add(5);
                    kind = EncounterKind::Traveler;
                }
                (PeopleKind::Sepat, Terrain::Mountain) => {
                    threshold = threshold.saturating_sub(5);
                }
                (PeopleKind::Ahjo, Terrain::Grass | Terrain::Farmland) => {
                    threshold = threshold.saturating_sub(3);
                    kind = EncounterKind::Traveler;
                }
                (PeopleKind::Laakso, Terrain::Swamp) => {
                    threshold = threshold.saturating_sub(4);
                    kind = EncounterKind::Wildlife;
                }
                (PeopleKind::Tzakhar, Terrain::Cave) => {
                    threshold = threshold.saturating_sub(6);
                    kind = EncounterKind::Wildlife;
                }
                (PeopleKind::Merak, Terrain::Coast) => {
                    threshold = threshold.saturating_sub(4);
                    kind = EncounterKind::Traveler;
                }
                (PeopleKind::Khor, Terrain::Tundra) => {
                    threshold = threshold.saturating_sub(5);
                }
                (PeopleKind::Shear, Terrain::Sand | Terrain::DeepDesert) => {
                    threshold = threshold.saturating_sub(4);
                }
                (PeopleKind::Hal, Terrain::Forest) => {
                    threshold = threshold.saturating_sub(4);
                    kind = EncounterKind::Wildlife;
                }
                _ => {}
            }
        }
        if (val % 100) < (threshold as u64) {
            Some(Encounter { kind, terrain })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterLogEntry {
    pub day: u32,
    pub hour: u32,
    pub kind: EncounterKind,
    pub terrain: Terrain,
    pub action: EncounterAction,
    pub hostile: bool,
}

const ENCOUNTER_LOG_CAP: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EncounterLog {
    entries: Vec<EncounterLogEntry>,
}

impl EncounterLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, entry: EncounterLogEntry) {
        self.entries.push(entry);
        if self.entries.len() > ENCOUNTER_LOG_CAP {
            let drop = self.entries.len() - ENCOUNTER_LOG_CAP;
            self.entries.drain(0..drop);
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, EncounterLogEntry> {
        self.entries.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct World {
    pub seed: u64,
    pub tick: u64,
    pub regions: Vec<Region>,
    pub charts_version: String,
    #[serde(default)]
    pub region_cols: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RegionNeighbors {
    pub north: Option<usize>,
    pub south: Option<usize>,
    pub east: Option<usize>,
    pub west: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub region_type: String,
    #[serde(default)]
    pub region_subtype: String,
    pub description: String,
    pub settlements: Vec<Settlement>,
    #[serde(default)]
    pub terrain: TerrainMap,
    #[serde(default)]
    pub neighbors: RegionNeighbors,
}

impl Region {
    pub fn danger_level(&self) -> DangerLevel {
        let total = self.terrain.width * self.terrain.height;
        if total == 0 {
            return DangerLevel::Safe;
        }
        let mut hostile = 0u32;
        for y in 0..self.terrain.height {
            for x in 0..self.terrain.width {
                if let Some(
                    Terrain::Forest
                    | Terrain::Mountain
                    | Terrain::Swamp
                    | Terrain::Cave
                    | Terrain::DeepDesert,
                ) = self.terrain.get(x, y)
                {
                    hostile += 1;
                }
            }
        }
        let ratio = hostile as f64 / total as f64;
        if ratio > 0.5 {
            DangerLevel::Dangerous
        } else if ratio > 0.25 {
            DangerLevel::Risky
        } else {
            DangerLevel::Safe
        }
    }

    pub fn danger_level_biased(&self, player_people: PeopleKind) -> DangerLevel {
        let base = self.danger_level();
        let dominant = self.settlements.first().and_then(|s| s.people.first());
        let bias = dominant.map_or(0.0, |p| {
            player_people.bias_toward(PeopleKind::from_name(&p.people))
        });
        match base {
            DangerLevel::Safe => {
                if bias < -0.15 {
                    DangerLevel::Risky
                } else {
                    DangerLevel::Safe
                }
            }
            DangerLevel::Risky => {
                if bias < -0.15 {
                    DangerLevel::Dangerous
                } else if bias > 0.05 {
                    DangerLevel::Safe
                } else {
                    DangerLevel::Risky
                }
            }
            DangerLevel::Dangerous => {
                if bias > 0.05 {
                    DangerLevel::Risky
                } else {
                    DangerLevel::Dangerous
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerLevel {
    Safe,
    Risky,
    Dangerous,
}

impl DangerLevel {
    pub fn glyph(self) -> char {
        match self {
            DangerLevel::Safe => '·',
            DangerLevel::Risky => '⚠',
            DangerLevel::Dangerous => '☠',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementService {
    Tavern,
    Temple,
    Forge,
    Hearth,
    TrapWorkshop,
    Archive,
    TradePost,
    Shrine,
}

impl SettlementService {
    pub fn glyph(self) -> char {
        match self {
            SettlementService::Tavern => '🍺',
            SettlementService::Temple => '⛪',
            SettlementService::Forge => '⚒',
            SettlementService::Hearth => '🏠',
            SettlementService::TrapWorkshop => '🪤',
            SettlementService::Archive => '📜',
            SettlementService::TradePost => '🏪',
            SettlementService::Shrine => '🕯',
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SettlementService::Tavern => "Tavern",
            SettlementService::Temple => "Temple",
            SettlementService::Forge => "Forge",
            SettlementService::Hearth => "Hearth",
            SettlementService::TrapWorkshop => "Trap Workshop",
            SettlementService::Archive => "Archive",
            SettlementService::TradePost => "Trade Post",
            SettlementService::Shrine => "Shrine",
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            SettlementService::Tavern => 2,
            SettlementService::Temple => 3,
            SettlementService::Forge => 3,
            SettlementService::Hearth => 2,
            SettlementService::TrapWorkshop => 2,
            SettlementService::Archive => 3,
            SettlementService::TradePost => 2,
            SettlementService::Shrine => 2,
        }
    }

    pub fn people(self) -> Option<PeopleKind> {
        match self {
            SettlementService::Forge => Some(PeopleKind::Sepat),
            SettlementService::Hearth => Some(PeopleKind::Ahjo),
            SettlementService::TrapWorkshop => Some(PeopleKind::Metsik),
            SettlementService::Archive => Some(PeopleKind::Arkit),
            SettlementService::TradePost => Some(PeopleKind::Vayla),
            SettlementService::Shrine => Some(PeopleKind::Laakso),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settlement {
    pub id: String,
    pub name: String,
    pub size: String,
    pub region: String,
    pub population: u32,
    pub description: String,
    pub people: Vec<Person>,
    #[serde(default)]
    pub services: Vec<SettlementService>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Need {
    Food,
    Money,
    Care,
    Presence,
    Safety,
}

impl fmt::Display for Need {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Need::Food => write!(f, "Food"),
            Need::Money => write!(f, "Money"),
            Need::Care => write!(f, "Care"),
            Need::Presence => write!(f, "Presence"),
            Need::Safety => write!(f, "Safety"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Needs {
    pub values: HashMap<Need, f64>,
}

impl Default for Needs {
    fn default() -> Self {
        let mut values = HashMap::new();
        values.insert(Need::Food, 0.8);
        values.insert(Need::Money, 0.8);
        values.insert(Need::Care, 0.8);
        values.insert(Need::Presence, 0.8);
        values.insert(Need::Safety, 0.8);
        Needs { values }
    }
}

impl Needs {
    pub fn get(&self, need: Need) -> f64 {
        self.values.get(&need).copied().unwrap_or(0.0)
    }

    pub fn satisfy(&mut self, need: Need, amount: f64) {
        let current = self.get(need);
        self.values.insert(need, (current + amount).clamp(0.0, 1.0));
    }

    pub fn decay(&mut self, need: Need, rate: f64) {
        let current = self.get(need);
        self.values.insert(need, (current - rate).clamp(0.0, 1.0));
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftAffinity {
    #[default]
    None,
    Word,
    Current,
    Still,
    Forge,
    Root,
}

impl CraftAffinity {
    pub fn from_chart_key(key: &str) -> Option<Self> {
        match key {
            "none" => Some(Self::None),
            "word" => Some(Self::Word),
            "current" => Some(Self::Current),
            "still" => Some(Self::Still),
            "forge" => Some(Self::Forge),
            "root" => Some(Self::Root),
            _ => None,
        }
    }

    pub fn to_chart_key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Word => "word",
            Self::Current => "current",
            Self::Still => "still",
            Self::Forge => "forge",
            Self::Root => "root",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NpcActivity {
    Sleep,
    Work,
    Socialize,
    Travel,
    Worship,
    Craft,
    Idle,
}

impl NpcActivity {
    pub fn name(self) -> &'static str {
        match self {
            NpcActivity::Sleep => "sleeping",
            NpcActivity::Work => "working",
            NpcActivity::Socialize => "socializing",
            NpcActivity::Travel => "traveling",
            NpcActivity::Worship => "worshipping",
            NpcActivity::Craft => "crafting",
            NpcActivity::Idle => "idle",
        }
    }

    pub fn is_available(self) -> bool {
        !matches!(self, NpcActivity::Sleep | NpcActivity::Travel)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcSchedule {
    pub blocks: [NpcActivity; 6],
}

impl Default for NpcSchedule {
    fn default() -> Self {
        NpcSchedule {
            blocks: [
                NpcActivity::Sleep,
                NpcActivity::Work,
                NpcActivity::Work,
                NpcActivity::Socialize,
                NpcActivity::Idle,
                NpcActivity::Sleep,
            ],
        }
    }
}

impl NpcSchedule {
    pub fn activity_at_hour(&self, hour: u32) -> NpcActivity {
        let block_idx = (hour / 4) as usize;
        self.blocks[block_idx.min(5)]
    }

    pub fn is_available_at_hour(&self, hour: u32) -> bool {
        self.activity_at_hour(hour).is_available()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombatAction {
    Attack,
    Parry,
    Feint,
    Yield,
}

impl CombatAction {
    pub fn name(self) -> &'static str {
        match self {
            CombatAction::Attack => "attack",
            CombatAction::Parry => "parry",
            CombatAction::Feint => "feint",
            CombatAction::Yield => "yield",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombatOutcome {
    pub player_injury: f64,
    pub npc_injury: f64,
    pub reputation_delta: f64,
    pub player_died: bool,
    pub npc_yielded: bool,
    pub flavor: String,
}

impl CombatOutcome {
    pub fn resolve(
        player_action: CombatAction,
        npc_action: CombatAction,
        player_trust: f64,
        npc_aggression: f64,
        seed: u64,
    ) -> Self {
        let mut rng = crate::rng::SeedRng::new(seed);
        let roll = rng.gen_range(1000) as f64 / 1000.0;

        // Death is rare (~1.2%)
        let player_died = roll < 0.012;

        let (player_injury, npc_injury, reputation_delta, npc_yielded, flavor) =
            match (player_action, npc_action) {
                (CombatAction::Yield, _) => (
                    0.1,
                    0.0,
                    -0.05,
                    false,
                    "You yield gracefully. They let you go with a warning.",
                ),
                (_, CombatAction::Yield) => (
                    0.0,
                    0.05,
                    0.02,
                    true,
                    "They yield. The duel ends with respect.",
                ),
                (CombatAction::Attack, CombatAction::Attack) => {
                    if player_trust > 0.5 {
                        (
                            0.15,
                            0.25,
                            0.01,
                            false,
                            "Your strike lands true. They stagger back, wounded.",
                        )
                    } else {
                        (
                            0.25,
                            0.15,
                            -0.01,
                            false,
                            "They meet your blow. You both bleed, but they seem stronger.",
                        )
                    }
                }
                (CombatAction::Attack, CombatAction::Parry) => (
                    0.2,
                    0.05,
                    -0.02,
                    false,
                    "They parry your attack. You overextend and take a hit.",
                ),
                (CombatAction::Attack, CombatAction::Feint) => (
                    0.05,
                    0.3,
                    0.03,
                    false,
                    "You see through their feint. Your attack catches them off-guard.",
                ),
                (CombatAction::Parry, CombatAction::Attack) => (
                    0.05,
                    0.2,
                    0.02,
                    false,
                    "You parry their attack. They stumble, exposed.",
                ),
                (CombatAction::Parry, CombatAction::Parry) => (
                    0.0,
                    0.0,
                    0.0,
                    false,
                    "You circle each other, blades raised. Neither commits.",
                ),
                (CombatAction::Parry, CombatAction::Feint) => (
                    0.1,
                    0.0,
                    -0.01,
                    false,
                    "Their feint draws your parry. They strike, but you deflect most of it.",
                ),
                (CombatAction::Feint, CombatAction::Attack) => (
                    0.3,
                    0.05,
                    -0.03,
                    false,
                    "Your feint fails. They punish your opening.",
                ),
                (CombatAction::Feint, CombatAction::Parry) => (
                    0.0,
                    0.1,
                    0.01,
                    false,
                    "Your feint draws their parry. You find an opening.",
                ),
                (CombatAction::Feint, CombatAction::Feint) => (
                    0.0,
                    0.0,
                    0.0,
                    false,
                    "Both feint. Neither commits. The dance continues.",
                ),
            };

        // Apply aggression modifier to injuries
        let player_injury = (player_injury * (1.0 + npc_aggression * 0.3)).min(0.5);
        let npc_injury = (npc_injury * (1.0 - npc_aggression * 0.2)).min(0.5);

        CombatOutcome {
            player_injury,
            npc_injury,
            reputation_delta,
            player_died,
            npc_yielded,
            flavor: flavor.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub social_class: String,
    pub craft_affinity: String,
    pub personality: Vec<String>,
    pub bias: String,
    pub needs: Needs,
    pub region: String,
    pub settlement: String,
    pub has_spouse: bool,
    pub children_count: u32,
    pub has_debt: bool,
    #[serde(default)]
    pub schedule: NpcSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub social_class: String,
    pub craft_affinity: CraftAffinity,
    pub personality: Vec<String>,
    pub region: String,
    pub settlement: String,
    pub perks: Vec<Perk>,
    pub household_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Perk {
    pub name: String,
    pub description: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerStart {
    pub person: Person,
    pub reroll_count: u32,
    pub point_buy_adjustments: Vec<Adjustment>,
    pub accepted: bool,
    #[serde(default)]
    pub inventory: Inventory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Adjustment {
    SwapProfession(String),
    SetCraft(CraftAffinity),
    AddPerk(Perk),
    CutHouseholdTie,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Debt {
    pub creditor_id: String,
    pub amount: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Household {
    pub id: String,
    pub head_id: String,
    pub spouse_id: Option<String>,
    pub children_ids: Vec<String>,
    pub location_settlement_id: String,
    pub has_debt: bool,
    pub debts: Vec<Debt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipKind {
    Spouse,
    Parent,
    Child,
    Sibling,
    Kin,
    Friend,
    Rival,
    Patron,
    Apprentice,
    Guildmate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipEvent {
    pub tick: u64,
    pub description: String,
}

fn default_trust_baseline() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    pub from_id: String,
    pub to_id: String,
    pub kind: RelationshipKind,
    pub strength: f64,
    pub trust: f64,
    #[serde(default = "default_trust_baseline")]
    pub trust_baseline: f64,
    pub history: Vec<RelationshipEvent>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Animal {
    Dog,
    Horse,
    Ox,
    Falcon,
    Goat,
}

impl Animal {
    pub fn name(self) -> &'static str {
        match self {
            Animal::Dog => "dog",
            Animal::Horse => "horse",
            Animal::Ox => "ox",
            Animal::Falcon => "falcon",
            Animal::Goat => "goat",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Animal::Dog => "loyal guardian and keen-nosed gatherer",
            Animal::Horse => "swift mount for distant roads",
            Animal::Ox => "strong back for heavy loads",
            Animal::Falcon => "sharp-eyed scout from the sky",
            Animal::Goat => "patient provider of milk",
        }
    }

    pub fn gathering_bonus(self) -> f64 {
        match self {
            Animal::Dog => 0.15,
            _ => 0.0,
        }
    }

    pub fn travel_speed_multiplier(self) -> f64 {
        match self {
            Animal::Horse => 0.7,
            _ => 1.0,
        }
    }

    pub fn carry_capacity_bonus(self) -> u32 {
        match self {
            Animal::Ox => 10,
            _ => 0,
        }
    }

    pub fn scouting_bonus(self) -> f64 {
        match self {
            Animal::Falcon => 0.2,
            _ => 0.0,
        }
    }

    pub fn milk_production(self) -> u32 {
        match self {
            Animal::Goat => 1,
            _ => 0,
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            Animal::Dog => 8,
            Animal::Horse => 25,
            Animal::Ox => 15,
            Animal::Falcon => 12,
            Animal::Goat => 6,
        }
    }

    pub fn food_per_tick(self) -> u32 {
        match self {
            Animal::Dog => 1,
            Animal::Horse => 2,
            Animal::Ox => 2,
            Animal::Falcon => 1,
            Animal::Goat => 1,
        }
    }

    pub fn rest_per_tick(self) -> u32 {
        match self {
            Animal::Dog => 1,
            Animal::Horse => 2,
            Animal::Ox => 2,
            Animal::Falcon => 1,
            Animal::Goat => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Companion {
    pub animal: Animal,
    pub name: String,
    pub food_need: f64,
    pub rest_need: f64,
    pub acquired_tick: u64,
    pub loyalty: f64,
}

impl Companion {
    pub fn new(animal: Animal, name: String, acquired_tick: u64) -> Self {
        Companion {
            animal,
            name,
            food_need: 0.0,
            rest_need: 0.0,
            acquired_tick,
            loyalty: 0.5,
        }
    }

    pub fn decay_needs(&mut self, ticks: u64) {
        self.food_need = (self.food_need + ticks as f64 * 0.5).min(100.0);
        self.rest_need = (self.rest_need + ticks as f64 * 0.3).min(100.0);
    }

    pub fn feed(&mut self, amount: f64) {
        self.food_need = (self.food_need - amount * 20.0).max(0.0);
    }

    pub fn rest(&mut self, amount: f64) {
        self.rest_need = (self.rest_need - amount * 25.0).max(0.0);
    }

    pub fn is_starving(&self) -> bool {
        self.food_need >= 80.0
    }

    pub fn is_exhausted(&self) -> bool {
        self.rest_need >= 80.0
    }

    pub fn is_alive(&self) -> bool {
        !(self.food_need >= 100.0 || self.rest_need >= 100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Caravan {
    pub id: String,
    pub origin: String,
    pub destination: String,
    pub goods: Vec<(ItemType, u32)>,
    pub departure_tick: u64,
    pub arrival_tick: u64,
    pub travel_cost: u32,
}

impl Caravan {
    pub fn generate(seed: u64, origin: String, destination: String, departure_tick: u64) -> Self {
        let mut rng = crate::rng::SeedRng::new(seed);
        let num_goods = 1 + rng.gen_range(4) as usize;
        let mut goods = Vec::new();
        let tradeable = ItemType::tradeable_items();

        for _ in 0..num_goods {
            let item = tradeable[rng.gen_range(tradeable.len() as u32) as usize];
            let quantity = 2 + rng.gen_range(6);
            goods.push((item, quantity));
        }

        let base_travel_time = 24 + rng.gen_range(48); // 1-3 days
        let arrival_tick = departure_tick + base_travel_time as u64;
        let travel_cost = 3 + rng.gen_range(5);

        Caravan {
            id: format!("caravan-{:016x}", seed),
            origin,
            destination,
            goods,
            departure_tick,
            arrival_tick,
            travel_cost,
        }
    }

    pub fn is_in_transit(&self, current_tick: u64) -> bool {
        current_tick >= self.departure_tick && current_tick < self.arrival_tick
    }

    pub fn has_arrived(&self, current_tick: u64) -> bool {
        current_tick >= self.arrival_tick
    }

    pub fn price_modifier(&self, item: ItemType, current_tick: u64) -> f64 {
        if !self.is_in_transit(current_tick) && !self.has_arrived(current_tick) {
            return 1.0;
        }

        let quantity: u32 = self
            .goods
            .iter()
            .filter(|(i, _)| *i == item)
            .map(|(_, q)| q)
            .sum();

        if quantity == 0 {
            return 1.0;
        }

        // More goods = lower price (supply increase)
        let modifier = 1.0 - (quantity as f64 * 0.05);
        modifier.clamp(0.7, 1.3)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Disease {
    Fever,
    Infection,
    Sprain,
    Exhaustion,
    Plague,
}

impl Disease {
    pub fn name(self) -> &'static str {
        match self {
            Disease::Fever => "fever",
            Disease::Infection => "infection",
            Disease::Sprain => "sprain",
            Disease::Exhaustion => "exhaustion",
            Disease::Plague => "plague",
        }
    }

    pub fn vitals_decay_modifier(self) -> f64 {
        match self {
            Disease::Fever => 1.3,
            Disease::Infection => 1.4,
            Disease::Sprain => 1.2,
            Disease::Exhaustion => 1.5,
            Disease::Plague => 1.8,
        }
    }

    pub fn recovery_ticks(self) -> u64 {
        match self {
            Disease::Fever => 48,
            Disease::Infection => 72,
            Disease::Sprain => 36,
            Disease::Exhaustion => 24,
            Disease::Plague => 120,
        }
    }

    pub fn contraction_probability(self, terrain: Terrain) -> f64 {
        match (self, terrain) {
            (Disease::Fever, Terrain::Swamp | Terrain::Forest) => 0.02,
            (Disease::Infection, Terrain::Swamp) => 0.03,
            (Disease::Sprain, Terrain::Mountain | Terrain::Forest) => 0.015,
            (Disease::Exhaustion, _) => 0.01,
            (Disease::Plague, Terrain::Settlement) => 0.005,
            _ => 0.002,
        }
    }

    pub fn can_contract(seed: u64, tick: u64, terrain: Terrain, disease: Disease) -> bool {
        let mut rng = crate::rng::SeedRng::new(seed.wrapping_add(tick));
        let roll = rng.gen_range(1000) as f64 / 1000.0;
        roll < disease.contraction_probability(terrain)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveDisease {
    pub disease: Disease,
    pub contracted_tick: u64,
    pub severity: f64,
}

impl ActiveDisease {
    pub fn new(disease: Disease, contracted_tick: u64) -> Self {
        ActiveDisease {
            disease,
            contracted_tick,
            severity: 1.0,
        }
    }

    pub fn is_recovered(&self, current_tick: u64) -> bool {
        current_tick >= self.contracted_tick + self.disease.recovery_ticks()
    }

    pub fn vitals_modifier(&self) -> f64 {
        self.disease.vitals_decay_modifier()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Shelter,
    Workshop,
    Shrine,
    Hearth,
    Trap,
}

impl BuildingType {
    pub fn name(self) -> &'static str {
        match self {
            BuildingType::Shelter => "shelter",
            BuildingType::Workshop => "workshop",
            BuildingType::Shrine => "shrine",
            BuildingType::Hearth => "hearth",
            BuildingType::Trap => "trap",
        }
    }

    pub fn materials_required(self) -> Vec<(ItemType, u32)> {
        match self {
            BuildingType::Shelter => vec![(ItemType::Wood, 5), (ItemType::Cloth, 2)],
            BuildingType::Workshop => vec![(ItemType::Wood, 8), (ItemType::Iron, 3)],
            BuildingType::Shrine => vec![(ItemType::Stone, 6), (ItemType::Cloth, 3)],
            BuildingType::Hearth => vec![(ItemType::Stone, 4), (ItemType::Wood, 2)],
            BuildingType::Trap => vec![(ItemType::Wood, 3), (ItemType::Iron, 1)],
        }
    }

    pub fn build_ticks(self) -> u64 {
        match self {
            BuildingType::Shelter => 48,
            BuildingType::Workshop => 72,
            BuildingType::Shrine => 96,
            BuildingType::Hearth => 36,
            BuildingType::Trap => 24,
        }
    }

    pub fn energy_cost(self) -> f64 {
        match self {
            BuildingType::Shelter => 0.3,
            BuildingType::Workshop => 0.4,
            BuildingType::Shrine => 0.5,
            BuildingType::Hearth => 0.2,
            BuildingType::Trap => 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Building {
    pub id: String,
    pub building_type: BuildingType,
    pub build_progress: f64,
    pub completed: bool,
    pub location: String,
    pub built_tick: Option<u64>,
}

impl Building {
    pub fn new(seed: u64, building_type: BuildingType, location: String) -> Self {
        Building {
            id: format!("building-{:016x}", seed),
            building_type,
            build_progress: 0.0,
            completed: false,
            location,
            built_tick: None,
        }
    }

    pub fn advance_construction(&mut self, ticks: u64, current_tick: u64) {
        if self.completed {
            return;
        }
        let total_ticks = self.building_type.build_ticks();
        let progress_per_tick = 1.0 / total_ticks as f64;
        self.build_progress = (self.build_progress + ticks as f64 * progress_per_tick).min(1.0);

        if self.build_progress >= 1.0 {
            self.completed = true;
            self.built_tick = Some(current_tick);
        }
    }

    pub fn is_complete(&self) -> bool {
        self.completed
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CropType {
    Grain,
    RootVegetable,
    Herb,
}

impl CropType {
    pub fn name(self) -> &'static str {
        match self {
            CropType::Grain => "grain",
            CropType::RootVegetable => "root vegetables",
            CropType::Herb => "herbs",
        }
    }

    pub fn growth_ticks(self) -> u64 {
        match self {
            CropType::Grain => 72,         // 3 days
            CropType::RootVegetable => 96, // 4 days
            CropType::Herb => 48,          // 2 days
        }
    }

    pub fn base_yield(self) -> u32 {
        match self {
            CropType::Grain => 4,
            CropType::RootVegetable => 3,
            CropType::Herb => 5,
        }
    }

    pub fn regional_suitability(self, terrain: Terrain) -> f64 {
        match (self, terrain) {
            (CropType::Grain, Terrain::Farmland | Terrain::Grass) => 1.2,
            (CropType::RootVegetable, Terrain::Forest | Terrain::Farmland) => 1.1,
            (CropType::Herb, Terrain::Forest | Terrain::Swamp) => 1.3,
            (_, Terrain::Farmland) => 1.0,
            _ => 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrowthStage {
    Planted,
    Sprouting,
    Growing,
    Mature,
    Ready,
}

impl GrowthStage {
    pub fn name(self) -> &'static str {
        match self {
            GrowthStage::Planted => "planted",
            GrowthStage::Sprouting => "sprouting",
            GrowthStage::Growing => "growing",
            GrowthStage::Mature => "mature",
            GrowthStage::Ready => "ready to harvest",
        }
    }

    pub fn progress_threshold(self) -> f64 {
        match self {
            GrowthStage::Planted => 0.0,
            GrowthStage::Sprouting => 0.2,
            GrowthStage::Growing => 0.5,
            GrowthStage::Mature => 0.8,
            GrowthStage::Ready => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Farm {
    pub id: String,
    pub crop: CropType,
    pub planted_tick: u64,
    pub growth_progress: f64,
    pub stage: GrowthStage,
    pub terrain: Terrain,
    pub weather_bonus: f64,
}

impl Farm {
    pub fn new(seed: u64, crop: CropType, planted_tick: u64, terrain: Terrain) -> Self {
        Farm {
            id: format!("farm-{:016x}", seed),
            crop,
            planted_tick,
            growth_progress: 0.0,
            stage: GrowthStage::Planted,
            terrain,
            weather_bonus: 0.0,
        }
    }

    pub fn update_growth(&mut self, current_tick: u64, weather: Weather) {
        let ticks_elapsed = current_tick.saturating_sub(self.planted_tick);
        let base_growth_rate = 1.0 / self.crop.growth_ticks() as f64;
        let suitability = self.crop.regional_suitability(self.terrain);
        let weather_mod = weather.gather_modifier();

        self.weather_bonus = (weather_mod - 1.0) * 0.5;
        let effective_rate = base_growth_rate * suitability * (1.0 + self.weather_bonus);
        self.growth_progress = (ticks_elapsed as f64 * effective_rate).min(1.0);

        self.stage = if self.growth_progress >= 1.0 {
            GrowthStage::Ready
        } else if self.growth_progress >= 0.8 {
            GrowthStage::Mature
        } else if self.growth_progress >= 0.5 {
            GrowthStage::Growing
        } else if self.growth_progress >= 0.2 {
            GrowthStage::Sprouting
        } else {
            GrowthStage::Planted
        };
    }

    pub fn is_ready(&self) -> bool {
        self.stage == GrowthStage::Ready
    }

    pub fn harvest_yield(&self) -> u32 {
        if !self.is_ready() {
            return 0;
        }
        let base = self.crop.base_yield();
        let suitability = self.crop.regional_suitability(self.terrain);
        (base as f64 * suitability * (1.0 + self.weather_bonus)).ceil() as u32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quest {
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

impl Quest {
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
        let deadline_tick = current_tick + 48 + rng.gen_range(72) as u64; // 2-5 days

        Quest {
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
    fn roundtrip_person() {
        let person = Person {
            id: "abc-123".into(),
            name: "Testi".into(),
            people: "metsik".into(),
            sex: "f".into(),
            age_band: "adult".into(),
            profession: "farmer".into(),
            social_class: "low".into(),
            craft_affinity: "none".into(),
            personality: vec!["stoic".into(), "curious".into()],
            bias: "metsik".into(),
            needs: {
                let mut v = HashMap::new();
                v.insert(Need::Food, 0.8);
                v.insert(Need::Safety, 0.5);
                v.insert(Need::Care, 0.6);
                v.insert(Need::Money, 0.3);
                v.insert(Need::Presence, 0.7);
                Needs { values: v }
            },
            region: "river_valley".into(),
            settlement: "hamlet-1".into(),
            has_spouse: true,
            children_count: 2,
            has_debt: false,
            schedule: NpcSchedule::default(),
        };
        roundtrip(&person);
    }

    #[test]
    fn roundtrip_settlement() {
        let s = Settlement {
            id: "set-1".into(),
            name: "Test Village".into(),
            size: "village".into(),
            region: "river_valley".into(),
            population: 120,
            description: "A test village".into(),
            people: vec![],
            services: vec![],
        };
        roundtrip(&s);
    }

    #[test]
    fn roundtrip_region() {
        let r = Region {
            id: "reg-1".into(),
            name: "River Valley".into(),
            region_type: "river_valley".into(),
            region_subtype: "flood_plain".into(),
            description: "Fertile lowlands".into(),
            settlements: vec![],
            terrain: TerrainMap::default(),
            neighbors: RegionNeighbors::default(),
        };
        roundtrip(&r);
    }

    #[test]
    fn roundtrip_world() {
        let w = World {
            seed: 42,
            tick: 0,
            regions: vec![],
            charts_version: "0.1.0".into(),
            region_cols: 1,
        };
        roundtrip(&w);
    }

    #[test]
    fn person_default_no_panic() {
        let p = Person::default();
        assert!(p.name.is_empty());
        assert!(p.personality.is_empty());
    }

    #[test]
    fn world_holds_many_persons() {
        let mut world = World::default();
        let region = Region {
            id: "r1".into(),
            name: "Test".into(),
            region_type: "river_valley".into(),
            region_subtype: "flood_plain".into(),
            description: "desc".into(),
            terrain: TerrainMap::default(),
            neighbors: RegionNeighbors::default(),
            settlements: vec![Settlement {
                id: "s1".into(),
                name: "V".into(),
                size: "village".into(),
                region: "river_valley".into(),
                population: 10_000,
                description: "desc".into(),
                people: (0..10_000)
                    .map(|i| Person {
                        id: format!("p{}", i),
                        ..Default::default()
                    })
                    .collect(),
                services: vec![],
            }],
        };
        world.regions.push(region);
        let total: usize = world
            .regions
            .iter()
            .flat_map(|r| r.settlements.iter())
            .map(|s| s.people.len())
            .sum();
        assert_eq!(total, 10_000);
    }

    #[test]
    fn roundtrip_household() {
        let h = Household {
            id: "hh-1".into(),
            head_id: "p1".into(),
            spouse_id: Some("p2".into()),
            children_ids: vec!["p3".into(), "p4".into()],
            location_settlement_id: "set-1".into(),
            has_debt: true,
            debts: vec![Debt {
                creditor_id: "p5".into(),
                amount: 50.0,
                description: "seed loan".into(),
            }],
        };
        roundtrip(&h);
    }

    #[test]
    fn roundtrip_relationship() {
        let r = Relationship {
            from_id: "p1".into(),
            to_id: "p2".into(),
            kind: RelationshipKind::Friend,
            strength: 0.7,
            trust: 0.5,
            trust_baseline: 0.5,
            history: vec![RelationshipEvent {
                tick: 10,
                description: "shared a meal".into(),
            }],
        };
        roundtrip(&r);
    }

    #[test]
    fn craft_affinity_roundtrip_chart_keys() {
        for key in &["none", "word", "current", "still", "forge", "root"] {
            let affinity = CraftAffinity::from_chart_key(key).unwrap();
            assert_eq!(affinity.to_chart_key(), *key);
        }
        roundtrip(&CraftAffinity::Forge);
    }

    #[test]
    fn relationship_kind_all_variants() {
        let variants = [
            RelationshipKind::Spouse,
            RelationshipKind::Parent,
            RelationshipKind::Child,
            RelationshipKind::Sibling,
            RelationshipKind::Kin,
            RelationshipKind::Friend,
            RelationshipKind::Rival,
            RelationshipKind::Patron,
            RelationshipKind::Apprentice,
            RelationshipKind::Guildmate,
        ];
        for v in &variants {
            roundtrip(v);
        }
    }

    #[test]
    fn need_enum_roundtrip() {
        roundtrip(&Need::Food);
        roundtrip(&Need::Safety);
    }

    #[test]
    fn roundtrip_player() {
        let p = Player {
            id: "player-1".into(),
            name: "Hero".into(),
            people: "metsik".into(),
            sex: "m".into(),
            age_band: "youth".into(),
            profession: "forester".into(),
            social_class: "low".into(),
            craft_affinity: CraftAffinity::Root,
            personality: vec!["curious".into()],
            region: "forest".into(),
            settlement: "set-1".into(),
            perks: vec![Perk {
                name: "Keen Eye".into(),
                description: "Spot details others miss".into(),
                source: "personality_traits".into(),
            }],
            household_id: Some("hh-1".into()),
        };
        roundtrip(&p);
    }

    #[test]
    fn player_default_valid() {
        let p = Player::default();
        assert!(p.name.is_empty());
        assert!(p.perks.is_empty());
        assert!(p.household_id.is_none());
    }

    #[test]
    fn roundtrip_player_start() {
        let ps = PlayerStart {
            person: Person::default(),
            reroll_count: 2,
            point_buy_adjustments: vec![
                Adjustment::SwapProfession("trader".into()),
                Adjustment::SetCraft(CraftAffinity::Current),
                Adjustment::AddPerk(Perk {
                    name: "Silver Tongue".into(),
                    description: "Better trade deals".into(),
                    source: "profession".into(),
                }),
                Adjustment::CutHouseholdTie,
            ],
            accepted: false,
            inventory: Inventory::default(),
        };
        roundtrip(&ps);
    }

    #[test]
    fn adjustment_all_variants_roundtrip() {
        roundtrip(&Adjustment::SwapProfession("smith".into()));
        roundtrip(&Adjustment::SetCraft(CraftAffinity::Forge));
        roundtrip(&Adjustment::AddPerk(Perk {
            name: "test".into(),
            description: "test perk".into(),
            source: "test".into(),
        }));
        roundtrip(&Adjustment::CutHouseholdTie);
    }

    #[test]
    fn needs_default_satisfied_baseline() {
        let needs = Needs::default();
        for need in &[
            Need::Food,
            Need::Money,
            Need::Care,
            Need::Presence,
            Need::Safety,
        ] {
            assert!(
                (needs.get(*need) - 0.8).abs() < f64::EPSILON,
                "default {} should be 0.8, got {}",
                need,
                needs.get(*need)
            );
        }
    }

    #[test]
    fn needs_decay_reduces_value() {
        let mut needs = Needs::default();
        needs.decay(Need::Food, 0.1);
        assert!(
            (needs.get(Need::Food) - 0.7).abs() < f64::EPSILON,
            "food after decay should be 0.7, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]
    fn needs_satisfy_increases_value() {
        let mut needs = Needs::default();
        needs.decay(Need::Food, 0.5);
        needs.satisfy(Need::Food, 0.3);
        assert!(
            (needs.get(Need::Food) - 0.6).abs() < f64::EPSILON,
            "food after satisfy should be 0.6, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]
    fn needs_satisfy_clamped_at_one() {
        let mut needs = Needs::default();
        needs.satisfy(Need::Food, 0.3);
        assert!(
            (needs.get(Need::Food) - 1.0).abs() < f64::EPSILON,
            "food should clamp at 1.0, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]
    fn needs_decay_clamped_at_zero() {
        let mut needs = Needs::default();
        needs.decay(Need::Food, 1.0);
        assert!(
            (needs.get(Need::Food)).abs() < f64::EPSILON,
            "food should clamp at 0.0, got {}",
            needs.get(Need::Food)
        );
        needs.decay(Need::Food, 0.5);
        assert!(
            (needs.get(Need::Food)).abs() < f64::EPSILON,
            "food should still be 0.0 after extra decay, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]
    fn need_display() {
        assert_eq!(format!("{}", Need::Food), "Food");
        assert_eq!(format!("{}", Need::Money), "Money");
        assert_eq!(format!("{}", Need::Care), "Care");
        assert_eq!(format!("{}", Need::Presence), "Presence");
        assert_eq!(format!("{}", Need::Safety), "Safety");
    }

    #[test]
    fn needs_roundtrip() {
        roundtrip(&Needs::default());
    }

    #[test]
    fn terrain_passability() {
        assert!(!Terrain::Water.passable(), "water must be impassable");
        assert!(!Terrain::Mountain.passable(), "mountain must be impassable");
        assert!(Terrain::Grass.passable(), "grass must be passable");
        assert!(Terrain::Forest.passable(), "forest must be passable");
        assert!(Terrain::Road.passable(), "road must be passable");
        assert!(
            Terrain::Settlement.passable(),
            "settlement must be passable"
        );
        assert!(Terrain::Farmland.passable(), "farmland must be passable");
        assert!(Terrain::Sand.passable(), "sand must be passable");
        assert!(Terrain::Swamp.passable(), "swamp must be passable");
        assert!(Terrain::Coast.passable(), "coast must be passable");
        assert!(Terrain::Cave.passable(), "cave must be passable");
        assert!(Terrain::Tundra.passable(), "tundra must be passable");
        assert!(
            Terrain::DeepDesert.passable(),
            "deep desert must be passable"
        );
    }

    #[test]
    fn player_pos_serialization() {
        let pos = PlayerPos {
            region_idx: 2,
            px: 15,
            py: 7,
        };
        roundtrip(&pos);
    }

    #[test]
    fn inventory_add_remove() {
        let mut inv = Inventory::default();
        assert_eq!(inv.get(ItemType::Food), 0);
        inv.add(ItemType::Food, 5);
        assert_eq!(inv.get(ItemType::Food), 5);
        assert!(inv.remove(ItemType::Food, 3));
        assert_eq!(inv.get(ItemType::Food), 2);
        assert!(!inv.remove(ItemType::Food, 5));
        assert_eq!(inv.get(ItemType::Food), 2);
        assert!(inv.remove(ItemType::Food, 2));
        assert_eq!(inv.get(ItemType::Food), 0);
    }

    #[test]
    fn inventory_roundtrip() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Herb, 10);
        inv.add(ItemType::Coin, 3);
        roundtrip(&inv);
    }

    #[test]
    fn item_type_gather() {
        assert_eq!(ItemType::gather_from(Terrain::Grass), Some(ItemType::Herb));
        assert_eq!(ItemType::gather_from(Terrain::Forest), Some(ItemType::Wood));
        assert_eq!(ItemType::gather_from(Terrain::Mountain), None);
        assert_eq!(
            ItemType::gather_from(Terrain::Settlement),
            Some(ItemType::Coin)
        );
        assert_eq!(ItemType::gather_from(Terrain::Water), None);
    }

    #[test]
    fn craft_recipes_valid() {
        let recipes = craft_recipes();
        assert!(!recipes.is_empty(), "must have at least one recipe");
        for recipe in &recipes {
            assert!(!recipe.inputs.is_empty(), "recipe must have inputs");
            assert!(recipe.output_count > 0, "must produce something");
        }
    }

    #[test]
    fn time_of_day_from_hour() {
        assert_eq!(TimeOfDay::from_hour(0), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(4), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(5), TimeOfDay::Dawn);
        assert_eq!(TimeOfDay::from_hour(8), TimeOfDay::Day);
        assert_eq!(TimeOfDay::from_hour(12), TimeOfDay::Day);
        assert_eq!(TimeOfDay::from_hour(18), TimeOfDay::Dusk);
        assert_eq!(TimeOfDay::from_hour(21), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(23), TimeOfDay::Night);
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
        assert!(!TimeOfDay::Day.is_dark());
        assert!(!TimeOfDay::Dawn.is_dark());
        assert!(!TimeOfDay::Dusk.is_dark());
    }

    #[test]
    fn player_vitals_tick_hunger_decay() {
        let mut v = PlayerVitals::new();
        let mut inv = Inventory::default();
        v.tick(5, &mut inv, Season::Thaw);
        assert!(v.hunger < 1.0, "hunger should decrease");
        assert!(v.energy < 1.0, "energy should decrease");
    }

    #[test]
    fn player_vitals_auto_eat() {
        let mut v = PlayerVitals::new();
        v.hunger = 0.2;
        let mut inv = Inventory::default();
        inv.add(ItemType::Food, 3);
        v.tick(1, &mut inv, Season::Thaw);
        assert!(v.hunger > 0.2, "should auto-eat when hungry");
        assert_eq!(inv.get(ItemType::Food), 2);
    }

    #[test]
    fn player_vitals_rest_restores_energy() {
        let mut v = PlayerVitals::new();
        v.energy = 0.1;
        v.rest();
        assert!(v.energy > 0.5, "rest should restore energy");
    }

    #[test]
    fn player_vitals_labels() {
        let full = PlayerVitals {
            hunger: 0.8,
            energy: 0.8,
        };
        assert_eq!(full.hunger_label(), "full");
        assert_eq!(full.energy_label(), "energized");
        let hungry = PlayerVitals {
            hunger: 0.5,
            energy: 0.5,
        };
        assert_eq!(hungry.hunger_label(), "hungry");
        assert_eq!(hungry.energy_label(), "tired");
    }

    #[test]
    fn player_vitals_serialization() {
        let v = PlayerVitals::new();
        roundtrip(&v);
    }

    #[test]
    fn item_type_base_prices() {
        assert!(ItemType::Herb.base_price() > 0);
        assert!(ItemType::Iron.base_price() > ItemType::Herb.base_price());
        assert_eq!(ItemType::Coin.base_price(), 1);
    }

    #[test]
    fn tradeable_items_excludes_coin() {
        let items = ItemType::tradeable_items();
        assert!(!items.contains(&ItemType::Coin));
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn buy_sell_round_trip() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Coin, 10);
        let price = ItemType::Herb.base_price();
        assert!(inv.remove(ItemType::Coin, price));
        inv.add(ItemType::Herb, 1);
        assert_eq!(inv.get(ItemType::Herb), 1);
        assert_eq!(inv.get(ItemType::Coin), 10 - price);
        assert!(inv.remove(ItemType::Herb, 1));
        inv.add(ItemType::Coin, price);
        assert_eq!(inv.get(ItemType::Coin), 10);
    }

    #[test]
    fn encounter_forest_higher_chance() {
        let mut encounters = 0;
        for seed in 0..100u64 {
            if Encounter::roll(Terrain::Forest, 10, seed).is_some() {
                encounters += 1;
            }
        }
        assert!(
            encounters > 10,
            "forest should have ~25% encounter rate, got {}/100",
            encounters
        );
    }

    #[test]
    fn encounter_settlement_none() {
        let mut encounters = 0;
        for seed in 0..100u64 {
            if Encounter::roll(Terrain::Settlement, 10, seed).is_some() {
                encounters += 1;
            }
        }
        assert_eq!(encounters, 0, "settlements should never have encounters");
    }

    #[test]
    fn encounter_deterministic() {
        for seed in 0..50u64 {
            let a = Encounter::roll(Terrain::Forest, 10, seed);
            let b = Encounter::roll(Terrain::Forest, 10, seed);
            assert_eq!(a, b, "encounter roll must be deterministic");
        }
    }

    #[test]
    fn encounter_hostile_kinds() {
        assert!(EncounterKind::Wildlife.is_hostile());
        assert!(EncounterKind::Bandit.is_hostile());
        assert!(!EncounterKind::Traveler.is_hostile());
        assert!(!EncounterKind::Storm.is_hostile());
    }

    #[test]
    fn season_cycle() {
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
    fn danger_level_safe_region() {
        let mut terrain = TerrainMap {
            width: 4,
            height: 4,
            tiles: vec![Terrain::Grass; 16],
        };
        for y in 0..4 {
            for x in 0..4 {
                terrain.set(x, y, Terrain::Grass);
            }
        }
        let region = Region {
            id: "r1".into(),
            name: "Safeville".into(),
            region_type: "river_valley".into(),
            region_subtype: "flood_plain".into(),
            description: String::new(),
            settlements: vec![],
            terrain,
            neighbors: RegionNeighbors::default(),
        };
        assert_eq!(region.danger_level(), DangerLevel::Safe);
    }

    #[test]
    fn danger_level_forest_heavy() {
        let mut terrain = TerrainMap {
            width: 4,
            height: 4,
            tiles: vec![Terrain::Grass; 16],
        };
        for y in 0..4 {
            for x in 0..4 {
                terrain.set(x, y, Terrain::Forest);
            }
        }
        let region = Region {
            id: "r2".into(),
            name: "Darkwood".into(),
            region_type: "forest".into(),
            region_subtype: "deep_wood".into(),
            description: String::new(),
            settlements: vec![],
            terrain,
            neighbors: RegionNeighbors::default(),
        };
        assert_eq!(region.danger_level(), DangerLevel::Dangerous);
    }

    #[test]
    fn terrain_travel_hours() {
        assert_eq!(Terrain::Road.travel_hours(), 1);
        assert_eq!(Terrain::Settlement.travel_hours(), 1);
        assert_eq!(Terrain::Grass.travel_hours(), 2);
        assert_eq!(Terrain::Forest.travel_hours(), 3);
        assert_eq!(Terrain::Swamp.travel_hours(), 3);
    }

    #[test]
    fn settlement_service_costs() {
        assert_eq!(SettlementService::Tavern.cost(), 2);
        assert_eq!(SettlementService::Temple.cost(), 3);
    }

    #[test]
    fn god_affinity_adjust_clamps() {
        let mut ga = GodAffinity::new();
        ga.adjust(GodName::Keuru, 2.0);
        assert!((ga.get(GodName::Keuru) - 1.0).abs() < f64::EPSILON);
        ga.adjust(GodName::Keuru, -3.0);
        assert!((ga.get(GodName::Keuru) + 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn god_affinity_ally_and_grudge() {
        let mut ga = GodAffinity::new();
        assert_eq!(ga.strongest_ally(), None);
        assert_eq!(ga.strongest_grudge(), None);
        ga.adjust(GodName::Keuru, 0.5);
        ga.adjust(GodName::Oltzed, -0.3);
        assert_eq!(ga.strongest_ally(), Some(GodName::Keuru));
        assert_eq!(ga.strongest_grudge(), Some(GodName::Oltzed));
    }

    #[test]
    fn collapse_roll_deterministic() {
        let ga = GodAffinity::new();
        for seed in 0..50u64 {
            let a = Collapse::roll(seed, &ga, 0.0);
            let b = Collapse::roll(seed, &ga, 0.0);
            assert_eq!(a.outcome, b.outcome);
            assert_eq!(a.died, b.died);
        }
    }

    #[test]
    fn collapse_high_rep_prefer_settlement() {
        let ga = GodAffinity::new();
        let mut settlement_count = 0u32;
        let mut ditch_count = 0u32;
        for seed in 0..200u64 {
            let c = Collapse::roll(seed, &ga, 0.8);
            if matches!(
                c.outcome,
                CollapseOutcome::SettlementBed | CollapseOutcome::FestivalBench
            ) {
                settlement_count += 1;
            }
            if matches!(c.outcome, CollapseOutcome::Ditch) {
                ditch_count += 1;
            }
        }
        assert!(settlement_count > 5, "high rep should get more safe beds");
        assert!(ditch_count < 50, "high rep should avoid ditches");
    }

    #[test]
    fn collapse_metsik_ally_beast_guarded() {
        let mut ga = GodAffinity::new();
        ga.adjust(GodName::Keuru, 0.8);
        let mut guarded = 0u32;
        let mut hostile = 0u32;
        for seed in 0..200u64 {
            let c = Collapse::roll(seed, &ga, 0.0);
            if matches!(c.outcome, CollapseOutcome::BeastGuarded) {
                guarded += 1;
            }
            if matches!(c.outcome, CollapseOutcome::HostileBeast) {
                hostile += 1;
            }
        }
        assert!(guarded > 5, "Keuru ally should get beast-guarded more");
        assert!(hostile < 30, "Keuru ally should avoid hostile beasts");
    }

    #[test]
    fn collapse_grudge_more_hostile() {
        let mut ga = GodAffinity::new();
        ga.adjust(GodName::Keuru, -0.8);
        let mut hostile = 0u32;
        for seed in 0..200u64 {
            let c = Collapse::roll(seed, &ga, 0.0);
            if matches!(
                c.outcome,
                CollapseOutcome::HostileBeast | CollapseOutcome::Ditch
            ) {
                hostile += 1;
            }
        }
        assert!(
            hostile > 40,
            "Keuru grudge should cause more hostile outcomes"
        );
    }

    #[test]
    fn collapse_biased_hostile_downgrades_safe() {
        let ga = GodAffinity::new();
        let mut ditch_from_safe = 0u32;
        for seed in 0..200u64 {
            let c = Collapse::roll_biased(
                seed,
                &ga,
                0.5,
                PeopleKind::Metsik.bias_toward(PeopleKind::Sepat),
            );
            if matches!(c.outcome, CollapseOutcome::Ditch) {
                ditch_from_safe += 1;
            }
        }
        let mut ditch_neutral = 0u32;
        for seed in 0..200u64 {
            let c = Collapse::roll_biased(
                seed,
                &ga,
                0.5,
                PeopleKind::Arkit.bias_toward(PeopleKind::Arkit),
            );
            if matches!(c.outcome, CollapseOutcome::Ditch) {
                ditch_neutral += 1;
            }
        }
        assert!(
            ditch_from_safe >= ditch_neutral,
            "hostile bias should produce more ditches: {ditch_from_safe} vs {ditch_neutral}"
        );
    }

    #[test]
    fn collapse_biased_ally_upgrades_ditch() {
        let ga = GodAffinity::new();
        let mut hut_from_ditch = 0u32;
        for seed in 0..200u64 {
            let c = Collapse::roll_biased(
                seed,
                &ga,
                0.3,
                PeopleKind::Metsik.bias_toward(PeopleKind::Metsik),
            );
            if matches!(c.outcome, CollapseOutcome::StrangerHut) {
                hut_from_ditch += 1;
            }
        }
        assert!(
            hut_from_ditch > 0,
            "ally bias should upgrade some ditches to StrangerHut"
        );
    }

    #[test]
    fn collapse_outcome_properties() {
        assert!(CollapseOutcome::BeastGuarded.is_beast_aided());
        assert!(!CollapseOutcome::BeastGuarded.is_hostile());
        assert!(CollapseOutcome::HostileBeast.is_hostile());
        assert!(CollapseOutcome::SettlementBed.is_safe());
        assert!(CollapseOutcome::GodCampsite.is_divine());
        assert_eq!(CollapseOutcome::GodCampsite.glyph(), '✦');
    }

    #[test]
    fn encounter_actions_per_kind() {
        let wildlife = EncounterKind::Wildlife.available_actions();
        assert!(wildlife.contains(&EncounterAction::Flee));
        assert!(wildlife.contains(&EncounterAction::Calm));

        let bandit = EncounterKind::Bandit.available_actions();
        assert!(bandit.contains(&EncounterAction::Bribe));
        assert!(bandit.contains(&EncounterAction::Intimidate));

        let traveler = EncounterKind::Traveler.available_actions();
        assert!(traveler.contains(&EncounterAction::Talk));
        assert!(traveler.contains(&EncounterAction::Trade));

        let storm = EncounterKind::Storm.available_actions();
        assert!(storm.contains(&EncounterAction::Shelter));
        assert!(storm.contains(&EncounterAction::PushThrough));
    }

    #[test]
    fn encounter_action_costs() {
        assert_eq!(EncounterAction::Bribe.coin_cost(), 2);
        assert_eq!(EncounterAction::Flee.coin_cost(), 0);
        assert!((EncounterAction::Flee.energy_cost() - 0.15).abs() < f64::EPSILON);
        assert!((EncounterAction::PushThrough.energy_cost() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn encounter_action_god_effects() {
        assert_eq!(
            EncounterAction::Calm.god_affinity_effect(),
            Some((GodName::Keuru, 0.05))
        );
        assert_eq!(
            EncounterAction::Talk.god_affinity_effect(),
            Some((GodName::Masa, 0.03))
        );
        assert_eq!(EncounterAction::Flee.god_affinity_effect(), None);
    }

    #[test]
    fn people_kind_from_str() {
        assert_eq!(PeopleKind::from_name("metsik"), PeopleKind::Metsik);
        assert_eq!(PeopleKind::from_name("Sepät"), PeopleKind::Sepat);
        assert_eq!(PeopleKind::from_name("vayla"), PeopleKind::Vayla);
    }

    #[test]
    fn people_bias_same_people() {
        assert!((PeopleKind::Metsik.bias_toward(PeopleKind::Metsik) - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn people_bias_metsik_sepat_hostile() {
        assert!(PeopleKind::Metsik.bias_toward(PeopleKind::Sepat) < -0.15);
        assert!(PeopleKind::Sepat.bias_toward(PeopleKind::Metsik) < -0.10);
    }

    #[test]
    fn people_bias_sepat_ahjo_allied() {
        assert!(PeopleKind::Sepat.bias_toward(PeopleKind::Ahjo) > 0.0);
        assert!(PeopleKind::Ahjo.bias_toward(PeopleKind::Sepat) > 0.0);
    }

    #[test]
    fn people_gather_bonus_no_match() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Laakso, Terrain::Forest),
            0
        );
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Metsik, Terrain::Mountain),
            0
        );
    }

    #[test]
    fn season_bias_modifier() {
        assert_eq!(Season::Green.bias_modifier(), 0.05);
        assert_eq!(Season::Frost.bias_modifier(), -0.05);
        assert_eq!(Season::Thaw.bias_modifier(), 0.0);
    }

    #[test]
    fn trade_price_modifier_hospitable() {
        let r = InterPeopleBias::trade_price_modifier(&["hospitable".into()]);
        assert!(r < 0.0, "hospitable should reduce price: {r}");
    }

    #[test]
    fn trade_price_modifier_mercenary() {
        let r = InterPeopleBias::trade_price_modifier(&["mercenary".into()]);
        assert!(r > 0.0, "mercenary should increase price: {r}");
    }

    #[test]
    fn encounter_modifier_devout() {
        let r = InterPeopleBias::encounter_modifier(&["devout".into()]);
        assert!(r.calm > 0.0, "devout should boost calm: {}", r.calm);
    }

    #[test]
    fn encounter_modifier_xenophobic() {
        let r = InterPeopleBias::encounter_modifier(&["xenophobic".into()]);
        assert!(r.flee > 0.0, "xenophobic should boost flee: {}", r.flee);
    }

    #[test]
    fn danger_level_biased_hostile_upgrades_danger() {
        let base = DangerLevel::Risky;
        let biased_safe = DangerLevel::Safe;
        let biased_dangerous = DangerLevel::Dangerous;
        assert!(base != biased_safe || base != biased_dangerous);
        assert!(matches!(base, DangerLevel::Risky));
    }

    #[test]
    fn people_bias_laakso_xenophobic() {
        assert!(PeopleKind::Laakso.bias_toward(PeopleKind::Vayla) < 0.0);
        assert!(PeopleKind::Laakso.bias_toward(PeopleKind::Ahjo) < 0.0);
    }

    #[test]
    fn inter_people_bias_price_modifier() {
        let ib = InterPeopleBias::new(PeopleKind::Metsik);
        assert!(
            ib.price_modifier(PeopleKind::Sepat) > 1.0,
            "Metsik buying from Sepat (hostile seller) should pay more"
        );
        assert!(
            ib.price_modifier(PeopleKind::Metsik) < 1.0,
            "Metsik buying from fellow Metsik (friendly seller) should pay less"
        );
    }

    #[test]
    fn personality_mod_hospitable() {
        let mod_val = InterPeopleBias::personality_mod(&["hospitable".into()]);
        assert!(mod_val > 0.0, "hospitable should increase trust");
    }

    #[test]
    fn personality_mod_xenophobic() {
        let mod_val = InterPeopleBias::personality_mod(&["xenophobic".into()]);
        assert!(mod_val < 0.0, "xenophobic should decrease trust");
    }

    #[test]
    fn greeting_cross_people() {
        let greeting = PeopleKind::Metsik.greeting_to(PeopleKind::Sepat);
        assert!(!greeting.is_empty());
        assert!(greeting.contains("Forest-people") || greeting.contains("iron-ore"));
    }

    #[test]
    fn people_gather_bonus_metsik_forest() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Metsik, Terrain::Forest),
            1
        );
    }

    #[test]
    fn people_gather_bonus_sepat_mountain() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Sepat, Terrain::Mountain),
            1
        );
    }

    #[test]
    fn people_gather_bonus_ahjo_farmland() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Ahjo, Terrain::Farmland),
            1
        );
    }

    #[test]
    fn durability_default_is_full() {
        let inv = Inventory::default();
        assert!(!inv.has(ItemType::Iron));
        assert_eq!(inv.durability(ItemType::Iron), 1.0);
        assert!(!inv.is_broken(ItemType::Iron));
    }

    #[test]
    fn durability_decay_reduces() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Iron, 3);
        inv.decay(ItemType::Iron, 0.3);
        assert!((inv.durability(ItemType::Iron) - 0.7).abs() < 0.001);
        assert!(!inv.is_broken(ItemType::Iron));
    }

    #[test]
    fn durability_broken_when_zero() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Iron, 1);
        inv.decay(ItemType::Iron, 1.5);
        assert!(inv.is_broken(ItemType::Iron));
        assert!(inv.durability(ItemType::Iron) <= 0.0);
    }

    #[test]
    fn repair_cost_scaled_by_base_price() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Iron, 1);
        inv.decay(ItemType::Iron, 0.5);
        let cost = inv.repair_cost(ItemType::Iron);
        assert!(cost > 0, "repair cost should be positive: got {}", cost);
        assert_eq!(cost, 5, "Iron(5) at 50%% wear: ceil((1-0.5)*5*2) = 5");
    }

    #[test]
    fn repair_restores_durability() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Wood, 2);
        inv.decay(ItemType::Wood, 0.4);
        assert!((inv.durability(ItemType::Wood) - 0.6).abs() < 0.001);
        let cost = inv.repair(ItemType::Wood);
        assert!(cost > 0);
        assert!((inv.durability(ItemType::Wood) - 1.0).abs() < 0.001);
    }

    #[test]
    fn repair_full_item_costs_nothing() {
        let inv = Inventory::default();
        assert_eq!(inv.repair_cost(ItemType::Iron), 0);
    }

    #[test]
    fn npc_memory_default_empty() {
        let mem = NpcMemory::default();
        assert_eq!(mem.count(), 0);
        assert!(mem.last().is_none());
        assert_eq!(mem.cumulative_trust(), 0.0);
    }

    #[test]
    fn npc_memory_add_interaction() {
        let mut mem = NpcMemory::default();
        mem.add(EncounterAction::Talk, 100, "TestSettlement".into(), 0.02);
        assert_eq!(mem.count(), 1);
        assert!(mem.last().is_some());
        assert!((mem.cumulative_trust() - 0.02).abs() < 0.001);
    }

    #[test]
    fn npc_memory_cumulative_trust() {
        let mut mem = NpcMemory::default();
        mem.add(EncounterAction::Talk, 100, "TestSettlement".into(), 0.02);
        mem.add(EncounterAction::Trade, 200, "TestSettlement".into(), 0.03);
        mem.add(
            EncounterAction::Intimidate,
            300,
            "TestSettlement".into(),
            -0.02,
        );
        assert!((mem.cumulative_trust() - 0.03).abs() < 0.001);
    }

    #[test]
    fn npc_memory_caps_at_10_interactions() {
        let mut mem = NpcMemory::default();
        for i in 0..15 {
            mem.add(EncounterAction::Talk, i * 10, "TestSettlement".into(), 0.01);
        }
        assert_eq!(mem.count(), 10);
    }

    #[test]
    fn npc_schedule_default_blocks() {
        let schedule = NpcSchedule::default();
        assert_eq!(schedule.blocks.len(), 6);
        assert_eq!(schedule.activity_at_hour(0), NpcActivity::Sleep);
        assert_eq!(schedule.activity_at_hour(4), NpcActivity::Work);
        assert_eq!(schedule.activity_at_hour(8), NpcActivity::Work);
        assert_eq!(schedule.activity_at_hour(12), NpcActivity::Socialize);
        assert_eq!(schedule.activity_at_hour(16), NpcActivity::Idle);
        assert_eq!(schedule.activity_at_hour(20), NpcActivity::Sleep);
    }

    #[test]
    fn npc_schedule_availability() {
        let schedule = NpcSchedule::default();
        assert!(!schedule.is_available_at_hour(0)); // Sleep
        assert!(schedule.is_available_at_hour(4)); // Work
        assert!(schedule.is_available_at_hour(8)); // Work
        assert!(schedule.is_available_at_hour(12)); // Socialize
        assert!(schedule.is_available_at_hour(16)); // Idle
        assert!(!schedule.is_available_at_hour(20)); // Sleep
    }

    #[test]
    fn npc_activity_availability() {
        assert!(!NpcActivity::Sleep.is_available());
        assert!(NpcActivity::Work.is_available());
        assert!(NpcActivity::Socialize.is_available());
        assert!(!NpcActivity::Travel.is_available());
        assert!(NpcActivity::Worship.is_available());
        assert!(NpcActivity::Craft.is_available());
        assert!(NpcActivity::Idle.is_available());
    }

    #[test]
    fn combat_action_names() {
        assert_eq!(CombatAction::Attack.name(), "attack");
        assert_eq!(CombatAction::Parry.name(), "parry");
        assert_eq!(CombatAction::Feint.name(), "feint");
        assert_eq!(CombatAction::Yield.name(), "yield");
    }

    #[test]
    fn combat_outcome_deterministic() {
        let outcome1 =
            CombatOutcome::resolve(CombatAction::Attack, CombatAction::Attack, 0.5, 0.5, 42);
        let outcome2 =
            CombatOutcome::resolve(CombatAction::Attack, CombatAction::Attack, 0.5, 0.5, 42);
        assert_eq!(outcome1.player_injury, outcome2.player_injury);
        assert_eq!(outcome1.npc_injury, outcome2.npc_injury);
        assert_eq!(outcome1.reputation_delta, outcome2.reputation_delta);
        assert_eq!(outcome1.player_died, outcome2.player_died);
    }

    #[test]
    fn combat_yield_no_death() {
        let outcome =
            CombatOutcome::resolve(CombatAction::Yield, CombatAction::Attack, 0.0, 0.5, 12345);
        assert!(!outcome.player_died);
        assert!(outcome.player_injury > 0.0);
    }

    #[test]
    fn npc_combat_action_varies_by_trust() {
        let aggressive = npc_combat_action(0.0, 0.8, 42);
        let defensive = npc_combat_action(0.9, 0.2, 42);
        // Different trust levels should produce different actions (with same seed)
        assert_ne!(aggressive, defensive);
    }

    #[test]
    fn combat_outcome_ranges_valid() {
        for seed in 0..100 {
            let outcome =
                CombatOutcome::resolve(CombatAction::Attack, CombatAction::Attack, 0.5, 0.5, seed);
            assert!(outcome.player_injury >= 0.0 && outcome.player_injury <= 0.5);
            assert!(outcome.npc_injury >= 0.0 && outcome.npc_injury <= 0.5);
            assert!(outcome.reputation_delta >= -0.1 && outcome.reputation_delta <= 0.1);
        }
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
        ];
        for w in weathers {
            assert!(w.gather_modifier() > 0.0 && w.gather_modifier() <= 1.0);
            assert!(w.travel_speed_modifier() > 0.0 && w.travel_speed_modifier() <= 1.0);
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
    fn quest_generation_deterministic() {
        let q1 = Quest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        let q2 = Quest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        assert_eq!(q1.id, q2.id);
        assert_eq!(q1.quest_type, q2.quest_type);
        assert_eq!(q1.description, q2.description);
        assert_eq!(q1.reward_coins, q2.reward_coins);
    }

    #[test]
    fn quest_types_variety() {
        let mut types = std::collections::HashSet::new();
        for seed in 0..20 {
            let q = Quest::generate(seed, "npc-1".into(), "Test NPC".into(), 100);
            types.insert(q.quest_type);
        }
        assert!(
            types.len() >= 3,
            "should generate at least 3 different quest types"
        );
    }

    #[test]
    fn quest_progress_tracking() {
        let mut q = Quest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
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
    fn quest_deadline_in_future() {
        let q = Quest::generate(42, "npc-1".into(), "Test NPC".into(), 100);
        assert!(q.deadline_tick > 100);
        assert!(q.deadline_tick <= 100 + 48 + 72); // max 5 days
    }

    #[test]
    fn quest_rewards_positive() {
        for seed in 0..10 {
            let q = Quest::generate(seed, "npc-1".into(), "Test NPC".into(), 100);
            assert!(q.reward_coins >= 3);
            assert!(q.reward_reputation > 0.0);
        }
    }

    #[test]
    fn crop_type_properties() {
        assert_eq!(CropType::Grain.name(), "grain");
        assert_eq!(CropType::Grain.growth_ticks(), 72);
        assert_eq!(CropType::Grain.base_yield(), 4);
        assert!(CropType::Grain.regional_suitability(Terrain::Farmland) > 1.0);
    }

    #[test]
    fn farm_growth_stages() {
        let mut farm = Farm::new(42, CropType::Grain, 100, Terrain::Farmland);
        assert_eq!(farm.stage, GrowthStage::Planted);
        assert!(!farm.is_ready());

        farm.update_growth(110, Weather::Clear);
        assert!(farm.growth_progress > 0.0);

        farm.update_growth(200, Weather::Clear);
        assert!(farm.is_ready());
        assert_eq!(farm.stage, GrowthStage::Ready);
    }

    #[test]
    fn farm_harvest_yield() {
        let mut farm = Farm::new(42, CropType::Herb, 100, Terrain::Forest);
        farm.update_growth(200, Weather::Clear);
        assert!(farm.is_ready());
        let yield_amount = farm.harvest_yield();
        assert!(yield_amount >= 5); // herb base yield is 5
    }

    #[test]
    fn farm_regional_suitability() {
        let grain_farmland = CropType::Grain.regional_suitability(Terrain::Farmland);
        let grain_desert = CropType::Grain.regional_suitability(Terrain::DeepDesert);
        assert!(grain_farmland > grain_desert);

        let herb_forest = CropType::Herb.regional_suitability(Terrain::Forest);
        let herb_grass = CropType::Herb.regional_suitability(Terrain::Grass);
        assert!(herb_forest > herb_grass);
    }

    #[test]
    fn farm_weather_effect() {
        let mut farm_clear = Farm::new(42, CropType::Grain, 100, Terrain::Farmland);
        let mut farm_storm = Farm::new(42, CropType::Grain, 100, Terrain::Farmland);

        farm_clear.update_growth(150, Weather::Clear);
        farm_storm.update_growth(150, Weather::Storm);

        assert!(farm_clear.growth_progress > farm_storm.growth_progress);
    }

    #[test]
    fn building_type_properties() {
        assert_eq!(BuildingType::Shelter.name(), "shelter");
        let materials = BuildingType::Shelter.materials_required();
        assert!(materials.len() >= 2);
        assert!(BuildingType::Shelter.build_ticks() > 0);
        assert!(BuildingType::Shelter.energy_cost() > 0.0);
    }

    #[test]
    fn building_construction_progress() {
        let mut building = Building::new(42, BuildingType::Shelter, "TestSettlement".into());
        assert!(!building.is_complete());
        assert_eq!(building.build_progress, 0.0);

        building.advance_construction(24, 100);
        assert!(building.build_progress > 0.0);
        assert!(!building.is_complete());

        building.advance_construction(100, 200);
        assert!(building.is_complete());
        assert!(building.built_tick.is_some());
    }

    #[test]
    fn building_material_requirements() {
        let shelter_mats = BuildingType::Shelter.materials_required();
        assert!(shelter_mats.iter().any(|(item, _)| *item == ItemType::Wood));

        let workshop_mats = BuildingType::Workshop.materials_required();
        assert!(workshop_mats
            .iter()
            .any(|(item, _)| *item == ItemType::Iron));
    }

    #[test]
    fn building_energy_costs_vary() {
        let shelter_cost = BuildingType::Shelter.energy_cost();
        let trap_cost = BuildingType::Trap.energy_cost();
        assert!(shelter_cost > trap_cost);
    }

    #[test]
    fn building_completion_sets_tick() {
        let mut building = Building::new(42, BuildingType::Hearth, "TestSettlement".into());
        building.advance_construction(100, 500);
        assert!(building.is_complete());
        assert_eq!(building.built_tick, Some(500));
    }

    #[test]
    fn disease_properties() {
        assert_eq!(Disease::Fever.name(), "fever");
        assert!(Disease::Fever.vitals_decay_modifier() > 1.0);
        assert!(Disease::Fever.recovery_ticks() > 0);
        assert!(Disease::Fever.contraction_probability(Terrain::Swamp) > 0.0);
    }

    #[test]
    fn disease_contraction_deterministic() {
        let result1 = Disease::can_contract(42, 100, Terrain::Swamp, Disease::Fever);
        let result2 = Disease::can_contract(42, 100, Terrain::Swamp, Disease::Fever);
        assert_eq!(result1, result2);
    }

    #[test]
    fn active_disease_recovery() {
        let disease = ActiveDisease::new(Disease::Fever, 100);
        assert!(!disease.is_recovered(120));
        assert!(!disease.is_recovered(147));
        assert!(disease.is_recovered(148));
        assert!(disease.is_recovered(200));
    }

    #[test]
    fn disease_vitals_modifier() {
        let disease = ActiveDisease::new(Disease::Plague, 100);
        assert!(disease.vitals_modifier() > 1.5);
    }

    #[test]
    fn disease_regional_probability() {
        let swamp_prob = Disease::Fever.contraction_probability(Terrain::Swamp);
        let desert_prob = Disease::Fever.contraction_probability(Terrain::DeepDesert);
        assert!(swamp_prob > desert_prob);
    }

    #[test]
    fn caravan_generation_creates_valid_goods() {
        let caravan = Caravan::generate(42, "Origin".into(), "Destination".into(), 100);
        assert_eq!(caravan.goods.len(), 1);
        assert!(!caravan.goods.is_empty());
        assert_eq!(caravan.origin, "Origin");
        assert_eq!(caravan.destination, "Destination");
    }

    #[test]
    fn caravan_generation_deterministic() {
        let c1 = Caravan::generate(42, "A".into(), "B".into(), 100);
        let c2 = Caravan::generate(42, "A".into(), "B".into(), 100);
        assert_eq!(c1.goods.len(), c2.goods.len());
        assert_eq!(c1.arrival_tick, c2.arrival_tick);
        assert_eq!(c1.travel_cost, c2.travel_cost);
    }

    #[test]
    fn caravan_transit_timing() {
        let caravan = Caravan::generate(42, "A".into(), "B".into(), 100);
        assert!(!caravan.is_in_transit(99));
        assert!(caravan.is_in_transit(100));
        assert!(caravan.is_in_transit(120));
        assert!(!caravan.is_in_transit(caravan.arrival_tick));
        assert!(caravan.has_arrived(caravan.arrival_tick));
    }

    #[test]
    fn caravan_price_modifier() {
        let mut caravan = Caravan::generate(42, "A".into(), "B".into(), 100);
        let item = ItemType::Wood;
        caravan.goods.clear();
        caravan.goods.push((item, 10));

        let no_caravan_mod = 1.0;
        let transit_mod = caravan.price_modifier(item, 110);
        assert!(transit_mod < no_caravan_mod);
    }

    #[test]
    fn caravan_price_no_effect_for_missing_items() {
        let caravan = Caravan::generate(42, "A".into(), "B".into(), 100);
        let mod_no_effect = caravan.price_modifier(ItemType::Herb, 110);
        assert!((0.9..=1.1).contains(&mod_no_effect));
    }

    #[test]
    fn animal_properties() {
        assert_eq!(Animal::Dog.name(), "dog");
        assert!(Animal::Dog.gathering_bonus() > 0.0);
        assert!(Animal::Horse.travel_speed_multiplier() < 1.0);
        assert!(Animal::Ox.carry_capacity_bonus() > 0);
        assert!(Animal::Falcon.scouting_bonus() > 0.0);
        assert!(Animal::Goat.milk_production() > 0);
    }

    #[test]
    fn animal_costs_and_needs() {
        assert!(Animal::Horse.cost() > Animal::Dog.cost());
        assert!(Animal::Dog.food_per_tick() > 0);
        assert!(Animal::Dog.rest_per_tick() > 0);
    }

    #[test]
    fn companion_creation() {
        let companion = Companion::new(Animal::Dog, "Rex".into(), 100);
        assert_eq!(companion.animal, Animal::Dog);
        assert_eq!(companion.name, "Rex");
        assert_eq!(companion.acquired_tick, 100);
        assert_eq!(companion.food_need, 0.0);
    }

    #[test]
    fn companion_need_decay() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);
        companion.decay_needs(10);
        assert!(companion.food_need > 0.0);
        assert!(companion.rest_need > 0.0);
        assert!(!companion.is_starving());
    }

    #[test]
    fn companion_feeding() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);
        companion.food_need = 50.0;
        companion.feed(1.0);
        assert!(companion.food_need < 50.0);
    }

    #[test]
    fn companion_starvation_death() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);
        companion.food_need = 100.0;
        assert!(!companion.is_alive());
    }

    #[test]
    fn encounter_log_starts_empty() {
        let log = EncounterLog::new();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn encounter_log_pushes_in_order() {
        let mut log = EncounterLog::new();
        for i in 0..5 {
            log.push(EncounterLogEntry {
                day: i,
                hour: 0,
                kind: EncounterKind::Wildlife,
                terrain: Terrain::Forest,
                action: EncounterAction::Flee,
                hostile: true,
            });
        }
        assert_eq!(log.len(), 5);
        let days: Vec<u32> = log.iter().map(|e| e.day).collect();
        assert_eq!(days, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn encounter_log_caps_at_twenty() {
        let mut log = EncounterLog::new();
        for i in 0..35 {
            log.push(EncounterLogEntry {
                day: i,
                hour: 0,
                kind: EncounterKind::Wildlife,
                terrain: Terrain::Forest,
                action: EncounterAction::Flee,
                hostile: true,
            });
        }
        assert_eq!(log.len(), 20);
        let first = log.iter().next().map(|e| e.day);
        let last = log.iter().next_back().map(|e| e.day);
        assert_eq!(first, Some(15));
        assert_eq!(last, Some(34));
    }
}
