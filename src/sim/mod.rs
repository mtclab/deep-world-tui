use crate::charts::Charts;
use crate::model::{Need, World};
use crate::rng::SeedRng;

pub mod agency;
pub mod aspiration;
pub mod beasts;
pub mod caravans;
pub mod collapse_log;
pub mod effects;
pub mod founding;
pub mod frontier;
pub mod god;
pub mod hints;
pub mod illness;
pub mod journal;
pub mod lifecycle;
pub mod migration;
pub mod milestones;
pub mod needs_dependent;
pub mod params;
pub mod quest_gen;
pub mod relationships;
pub mod reputation;
pub mod rest;
pub mod rumors;
pub mod signals;
pub mod structures;
pub mod wants;
pub mod wayfarers;
pub mod weather;

use effects::{EffectContext, EffectQueue};
pub use journal::{Journal, JournalEntry, Voice};
pub use params::SimParams;
use relationships::RelationshipTracker;
use reputation::ReputationStore;

pub fn tick_needs_with_params(world: &mut World, dt: f64, params: &SimParams) {
    tick_needs_lod(world, dt, params, None);
}

/// Needs decay, two-rate-LOD aware. `lod = Some((active_region, tick))` ticks the
/// active region live and every other region only at the daily boundary (a day's
/// decay in one pass); `lod = None` ticks every region live (tests, simple tick).
pub fn tick_needs_lod(world: &mut World, dt: f64, params: &SimParams, lod: Option<(usize, u64)>) {
    let rates: [(Need, f64); 5] = [
        (Need::Food, params.food_decay_rate),
        (Need::Money, params.money_decay_rate),
        (Need::Care, params.care_decay_rate),
        (Need::Presence, params.presence_decay_rate),
        (Need::Safety, params.safety_decay_rate),
    ];
    for (ri, region) in world.regions.iter_mut().enumerate() {
        let mult = match lod {
            None => dt,
            Some((active, tick)) => match region_tick_mode(ri, active, tick) {
                RegionTick::Live => dt,
                RegionTick::DailyBatch => dt * 24.0,
                RegionTick::Skip => continue,
            },
        };
        for settlement in &mut region.settlements {
            for person in &mut settlement.people {
                for (need, rate) in &rates {
                    person.needs.decay(*need, rate * mult);
                }
            }
        }
    }
}

pub fn tick_needs(world: &mut World, dt: f64) {
    tick_needs_with_params(world, dt, &SimParams::default());
}

pub fn tick(world: &mut World) {
    tick_needs(world, 1.0);
    world.tick += 1;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimState {
    pub world: World,
    pub effect_queue: EffectQueue,
    // Default these so a save missing them still loads (both derive Default).
    // Matches the treatment of every other non-world-snapshot field.
    #[serde(default)]
    pub relationships: RelationshipTracker,
    #[serde(default)]
    pub reputation: ReputationStore,
    // Late-added gameplay state: default so saves written before obligations
    // existed still load (it's not part of the original world snapshot).
    #[serde(default)]
    pub obligations: Vec<needs_dependent::Obligation>,
    pub charts: Charts,
    #[serde(default)]
    pub journal: Journal,
    #[serde(default = "SimParams::default")]
    pub params: SimParams,
    #[serde(default)]
    pub npc_memories: indexmap::IndexMap<String, crate::model::NpcMemory>,
    #[serde(default)]
    pub quests: Vec<crate::model::Quest>,
    #[serde(default)]
    pub aided_npcs: Vec<String>,
    #[serde(default)]
    pub discoveries: crate::model::DiscoveryStore,
    #[serde(default)]
    pub memorials: Vec<crate::model::memorial::Memorial>,
    #[serde(default)]
    pub structures: Vec<crate::sim::structures::Structure>,
    #[serde(default)]
    pub build_sites: Vec<crate::sim::structures::BuildSite>,
    #[serde(default)]
    pub caravans: Vec<crate::model::economy::Caravan>,
    /// The web of standings among the province's settlements (#560): which towns
    /// have grown into trade partners and which into rivals. Drifts in the daily
    /// sim from the caravans they exchange and the goods they compete for.
    #[serde(default)]
    pub province_ties: crate::model::province::ProvinceTies,
    /// The last town the player carried goods INTO (#565 slice 3): if they then
    /// provision a rival of it, the player is a cart crossing between enemies and
    /// the rivalry thaws a little — trade is the solvent. Name, not id, to match
    /// the province ties.
    #[serde(default)]
    pub last_provisioned_town: Option<String>,
    /// The ungoverned country and who has gone into it (#623): the restless the
    /// settled lands have shed to the open road. Seed of the bands to come.
    #[serde(default)]
    pub frontier: crate::sim::frontier::Frontier,
    /// The land's wild beasts as actors on the grid (#637): each creature on its
    /// own tile, to be hunted or fled. Restocked from each region's wildness.
    #[serde(default)]
    pub beasts: Vec<crate::sim::beasts::WildBeast>,
    /// People on the move between towns (#641 slice 3): a migration is no longer
    /// an instant teleport in the roster — the migrant leaves their town, walks
    /// the road as a party for a day or two (seen on the grid), then arrives.
    #[serde(default)]
    pub migrant_parties: Vec<crate::sim::migration::MigrantParty>,
    /// Wandering folk on the road (#649 slice 2): travellers, bards, pilgrims,
    /// and hermits as actors on the grid, met by walking up to them — the old
    /// roadside encounters, now seen, not popped.
    #[serde(default)]
    pub wayfarers: Vec<crate::sim::wayfarers::Wayfarer>,
    /// Two-rate LOD (entity-first epic, deep-world-godot#50): the region the
    /// player is in. With every soul now a real agent, ticking the whole
    /// province's per-person systems every game-hour is too costly; the active
    /// region ticks live each hour, while the rest advance once a day in a
    /// batched step (a day's worth in one pass). Set by the App before each
    /// step; defaults to 0 (and old saves load fine).
    #[serde(default)]
    pub active_region: usize,
}

impl SimState {
    pub fn new(seed: u64, charts: Charts) -> Self {
        Self::new_capped(seed, charts, None)
    }

    /// Build a sim whose settlements are capped to `max_pop` souls each — a test
    /// affordance so soak/long-run tests step small rosters instead of a whole
    /// 8.5k–121k province, while exercising the same entity-first paths
    /// deterministically. `None` is the real game (every soul real). See
    /// [`crate::gen::world::generate_world_capped`].
    pub fn new_capped(seed: u64, charts: Charts, max_pop: Option<usize>) -> Self {
        let mut world = crate::gen::world::generate_world_capped(seed, &charts, max_pop);
        structures::generate_world_structures(seed, &mut world);
        let mut discoveries = crate::model::DiscoveryStore::new();
        {
            let mut rng = SeedRng::new(seed).fork_for("discoveries");
            for (ri, region) in world.regions.iter().enumerate() {
                let w = region.terrain.width.max(1);
                let h = region.terrain.height.max(1);
                let region_discs =
                    crate::model::discovery::generate_region_discoveries(&mut rng, ri, w, h);
                discoveries.entries.extend(region_discs);
            }
        }
        let mut sim = SimState {
            world,
            effect_queue: EffectQueue::new(),
            relationships: RelationshipTracker::new(),
            reputation: ReputationStore::new(),
            obligations: Vec::new(),
            charts,
            journal: Journal::default(),
            params: SimParams::default(),
            npc_memories: indexmap::IndexMap::new(),
            quests: Vec::new(),
            aided_npcs: Vec::new(),
            discoveries,
            memorials: vec![],
            structures: Vec::new(),
            build_sites: Vec::new(),
            caravans: Vec::new(),
            province_ties: crate::model::province::ProvinceTies::default(),
            last_provisioned_town: None,
            frontier: crate::sim::frontier::Frontier::default(),
            beasts: Vec::new(),
            migrant_parties: Vec::new(),
            wayfarers: Vec::new(),
            active_region: 0,
        };
        sim.init_npc_wants();
        sim
    }

    fn init_npc_wants(&mut self) {
        let seed = self.world.seed;
        // Set each person's wants in place. This used to look every person up by
        // id (a full-roster scan per person), which was O(n^2) — invisible at the
        // old ~400-person sample, a hard hang once every soul is real (121k ->
        // 1.5e10 ops). Entity-first epic, deep-world-godot#50.
        for region in &mut self.world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    person.wants = wants::generate_npc_wants(seed, &person.id, &person.people);
                }
            }
        }
    }

    pub fn step(&mut self) {
        sim_tick(self);
    }

    pub fn log_journal(&mut self, tick: u64, text: String) {
        self.journal.log(tick, Voice::Encounter, text);
    }

    pub fn log(&mut self, tick: u64, voice: Voice, text: String) {
        self.journal.log(tick, voice, text);
    }
}

/// Two-rate LOD dispatch (entity-first epic): how a region advances this tick.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RegionTick {
    /// The player's region — full hourly resolution.
    Live,
    /// A distant region, caught up once a day with a day's worth in one pass.
    DailyBatch,
    /// A distant region on a non-daily hour — its hourly work is deferred.
    Skip,
}

/// Decide a region's tick mode: the active region is always Live; every other
/// region advances only at the daily boundary (a 24-hour batch), and is skipped
/// the rest of the day. Per-hour rates are multiplied by 24 on the batch so the
/// daily effect matches having ticked it live every hour.
#[inline]
pub fn region_tick_mode(region_idx: usize, active_region: usize, tick: u64) -> RegionTick {
    if region_idx == active_region {
        RegionTick::Live
    } else if tick.is_multiple_of(24) {
        RegionTick::DailyBatch
    } else {
        RegionTick::Skip
    }
}

pub fn sim_tick(sim: &mut SimState) {
    sim.world.tick += 1;
    let current_tick = sim.world.tick;
    let due = sim.effect_queue.due(current_tick);
    let descs: Vec<String> = due
        .iter()
        .map(|e| match e {
            effects::Effect::Immediate { description, .. } => description.clone(),
            effects::Effect::Deferred { description, .. } => description.clone(),
        })
        .filter(|d| !d.is_empty())
        .collect();
    {
        let mut ctx = EffectContext {
            world: &mut sim.world,
            relationships: &mut sim.relationships,
            reputation: &mut sim.reputation,
            current_tick,
        };
        for effect in &due {
            effects::apply_effect(&mut ctx, effect);
        }
    }
    for desc in descs {
        sim.log_journal(current_tick, desc);
    }
    let lod = Some((sim.active_region, current_tick));
    tick_needs_lod(&mut sim.world, 1.0, &sim.params, lod);
    needs_dependent::propagate_dependent_needs(&mut sim.world, &sim.obligations);
    reputation::spread_reputation(&mut sim.reputation, &sim.world, &sim.province_ties, 1.0);
    sim.relationships.tick_converge(1.0);
    tick_npc_illness(sim, current_tick);
    // Relation drift, two-rate LOD: the active region every hour; distant regions
    // once a day. A bond's slow fade reads the same at a day's resolution.
    for (ri, region) in sim.world.regions.iter_mut().enumerate() {
        if region_tick_mode(ri, sim.active_region, current_tick) == RegionTick::Skip {
            continue;
        }
        for settlement in region.settlements.iter_mut() {
            for person in settlement.people.iter_mut() {
                crate::model::relation::decay_relations(&mut person.relations);
            }
        }
    }
    migration::tick_migration(sim, current_tick);
    migration::complete_migrant_arrivals(sim, current_tick);
    let hour = ((current_tick % 24) / 4) as u32;
    sim.world.tick_npc_wants(current_tick, 24);
    sim.world.recompute_all_schedules(hour);
    tick_build_sites(sim);
    tick_structure_decay(sim);
    tick_caravans(sim);
    tick_province_ties(sim);
    tick_weather_fronts(sim);
    tick_settlement_life(sim);
    tick_winter_relief(sim);
    tick_alliance_relief(sim);
    tick_faith_spread(sim);
    tick_faith_upheavals(sim);
    tick_plague(sim);
    tick_plague_spread(sim);
    tick_raids(sim);
    frontier::tick_frontier(sim);
    beasts::tick_beasts(sim);
    wayfarers::tick_wayfarers(sim);
    lifecycle::tick_lifecycle(sim);
    // Each season-turn the world builds back a little: ghost towns reopen,
    // founding parties take rich empty land — slowly, the way the Fall's
    // long tail allows.
    if current_tick.is_multiple_of(24) {
        let day = (current_tick / 24) as u32;
        if day > 0 && day.is_multiple_of(30) {
            founding::tick_world_building(sim, day / 30);
        }
    }

    // Authoritative cap: a town's trade goods can never hold above what its
    // people can (population*0.5, #540). Goods are clamped where they are made,
    // but several systems shrink a population *after* that — famine decline,
    // band raids, deaths of age — so without a final sweep a shrunk town could
    // end a tick holding goods above its new cap. One pass here guarantees the
    // invariant at every tick's close, whichever system did the shrinking.
    for region in sim.world.regions.iter_mut() {
        for s in region.settlements.iter_mut() {
            let cap = s.population as f64 * 0.5;
            for v in s.goods_stock.values_mut() {
                *v = v.min(cap);
            }
            // Invariant: a settlement with no souls left has nothing open. A town
            // can be emptied by any path (famine flight, deaths, the last migrant
            // leaving); whichever did it, no services stand in a ghost town.
            if s.people.is_empty() && !s.services.is_empty() {
                s.services.clear();
            }
        }
    }
}

/// Daily weather fronts: each region's sky persists (~55%), drifts in from a
/// neighbor (~15%), or turns with the terrain's own tendencies (~30%) —
/// replacing the old hourly hash where sun could follow blizzard by the hour.
fn tick_weather_fronts(sim: &mut SimState) {
    use crate::model::Weather;
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = tick / 24;
    let seed = sim.world.seed;
    let current: Vec<Weather> = sim.world.regions.iter().map(|r| r.weather).collect();
    for (ri, region) in sim.world.regions.iter_mut().enumerate() {
        // Wild game recovers slowly; the green season breeds faster.
        let season = crate::model::Season::from_day(day as u32);
        let recover = if season == crate::model::Season::Green {
            0.02
        } else {
            0.01
        };
        region.game_richness = (region.game_richness + recover).min(1.0);
        let mut rng = SeedRng::new(seed).fork_for(&format!("front-{day}-{ri}"));
        let roll = rng.gen_range(100);
        region.weather = if roll < 55 {
            current[ri]
        } else if roll < 70 {
            let nbs: Vec<usize> = [
                region.neighbors.north,
                region.neighbors.east,
                region.neighbors.south,
                region.neighbors.west,
            ]
            .into_iter()
            .flatten()
            .filter(|&n| n < current.len())
            .collect();
            if nbs.is_empty() {
                Weather::generate(seed, day, region_work_terrain(&region.region_type))
            } else {
                current[nbs[rng.gen_range(nbs.len() as u32) as usize]]
            }
        } else {
            Weather::generate(seed, day, region_work_terrain(&region.region_type))
        };
    }
}

/// The terrain a settlement farms and fishes, derived from its region type.
pub(crate) fn region_work_terrain(region_type: &str) -> crate::model::Terrain {
    use crate::model::Terrain;
    match region_type {
        "river_valley" | "delta" => Terrain::Farmland,
        "forest" => Terrain::Forest,
        "coast" => Terrain::Coast,
        "upland" => Terrain::Mountain,
        _ => Terrain::Grass,
    }
}

/// The crop a settlement's farmers favor on the given ground.
fn best_crop_for(terrain: crate::model::Terrain) -> crate::model::economy::CropType {
    use crate::model::economy::CropType;
    // A settlement's farmers plant what feeds the stores; fiber crops are a
    // choice someone makes on their own ground.
    CropType::all()
        .into_iter()
        .filter(|c| c.is_food())
        .max_by(|a, b| {
            a.regional_suitability(terrain)
                .partial_cmp(&b.regional_suitability(terrain))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(CropType::Grain)
}

/// Settlement daily life: farmers plant and harvest, fishers and herders bring
/// food in, the population eats from the common stores, hearth-keepers slow
/// spoilage, builders raise buildings, and soldiers/singers steady the
/// settlement's safety and spirits. The Farm/CropType and Building/BuildingType
/// systems were fully modeled but never instantiated — settlements had no
/// food economy and never built anything.
fn tick_settlement_life(sim: &mut SimState) {
    use crate::model::economy::{Building, BuildingType, Farm};
    use crate::model::Need;

    let tick = sim.world.tick;
    // Heavy work happens once per day.
    if !tick.is_multiple_of(24) {
        return;
    }
    let season = crate::model::Season::from_day((tick / 24) as u32);
    let seed = sim.world.seed;
    // Cloned up front so a growing town can mint real new residents while the
    // world (sim.world) is borrowed by the region loop below.
    let life_charts = sim.charts.clone();
    let mut completed_msgs: Vec<String> = Vec::new();
    // Souls who fall off the bottom of the hunger ladder this day and leave for
    // the open country (entity-first slice 5). Gathered while the world is
    // borrowed by the region loop, then fed into the frontier once it is free.
    let mut new_wanderers: u32 = 0;

    for region in sim.world.regions.iter_mut() {
        let terrain = region_work_terrain(&region.region_type);
        let weather = region.weather;
        let region_richness = region.game_richness;
        let mut richness_draw = 0.0_f64;
        let mut grown_footprints: Vec<(u32, u32, u32, crate::gen::building::BuildCharacter)> =
            Vec::new();
        let (map_w, map_h) = (region.terrain.width, region.terrain.height);
        let rtype = region.region_type.clone();
        // The terrain is read (not painted) through the settlement loop below,
        // and the loop only mutates a *different* field (`settlements`), so a
        // disjoint borrow lets us read the live tiles without cloning the whole
        // sector every region every day.
        let region_terrain_snapshot = &region.terrain;
        // Today's district boxes (with their farmland skirt): growth may
        // only claim ground no neighbor already holds.
        let district_boxes: Vec<(usize, usize, usize, usize)> = region
            .settlements
            .iter()
            .map(|s| {
                let (x, y, d) = (s.map_x as usize, s.map_y as usize, s.district as usize);
                (x, y, x + d + 2, y + d + 2)
            })
            .collect();
        let coastal = matches!(
            region.region_type.as_str(),
            "coast" | "delta" | "river_valley"
        );
        // Agency context (entity-first slice 8): facts the per-agent needs
        // selector reads, gathered once per region/day so the decision stays
        // O(1) per soul. The land is dangerous in a march or under a real
        // beast/raid threat (Safety). The best-fed *other* town is where a soul
        // that must leave will flee to, if it does not turn outlaw.
        let region_under_threat = region.is_march
            || region.danger_level() == crate::model::economy::DangerLevel::Dangerous;
        let best_fed: Option<usize> = region
            .settlements
            .iter()
            .enumerate()
            .filter(|(_, s)| s.population > 0 && s.food_stock / s.population.max(1) as f64 >= 1.5)
            .max_by(|(_, a), (_, b)| {
                let da = a.food_stock / a.population.max(1) as f64;
                let db = b.food_stock / b.population.max(1) as f64;
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i);
        let season_ration = 0.15 * season.consumption_modifier();
        // Leavers, gathered while a single settlement is borrowed, applied after
        // the settlement loop (when the whole region roster is free again).
        let mut region_bandits: u32 = 0;
        let mut pending_migrants: Vec<(usize, crate::model::Person)> = Vec::new();

        for (s_idx, settlement) in region.settlements.iter_mut().enumerate() {
            let sample = settlement.people.len().max(1) as f64;
            // What this ground can carry, by the canon's hydraulic
            // principles — and how far its trade reach extends.
            // The founding corner: the same fixed point worldgen sampled,
            // however wide the district has grown since.
            let (cx, cy) = (settlement.map_x as usize + 1, settlement.map_y as usize + 1);
            let cap = crate::gen::town::carrying_capacity(
                &region_terrain_snapshot.tiles,
                region_terrain_snapshot.width,
                region_terrain_snapshot.height,
                cx,
                cy,
                &rtype,
            );
            let trade = crate::gen::town::trade_factor(
                &region_terrain_snapshot.tiles,
                region_terrain_snapshot.width,
                region_terrain_snapshot.height,
                cx,
                cy,
            );
            // The people vec is a sample of the population; scale producers
            // up — but local fields feed at most what the land carries. The
            // rest must ride the road.
            let scale = (settlement.population.max(1) as f64 / sample)
                .min((cap.max(1) as f64 / sample).max(1.0))
                .max(1.0);

            // --- farms: plant, grow, harvest ---
            let farmers = settlement.profession_count("farmer");
            let farm_cap = farmers.min(4);
            let frost = season == crate::model::Season::Frost;
            while settlement.farms.len() < farm_cap && !frost && settlement.food_stock >= 1.0 {
                // Seed comes out of the stores — nothing from nothing.
                settlement.food_stock -= 1.0;
                let crop = best_crop_for(terrain);
                let farm_seed = seed
                    .wrapping_add(tick)
                    .wrapping_add(settlement.farms.len() as u64 * 7919)
                    ^ settlement.id.len() as u64;
                settlement
                    .farms
                    .push(Farm::new(farm_seed, crop, tick, terrain));
            }
            let mut harvested = 0.0;
            for farm in settlement.farms.iter_mut() {
                farm.update_growth(tick, weather);
                if farm.is_ready() {
                    harvested += farm.harvest_yield() as f64 * scale;
                    // Replant the same field.
                    farm.planted_tick = tick;
                    farm.growth_progress = 0.0;
                    farm.stage = crate::model::economy::GrowthStage::Planted;
                }
            }
            // Frost kills standing crops — except the winter-rye, which is
            // why anyone plants it.
            if frost {
                settlement.farms.retain(|f| f.crop.survives_frost());
            }

            // --- other food producers (per sampled person, scaled) ---
            let fishers = if coastal {
                settlement.profession_count("fisher") + settlement.profession_count("sailor")
            } else {
                0
            };
            let herders = settlement.profession_count("herder");
            let handlers = settlement.profession_count("beast-handler");
            let gathered = (fishers as f64 * 1.0 + herders as f64 * 0.8 + handlers as f64 * 0.5)
                * scale
                * weather.gather_modifier();
            let trap_food = if settlement.has_building(BuildingType::Trap) {
                2.0 * region_richness
            } else {
                0.0
            };
            // Hunting pressure scales with the game that is actually there — you
            // cannot over-hunt an empty wood. Both terms fall with the land's
            // richness, so a depleted region always breeds back faster than it is
            // drawn down (entity-first epic: with every soul real, the handler
            // count is the true population's, not a sample's, so an unscaled draw
            // would strip the land bare and never recover).
            richness_draw += (trap_food * 0.002 + handlers as f64 * 0.001) * region_richness;
            // The land yields at most what it can carry (a small margin over
            // its own mouths): everything beyond that must arrive by road or
            // water. This is the hinterland principle made arithmetic.
            let land_yield_cap = cap as f64 * 0.18;
            settlement.food_stock += (harvested + gathered + trap_food).min(land_yield_cap);

            // --- the people act on their needs (entity-first slice 8) + spoilage ---
            // Each soul scores its drives and acts on the most pressing one it can
            // afford: the Food column (forage → eat → buy → work → beg → steal →
            // leave) draws the granary down at the old rate, while Care, Safety,
            // and Presence are met by the town's healer, shelter, and company.
            // Winter draws harder on the stores (#570) via the season ration. A
            // soul that exhausts every option leaves — lawfully to a fed town, or
            // to the road as a brigand, by its own disposition.
            {
                let migrate_target = best_fed.filter(|&bf| bf != s_idx);
                let ctx = agency::town_context(
                    settlement,
                    region_richness,
                    region_under_threat,
                    migrate_target,
                    season_ration,
                    tick,
                );
                let (departures, _eaten) = agency::step_agents(settlement, &ctx);
                if !departures.is_empty() {
                    let mut leave_ids: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    for (idx, dep) in departures {
                        let p = &settlement.people[idx];
                        leave_ids.insert(p.id.clone());
                        match dep {
                            agency::Departure::Bandit => region_bandits += 1,
                            agency::Departure::Migrate { to } => {
                                pending_migrants.push((to, p.clone()))
                            }
                        }
                    }
                    let n_left = leave_ids.len();
                    settlement.people.retain(|p| !leave_ids.contains(&p.id));
                    settlement.population = settlement.people.len() as u32;
                    if region_bandits > 0 {
                        completed_msgs.push(format!(
                            "Word on the road: {n_left} left {} — some for a kinder town, some for the dark.",
                            settlement.name
                        ));
                    }
                }
            }

            // --- the people pursue their lives (purposeful-agents #53) ---
            // A settled soul works toward a standing aspiration — to master a
            // trade, to marry — resolving over many days into a real life event.
            for ev in crate::sim::aspiration::tick_settlement_aspirations(settlement, seed, tick) {
                completed_msgs.push(ev);
            }
            let keepers = settlement.profession_count("hearth-keeper") as f64;
            let hearth = if settlement.has_building(BuildingType::Hearth) {
                0.5
            } else {
                0.0
            };
            let spoil_rate = (0.03 - keepers * 0.005 - hearth * 0.01).max(0.0);
            settlement.food_stock *= 1.0 - spoil_rate;

            // --- trade goods: the crafts make what they make (#540) ---
            // System-first, player-absent: each trade adds to the town's goods
            // stock by its hands, scaled like food, capped by what it can hold.
            // A spot of upkeep is spent each day, so a town that loses its
            // crafters runs its stock down rather than holding it forever.
            {
                use crate::model::ItemType;
                // The whole working town, not four trades (#671): every hand
                // that makes a thing adds its thing to the stores. A civilization
                // runs on its foresters and herders and herbalists as much as on
                // its smiths — the goods a town holds should read like the trades
                // it keeps. (Only trades the world actually generates are read:
                // the `profession` roster, not the gift/craft-sense table.)
                let pc = |p: &str| settlement.profession_count(p) as f64;
                let smiths = pc("smith");
                let miners = pc("miner");
                let weavers = pc("weaver");
                let carpenters = pc("carpenter");
                let foresters = pc("forester");
                let herders = pc("herder");
                let handlers = pc("beast-handler");
                let herbalists = pc("herbalist");
                let healers = pc("healer");
                let potters = pc("potter");
                let brewers = pc("brewer");
                // Real division of labour (#674): the stone-cutters, tanners,
                // glass-workers, rope-makers, dyers, butchers, and foragers each
                // stock their own ware — and the glass and cordage that no trade
                // used to make now have hands behind them.
                let masons = pc("mason") + pc("labourer") * 0.3 + pc("fence-builder");
                let tanners = pc("tanner");
                let butchers = pc("butcher");
                let glassworkers = pc("glass-worker");
                let ropemakers = pc("rope-maker");
                let dyers = pc("dyer");
                let foragers = pc("forager");
                let cap = settlement.population as f64 * 0.5;
                // The crafts make less in deep Frost, a touch more in high
                // Green (#570): the year drives the goods economy, not only the
                // crops.
                let prod = season.production_modifier();
                // Hides come off herded and handled stock — domestic animals,
                // not the wild game the trappers thin (#671). So the hide trade
                // adds no extra draw on the region's wildness; the food block's
                // existing handler/trap draw already accounts for the wild take.
                let made = [
                    // Stone, ore, and the forge: miners dig, smiths work.
                    (
                        ItemType::Stone,
                        (miners * 0.4 + masons * 0.5) * scale * prod,
                    ),
                    (ItemType::Iron, (miners * 0.6 + smiths * 0.3) * scale * prod),
                    (ItemType::Tool, smiths * 0.4 * scale * prod),
                    (ItemType::Nails, smiths * 0.25 * scale * prod),
                    // Glass at the kiln — a trade of its own now (#674), where
                    // before no hand stocked it.
                    (ItemType::Glass, glassworkers * 0.3 * scale * prod),
                    // Wood and its by-products: foresters fell, carpenters work,
                    // rope-makers lay cordage from the cut withies.
                    (
                        ItemType::Wood,
                        (carpenters * 0.6 + foresters * 0.5) * scale * prod,
                    ),
                    (ItemType::Branches, foresters * 0.4 * scale * prod),
                    (ItemType::Cordage, ropemakers * 0.5 * scale * prod),
                    // Cloth from the looms and the dye-vats.
                    (
                        ItemType::Cloth,
                        (weavers * 0.5 + dyers * 0.3) * scale * prod,
                    ),
                    // The hide chain: herders, handlers, and butchers bring
                    // skins; the tanners turn them to leather.
                    (
                        ItemType::Hide,
                        (herders * 0.4 + handlers * 0.3 + butchers * 0.3)
                            * scale
                            * region_richness
                            * prod,
                    ),
                    (
                        ItemType::Leather,
                        (tanners * 0.5 + herders * 0.15) * scale * prod,
                    ),
                    // The healing trades: herbalists and foragers gather and
                    // brew, healers dress wounds.
                    (
                        ItemType::Herb,
                        (herbalists * 0.5 + foragers * 0.4) * scale * prod,
                    ),
                    (
                        ItemType::Salve,
                        (herbalists * 0.3 + healers * 0.2) * scale * prod,
                    ),
                    (ItemType::Bandage, healers * 0.3 * scale * prod),
                    // The settled crafts of a deeper shelf (#671 slice 2): the
                    // potter fires Pottery, the brewer mashes Ale, and the
                    // foresters burn a little Charcoal off the wood they fell.
                    (ItemType::Pottery, potters * 0.4 * scale * prod),
                    (ItemType::Ale, brewers * 0.4 * scale * prod),
                    (ItemType::Charcoal, foresters * 0.2 * scale * prod),
                ];
                // The long tail of data-defined trade goods is produced from
                // data rules (#678 slice 3): each names the trades that make it,
                // so the shelf grows by editing data/production.ron, not code.
                // Computed here (an immutable read of profession counts) before
                // the apply loop borrows the stock mutably.
                let season_name = match season {
                    crate::model::Season::Thaw => "thaw",
                    crate::model::Season::Green => "green",
                    crate::model::Season::Frost => "frost",
                };
                let data_made: Vec<(ItemType, f64)> = crate::model::production_rules()
                    .iter()
                    .filter_map(|r| {
                        crate::model::good_id(&r.good).map(|gid| {
                            (
                                ItemType::Good(gid),
                                r.amount(coastal, region_richness, &rtype, season_name, &pc)
                                    * scale
                                    * prod,
                            )
                        })
                    })
                    .collect();
                for (item, amount) in made.into_iter().chain(data_made) {
                    // Daily upkeep: the town spends a little of each good it
                    // keeps (building, mending, wearing out); and never holds
                    // more than its current size can — a shrunken town sheds the
                    // surplus its lost hands once made.
                    let kept = (settlement.good(item) * 0.97).min(cap);
                    settlement.goods_stock.insert(item, kept);
                    if amount > 0.0 {
                        settlement.produce_good(item, amount, cap);
                    }
                }
            }

            // (the people already fed themselves along the hunger ladder above,
            // entity-first slice 3 — no separate uniform per-head feed)

            // --- growth, famine, and decline ---
            // Promotion/demotion by head-count; a new village raises a Temple.
            let new_size = crate::model::Settlement::size_for_population(settlement.population);
            if new_size != settlement.size {
                let grew = matches!(
                    (settlement.size.as_str(), new_size),
                    ("hamlet", _) | ("village", "town") | ("village", "city") | ("town", "city")
                );
                settlement.size = new_size.to_string();
                if grew {
                    // The place grows on the map too: new houses past the
                    // old wall (footprint painted after this loop). Clamp the
                    // anchor so the grown square stays on the map.
                    let n = settlement.footprint();
                    settlement.map_x = settlement
                        .map_x
                        .min(map_w.saturating_sub(n as usize) as u32);
                    settlement.map_y = settlement
                        .map_y
                        .min(map_h.saturating_sub(n as usize) as u32);
                    let character = crate::gen::building::BuildCharacter::from_people(
                        &settlement
                            .people
                            .first()
                            .map(|p| p.people.clone())
                            .unwrap_or_default(),
                    );
                    grown_footprints.push((settlement.map_x, settlement.map_y, n, character));
                    if !settlement
                        .services
                        .contains(&crate::model::SettlementService::Temple)
                    {
                        settlement
                            .services
                            .push(crate::model::SettlementService::Temple);
                    }
                    completed_msgs.push(format!(
                        "{} has grown into a proper {}.",
                        settlement.name, new_size
                    ));
                }
            }
            // The hinterland feeds what the fields cannot: a settlement past
            // its land's capacity lives on grain moved by road and water —
            // and without that reach, the shortfall stands and the famine
            // machinery below does the canon's work.
            if settlement.population > cap && trade >= 1.4 {
                // One day's meals for every head the land cannot feed.
                settlement.food_stock += (settlement.population - cap) as f64 * 0.15;
            }
            // Slow recovery toward what the land can carry (the Fall's
            // long tail: roughly a tenth of a percent a day, fed places only).
            let per_head_now = settlement.food_stock / settlement.population.max(1) as f64;
            // A place grows when it is both fed and furnished (#540): bread keeps
            // it alive, but tools and cloth — made at home or carried in by trade
            // — are what let it thrive. A goods-starved town holds where it is
            // even with full granaries. The bar is low, so only a place genuinely
            // cut off from the goods economy stalls.
            let furnished = (settlement.good(crate::model::ItemType::Tool)
                + settlement.good(crate::model::ItemType::Cloth))
                / settlement.population.max(1) as f64;
            if per_head_now >= 1.5 && settlement.population < cap && furnished >= 0.03 {
                // Growth is real residents joining the roster (entity-first epic):
                // a bare `population += n` would be wiped by a later
                // `population = people.len()`.
                let grow_n = (settlement.population / 1000).max(1) as usize;
                let mut grow_rng = SeedRng::new(seed)
                    .fork_for(&format!("settlement-growth-{}-{}", settlement.id, tick));
                settlement.add_residents(grow_n, &mut grow_rng, &life_charts);
            }
            // The district grows with the households. The anchor shifts
            // up/left only as far as the map edge forces it, and growth only
            // claims ground no neighbor holds — a town is hemmed by its
            // neighbors and the land, exactly as real towns are.
            // Towns sprawl to 48 tiles; a city (15k+ on the canon
            // hierarchy) to 72 — the Tier-II city rightly dominates its
            // sector.
            let max_edge = if settlement.population >= 15_000 {
                72
            } else {
                48
            };
            let mut wanted =
                (crate::model::Settlement::footprint_for_population(settlement.population)
                    as usize)
                    .min(max_edge)
                    .min(map_w.saturating_sub(6))
                    .min(map_h.saturating_sub(6));
            wanted -= wanted % 2;
            if wanted > settlement.district as usize {
                let ax = (settlement.map_x as usize)
                    .min(map_w.saturating_sub(wanted + 2))
                    .max(2);
                let ay = (settlement.map_y as usize)
                    .min(map_h.saturating_sub(wanted + 2))
                    .max(2);
                let gx1 = ax + wanted + 2;
                let gy1 = ay + wanted + 2;
                let blocked = district_boxes
                    .iter()
                    .enumerate()
                    .any(|(j, b)| j != s_idx && ax < b.2 && b.0 < gx1 && ay < b.3 && b.1 < gy1);
                if !blocked {
                    settlement.map_x = ax as u32;
                    settlement.map_y = ay as u32;
                    settlement.district = wanted as u32;
                    let character = crate::gen::building::BuildCharacter::from_people(
                        &settlement
                            .people
                            .first()
                            .map(|p| p.people.clone())
                            .unwrap_or_default(),
                    );
                    grown_footprints.push((ax as u32, ay as u32, wanted as u32, character));
                }
            }

            // Famine: empty stores for a week start an exodus; a month of it
            // leaves the place standing empty.
            if settlement.population > 0 {
                if settlement.food_stock <= 0.0 {
                    settlement.famine_days += 1;
                } else {
                    settlement.famine_days = 0;
                }
                if settlement.famine_days == 7 {
                    completed_msgs.push(format!(
                        "Hunger drives people from {} — the road is full of carts.",
                        settlement.name
                    ));
                }
                if settlement.famine_days > 7 {
                    let leaving = (settlement.population as f64 * 0.02).ceil() as usize;
                    // Real souls take to the road — leave at least one behind.
                    let leaving = leaving.min(settlement.people.len().saturating_sub(1));
                    settlement.remove_residents(leaving);
                }
                if settlement.famine_days > 21 && settlement.population <= 10 {
                    completed_msgs.push(format!(
                        "{} stands empty. The hunger took it.",
                        settlement.name
                    ));
                    settlement.population = 0;
                    settlement.people.clear();
                    settlement.services.clear();
                    settlement.farms.clear();
                    settlement.description = "Abandoned. Doors hang open; nothing moves.".into();
                }
            }

            // Goods cannot float above what the town's people can hold. Stock was
            // capped to population*0.5 when produced, but the decline above can
            // shrink the population after that — re-clamp to the final count so
            // the cap invariant holds at the end of every tick, not just at
            // production (the same correction the plague toll makes, #540).
            {
                let cap = settlement.population as f64 * 0.5;
                for v in settlement.goods_stock.values_mut() {
                    *v = v.min(cap);
                }
            }

            // --- festivals: begin, run, and lift spirits ---
            let day = (tick / 24) as u32;
            if !settlement.in_festival(day) && season.festival_chance() > 0 {
                let h = seed.wrapping_mul(2654435761)
                    ^ (settlement.id.len() as u64).wrapping_mul(40503)
                    ^ (day as u64);
                // Roughly a third of the old per-visit chance, per day.
                if h % 300 < season.festival_chance() as u64 {
                    settlement.festival_until_day = day + 2; // three days
                    completed_msgs.push(format!(
                        "There's a festival in {} — the doors are open and the drink flows.",
                        settlement.name
                    ));
                }
            }
            if settlement.in_festival(day) {
                for person in settlement.people.iter_mut() {
                    person.needs.satisfy(Need::Presence, 0.05);
                    person.needs.satisfy(Need::Care, 0.03);
                }
            }

            // --- guardians and gatherers of spirit ---
            let safety_hands = settlement.profession_count("soldier")
                + settlement.profession_count("fence-builder")
                + settlement.profession_count("path-finder");
            let shelter = settlement.has_building(BuildingType::Shelter);
            let shrine = settlement.has_building(BuildingType::Shrine);
            let singers = settlement.profession_count("singer");
            for person in settlement.people.iter_mut() {
                if safety_hands > 0 || shelter {
                    person.needs.satisfy(
                        Need::Safety,
                        0.02 * (safety_hands.min(3) as f64) + if shelter { 0.02 } else { 0.0 },
                    );
                }
                if singers > 0 {
                    person
                        .needs
                        .satisfy(Need::Presence, 0.02 * singers.min(2) as f64);
                }
                if shrine {
                    person.needs.satisfy(Need::Care, 0.02);
                }
            }

            // --- the social web lives (#548 living relationships) ---
            // One sampled pair of residents forms or deepens a bond each day, so
            // the web grows and shifts rather than only fading; shared hardship
            // (famine) works it the harder. Pure helper, deterministic per town.
            {
                let id_hash = settlement
                    .id
                    .bytes()
                    .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(b as u64));
                let name = settlement.name.clone();
                let famine = settlement.famine_days > 0;
                if let Some(msg) = crate::model::relation::evolve_settlement_relations(
                    &mut settlement.people,
                    &name,
                    famine,
                    seed ^ id_hash,
                    tick,
                ) {
                    completed_msgs.push(msg);
                }
            }
            // A town riven by feud and rivalry lives uneasy (#552): its people's
            // sense of safety frays, and now and then the grudges are talked of.
            let unrest = crate::model::relation::feud_load(&settlement.people);
            if unrest > 0.25 {
                for person in settlement.people.iter_mut() {
                    person.needs.decay(Need::Safety, 0.01);
                }
                let h = crate::rng::mix_u64(
                    seed ^ tick.wrapping_mul(0x2545_F491_4F6C_DD1D)
                        ^ (settlement.population as u64),
                );
                if crate::rng::unit_from_hash(h) < 0.10 {
                    completed_msgs.push(format!(
                        "Old grudges stir in {} — tempers run short there.",
                        settlement.name
                    ));
                }
            }

            // --- the council lives (#556 living politics) ---
            // Faction standings drift toward the town's character: the Crafters
            // rise where much is made, the Traders where trade prospers, the
            // Elders in an old, stable, low-feud town. A real dominant faction
            // emerges (and its price-lever the player feels) instead of the
            // frozen 0.5/0.5/0.5.
            {
                // Per-capita DENSITIES, not raw head-counts (#556 fix): once
                // every soul is real (entity-first epic), a 9k town has hundreds
                // of makers and the raw maker count swamps the rarer trader and
                // keeper counts, so every council froze to Crafters. Shares make
                // the pull scale-invariant — a town's *character*, not its size,
                // decides its council. Rarer trades (traders, keepers) carry a
                // heavier weight so they are not perpetually outnumbered.
                let pop = settlement.population.max(1) as f64;
                let maker_share = (settlement.profession_count("smith")
                    + settlement.profession_count("miner")
                    + settlement.profession_count("weaver")
                    + settlement.profession_count("carpenter"))
                    as f64
                    / pop;
                let merchant_share = (settlement.profession_count("trader")
                    + settlement.profession_count("sailor"))
                    as f64
                    / pop;
                let keeper_share = (settlement.profession_count("priest")
                    + settlement.profession_count("scribe")
                    + settlement.profession_count("healer"))
                    as f64
                    / pop;
                let goods_total: f64 = [
                    crate::model::ItemType::Iron,
                    crate::model::ItemType::Tool,
                    crate::model::ItemType::Cloth,
                    crate::model::ItemType::Wood,
                ]
                .iter()
                .map(|it| settlement.good(*it))
                .sum();
                let goods_density = goods_total / pop;
                // food per head — already scale-invariant.
                let prosperity = (settlement.food_stock / pop).min(3.0);
                // The season leans the council (#570): Frost turns it inward to
                // the Elders, Green expansive to the Traders, Thaw to the hands
                // that rebuild. Small, added to the town's own character.
                let (lean_c, lean_t, lean_e) = season.council_lean();
                // Base of 0.5 so no faction is ever wholly without a voice.
                let crafter_pull = 0.5 + maker_share * 6.0 + goods_density * 0.3 + lean_c;
                let trader_pull = 0.5 + merchant_share * 18.0 + prosperity * 0.6 + lean_t;
                // A stable, low-feud town keeps faith with its Elders.
                let elder_pull = 0.5 + keeper_share * 18.0 + (1.0 - unrest) * 0.8 + lean_e;
                settlement
                    .politics
                    .drift_toward(crafter_pull, trader_pull, elder_pull, 0.03);
            }
            // --- the faith lives (#595 slice 1) ---
            // The town's devotion drifts toward its council's god (the powers
            // that hold it shape what it worships), and a god's holy day pulls
            // the whole province toward that god a little harder. Seeded from the
            // people's patron on first touch. A turn of the prevailing faith is
            // talked of on the road, once.
            {
                use crate::model::economy::SettlementFaith;
                let day = (tick / 24) as u32;
                if settlement.faith.devotion.is_empty() {
                    settlement.faith = SettlementFaith::seeded(settlement.patron_seed_god());
                }
                let council_god = settlement.politics.dominant_faction().god();
                settlement.faith.drift_toward(council_god, 0.01);
                if let Some(holy) = crate::model::calendar::holy_day_god(day) {
                    settlement.faith.drift_toward(holy, 0.03);
                }
                let now = settlement.faith.prevailing();
                if let Some(g) = now {
                    if settlement.faith.announced.is_some() && settlement.faith.announced != Some(g)
                    {
                        completed_msgs.push(format!(
                            "{} has turned to the worship of {}.",
                            settlement.name,
                            g.label()
                        ));
                    }
                    settlement.faith.announced = Some(g);
                }
            }
            // The council turns over now and then (#556): an election, a
            // dispute, or a festival — shifting who holds it, and talked of when
            // it lands. Checked once a season per town.
            {
                let day = (tick / 24) as u32;
                if day > 0 && day.is_multiple_of(30) {
                    let id_hash = settlement
                        .id
                        .bytes()
                        .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(b as u64));
                    let ev_seed = seed ^ id_hash ^ (day as u64).wrapping_shl(20);
                    if let Some(ev) = settlement.politics.roll_leadership_event(ev_seed) {
                        use crate::model::economy::LeadershipEvent;
                        let faction = settlement.politics.dominant_faction().label();
                        completed_msgs.push(match ev {
                            LeadershipEvent::Election => format!(
                                "An election in {} — the {} take the council.",
                                settlement.name, faction
                            ),
                            LeadershipEvent::Dispute => format!(
                                "A dispute splits the council in {}; the {} hold it, but barely.",
                                settlement.name, faction
                            ),
                            LeadershipEvent::Festival => format!(
                                "A council-festival in {} — the {} are feasted.",
                                settlement.name, faction
                            ),
                        });
                    }
                }
            }

            // --- construction ---
            let builders = settlement.profession_count("carpenter")
                + settlement.profession_count("labourer")
                + settlement.profession_count("forester")
                + settlement.profession_count("miner")
                + settlement.profession_count("weaver");
            if builders > 0 {
                let underway = settlement.buildings.iter().any(|b| !b.is_complete());
                if !underway {
                    // Raise what's missing, in order of need.
                    let wanted = [
                        BuildingType::Shelter,
                        BuildingType::Hearth,
                        BuildingType::Trap,
                        BuildingType::Workshop,
                        BuildingType::Shrine,
                    ]
                    .into_iter()
                    .find(|k| !settlement.has_building(*k));
                    if let Some(kind) = wanted {
                        let bseed = seed.wrapping_add(tick) ^ (settlement.id.len() as u64) << 8;
                        settlement.buildings.push(Building::new(
                            bseed,
                            kind,
                            settlement.id.clone(),
                        ));
                    }
                }
                let workshop_boost = if settlement.has_building(BuildingType::Workshop) {
                    1.5
                } else {
                    1.0
                };
                // Raising a building needs materials, not just hands (#540
                // demand): a town draws on its Wood and Iron, and short of them
                // the work crawls. A real consequence to a goods shortage —
                // which is what makes the iron a smithless town must import
                // actually matter.
                let materials = settlement.good(crate::model::ItemType::Wood)
                    + settlement.good(crate::model::ItemType::Iron);
                let material_factor = if materials >= 2.0 { 1.0 } else { 0.4 };
                let crew = ((builders as f64) * workshop_boost * material_factor).round() as u64;
                let building_now = settlement.buildings.iter().any(|b| !b.is_complete());
                if building_now && materials >= 2.0 {
                    let w = settlement.good(crate::model::ItemType::Wood);
                    settlement
                        .goods_stock
                        .insert(crate::model::ItemType::Wood, (w - 0.3).max(0.0));
                    let i = settlement.good(crate::model::ItemType::Iron);
                    settlement
                        .goods_stock
                        .insert(crate::model::ItemType::Iron, (i - 0.2).max(0.0));
                }
                for b in settlement.buildings.iter_mut() {
                    if !b.is_complete() {
                        b.advance_construction(crew, tick);
                        if b.is_complete() {
                            completed_msgs.push(format!(
                                "They raised a new {} in {}.",
                                b.building_type.name(),
                                settlement.name
                            ));
                        }
                        break; // one project at a time
                    }
                }
            }
        }
        // The day's lawful migrants join the town they fled to (entity-first
        // slice 8). Bandits become wanderers, fed to the frontier after the
        // region loop. Done here, after the settlement loop, so the whole region
        // roster is free to write into.
        new_wanderers += region_bandits;
        for (to, mut p) in pending_migrants {
            if let Some(dest) = region.settlements.get_mut(to) {
                p.region = dest.region.clone();
                p.settlement = dest.id.clone();
                // A fed town takes them in; their need eases on arrival.
                p.needs.satisfy(crate::model::Need::Food, 0.2);
                dest.people.push(p);
                dest.population = dest.people.len() as u32;
            }
        }
        // --- trade drift: the Bronze Road smooths a good across the region
        // (#540 living economy). A good drifts from where it is plentiful toward
        // each town's fair share by size — surplus shipped, scarcity supplied —
        // conserving the regional total. System-first: no player, no routing,
        // just supply finding demand across the connected province.
        {
            use crate::model::ItemType;
            const DRIFT_RATE: f64 = 0.10;
            let total_pop: f64 = region.settlements.iter().map(|s| s.population as f64).sum();
            if total_pop > 0.0 {
                // Every good a town holds drifts toward fair shares (#678 s2):
                // the registry goods flow on the same Bronze Road as the core
                // four. Each good is smoothed independently, so iteration order
                // does not affect the result (determinism holds).
                let mut items: std::collections::HashSet<ItemType> =
                    std::collections::HashSet::new();
                for s in region.settlements.iter() {
                    items.extend(s.goods_stock.keys().copied());
                }
                for item in items {
                    let total: f64 = region.settlements.iter().map(|s| s.good(item)).sum();
                    if total <= 0.0 {
                        continue;
                    }
                    for s in region.settlements.iter_mut() {
                        let target = total * (s.population as f64 / total_pop);
                        let cur = s.good(item);
                        let next = cur - DRIFT_RATE * (cur - target);
                        s.goods_stock.insert(item, next.max(0.0));
                    }
                }
            }
        }
        // Paint grown footprints: the settlement's square of ground expands
        // with its size, clamped to the map's edge.
        for (ax, ay, n, character) in grown_footprints {
            crate::gen::town::lay_town(
                &mut region.terrain,
                ax as usize,
                ay as usize,
                n as usize,
                character,
            );
        }
        region.game_richness = (region.game_richness - richness_draw).max(0.0);
    }
    // The day's desperate take to the open country, swelling the frontier's pool
    // of wanderers — the raw material its bands muster from (#623, slice 5).
    for _ in 0..new_wanderers {
        sim.frontier.take_the_road();
    }
    for msg in completed_msgs {
        let t = sim.world.tick;
        sim.log(t, Voice::Rumor, msg);
    }
}

/// Spawn trade caravans between settlements and retire them once their goods
/// have dispersed. An arrived caravan lowers prices for the goods it carried at
/// its destination (see App::caravan_price_modifier).
/// The named cities of the wider continent (the Archive's register, by way
/// of great_cities_of_the_ages.md): the playable province is one corner of a
/// world of twelve to fifteen million, and these are where its long roads
/// lead. They never appear on the map — they appear in the goods, the talk,
/// and the caravan manifests.
pub const CANON_CITIES: &[(&str, &str)] = &[
    ("Sampa Crossing", "grain off the Basin surplus"),
    ("Vessenath", "furs, steel, and lake fish"),
    ("Halkess", "grain at the price Halkess sets"),
    ("Velkarath", "salvage and harbor-goods from the old capital"),
    ("Keuramark", "northern timber and amber"),
];

/// A canon city's head-count (population_scale_and_settlement_hierarchy.md):
/// fifteen thousand and up — an order of scale beyond any province town.
/// What a caravan off the long roads carries — the named city's specialty
/// (great_cities_of_the_ages.md): the exotic imports the province cannot make.
/// `None` for a province-town origin (it keeps its varied generated cargo).
fn canon_city_cargo(origin: &str) -> Option<Vec<(crate::model::ItemType, u32)>> {
    use crate::model::{good_id, ItemType};
    let g = |slug: &str, q: u32| good_id(slug).map(|id| (ItemType::Good(id), q));
    let cargo: Vec<(ItemType, u32)> = match origin {
        "Sampa Crossing" => [g("grain", 8)].into_iter().flatten().collect(),
        "Vessenath" => [g("fur", 3), Some((ItemType::Iron, 5)), g("fish", 4)]
            .into_iter()
            .flatten()
            .collect(),
        "Halkess" => [g("grain", 7)].into_iter().flatten().collect(),
        "Velkarath" => [Some((ItemType::Iron, 4)), g("rope", 4), g("salt", 3)]
            .into_iter()
            .flatten()
            .collect(),
        "Keuramark" => [Some((ItemType::Wood, 6)), g("amber", 2), g("tar", 3)]
            .into_iter()
            .flatten()
            .collect(),
        _ => return None,
    };
    (!cargo.is_empty()).then_some(cargo)
}

pub fn city_population(idx: usize) -> u32 {
    match idx {
        0 => 16_000, // Sampa Crossing — Basin crossroads
        1 => 21_000, // Vessenath — lake-city, steel + furs
        2 => 14_000, // Halkess — walled grain-market
        3 => 19_000, // Velkarath — the diminished old capital
        _ => 12_000, // Keuramark — northern frontier city
    }
}

/// The districts a traveller walks past in a great city — the variety the
/// province has not (#456): a city reads as quarters, not one square.
pub fn city_districts(idx: usize) -> &'static [&'static str] {
    match idx {
        0 => &[
            "the grain-wharves",
            "the long market",
            "the Basin-road caravanserai",
            "the weighhouse quarter",
        ],
        1 => &[
            "the steelworks",
            "the fur-halls",
            "the lakefront docks",
            "the smoke-quarter tenements",
        ],
        2 => &[
            "the granary ring",
            "the counting-houses",
            "the walled merchant quarter",
            "the toll-gate market",
        ],
        3 => &[
            "the fallen harbour",
            "the salvage-yards",
            "the old palace district (half-ruined)",
            "the new town within the old",
        ],
        _ => &[
            "the timber-yards",
            "the amber-traders' row",
            "the frontier market",
            "the long-winter lodges",
        ],
    }
}

/// The deeper services a great city keeps that a province town does not (#456).
pub fn city_services(idx: usize) -> &'static [&'static str] {
    match idx {
        0 => &[
            "a great market",
            "a moneychanger",
            "a caravan-hall",
            "a grain-exchange",
        ],
        1 => &[
            "a steel-forge of masters",
            "a furriers' guild",
            "a harbourmaster",
            "a great market",
        ],
        2 => &[
            "a grain-exchange",
            "a counting-house",
            "a moneychanger",
            "a court of weights",
        ],
        3 => &[
            "a salvagers' hall",
            "a scholars' archive",
            "a harbourmaster",
            "a great market",
        ],
        _ => &[
            "a timber-hall",
            "an amber-market",
            "a furriers' guild",
            "a great market",
        ],
    }
}

/// The felt arrival at a great city (#456): canon scale and character. (Moved
/// here from the journey so the city screen and the journal both read it.)
pub fn city_arrival(idx: usize) -> &'static str {
    match idx {
        0 => "Sampa Crossing sprawls where the Basin roads meet — fifteen thousand souls and more, grain-barges thick on the water, a market quarter that does not empty from dawn to dark. After a province of a few hundred, the crowd alone is a kind of vertigo.",
        1 => "Vessenath rises grey above its lake, a city of twenty thousand under a haze of forge-smoke — steel-halls and fur-markets, the cold water crowded with fishing craft. You have never seen so many roofs stand in one place.",
        2 => "Halkess holds the grain-price of the south in its fists — a walled city of merchants and granaries whose scales are the law for a hundred leagues. Coin moves through its counting-houses in rivers.",
        3 => "Velkarath broods over the harbour of the old capital — half its grandeur fallen, half still standing, salvage-crews picking the bones of the world before the Fall. It is a city haunted by how much larger it once was.",
        _ => "Keuramark stands at the treeline of the north, a frontier city of log halls and amber-traders — the last great market before the cold country, loud with timber-crews and the long-winter trade.",
    }
}

/// How likely a caravan leaving `origin` is to make for each town in `names`
/// (#560 living province): a rival town never sees the cart (weight 0), the
/// origin itself never (0), a neutral town keeps a base chance, and a partner
/// scales up with the strength of the bond. Pure so the routing is testable.
fn caravan_destination_weights(
    names: &[String],
    origin: &str,
    ties: &crate::model::province::ProvinceTies,
) -> Vec<f64> {
    use crate::model::province::TieKind;
    names
        .iter()
        .map(|n| {
            if n == origin {
                return 0.0;
            }
            if ties.tie(origin, n) == TieKind::Rival {
                0.0 // bad blood — no cart crosses
            } else {
                1.0 + 4.0 * ties.bond(origin, n).max(0.0)
            }
        })
        .collect()
}

fn tick_caravans(sim: &mut SimState) {
    let tick = sim.world.tick;
    // Imports land (#goods-phase2b): an arrived caravan unloads its cargo into
    // the destination town's stock, once — goods physically move region→region.
    // A raided cart carries nothing in. Off-map destinations (the named cities)
    // aren't on the settlement list, so their share simply leaves the province.
    let arrivals: Vec<(String, Vec<(crate::model::ItemType, u32)>)> = sim
        .caravans
        .iter_mut()
        .filter(|c| tick >= c.arrival_tick && !c.unloaded && !c.raided)
        .map(|c| {
            c.unloaded = true;
            (c.destination.clone(), c.goods.clone())
        })
        .collect();
    if !arrivals.is_empty() {
        for region in sim.world.regions.iter_mut() {
            for s in region.settlements.iter_mut() {
                let cap = s.population as f64 * 0.5;
                for (dest, goods) in &arrivals {
                    if &s.name == dest {
                        for (item, qty) in goods {
                            s.produce_good(*item, *qty as f64, cap);
                        }
                    }
                }
            }
        }
    }
    // Goods disperse ~2 days after arrival.
    sim.caravans.retain(|c| tick < c.arrival_tick + 48);

    if !tick.is_multiple_of(24) {
        return;
    }
    let towns: Vec<(String, f64)> = sim
        .world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .map(|s| (s.name.clone(), s.politics.openness()))
        .collect();
    let names: Vec<String> = towns.iter().map(|(n, _)| n.clone()).collect();
    if names.len() < 2 {
        return;
    }
    // Towns under plague close their roads (#604 slice 3): they send no carts,
    // and no cart rides into a stricken town.
    let plagued: std::collections::HashSet<String> = sim
        .world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .filter(|s| s.plague_days > 0)
        .map(|s| s.name.clone())
        .collect();
    let day = tick / 24;
    let mut rng =
        SeedRng::new(sim.world.seed.wrapping_add(day.wrapping_mul(7919))).fork_for("caravan");
    // The roads run thick in Green and thin in Frost (#570): the daily chance a
    // cart sets out turns with the season, in place of the old flat cadence.
    let season = crate::model::Season::from_day(day as u32);
    if rng.gen_range(100) >= season.caravan_chance() {
        return;
    }
    // A third of the caravans ride the LONG roads: they come from the named
    // cities of the continent the province is a corner of.
    let origin = if rng.gen_range(3) == 0 {
        let (city, _) = CANON_CITIES[rng.gen_range(CANON_CITIES.len() as u32) as usize];
        city.to_string()
    } else {
        // Which town sends the cart is coloured by its council (#560): a
        // Traders-led town throws its gates wide and ships more often, an
        // Elders-led town keeps to itself. Weighted by each town's openness.
        let total: f64 = towns.iter().map(|(_, o)| *o).sum();
        let mut roll = rng.gen_f64() * total;
        let mut chosen = 0usize;
        for (i, (_, o)) in towns.iter().enumerate() {
            roll -= o;
            if roll <= 0.0 {
                chosen = i;
                break;
            }
        }
        names[chosen].clone()
    };
    // Where the caravan goes is no longer a blind draw (#560 living province):
    // from one of the province's own towns it rides the standing roads — a
    // partner town is far likelier to see the cart, a rival town never does.
    // (Long-road caravans from the off-map cities still pick a destination at
    // large.) The feedback closes the loop: the partnerships the carts deepen
    // then pull still more carts.
    let origin_is_local = names.iter().any(|n| n == &origin);
    let dest = if origin_is_local {
        let weights = caravan_destination_weights(&names, &origin, &sim.province_ties);
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            // Everyone a rival (or only self) — fall back to any other town.
            let mut d = rng.gen_range(names.len() as u32) as usize;
            if names[d] == origin {
                d = (d + 1) % names.len();
            }
            names[d].clone()
        } else {
            let mut roll = rng.gen_f64() * total;
            let mut chosen = 0usize;
            for (i, w) in weights.iter().enumerate() {
                roll -= w;
                if roll <= 0.0 {
                    chosen = i;
                    break;
                }
            }
            names[chosen].clone()
        }
    } else {
        let mut d = rng.gen_range(names.len() as u32) as usize;
        if names[d] == origin {
            d = (d + 1) % names.len();
        }
        names[d].clone()
    };
    // A caravan between two of the province's own towns deepens their
    // partnership (#560): the more carts cross between them, the closer they
    // grow — and an open, Traders-led origin forges the bond faster than an
    // insular, Elders-led one. Caravans down the long roads from the named
    // cities of the continent are not province ties — those cities are off the
    // map.
    if origin_is_local {
        let openness = towns
            .iter()
            .find(|(n, _)| n == &origin)
            .map(|(_, o)| *o)
            .unwrap_or(1.0);
        sim.province_ties.nudge(&origin, &dest, 0.05 * openness);
    }
    // The quarantine: a plagued origin keeps its carts home (its closed roads
    // are talked of now and then), and no cart rides into a plagued town.
    if plagued.contains(&origin) {
        if rng.gen_range(100) < 20 {
            let t = sim.world.tick;
            sim.log(
                t,
                Voice::Rumor,
                format!("{origin} has closed its roads against the sickness."),
            );
        }
        return;
    }
    if plagued.contains(&dest) {
        return;
    }
    let mut caravan = crate::model::economy::Caravan::generate(
        sim.world.seed.wrapping_add(tick),
        origin,
        dest,
        tick,
    );
    // Off the long roads, a caravan carries its city's specialty, not random
    // wares (#goods-phase2b): the exotic imports the province can't make —
    // Keuramark amber, Sampa grain, Vessenath furs-and-steel.
    if let Some(cargo) = canon_city_cargo(&caravan.origin) {
        caravan.goods = cargo;
    }
    // War on the contested edge (#579 slice 1): while the province's polity is
    // at tension with its rival, the roads are watched and raided — a share of
    // the carts never reach the market, so the goods they would have carried
    // stay dear. A taken caravan is talked of on the road. Deterministic per
    // day, on the same season clock the rumor and the levy use.
    let season_ord = (day as u32 / 30) % 4;
    let war_year = day as u32 / 120;
    if sim
        .world
        .polity
        .in_tension(sim.world.seed, season_ord, war_year)
        && rng.gen_range(100) < 35
    {
        let t = sim.world.tick;
        sim.log(
            t,
            Voice::Rumor,
            format!(
                "A caravan was taken on the road to {} — the war makes the carts a prize.",
                caravan.destination
            ),
        );
        return;
    }
    sim.caravans.push(caravan);
}

/// The province's web of standings drifts each day (#560): towns that make the
/// same good in the same region grate toward rivalry, and every tie fades a
/// little toward neutral when no fresh trade or friction keeps it up. Partnership
/// is fed by the caravans (see `tick_caravans`); this is the friction and the
/// forgetting.
fn tick_province_ties(sim: &mut SimState) {
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    // Competition: within a region, two living towns whose signature good is the
    // same compete in one market — a small daily push toward rivalry.
    for region in &sim.world.regions {
        let makers: Vec<(&str, crate::model::ItemType)> = region
            .settlements
            .iter()
            .filter(|s| s.population > 0)
            .filter_map(|s| s.signature_good().map(|g| (s.name.as_str(), g)))
            .collect();
        for i in 0..makers.len() {
            for j in (i + 1)..makers.len() {
                if makers[i].1 == makers[j].1 {
                    sim.province_ties.nudge(makers[i].0, makers[j].0, -0.02);
                }
            }
        }
    }
    // The slow forgetting: a tie not kept up lapses toward neutral.
    sim.province_ties.decay(0.01);

    // A partnership that has just formed or a rivalry that has just hardened is
    // talked of on the roads (#560 slice 4) — the province's trade-news.
    let news = sim.province_ties.newly_crossed();
    for (a, b, kind) in news {
        use crate::model::province::TieKind;
        let line = match kind {
            TieKind::Partner => {
                format!("{a} and {b} have grown close — their caravans run thick between them.")
            }
            TieKind::Rival => {
                format!("Bad blood between {a} and {b} now — no cart crosses between them.")
            }
            TieKind::Neutral => continue,
        };
        let t = sim.world.tick;
        sim.log(t, Voice::Rumor, line);
    }
}

/// Winter relief along the partner-roads (#570 slice 2): in deep Frost, a town
/// whose stores are full sends grain to a *partner* town gone short — the
/// living province caring for its own through the hard season. The relief moves
/// real meals between the two, deepens the partnership that carried it, rides as
/// a relief caravan, and is talked of on the road. System-first: the towns do
/// this with no player input, and only between towns already bound as partners.
fn tick_winter_relief(sim: &mut SimState) {
    use crate::model::economy::Caravan;
    use crate::model::province::TieKind;
    use crate::model::ItemType;
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = (tick / 24) as u32;
    if crate::model::Season::from_day(day) != crate::model::Season::Frost {
        return;
    }
    // Snapshot each living town's stores and where it sits, by name.
    let mut loc: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    let mut store: std::collections::HashMap<String, (f64, u32)> = std::collections::HashMap::new();
    for (ri, region) in sim.world.regions.iter().enumerate() {
        for (si, s) in region.settlements.iter().enumerate() {
            if s.population == 0 {
                continue;
            }
            loc.insert(s.name.clone(), (ri, si));
            store.insert(s.name.clone(), (s.food_stock, s.population));
        }
    }
    let per_head = |name: &str| -> f64 {
        store
            .get(name)
            .map(|(f, p)| f / (*p).max(1) as f64)
            .unwrap_or(0.0)
    };
    // Partner pairs only, in a sorted order so the relief is deterministic.
    let mut pairs: Vec<(String, String)> = sim
        .province_ties
        .bonds
        .keys()
        .filter(|(a, b)| {
            sim.province_ties.tie(a, b) == TieKind::Partner
                && store.contains_key(a)
                && store.contains_key(b)
        })
        .cloned()
        .collect();
    pairs.sort();
    // A full town comfortably feeds itself above this per-head; a short one
    // below the lower mark needs help. Relief flows from the surplus to the
    // shortfall, never beyond either bound.
    const COMFORTABLE: f64 = 1.5;
    const SHORT: f64 = 0.7;
    let mut transfers: Vec<(String, String, f64)> = Vec::new();
    for (a, b) in &pairs {
        // Whichever side has the surplus is the donor; the short side, the
        // recipient. If neither is short or neither has spare, no cart rolls.
        let (donor, recip) = if per_head(a) >= per_head(b) {
            (a, b)
        } else {
            (b, a)
        };
        let spare = (per_head(donor) - COMFORTABLE) * store[donor].1 as f64;
        let need = (SHORT - per_head(recip)) * store[recip].1 as f64;
        let relief = spare.min(need);
        if relief >= 1.0 {
            transfers.push((donor.clone(), recip.clone(), relief));
        }
    }
    for (donor, recip, relief) in transfers {
        if let Some(&(ri, si)) = loc.get(&donor) {
            sim.world.regions[ri].settlements[si].food_stock -= relief;
        }
        if let Some(&(ri, si)) = loc.get(&recip) {
            sim.world.regions[ri].settlements[si].food_stock += relief;
        }
        // The kindness deepens the partnership that carried it.
        sim.province_ties.nudge(&donor, &recip, 0.04);
        // A relief caravan of grain rides the partner-road.
        let mut caravan = Caravan::generate(
            sim.world
                .seed
                .wrapping_add(tick)
                .wrapping_add(si_hash(&recip)),
            donor.clone(),
            recip.clone(),
            tick,
        );
        caravan.goods = vec![(ItemType::Food, (relief as u32).max(1))];
        sim.caravans.push(caravan);
        sim.log(
            tick,
            Voice::Rumor,
            format!(
                "In the hard winter {donor} sent grain to {recip} — the partner-road runs even through the frost."
            ),
        );
    }
}

/// Rivalries with teeth (#579 slice 2): a deep ProvinceTie rivalry now and then
/// spills into a raid. The stronger of the two rival towns falls on the weaker —
/// carrying off food and a trade good, fraying the victim's sense of safety —
/// and the raid deepens the rivalry it sprang from, a feedback the peace must
/// break. System-first and deterministic: the bonds are read in sorted order and
/// the roll is seeded, so a given world always raids the same way.
/// Plague rides the caravans (#604 slice 2): a cart that set out from a plagued
/// town can carry the contagion to where it lands — the trade that carries goods
/// carries the sickness — so a plague creeps along the busy partner-roads, town
/// to town. The road it rode is talked of. System-first and deterministic: the
/// caravans are read in order, the roll seeded per caravan per day, the new
/// outbreaks applied after.
fn tick_plague_spread(sim: &mut SimState) {
    use crate::model::Need;
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = tick / 24;
    let seed = sim.world.seed;
    let mut loc: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    let mut plagued: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    for (ri, region) in sim.world.regions.iter().enumerate() {
        for (si, s) in region.settlements.iter().enumerate() {
            if s.population == 0 {
                continue;
            }
            loc.insert(s.name.clone(), (ri, si));
            plagued.insert(s.name.clone(), s.plague_days > 0);
        }
    }
    // Read the carts in order; a cart out of a plagued town can carry the
    // sickness to a healthy destination.
    let mut new_outbreaks: Vec<(String, String)> = Vec::new();
    for c in &sim.caravans {
        if plagued.get(&c.origin).copied().unwrap_or(false)
            && !plagued.get(&c.destination).copied().unwrap_or(true)
        {
            let mut rng =
                SeedRng::new(seed ^ si_hash(&c.id)).fork_for(&format!("plague-ride-{day}"));
            if rng.gen_range(100) < 30 {
                new_outbreaks.push((c.origin.clone(), c.destination.clone()));
            }
        }
    }
    let mut msgs: Vec<String> = Vec::new();
    for (from, to) in new_outbreaks {
        if let Some(&(ri, si)) = loc.get(&to) {
            let s = &mut sim.world.regions[ri].settlements[si];
            if s.plague_days == 0 {
                s.plague_days = 1;
                for person in s.people.iter_mut() {
                    person.needs.decay(Need::Safety, 0.05);
                }
                msgs.push(format!(
                    "The sickness rode the road from {from} to {to} — a cart carried more than goods."
                ));
            }
        }
    }
    for m in msgs {
        sim.log(tick, Voice::Rumor, m);
    }
}

/// A town can fall to a plague (#604 slice 1): under the conditions that breed
/// it — a famine-weakened town, or a plague-year season (#417) — a sickness
/// breaks out, grips the town for a span (sickening its people and taking a
/// toll), then runs its course. Talked of at its start and its end. System-first
/// and deterministic: the outbreak roll is seeded per town per day.
fn tick_plague(sim: &mut SimState) {
    use crate::model::{Need, Season, WorldEvent};
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = (tick / 24) as u32;
    let seed = sim.world.seed;
    let season = Season::from_day(day);
    let plague_year =
        WorldEvent::current(seed, season, day / Season::YEAR_DAYS) == Some(WorldEvent::PlagueYear);
    // A province at war breeds sickness too (#604 slice 3): the crowding and
    // displacement of a war season raise the odds an outbreak catches.
    let at_war = sim.world.polity.in_tension(seed, (day / 30) % 4, day / 120);
    let mut msgs: Vec<String> = Vec::new();
    for region in sim.world.regions.iter_mut() {
        for s in region.settlements.iter_mut() {
            if s.population == 0 {
                continue;
            }
            if s.plague_days == 0 {
                // The conditions that breed a plague raise its odds: a
                // famine-weakened town, a war season, and above all a
                // plague-year.
                let mut rng =
                    SeedRng::new(seed ^ si_hash(&s.name)).fork_for(&format!("plague-{day}"));
                let mut chance = 3u32; // per thousand, in an ordinary season
                if s.famine_days > 0 {
                    chance += 8;
                }
                if at_war {
                    chance += 5;
                }
                if plague_year {
                    chance += 15;
                }
                if rng.gen_range(1000) < chance {
                    s.plague_days = 1;
                    for person in s.people.iter_mut() {
                        person.needs.decay(Need::Safety, 0.05);
                    }
                    msgs.push(format!("A sickness has broken out in {}.", s.name));
                }
            } else {
                // The plague burns: it sickens the people and takes a small
                // toll each day, then runs its course after a couple of weeks.
                s.plague_days += 1;
                for person in s.people.iter_mut() {
                    person.needs.decay(Need::Safety, 0.04);
                    person.needs.decay(Need::Care, 0.03);
                }
                let toll = ((s.population as f64) * 0.004).ceil() as usize;
                s.remove_residents(toll);
                // A smaller town holds less: keep the goods cap (population/2)
                // in step with the toll so stores never sit above what the
                // shrunken town can hold.
                let cap = s.population as f64 * 0.5;
                for v in s.goods_stock.values_mut() {
                    *v = v.min(cap);
                }
                if s.plague_days >= 16 {
                    s.plague_days = 0;
                    msgs.push(format!("The sickness in {} has run its course.", s.name));
                }
            }
        }
    }
    for m in msgs {
        sim.log(tick, Voice::Rumor, m);
    }
}

/// Revival and schism (#595 slice 3): now and then a town's faith shifts hard.
/// A revival is a sudden fervor that drives the town toward one god; a schism is
/// two gods contending near-equally for its devotion, which leaves the town
/// uneasy — its people's sense of safety frays. Both are talked of on the road.
/// Deterministic: the roll is seeded per town per day.
fn tick_faith_upheavals(sim: &mut SimState) {
    use crate::model::{GodName, Need};
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = tick / 24;
    let seed = sim.world.seed;
    let mut msgs: Vec<String> = Vec::new();
    for region in sim.world.regions.iter_mut() {
        for s in region.settlements.iter_mut() {
            if s.population == 0 {
                continue;
            }
            let mut rng =
                SeedRng::new(seed ^ si_hash(&s.name)).fork_for(&format!("faith-upheaval-{day}"));
            if rng.gen_range(1000) < 12 {
                // A revival: the town turns fervently to one of the Five.
                let target = GodName::all()[rng.gen_range(5) as usize];
                s.faith.drift_toward(target, 0.3);
                // Mark the new prevailing as announced so the ordinary turn-of-
                // faith line does not also fire for the same shift.
                s.faith.announced = s.faith.prevailing();
                msgs.push(format!(
                    "A revival sweeps {} — they turn fervent to {}.",
                    s.name,
                    target.label()
                ));
            } else {
                // A schism: the top two gods contend near-equally, and the town
                // lives uneasy under it.
                let mut tops: Vec<(GodName, f64)> = GodName::all()
                    .iter()
                    .map(|&g| (g, s.faith.get(g)))
                    .collect();
                tops.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                if tops[0].1 > 0.28 && (tops[0].1 - tops[1].1) < 0.05 {
                    for person in s.people.iter_mut() {
                        person.needs.decay(Need::Safety, 0.02);
                    }
                    if rng.gen_range(100) < 8 {
                        msgs.push(format!(
                            "{} is split between the worship of {} and {} — an uneasy town.",
                            s.name,
                            tops[0].0.label(),
                            tops[1].0.label()
                        ));
                    }
                }
            }
        }
    }
    for m in msgs {
        sim.log(tick, Voice::Rumor, m);
    }
}

/// Faith rides the roads (#595 slice 2): devotion spreads like trade. A town
/// bound to a partner of another god slowly takes on some of that god's
/// devotion — the caravans carry belief, not only goods — so a faith can sweep
/// a whole partner bloc. System-first and deterministic: each town's prevailing
/// god is read into a snapshot first, the partner pairs are taken in sorted
/// order, and the drift is applied after, so the spread runs the same every run.
fn tick_faith_spread(sim: &mut SimState) {
    use crate::model::province::TieKind;
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let mut loc: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    let mut god: std::collections::HashMap<String, crate::model::GodName> =
        std::collections::HashMap::new();
    for (ri, region) in sim.world.regions.iter().enumerate() {
        for (si, s) in region.settlements.iter().enumerate() {
            if s.population == 0 {
                continue;
            }
            loc.insert(s.name.clone(), (ri, si));
            god.insert(s.name.clone(), s.prevailing_god());
        }
    }
    let mut pairs: Vec<(String, String)> = sim
        .province_ties
        .bonds
        .keys()
        .filter(|(a, b)| {
            sim.province_ties.tie(a, b) == TieKind::Partner
                && god.contains_key(a)
                && god.contains_key(b)
        })
        .cloned()
        .collect();
    pairs.sort();
    // Collect the drifts off the snapshot, then apply — so neither town's shift
    // colours what its partner carries this day.
    let mut drifts: Vec<(String, crate::model::GodName)> = Vec::new();
    for (a, b) in &pairs {
        let (ga, gb) = (god[a], god[b]);
        if ga != gb {
            drifts.push((a.clone(), gb));
            drifts.push((b.clone(), ga));
        }
    }
    for (town, target) in drifts {
        if let Some(&(ri, si)) = loc.get(&town) {
            sim.world.regions[ri].settlements[si]
                .faith
                .drift_toward(target, 0.01);
        }
    }
}

fn tick_raids(sim: &mut SimState) {
    use crate::model::province::TieKind;
    use crate::model::Need;
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = (tick / 24) as u32;
    // Where each living town sits, and how many it musters (population stands in
    // for strength — the bigger town raids the smaller).
    let mut loc: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    let mut pop: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for (ri, region) in sim.world.regions.iter().enumerate() {
        for (si, s) in region.settlements.iter().enumerate() {
            if s.population == 0 {
                continue;
            }
            loc.insert(s.name.clone(), (ri, si));
            pop.insert(s.name.clone(), s.population);
        }
    }
    // Only the hard rivalries raid — a bond well past the rival threshold.
    const DEEP_RIVAL: f64 = -0.7;
    let mut pairs: Vec<(String, String)> = sim
        .province_ties
        .bonds
        .iter()
        .filter(|(_, &v)| v <= DEEP_RIVAL)
        .map(|(k, _)| k.clone())
        .filter(|(a, b)| {
            sim.province_ties.tie(a, b) == TieKind::Rival
                && loc.contains_key(a)
                && loc.contains_key(b)
        })
        .collect();
    pairs.sort();
    for (a, b) in pairs {
        // The stronger town is the aggressor; ties broken by name for
        // determinism.
        let (raider, victim) = if pop[&a] >= pop[&b] {
            (a.clone(), b.clone())
        } else {
            (b.clone(), a.clone())
        };
        // A seeded daily roll: raids are occasional, not constant — roughly one
        // week in six a deep rivalry boils over.
        let mut rng = SeedRng::new(
            sim.world
                .seed
                .wrapping_add(si_hash(&raider) ^ si_hash(&victim)),
        )
        .fork_for(&format!("raid-{day}"));
        if rng.gen_range(100) >= 16 {
            continue;
        }
        let (vri, vsi) = loc[&victim];
        // Carry off a fifth of the victim's stores and a measure of its chief
        // trade good; the loot lands in the raider's hands.
        let looted_food;
        let looted_good;
        {
            let v = &mut sim.world.regions[vri].settlements[vsi];
            looted_food = v.food_stock * 0.2;
            v.food_stock -= looted_food;
            looted_good = v.signature_good().map(|g| (g, (v.good(g) * 0.3).floor()));
            if let Some((g, amt)) = looted_good {
                if amt > 0.0 {
                    v.produce_good(g, -amt, f64::MAX);
                }
            }
            // The raid frays the town's nerve.
            for person in v.people.iter_mut() {
                person.needs.decay(Need::Safety, 0.08);
            }
        }
        if let Some(&(rri, rsi)) = loc.get(&raider) {
            let r = &mut sim.world.regions[rri].settlements[rsi];
            r.food_stock += looted_food;
            if let Some((g, amt)) = looted_good {
                if amt > 0.0 {
                    let cap = r.population as f64 * 0.5;
                    r.produce_good(g, amt, cap);
                }
            }
        }
        // The raid deepens the bad blood it came from.
        sim.province_ties.nudge(&raider, &victim, -0.05);
        sim.log(
            tick,
            Voice::Rumor,
            format!("{raider} fell on {victim} in the night — stores carried off, and no peace between them."),
        );
        // The bloc answers (#579 slice 3): the victim's deep allies take the
        // raid as their own quarrel and sour on the raider. Read the standings
        // first, then nudge — a raid on one is bad blood for all of them.
        let allies: Vec<String> = sim
            .province_ties
            .bonds
            .iter()
            .filter(|(_, &v)| v >= DEEP_ALLY)
            .filter_map(|((p, q), _)| {
                if p == &victim && *q != raider && loc.contains_key(q) {
                    Some(q.clone())
                } else if q == &victim && *p != raider && loc.contains_key(p) {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect();
        if !allies.is_empty() {
            for ally in &allies {
                sim.province_ties.nudge(ally, &raider, -0.04);
            }
            sim.log(
                tick,
                Voice::Rumor,
                format!("{victim}'s friends have not forgotten the raid — {raider} finds the roads colder for it."),
            );
        }
    }
}

/// The standing at which a partnership becomes an alliance (#579 slice 3) — deep
/// enough to feed its own year-round and to answer a raid on a partner.
const DEEP_ALLY: f64 = 0.7;

/// Year-round mutual aid among allies (#579 slice 3): winter relief feeds an
/// ordinary partner only through the Frost, but a deep alliance feeds its own in
/// any season — a town gone to famine is sent grain by its sworn partners
/// whatever the time of year. The relief deepens the alliance, rides as a
/// caravan, and is talked of. System-first and deterministic (sorted pairs).
fn tick_alliance_relief(sim: &mut SimState) {
    use crate::model::economy::Caravan;
    use crate::model::ItemType;
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let mut loc: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    let mut store: std::collections::HashMap<String, (f64, u32)> = std::collections::HashMap::new();
    for (ri, region) in sim.world.regions.iter().enumerate() {
        for (si, s) in region.settlements.iter().enumerate() {
            if s.population == 0 {
                continue;
            }
            loc.insert(s.name.clone(), (ri, si));
            store.insert(s.name.clone(), (s.food_stock, s.population));
        }
    }
    let per_head = |name: &str| -> f64 {
        store
            .get(name)
            .map(|(f, p)| f / (*p).max(1) as f64)
            .unwrap_or(0.0)
    };
    let mut pairs: Vec<(String, String)> = sim
        .province_ties
        .bonds
        .iter()
        .filter(|(_, &v)| v >= DEEP_ALLY)
        .map(|(k, _)| k.clone())
        .filter(|(a, b)| store.contains_key(a) && store.contains_key(b))
        .collect();
    pairs.sort();
    // A town below this per-head is in famine; an ally above the comfortable
    // mark can spare grain. Aid flows from the surplus to the hunger.
    const COMFORTABLE: f64 = 1.5;
    const FAMINE: f64 = 0.4;
    let mut transfers: Vec<(String, String, f64)> = Vec::new();
    for (a, b) in &pairs {
        let (donor, recip) = if per_head(a) >= per_head(b) {
            (a, b)
        } else {
            (b, a)
        };
        if per_head(recip) >= FAMINE {
            continue;
        }
        let spare = (per_head(donor) - COMFORTABLE) * store[donor].1 as f64;
        let need = (FAMINE - per_head(recip)) * store[recip].1 as f64;
        let aid = spare.min(need);
        if aid >= 1.0 {
            transfers.push((donor.clone(), recip.clone(), aid));
        }
    }
    for (donor, recip, aid) in transfers {
        if let Some(&(ri, si)) = loc.get(&donor) {
            sim.world.regions[ri].settlements[si].food_stock -= aid;
        }
        if let Some(&(ri, si)) = loc.get(&recip) {
            sim.world.regions[ri].settlements[si].food_stock += aid;
        }
        sim.province_ties.nudge(&donor, &recip, 0.04);
        let mut caravan = Caravan::generate(
            sim.world
                .seed
                .wrapping_add(tick)
                .wrapping_add(si_hash(&recip)),
            donor.clone(),
            recip.clone(),
            tick,
        );
        caravan.goods = vec![(ItemType::Food, (aid as u32).max(1))];
        sim.caravans.push(caravan);
        sim.log(
            tick,
            Voice::Rumor,
            format!("{donor} sent grain to {recip} in its hunger — the alliance feeds its own, season be damned."),
        );
    }
}

/// A small stable hash of a name, to spread relief-caravan seeds apart within a
/// single day's relief so two reliefs do not collide on one id.
fn si_hash(name: &str) -> u64 {
    crate::rng::fnv1a_hash(name)
}

/// Remove structures that have fully weathered away (decay ratio >= 1.0).
/// Tarp/lean-to return None from decay_tick and are never removed here.
fn tick_structure_decay(sim: &mut SimState) {
    let tick = sim.world.tick;
    let alive = |s: &crate::sim::structures::Structure| {
        s.decay_tick(tick, 24).is_none_or(|ratio| ratio < 1.0)
    };
    for region in &mut sim.world.regions {
        region.structures.retain(&alive);
    }
    sim.structures.retain(&alive);
}

fn tick_build_sites(sim: &mut SimState) {
    use crate::sim::structures::Structure;
    let tick = sim.world.tick;
    let mut completed = Vec::new();
    for site in &mut sim.build_sites {
        // Big builds rise only under working hands (see App::work_site);
        // camps and shelters finish on their own short clocks.
        if site.kind.needs_labor() {
            if site.hours_done >= site.kind.build_hours() {
                // fall through to completion below
            } else {
                continue;
            }
        } else {
            site.hours_done += 1;
        }
        if site.hours_done >= site.kind.build_hours() {
            completed.push(Structure {
                kind: site.kind,
                region_idx: site.region_idx,
                x: site.x,
                y: site.y,
                built_tick: tick,
                last_maintenance_tick: tick,
                // A shrine keeps the name of the god it was raised to.
                name: site.dedication.clone(),
                is_npc_built: false,
                stash: Default::default(),
            });
        }
    }
    sim.build_sites
        .retain(|s| s.hours_done < s.kind.build_hours());
    for structure in completed {
        if let Some(region) = sim.world.regions.get_mut(structure.region_idx) {
            region.structures.push(structure.clone());
        }
        sim.structures.push(structure);
    }
}

fn tick_npc_illness(sim: &mut SimState, current_tick: u64) {
    use crate::sim::illness;

    // Two-rate LOD: sickness is checked live in the active region; distant
    // regions are caught up once a day (this is the heaviest hourly per-person
    // pass at province scale).
    let active = sim.active_region;
    let person_info: Vec<(usize, usize)> = sim
        .world
        .regions
        .iter()
        .enumerate()
        .filter(|(ri, _)| region_tick_mode(*ri, active, current_tick) != RegionTick::Skip)
        .flat_map(|(ri, region)| {
            region
                .settlements
                .iter()
                .enumerate()
                .map(move |(si, _)| (ri, si))
        })
        .collect();

    for (ri, si) in person_info {
        let has_healer = sim
            .world
            .regions
            .get(ri)
            .and_then(|r| r.settlements.get(si))
            .map(illness::settlement_has_healer)
            .unwrap_or(false);
        let terrain = sim
            .world
            .regions
            .get(ri)
            .and_then(|r| r.terrain.get(0, 0))
            .unwrap_or(crate::model::Terrain::Grass);
        let settlement = match sim
            .world
            .regions
            .get_mut(ri)
            .and_then(|r| r.settlements.get_mut(si))
        {
            Some(s) => s,
            None => continue,
        };
        let person_count = settlement.people.len();
        let mut new_illnesses: Vec<(usize, crate::model::ActiveDisease)> = Vec::new();

        for i in 0..person_count {
            illness::apply_illness_effects(&mut settlement.people[i], current_tick);
        }

        let ill_count = settlement
            .people
            .iter()
            .filter(|p| !p.illnesses.is_empty())
            .count();
        let cap = (settlement.people.len().max(1) * 30 / 100).max(1);

        if ill_count >= cap {
            continue;
        }

        for i in 0..person_count {
            let person = &settlement.people[i];
            if person.illnesses.len() >= 2 {
                continue;
            }
            let person_id_bytes = person.id.as_bytes();
            let mut seed_val: u64 = sim.world.seed;
            for &b in person_id_bytes.iter().take(8) {
                seed_val = seed_val.wrapping_shl(8).wrapping_add(b as u64);
            }
            seed_val = seed_val.wrapping_add(current_tick);
            let existing = person.illnesses.len();
            if let Some(disease) = illness::tick_illness(
                seed_val,
                current_tick,
                terrain,
                &person.needs,
                0,
                has_healer,
                existing,
            )
            .filter(|d| {
                d.disease != crate::model::Disease::ChildbirthComplication
                    || illness::can_contract_childbirth(&person.sex, &person.age_band)
            }) {
                new_illnesses.push((i, disease));
            }
        }

        for (i, disease) in new_illnesses {
            if settlement
                .people
                .iter()
                .filter(|p| !p.illnesses.is_empty())
                .count()
                < cap
            {
                settlement.people[i].illnesses.push(disease);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::gen::world::generate_world_capped;

    /// Soak cap for lib tests: build a small world (≤300 souls/settlement) so
    /// tests step real entities through the same paths without paying the whole
    /// 8.5k–121k province each time. Deterministic; magnitude-independent tests
    /// only.
    const TEST_POP_CAP: Option<usize> = Some(300);
    fn generate_world(seed: u64, charts: &Charts) -> World {
        generate_world_capped(seed, charts, TEST_POP_CAP)
    }
    fn test_sim(seed: u64, charts: Charts) -> SimState {
        SimState::new_capped(seed, charts, TEST_POP_CAP)
    }

    #[test]
    fn caravan_routes_follow_the_province_ties() {
        use crate::model::province::ProvinceTies;
        let names = vec![
            "Home".to_string(),
            "Partner".to_string(),
            "Neutral".to_string(),
            "Rival".to_string(),
        ];
        let mut ties = ProvinceTies::default();
        ties.nudge("Home", "Partner", 0.8);
        ties.nudge("Home", "Rival", -0.8);
        let w = caravan_destination_weights(&names, "Home", &ties);
        assert_eq!(w[0], 0.0, "the origin never ships to itself");
        assert_eq!(w[3], 0.0, "a rival town never sees the cart");
        assert!(w[1] > w[2], "a partner is favoured over a neutral town");
        assert!(w[2] > 0.0, "a neutral town keeps a base chance");
    }

    #[test]
    fn winter_relief_flows_from_a_full_partner_to_a_short_one() {
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        // Two real towns of the generated province.
        let mut names = Vec::new();
        'outer: for region in &sim.world.regions {
            for s in &region.settlements {
                if s.population > 1 {
                    names.push(s.name.clone());
                    if names.len() == 2 {
                        break 'outer;
                    }
                }
            }
        }
        assert_eq!(names.len(), 2, "the test world has at least two towns");
        let (full, short) = (names[0].clone(), names[1].clone());
        // Bind them as partners and set their stores: one comfortably full, one
        // gone empty.
        sim.province_ties.nudge(&full, &short, 0.8);
        let set_food = |sim: &mut SimState, name: &str, f: f64| {
            for region in sim.world.regions.iter_mut() {
                for s in region.settlements.iter_mut() {
                    if s.name == name {
                        s.food_stock = f * s.population.max(1) as f64;
                    }
                }
            }
        };
        let get_food = |sim: &SimState, name: &str| -> f64 {
            sim.world
                .regions
                .iter()
                .flat_map(|r| r.settlements.iter())
                .find(|s| s.name == name)
                .map(|s| s.food_stock)
                .unwrap_or(0.0)
        };
        set_food(&mut sim, &full, 3.0);
        set_food(&mut sim, &short, 0.0);
        let short_before = get_food(&sim, &short);
        let full_before = get_food(&sim, &full);
        // A day in deep Frost (day-of-year 70).
        sim.world.tick = 70 * 24;
        tick_winter_relief(&mut sim);
        assert!(
            get_food(&sim, &short) > short_before,
            "the short partner receives grain"
        );
        assert!(
            get_food(&sim, &full) < full_before,
            "the relief comes out of the full town's stores"
        );
        assert!(
            sim.caravans
                .iter()
                .any(|c| c.origin == full && c.destination == short),
            "a relief caravan rides the partner-road"
        );
        // Out of season, no relief moves at all.
        let mut summer = test_sim(42, charts::load_charts().unwrap());
        summer.province_ties.nudge(&full, &short, 0.8);
        set_food(&mut summer, &full, 3.0);
        set_food(&mut summer, &short, 0.0);
        summer.world.tick = 40 * 24; // Green
        let caravans_before = summer.caravans.len();
        tick_winter_relief(&mut summer);
        assert_eq!(
            summer.caravans.len(),
            caravans_before,
            "relief is a winter thing — none flows in Green"
        );
    }

    #[test]
    fn a_plagued_town_closes_its_roads() {
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        // Plague every town: with no healthy origin or destination, the
        // quarantine should keep every cart home.
        for region in sim.world.regions.iter_mut() {
            for s in region.settlements.iter_mut() {
                s.plague_days = 5;
            }
        }
        sim.caravans.clear();
        for d in 1..=60u64 {
            sim.world.tick = d * 24;
            tick_caravans(&mut sim);
        }
        assert!(
            sim.caravans.is_empty(),
            "no cart sets out while every town is under quarantine"
        );
    }

    #[test]
    fn plague_rides_a_caravan_to_a_healthy_town() {
        use crate::model::economy::Caravan;
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        let mut names = Vec::new();
        'outer: for region in &sim.world.regions {
            for s in &region.settlements {
                if s.population > 1 {
                    names.push(s.name.clone());
                    if names.len() == 2 {
                        break 'outer;
                    }
                }
            }
        }
        assert_eq!(names.len(), 2);
        let (sick, healthy) = (names[0].clone(), names[1].clone());
        let plague_of = |sim: &SimState, name: &str| -> u32 {
            sim.world
                .regions
                .iter()
                .flat_map(|r| r.settlements.iter())
                .find(|s| s.name == name)
                .map(|s| s.plague_days)
                .unwrap_or(0)
        };
        // One town plagued; a cart rides from it to the healthy one each day.
        for region in sim.world.regions.iter_mut() {
            for s in region.settlements.iter_mut() {
                if s.name == sick {
                    s.plague_days = 3;
                }
            }
        }
        let mut caught = false;
        for d in 1..=40u64 {
            sim.world.tick = d * 24;
            // Refresh a fresh cart from the sick town each day.
            sim.caravans.clear();
            sim.caravans.push(Caravan::generate(
                sim.world.seed.wrapping_add(d),
                sick.clone(),
                healthy.clone(),
                d * 24,
            ));
            // Keep the source plagued so it can keep carrying.
            for region in sim.world.regions.iter_mut() {
                for s in region.settlements.iter_mut() {
                    if s.name == sick {
                        s.plague_days = 3;
                    }
                }
            }
            tick_plague_spread(&mut sim);
            if plague_of(&sim, &healthy) > 0 {
                caught = true;
                break;
            }
        }
        assert!(caught, "the plague rides the caravan to the healthy town");
    }

    #[test]
    fn a_plague_breaks_out_and_runs_its_course() {
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        // Weaken every town with famine so a plague is likely to catch.
        for region in sim.world.regions.iter_mut() {
            for s in region.settlements.iter_mut() {
                s.famine_days = 5;
            }
        }
        let mut broke_out = false;
        let mut ran_course = false;
        // A couple of years of days; an outbreak should land, then end.
        for d in 1..=200u64 {
            sim.world.tick = d * 24;
            tick_plague(&mut sim);
            let j = &sim.journal.entries;
            if j.iter()
                .any(|e| e.text.contains("A sickness has broken out"))
            {
                broke_out = true;
            }
            if j.iter().any(|e| e.text.contains("has run its course")) {
                ran_course = true;
                break;
            }
        }
        assert!(
            broke_out,
            "a plague breaks out under famine within two years"
        );
        assert!(ran_course, "and the plague runs its course");
    }

    #[test]
    fn a_revival_sweeps_a_town_within_the_season() {
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        let mut swept = false;
        for d in 1..=90u64 {
            sim.world.tick = d * 24;
            tick_faith_upheavals(&mut sim);
            if sim
                .journal
                .entries
                .iter()
                .any(|e| e.text.contains("A revival sweeps"))
            {
                swept = true;
                break;
            }
        }
        assert!(swept, "a revival lands somewhere within the season");
    }

    #[test]
    fn faith_spreads_along_the_partner_roads() {
        use crate::model::economy::SettlementFaith;
        use crate::model::GodName;
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        let mut names = Vec::new();
        'outer: for region in &sim.world.regions {
            for s in &region.settlements {
                if s.population > 1 {
                    names.push(s.name.clone());
                    if names.len() == 2 {
                        break 'outer;
                    }
                }
            }
        }
        assert_eq!(names.len(), 2);
        let (a, b) = (names[0].clone(), names[1].clone());
        // Partners of two different gods.
        sim.province_ties.nudge(&a, &b, 0.8);
        let set_faith = |sim: &mut SimState, name: &str, g: GodName| {
            for region in sim.world.regions.iter_mut() {
                for s in region.settlements.iter_mut() {
                    if s.name == name {
                        s.faith = SettlementFaith::seeded(g);
                    }
                }
            }
        };
        let masa_in = |sim: &SimState, name: &str| -> f64 {
            sim.world
                .regions
                .iter()
                .flat_map(|r| r.settlements.iter())
                .find(|s| s.name == name)
                .map(|s| s.faith.get(GodName::Masa))
                .unwrap_or(0.0)
        };
        set_faith(&mut sim, &a, GodName::Keuru);
        set_faith(&mut sim, &b, GodName::Masa);
        let a_masa_before = masa_in(&sim, &a);
        for d in 1..=30u64 {
            sim.world.tick = d * 24;
            tick_faith_spread(&mut sim);
        }
        assert!(
            masa_in(&sim, &a) > a_masa_before,
            "the Masa-worshipping partner spreads its god to the Keuru town"
        );
    }

    #[test]
    fn an_alliance_feeds_a_starving_partner_year_round() {
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        let mut names = Vec::new();
        'outer: for region in &sim.world.regions {
            for s in &region.settlements {
                if s.population > 1 {
                    names.push(s.name.clone());
                    if names.len() == 2 {
                        break 'outer;
                    }
                }
            }
        }
        assert_eq!(names.len(), 2);
        let (full, starving) = (names[0].clone(), names[1].clone());
        // A deep alliance, one ally comfortable and one in famine.
        sim.province_ties.nudge(&full, &starving, 0.85);
        let set_food = |sim: &mut SimState, name: &str, ph: f64| {
            for region in sim.world.regions.iter_mut() {
                for s in region.settlements.iter_mut() {
                    if s.name == name {
                        s.food_stock = ph * s.population.max(1) as f64;
                    }
                }
            }
        };
        let get_food = |sim: &SimState, name: &str| -> f64 {
            sim.world
                .regions
                .iter()
                .flat_map(|r| r.settlements.iter())
                .find(|s| s.name == name)
                .map(|s| s.food_stock)
                .unwrap_or(0.0)
        };
        set_food(&mut sim, &full, 3.0);
        set_food(&mut sim, &starving, 0.0);
        let before = get_food(&sim, &starving);
        // High Green — the season winter relief would NOT fire in.
        sim.world.tick = 45 * 24;
        assert_eq!(
            crate::model::Season::from_day(45),
            crate::model::Season::Green
        );
        tick_alliance_relief(&mut sim);
        assert!(
            get_food(&sim, &starving) > before,
            "the alliance feeds its starving partner even in Green"
        );
        assert!(
            sim.caravans
                .iter()
                .any(|c| c.origin == full && c.destination == starving),
            "an aid caravan rides the alliance road"
        );
    }

    #[test]
    fn a_deep_rivalry_boils_over_into_a_raid() {
        let charts = charts::load_charts().unwrap();
        let mut sim = test_sim(42, charts);
        let mut names = Vec::new();
        'outer: for region in &sim.world.regions {
            for s in &region.settlements {
                if s.population > 1 {
                    names.push(s.name.clone());
                    if names.len() == 2 {
                        break 'outer;
                    }
                }
            }
        }
        assert_eq!(names.len(), 2);
        let (x, y) = (names[0].clone(), names[1].clone());
        // A hard rivalry, and a victim with stores to lose.
        sim.province_ties.nudge(&x, &y, -0.9);
        for region in sim.world.regions.iter_mut() {
            for s in region.settlements.iter_mut() {
                if s.name == x || s.name == y {
                    s.food_stock = 100.0 * s.population.max(1) as f64;
                }
            }
        }
        let total_food = |sim: &SimState| -> f64 {
            sim.world
                .regions
                .iter()
                .flat_map(|r| r.settlements.iter())
                .filter(|s| s.name == x || s.name == y)
                .map(|s| s.food_stock)
                .sum()
        };
        let before = total_food(&sim);
        // Run a season of days; a deep rivalry raids roughly one week in six,
        // so a raid should land — and a raid moves food between the two but the
        // victim's loss to fear is none (loot is carried off, not destroyed).
        let mut raided = false;
        for d in 1..=60u64 {
            sim.world.tick = d * 24;
            tick_raids(&mut sim);
            if sim
                .journal
                .entries
                .iter()
                .any(|e| e.text.contains("fell on") && e.text.contains(" in the night"))
            {
                raided = true;
                break;
            }
        }
        assert!(raided, "a deep rivalry boils over within the season");
        // Loot is carried off, not vanished: the pair's combined food is
        // conserved across a raid (within rounding).
        assert!(
            (total_food(&sim) - before).abs() < 1.0,
            "raided food is looted, not destroyed"
        );
    }

    #[test]
    fn simstate_loads_when_obligations_field_missing() {
        // A save written before `obligations` existed omits the field entirely.
        // #[serde(default)] must let it load as an empty Vec rather than error.
        let charts = charts::load_charts().unwrap();
        let sim = test_sim(42, charts);
        let s = ron::ser::to_string(&sim).unwrap();
        assert!(
            s.contains("obligations:[]"),
            "fresh state has empty obligations"
        );
        let stripped = s.replace("obligations:[],", "");
        let back: SimState =
            ron::from_str(&stripped).expect("SimState must load without an obligations field");
        assert!(back.obligations.is_empty());
    }

    /// Remove a top-level `key:(...)` group (with its trailing comma) from a
    /// compact-RON struct body by brace-matching the parentheses.
    fn drop_ron_group(s: &str, key: &str) -> String {
        let needle = format!("{key}:(");
        let start = s.find(&needle).expect("key present");
        let open = start + needle.len() - 1; // index of '('
        let mut depth = 0i32;
        let mut end = open;
        for (i, c) in s[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let mut after = end + 1;
        if s[after..].starts_with(',') {
            after += 1;
        }
        format!("{}{}", &s[..start], &s[after..])
    }

    #[test]
    fn simstate_loads_when_reputation_and_relationships_missing() {
        // Saves predating these fields omit them; #[serde(default)] must fill in
        // the Default stores rather than failing to deserialize.
        let charts = charts::load_charts().unwrap();
        let sim = test_sim(42, charts);
        let s = ron::ser::to_string(&sim).unwrap();
        let stripped = drop_ron_group(&drop_ron_group(&s, "reputation"), "relationships");
        let _back: SimState = ron::from_str(&stripped)
            .expect("SimState must load without reputation/relationships fields");
    }

    #[test]
    fn tick_needs_food_highest_decay() {
        let charts = charts::load_charts().unwrap();
        let mut world = generate_world(42, &charts);
        let food_before = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let safety_before = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Safety);
        tick_needs(&mut world, 1.0);
        let food_after = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let safety_after = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Safety);
        let food_drop = food_before - food_after;
        let safety_drop = safety_before - safety_after;
        assert!(
            food_drop > safety_drop,
            "food decay ({:.4}) should exceed safety ({:.4})",
            food_drop,
            safety_drop
        );
    }

    #[test]
    fn tick_needs_exact_values() {
        let charts = charts::load_charts().unwrap();
        let mut world = generate_world(42, &charts);
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    person.needs = crate::model::Needs::default();
                }
            }
        }
        let params = SimParams::default();
        tick_needs_with_params(&mut world, 1.0, &params);
        let p = &world.regions[0].settlements[0].people[0];
        assert!(
            (p.needs.get(Need::Food) - (0.8 - params.food_decay_rate)).abs() < f64::EPSILON,
            "food after 1 tick: expected {}, got {}",
            0.8 - params.food_decay_rate,
            p.needs.get(Need::Food)
        );
        assert!(
            (p.needs.get(Need::Safety) - (0.8 - params.safety_decay_rate)).abs() < f64::EPSILON,
            "safety after 1 tick: expected {}, got {}",
            0.8 - params.safety_decay_rate,
            p.needs.get(Need::Safety)
        );
    }

    #[test]
    fn tick_needs_clamped_at_zero() {
        let charts = charts::load_charts().unwrap();
        let mut world = generate_world(42, &charts);
        for _ in 0..200 {
            tick_needs(&mut world, 1.0);
        }
        for region in &world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    for need in &[
                        Need::Food,
                        Need::Money,
                        Need::Care,
                        Need::Presence,
                        Need::Safety,
                    ] {
                        assert!(
                            person.needs.get(*need) >= 0.0,
                            "{} went negative: {}",
                            need,
                            person.needs.get(*need)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn tick_needs_10_ticks_food_below_half() {
        let charts = charts::load_charts().unwrap();
        let mut world = generate_world(42, &charts);
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    person.needs = crate::model::Needs::default();
                }
            }
        }
        for _ in 0..10 {
            tick_needs(&mut world, 1.0);
        }
        for region in &world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    let food = person.needs.get(Need::Food);
                    let safety = person.needs.get(Need::Safety);
                    assert!(food < 0.5, "food after 10 ticks: {} (expect < 0.5)", food);
                    assert!(
                        safety > 0.5,
                        "safety after 10 ticks: {} (expect > 0.5)",
                        safety
                    );
                }
            }
        }
    }

    #[test]
    fn tick_deterministic() {
        let charts = charts::load_charts().unwrap();
        let mut a = generate_world(42, &charts);
        let mut b = generate_world(42, &charts);
        for _ in 0..5 {
            tick_needs(&mut a, 1.0);
            tick_needs(&mut b, 1.0);
        }
        let pa = &a.regions[0].settlements[0].people[0];
        let pb = &b.regions[0].settlements[0].people[0];
        assert_eq!(pa.needs, pb.needs, "tick_needs must be deterministic");
    }

    #[test]
    fn tick_increments_tick_counter() {
        let charts = charts::load_charts().unwrap();
        let mut world = generate_world(42, &charts);
        assert_eq!(world.tick, 0);
        tick(&mut world);
        assert_eq!(world.tick, 1);
        tick(&mut world);
        assert_eq!(world.tick, 2);
    }

    fn make_sim(seed: u64) -> SimState {
        let charts = charts::load_charts().unwrap();
        SimState::new_capped(seed, charts, Some(300))
    }

    #[test]
    fn sim_tick_100_deterministic() {
        let mut a = make_sim(42);
        let mut b = make_sim(42);
        for _ in 0..100 {
            a.step();
            b.step();
        }
        assert_eq!(a.world.tick, b.world.tick);
        let pa = &a.world.regions[0].settlements[0].people[0];
        let pb = &b.world.regions[0].settlements[0].people[0];
        assert_eq!(
            pa.needs, pb.needs,
            "sim needs must be deterministic after 100 ticks"
        );
        assert_eq!(a.world.regions.len(), b.world.regions.len());
    }

    #[test]
    fn sim_tick_advances_time() {
        let mut sim = make_sim(42);
        assert_eq!(sim.world.tick, 0);
        sim.step();
        assert_eq!(sim.world.tick, 1);
        for _ in 0..99 {
            sim.step();
        }
        assert_eq!(sim.world.tick, 100);
    }

    #[test]
    fn sim_tick_needs_decay_over_time() {
        let mut sim = make_sim(42);
        let food_before = sim.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        for _ in 0..10 {
            sim.step();
        }
        let food_after = sim.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            food_after < food_before,
            "food should decay over 10 sim ticks: before={}, after={}",
            food_before,
            food_after
        );
    }

    #[test]
    fn sim_tick_empty_queue_no_panic() {
        let mut sim = make_sim(42);
        for _ in 0..5 {
            sim.step();
        }
    }

    #[test]
    fn sim_tick_fire_scheduled_effect() {
        let mut sim_with = make_sim(42);
        let mut sim_without = make_sim(42);
        sim_with.effect_queue.queue(effects::Effect::deferred(
            "feast",
            3,
            vec![effects::Change::NeedDelta {
                person_id: sim_with.world.regions[0].settlements[0].people[0]
                    .id
                    .clone(),
                need: Need::Food,
                delta: 0.5,
            }],
        ));
        for _ in 0..5 {
            sim_with.step();
            sim_without.step();
        }
        let food_with = sim_with.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let food_without = sim_without.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            food_with > food_without,
            "feast effect should result in higher food: with={}, without={}",
            food_with,
            food_without
        );
    }

    #[test]
    fn sim_state_new_generates_world() {
        let sim = make_sim(42);
        assert_eq!(sim.world.seed, 42);
        assert!(!sim.world.regions.is_empty());
        assert!(sim.effect_queue.is_empty());
    }
}

#[cfg(test)]
mod determinism_tests {
    use super::*;
    use crate::charts;
    use crate::sim::effects::{Change, Effect};

    fn make_sim(seed: u64) -> SimState {
        let charts = charts::load_charts().unwrap();
        SimState::new_capped(seed, charts, Some(300))
    }

    #[test]
    fn same_seed_same_choices_100_ticks_identical() {
        let mut a = make_sim(42);
        let mut b = make_sim(42);
        let person_id = a.world.regions[0].settlements[0].people[0].id.clone();
        for tick in [10u64, 30, 50, 70] {
            let effect = Effect::immediate(
                "fed",
                vec![Change::NeedDelta {
                    person_id: person_id.clone(),
                    need: Need::Food,
                    delta: 0.1,
                }],
            );
            let mut ctx_a = effects::EffectContext {
                world: &mut a.world,
                relationships: &mut a.relationships,
                reputation: &mut a.reputation,
                current_tick: tick,
            };
            let mut ctx_b = effects::EffectContext {
                world: &mut b.world,
                relationships: &mut b.relationships,
                reputation: &mut b.reputation,
                current_tick: tick,
            };
            effects::apply_immediate(&mut ctx_a, &effect);
            effects::apply_immediate(&mut ctx_b, &effect);
            a.step();
            b.step();
        }
        for _ in 0..96 {
            a.step();
            b.step();
        }
        for (ra, rb) in a.world.regions.iter().zip(b.world.regions.iter()) {
            assert_eq!(ra.id, rb.id);
            assert_eq!(ra.settlements.len(), rb.settlements.len());
            for (sa, sb) in ra.settlements.iter().zip(rb.settlements.iter()) {
                assert_eq!(sa.people.len(), sb.people.len());
                for (pa, pb) in sa.people.iter().zip(sb.people.iter()) {
                    assert_eq!(
                        pa.needs, pb.needs,
                        "needs must be identical for person {}",
                        pa.id
                    );
                }
            }
        }
    }

    #[test]
    fn same_seed_deferred_effects_identical_order() {
        let mut a = make_sim(42);
        let mut b = make_sim(42);
        let person_id_a = a.world.regions[0].settlements[0].people[0].id.clone();
        let person_id_b = b.world.regions[0].settlements[0].people[0].id.clone();
        for (tick, delta) in [(5u64, 0.1), (5, 0.05), (10, 0.08), (15, 0.12)] {
            a.effect_queue.queue(Effect::deferred(
                "event",
                tick,
                vec![Change::NeedDelta {
                    person_id: person_id_a.clone(),
                    need: Need::Food,
                    delta,
                }],
            ));
            b.effect_queue.queue(Effect::deferred(
                "event",
                tick,
                vec![Change::NeedDelta {
                    person_id: person_id_b.clone(),
                    need: Need::Food,
                    delta,
                }],
            ));
        }
        for _ in 0..20 {
            a.step();
            b.step();
        }
        let food_a = a.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let food_b = b.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            (food_a - food_b).abs() < f64::EPSILON,
            "deferred effect order must be deterministic: a={}, b={}",
            food_a,
            food_b
        );
    }

    #[test]
    fn different_seed_different_world() {
        let mut a = make_sim(42);
        let mut b = make_sim(99);
        for _ in 0..10 {
            a.step();
            b.step();
        }
        let names_a: Vec<&str> = a.world.regions.iter().map(|r| r.name.as_str()).collect();
        let names_b: Vec<&str> = b.world.regions.iter().map(|r| r.name.as_str()).collect();
        assert_ne!(
            names_a, names_b,
            "different seeds should produce different region names"
        );
    }

    #[test]
    fn same_seed_same_choices_full_sim_identical() {
        let mut a = make_sim(77);
        let mut b = make_sim(77);
        let person_id = a.world.regions[0].settlements[0].people[0].id.clone();
        a.effect_queue.queue(Effect::deferred(
            "late feast",
            50,
            vec![Change::NeedDelta {
                person_id: person_id.clone(),
                need: Need::Food,
                delta: 0.3,
            }],
        ));
        b.effect_queue.queue(Effect::deferred(
            "late feast",
            50,
            vec![Change::NeedDelta {
                person_id: person_id.clone(),
                need: Need::Food,
                delta: 0.3,
            }],
        ));
        for _ in 0..100 {
            a.step();
            b.step();
        }
        assert_eq!(a.world.tick, b.world.tick);
        assert_eq!(a.world.regions.len(), b.world.regions.len());
        let food_a = a.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let food_b = b.world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        assert!(
            (food_a - food_b).abs() < f64::EPSILON,
            "full sim determinism: a={}, b={}",
            food_a,
            food_b
        );
    }
}
