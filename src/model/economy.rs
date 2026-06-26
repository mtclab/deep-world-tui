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

    /// The durability a freshly-made piece of this quality starts at (#547):
    /// the inverse of `from_durability`, so a crafted masterwork reads back as
    /// masterwork and lasts the longest, a rough piece the least.
    pub fn starting_durability(self) -> f64 {
        match self {
            QualityTier::Masterwork => 0.98,
            QualityTier::Fine => 0.82,
            QualityTier::Sturdy => 0.55,
            QualityTier::Rough => 0.28,
        }
    }

    /// What the work fetches at market, relative to a plain sturdy piece (#547):
    /// a masterwork sells dearer, a rough piece cheap.
    pub fn sell_multiplier(self) -> f64 {
        match self {
            QualityTier::Masterwork => 1.3,
            QualityTier::Fine => 1.12,
            QualityTier::Sturdy => 1.0,
            QualityTier::Rough => 0.8,
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
    /// A wild creature's hide, taken by hunting or trapping. Tans to leather,
    /// trades well, the start of the warm-coat chain.
    Hide,
    /// Tanned hide — the intermediate between a raw skin and a worn coat.
    Leather,
    /// A warm coat of leather and cloth: worn, it softens the bite of harsh
    /// weather. Wears with the seasons.
    Coat,
    /// A herb poultice: applied at rest, it speeds an infection or venom
    /// through its course faster than a plain bandage.
    Salve,
    /// Riverbank and shore clay, dug raw. The potter's stock — fired into
    /// pottery (#671).
    Clay,
    /// Fired earthenware: jars, crocks, bowls. The vessels a settled people
    /// store and carry their stores in; trades well.
    Pottery,
    /// Charcoal, burnt down from wood in a slow mound. The hot, clean fuel a
    /// forge wants and a cold hearth is glad of.
    Charcoal,
    /// Ale, brewed from grain and bittered with herb. The tavern's comfort and
    /// a keeping-store of a good harvest.
    Ale,
    /// A data-defined trade good (#678): the long tail of the economy — salt,
    /// amber, silk, lamp-oil, the thousand wares a civilization moves that need
    /// no special code behaviour. Bought, carried, priced, and sold through this
    /// one variant; defined in `data/goods.ron`. Serialises as a stable slug.
    Good(crate::model::goods::GoodId),
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
            ItemType::Hide => "Hide",
            ItemType::Leather => "Leather",
            ItemType::Coat => "Coat",
            ItemType::Salve => "Salve",
            ItemType::Clay => "Clay",
            ItemType::Pottery => "Pottery",
            ItemType::Charcoal => "Charcoal",
            ItemType::Ale => "Ale",
            ItemType::Good(g) => g.name(),
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
            ItemType::Hide => 6,
            ItemType::Leather => 9,
            ItemType::Coat => 18,
            ItemType::Salve => 6,
            ItemType::Clay => 2,
            ItemType::Pottery => 7,
            ItemType::Charcoal => 3,
            ItemType::Ale => 4,
            ItemType::Good(g) => g.price(),
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
            ItemType::Hide,
            ItemType::Leather,
            ItemType::Coat,
            ItemType::Salve,
            ItemType::Clay,
            ItemType::Pottery,
            ItemType::Charcoal,
            ItemType::Ale,
        ]
    }

    pub fn gather_from(terrain: Terrain) -> Option<ItemType> {
        match terrain {
            Terrain::Grass | Terrain::Farmland | Terrain::Tundra => Some(ItemType::Herb),
            Terrain::Forest => Some(ItemType::Wood),
            Terrain::Settlement => Some(ItemType::Coin),
            Terrain::House | Terrain::Wall | Terrain::Floor | Terrain::Door | Terrain::Hearth => {
                None
            }
            // Shore and riverbank clay is dug along the coast (#671); open
            // water still gives only water to drink.
            Terrain::Coast => Some(ItemType::Clay),
            Terrain::Water => Some(ItemType::Water),
            Terrain::Mountain => Some(ItemType::Stone),
            Terrain::Swamp => Some(ItemType::Branches),
            Terrain::Sand | Terrain::Steppe => Some(ItemType::Tinder),
            Terrain::Cave => Some(ItemType::Stone),
            Terrain::Road => None,
        }
    }
}

/// The trade good a profession plies its craft to make (per-agent economy #54,
/// slice 1) — what a working smith, weaver, or miner actually puts on the shelf.
/// `None` for trades that make no tradeable good of their own (labourers, guards,
/// farmers — whose yield is food, handled by the granary).
pub fn trade_good(profession: &str) -> Option<ItemType> {
    Some(match profession {
        "smith" => ItemType::Tool,
        "weaver" => ItemType::Cloth,
        "miner" => ItemType::Iron,
        "carpenter" => ItemType::Wood,
        "mason" => ItemType::Stone,
        "potter" => ItemType::Pottery,
        "brewer" => ItemType::Ale,
        "tanner" => ItemType::Leather,
        "herder" => ItemType::Hide,
        "forester" => ItemType::Charcoal,
        "healer" => ItemType::Bandage,
        _ => return None,
    })
}

/// The craft-sense (#441 gift) that masters a trade — so a gifted producer of the
/// matching sense makes more of its good (per-agent economy #54, slice 2). `None`
/// for a trade no sense governs (its gifted are no better than its journeymen).
pub fn profession_craft_sense(profession: &str) -> Option<crate::model::gift::CraftSense> {
    use crate::model::gift::CraftSense;
    Some(match profession {
        "smith" | "miner" | "mason" => CraftSense::IronEar,
        "forester" | "herbalist" | "forager" | "farmer" | "herder" => CraftSense::RootEye,
        "fisher" | "sailor" | "trader" => CraftSense::ScaleHand,
        "healer" | "scribe" | "priest" => CraftSense::StillSense,
        _ => return None,
    })
}

/// What a day's labour at a trade is worth (per-agent economy #54, slice 1): a
/// skilled craft or a specialist earns more than common labour. A multiplier on
/// the base wage — so an agent's coin reflects the worth of what it actually does.
pub fn trade_wage(profession: &str) -> u32 {
    if trade_good(profession).is_some()
        || matches!(profession, "scribe" | "trader" | "priest" | "fisher")
    {
        2
    } else {
        1
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

    /// Add `count` of an item at a given starting `durability` (#547 craft
    /// quality): the new pieces blend into any existing stack by count — a
    /// masterwork mixed into a worn lot raises the lot's average, a rough piece
    /// lowers it. (The inventory keeps one durability per type, so a stack's
    /// quality is its weighted average, not per-piece.)
    pub fn add_with_quality(&mut self, item: ItemType, count: u32, durability: f64) {
        let old_count = self.get(item) as f64;
        let old_dur = self.durability(item);
        *self.items.entry(item).or_insert(0) += count;
        let total = old_count + count as f64;
        let blended = if total > 0.0 {
            (old_dur * old_count + durability.clamp(0.0, 1.0) * count as f64) / total
        } else {
            durability
        };
        self.durability.insert(item, blended);
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
        // The hide -> leather -> coat chain (#414): hunting feeds the warmth
        // that softens harsh-weather decay; a salve closes the luck-body loop.
        // Appended after Tool so existing recipe indices stay put.
        CraftRecipe {
            name: "Leather".into(),
            inputs: vec![(ItemType::Hide, 2)],
            output: ItemType::Leather,
            output_count: 1,
            people: None,
        },
        CraftRecipe {
            name: "Warm Coat".into(),
            inputs: vec![(ItemType::Leather, 2), (ItemType::Cloth, 1)],
            output: ItemType::Coat,
            output_count: 1,
            people: None,
        },
        CraftRecipe {
            name: "Salve".into(),
            inputs: vec![(ItemType::Herb, 3)],
            output: ItemType::Salve,
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
        // Filling the craftable gaps (#529): Nails, Tinder, and Glass had no
        // recipe — buyable only. Now the bench makes them.
        CraftRecipe {
            // A bar of iron drawn and cut to nails — the smith's bread work.
            name: "Nails".into(),
            inputs: vec![(ItemType::Iron, 1)],
            output: ItemType::Nails,
            output_count: 4,
            people: None,
        },
        CraftRecipe {
            // Deadfall split fine and dried — the fire's first food.
            name: "Tinder".into(),
            inputs: vec![(ItemType::Branches, 2)],
            output: ItemType::Tinder,
            output_count: 3,
            people: None,
        },
        CraftRecipe {
            // The Mëräk make deep-glass at the tideline — sand-stone fused in a
            // sea-coal fire, a craft no upworld kiln has matched (canon).
            name: "Mëräk Deep-Glass".into(),
            inputs: vec![(ItemType::Stone, 3), (ItemType::Tinder, 1)],
            output: ItemType::Glass,
            output_count: 1,
            people: Some(PeopleKind::Merak),
        },
        // New chains for a deeper shelf (#671 slice 2): a settled people fires
        // clay into pots, burns wood to charcoal, brews grain into ale.
        CraftRecipe {
            // Clay thrown and fired — the jars and crocks a larder needs.
            name: "Pottery".into(),
            inputs: vec![(ItemType::Clay, 3)],
            output: ItemType::Pottery,
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            // Wood banked under turf and burnt slow — the forge's clean fuel.
            name: "Charcoal".into(),
            inputs: vec![(ItemType::Wood, 3)],
            output: ItemType::Charcoal,
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            // Grain mashed and bittered with herb — the tavern's comfort.
            name: "Ale".into(),
            inputs: vec![(ItemType::Food, 2), (ItemType::Herb, 1)],
            output: ItemType::Ale,
            output_count: 2,
            people: None,
        },
        // Registry-good craft chains (#678 slice 3c): the player can work the
        // long tail by hand, not only buy or forage it. Multi-step where the
        // world's chains are — grain to flour to bread, ore to bronze, fleece
        // to thread to linen.
        CraftRecipe {
            name: "Flour".into(),
            inputs: vec![(rg("grain"), 3)],
            output: rg("flour"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Bread".into(),
            inputs: vec![(rg("flour"), 2)],
            output: rg("bread"),
            output_count: 3,
            people: None,
        },
        CraftRecipe {
            name: "Bronze".into(),
            inputs: vec![(rg("copper"), 2), (rg("tin"), 1)],
            output: rg("bronze"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Thread".into(),
            inputs: vec![(rg("wool"), 2)],
            output: rg("thread"),
            output_count: 3,
            people: None,
        },
        CraftRecipe {
            name: "Linen".into(),
            inputs: vec![(rg("thread"), 3)],
            output: rg("linen"),
            output_count: 1,
            people: None,
        },
        CraftRecipe {
            name: "Rope".into(),
            inputs: vec![(rg("hemp"), 3)],
            output: rg("rope"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Candle".into(),
            inputs: vec![(rg("tallow"), 2)],
            output: rg("candle"),
            output_count: 3,
            people: None,
        },
        CraftRecipe {
            name: "Soap".into(),
            inputs: vec![(rg("tallow"), 1), (rg("lime"), 1)],
            output: rg("soap"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Mead".into(),
            inputs: vec![(rg("honey"), 2)],
            output: rg("mead"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Vinegar".into(),
            inputs: vec![(rg("wine"), 1)],
            output: rg("vinegar"),
            output_count: 2,
            people: None,
        },
        // ── Variety pass (GOODS.md): the named grains all mill to flour, so a
        // rye plain and a wheat south feed the same baker — different inputs,
        // one product. Plus the dairy, brewing, must, fish and timber chains
        // that give the new wares a source and a sink (not just a price).
        CraftRecipe {
            name: "Mill Rye".into(),
            inputs: vec![(rg("rye"), 3)],
            output: rg("flour"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Mill Wheat".into(),
            inputs: vec![(rg("wheat"), 3)],
            output: rg("flour"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Mill Barley".into(),
            inputs: vec![(rg("barley"), 3)],
            output: rg("flour"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Rye Bread".into(),
            inputs: vec![(rg("flour"), 2)],
            output: rg("rye-bread"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Crispbread".into(),
            inputs: vec![(rg("flour"), 2)],
            output: rg("crispbread"),
            output_count: 3,
            people: None,
        },
        CraftRecipe {
            name: "Malt".into(),
            inputs: vec![(rg("barley"), 2)],
            output: rg("malt"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Brew Sahti".into(),
            inputs: vec![(rg("malt"), 2), (ItemType::Herb, 1)],
            output: rg("sahti"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Press Curd".into(),
            inputs: vec![(rg("milk"), 3)],
            output: rg("curd"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Goat Cheese".into(),
            inputs: vec![(rg("goat-milk"), 3)],
            output: rg("goat-cheese"),
            output_count: 1,
            people: None,
        },
        CraftRecipe {
            name: "Press Must".into(),
            inputs: vec![(rg("grape"), 3)],
            output: rg("must"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Vintage".into(),
            inputs: vec![(rg("must"), 2)],
            output: rg("wine"),
            output_count: 1,
            people: None,
        },
        CraftRecipe {
            name: "Try Fish-Oil".into(),
            inputs: vec![(rg("fish"), 3)],
            output: rg("fish-oil"),
            output_count: 2,
            people: None,
        },
        CraftRecipe {
            name: "Saw Planks".into(),
            inputs: vec![(ItemType::Wood, 2)],
            output: rg("pine-plank"),
            output_count: 3,
            people: None,
        },
    ]
}

/// A registry trade good as an `ItemType`, by slug — for recipe tables. Every
/// slug here is authored in `data/goods.ron`, so the lookup cannot miss.
fn rg(slug: &str) -> ItemType {
    ItemType::Good(crate::model::good_id(slug).expect("recipe names a known good"))
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

    /// A single-width sign to paint over a service building's door on the map,
    /// so the tavern, the temple, the forge can be told from a plain home and
    /// each other from the street — without knocking on every door (#458). The
    /// emoji `glyph` is double-width and would break the tile grid; these are
    /// monospace-safe.
    pub fn map_sign(self) -> char {
        match self {
            SettlementService::Tavern => 'T',
            SettlementService::Temple => 'C',
            SettlementService::Forge => 'F',
            SettlementService::Hearth => 'H',
            SettlementService::TrapWorkshop => 'W',
            SettlementService::Archive => 'A',
            SettlementService::TradePost => '$',
            SettlementService::Shrine => 'S',
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

/// The in-kind barter an enclave of the Five offers for a good you lay down
/// (#454): the same fixed, coin-free rates as their roadside traders — they
/// take no coin and do not haggle, the rate is the rate. Returns how many of
/// the offered good it costs and what you get for it, or `None` if these people
/// want nothing of what you offer. Goods only — never `Coin`.
pub fn enclave_barter(
    people: PeopleKind,
    offered: ItemType,
) -> Option<(u32, Vec<(ItemType, u32)>)> {
    use ItemType as I;
    let deal: (u32, &[(ItemType, u32)]) = match (people, offered) {
        // The Khör give härkä-leather and steppe-butter for metal.
        (PeopleKind::Khor, I::Tool) => (1, &[(I::Hide, 2), (I::Food, 2)]),
        (PeopleKind::Khor, I::Iron) => (1, &[(I::Hide, 2), (I::Food, 1)]),
        // The Mëräk give deep-fish and deep-glass for surface make.
        (PeopleKind::Merak, I::Tool) => (1, &[(I::Food, 3), (I::Glass, 1)]),
        (PeopleKind::Merak, I::Cloth) => (1, &[(I::Food, 2)]),
        // The Tzäkhar give worked metal for surface food (they take two).
        (PeopleKind::Tzakhar, I::Food) => (2, &[(I::Iron, 2), (I::Tool, 1)]),
        (PeopleKind::Tzakhar, I::Herb) => (2, &[(I::Iron, 1)]),
        // The Häl bring canopy physic and fruit down for cloth and tools.
        (PeopleKind::Hal, I::Tool) => (1, &[(I::Salve, 1), (I::Herb, 2), (I::Food, 1)]),
        (PeopleKind::Hal, I::Cloth) => (1, &[(I::Herb, 2), (I::Food, 2)]),
        // The She'ar give steppe game and succulent-physic for what wards the sun.
        (PeopleKind::Shear, I::Cloth) => (1, &[(I::Food, 2), (I::Herb, 1)]),
        (PeopleKind::Shear, I::Tool) => (1, &[(I::Food, 2), (I::Herb, 2)]),
        _ => return None,
    };
    Some((deal.0, deal.1.to_vec()))
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

    /// Drift the faction standings toward the town's character (#556 living
    /// politics): each standing steps toward its share of the three pulls — a
    /// town that makes much lifts the Crafters, one that trades and prospers the
    /// Traders, an old and stable one the Elders. Pure and deterministic; the
    /// standings converge on the town's nature over time, so a real dominant
    /// faction emerges instead of the frozen 0.5/0.5/0.5.
    pub fn drift_toward(
        &mut self,
        crafter_pull: f64,
        trader_pull: f64,
        elder_pull: f64,
        rate: f64,
    ) {
        let sum = (crafter_pull + trader_pull + elder_pull).max(1e-9);
        let tc = crafter_pull / sum;
        let tt = trader_pull / sum;
        let te = elder_pull / sum;
        self.crafter_standing =
            (self.crafter_standing + rate * (tc - self.crafter_standing)).clamp(0.0, 1.0);
        self.trader_standing =
            (self.trader_standing + rate * (tt - self.trader_standing)).clamp(0.0, 1.0);
        self.elder_standing =
            (self.elder_standing + rate * (te - self.elder_standing)).clamp(0.0, 1.0);
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

    /// How readily the town opens its roads to the rest of the province (#560
    /// living province): a Traders council throws the gates wide — caravans go
    /// out more often and partnerships form faster — while an Elders council
    /// keeps to itself. Scales a town's outbound trade and how strongly its
    /// caravans deepen a tie. Weighted by how firmly the faction actually holds,
    /// so a contested council moves the needle less than a settled one.
    pub fn openness(&self) -> f64 {
        let (target, hold) = match self.dominant_faction() {
            Faction::Traders => (1.6, self.trader_standing),
            Faction::Crafters => (1.0, self.crafter_standing),
            Faction::Elders => (0.5, self.elder_standing),
        };
        // Pull from the neutral 1.0 toward the faction's target by how firmly it
        // sits (a third-share is no grip at all; a clear majority, full grip).
        let grip = ((hold - 0.34) / 0.5).clamp(0.0, 1.0);
        1.0 + (target - 1.0) * grip
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

/// A settlement's living devotion (#595): how the town's faith is shared among
/// the Five. Seeded from its people's patron god, it drifts in the daily sim
/// toward the town's character and its holy days — belief that breathes, instead
/// of a fixed patron. The mirror, for faith, of `SettlementPolitics`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SettlementFaith {
    /// Per-god devotion weight; the prevailing god is the strongest. Empty until
    /// first seeded/drifted, when it fills to all Five.
    #[serde(default)]
    pub devotion: std::collections::HashMap<GodName, f64>,
    /// The last prevailing god the world announced for this town (#595), so a
    /// turn of faith is talked of once, not every day.
    #[serde(default)]
    pub announced: Option<GodName>,
}

impl SettlementFaith {
    /// A faith seeded toward a patron god — most of the town keeps it, the rest
    /// spread thin among the others.
    pub fn seeded(patron: GodName) -> Self {
        let mut devotion = std::collections::HashMap::new();
        for g in GodName::all() {
            devotion.insert(g, if g == patron { 0.4 } else { 0.15 });
        }
        Self {
            devotion,
            announced: None,
        }
    }

    pub fn get(&self, god: GodName) -> f64 {
        self.devotion.get(&god).copied().unwrap_or(0.0)
    }

    /// The town's prevailing god — the strongest devotion, by a fixed god order
    /// so a tie resolves the same way every run (determinism). `None` only for a
    /// faith never seeded.
    pub fn prevailing(&self) -> Option<GodName> {
        let mut best: Option<(GodName, f64)> = None;
        for g in GodName::all() {
            let v = self.get(g);
            if v > 0.0 && best.map(|(_, b)| v > b).unwrap_or(true) {
                best = Some((g, v));
            }
        }
        best.map(|(g, _)| g)
    }

    /// A town whose faith is split — its two strongest gods run near-even, each
    /// with real weight (#614). Such a town is ripe for a schism, and the world
    /// posts a call for a devotee to come and steady it. Deterministic.
    pub fn is_contested(&self) -> bool {
        let mut weights: Vec<f64> = GodName::all().iter().map(|&g| self.get(g)).collect();
        weights.sort_by(|a, b| b.partial_cmp(a).unwrap());
        weights[0] >= 0.28 && weights[1] >= 0.24 && (weights[0] - weights[1]) <= 0.06
    }

    /// Drift the devotion a little toward a god (#595): the target rises, the
    /// rest ease back, and the whole is renormalised so devotion is always a
    /// share of one. Fills to all Five on first touch. Pure and deterministic.
    pub fn drift_toward(&mut self, target: GodName, rate: f64) {
        for g in GodName::all() {
            self.devotion.entry(g).or_insert(0.2);
        }
        for g in GodName::all() {
            let tgt = if g == target { 1.0 } else { 0.0 };
            let e = self.devotion.get_mut(&g).unwrap();
            *e += rate * (tgt - *e);
        }
        let sum: f64 = self.devotion.values().sum();
        if sum > 0.0 {
            for v in self.devotion.values_mut() {
                *v /= sum;
            }
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
    #[serde(default)]
    pub politics: SettlementPolitics,
    /// The town's living devotion (#595): how its faith is shared among the
    /// Five, drifting in the daily sim. Empty until first drifted, when it
    /// seeds from the people's patron.
    #[serde(default)]
    pub faith: SettlementFaith,
    /// Communal food supply, in meals. Farms/fishers/herders fill it, the
    /// population draws from it, and scarcity moves market prices.
    #[serde(default)]
    pub food_stock: f64,
    /// The town's common purse (entity-first slice 4, deep-world-godot#50): the
    /// market's takings and the wage-fund. Buying a meal pays into it; taking
    /// work draws a wage out of it, so coin is **conserved** — the sum of every
    /// resident's purse and this treasury holds steady under ordinary trade. A
    /// town whose treasury runs dry can offer no work, and its broke go
    /// desperate. Seeded at worldgen, scaled to population. Default 0 so older
    /// saves load (topped up on load like the rosters).
    #[serde(default)]
    pub treasury: u32,
    /// Trade goods the settlement's own crafts have made and not yet spent or
    /// sold (#540 living economy): a goods analogue of `food_stock`. Produced by
    /// the trades in the daily sim, capped by what the place can hold. A good a
    /// town makes in plenty is cheap there; one it lacks is dear.
    #[serde(default)]
    pub goods_stock: std::collections::HashMap<ItemType, f64>,
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
    /// Days a plague has gripped the town (#604): 0 when healthy. While it
    /// burns it sickens the people and takes a toll, then runs its course.
    #[serde(default)]
    pub plague_days: u32,
    /// Anchor tile (top-left of the footprint) on the region map. A
    /// settlement is not a point: its footprint scales with its size and is
    /// painted as Settlement terrain, so a town LOOKS like a town.
    #[serde(default)]
    pub map_x: u32,
    #[serde(default)]
    pub map_y: u32,
    /// The district edge actually painted on the map (0 = not yet laid).
    /// Kept so tile containment always matches the drawn town even while
    /// the population (and so the wanted footprint) moves.
    #[serde(default)]
    pub district: u32,
    /// A lasting deed the town remembers the player by (#565 the player's mark):
    /// the stranger who kept them fed through a lean year. Set when the player
    /// provisions a town in famine; surfaced in its talk and on the wind, so a
    /// long life is legible in the places it changed.
    #[serde(default)]
    pub remembered_deed: Option<String>,
}

impl Settlement {
    /// Add `n` real residents, generated for this settlement (entity-first epic,
    /// deep-world-godot#50): population growth is a growth of the roster now, not
    /// a bare count bump (which a later `population = people.len()` would wipe).
    /// Keeps an enclave an enclave — new souls share the people if the roster is
    /// uniform (#454). Population follows the roster.
    pub fn add_residents(
        &mut self,
        n: usize,
        rng: &mut crate::rng::SeedRng,
        charts: &crate::charts::Charts,
    ) {
        let enclave = match self.people.first().map(|p| p.people.clone()) {
            Some(k) if self.people.iter().all(|p| p.people == k) => Some(k),
            _ => None,
        };
        for _ in 0..n {
            let mut p = crate::gen::person::generate_person_from(
                rng.fork(),
                &self.region,
                &self.id,
                charts,
            );
            if let Some(h) = &enclave {
                p.people = h.clone();
            }
            self.people.push(p);
        }
        self.population = self.people.len() as u32;
    }

    /// Remove up to `n` residents (entity-first epic): a population loss — famine
    /// flight, a plague's toll — takes real souls off the roster, not just the
    /// count (which `population = people.len()` would otherwise restore).
    pub fn remove_residents(&mut self, n: usize) {
        let keep = self.people.len().saturating_sub(n);
        self.people.truncate(keep);
        self.population = self.people.len() as u32;
    }

    /// District edge in tiles for a head-count: roofs follow households
    /// (one roof per ~7 souls), the edge follows the roofs — but a roof is a
    /// real walled building on a plot now (#458), not a single even/even cell,
    /// so the edge leaves each building room and a street. Quantized to steps
    /// of 4 so the town is not repainted every birth. Clamped to what a sector
    /// can hold — the full sprawl of the great towns lands with the sector
    /// rescale.
    pub fn footprint_for_population(population: u32) -> u32 {
        // One roof per ~7 souls (a household). The district must hold them all
        // as real walkable rooms, so it is sized to the plot pitch the building
        // generator uses (~6 tiles a plot: a room plus its street), plus a
        // one-plot margin for the lane/skirt. A bigger town widens the pitch,
        // so a square-root of the roofs gives the side in plots.
        let roofs = (population.max(7) / 7).max(1) as f64;
        let side = roofs.sqrt().ceil(); // plots per side
        let edge = 6.0 * side + 4.0; // stride-6 plots + margin
        (edge as u32).max(8)
    }

    /// Footprint edge in tiles for a size tier (legacy callers; the real
    /// sizing is by population). Sized for stride-6 rooms (a hamlet still holds
    /// a handful of dwellings, a town a proper district).
    pub fn footprint_for_size(size: &str) -> u32 {
        match size {
            "city" => 36,
            "town" => 24,
            "village" => 16,
            "hamlet" => 10,
            _ => 6,
        }
    }

    /// If this settlement is a non-human **enclave** — its people are one of
    /// the canon Five — which people it belongs to (#454). The Five keep their
    /// own ground and barter in kind; an enclave is read and traded with
    /// differently than a town of the human regions. Derived from the dominant
    /// people, so nothing new is persisted.
    pub fn enclave_people(&self) -> Option<crate::model::PeopleKind> {
        let dominant = self.people.first()?;
        let pk = crate::model::PeopleKind::from_name(&dominant.people);
        pk.is_of_the_five().then_some(pk)
    }

    /// The settlement's name, marked as an enclave of the Five where it is one
    /// ("Vaskiluuri, a Tzäkhar enclave"), so the map and the menus name it for
    /// what it is.
    pub fn display_name(&self) -> String {
        match self.enclave_people() {
            Some(pk) => format!("{}, a {} enclave", self.name, pk.label()),
            None => self.name.clone(),
        }
    }

    /// This settlement's footprint edge in tiles: the painted district if
    /// one is laid, else what the head-count wants.
    pub fn footprint(&self) -> u32 {
        if self.district > 0 {
            self.district
        } else {
            Self::footprint_for_population(self.population)
        }
    }

    /// Whether the given map tile lies inside this settlement's footprint.
    pub fn contains_tile(&self, x: usize, y: usize) -> bool {
        let n = self.footprint() as usize;
        let (ax, ay) = (self.map_x as usize, self.map_y as usize);
        x >= ax && x < ax + n && y >= ay && y < ay + n
    }

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

    /// The effective working hands of a trade (per-agent economy #54, slice 2):
    /// not a flat head-count, but the sum of each producer's worth — a craftsperson
    /// whose innate craft-gift (#441) matches the trade is worth half again as
    /// much. So a town's output reads from *who* keeps a trade, not only how many:
    /// a hamlet with a master smith out-forges a bigger town of journeymen. For a
    /// trade no gift masters, this is exactly the head-count. Deterministic; the
    /// gift is rare, so the total barely moves — it only varies by who is present.
    pub fn trade_power(&self, profession: &str) -> f64 {
        let sense = profession_craft_sense(profession);
        self.people
            .iter()
            .filter(|p| p.profession == profession)
            .map(|p| {
                if sense.is_some() && p.gift.sense() == sense {
                    1.5
                } else {
                    1.0
                }
            })
            .sum()
    }

    /// Current stock of a trade good the settlement holds (#540). `0.0` for a
    /// good it does not keep.
    pub fn good(&self, item: ItemType) -> f64 {
        self.goods_stock.get(&item).copied().unwrap_or(0.0)
    }

    /// Feed the residents one at a time along the hunger ladder (entity-first
    /// slice 3, deep-world-godot#50). Each hungry soul, in stable roster order,
    /// climbs the ladder: it eats a ration from the granary if the stores can
    /// cover one; else buys a meal from outside with personal coin (caravan /
    /// hinterland) if it can afford the price; else takes work to earn coin for
    /// next time (and goes a little hungrier now) if the town has work to give;
    /// else goes hungry (Food decays; sickness and decline follow elsewhere).
    /// This replaces the old uniform per-head feed: now individuals diverge — in
    /// a lean season the coinless poor go without while the moneyed buy through
    /// it, and the able-but-hungry take work. Deterministic (roster order, no
    /// RNG). Returns the ration of food actually drawn from the granary so the
    /// caller can account consumption.
    ///
    /// Slice 4: coin is **conserved**. Buying a meal pays the price into the
    /// town treasury; taking work draws a wage out of it. Work is only on offer
    /// while the treasury can pay it, so a town that runs out of coin offers no
    /// work and its broke go hungry (and, later, desperate). The sum of every
    /// purse plus the treasury is invariant across a call (barring the u32 floor
    /// at 0).
    pub fn feed_people_ladder(&mut self, ration: f64, food_price: u32, wage: u32) -> f64 {
        use crate::model::Need;
        let mut stock = self.food_stock;
        let mut treasury = self.treasury;
        let mut eaten = 0.0;
        for p in self.people.iter_mut() {
            if stock > 0.0 {
                // A stocked granary feeds everyone their daily ration — the town
                // draws its stores down at the old aggregate rate (so famine
                // still bites when the harvest fails) and no soul goes hungry
                // while there is food to eat. The last hungry mouths split
                // whatever ration is left, so the granary empties cleanly to 0.
                let bite = ration.min(stock);
                stock -= bite;
                eaten += bite;
                p.needs.satisfy(Need::Food, 0.10 * (bite / ration).min(1.0));
            } else if p.needs.get(Need::Food) >= 0.7 {
                // Granary empty, but this soul is not hungry yet — it needs
                // nothing this tick and does not spend coin chasing food.
                continue;
            } else if p.coins >= food_price {
                // buy a meal from the market — coin moves to the town's takings
                p.coins -= food_price;
                treasury = treasury.saturating_add(food_price);
                p.needs.satisfy(Need::Food, 0.10);
            } else if treasury >= wage {
                // take work — the town pays a wage out of its purse
                treasury -= wage;
                p.coins = p.coins.saturating_add(wage);
                p.needs.satisfy(Need::Money, 0.10);
                p.needs.decay(Need::Food, 0.05);
            } else {
                // no food it can reach, no coin, no work to be had — go hungry
                p.needs.decay(Need::Food, 0.05);
            }
        }
        self.food_stock = stock;
        self.treasury = treasury;
        eaten
    }

    /// A town cut off from the goods economy (#540, #614): its tools and cloth
    /// per head have run below the thriving bar, so it stalls though its
    /// granaries may be full. The same `furnished` test the growth sim uses,
    /// gated to a town big enough that the shortfall is real (not a hamlet that
    /// simply makes nothing). Deterministic.
    pub fn is_goods_starved(&self) -> bool {
        if self.population < 40 {
            return false;
        }
        let furnished = (self.good(ItemType::Tool) + self.good(ItemType::Cloth))
            / self.population.max(1) as f64;
        furnished < 0.03
    }

    /// Add to a good's stock, capped at `cap` (#540): a town holds only so much
    /// before the surplus goes nowhere (it sells, or it sits).
    pub fn produce_good(&mut self, item: ItemType, amount: f64, cap: f64) {
        let e = self.goods_stock.entry(item).or_insert(0.0);
        *e = (*e + amount).min(cap).max(0.0);
    }

    /// The trade good this town is built to make most, read off its crafts
    /// (#560 living province): the good whose producers are thickest here. Two
    /// towns with the same signature compete in the same market — the seed of a
    /// rivalry. `None` for a town that makes no trade good of its own.
    /// The god the town's dominant people keep by tradition (#595) — the seed
    /// its living faith starts from. Masa (the common people's god) for a town
    /// of no clear patron.
    pub fn patron_seed_god(&self) -> GodName {
        self.people
            .first()
            .and_then(|p| crate::model::PeopleKind::from_name(&p.people).patron_god())
            .unwrap_or(GodName::Masa)
    }

    /// The town's prevailing god right now (#595): the strongest of its living
    /// devotion, or — before its faith has drifted at all — its people's patron.
    pub fn prevailing_god(&self) -> GodName {
        self.faith
            .prevailing()
            .unwrap_or_else(|| self.patron_seed_god())
    }

    pub fn signature_good(&self) -> Option<ItemType> {
        let smiths = self.profession_count("smith") as f64;
        let miners = self.profession_count("miner") as f64;
        let weavers = self.profession_count("weaver") as f64;
        let carpenters = self.profession_count("carpenter") as f64;
        // Mirror the daily production rates so the signature is what the town
        // actually turns out in plenty.
        let weights = [
            (ItemType::Iron, miners * 0.6 + smiths * 0.3),
            (ItemType::Tool, smiths * 0.4),
            (ItemType::Cloth, weavers * 0.5),
            (ItemType::Wood, carpenters * 0.6),
        ];
        weights
            .into_iter()
            .filter(|&(_, w)| w > 0.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(item, _)| item)
    }

    /// Whether a completed building of this type stands here.
    pub fn has_building(&self, kind: BuildingType) -> bool {
        self.buildings
            .iter()
            .any(|b| b.building_type == kind && b.is_complete())
    }

    /// Size tier from the head-count, on the CANON hierarchy (Rennik's
    /// survey, 155 AF): steading 5–50, hamlet 50–500, village 500–3,000,
    /// town 3,000–15,000, city 15,000+. Settlements grow into their tier and
    /// shrink back as their population moves.
    pub fn size_for_population(population: u32) -> &'static str {
        if population >= 15_000 {
            "city"
        } else if population >= 3_000 {
            "town"
        } else if population >= 500 {
            "village"
        } else if population >= 50 {
            "hamlet"
        } else {
            "steading"
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
    /// Fallen to a band on the road (#641 slice 4): its goods carried off, the
    /// train limping on as a wreck. Set when the frontier preys on it.
    #[serde(default)]
    pub raided: bool,
    /// Set once its cargo has been unloaded into the destination's stock on
    /// arrival (#goods-phase2b) — so a caravan deposits its goods exactly once,
    /// physically moving wares region→region (the import).
    #[serde(default)]
    pub unloaded: bool,
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
            raided: false,
            unloaded: false,
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
    /// Not caught from the land — carried in on a bite. A venomous strike
    /// (adder, keth-vaal) courses the blood; it is never contracted by terrain.
    Venom,
    /// Lieska-kuume, flame-fever: the heatstroke that comes of working the gift
    /// past a day's strength. Acute, short, the body's first protest (#427).
    FlameFever,
    /// Rauta-särky, iron-ache: the chronic ache of a gift worked hard for too
    /// long. It lingers, and rest answers it only slowly (#427).
    IronAche,
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
            Disease::Venom => "venom",
            Disease::FlameFever => "flame_fever",
            Disease::IronAche => "iron_ache",
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
            Disease::Venom => 1.5,
            Disease::FlameFever => 1.4,
            Disease::IronAche => 1.2,
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
            Disease::Venom => 54,
            Disease::FlameFever => 36,
            Disease::IronAche => 120,
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
            // Venom and the craft-fevers are never taken from the land.
            (Disease::Venom, _) => 0.0,
            (Disease::FlameFever | Disease::IronAche, _) => 0.0,
            _ => 0.002,
        }
    }

    /// The daily chance an active, full-severity, untreated case takes the
    /// sufferer in a world with no medicine — the great leveller of the post-Fall
    /// age. Rolled once per day while the illness runs; scaled down by tending,
    /// shelter, food, and a healer (see `App::check_illness_mortality`), up by
    /// severity, a plague year, and a cursed star. The acute killers (plague,
    /// childbirth, venom, a wound gone bad) bite hardest; the chronic aches
    /// rarely kill outright.
    pub fn daily_mortality(self) -> f64 {
        match self {
            Disease::Plague => 0.060,
            Disease::ChildbirthComplication => 0.050,
            Disease::Venom => 0.045,
            Disease::Infection => 0.035,
            Disease::MarshFever => 0.030,
            Disease::Fever => 0.022,
            Disease::FlameFever => 0.018,
            Disease::WinterCough => 0.012,
            Disease::BloodAche => 0.006,
            Disease::IronAche => 0.004,
            // Injuries and the spent body: real burdens, but not killers in
            // their own right (exhaustion and starvation kill through vitals).
            Disease::Sprain | Disease::Exhaustion | Disease::ForgeBlindness => 0.0,
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

    /// A salved illness eases harder and runs its course two days faster — for
    /// the wounds a poultice actually answers (infection, venom).
    pub fn tend_strong(&mut self) {
        self.severity = (self.severity - 0.5).max(1.0);
        self.contracted_tick = self.contracted_tick.saturating_sub(48);
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
    /// Autumn field, spring bread: the one crop the frost does not take.
    WinterRye,
    /// The insurance crop of sand and dry steppe — low yield, never zero.
    DroughtMillet,
    /// Grown for fiber, not the pot: the harvest is cloth.
    Flax,
    /// The cold garden's fast crop, at home on tundra ground.
    SnowPea,
}

impl CropType {
    pub fn name(self) -> &'static str {
        match self {
            CropType::Grain => "flood-barley",
            CropType::RootVegetable => "root vegetables",
            CropType::Herb => "herbs",
            CropType::Mushroom => "mushrooms",
            CropType::Berry => "berries",
            CropType::Flatroot => "flatroot",
            CropType::WinterRye => "winter-rye",
            CropType::DroughtMillet => "drought-millet",
            CropType::Flax => "flax",
            CropType::SnowPea => "snow-peas",
        }
    }

    /// What the harvest puts in the sack. Flax is fiber, everything else food.
    pub fn harvest_item(self) -> ItemType {
        match self {
            CropType::Flax => ItemType::Cloth,
            _ => ItemType::Food,
        }
    }

    /// Whether the crop feeds anyone. Settlement farms plant only these;
    /// flax is a choice someone makes on their own ground.
    pub fn is_food(self) -> bool {
        !matches!(self, CropType::Flax)
    }

    /// Winter-rye holds through the Frost; everything else dies standing.
    pub fn survives_frost(self) -> bool {
        matches!(self, CropType::WinterRye)
    }

    /// Parse a crop from the player's word.
    pub fn from_name(name: &str) -> Option<CropType> {
        match name.to_ascii_lowercase().as_str() {
            "flood-barley" | "barley" | "grain" => Some(CropType::Grain),
            "roots" | "rootvegetable" | "root-vegetables" => Some(CropType::RootVegetable),
            "herb" | "herbs" => Some(CropType::Herb),
            "mushroom" | "mushrooms" => Some(CropType::Mushroom),
            "berry" | "berries" => Some(CropType::Berry),
            "flatroot" => Some(CropType::Flatroot),
            "winter-rye" | "winterrye" | "rye" => Some(CropType::WinterRye),
            "drought-millet" | "millet" => Some(CropType::DroughtMillet),
            "flax" => Some(CropType::Flax),
            "snow-peas" | "snowpea" | "snowpeas" | "peas" => Some(CropType::SnowPea),
            _ => None,
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
            CropType::WinterRye => 120,
            CropType::DroughtMillet => 84,
            CropType::Flax => 96,
            CropType::SnowPea => 48,
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
            CropType::WinterRye => 3,
            CropType::DroughtMillet => 2,
            CropType::Flax => 2,
            CropType::SnowPea => 2,
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
            (CropType::WinterRye, Terrain::Farmland | Terrain::Grass) => 1.05,
            (CropType::DroughtMillet, Terrain::Sand | Terrain::Steppe) => 1.25,
            (CropType::Flax, Terrain::Farmland | Terrain::Grass) => 1.1,
            (CropType::SnowPea, Terrain::Tundra) => 1.25,
            (_, Terrain::Farmland) => 1.0,
            _ => 0.7,
        }
    }

    /// Every crop anyone might plant.
    pub fn all() -> [CropType; 10] {
        [
            CropType::Grain,
            CropType::RootVegetable,
            CropType::Herb,
            CropType::Mushroom,
            CropType::Berry,
            CropType::Flatroot,
            CropType::WinterRye,
            CropType::DroughtMillet,
            CropType::Flax,
            CropType::SnowPea,
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

/// A field the player works: the Farm machinery, pinned to a tile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerFarm {
    pub farm: Farm,
    pub region_idx: usize,
    pub x: u32,
    pub y: u32,
}

// Quest struct moved to quest.rs — re-exported via pub use
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_standings_drift_toward_the_pull() {
        let mut p = SettlementPolitics::new(); // 0.5 / 0.5 / 0.5
                                               // A town that trades and prospers lifts the Traders.
        for _ in 0..300 {
            p.drift_toward(0.5, 8.0, 0.5, 0.05);
        }
        assert_eq!(p.dominant_faction(), Faction::Traders);
        // Turn it into a town of makers, and the Crafters take the council.
        for _ in 0..300 {
            p.drift_toward(8.0, 0.5, 0.5, 0.05);
        }
        assert_eq!(p.dominant_faction(), Faction::Crafters);
    }

    #[test]
    fn quality_starting_durability_roundtrips() {
        for tier in [
            QualityTier::Rough,
            QualityTier::Sturdy,
            QualityTier::Fine,
            QualityTier::Masterwork,
        ] {
            assert_eq!(
                QualityTier::from_durability(tier.starting_durability()),
                tier,
                "a freshly-made {tier:?} reads back as {tier:?}"
            );
        }
        assert!(QualityTier::Masterwork.sell_multiplier() > QualityTier::Rough.sell_multiplier());
    }

    #[test]
    fn add_with_quality_blends_into_a_stack() {
        let mut inv = Inventory::default();
        inv.add_with_quality(ItemType::Tool, 1, 0.2); // a rough one
        assert!((inv.durability(ItemType::Tool) - 0.2).abs() < 1e-9);
        inv.add_with_quality(ItemType::Tool, 1, 1.0); // a masterwork one
                                                      // Two pieces, durabilities 0.2 and 1.0 → average 0.6.
        assert!((inv.durability(ItemType::Tool) - 0.6).abs() < 1e-9);
        assert_eq!(inv.get(ItemType::Tool), 2);
    }

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

        // The coast gives clay now (#671 slice 2), not water; open water still
        // gives water to drink.
        assert_eq!(ItemType::gather_from(Terrain::Coast), Some(ItemType::Clay));

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

        // 24 = all item kinds minus Coin (Tool/Bandage/Trap added with #310,
        // Hide with #413, Leather/Coat/Salve with #414, Clay/Pottery/Charcoal/
        // Ale with #671).
        assert_eq!(items.len(), 24);
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
    fn the_craftable_gaps_are_filled() {
        // Nails, Tinder, and Glass used to be buy-only; #529 gives them recipes.
        let outputs: std::collections::HashSet<ItemType> =
            craft_recipes().iter().map(|r| r.output).collect();
        for it in [ItemType::Nails, ItemType::Tinder, ItemType::Glass] {
            assert!(outputs.contains(&it), "{it:?} should now be craftable");
        }
        // The Mëräk glass is theirs alone; nails and tinder are anyone's bench.
        let glass = craft_recipes()
            .into_iter()
            .find(|r| r.output == ItemType::Glass)
            .unwrap();
        assert_eq!(glass.people, Some(PeopleKind::Merak));
    }

    #[test]
    fn gear_chain_recipes_close_the_loop() {
        let r = craft_recipes();
        let find = |out| r.iter().find(|c| c.output == out);
        // Hide -> Leather -> Coat, and Herb -> Salve.
        let leather = find(ItemType::Leather).expect("leather recipe");
        assert!(leather.inputs.iter().any(|(i, _)| *i == ItemType::Hide));
        let coat = find(ItemType::Coat).expect("coat recipe");
        assert!(coat.inputs.iter().any(|(i, _)| *i == ItemType::Leather));
        let salve = find(ItemType::Salve).expect("salve recipe");
        assert!(salve.inputs.iter().any(|(i, _)| *i == ItemType::Herb));
    }

    #[test]
    fn salve_tends_harder_than_a_bandage() {
        let mut bandaged = ActiveDisease::new(Disease::Infection, 1000);
        let mut salved = ActiveDisease::new(Disease::Infection, 1000);
        bandaged.worsen(100); // drive severity to the cap
        salved.worsen(100);
        bandaged.tend();
        salved.tend_strong();
        assert!(
            salved.severity < bandaged.severity,
            "salve should ease severity more: salve={} bandage={}",
            salved.severity,
            bandaged.severity
        );
        assert!(
            salved.contracted_tick < bandaged.contracted_tick,
            "salve should shorten the course more"
        );
    }

    #[test]

    fn settlement_service_costs() {
        assert_eq!(SettlementService::Tavern.cost(), 2);

        assert_eq!(SettlementService::Temple.cost(), 3);
    }

    #[test]
    fn service_map_signs_are_single_width_and_distinct() {
        let all = [
            SettlementService::Tavern,
            SettlementService::Temple,
            SettlementService::Forge,
            SettlementService::Hearth,
            SettlementService::TrapWorkshop,
            SettlementService::Archive,
            SettlementService::TradePost,
            SettlementService::Shrine,
        ];
        let mut seen = std::collections::HashSet::new();
        for s in all {
            let c = s.map_sign();
            // ASCII so it is one terminal cell — the tile grid must not skew.
            assert!(c.is_ascii(), "{:?} sign {c:?} must be single-width", s);
            assert!(seen.insert(c), "{:?} sign {c:?} collides with another", s);
        }
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
            is_march: false,
            known_fed: None,
            known_fed_as_of: 0,
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
            is_march: false,
            known_fed: None,
            known_fed_as_of: 0,
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

            // A populated faith and an active plague, so the roundtrip proves
            // the living-world fields survive a save, not just that they compile.
            faith: SettlementFaith::seeded(crate::model::GodName::Keuru),
            food_stock: 0.0,
            treasury: 0,
            goods_stock: Default::default(),
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            plague_days: 7,
            map_x: 0,
            map_y: 0,
            district: 0,
            remembered_deed: None,
        };

        roundtrip(&s);
        // And the faith's prevailing god is preserved across the save.
        let json = ron::ser::to_string(&s).unwrap();
        let back: Settlement = ron::from_str(&json).unwrap();
        assert_eq!(back.faith.prevailing(), Some(crate::model::GodName::Keuru));
        assert_eq!(back.plague_days, 7);
    }

    #[test]
    fn enclave_barter_is_fixed_in_kind_and_coinless() {
        use crate::model::PeopleKind as P;
        // Each of the Five takes a good and gives goods — never coin, neither
        // taken nor given.
        let deals = [
            (P::Khor, ItemType::Tool),
            (P::Merak, ItemType::Cloth),
            (P::Tzakhar, ItemType::Food),
            (P::Hal, ItemType::Tool),
            (P::Shear, ItemType::Cloth),
        ];
        for (pk, offered) in deals {
            let (cost, gives) = enclave_barter(pk, offered).expect("the Five trade their goods");
            assert!(cost >= 1, "{pk:?} asks for at least one {offered:?}");
            assert!(!gives.is_empty(), "{pk:?} gives something back");
            assert_ne!(offered, ItemType::Coin, "never trades for coin");
            for (item, qty) in &gives {
                assert_ne!(*item, ItemType::Coin, "{pk:?} never pays in coin");
                assert!(*qty >= 1);
            }
        }
        // They want nothing the deal doesn't name — and never coin.
        assert!(enclave_barter(P::Khor, ItemType::Coin).is_none());
        assert!(enclave_barter(P::Merak, ItemType::Stone).is_none());
    }

    #[test]
    fn enclave_is_recognised_from_its_people() {
        let mut s = Settlement {
            id: "e".into(),
            name: "Vaskiluuri".into(),
            size: "hamlet".into(),
            region: "cave".into(),
            population: 30,
            description: String::new(),
            people: vec![Person {
                people: "khör".into(),
                ..Default::default()
            }],
            services: vec![],
            politics: SettlementPolitics::new(),
            faith: Default::default(),
            food_stock: 0.0,
            treasury: 0,
            goods_stock: Default::default(),
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            plague_days: 0,
            map_x: 0,
            map_y: 0,
            district: 0,
            remembered_deed: None,
        };
        assert_eq!(s.enclave_people(), Some(crate::model::PeopleKind::Khor));
        assert_eq!(s.display_name(), "Vaskiluuri, a Khör enclave");

        // A human-led settlement is no enclave.
        s.people = vec![Person {
            people: "metsik".into(),
            ..Default::default()
        }];
        assert_eq!(s.enclave_people(), None);
        assert_eq!(s.display_name(), "Vaskiluuri");

        // A people-less shell is no enclave either.
        s.people = vec![];
        assert_eq!(s.enclave_people(), None);
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
            is_march: false,
            known_fed: None,
            known_fed_as_of: 0,
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
            polity: Default::default(),
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

        let steppe_prob = Disease::Fever.contraction_probability(Terrain::Steppe);

        assert!(swamp_prob > steppe_prob);
    }

    #[test]

    fn crop_type_properties() {
        assert_eq!(CropType::Grain.name(), "flood-barley");

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

        let grain_steppe = CropType::Grain.regional_suitability(Terrain::Steppe);

        assert!(grain_farmland > grain_steppe);

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
    fn faith_seeds_drifts_and_picks_a_prevailing_god() {
        // A seeded faith prevails toward its patron.
        let f = SettlementFaith::seeded(GodName::Keuru);
        assert_eq!(f.prevailing(), Some(GodName::Keuru));
        // A fresh faith has none until touched.
        assert_eq!(SettlementFaith::default().prevailing(), None);
        // Drift pulls the prevailing god over toward the target, and devotion
        // stays a share of one.
        let mut f = SettlementFaith::seeded(GodName::Keuru);
        for _ in 0..200 {
            f.drift_toward(GodName::Masa, 0.05);
        }
        assert_eq!(f.prevailing(), Some(GodName::Masa));
        let sum: f64 = f.devotion.values().sum();
        assert!((sum - 1.0).abs() < 1e-6, "devotion is a share of one");
        // Prevailing is deterministic across runs (fixed god order on ties).
        let g = SettlementFaith::default();
        let mut tied = g.clone();
        for god in GodName::all() {
            tied.devotion.insert(god, 0.2);
        }
        assert_eq!(tied.prevailing(), Some(GodName::Oltzed), "tie → first god");
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

    #[test]
    fn trader_council_opens_roads_elder_council_closes_them() {
        let mut trader = SettlementPolitics::new();
        trader.trader_standing = 0.9;
        let mut elder = SettlementPolitics::new();
        elder.elder_standing = 0.9;

        assert!(
            trader.openness() > 1.0,
            "a firm Traders council throws the gates wide"
        );
        assert!(
            elder.openness() < 1.0,
            "a firm Elders council keeps to itself"
        );
        assert!(trader.openness() > elder.openness());

        // A contested council barely moves the needle off the neutral 1.0.
        let mut split = SettlementPolitics::new();
        split.trader_standing = 0.36;
        assert!(
            (split.openness() - 1.0).abs() < 0.2,
            "a near-tie is middling"
        );
    }
}

#[cfg(test)]
mod good_item_tests {
    use super::*;
    use crate::model::goods::good_id;

    #[test]
    fn a_good_item_has_name_and_price_from_the_registry() {
        let salt = ItemType::Good(good_id("salt").unwrap());
        assert_eq!(salt.name(), "Salt");
        assert_eq!(salt.base_price(), 4);
        let silk = ItemType::Good(good_id("silk").unwrap());
        assert!(
            silk.base_price() > salt.base_price(),
            "silk dearer than salt"
        );
    }

    #[test]
    fn inventory_with_goods_round_trips_and_old_saves_still_load() {
        let mut inv = Inventory::default();
        inv.add(ItemType::Food, 3); // core item
        inv.add(ItemType::Good(good_id("amber").unwrap()), 2);
        inv.add(ItemType::Good(good_id("salt").unwrap()), 5);
        let ron = ron::ser::to_string(&inv).unwrap();
        // Goods serialise as slugs, core items as bare idents.
        assert!(ron.contains("Good(\"amber\")"), "good as slug key: {ron}");
        assert!(ron.contains("Food:3"), "core item unchanged: {ron}");
        let back: Inventory = ron::from_str(&ron).unwrap();
        assert_eq!(inv, back);
        // An old save (core variants only, no goods) still parses unchanged.
        let old: Inventory =
            ron::from_str("(items:{Food:9,Tool:1},coins:7,durability:{})").unwrap();
        assert_eq!(old.get(ItemType::Food), 9);
        assert_eq!(old.coins, 7);
    }
}
