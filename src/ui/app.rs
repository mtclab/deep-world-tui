use crate::charts::Charts;
use crate::gen::player::generate_player_start;
use crate::model::{
    craft_recipes, Collapse, Encounter, EncounterAction, FestivalKind, GameClock, GodAffinity,
    GodName, InterPeopleBias, Inventory, ItemType, Need, PeopleKind, PlayerPos, PlayerStart,
    PlayerVitals, Settlement, SettlementService, TensionEvent, Terrain, WitnessLevel,
};
use crate::rng::SeedRng;
use crate::save::{self, SaveData};
use crate::sim::SimState;

use super::event::AppEvent;

#[derive(Clone)]
pub enum Screen {
    CharacterCreation,
    World,
    Map {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Overmap {
        region_idx: usize,
    },
    Inventory,
    Craft {
        scroll: u16,
    },
    WorldAlerts {
        scroll: u16,
    },
    Location {
        region_idx: usize,
        settlement_idx: usize,
        scroll: u16,
    },
    Npc {
        region_idx: usize,
        settlement_idx: usize,
        person_idx: usize,
        scroll: u16,
    },
    Journal {
        scroll: u16,
    },
    Talk {
        region_idx: usize,
        settlement_idx: usize,
        person_idx: usize,
        scroll: u16,
    },
    Market {
        region_idx: usize,
        settlement_idx: usize,
        scroll: u16,
    },
    Encounter,
    Collapse,
    GameOver,
    Help,
    Settings,
}

pub struct App {
    pub sim: Option<SimState>,
    pub player_start: Option<PlayerStart>,
    pub running: bool,
    pub tick_interval: u64,
    pub screen: Screen,
    pub status_msg: Option<String>,
    pub player_pos: Option<PlayerPos>,
    pub clock: GameClock,
    pub vitals: PlayerVitals,
    pub encounter: Option<Encounter>,
    pub collapse: Option<Collapse>,
    pub god_affinity: GodAffinity,
    pub inter_people_bias: InterPeopleBias,
    pub llm_enabled: bool,
    pub llm_endpoint: String,
    pub llm_model: String,
    pub monochrome: bool,
    pub previous_screen: Option<Screen>,
    pub encounters_had: u32,
    pub collapses_had: u32,
    seed: u64,
    charts: Charts,
    player_rng: Option<SeedRng>,
}

impl App {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let settings = crate::ui::AppSettings::load();
        let player_rng = SeedRng::new(seed);
        App {
            sim: None,
            player_start: None,
            running: true,
            tick_interval: 100,
            screen: Screen::CharacterCreation,
            status_msg: None,
            player_pos: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            encounter: None,
            collapse: None,
            god_affinity: GodAffinity::new(),
            inter_people_bias: InterPeopleBias::default(),
            llm_enabled: settings.llm_enabled,
            llm_endpoint: settings.llm_endpoint,
            llm_model: settings.llm_model,
            monochrome: settings.monochrome,
            previous_screen: None,
            encounters_had: 0,
            collapses_had: 0,
            seed,
            charts,
            player_rng: Some(player_rng),
        }
    }

    pub fn save_settings(&self) {
        let settings = crate::ui::AppSettings {
            llm_enabled: self.llm_enabled,
            llm_endpoint: self.llm_endpoint.clone(),
            llm_model: self.llm_model.clone(),
            monochrome: self.monochrome,
        };
        settings.save();
    }

    pub fn generate_player(&mut self) {
        if let Some(ref mut rng) = self.player_rng {
            let ps = generate_player_start(rng, &self.charts);
            let pk = PeopleKind::from_name(&ps.person.people);
            self.inter_people_bias = InterPeopleBias::new(pk);
            self.player_start = Some(ps);
        }
    }

    pub fn reroll_player(&mut self) {
        if let Some(ref mut rng) = self.player_rng {
            if let Some(ref mut ps) = self.player_start {
                ps.reroll(rng, &self.charts);
            }
        }
    }

    pub fn accept_player(&mut self) {
        if let Some(mut ps) = self.player_start.take() {
            ps.accepted = true;
            self.player_start = Some(ps);
            let sim = SimState::new(self.seed, self.charts.clone());
            self.sim = Some(sim);
            self.screen = Screen::World;
        }
    }

    pub fn settlement_list(&self) -> Vec<(usize, usize, String)> {
        let mut out = Vec::new();
        if let Some(ref sim) = self.sim {
            for (ri, region) in sim.world.regions.iter().enumerate() {
                for (si, sett) in region.settlements.iter().enumerate() {
                    out.push((ri, si, format!("{} — {}", sett.name, region.name)));
                }
            }
        }
        out
    }

    pub fn enter_settlement(&mut self, region_idx: usize, settlement_idx: usize) {
        if let Some(npc_people) = self.current_settlement_people() {
            let bias = self.inter_people_bias.effective_bias(npc_people)
                + self.clock.season().bias_modifier();
            if bias < -0.20 {
                self.status_msg = Some(format!(
                    "Guards block your path. 'No {} allowed beyond this point.' You turn back.",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
            if bias < -0.10 {
                self.status_msg = Some(
                    "A guard eyes you suspiciously but lets you pass. 'Keep your head down.'"
                        .into(),
                );
            }
        }

        // Create obligations for NPCs with dependents
        if let Some(ref mut sim) = self.sim {
            if let Some(pos) = self.player_pos {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if let Some(settlement) = region.settlements.get(settlement_idx) {
                        for person in &settlement.people {
                            if person.has_spouse || person.children_count > 0 {
                                let obl = crate::sim::needs_dependent::Obligation {
                                    caregiver_id: person.id.clone(),
                                    dependent_id: person.id.clone(),
                                    need: crate::model::Need::Care,
                                    strength: if person.children_count > 0 {
                                        0.15
                                    } else {
                                        0.10
                                    },
                                };
                                sim.obligations.push(obl);
                            }
                        }
                    }
                }
            }
        }

        let season = self.clock.season();
        if season.festival_chance() > 0 {
            let hash = self.seed.wrapping_mul(2654435761)
                ^ (region_idx as u64).wrapping_mul(40503)
                ^ (settlement_idx as u64).wrapping_mul(92000)
                ^ (self.clock.day as u64);
            let val = hash % 100;
            if val < season.festival_chance() as u64 {
                let people = self
                    .current_settlement_people()
                    .unwrap_or(self.inter_people_bias.player_people);
                let festival = FestivalKind::for_people(people);
                self.god_affinity.adjust(festival.patron_god(), 0.03);
                let bias = self.current_settlement_people().map_or(0.0, |p| {
                    self.inter_people_bias.effective_bias(p) + season.bias_modifier()
                });
                if bias > -0.10 {
                    self.vitals.hunger = (self.vitals.hunger + 0.2).min(1.0);
                    self.vitals.energy = (self.vitals.energy + 0.1).min(1.0);
                }
                self.status_msg = Some(format!(
                    "A {} is underway! {}",
                    festival.label(),
                    festival.flavor()
                ));
            }
        }
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_settlement(&mut self) {
        self.screen = Screen::World;
    }

    pub fn current_settlement(&self) -> Option<&Settlement> {
        match &self.screen {
            Screen::Location {
                region_idx,
                settlement_idx,
                ..
            }
            | Screen::Npc {
                region_idx,
                settlement_idx,
                ..
            }
            | Screen::Talk {
                region_idx,
                settlement_idx,
                ..
            } => self.sim.as_ref().and_then(|sim| {
                sim.world
                    .regions
                    .get(*region_idx)
                    .and_then(|r| r.settlements.get(*settlement_idx))
            }),
            _ => None,
        }
    }

    pub fn enter_npc(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_npc(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn enter_journal(&mut self) {
        self.screen = Screen::Journal { scroll: 0 };
    }

    pub fn exit_journal(&mut self) {
        self.screen = Screen::World;
    }

    pub fn save_game(&mut self) {
        if let Some(ref sim) = self.sim {
            let data = SaveData {
                sim: sim.clone(),
                player_start: self.player_start.clone(),
                clock: self.clock,
                vitals: self.vitals,
                player_pos: self.player_pos,
                god_affinity: self.god_affinity,
                inter_people_bias: self.inter_people_bias.clone(),
                encounters_had: self.encounters_had,
                collapses_had: self.collapses_had,
            };
            match save::save_game(&data, "save.ron") {
                Ok(()) => self.status_msg = Some("Saved to save.ron".into()),
                Err(e) => self.status_msg = Some(format!("Save failed: {}", e)),
            }
        }
    }

    pub fn load_game(&mut self) {
        match save::load_game("save.ron") {
            Ok(data) => {
                self.sim = Some(data.sim);
                self.player_start = data.player_start;
                self.clock = data.clock;
                self.vitals = data.vitals;
                self.player_pos = data.player_pos;
                self.god_affinity = data.god_affinity;
                self.inter_people_bias = data.inter_people_bias;
                self.encounters_had = data.encounters_had;
                self.collapses_had = data.collapses_had;
                self.screen = Screen::World;
                self.status_msg = Some("Loaded from save.ron".into());
            }
            Err(e) => self.status_msg = Some(format!("Load failed: {}", e)),
        }
    }

    pub fn enter_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if region.region_type == "forest" && self.god_affinity.get(GodName::Keuru) > 0.2 {
                    self.god_affinity.adjust(GodName::Keuru, 0.01);
                }
                if region.region_type == "river_valley"
                    && self.god_affinity.get(GodName::Masa) > 0.2
                {
                    self.god_affinity.adjust(GodName::Masa, 0.01);
                }
            }
        }
        self.record_npc_memory(settlement_idx, person_idx, EncounterAction::Talk, 0.01);
        self.screen = Screen::Talk {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn give_food(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Food, 1) {
                self.status_msg = Some("No food to give".into());
                return;
            }
        }
        let npc_people = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| PeopleKind::from_name(&p.people))
        });
        if let Some(npc_pk) = npc_people {
            let mut bias = self.inter_people_bias.effective_bias(npc_pk);
            if let Some(god) = npc_pk.patron_god() {
                if self.god_affinity.get(god) > 0.4 {
                    bias += 0.05;
                }
            }
            if bias < -0.20 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Food, 1);
                }
                self.status_msg = Some(format!(
                    "'Keep your food, {}.' They push it back. 'We don't take from clearing-sympathizers.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Food, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    let mut trust_bonus = 0.05;
                    let mut rep_bonus = 0.02;
                    if self.god_affinity.get(GodName::Oltzed) > 0.3 {
                        trust_bonus += 0.02;
                        rep_bonus += 0.01;
                    }
                    if self.god_affinity.get(GodName::Masa) > 0.3 {
                        trust_bonus += 0.01;
                    }
                    let npc_people_pk = PeopleKind::from_name(&person.people);
                    sim.relationships.update_relationship_biased_full(
                        pid,
                        &person_id,
                        "gave food",
                        sim.world.tick,
                        trust_bonus,
                        0.03,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                        &person.personality,
                    );
                    sim.reputation.adjust_local_biased(
                        pid,
                        sid,
                        rep_bonus,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                    );
                }
                self.status_msg = Some(format!("Gave food to {}", person.name));
                self.god_affinity.adjust(GodName::Oltzed, 0.02);
                self.god_affinity.adjust(GodName::Masa, 0.01);
                if let Some(god) = PeopleKind::from_name(&person.people).patron_god() {
                    self.god_affinity.adjust(god, 0.01);
                }
            }
        }
    }

    pub fn give_coin(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Coin, 1) {
                self.status_msg = Some("No coin to give".into());
                return;
            }
        }
        let npc_people = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| PeopleKind::from_name(&p.people))
        });
        if let Some(npc_pk) = npc_people {
            let mut bias = self.inter_people_bias.effective_bias(npc_pk);
            if let Some(god) = npc_pk.patron_god() {
                if self.god_affinity.get(god) > 0.4 {
                    bias += 0.05;
                }
            }
            if bias < -0.20 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Coin, 1);
                }
                self.status_msg = Some(format!(
                    "The coin is set back on the table. 'We don't take {} coin here.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Money, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    let mut trust_bonus = 0.03;
                    let mut rep_bonus = 0.01;
                    if self.god_affinity.get(GodName::Oltzed) > 0.3 {
                        trust_bonus += 0.01;
                        rep_bonus += 0.01;
                    }
                    if self.god_affinity.get(GodName::Masa) > 0.3 {
                        trust_bonus += 0.01;
                    }
                    let npc_people_pk = PeopleKind::from_name(&person.people);
                    sim.relationships.update_relationship_biased_full(
                        pid,
                        &person_id,
                        "gave coin",
                        sim.world.tick,
                        trust_bonus,
                        rep_bonus,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                        &person.personality,
                    );
                    sim.reputation.adjust_local_biased(
                        pid,
                        sid,
                        0.01,
                        self.inter_people_bias.player_people,
                        npc_people_pk,
                    );
                }
                self.status_msg = Some(format!("Gave coin to {}", person.name));
                self.god_affinity.adjust(GodName::Oltzed, 0.02);
                self.god_affinity.adjust(GodName::Masa, 0.01);
                if let Some(god) = PeopleKind::from_name(&person.people).patron_god() {
                    self.god_affinity.adjust(god, 0.01);
                }
            }
        }
    }

    pub fn enter_alerts(&mut self) {
        self.screen = Screen::WorldAlerts { scroll: 0 };
    }

    pub fn exit_alerts(&mut self) {
        self.screen = Screen::World;
    }

    pub fn enter_map(&mut self, region_idx: usize) {
        let (px, py) = if let Some(ref pos) = self.player_pos {
            if pos.region_idx == region_idx {
                (pos.px, pos.py)
            } else {
                self.find_settlement_pos(region_idx)
            }
        } else {
            self.find_settlement_pos(region_idx)
        };
        self.player_pos = Some(PlayerPos { region_idx, px, py });
        self.screen = Screen::Map { region_idx, px, py };
    }

    fn find_settlement_pos(&self, region_idx: usize) -> (usize, usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if let Some(pos) = region
                    .terrain
                    .tiles
                    .iter()
                    .position(|&t| t == Terrain::Settlement)
                {
                    return (pos % region.terrain.width, pos / region.terrain.width);
                }
            }
        }
        (20, 10)
    }

    pub fn exit_map(&mut self) {
        self.screen = Screen::World;
    }

    pub fn enter_overmap(&mut self) {
        let region_idx = match &self.screen {
            Screen::Map { region_idx, .. } => *region_idx,
            _ => 0,
        };
        self.screen = Screen::Overmap { region_idx };
    }

    pub fn exit_overmap(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn gather(&mut self) {
        if self.clock.time_of_day().is_dark() {
            self.status_msg = Some("Too dark to gather".into());
            return;
        }
        let (terrain_item, terrain) = self
            .player_pos
            .and_then(|pos| {
                self.sim.as_ref().map(|sim| {
                    let region = sim.world.regions.get(pos.region_idx);
                    let t = region.and_then(|r| r.terrain.get(pos.px, pos.py));
                    let item = t.and_then(ItemType::gather_from);
                    (item, t)
                })
            })
            .unwrap_or((None, None));
        if let (Some(item), Some(terrain)) = (terrain_item, terrain) {
            match terrain {
                Terrain::Forest => {
                    self.god_affinity.adjust(GodName::Keuru, 0.03);
                    self.god_affinity.adjust(GodName::Oltzed, -0.01);
                }
                Terrain::Grass | Terrain::Farmland => {
                    self.god_affinity.adjust(GodName::Oltzed, 0.03);
                    self.god_affinity.adjust(GodName::Keuru, -0.01);
                }
                _ => {}
            }
            let season = self.clock.season();
            let mult = season.gather_multiplier();
            let pp = self.inter_people_bias.player_people;
            let people_bonus = Terrain::people_gather_bonus(pp, terrain);
            let base = 1 + people_bonus;
            let tool_bonus = if let Some(ref ps) = self.player_start {
                let best_tool = [ItemType::Iron, ItemType::Wood, ItemType::Stone]
                    .into_iter()
                    .filter(|t| ps.inventory.has(*t) && !ps.inventory.is_broken(*t))
                    .max_by_key(|t| t.base_price());
                if best_tool.is_some() {
                    1
                } else {
                    0
                }
            } else {
                0
            };
            let count = ((base + tool_bonus) as f64 * mult).floor() as u32;
            let mut boon_msg = None;
            let patron = terrain.patron_god();
            let count = if let Some(god) = patron {
                if self.god_affinity.get(god) > 0.5 && count > 0 {
                    boon_msg = Some("The land yields generously under your hands.");
                    count + 1
                } else {
                    count
                }
            } else {
                count
            };
            if count == 0 {
                self.status_msg = Some(format!(
                    "Too scarce in {} to gather {}",
                    season,
                    item.name()
                ));
                return;
            }
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, count);
                let decay_items = [
                    ItemType::Wood,
                    ItemType::Stone,
                    ItemType::Iron,
                    ItemType::Cloth,
                ];
                for di in decay_items {
                    if ps.inventory.has(di) {
                        ps.inventory.decay(di, 0.05);
                    }
                }
            }
            self.advance_clock_hour();
            let msg = format!("Gathered {} {} (1h, {})", count, item.name(), season);
            self.status_msg = Some(if let Some(b) = boon_msg {
                format!("{}. {}", msg, b)
            } else {
                msg
            });
        } else {
            self.status_msg = Some("Nothing to gather here".into());
        }
    }

    pub fn enter_inventory(&mut self) {
        self.screen = Screen::Inventory;
    }

    pub fn exit_inventory(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn enter_craft(&mut self) {
        self.screen = Screen::Craft { scroll: 0 };
    }

    pub fn exit_craft(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn craft_recipe(&mut self, recipe_idx: usize) {
        let player_people = self.inter_people_bias.player_people;
        let bias_bonus = self.current_settlement_people().map_or(0u32, |npc_people| {
            if self.inter_people_bias.effective_bias(npc_people) > 0.10 {
                1
            } else {
                0
            }
        });
        let recipes: Vec<_> = craft_recipes()
            .into_iter()
            .filter(|r| r.people.is_none() || r.people == Some(player_people))
            .collect();
        if let Some(recipe) = recipes.get(recipe_idx) {
            if let Some(ref mut ps) = self.player_start {
                let inv = &mut ps.inventory;
                let can_craft = recipe
                    .inputs
                    .iter()
                    .all(|(item, count)| inv.get(*item) >= *count);
                if can_craft {
                    for (item, count) in &recipe.inputs {
                        inv.remove(*item, *count);
                    }
                    let output_count = recipe.output_count + bias_bonus;
                    let flavor = if bias_bonus > 0 {
                        " Skilled hands guide yours."
                    } else {
                        ""
                    };
                    inv.add(recipe.output, output_count);
                    inv.decay(ItemType::Iron, 0.03);
                    inv.decay(ItemType::Wood, 0.04);
                    self.advance_clock(2);
                    self.status_msg = Some(format!(
                        "Crafted {} (x{}) (2h){}",
                        recipe.name, output_count, flavor
                    ));
                } else {
                    self.status_msg = Some("Not enough materials".into());
                }
            }
        }
    }

    pub fn enter_market(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Market {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_market(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn current_settlement_people(&self) -> Option<PeopleKind> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let settlement = region.settlements.first()?;
        let dominant = settlement.people.first()?;
        Some(PeopleKind::from_name(&dominant.people))
    }

    pub fn npc_memory(&self, person_id: &str) -> Option<&crate::model::NpcMemory> {
        self.sim
            .as_ref()
            .and_then(|sim| sim.npc_memories.get(person_id))
    }

    pub fn has_met_npc(&self, person_id: &str) -> bool {
        self.npc_memory(person_id).is_some_and(|m| m.count() > 0)
    }

    pub fn npc_trust_bonus(&self, person_id: &str) -> f64 {
        self.npc_memory(person_id)
            .map_or(0.0, |m| m.cumulative_trust().clamp(-0.3, 0.3))
    }

    pub fn record_npc_memory(
        &mut self,
        settlement_idx: usize,
        person_idx: usize,
        action: EncounterAction,
        trust_delta: f64,
    ) {
        let (person_id, settlement_name, _region_idx) = if let Some(ref sim) = self.sim {
            let pos = match self.player_pos {
                Some(p) => p,
                None => return,
            };
            let region = sim.world.regions.get(pos.region_idx);
            let settlement = region.and_then(|r| r.settlements.get(settlement_idx));
            let person = settlement.and_then(|s| s.people.get(person_idx));
            match (person, settlement) {
                (Some(p), Some(s)) => (p.id.clone(), s.name.clone(), pos.region_idx),
                _ => return,
            }
        } else {
            return;
        };
        let tick = (self.clock.day * 24 + self.clock.hour) as u64;
        if let Some(ref mut sim) = self.sim {
            sim.npc_memories.entry(person_id).or_default().add(
                action,
                tick,
                settlement_name,
                trust_delta,
            );
        }
    }

    pub fn buy_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot buy that".into());
            return;
        }
        let base_price = item.base_price();
        let seller_people = self.current_settlement_people();
        let modifier = seller_people
            .map(|sp| self.inter_people_bias.price_modifier(sp))
            .unwrap_or(1.0);
        let price = ((base_price as f64 * modifier).ceil() as u32).max(1);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(ItemType::Coin, price) {
                ps.inventory.add(item, 1);
                self.advance_clock_hour();
                self.status_msg =
                    Some(format!("Bought 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Masa, 0.02);
            } else {
                self.status_msg = Some(format!("Need {} coins", price));
            }
        }
    }

    pub fn sell_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot sell that".into());
            return;
        }
        let base_price = item.base_price();
        let buyer_people = self.current_settlement_people();
        let modifier = buyer_people
            .map(|bp| 2.0 - self.inter_people_bias.price_modifier(bp))
            .unwrap_or(1.0);
        let price = ((base_price as f64 * modifier).floor() as u32).max(1);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(item, 1) {
                ps.inventory.add(ItemType::Coin, price);
                self.advance_clock_hour();
                self.status_msg = Some(format!("Sold 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Masa, 0.01);
            } else {
                self.status_msg = Some(format!("No {} to sell", item.name()));
            }
        }
    }

    pub fn player_inventory(&self) -> Inventory {
        self.player_start
            .as_ref()
            .map(|ps| ps.inventory.clone())
            .unwrap_or_default()
    }

    pub fn advance_clock(&mut self, hours: u32) {
        let season = self.clock.season();
        self.clock.advance(hours);
        if let Some(ref mut ps) = self.player_start {
            self.vitals.tick(hours, &mut ps.inventory, season);
        }
        if let Some(ref mut sim) = self.sim {
            for _ in 0..hours {
                sim.step();
            }
        }
        if let Some((event, a, b)) = TensionEvent::roll(self.seed, self.clock.day) {
            let shift = event.bias_shift();
            if self.inter_people_bias.player_people == a {
                self.inter_people_bias.mod_toward(b, shift);
            } else if self.inter_people_bias.player_people == b {
                self.inter_people_bias.mod_toward(a, -shift);
            }
            self.status_msg = Some(event.flavor(a, b));
        }
        self.check_collapse();
    }

    pub fn check_encounter(&mut self, terrain: Terrain) {
        let pp = Some(self.inter_people_bias.player_people);
        if let Some(enc) = Encounter::roll_biased(terrain, self.clock.hour, self.seed, pp) {
            self.encounter = Some(enc);
            self.encounters_had += 1;
            self.screen = Screen::Encounter;
        }
    }

    pub fn dismiss_encounter(&mut self) {
        self.resolve_encounter(EncounterAction::Flee);
    }

    pub fn resolve_encounter(&mut self, action: EncounterAction) {
        let terrain = self.encounter.map(|e| e.terrain).unwrap_or(Terrain::Grass);
        let witness = WitnessLevel::roll(self.seed.wrapping_mul(7919), terrain);
        let rep_mult = witness.reputation_multiplier();
        let enc_mod = match terrain {
            Terrain::Settlement | Terrain::Road => self
                .sim
                .as_ref()
                .and_then(|sim| {
                    let pos = self.player_pos?;
                    let region = sim.world.regions.get(pos.region_idx)?;
                    let settlement = region.settlements.first()?;
                    let person = settlement.people.first()?;
                    Some(InterPeopleBias::encounter_modifier(&person.personality))
                })
                .unwrap_or_default(),
            _ => InterPeopleBias::encounter_modifier(&[]),
        };
        let people_bias_mod = self.current_settlement_people().map_or(0.0, |npc_people| {
            self.inter_people_bias.effective_bias(npc_people) + self.clock.season().bias_modifier()
        });
        let god_calm_bonus = if self.god_affinity.get(GodName::Keuru) > 0.4 {
            0.03
        } else {
            0.0
        };
        let god_intimidate_bonus = if self.god_affinity.get(GodName::Oltzed) > 0.4 {
            0.03
        } else {
            0.0
        };
        // Trust bonus from NPC memory (if we know this person)
        let trust_bonus = self.current_settlement_people().map_or(0.0, |_npc_people| {
            if let Some(ref sim) = self.sim {
                let pos = match self.player_pos {
                    Some(p) => p,
                    None => return 0.0,
                };
                let region = sim.world.regions.get(pos.region_idx);
                let settlement = region.and_then(|r| r.settlements.first());
                let person = settlement.and_then(|s| s.people.first());
                if let Some(p) = person {
                    sim.npc_memories
                        .get(&p.id)
                        .map_or(0.0, |m| m.cumulative_trust().clamp(-0.3, 0.3))
                } else {
                    0.0
                }
            } else {
                0.0
            }
        });
        let talk_success = people_bias_mod + trust_bonus > -0.20;
        let trade_bonus = people_bias_mod + trust_bonus > 0.05;
        let msg = match action {
            EncounterAction::Flee => {
                if enc_mod.flee > 0.05 {
                    "You fled quickly! Your instincts served you.".into()
                } else {
                    "You fled! (cost some energy)".into()
                }
            }
            EncounterAction::Bribe => {
                let base_cost: u32 = 2;
                let effective_cost =
                    ((base_cost as f64) * (1.0 + enc_mod.bribe_cost.abs())).max(1.0) as u32;
                if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.get(ItemType::Coin) >= effective_cost {
                        ps.inventory.remove(ItemType::Coin, effective_cost);
                        format!("You paid {} coins to be left alone.", effective_cost)
                    } else {
                        ps.inventory
                            .remove(ItemType::Coin, ps.inventory.get(ItemType::Coin));
                        "You gave what you had. They seemed satisfied.".into()
                    }
                } else {
                    "You gestured peacefully. They let you pass.".into()
                }
            }
            EncounterAction::Calm => {
                if enc_mod.calm + god_calm_bonus > 0.03 {
                    "Your calm presence soothed the beast. It bows its head.".into()
                } else {
                    "The beast settled. It watches you leave.".into()
                }
            }
            EncounterAction::Intimidate => {
                if enc_mod.intimidate + god_intimidate_bonus > 0.03 {
                    "You loomed large. They scattered before you.".into()
                } else {
                    "You stared them down. They backed off.".into()
                }
            }
            EncounterAction::Talk => {
                if !talk_success {
                    "They turned away coldly. No wisdom shared.".into()
                } else if enc_mod.talk > 0.03 {
                    "The traveler warmed to you quickly. Wisdom flows freely.".into()
                } else if enc_mod.talk < -0.02 {
                    "Words came slow. They barely shared a thing.".into()
                } else {
                    "The traveler shared road wisdom (1h).".into()
                }
            }
            EncounterAction::Trade => {
                if let Some(ref mut ps) = self.player_start {
                    let base_herbs = if trade_bonus { 2 } else { 1 };
                    let herbs = if enc_mod.trade > 0.02 {
                        base_herbs + 1
                    } else {
                        base_herbs
                    };
                    ps.inventory.add(ItemType::Herb, herbs);
                    if herbs >= 3 {
                        "A generous trade — three herbs for your news! (1h)".into()
                    } else if herbs == 2 {
                        "A good trade — two herbs (1h)".into()
                    } else {
                        "Traded news for herbs (1h)".into()
                    }
                } else {
                    "Traded news for herbs (1h)".into()
                }
            }
            EncounterAction::Shelter => "You waited out the storm (1h).".into(),
            EncounterAction::PushThrough => {
                if enc_mod.push_through > 0.03 {
                    "You surged ahead — nothing could slow you!".into()
                } else {
                    "You pushed through regardless!".into()
                }
            }
        };
        let encounter_data = self.encounter.map(|e| e.kind);
        let pos_opt = self.player_pos;
        let npc_people = self
            .current_settlement_people()
            .unwrap_or(PeopleKind::Metsik);
        let player_people = self.inter_people_bias.player_people;
        let pid = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        if let Some(ref mut sim) = self.sim {
            if let Some(kind) = encounter_data {
                let journal_text = format!("Encounter ({:?}): {} — {}", kind, action.label(), msg);
                sim.log_journal(sim.world.tick, journal_text);
            }
            if rep_mult > 0.0 {
                let rep_delta = match action {
                    EncounterAction::Talk => 0.02,
                    EncounterAction::Trade => 0.03,
                    EncounterAction::Calm => 0.01,
                    EncounterAction::Shelter => 0.01,
                    EncounterAction::Bribe => 0.005,
                    EncounterAction::Flee => 0.0,
                    EncounterAction::Intimidate => -0.01,
                    EncounterAction::PushThrough => -0.005,
                };
                if rep_delta != 0.0 {
                    if let (Some(ref pid), Some(pos)) = (&pid, pos_opt) {
                        if let Some(region) = sim.world.regions.get(pos.region_idx) {
                            if let Some(settlement) = region.settlements.first() {
                                let sid = settlement.id.clone();
                                sim.reputation.adjust_local_biased(
                                    pid,
                                    &sid,
                                    rep_delta * rep_mult,
                                    player_people,
                                    npc_people,
                                );
                            }
                        }
                    }
                }
            }
        }
        if let Some(ref mut ps) = self.player_start {
            let combat_decay = match action {
                EncounterAction::Flee | EncounterAction::Calm | EncounterAction::Talk => 0.0,
                EncounterAction::Intimidate | EncounterAction::PushThrough => 0.08,
                EncounterAction::Shelter => 0.02,
                EncounterAction::Bribe | EncounterAction::Trade => 0.01,
            };
            if combat_decay > 0.0 {
                for tool in [ItemType::Iron, ItemType::Wood, ItemType::Stone] {
                    if ps.inventory.has(tool) {
                        ps.inventory.decay(tool, combat_decay);
                    }
                }
            }
        }
        // Record NPC memory for this encounter
        let trust_delta = match action {
            EncounterAction::Talk => 0.02,
            EncounterAction::Trade => 0.03,
            EncounterAction::Calm => 0.01,
            EncounterAction::Shelter => 0.01,
            EncounterAction::Bribe => 0.005,
            EncounterAction::Flee => 0.0,
            EncounterAction::Intimidate => -0.02,
            EncounterAction::PushThrough => -0.01,
        };
        if let Some(pos) = self.player_pos {
            if let Some(ref sim) = self.sim {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if let Some(settlement) = region.settlements.first() {
                        if let Some(person) = settlement.people.first() {
                            let person_id = person.id.clone();
                            let settlement_name = settlement.name.clone();
                            let tick = (self.clock.day * 24 + self.clock.hour) as u64;
                            if let Some(ref mut sim) = self.sim {
                                sim.npc_memories.entry(person_id).or_default().add(
                                    action,
                                    tick,
                                    settlement_name,
                                    trust_delta,
                                );
                            }
                        }
                    }
                }
            }
        }
        self.encounter = None;
        let msg_with_witness = match witness {
            WitnessLevel::Unseen => format!("{}. {}", msg, witness.flavor()),
            WitnessLevel::Rumored => format!("{}. {}", msg, witness.flavor()),
            WitnessLevel::Seen => msg,
        };
        self.status_msg = Some(msg_with_witness);
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn check_collapse(&mut self) {
        if self.vitals.hunger > 0.0 && self.vitals.energy > 0.0 {
            return;
        }
        let local_rep = self
            .player_start
            .as_ref()
            .and_then(|ps| {
                let pid = &ps.person.id;
                self.sim.as_ref().and_then(|sim| {
                    let pos = self.player_pos?;
                    let region = sim.world.regions.get(pos.region_idx)?;
                    let settlement = region.settlements.first()?;
                    Some(sim.reputation.get(pid, &settlement.id))
                })
            })
            .unwrap_or(0.0);
        let _local_people = self.current_settlement_people();
        let eff_bias = self
            .current_settlement_people()
            .map_or(0.0, |p| self.inter_people_bias.effective_bias(p));
        let collapse = Collapse::roll_biased(self.seed, &self.god_affinity, local_rep, eff_bias);
        let outcome = collapse.outcome;
        let hours = outcome.hours_passed();
        let died = collapse.died;
        self.vitals.hunger = (self.vitals.hunger + outcome.hunger_restore()).min(1.0);
        self.vitals.energy = (self.vitals.energy + outcome.energy_restore()).min(1.0);
        if let Some(ref mut ps) = self.player_start {
            let loss = outcome.coin_loss();
            ps.inventory.remove(ItemType::Coin, loss);
            if let Some(item) = outcome.item_loss() {
                ps.inventory.remove(item, 1);
            }
        }
        if outcome.is_divine() {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(ItemType::Herb, 3);
                ps.inventory.add(ItemType::Food, 2);
            }
        }
        if outcome.is_beast_aided() {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(ItemType::Herb, 1);
            }
        }
        if outcome.is_hostile() {
            self.vitals.hunger = 0.15;
            self.vitals.energy = 0.1;
        }
        self.advance_clock(hours);
        self.collapse = Some(collapse);
        self.collapses_had += 1;
        if let Some(ref mut sim) = self.sim {
            let journal_text = if died {
                format!("COLLAPSED — {:?}. You did not wake.", outcome)
            } else if let Some(god) = collapse.rescued_by {
                format!("COLLAPSED — {:?}. {} intervened.", outcome, god.label())
            } else {
                format!("COLLAPSED — {:?}", outcome)
            };
            sim.log_journal(sim.world.tick, journal_text);
        }
        if died {
            self.screen = Screen::GameOver;
        } else {
            self.screen = Screen::Collapse;
        }
    }

    pub fn dismiss_collapse(&mut self) {
        self.collapse = None;
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn restart_game(&mut self) {
        self.sim = None;
        self.player_start = None;
        self.collapse = None;
        self.encounter = None;
        self.clock = GameClock::default();
        self.vitals = PlayerVitals::default();
        self.player_pos = None;
        self.screen = Screen::CharacterCreation;
        self.status_msg = None;
        self.running = true;
    }

    pub fn use_service(&mut self, service: SettlementService) {
        if let Some(service_people) = service.people() {
            if self.inter_people_bias.player_people != service_people {
                let bias = self
                    .inter_people_bias
                    .player_people
                    .bias_toward(service_people);
                if bias < -0.05 {
                    self.status_msg = Some(format!(
                        "The {} is for {} hands only. You are not welcome.",
                        service.label(),
                        service_people.label()
                    ));
                    return;
                }
            }
        }
        if let Some(npc_people) = self.current_settlement_people() {
            let mut bias = self.inter_people_bias.effective_bias(npc_people);
            bias += self.clock.season().bias_modifier();
            if bias < -0.15 {
                self.status_msg = Some(format!(
                    "They refuse to serve {} here. 'We don't serve your kind.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
            if bias < -0.05 {
                let cost_extra = service.cost();
                let total = cost_extra + 1;
                if let Some(ref ps) = self.player_start {
                    if ps.inventory.get(ItemType::Coin) < total {
                        self.status_msg = Some(format!(
                            "They serve you grudgingly. Price is {} coins (surcharge for outsiders). You can't afford it.",
                            total
                        ));
                        return;
                    }
                }
            }
        }
        let mut cost = service.cost();
        if let Some(npc_people) = self.current_settlement_people() {
            let bias = self.inter_people_bias.effective_bias(npc_people);
            if bias < -0.05 {
                cost += 1;
            }
        }
        let npc_personality = self.sim.as_ref().and_then(|sim| {
            let pos = self.player_pos?;
            let region = sim.world.regions.get(pos.region_idx)?;
            let settlement = region.settlements.first()?;
            settlement.people.first().map(|p| p.personality.clone())
        });
        if let Some(ref personality) = npc_personality {
            let price_mod = InterPeopleBias::trade_price_modifier(personality);
            let extra = (service.cost() as f64 * price_mod).ceil() as u32;
            cost = cost.saturating_add(extra);
        }
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Coin, cost) {
                self.status_msg = Some(format!("Need {} coins for {}", cost, service.label()));
                return;
            }
        }
        match service {
            SettlementService::Tavern => {
                self.vitals.energy = (self.vitals.energy + 0.4).min(1.0);
                self.vitals.hunger = (self.vitals.hunger + 0.2).min(1.0);
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Rested at tavern (+energy, +hunger, 2h, {} coins)",
                    cost
                ));
            }
            SettlementService::Temple => {
                self.vitals.hunger = (self.vitals.hunger + 0.5).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.3).min(1.0);
                self.advance_clock(3);
                self.status_msg = Some(format!(
                    "Blessed at temple (+hunger, +energy, 3h, {} coins)",
                    cost
                ));
            }
            SettlementService::Forge => {
                self.god_affinity.adjust(GodName::Oltzed, 0.02);
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Iron, 2);
                    let mut repaired = Vec::new();
                    for tool in [
                        ItemType::Iron,
                        ItemType::Wood,
                        ItemType::Stone,
                        ItemType::Cloth,
                    ] {
                        let cost = ps.inventory.repair_cost(tool);
                        if cost > 0 && ps.inventory.get(ItemType::Coin) >= cost {
                            ps.inventory.remove(ItemType::Coin, cost);
                            ps.inventory.repair(tool);
                            repaired.push(tool.name());
                        }
                    }
                    let repair_msg = if repaired.is_empty() {
                        String::new()
                    } else {
                        format!(" Repaired: {}.", repaired.join(", "))
                    };
                    ps.inventory.add(ItemType::Iron, 0); // no-op to ensure key exists
                    self.advance_clock(3);
                    self.status_msg = Some(format!(
                        "Worked at the forge (+2 Iron, 3h, {} coins){}",
                        cost, repair_msg
                    ));
                } else {
                    self.advance_clock(3);
                    self.status_msg = Some(format!("Worked at the forge (3h, {} coins)", cost));
                }
            }
            SettlementService::Hearth => {
                self.vitals.hunger = (self.vitals.hunger + 0.6).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.5).min(1.0);
                self.god_affinity.adjust(GodName::Oltzed, 0.03);
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Warmed by the hearth (+hunger, +energy, 2h, {} coins)",
                    cost
                ));
            }
            SettlementService::TrapWorkshop => {
                self.god_affinity.adjust(GodName::Keuru, 0.03);
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Herb, 2);
                }
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Learned trapping at the workshop (+2 Herb, 2h, {} coins)",
                    cost
                ));
            }
            SettlementService::Archive => {
                self.vitals.energy = (self.vitals.energy + 0.4).min(1.0);
                self.god_affinity.adjust(GodName::Sampsa, 0.02);
                self.advance_clock(3);
                self.status_msg = Some(format!(
                    "Studied in the archive (+energy, Sampsa +0.02, 3h, {} coins)",
                    cost
                ));
            }
            SettlementService::TradePost => {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Coin, 2);
                }
                self.god_affinity.adjust(GodName::Masa, 0.02);
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Traded at the post (+2 Coin, Masa +0.02, 2h, {} coins)",
                    cost
                ));
            }
            SettlementService::Shrine => {
                self.vitals.hunger = (self.vitals.hunger + 0.3).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.3).min(1.0);
                self.god_affinity.adjust(GodName::Kukri, 0.03);
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Prayed at the shrine (+hunger, +energy, Kukri +0.03, 2h, {} coins)",
                    cost
                ));
            }
        }
    }

    pub fn advance_clock_hour(&mut self) {
        self.advance_clock(1);
    }

    pub fn rest(&mut self) {
        self.advance_clock(8);
        self.vitals.rest();
        self.god_affinity.adjust(GodName::Kukri, 0.02);
        if self.god_affinity.get(GodName::Kukri) > 0.5 {
            self.vitals.energy = (self.vitals.energy + 0.05).min(1.0);
            self.status_msg = Some("Rested deeply. Dreams of clear water. (8h)".into());
        } else {
            self.status_msg = Some("Rested (8h)".into());
        }
    }

    pub fn clock_str(&self) -> String {
        let tod = self.clock.time_of_day();
        let season = self.clock.season();
        format!(
            "D{} {:02}:00 {} {}",
            self.clock.day,
            self.clock.hour,
            tod.glyph(),
            season.glyph()
        )
    }
}

enum MoveResult {
    EdgeTransition {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Step {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Blocked {
        msg: String,
    },
}

impl App {
    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let result = self.compute_move(dx, dy);
        match result {
            Some(MoveResult::EdgeTransition { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.region_idx = region_idx;
                    p.px = px;
                    p.py = py;
                }
                let terrain = self
                    .sim
                    .as_ref()
                    .and_then(|sim| sim.world.regions.get(region_idx))
                    .and_then(|r| r.terrain.get(px, py))
                    .unwrap_or(Terrain::Grass);
                let bias_mod = self.current_settlement_people().map_or(0, |npc_people| {
                    let bias = self.inter_people_bias.effective_bias(npc_people)
                        + self.clock.season().bias_modifier();
                    if bias < -0.15 {
                        1
                    } else if bias > 0.10 {
                        -1
                    } else {
                        0
                    }
                });
                let hours = (terrain.travel_hours() as i32 + bias_mod).max(1) as u32;
                self.advance_clock(hours);
                self.check_encounter(terrain);
                if self.encounter.is_none() {
                    self.screen = Screen::Map { region_idx, px, py };
                }
            }
            Some(MoveResult::Step { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.px = px;
                    p.py = py;
                }
                let terrain = self
                    .sim
                    .as_ref()
                    .and_then(|sim| sim.world.regions.get(region_idx))
                    .and_then(|r| r.terrain.get(px, py))
                    .unwrap_or(Terrain::Grass);
                let bias_mod = self.current_settlement_people().map_or(0, |npc_people| {
                    let bias = self.inter_people_bias.effective_bias(npc_people)
                        + self.clock.season().bias_modifier();
                    if bias < -0.15 {
                        1
                    } else if bias > 0.10 {
                        -1
                    } else {
                        0
                    }
                });
                let hours = (terrain.travel_hours() as i32 + bias_mod).max(1) as u32;
                self.advance_clock(hours);
                self.check_encounter(terrain);
                if self.encounter.is_none() {
                    self.screen = Screen::Map { region_idx, px, py };
                }
            }
            Some(MoveResult::Blocked { msg }) => {
                self.status_msg = Some(msg);
            }
            None => {}
        }
    }

    fn compute_move(&self, dx: i32, dy: i32) -> Option<MoveResult> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let map_w = region.terrain.width;
        let map_h = region.terrain.height;
        let nx = pos.px as i32 + dx;
        let ny = pos.py as i32 + dy;

        if nx < 0 {
            region
                .neighbors
                .west
                .map(|west| MoveResult::EdgeTransition {
                    region_idx: west,
                    px: map_w - 1,
                    py: pos.py,
                })
        } else if nx >= map_w as i32 {
            region
                .neighbors
                .east
                .map(|east| MoveResult::EdgeTransition {
                    region_idx: east,
                    px: 0,
                    py: pos.py,
                })
        } else if ny < 0 {
            region
                .neighbors
                .north
                .map(|north| MoveResult::EdgeTransition {
                    region_idx: north,
                    px: pos.px,
                    py: map_h - 1,
                })
        } else if ny >= map_h as i32 {
            region
                .neighbors
                .south
                .map(|south| MoveResult::EdgeTransition {
                    region_idx: south,
                    px: pos.px,
                    py: 0,
                })
        } else {
            let ux = nx as usize;
            let uy = ny as usize;
            let terrain = region.terrain.get(ux, uy);
            if let Some(t) = terrain {
                if t.passable() {
                    Some(MoveResult::Step {
                        region_idx: pos.region_idx,
                        px: ux,
                        py: uy,
                    })
                } else {
                    Some(MoveResult::Blocked {
                        msg: format!("Blocked: {:?}", t),
                    })
                }
            } else {
                None
            }
        }
    }

    pub fn player_on_settlement(&self) -> Option<(usize, usize)> {
        if let Some(ref pos) = self.player_pos {
            if let Some(ref sim) = self.sim {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if region.terrain.get(pos.px, pos.py) == Some(Terrain::Settlement) {
                        let mut idx = 0;
                        for (si, _sett) in region.settlements.iter().enumerate() {
                            let spacing = 40 / region.settlements.len().max(1);
                            let sx = (spacing / 2 + si * spacing).min(39);
                            if sx == pos.px {
                                idx = si;
                                break;
                            }
                        }
                        return Some((pos.region_idx, idx));
                    }
                }
            }
        }
        None
    }

    pub fn critical_need_people(&self) -> Vec<(String, String, String, Need, f64)> {
        let mut out = Vec::new();
        if let Some(ref sim) = self.sim {
            let needs = [
                Need::Food,
                Need::Money,
                Need::Care,
                Need::Presence,
                Need::Safety,
            ];
            for region in &sim.world.regions {
                for settlement in &region.settlements {
                    for person in &settlement.people {
                        for need in &needs {
                            let val = person.needs.get(*need);
                            if val < 0.2 {
                                out.push((
                                    person.name.clone(),
                                    person.settlement.clone(),
                                    person.profession.clone(),
                                    *need,
                                    val,
                                ));
                            }
                        }
                    }
                }
            }
        }
        out.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => match self.screen {
                Screen::CharacterCreation => match key.code {
                    crossterm::event::KeyCode::Char('r') => {
                        self.reroll_player();
                    }
                    crossterm::event::KeyCode::Enter => {
                        if self.player_start.is_none() {
                            self.generate_player();
                        }
                        self.accept_player();
                    }
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    _ => {}
                },
                Screen::World => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char('a') => {
                        if let Some(ref mut sim) = self.sim {
                            for _ in 0..10 {
                                sim.step();
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        let list = self.settlement_list();
                        if let Some((ri, si, _)) = list.get(idx) {
                            self.enter_settlement(*ri, *si);
                        }
                    }
                    crossterm::event::KeyCode::Char('j') => {
                        self.enter_journal();
                    }
                    crossterm::event::KeyCode::Char('s') => {
                        self.save_game();
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.load_game();
                    }
                    crossterm::event::KeyCode::Char('!') => {
                        self.enter_alerts();
                    }
                    crossterm::event::KeyCode::Char('m') => {
                        self.enter_map(0);
                    }
                    _ => {}
                },
                Screen::Map {
                    region_idx,
                    px: _,
                    py: _,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_map();
                    }
                    crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Left => {
                        self.move_player(-1, 0);
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.move_player(1, 0);
                    }
                    crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                        self.move_player(0, -1);
                    }
                    crossterm::event::KeyCode::Char('j') => {
                        self.move_player(0, 1);
                    }
                    crossterm::event::KeyCode::Right => {
                        self.move_player(1, 0);
                    }
                    crossterm::event::KeyCode::Down => {
                        self.move_player(0, 1);
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some((ri, si)) = self.player_on_settlement() {
                            self.enter_settlement(ri, si);
                        }
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char(c @ '1'..='9') => {
                        let idx = (c as usize) - ('1' as usize);
                        if idx != region_idx {
                            self.enter_map(
                                idx.min(
                                    self.sim
                                        .as_ref()
                                        .map(|s| s.world.regions.len())
                                        .unwrap_or(1)
                                        - 1,
                                ),
                            );
                        }
                    }
                    crossterm::event::KeyCode::Char('M') => {
                        self.enter_overmap();
                    }
                    crossterm::event::KeyCode::Char('i') => {
                        self.enter_inventory();
                    }
                    crossterm::event::KeyCode::Char('g') => {
                        self.gather();
                    }
                    crossterm::event::KeyCode::Char('r') => {
                        self.rest();
                    }
                    crossterm::event::KeyCode::Char('c') => {
                        self.enter_craft();
                    }
                    crossterm::event::KeyCode::Char('?') => {
                        self.previous_screen = Some(self.screen.clone());
                        self.screen = Screen::Help;
                    }
                    crossterm::event::KeyCode::Char(',') => {
                        self.previous_screen = Some(self.screen.clone());
                        self.screen = Screen::Settings;
                    }
                    _ => {}
                },
                Screen::Overmap { region_idx } => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('M') => {
                        self.exit_overmap();
                    }
                    crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Left => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(west) = region.neighbors.west {
                                    self.screen = Screen::Overmap { region_idx: west };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('l') | crossterm::event::KeyCode::Right => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(east) = region.neighbors.east {
                                    self.screen = Screen::Overmap { region_idx: east };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(north) = region.neighbors.north {
                                    self.screen = Screen::Overmap { region_idx: north };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(south) = region.neighbors.south {
                                    self.screen = Screen::Overmap { region_idx: south };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.screen = Screen::Map {
                            region_idx,
                            px: 20,
                            py: 10,
                        };
                    }
                    _ => {}
                },
                Screen::Inventory => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('i') => {
                        self.exit_inventory();
                    }
                    _ => {}
                },
                Screen::Craft { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_craft();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Char(c @ '1'..='9') => {
                        let idx = (c as usize) - ('1' as usize);
                        self.craft_recipe(idx);
                    }
                    _ => {}
                },
                Screen::Location {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_settlement();
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.enter_npc(region_idx, settlement_idx, 0);
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        self.enter_npc(region_idx, settlement_idx, idx);
                    }
                    crossterm::event::KeyCode::Char('m') => {
                        self.enter_market(region_idx, settlement_idx);
                    }
                    crossterm::event::KeyCode::Char('s') => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(settlement) = region.settlements.get(settlement_idx) {
                                    if let Some(&service) = settlement.services.first() {
                                        self.use_service(service);
                                    } else {
                                        self.status_msg = Some("No services here".into());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Screen::Npc {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_npc(region_idx, settlement_idx);
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char('t') => {
                        self.enter_talk(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::WorldAlerts { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_alerts();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Journal { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_journal();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Talk {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_talk(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Char('f') => {
                        self.give_food(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Char('c') => {
                        self.give_coin(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Market { ref mut scroll, .. } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_market();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='6').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        let items = ItemType::tradeable_items();
                        if let Some(&item) = items.get(idx) {
                            self.buy_item(item);
                        }
                    }
                    crossterm::event::KeyCode::Char(c) if ('a'..='f').contains(&c) => {
                        let idx = (c as usize) - ('a' as usize);
                        let items = ItemType::tradeable_items();
                        if let Some(&item) = items.get(idx) {
                            self.sell_item(item);
                        }
                    }
                    _ => {}
                },
                Screen::Encounter => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.resolve_encounter(EncounterAction::Flee);
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.resolve_encounter(EncounterAction::Flee);
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        if let Some(enc) = self.encounter {
                            for action in enc.kind.available_actions() {
                                if action.key() == c {
                                    self.resolve_encounter(action);
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Screen::Collapse => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Enter => {
                        self.dismiss_collapse();
                    }
                    _ => {}
                },
                Screen::GameOver => match key.code {
                    crossterm::event::KeyCode::Char('r') => {
                        self.restart_game();
                    }
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    _ => {}
                },
                Screen::Help => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('?') => {
                        if let Some(prev) = self.previous_screen.take() {
                            self.screen = prev;
                        }
                    }
                    _ => {}
                },
                Screen::Settings => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char(',') => {
                        if let Some(prev) = self.previous_screen.take() {
                            self.screen = prev;
                        }
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.llm_enabled = !self.llm_enabled;
                        self.status_msg = Some(if self.llm_enabled {
                            "LLM narrator enabled".into()
                        } else {
                            "LLM narrator disabled (using voice.rs templates)".into()
                        });
                        self.save_settings();
                    }
                    crossterm::event::KeyCode::Char('m') => {
                        self.monochrome = !self.monochrome;
                        self.status_msg = Some(if self.monochrome {
                            "Monochrome mode on".into()
                        } else {
                            "Full color mode on".into()
                        });
                        self.save_settings();
                    }
                    crossterm::event::KeyCode::Char('e') => {
                        let endpoints = [
                            "http://localhost:11434/v1",
                            "http://localhost:8080/v1",
                            "https://api.openai.com/v1",
                        ];
                        let idx = endpoints
                            .iter()
                            .position(|e| *e == self.llm_endpoint)
                            .unwrap_or(0);
                        self.llm_endpoint = endpoints[(idx + 1) % endpoints.len()].to_string();
                        self.status_msg = Some(format!("Endpoint: {}", self.llm_endpoint));
                        self.save_settings();
                    }
                    crossterm::event::KeyCode::Char('o') => {
                        let models = ["llama3", "mistral", "gemma2", "phi3", "qwen2"];
                        let idx = models
                            .iter()
                            .position(|m| *m == self.llm_model)
                            .unwrap_or(0);
                        self.llm_model = models[(idx + 1) % models.len()].to_string();
                        self.status_msg = Some(format!("Model: {}", self.llm_model));
                        self.save_settings();
                    }
                    _ => {}
                },
            },
            AppEvent::Tick => {}
        }
    }
}
