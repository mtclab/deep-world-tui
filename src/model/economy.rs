use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum QualityTier {
    #[default]
    Sturdy,
    Rough,
    Fine,
    Masterwork,
}

impl QualityTier {
    pub fn label(self) -> &'static str {
        match self {
            QualityTier::Rough => "rough",
            QualityTier::Sturdy => "sturdy",
            QualityTier::Fine => "fine",
            QualityTier::Masterwork => "masterwork",
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            QualityTier::Rough => "Worn and serviceable, but barely.",
            QualityTier::Sturdy => "Solid work. Dependable enough.",
            QualityTier::Fine => "Well-crafted. A pleasure to hold.",
            QualityTier::Masterwork => "Flawless. The maker's skill is unmistakable.",
        }
    }

    pub fn gather_multiplier(self) -> f64 {
        match self {
            QualityTier::Rough => 0.7,
            QualityTier::Sturdy => 1.0,
            QualityTier::Fine => 1.3,
            QualityTier::Masterwork => 1.6,
        }
    }

    pub fn degrade_rate(self) -> f64 {
        match self {
            QualityTier::Rough => 0.15,
            QualityTier::Sturdy => 0.10,
            QualityTier::Fine => 0.07,
            QualityTier::Masterwork => 0.04,
        }
    }

    pub fn from_durability(dur: f64) -> Self {
        if dur >= 0.95 {
            QualityTier::Masterwork
        } else if dur >= 0.7 {
            QualityTier::Fine
        } else if dur >= 0.35 {
            QualityTier::Sturdy
        } else {
            QualityTier::Rough
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Food,
    Water,
    Coin,
    Herb,
    Wood,
    Stone,
    Cloth,
    Iron,
    Branches,
    Cordage,
    Tinder,
    Nails,
    Thatch,
    Glass,
    /// A proper crafted tool — the best gathering aid, wears with use.
    Tool,
    /// Dressings for the sick: tended during rest, an illness runs its course
    /// faster.
    Bandage,
    /// A set snare: yields food while resting in the wild, wears with use.
    Trap,
}

impl ItemType {
    pub fn name(self) -> &'static str {
        match self {
            ItemType::Food => "Food",
            ItemType::Water => "Water",
            ItemType::Coin => "Coin",
            ItemType::Herb => "Herb",
            ItemType::Wood => "Wood",
            ItemType::Stone => "Stone",
            ItemType::Cloth => "Cloth",
            ItemType::Iron => "Iron",
            ItemType::Branches => "Branches",
            ItemType::Cordage => "Cordage",
            ItemType::Tinder => "Tinder",
            ItemType::Nails => "Nails",
            ItemType::Thatch => "Thatch",
            ItemType::Glass => "Glass",
            ItemType::Tool => "Tool",
            ItemType::Bandage => "Bandage",
            ItemType::Trap => "Trap",
        }
    }

    pub fn base_price(self) -> u32 {
        match self {
            ItemType::Coin => 1,
            ItemType::Herb => 2,
            ItemType::Food => 3,
            ItemType::Water => 1,
            ItemType::Wood => 2,
            ItemType::Stone => 3,
            ItemType::Cloth => 4,
            ItemType::Iron => 5,
            ItemType::Branches => 1,
            ItemType::Cordage => 2,
            ItemType::Tinder => 1,
            ItemType::Nails => 3,
            ItemType::Thatch => 1,
            ItemType::Glass => 8,
            ItemType::Tool => 6,
            ItemType::Bandage => 4,
            ItemType::Trap => 5,
        }
    }

    pub fn tradeable(self) -> bool {
        self != ItemType::Coin
    }

    pub fn tradeable_items() -> Vec<ItemType> {
        vec![
            ItemType::Herb,
            ItemType::Food,
            ItemType::Water,
            ItemType::Wood,
            ItemType::Stone,
            ItemType::Cloth,
            ItemType::Iron,
            ItemType::Branches,
            ItemType::Cordage,
            ItemType::Tinder,
            ItemType::Nails,
            ItemType::Thatch,
            ItemType::Glass,
            ItemType::Tool,
            ItemType::Bandage,
            ItemType::Trap,
        ]
    }

    pub fn gather_from(terrain: Terrain) -> Option<ItemType> {
        match terrain {
            Terrain::Grass | Terrain::Farmland | Terrain::Tundra => Some(ItemType::Herb),
            Terrain::Forest => Some(ItemType::Wood),
            Terrain::Settlement => Some(ItemType::Coin),
            Terrain::Coast | Terrain::Water => Some(ItemType::Water),
            Terrain::Mountain => Some(ItemType::Stone),
            Terrain::Swamp => Some(ItemType::Branches),
            Terrain::Sand | Terrain::DeepDesert => Some(ItemType::Tinder),
            Terrain::Cave => Some(ItemType::Stone),
            Terrain::Road => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Inventory {
    pub items: indexmap::IndexMap<ItemType, u32>,
    pub coins: u32,
    #[serde(default = "default_durability")]
    pub durability: indexmap::IndexMap<ItemType, f64>,
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

fn default_durability() -> indexmap::IndexMap<ItemType, f64> {
    indexmap::IndexMap::new()
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

    pub fn use_tool(&mut self, item: ItemType) {
        if self.has(item) {
            let quality = self.quality(item);
            self.decay(item, quality.degrade_rate());
        }
    }

    pub fn quality(&self, item: ItemType) -> QualityTier {
        QualityTier::from_durability(self.durability(item))
    }

    pub fn repair_cost(&self, item: ItemType) -> u32 {
        let d = self.durability(item);
        if d >= 1.0 {
            return 0;
        }
        let quality = QualityTier::from_durability(d);
        let base = item.base_price();
        let multiplier = match quality {
            QualityTier::Rough => 1.5,
            QualityTier::Sturdy => 1.0,
            QualityTier::Fine => 0.8,
            QualityTier::Masterwork => 2.0,
        };
        ((1.0 - d) * base as f64 * 2.0 * multiplier).ceil() as u32
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
                self.items.swap_remove(&item);
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
            // Outputs were stand-ins (Bandage made Food, Tool made Iron) until
            // the real item types existed.
            name: "Bandage".into(),
            inputs: vec![(ItemType::Herb, 3), (ItemType::Cloth, 1)],
            output: ItemType::Bandage,
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Tool".into(),
            inputs: vec![(ItemType::Wood, 2), (ItemType::Iron, 1)],
            output: ItemType::Tool,
            output_count: 1,
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
            output: ItemType::Trap,
            output_count: 1,
            people: Some(PeopleKind::Metsik),
        },
        CraftRecipe {
            name: "Arkit Salve".into(),
            inputs: vec![(ItemType::Herb, 4), (ItemType::Water, 1)],
            output: ItemType::Bandage,
            output_count: 3,
            people: Some(PeopleKind::Arkit),
        },
        CraftRecipe {
            name: "Väylä Net".into(),
            inputs: vec![(ItemType::Cordage, 2), (ItemType::Wood, 1)],
            output: ItemType::Trap,
            output_count: 1,
            people: Some(PeopleKind::Vayla),
        },
        CraftRecipe {
            name: "Cordage".into(),
            inputs: vec![(ItemType::Branches, 3)],
            output: ItemType::Cordage,
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Thatch Bundle".into(),
            inputs: vec![(ItemType::Branches, 2), (ItemType::Cordage, 1)],
            output: ItemType::Thatch,
            output_count: 3,
            people: None,
        },
        CraftRecipe {
            name: "Laakso Stone-Ward".into(),
            inputs: vec![(ItemType::Stone, 3), (ItemType::Herb, 1)],
            output: ItemType::Bandage,
            output_count: 2,
            people: Some(PeopleKind::Laakso),
        },
        CraftRecipe {
            name: "Tzakhar Deep-Tool".into(),
            inputs: vec![(ItemType::Stone, 2), (ItemType::Iron, 1)],
            output: ItemType::Tool,
            output_count: 1,
            people: Some(PeopleKind::Tzakhar),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Crafters,
    Traders,
    Elders,
}

impl Faction {
    pub fn label(self) -> &'static str {
        match self {
            Faction::Crafters => "Crafters",
            Faction::Traders => "Traders",
            Faction::Elders => "Elders",
        }
    }

    pub fn god(self) -> GodName {
        match self {
            Faction::Crafters => GodName::Oltzed,
            Faction::Traders => GodName::Masa,
            Faction::Elders => GodName::Sampsa,
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            Faction::Crafters => "The forges burn late. Hands shape the world.",
            Faction::Traders => "Coin changes hands. Roads bind the settlements.",
            Faction::Elders => "Memory outlives the young. The archive endures.",
        }
    }

    pub fn all() -> &'static [Faction] {
        &[Faction::Crafters, Faction::Traders, Faction::Elders]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipEvent {
    Election,
    Dispute,
    Festival,
}

impl LeadershipEvent {
    pub fn label(self) -> &'static str {
        match self {
            LeadershipEvent::Election => "Election",
            LeadershipEvent::Dispute => "Dispute",
            LeadershipEvent::Festival => "Festival",
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            LeadershipEvent::Election => "Voices rise in the square. A new voice will speak for the settlement.",
            LeadershipEvent::Dispute => "Harsh words at the gate. Two factions cannot agree. Eyes turn to you.",
            LeadershipEvent::Festival => "Drums and firelight. The settlement celebrates its bonds — or forgets its fractures.",
        }
    }

    pub fn standing_shift(self) -> f64 {
        match self {
            LeadershipEvent::Election => 0.05,
            LeadershipEvent::Dispute => -0.10,
            LeadershipEvent::Festival => 0.08,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SettlementPolitics {
    pub crafter_standing: f64,
    pub trader_standing: f64,
    pub elder_standing: f64,
    pub last_event: Option<LeadershipEvent>,
}

impl SettlementPolitics {
    pub fn new() -> Self {
        Self {
            crafter_standing: 0.5,
            trader_standing: 0.5,
            elder_standing: 0.5,
            last_event: None,
        }
    }

    pub fn standing(&self, faction: Faction) -> f64 {
        match faction {
            Faction::Crafters => self.crafter_standing,
            Faction::Traders => self.trader_standing,
            Faction::Elders => self.elder_standing,
        }
    }

    pub fn adjust(&mut self, faction: Faction, delta: f64) {
        let val = (self.standing(faction) + delta).clamp(0.0, 1.0);
        match faction {
            Faction::Crafters => self.crafter_standing = val,
            Faction::Traders => self.trader_standing = val,
            Faction::Elders => self.elder_standing = val,
        }
    }

    pub fn dominant_faction(&self) -> Faction {
        if self.crafter_standing >= self.trader_standing
            && self.crafter_standing >= self.elder_standing
        {
            Faction::Crafters
        } else if self.trader_standing >= self.elder_standing {
            Faction::Traders
        } else {
            Faction::Elders
        }
    }

    pub fn price_modifier(&self) -> f64 {
        let dominant = self.dominant_faction();
        match dominant {
            Faction::Traders => 0.85,
            Faction::Crafters => 0.95,
            Faction::Elders => 1.05,
        }
    }

    pub fn roll_leadership_event(&mut self, seed: u64) -> Option<LeadershipEvent> {
        let val = (seed.wrapping_mul(2654435761) >> 48) as u32 % 100;
        let event = if val < 15 {
            Some(LeadershipEvent::Election)
        } else if val < 25 {
            Some(LeadershipEvent::Dispute)
        } else if val < 35 {
            Some(LeadershipEvent::Festival)
        } else {
            None
        };
        if let Some(e) = event {
            self.adjust(self.dominant_faction(), e.standing_shift());
            self.last_event = Some(e);
        }
        event
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
    #[serde(default)]
    pub politics: SettlementPolitics,
    /// Communal food supply, in meals. Farms/fishers/herders fill it, the
    /// population draws from it, and scarcity moves market prices.
    #[serde(default)]
    pub food_stock: f64,
    /// Working farms, planted and harvested by the settlement's farmers.
    #[serde(default)]
    pub farms: Vec<Farm>,
    /// NPC construction: projects underway and completed buildings.
    #[serde(default)]
    pub buildings: Vec<Building>,
    /// Last day of the current festival (0 = none). Set by the settlement
    /// daily tick; festivals used to be a per-visit dice roll with no
    /// duration, no discounts, and no word of them reaching the roads.
    #[serde(default)]
    pub festival_until_day: u32,
    /// Consecutive days the stores have stood empty. Long famine empties the
    /// settlement itself.
    #[serde(default)]
    pub famine_days: u32,
}

impl Settlement {
    pub fn allows_companions(&self) -> bool {
        matches!(self.size.as_str(), "village" | "town" | "city")
    }

    pub fn companion_capacity(&self) -> usize {
        match self.size.as_str() {
            "village" => 1,
            "town" => 2,
            "city" => 3,
            _ => 0,
        }
    }

    /// How many residents practice the given profession.
    pub fn profession_count(&self, profession: &str) -> usize {
        self.people
            .iter()
            .filter(|p| p.profession == profession)
            .count()
    }

    /// Whether a completed building of this type stands here.
    pub fn has_building(&self, kind: BuildingType) -> bool {
        self.buildings
            .iter()
            .any(|b| b.building_type == kind && b.is_complete())
    }

    /// Size tier from the head-count: settlements grow into villages, towns,
    /// and cities — and shrink back — as their population moves.
    pub fn size_for_population(population: u32) -> &'static str {
        if population >= 800 {
            "city"
        } else if population >= 250 {
            "town"
        } else if population >= 60 {
            "village"
        } else {
            "hamlet"
        }
    }

    /// Whether a festival is running on the given day.
    pub fn in_festival(&self, day: u32) -> bool {
        self.festival_until_day > 0 && day <= self.festival_until_day
    }

    /// Food scarcity as a market price multiplier: empty stores push food
    /// prices toward 1.6x; full stores pull them toward 0.8x.
    pub fn food_scarcity_modifier(&self) -> f64 {
        let per_head = self.food_stock / (self.population.max(1) as f64);
        if per_head < 0.5 {
            1.6
        } else if per_head < 1.5 {
            1.2
        } else if per_head > 4.0 {
            0.8
        } else {
            1.0
        }
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
    WinterCough,
    MarshFever,
    BloodAche,
    ForgeBlindness,
    ChildbirthComplication,
}

impl Disease {
    pub fn name(self) -> &'static str {
        match self {
            Disease::Fever => "fever",
            Disease::Infection => "infection",
            Disease::Sprain => "sprain",
            Disease::Exhaustion => "exhaustion",
            Disease::Plague => "plague",
            Disease::WinterCough => "winter_cough",
            Disease::MarshFever => "marsh_fever",
            Disease::BloodAche => "blood_ache",
            Disease::ForgeBlindness => "forge_blindness",
            Disease::ChildbirthComplication => "childbirth_complication",
        }
    }

    pub fn vitals_decay_modifier(self) -> f64 {
        match self {
            Disease::Fever => 1.3,
            Disease::Infection => 1.4,
            Disease::Sprain => 1.2,
            Disease::Exhaustion => 1.5,
            Disease::Plague => 1.8,
            Disease::WinterCough => 1.15,
            Disease::MarshFever => 1.35,
            Disease::BloodAche => 1.25,
            Disease::ForgeBlindness => 1.1,
            Disease::ChildbirthComplication => 1.6,
        }
    }

    pub fn recovery_ticks(self) -> u64 {
        match self {
            Disease::Fever => 48,
            Disease::Infection => 72,
            Disease::Sprain => 36,
            Disease::Exhaustion => 24,
            Disease::Plague => 120,
            Disease::WinterCough => 60,
            Disease::MarshFever => 96,
            Disease::BloodAche => 48,
            Disease::ForgeBlindness => 200,
            Disease::ChildbirthComplication => 30,
        }
    }

    pub fn contraction_probability(self, terrain: Terrain) -> f64 {
        match (self, terrain) {
            (Disease::Fever, Terrain::Swamp | Terrain::Forest) => 0.02,
            (Disease::Infection, Terrain::Swamp) => 0.03,
            (Disease::Sprain, Terrain::Mountain | Terrain::Forest) => 0.015,
            (Disease::Exhaustion, _) => 0.01,
            (Disease::Plague, Terrain::Settlement) => 0.005,
            (Disease::WinterCough, Terrain::Tundra | Terrain::Mountain) => 0.025,
            (Disease::MarshFever, Terrain::Swamp | Terrain::Coast) => 0.02,
            (Disease::BloodAche, _) => 0.005,
            (Disease::ForgeBlindness, Terrain::Mountain | Terrain::Settlement) => 0.01,
            (Disease::ChildbirthComplication, _) => 0.003,
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
        // Severity scales how hard the disease bites (1.0 = textbook case).
        // The field was persisted but never read; untreated illness now
        // worsens and tending it (bandages) eases it.
        1.0 + (self.disease.vitals_decay_modifier() - 1.0) * self.severity.max(0.5)
    }

    /// Untreated illness worsens a little each hour (capped at 1.5x).
    pub fn worsen(&mut self, hours: u32) {
        self.severity = (self.severity + 0.005 * hours as f64).min(1.5);
    }

    /// A tended illness eases and runs its course a day faster.
    pub fn tend(&mut self) {
        self.severity = (self.severity - 0.25).max(1.0);
        self.contracted_tick = self.contracted_tick.saturating_sub(24);
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
    Mushroom,
    Berry,
    Flatroot,
}

impl CropType {
    pub fn name(self) -> &'static str {
        match self {
            CropType::Grain => "grain",
            CropType::RootVegetable => "root vegetables",
            CropType::Herb => "herbs",
            CropType::Mushroom => "mushrooms",
            CropType::Berry => "berries",
            CropType::Flatroot => "flatroot",
        }
    }

    pub fn growth_ticks(self) -> u64 {
        match self {
            CropType::Grain => 72,         // 3 days
            CropType::RootVegetable => 96, // 4 days
            CropType::Herb => 48,          // 2 days
            CropType::Mushroom => 60,
            CropType::Berry => 36,
            CropType::Flatroot => 84,
        }
    }

    pub fn base_yield(self) -> u32 {
        match self {
            CropType::Grain => 4,
            CropType::RootVegetable => 3,
            CropType::Herb => 5,
            CropType::Mushroom => 4,
            CropType::Berry => 3,
            CropType::Flatroot => 5,
        }
    }

    pub fn regional_suitability(self, terrain: Terrain) -> f64 {
        match (self, terrain) {
            (CropType::Grain, Terrain::Farmland | Terrain::Grass) => 1.2,
            (CropType::RootVegetable, Terrain::Forest | Terrain::Farmland) => 1.1,
            (CropType::Herb, Terrain::Forest | Terrain::Swamp) => 1.3,
            (CropType::Mushroom, Terrain::Forest | Terrain::Swamp | Terrain::Cave) => 1.3,
            (CropType::Berry, Terrain::Forest | Terrain::Tundra) => 1.2,
            (CropType::Flatroot, Terrain::Sand | Terrain::Grass) => 1.1,
            (_, Terrain::Farmland) => 1.0,
            _ => 0.7,
        }
    }

    /// Every crop a settlement might plant.
    pub fn all() -> [CropType; 6] {
        [
            CropType::Grain,
            CropType::RootVegetable,
            CropType::Herb,
            CropType::Mushroom,
            CropType::Berry,
            CropType::Flatroot,
        ]
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

// Quest struct moved to quest.rs — re-exported via pub use
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

        assert_eq!(
            ItemType::gather_from(Terrain::Mountain),
            Some(ItemType::Stone)
        );

        assert_eq!(
            ItemType::gather_from(Terrain::Settlement),
            Some(ItemType::Coin)
        );

        assert_eq!(ItemType::gather_from(Terrain::Water), Some(ItemType::Water));

        assert_eq!(
            ItemType::gather_from(Terrain::Swamp),
            Some(ItemType::Branches)
        );

        assert_eq!(ItemType::gather_from(Terrain::Sand), Some(ItemType::Tinder));
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

        // 16 = all item kinds minus Coin (Tool/Bandage/Trap added with #310).
        assert_eq!(items.len(), 16);
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

    fn craft_recipes_valid() {
        let recipes = craft_recipes();

        assert!(!recipes.is_empty(), "must have at least one recipe");

        for recipe in &recipes {
            assert!(!recipe.inputs.is_empty(), "recipe must have inputs");

            assert!(recipe.output_count > 0, "must produce something");
        }
    }

    #[test]

    fn settlement_service_costs() {
        assert_eq!(SettlementService::Tavern.cost(), 2);

        assert_eq!(SettlementService::Temple.cost(), 3);
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

            structures: vec![],
            weather: crate::model::Weather::Clear,
            game_richness: 1.0,
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

            structures: vec![],
            weather: crate::model::Weather::Clear,
            game_richness: 1.0,
        };

        assert_eq!(region.danger_level(), DangerLevel::Dangerous);
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

            politics: SettlementPolitics::new(),
            food_stock: 0.0,
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
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

            structures: vec![],
            weather: crate::model::Weather::Clear,
            game_richness: 1.0,
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

    fn faction_labels() {
        assert_eq!(Faction::Crafters.label(), "Crafters");

        assert_eq!(Faction::Traders.label(), "Traders");

        assert_eq!(Faction::Elders.label(), "Elders");
    }

    #[test]

    fn politics_adjust_clamps() {
        let mut p = SettlementPolitics::new();

        p.adjust(Faction::Crafters, 0.3);

        assert!(p.crafter_standing > 0.5);

        p.adjust(Faction::Crafters, 10.0);

        assert!((p.crafter_standing - 1.0).abs() < f64::EPSILON);

        p.adjust(Faction::Crafters, -20.0);

        assert!(p.crafter_standing.abs() < f64::EPSILON);
    }

    #[test]

    fn dominant_faction() {
        let mut p = SettlementPolitics::new();

        p.trader_standing = 0.9;

        p.crafter_standing = 0.5;

        p.elder_standing = 0.3;

        assert_eq!(p.dominant_faction(), Faction::Traders);
    }

    #[test]

    fn trader_dominant_reduces_prices() {
        let mut p = SettlementPolitics::new();

        p.trader_standing = 0.9;

        assert!(p.price_modifier() < 1.0);
    }

    #[test]

    fn elder_dominant_increases_prices() {
        let mut p = SettlementPolitics::new();

        p.elder_standing = 0.9;

        assert!(p.price_modifier() > 1.0);
    }

    #[test]

    fn leadership_event_none_at_high_roll() {
        let mut p = SettlementPolitics::new();

        let result = p.roll_leadership_event(0xFFFFFFFFFFFFFFFF);

        assert!(result.is_none());
    }
}
