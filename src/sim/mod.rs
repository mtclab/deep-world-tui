use crate::charts::Charts;
use crate::gen::world::generate_world;
use crate::model::{Need, World};
use crate::rng::SeedRng;

pub mod collapse_log;
pub mod effects;
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
pub mod weather;

use effects::{EffectContext, EffectQueue};
pub use journal::{Journal, JournalEntry, Voice};
pub use params::SimParams;
use relationships::RelationshipTracker;
use reputation::ReputationStore;

pub fn tick_needs_with_params(world: &mut World, dt: f64, params: &SimParams) {
    let rates: [(Need, f64); 5] = [
        (Need::Food, params.food_decay_rate),
        (Need::Money, params.money_decay_rate),
        (Need::Care, params.care_decay_rate),
        (Need::Presence, params.presence_decay_rate),
        (Need::Safety, params.safety_decay_rate),
    ];
    for region in &mut world.regions {
        for settlement in &mut region.settlements {
            for person in &mut settlement.people {
                for (need, rate) in &rates {
                    person.needs.decay(*need, rate * dt);
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
}

impl SimState {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let mut world = generate_world(seed, &charts);
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
        };
        sim.init_npc_wants();
        sim
    }

    fn init_npc_wants(&mut self) {
        let seed = self.world.seed;
        let person_info: Vec<(String, String)> = self
            .world
            .regions
            .iter()
            .flat_map(|r| r.settlements.iter())
            .flat_map(|s| s.people.iter())
            .map(|p| (p.id.clone(), p.people.clone()))
            .collect();
        for (id, people) in person_info {
            let wants = wants::generate_npc_wants(seed, &id, &people);
            self.world.set_wants_for_person(&id, wants);
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
    tick_needs_with_params(&mut sim.world, 1.0, &sim.params);
    needs_dependent::propagate_dependent_needs(&mut sim.world, &sim.obligations);
    reputation::spread_reputation(&mut sim.reputation, &sim.world, 1.0);
    sim.relationships.tick_converge(1.0);
    tick_npc_illness(sim, current_tick);
    for region in sim.world.regions.iter_mut() {
        for settlement in region.settlements.iter_mut() {
            for person in settlement.people.iter_mut() {
                crate::model::relation::decay_relations(&mut person.relations);
            }
        }
    }
    migration::tick_migration(sim, current_tick);
    let hour = ((current_tick % 24) / 4) as u32;
    sim.world.tick_npc_wants(current_tick, 24);
    sim.world.recompute_all_schedules(hour);
    tick_build_sites(sim);
    tick_structure_decay(sim);
    tick_caravans(sim);
    tick_weather_fronts(sim);
    tick_settlement_life(sim);
    lifecycle::tick_lifecycle(sim);
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
    CropType::all()
        .into_iter()
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
    let mut completed_msgs: Vec<String> = Vec::new();

    for region in sim.world.regions.iter_mut() {
        let terrain = region_work_terrain(&region.region_type);
        let weather = region.weather;
        let region_richness = region.game_richness;
        let mut richness_draw = 0.0_f64;
        let coastal = matches!(
            region.region_type.as_str(),
            "coast" | "delta" | "river_valley"
        );

        for settlement in region.settlements.iter_mut() {
            let sample = settlement.people.len().max(1) as f64;
            // The people vec is a sample of the population; scale producers up.
            let scale = (settlement.population.max(1) as f64 / sample).max(1.0);

            // --- farms: plant, grow, harvest ---
            let farmers = settlement.profession_count("farmer");
            let farm_cap = farmers.min(4);
            let frost = season == crate::model::Season::Frost;
            while settlement.farms.len() < farm_cap && !frost {
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
            // Frost kills standing crops.
            if frost {
                settlement.farms.clear();
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
            richness_draw += trap_food * 0.002 + handlers as f64 * 0.001;
            settlement.food_stock += harvested + gathered + trap_food;

            // --- consumption + spoilage ---
            let eaten = settlement.population as f64 * 0.15;
            settlement.food_stock = (settlement.food_stock - eaten).max(0.0);
            let keepers = settlement.profession_count("hearth-keeper") as f64;
            let hearth = if settlement.has_building(BuildingType::Hearth) {
                0.5
            } else {
                0.0
            };
            let spoil_rate = (0.03 - keepers * 0.005 - hearth * 0.01).max(0.0);
            settlement.food_stock *= 1.0 - spoil_rate;

            // --- the stores feed (or fail) the people ---
            let per_head = settlement.food_stock / settlement.population.max(1) as f64;
            for person in settlement.people.iter_mut() {
                if per_head >= 1.0 {
                    person.needs.satisfy(Need::Food, 0.10);
                } else if per_head < 0.4 {
                    person.needs.decay(Need::Food, 0.05);
                }
            }

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
                    let leaving = (settlement.population as f64 * 0.02).ceil() as u32;
                    settlement.population = settlement.population.saturating_sub(leaving);
                    if settlement.people.len() > 1 && settlement.famine_days % 3 == 0 {
                        settlement.people.pop();
                    }
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
                let crew = ((builders as f64) * workshop_boost).round() as u64;
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
        region.game_richness = (region.game_richness - richness_draw).max(0.0);
    }
    for msg in completed_msgs {
        let t = sim.world.tick;
        sim.log(t, Voice::Rumor, msg);
    }
}

/// Spawn trade caravans between settlements and retire them once their goods
/// have dispersed. An arrived caravan lowers prices for the goods it carried at
/// its destination (see App::caravan_price_modifier).
fn tick_caravans(sim: &mut SimState) {
    let tick = sim.world.tick;
    // Goods disperse ~2 days after arrival.
    sim.caravans.retain(|c| tick < c.arrival_tick + 48);

    if !tick.is_multiple_of(24) {
        return;
    }
    let names: Vec<String> = sim
        .world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .map(|s| s.name.clone())
        .collect();
    if names.len() < 2 {
        return;
    }
    let day = tick / 24;
    let mut rng =
        SeedRng::new(sim.world.seed.wrapping_add(day.wrapping_mul(7919))).fork_for("caravan");
    // ~1 caravan every other day on average.
    if rng.gen_range(2) == 0 {
        return;
    }
    let o = rng.gen_range(names.len() as u32) as usize;
    let mut d = rng.gen_range(names.len() as u32) as usize;
    if d == o {
        d = (d + 1) % names.len();
    }
    let caravan = crate::model::economy::Caravan::generate(
        sim.world.seed.wrapping_add(tick),
        names[o].clone(),
        names[d].clone(),
        tick,
    );
    sim.caravans.push(caravan);
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
                name: None,
                is_npc_built: false,
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

    let person_info: Vec<(usize, usize)> = sim
        .world
        .regions
        .iter()
        .enumerate()
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
    use crate::gen::world::generate_world;

    #[test]
    fn simstate_loads_when_obligations_field_missing() {
        // A save written before `obligations` existed omits the field entirely.
        // #[serde(default)] must let it load as an empty Vec rather than error.
        let charts = charts::load_charts().unwrap();
        let sim = SimState::new(42, charts);
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
        let sim = SimState::new(42, charts);
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
        SimState::new(seed, charts)
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
        SimState::new(seed, charts)
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
