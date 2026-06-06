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
        }
    }

    pub fn passable(self) -> bool {
        !matches!(self, Terrain::Water | Terrain::Mountain)
    }

    pub fn travel_hours(self) -> u32 {
        match self {
            Terrain::Road | Terrain::Settlement => 1,
            Terrain::Grass | Terrain::Farmland | Terrain::Sand => 2,
            Terrain::Forest | Terrain::Swamp => 3,
            Terrain::Water | Terrain::Mountain => 2,
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
            Terrain::Grass | Terrain::Farmland => Some(ItemType::Herb),
            Terrain::Forest => Some(ItemType::Wood),
            Terrain::Settlement => Some(ItemType::Coin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Inventory {
    pub items: std::collections::HashMap<ItemType, u32>,
}

impl Inventory {
    pub fn get(&self, item: ItemType) -> u32 {
        self.items.get(&item).copied().unwrap_or(0)
    }

    pub fn add(&mut self, item: ItemType, count: u32) {
        *self.items.entry(item).or_insert(0) += count;
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
}

pub fn craft_recipes() -> Vec<CraftRecipe> {
    vec![
        CraftRecipe {
            name: "Bandage".into(),
            inputs: vec![(ItemType::Herb, 3), (ItemType::Cloth, 1)],
            output: ItemType::Food,
            output_count: 2,
        },
        CraftRecipe {
            name: "Tool".into(),
            inputs: vec![(ItemType::Wood, 2), (ItemType::Iron, 1)],
            output: ItemType::Iron,
            output_count: 2,
        },
        CraftRecipe {
            name: "Meal".into(),
            inputs: vec![(ItemType::Herb, 2), (ItemType::Food, 1)],
            output: ItemType::Food,
            output_count: 3,
        },
    ]
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
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn from_day(day: u32) -> Self {
        match day % 360 {
            0..=89 => Season::Spring,
            90..=179 => Season::Summer,
            180..=269 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    pub fn gather_multiplier(self) -> f64 {
        match self {
            Season::Spring => 1.0,
            Season::Summer => 1.2,
            Season::Autumn => 0.8,
            Season::Winter => 0.3,
        }
    }

    pub fn need_decay_multiplier(self) -> f64 {
        match self {
            Season::Winter => 1.3,
            _ => 1.0,
        }
    }

    pub fn glyph(self) -> char {
        match self {
            Season::Spring => '❀',
            Season::Summer => '✿',
            Season::Autumn => '🍂',
            Season::Winter => '❄',
        }
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Season::Spring => write!(f, "Spring"),
            Season::Summer => write!(f, "Summer"),
            Season::Autumn => write!(f, "Autumn"),
            Season::Winter => write!(f, "Winter"),
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
    Metsik,
    Ahjo,
    Vayla,
}

impl GodName {
    pub fn label(self) -> &'static str {
        match self {
            GodName::Metsik => "Metsik",
            GodName::Ahjo => "Ahjo",
            GodName::Vayla => "Väylä",
        }
    }

    pub fn domains(self) -> &'static str {
        match self {
            GodName::Metsik => "forests, beasts, wild places",
            GodName::Ahjo => "hearth, craft, settlement",
            GodName::Vayla => "rivers, paths, travelers",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            GodName::Metsik => '🌲',
            GodName::Ahjo => '🔥',
            GodName::Vayla => '🌊',
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct GodAffinity {
    pub metsik: f64,
    pub ahjo: f64,
    pub vayla: f64,
}

impl GodAffinity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, god: GodName) -> f64 {
        match god {
            GodName::Metsik => self.metsik,
            GodName::Ahjo => self.ahjo,
            GodName::Vayla => self.vayla,
        }
    }

    pub fn adjust(&mut self, god: GodName, delta: f64) {
        let val = match god {
            GodName::Metsik => &mut self.metsik,
            GodName::Ahjo => &mut self.ahjo,
            GodName::Vayla => &mut self.vayla,
        };
        *val = (*val + delta).clamp(-1.0, 1.0);
    }

    pub fn strongest_ally(&self) -> Option<GodName> {
        let gods = [
            (GodName::Metsik, self.metsik),
            (GodName::Ahjo, self.ahjo),
            (GodName::Vayla, self.vayla),
        ];
        let best = gods
            .iter()
            .filter(|(_, v)| *v > 0.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        best.map(|(g, _)| *g)
    }

    pub fn strongest_grudge(&self) -> Option<GodName> {
        let gods = [
            (GodName::Metsik, self.metsik),
            (GodName::Ahjo, self.ahjo),
            (GodName::Vayla, self.vayla),
        ];
        let worst = gods
            .iter()
            .filter(|(_, v)| *v < 0.0)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        worst.map(|(g, _)| *g)
    }
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
                GodName::Metsik => {
                    weights[0].1 += bonus / 3;
                    weights[1].1 += bonus;
                    weights[2].1 = weights[2].1.saturating_sub(bonus);
                    weights[3].1 = weights[3].1.saturating_sub(bonus / 2);
                    weights[6].1 += bonus / 4;
                }
                GodName::Ahjo => {
                    weights[0].1 += bonus / 3;
                    weights[4].1 += bonus / 2;
                    weights[5].1 += bonus;
                    weights[8].1 += bonus / 2;
                    weights[9].1 = weights[9].1.saturating_sub(bonus / 2);
                }
                GodName::Vayla => {
                    weights[0].1 += bonus / 3;
                    weights[6].1 += bonus / 2;
                    weights[7].1 += bonus;
                    weights[4].1 += bonus / 4;
                }
            }
        }

        if let Some(grudge) = affinity.strongest_grudge() {
            let penalty = (affinity.get(grudge).abs() * 60.0) as u32;
            match grudge {
                GodName::Metsik => {
                    weights[2].1 += penalty;
                    weights[3].1 += penalty / 2;
                    weights[9].1 += penalty / 3;
                    weights[1].1 = weights[1].1.saturating_sub(penalty);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Ahjo => {
                    weights[9].1 += penalty;
                    weights[4].1 = weights[4].1.saturating_sub(penalty / 2);
                    weights[5].1 = weights[5].1.saturating_sub(penalty / 3);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Vayla => {
                    weights[9].1 += penalty / 2;
                    weights[7].1 = weights[7].1.saturating_sub(penalty / 2);
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

        if affinity.get(GodName::Metsik) > 0.5 {
            weights[1].1 += 60;
            weights[2].1 = weights[2].1.saturating_sub(30);
        }

        if affinity.get(GodName::Vayla) > 0.4 {
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
            Some(GodName::Metsik)
        } else if chosen == CollapseOutcome::Riverbank {
            Some(GodName::Vayla)
        } else if chosen == CollapseOutcome::FestivalBench
            || chosen == CollapseOutcome::SettlementBed
        {
            Some(GodName::Ahjo)
        } else {
            None
        };

        Collapse {
            outcome: chosen,
            died,
            rescued_by,
        }
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
            EncounterAction::Calm => Some((GodName::Metsik, 0.05)),
            EncounterAction::Intimidate => Some((GodName::Metsik, -0.02)),
            EncounterAction::Talk => Some((GodName::Vayla, 0.03)),
            EncounterAction::Trade => Some((GodName::Vayla, 0.04)),
            EncounterAction::Bribe => Some((GodName::Ahjo, -0.01)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    pub kind: EncounterKind,
}

impl Encounter {
    pub fn roll(terrain: Terrain, hour: u32, seed: u64) -> Option<Self> {
        let hash = seed.wrapping_mul(2654435761)
            ^ (terrain as u64).wrapping_mul(40503)
            ^ (hour as u64).wrapping_mul(92000);
        let val = hash % 100;
        let (threshold, kind) = match terrain {
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
            Terrain::Road => (5, EncounterKind::Traveler),
            Terrain::Settlement => (0, EncounterKind::Traveler),
            _ => (8, EncounterKind::Wildlife),
        };
        if (val % 100) < threshold {
            Some(Encounter { kind })
        } else {
            None
        }
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
                if let Some(Terrain::Forest | Terrain::Mountain | Terrain::Swamp) =
                    self.terrain.get(x, y)
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
}

impl SettlementService {
    pub fn glyph(self) -> char {
        match self {
            SettlementService::Tavern => '🍺',
            SettlementService::Temple => '⛪',
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SettlementService::Tavern => "Tavern",
            SettlementService::Temple => "Temple",
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            SettlementService::Tavern => 2,
            SettlementService::Temple => 3,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    pub from_id: String,
    pub to_id: String,
    pub kind: RelationshipKind,
    pub strength: f64,
    pub trust: f64,
    pub history: Vec<RelationshipEvent>,
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
        v.tick(5, &mut inv, Season::Spring);
        assert!(v.hunger < 1.0, "hunger should decrease");
        assert!(v.energy < 1.0, "energy should decrease");
    }

    #[test]
    fn player_vitals_auto_eat() {
        let mut v = PlayerVitals::new();
        v.hunger = 0.2;
        let mut inv = Inventory::default();
        inv.add(ItemType::Food, 3);
        v.tick(1, &mut inv, Season::Spring);
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
        assert_eq!(Season::from_day(1), Season::Spring);
        assert_eq!(Season::from_day(90), Season::Summer);
        assert_eq!(Season::from_day(180), Season::Autumn);
        assert_eq!(Season::from_day(270), Season::Winter);
        assert_eq!(Season::from_day(360), Season::Spring);
    }

    #[test]
    fn season_gather_multiplier() {
        assert!((Season::Summer.gather_multiplier() - 1.2).abs() < f64::EPSILON);
        assert!((Season::Winter.gather_multiplier() - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn winter_faster_hunger_decay() {
        assert!((Season::Winter.need_decay_multiplier() - 1.3).abs() < f64::EPSILON);
    }

    #[test]
    fn clock_season() {
        let clock = GameClock::new(90, 12);
        assert_eq!(clock.season(), Season::Summer);
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
        ga.adjust(GodName::Metsik, 2.0);
        assert!((ga.get(GodName::Metsik) - 1.0).abs() < f64::EPSILON);
        ga.adjust(GodName::Metsik, -3.0);
        assert!((ga.get(GodName::Metsik) + 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn god_affinity_ally_and_grudge() {
        let mut ga = GodAffinity::new();
        assert_eq!(ga.strongest_ally(), None);
        assert_eq!(ga.strongest_grudge(), None);
        ga.adjust(GodName::Metsik, 0.5);
        ga.adjust(GodName::Ahjo, -0.3);
        assert_eq!(ga.strongest_ally(), Some(GodName::Metsik));
        assert_eq!(ga.strongest_grudge(), Some(GodName::Ahjo));
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
        ga.adjust(GodName::Metsik, 0.8);
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
        assert!(guarded > 5, "Metsik ally should get beast-guarded more");
        assert!(hostile < 30, "Metsik ally should avoid hostile beasts");
    }

    #[test]
    fn collapse_grudge_more_hostile() {
        let mut ga = GodAffinity::new();
        ga.adjust(GodName::Metsik, -0.8);
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
            "Metsik grudge should cause more hostile outcomes"
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
            Some((GodName::Metsik, 0.05))
        );
        assert_eq!(
            EncounterAction::Talk.god_affinity_effect(),
            Some((GodName::Vayla, 0.03))
        );
        assert_eq!(EncounterAction::Flee.god_affinity_effect(), None);
    }
}
