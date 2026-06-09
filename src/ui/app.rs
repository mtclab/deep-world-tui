use crate::charts::Charts;
use crate::gen::companion::settlement_companions;
use crate::gen::player::generate_player_start;
use crate::model::{
    craft_recipes, Collapse, CollapseOutcome, DeathCause, Encounter, EncounterAction, EncounterLog,
    EncounterLogEntry, FestivalKind, GameClock, GodAffinity, GodName, InterPeopleBias, Inventory,
    ItemType, Need, PeopleKind, PlayerPos, PlayerStart, PlayerVitals, Settlement,
    SettlementService, TensionEvent, Terrain, Weather, WitnessLevel,
};
use crate::rng::SeedRng;
use crate::save::{self, LineageRecord, SaveData};
use crate::save_migrations;
use crate::sim::collapse_log::CollapseEvent;
use crate::sim::hints;
use crate::sim::hints::HintTracker;
use crate::sim::SimState;

use super::event::AppEvent;

#[derive(Clone)]
pub enum Screen {
    TitleScreen,
    SaveBrowser {
        scroll: u16,
        delete_confirm: Option<usize>,
    },
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
    EncounterLog {
        scroll: u16,
    },
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
    pub collapse: Option<Collapse>,
    pub death_cause: Option<DeathCause>,
    pub encounter: Option<Encounter>,
    pub god_affinity: GodAffinity,
    pub inter_people_bias: InterPeopleBias,
    pub llm_enabled: bool,
    pub llm_endpoint: String,
    pub llm_model: String,
    pub monochrome: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub language: String,
    pub audio_enabled: bool,
    pub audio_volume: f32,
    pub previous_screen: Option<Screen>,
    pub encounters_had: u32,
    pub encounter_log: EncounterLog,
    pub collapses_had: u32,
    pub collapse_log: Vec<CollapseEvent>,
    pub lineage: Vec<LineageRecord>,
    pub save_entries: Vec<crate::save::SaveEntry>,
    pub hint_tracker: HintTracker,
    pub milestones: crate::sim::milestones::MilestoneTracker,
    pub elder: bool,
    pub tick_count: u64,
    pub flash_frames: u8,
    seed: u64,
    charts: Charts,
    player_rng: Option<SeedRng>,
}

impl App {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let settings = crate::ui::AppSettings::load();
        let user_config = crate::config::load();
        let monochrome = settings.monochrome || user_config.display.monochrome;
        let high_contrast = settings.high_contrast || user_config.display.high_contrast;
        let reduced_motion = settings.reduced_motion || user_config.display.reduced_motion;
        let player_rng = SeedRng::new(seed);
        App {
            sim: None,
            player_start: None,
            running: true,
            tick_interval: 100,
            screen: Screen::TitleScreen,
            status_msg: None,
            player_pos: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            encounter: None,
            collapse: None,
            death_cause: None,
            god_affinity: GodAffinity::new(),
            inter_people_bias: InterPeopleBias::default(),
            llm_enabled: settings.llm_enabled,
            llm_endpoint: settings.llm_endpoint,
            llm_model: settings.llm_model,
            monochrome,
            high_contrast,
            reduced_motion,
            language: settings.language,
            audio_enabled: settings.audio_enabled,
            audio_volume: settings.audio_volume,
            previous_screen: None,
            encounters_had: 0,
            encounter_log: EncounterLog::new(),
            collapses_had: 0,
            collapse_log: Vec::new(),
            lineage: Vec::new(),
            save_entries: Vec::new(),
            hint_tracker: HintTracker::default(),
            milestones: crate::sim::milestones::MilestoneTracker::new(),
            elder: false,
            tick_count: 0,
            flash_frames: 0,
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
            high_contrast: self.high_contrast,
            reduced_motion: self.reduced_motion,
            language: self.language.clone(),
            audio_enabled: self.audio_enabled,
            audio_volume: self.audio_volume,
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
            let pk = PeopleKind::from_name(&ps.person.people);
            self.inter_people_bias = InterPeopleBias::new(pk);
            let sim = SimState::new(self.seed, self.charts.clone());
            self.sim = Some(sim);
            if let Some(ref mut sim) = self.sim {
                let quests = crate::sim::quest_gen::generate_initial_quests(
                    self.seed,
                    pk,
                    &sim.world.regions,
                );
                sim.quests = quests;
            }
            self.player_start = Some(ps);
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
        self.milestones.settlements_visited += 1;
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

    #[allow(deprecated)]
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
                collapse_log: self.collapse_log.clone(),
                lineage: self.lineage.clone(),
                milestones: self.milestones.clone(),
                version: save_migrations::CURRENT_SAVE_VERSION,
                first_run: false,
                hint_tracker: self.hint_tracker.clone(),
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
                let last_collapse_died = data.collapse_log.last().map(|c| c.died).unwrap_or(false);
                self.sim = Some(data.sim);
                self.player_start = data.player_start;
                self.clock = data.clock;
                self.vitals = data.vitals;
                self.player_pos = data.player_pos;
                self.god_affinity = data.god_affinity;
                self.inter_people_bias = data.inter_people_bias;
                self.encounters_had = data.encounters_had;
                self.collapses_had = data.collapses_had;
                self.collapse_log = data.collapse_log;
                self.lineage = data.lineage;
                self.hint_tracker = data.hint_tracker;
                self.milestones = data.milestones;
                self.elder = self
                    .milestones
                    .has(crate::sim::milestones::MilestoneKind::ElderAchieved);
                if last_collapse_died {
                    self.continue_as_npc();
                } else {
                    self.screen = Screen::World;
                    self.status_msg = Some("Loaded from save.ron".into());
                }
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
                self.check_quests_on_aid(&person_id);
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
                self.check_quests_on_aid(&person_id);
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
            self.fire_hint(hints::HINT_FIRST_GATHER);
            self.play_sound(crate::audio::SoundEvent::Gather);
            self.check_quests_on_gather();
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

    pub fn open_encounter_log(&mut self) {
        self.previous_screen = Some(self.screen.clone());
        self.screen = Screen::EncounterLog { scroll: 0 };
    }

    pub fn exit_encounter_log(&mut self) {
        self.screen = self.previous_screen.clone().unwrap_or(Screen::World);
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
        let inter_mod = seller_people
            .map(|sp| self.inter_people_bias.price_modifier(sp))
            .unwrap_or(1.0);
        let rep_mod = self.reputation_in_current_settlement();
        let modifier = inter_mod * rep_mod;
        let price = ((base_price as f64 * modifier).ceil() as u32).max(1);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(ItemType::Coin, price) {
                ps.inventory.add(item, 1);
                self.advance_clock_hour();
                self.fire_hint(hints::HINT_FIRST_TRADE);
                self.check_quests_on_tick();
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
        let inter_mod = buyer_people
            .map(|bp| 2.0 - self.inter_people_bias.price_modifier(bp))
            .unwrap_or(1.0);
        let rep_mod = self.reputation_in_current_settlement();
        let modifier = inter_mod * rep_mod;
        let price = ((base_price as f64 * modifier).floor() as u32).max(1);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(item, 1) {
                ps.inventory.add(ItemType::Coin, price);
                self.advance_clock_hour();
                self.fire_hint(hints::HINT_FIRST_TRADE);
                self.check_quests_on_tick();
                self.status_msg = Some(format!("Sold 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Masa, 0.01);
            } else {
                self.status_msg = Some(format!("No {} to sell", item.name()));
            }
        }
    }

    pub fn reputation_in_current_settlement(&self) -> f64 {
        let mut rep = 0.5;
        if let (Some(ref ps), Some(ref sim), Some(pos)) =
            (&self.player_start, &self.sim, self.player_pos)
        {
            if let Some(region) = sim.world.regions.get(pos.region_idx) {
                if let Some(settlement) = region.settlements.first() {
                    rep = sim.reputation.get(&ps.person.id, &settlement.id);
                }
            }
        }
        rep
    }

    pub fn quote_buy_price(&self, item: ItemType) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|sp| self.inter_people_bias.price_modifier(sp))
            .unwrap_or(1.0);
        let rep_mod = self.reputation_in_current_settlement();
        let m = inter_mod * rep_mod;
        ((base as f64 * m).ceil() as u32).max(1)
    }

    pub fn quote_sell_price(&self, item: ItemType) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|bp| 2.0 - self.inter_people_bias.price_modifier(bp))
            .unwrap_or(1.0);
        let rep_mod = self.reputation_in_current_settlement();
        let m = inter_mod * rep_mod;
        ((base as f64 * m).floor() as u32).max(1)
    }

    pub fn npc_will_engage(
        &self,
        npc_people_name: &str,
        npc_id: &str,
    ) -> crate::sim::signals::EngagementLevel {
        let bias = crate::model::PeopleKind::from_name(npc_people_name);
        let inter_bias = self.inter_people_bias.effective_bias(bias);
        let rep_drag = (inter_bias * -0.5).clamp(-0.2, 0.2);
        let effective_rep = (self.reputation_in_current_settlement() + rep_drag).clamp(0.0, 1.0);
        let _ = npc_id;
        crate::sim::signals::engagement_for(effective_rep)
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
            for companion in &mut ps.companions {
                companion.decay_needs(hours as u64);
            }
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
        self.check_milestones();
        if self.elder {
            let elder_settlement_id = self
                .sim
                .as_ref()
                .and_then(|s| {
                    let pos = self.player_pos?;
                    let region = s.world.regions.get(pos.region_idx)?;
                    region.settlements.first().map(|s| s.id.clone())
                })
                .unwrap_or_default();
            let elder_player_id = self
                .player_start
                .as_ref()
                .map(|ps| ps.person.id.clone())
                .unwrap_or_default();
            if let Some(ref mut sim) = self.sim {
                if !elder_player_id.is_empty() && !elder_settlement_id.is_empty() {
                    sim.reputation
                        .adjust_settlement(&elder_player_id, &elder_settlement_id, 0.005);
                }
            }
        }
        self.check_quests_on_tick();
        self.check_collapse();
    }

    fn log_travel(&mut self, terrain: Terrain) {
        if let Some(ref mut sim) = self.sim {
            let tod = self.clock.time_of_day();
            let weather = Weather::generate(sim.world.seed, sim.world.tick, terrain);
            let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                .fork_for(&format!("travel-journal-{}", sim.world.tick));
            let text = crate::sim::journal::travel_text(&mut rng, tod, weather);
            sim.log(sim.world.tick, crate::sim::journal::Voice::Travel, text);
        }
    }

    pub fn check_encounter(&mut self, terrain: Terrain) {
        let pp = Some(self.inter_people_bias.player_people);
        if let Some(enc) =
            Encounter::roll_biased(terrain, self.clock.hour, self.clock.day, self.seed, pp)
        {
            self.encounter = Some(enc);
            self.encounters_had += 1;
            self.fire_hint(hints::HINT_FIRST_ENCOUNTER);
            self.screen = Screen::Encounter;
            self.trigger_flash();
        }
    }

    pub fn check_discovery(&mut self, region_idx: usize, px: usize, py: usize) {
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let px_u32 = px as u32;
        let py_u32 = py as u32;
        if let Some(ref mut sim) = self.sim {
            let disc_id = match sim.discoveries.at_position(region_idx, px_u32, py_u32) {
                Some(d) => d.id.clone(),
                None => return,
            };
            if sim.discoveries.observe(&disc_id, tick, player_id) {
                let kind = sim
                    .discoveries
                    .entries
                    .iter()
                    .find(|d| d.id == disc_id)
                    .map(|d| d.kind);
                if let Some(kind) = kind {
                    sim.log_journal(tick, kind.observe_text().to_string());
                    self.status_msg = Some(format!("I found a {}!", kind.label()));
                }
            }
        }
    }

    pub fn dismiss_encounter(&mut self) {
        self.resolve_encounter(EncounterAction::Flee);
    }

    pub fn check_memorial(&mut self) {
        if let Some(pos) = self.player_pos {
            if let Some(ref sim) = self.sim {
                if let Some(memorial) = sim
                    .memorials
                    .iter()
                    .find(|m| m.at_position(pos.region_idx, pos.px as u32, pos.py as u32))
                {
                    self.status_msg = Some(memorial.text.clone());
                }
            }
        }
    }

    fn check_quests_on_tick(&mut self) {
        let current_day = self.clock.day;
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);

        let inventory = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.clone())
            .unwrap_or_default();
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let current_settlement_id = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                let region = sim.world.regions.get(pos.region_idx)?;
                region.settlements.first().map(|s| s.id.clone())
            })
            .unwrap_or_default();
        let local_rep = self
            .sim
            .as_ref()
            .and_then(|sim| {
                sim.reputation
                    .get_entry(&player_id, &current_settlement_id)
                    .map(|e| e.reputation.local)
            })
            .unwrap_or(0.5);
        let aided_npcs = self
            .sim
            .as_ref()
            .map(|s| s.aided_npcs.clone())
            .unwrap_or_default();

        if self.sim.is_none() {
            return;
        }

        let result = crate::sim::quest_gen::check_quests(
            &mut self.sim.as_mut().unwrap().quests,
            &inventory,
            region_idx,
            &aided_npcs,
            local_rep,
            current_day,
        );

        if result.completed.is_empty() && result.expired.is_empty() {
            return;
        }

        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);

        let completed_rewards: Vec<crate::model::quest::QuestReward> = result
            .completed
            .iter()
            .filter_map(|&idx| self.sim.as_ref()?.quests.get(idx).map(|q| q.reward.clone()))
            .collect();

        if let Some(ref mut sim) = self.sim {
            for _ in &result.completed {
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Travel,
                    "I did what was asked. The world shifts, just a little.".into(),
                );
            }
            for _ in &result.expired {
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Travel,
                    "What was asked of me fades. The moment has passed.".into(),
                );
            }

            let mut to_remove: Vec<usize> = result.completed;
            to_remove.extend(&result.expired);
            to_remove.sort_unstable();
            to_remove.dedup();
            to_remove.sort_unstable_by(|a, b| b.cmp(a));
            for &idx in &to_remove {
                if idx < sim.quests.len() {
                    sim.quests.remove(idx);
                }
            }
        }

        for reward in completed_rewards {
            if let Some(ref mut ps) = self.player_start {
                if let Some(ref mut sim) = self.sim {
                    crate::sim::quest_gen::apply_quest_reward(
                        &reward,
                        &mut ps.inventory,
                        &mut sim.reputation,
                        &player_id,
                        &current_settlement_id,
                        &mut sim.relationships,
                        sim.world.tick,
                    );
                }
            }
        }
    }

    fn check_quests_on_travel(&mut self, region_idx: usize) {
        let _ = region_idx;
        self.check_quests_on_tick();
    }

    fn check_quests_on_gather(&mut self) {
        self.check_quests_on_tick();
    }

    fn check_quests_on_aid(&mut self, npc_id: &str) {
        if let Some(ref mut sim) = self.sim {
            if !sim.aided_npcs.contains(&npc_id.to_string()) {
                sim.aided_npcs.push(npc_id.to_string());
            }
        }
        self.check_quests_on_tick();
    }

    pub fn adopt_companion(&mut self, region_idx: usize, settlement_idx: usize) {
        let ps = match self.player_start {
            Some(ref mut ps) => ps,
            None => {
                self.status_msg = Some("No character yet.".into());
                return;
            }
        };
        if ps.companions.len() >= 3 {
            self.status_msg = Some("I cannot take another companion. My hands are full.".into());
            return;
        }
        let (capacity, settlement_name) = if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if let Some(settlement) = region.settlements.get(settlement_idx) {
                    if !settlement.allows_companions() {
                        self.status_msg = Some(
                            "This place is too small for stable animals. No companions here."
                                .into(),
                        );
                        return;
                    }
                    (settlement.companion_capacity(), settlement.name.clone())
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        };
        let animal_rng_seed = self
            .seed
            .wrapping_add(region_idx as u64 * 997)
            .wrapping_add(settlement_idx as u64 * 31);
        let mut rng = SeedRng::new(animal_rng_seed);
        let available = settlement_companions(&mut rng, capacity);
        if available.is_empty() {
            self.status_msg = Some("No companions available.".into());
            return;
        }
        let companion = available.into_iter().next().unwrap();
        if self
            .player_start
            .as_ref()
            .is_some_and(|ps| ps.has_companion_kind(companion.animal))
        {
            self.status_msg = Some(format!(
                "I already travel with a {}. No need for another.",
                companion.animal.name()
            ));
            return;
        }
        let cost = companion.animal.cost();
        if self
            .player_start
            .as_ref()
            .map_or(0, |ps| ps.inventory.get(ItemType::Coin))
            < cost
        {
            self.status_msg = Some(format!(
                "The {} in {} costs {} coin. I lack that much.",
                companion.animal.name(),
                settlement_name,
                cost
            ));
            return;
        }
        let _ = self
            .player_start
            .as_mut()
            .unwrap()
            .inventory
            .remove(ItemType::Coin, cost);
        let name = companion.name.clone();
        let animal = companion.animal;
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        self.player_start
            .as_mut()
            .unwrap()
            .adopt_companion(animal, name.clone(), tick);
        if let Some(ref mut sim) = self.sim {
            sim.log_journal(
                tick,
                format!(
                    "A {} called {} joins me from the yards of {}. My road grows less lonely.",
                    animal.name(),
                    name,
                    settlement_name
                ),
            );
        }
        self.status_msg = Some(format!("{} the {} joins me.", name, animal.name()));
    }

    pub fn start_build(&mut self) {
        let pos = match self.player_pos {
            Some(p) => p,
            None => {
                self.status_msg = Some("No position".into());
                return;
            }
        };
        let region_idx = pos.region_idx;
        let px = pos.px as u32;
        let py = pos.py as u32;
        let inv = match self.player_start.as_ref() {
            Some(ps) => ps.inventory.clone(),
            None => {
                self.status_msg = Some("No inventory".into());
                return;
            }
        };
        let sim = match self.sim.as_mut() {
            Some(s) => s,
            None => return,
        };
        let candidates: Vec<crate::sim::structures::BuildKind> =
            crate::sim::structures::BuildKind::all()
                .iter()
                .filter(|k| {
                    k.cost()
                        .iter()
                        .all(|(item, count)| inv.get(*item) >= *count)
                        && !k.cost().is_empty()
                })
                .cloned()
                .collect();
        if candidates.is_empty() {
            self.status_msg = Some("Nothing to build — need materials".into());
            return;
        }
        let kind = candidates[0];
        if let Some(ref mut ps) = self.player_start {
            for (item, count) in kind.cost() {
                ps.inventory.remove(item, count);
            }
        }
        if kind.is_short_build() {
            let structure = crate::sim::structures::Structure {
                kind,
                region_idx,
                x: px,
                y: py,
                built_tick: sim.world.tick,
                last_maintenance_tick: sim.world.tick,
                name: None,
                is_npc_built: false,
            };
            if let Some(region) = sim.world.regions.get_mut(region_idx) {
                region.structures.push(structure.clone());
            }
            sim.structures.push(structure);
            self.fire_hint(hints::HINT_FIRST_STRUCTURE);
            self.status_msg = Some(format!("Built {}!", kind.label()));
        } else {
            let site = crate::sim::structures::BuildSite {
                kind,
                region_idx,
                x: px,
                y: py,
                hours_done: 0,
                started_tick: sim.world.tick,
            };
            sim.build_sites.push(site);
            self.status_msg = Some(format!(
                "Started building {} ({}h)",
                kind.label(),
                kind.build_hours()
            ));
        }
    }

    fn structure_at_player(&self) -> Option<crate::sim::structures::Structure> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        region
            .structures
            .iter()
            .find(|s| s.at_position(pos.region_idx, pos.px as u32, pos.py as u32))
            .cloned()
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
        let elder_talk_bonus = if self.elder { 0.10 } else { 0.0 };
        let companion_mood_bonus: f64 = self.player_start
            .as_ref()
            .and_then(|ps| ps.companions.first())
            .map(|c| c.mood().encounter_bonus())
            .unwrap_or(0.0);
        let talk_success = people_bias_mod + trust_bonus + elder_talk_bonus + companion_mood_bonus > -0.20;
        let trade_bonus = people_bias_mod + trust_bonus + companion_mood_bonus > 0.05;
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
                self.play_sound(crate::audio::SoundEvent::Combat);
                if enc_mod.intimidate + god_intimidate_bonus > 0.03 {
                    "You loomed large. They scattered before you.".into()
                } else {
                    "You stared them down. They backed off.".into()
                }
            }
            EncounterAction::Talk => {
                if !talk_success {
                    "They turned away coldly. No wisdom shared.".into()
                } else if self.elder && enc_mod.talk + elder_talk_bonus > 0.08 {
                    "An elder speaks. Even the hostile listen. Wisdom flows freely.".into()
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
                    self.play_sound(crate::audio::SoundEvent::Trade);
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
        let world_tick = self.sim.as_ref().map(|s| s.world.tick);
        let outside_intervention: Option<String> = match (encounter_data, pos_opt, &pid, world_tick)
        {
            (Some(kind), Some(pos), Some(pid), Some(tick)) if kind.can_have_outside_help() => {
                if let Some(sim) = self.sim.as_ref() {
                    if let Some(region) = sim.world.regions.get(pos.region_idx) {
                        if let Some(settlement) = region.settlements.first() {
                            let rep = sim.reputation.get(pid, &settlement.id);
                            let help_threshold = 0.70_f64;
                            let avoid_threshold = 0.25_f64;
                            if rep >= help_threshold || rep <= avoid_threshold {
                                let mut hasher = self.seed.wrapping_mul(2_654_435_761);
                                hasher ^= tick;
                                hasher ^= match kind {
                                    crate::model::EncounterKind::Wildlife => 0xA1,
                                    crate::model::EncounterKind::Bandit => 0xB2,
                                    _ => 0x00,
                                };
                                let roll = (hasher.rotate_left(13) as f64) / (u32::MAX as f64);
                                if roll < 0.02 {
                                    Some(if rep >= help_threshold {
                                        "A passing trader steps from the road, recognizing you. The bandit recoils.".to_string()
                                    } else {
                                        "The bandit glances at you, then at the road behind. He waves you on, not bothering to clean the act.".to_string()
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(ref mut sim) = self.sim {
            if let Some(_kind) = encounter_data {
                let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                    .fork_for(&format!("encounter-journal-{}", sim.world.tick));
                let voice_text = crate::sim::journal::encounter_text(&mut rng);
                let journal_text = format!("{} — {} — {}", voice_text, action.label(), msg);
                sim.log_journal(sim.world.tick, journal_text);
                if let Some(ref note) = outside_intervention {
                    sim.log_journal(sim.world.tick, format!("  * {}", note));
                }
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
        let msg = if let Some(note) = outside_intervention {
            format!("{} {}", msg, note)
        } else {
            msg
        };
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
        if let Some(kind) = encounter_data {
            self.encounter_log.push(EncounterLogEntry {
                day: self.clock.day,
                hour: self.clock.hour,
                kind,
                terrain,
                action,
                hostile: kind.is_hostile(),
            });
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

    pub fn check_milestones(&mut self) {
        use crate::sim::milestones::{core_peoples, faction_key, MilestoneKind};

        let day = self.clock.day;
        let fired = self.milestones.check_day_milestones(day);
        for kind in &fired {
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(tick, kind.voice(), kind.journal_text());
            }
        }

        if !self.elder && self.milestones.check_elder(day) {
            self.elder = true;
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    MilestoneKind::ElderAchieved.voice(),
                    MilestoneKind::ElderAchieved.journal_text(),
                );
            }
        }

        let has_player_structure = self
            .sim
            .as_ref()
            .is_some_and(|sim| sim.structures.iter().any(|s| !s.is_npc_built));
        if has_player_structure && !self.milestones.has(MilestoneKind::FirstStructureBuilt) {
            self.milestones
                .record(MilestoneKind::FirstStructureBuilt, day);
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    MilestoneKind::FirstStructureBuilt.voice(),
                    MilestoneKind::FirstStructureBuilt.journal_text(),
                );
            }
        }

        if let Some(ref ps) = self.player_start {
            if !ps.companions.is_empty()
                && !self.milestones.has(MilestoneKind::FirstCompanionAdopted)
            {
                self.milestones
                    .record(MilestoneKind::FirstCompanionAdopted, day);
                if let Some(ref mut sim) = self.sim {
                    let tick = sim.world.tick;
                    sim.log(
                        tick,
                        MilestoneKind::FirstCompanionAdopted.voice(),
                        MilestoneKind::FirstCompanionAdopted.journal_text(),
                    );
                }
            }
        }

        let people_endings_to_fire: Vec<PeopleKind> = {
            let player_id = match self.player_start {
                Some(ref ps) => ps.person.id.clone(),
                None => String::new(),
            };
            if player_id.is_empty() {
                Vec::new()
            } else if let Some(ref sim) = self.sim {
                core_peoples()
                    .iter()
                    .copied()
                    .filter(|&people| {
                        let kind = MilestoneKind::PeopleEnding { people };
                        if self.milestones.has(kind) {
                            return false;
                        }
                        let fk = faction_key(people);
                        let total: f64 = sim
                            .reputation
                            .entries
                            .values()
                            .filter(|e| e.person_id == player_id)
                            .map(|e| e.reputation.by_faction.get(fk).copied().unwrap_or(0.5))
                            .sum::<f64>();
                        let count = sim
                            .reputation
                            .entries
                            .values()
                            .filter(|e| e.person_id == player_id)
                            .count();
                        count > 0 && total / count as f64 >= 0.9
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        for people in people_endings_to_fire {
            let kind = MilestoneKind::PeopleEnding { people };
            self.milestones.record(kind, day);
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(tick, kind.voice(), kind.journal_text());
            }
        }
    }

    #[allow(deprecated)]
    pub fn check_collapse(&mut self) {
        if self.vitals.hunger > 0.0 && self.vitals.energy > 0.0 {
            return;
        }
        let vitals_before = self.vitals;
        let region_name = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                let region = sim.world.regions.get(pos.region_idx)?;
                Some(region.name.clone())
            })
            .unwrap_or_else(|| "unknown".to_string());
        let weather = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                let region = sim.world.regions.get(pos.region_idx)?;
                let terrain = region.terrain.get(0, 0)?;
                Some(
                    crate::model::Weather::generate(sim.world.seed, sim.world.tick, terrain)
                        .name()
                        .to_string(),
                )
            })
            .unwrap_or_else(|| "unknown".to_string());
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
        let rescued_by = collapse.rescued_by;
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
        let tick = self.sim.as_ref().map(|sim| sim.world.tick).unwrap_or(0);
        self.collapse_log.push(CollapseEvent {
            tick,
            vitals_before,
            region: region_name,
            weather,
            outcome,
            died,
            rescued_by,
        });
        self.collapse = Some(collapse);
        self.collapses_had += 1;
        self.fire_hint(hints::HINT_FIRST_COLLAPSE);
        if let Some(ref mut sim) = self.sim {
            let voice_text = if died {
                "I collapsed. The dark took me.".into()
            } else if let Some(god) = rescued_by {
                format!("I collapsed. {} held me back from the edge.", god.label())
            } else {
                "I collapsed. The world swam and went dark.".into()
            };
            sim.log(sim.world.tick, crate::sim::journal::Voice::Scar, voice_text);
        }
        if let Some(ref mut sim) = self.sim {
            if let Some(pos) = self.player_pos {
                let memorial = crate::model::memorial::Memorial::generate(
                    sim.world.seed,
                    sim.world.tick,
                    pos.region_idx,
                    pos.px as u32,
                    pos.py as u32,
                );
                sim.memorials.push(memorial);
                if !died {
                    let recovery_region = crate::model::memorial::pick_recovery_region(
                        sim.world.seed,
                        pos.region_idx,
                        sim.world.regions.len(),
                    );
                    let recovery_god = crate::model::memorial::pick_recovery_god(sim.world.seed);
                    self.god_affinity.adjust(recovery_god, 0.01);
                    if let Some(ref mut p) = self.player_pos {
                        p.region_idx = recovery_region;
                    }
                }
            }
        }
        if died {
            let outcome = self
                .collapse
                .map(|c| c.outcome)
                .unwrap_or(CollapseOutcome::Ditch);
            self.death_cause = Some(DeathCause::from_collapse_and_vitals(outcome, self.vitals));
            if self.player_start.is_some() {
                let save_data = SaveData {
                    sim: self
                        .sim
                        .clone()
                        .unwrap_or_else(|| SimState::new(self.seed, self.charts.clone())),
                    player_start: self.player_start.clone(),
                    clock: self.clock,
                    vitals: self.vitals,
                    player_pos: self.player_pos,
                    god_affinity: self.god_affinity,
                    inter_people_bias: self.inter_people_bias.clone(),
                    encounters_had: self.encounters_had,
                    collapses_had: self.collapses_had,
                    collapse_log: self.collapse_log.clone(),
                    lineage: self.lineage.clone(),
                    milestones: self.milestones.clone(),
                    version: save_migrations::CURRENT_SAVE_VERSION,
                    first_run: false,
                    hint_tracker: self.hint_tracker.clone(),
                };
                let _ = save::save_lineage(&save_data, self.seed);
            }
            self.continue_as_npc();
        } else {
            self.screen = Screen::Collapse;
        }
    }

    pub fn dismiss_collapse(&mut self) {
        self.collapse = None;
        self.vitals.hunger = 0.4;
        self.vitals.energy = 0.5;
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn restart_game(&mut self) {
        self.sim = None;
        self.player_start = None;
        self.collapse = None;
        self.death_cause = None;
        self.encounter = None;
        self.clock = GameClock::default();
        self.vitals = PlayerVitals::default();
        self.player_pos = None;
        self.screen = Screen::CharacterCreation;
        self.status_msg = None;
        self.running = true;
        self.lineage.clear();
        self.hint_tracker = HintTracker::default();
    }

    fn find_related_npc(&self, dead_person: &crate::model::Person) -> Option<usize> {
        let sim = self.sim.as_ref()?;
        let pos = self.player_pos?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let settlement = region.settlements.first()?;
        let dead_id = &dead_person.id;

        // 1. Find person with highest bond to dead character
        let mut best_idx: Option<usize> = None;
        let mut best_strength: f64 = -1.0;
        for (idx, person) in settlement.people.iter().enumerate() {
            if person.id == *dead_id {
                continue;
            }
            if let Some(rel) = sim.relationships.get(dead_id, &person.id) {
                if rel.strength > best_strength {
                    best_strength = rel.strength;
                    best_idx = Some(idx);
                }
            }
            if let Some(rel) = sim.relationships.get(&person.id, dead_id) {
                if rel.strength > best_strength {
                    best_strength = rel.strength;
                    best_idx = Some(idx);
                }
            }
        }
        if best_idx.is_some() {
            return best_idx;
        }

        // 2. Prefer spouse
        if dead_person.has_spouse {
            for (idx, person) in settlement.people.iter().enumerate() {
                if person.id == *dead_id {
                    continue;
                }
                if let Some(rel) = sim.relationships.get(dead_id, &person.id) {
                    if rel.kind == crate::model::RelationshipKind::Spouse {
                        return Some(idx);
                    }
                }
                if let Some(rel) = sim.relationships.get(&person.id, dead_id) {
                    if rel.kind == crate::model::RelationshipKind::Spouse {
                        return Some(idx);
                    }
                }
            }
        }

        // 3. Same people kind
        let dead_people_kind = dead_person.people.as_str();
        for (idx, person) in settlement.people.iter().enumerate() {
            if person.id == *dead_id {
                continue;
            }
            if person.people == dead_people_kind {
                return Some(idx);
            }
        }

        // 4. First adult in settlement (age_band != "child")
        for (idx, person) in settlement.people.iter().enumerate() {
            if person.id == *dead_id {
                continue;
            }
            if person.age_band != "child" {
                return Some(idx);
            }
        }

        // 5. Any person
        settlement.people.iter().position(|p| p.id != *dead_id)
    }

    fn continue_as_npc(&mut self) {
        let dead_ps = match &self.player_start {
            Some(ps) => ps.clone(),
            None => {
                self.screen = Screen::GameOver;
                return;
            }
        };
        let dead_person = dead_ps.person.clone();
        let settlement_id = dead_person.settlement.clone();

        // Find a related NPC
        let npc_idx = match self.find_related_npc(&dead_person) {
            Some(idx) => idx,
            None => {
                self.screen = Screen::GameOver;
                return;
            }
        };

        // Get the NPC person
        let npc_person = {
            let pos = match self.player_pos {
                Some(p) => p,
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            let region = match self
                .sim
                .as_ref()
                .and_then(|s| s.world.regions.get(pos.region_idx))
            {
                Some(r) => r,
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            let settlement = match region.settlements.first() {
                Some(s) => s,
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            match settlement.people.get(npc_idx) {
                Some(p) => p.clone(),
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            }
        };

        // Add lineage record
        let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        let cause = self
            .collapse
            .as_ref()
            .map(|c| format!("{:?}", c.outcome))
            .unwrap_or_else(|| "unknown".to_string());

        self.lineage.push(LineageRecord {
            predecessor_name: dead_person.name.clone(),
            predecessor_id: dead_person.id.clone(),
            cause,
            settlement_id: settlement_id.clone(),
            tick,
        });

        // Create new PlayerStart from NPC
        let new_player_start = PlayerStart {
            person: npc_person.clone(),
            reroll_count: 0,
            point_buy_adjustments: Vec::new(),
            accepted: true,
            inventory: Inventory::default(),
            companions: vec![],
        };

        // Add memorial journal entry
        let memorial = format!(
            "{} passed on. You carry their memory forward.",
            dead_person.name
        );
        if let Some(ref mut sim) = self.sim {
            sim.log_journal(sim.world.tick, memorial);
        }

        // +0.15 reputation boost
        if let Some(ref mut sim) = self.sim {
            sim.reputation
                .adjust_local(&npc_person.id, &settlement_id, 0.15);
        }

        // Switch player
        self.player_start = Some(new_player_start);
        self.vitals = PlayerVitals::default();
        self.inter_people_bias = InterPeopleBias::new(PeopleKind::from_name(&npc_person.people));

        // Continue on Map screen
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn use_service(&mut self, service: SettlementService) {
        // Time-of-day gate: Night/DeepNight — doors shut, sleep. Refuse with journal line.
        let tod = crate::model::TimeOfDay::from_hour(self.clock.hour);
        if tod.blocks_services() {
            let line = "The door is shut. I sleep.".to_string();
            self.status_msg = Some(line.clone());
            if let Some(ref mut sim) = self.sim {
                sim.log_journal(sim.world.tick, line);
            }
            return;
        }
        // Check if service provider is available at current hour
        if let Some(ref sim) = self.sim {
            if let Some(pos) = self.player_pos {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if let Some(settlement) = region.settlements.first() {
                        if let Some(person) = settlement.people.first() {
                            if !person.schedule.is_available_at_hour(self.clock.hour) {
                                self.status_msg = Some(format!(
                                    "The {} is closed. {} is {}.",
                                    service.label(),
                                    person.name,
                                    person.schedule.activity_at_hour(self.clock.hour).name()
                                ));
                                return;
                            }
                        }
                    }
                }
            }
        }
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
            let rep_mod = crate::sim::signals::reputation_price_modifier(
                self.reputation_in_current_settlement(),
            );
            let combined = price_mod * rep_mod;
            let extra = (service.cost() as f64 * combined).ceil() as u32;
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
        use crate::sim::rest::{tile_rest_quality, RestQuality};

        let tod = crate::model::TimeOfDay::from_hour(self.clock.hour);
        let was_deep_night = tod == crate::model::TimeOfDay::DeepNight;
        let on_settlement = self.player_on_settlement().is_some();
        let quality = if was_deep_night {
            RestQuality::OutInCold
        } else {
            tile_rest_quality(on_settlement, false, false, false)
        };
        let stamina_gain = quality.stamina_per_hour() * 8.0;
        let morale_gain = quality.morale_per_hour() * 8.0;
        let encounter_risk = quality.encounter_risk_per_hour() * 8.0;

        let quality_label = crate::sim::journal::rest_quality_label(
            on_settlement,
            quality == RestQuality::Inn,
            false,
            false,
        );

        self.advance_clock(8);
        self.vitals.rest();
        if let Some(ref mut ps) = self.player_start {
            for companion in &mut ps.companions {
                companion.rest(1.0);
                let action_seed = self.seed.wrapping_add((self.clock.day as u64 * 24 + self.clock.hour as u64) * 137);
                if let Some(action) = companion.autonomous_action(action_seed) {
                    let flavor = companion.apply_action(action);
                    self.status_msg = Some(format!("{} {}", companion.name, flavor));
                }
                // Companion eats from shared inventory when hungry
                if companion.food_need > 50.0 && ps.inventory.has(crate::model::ItemType::Food) {
                    ps.inventory.remove(crate::model::ItemType::Food, 1);
                    companion.feed(0.5);
                }
            }
        }
        let structure_bonus = match self.structure_at_player() {
            Some(s) => match s.kind {
                crate::sim::structures::BuildKind::Tarp => 0.05,
                crate::sim::structures::BuildKind::LeanTo => 0.10,
                crate::sim::structures::BuildKind::TarpTent => 0.15,
                crate::sim::structures::BuildKind::Laavu => 0.20,
                crate::sim::structures::BuildKind::Kota => 0.30,
                crate::sim::structures::BuildKind::Cabin => 0.45,
                crate::sim::structures::BuildKind::Longhouse => 0.60,
                crate::sim::structures::BuildKind::Home => 0.80,
            },
            None => 0.0,
        };
        if structure_bonus > 0.0 {
            self.vitals.energy = (self.vitals.energy + structure_bonus).min(1.0);
            self.vitals.hunger = (self.vitals.hunger - structure_bonus * 0.3).max(0.0);
        }
        self.vitals.energy = (self.vitals.energy + stamina_gain / 8.0).min(1.0);
        self.god_affinity
            .adjust(GodName::Kukri, 0.02 + morale_gain * 0.1);
        if quality == RestQuality::Inn {
            self.vitals.energy = (self.vitals.energy + 0.05).min(1.0);
        }

        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        if let Some(ref mut sim) = self.sim {
            sim.log_journal(tick, quality.journal_flavor().to_string());
            let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                .fork_for(&format!("rest-journal-{}", sim.world.tick));
            let text = crate::sim::journal::rest_text(&mut rng, quality_label);
            sim.log(sim.world.tick, crate::sim::journal::Voice::Rest, text);
        }

        if encounter_risk > 0.0 {
            let roll = {
                let mut rng = crate::rng::SeedRng::new(self.seed.wrapping_add(tick));
                rng.gen_f64()
            };
            if roll < encounter_risk {
                self.status_msg = Some(format!("Restless night. {}", quality.journal_flavor()));
            } else {
                self.status_msg = Some(quality.journal_flavor().to_string());
            }
        } else {
            self.status_msg = Some(quality.journal_flavor().to_string());
        }

        // God-prayer mini-encounter: a quiet dream from the patron of the
        // land we rest in. Logged to the journal so the world remembers.
        let dominant = self.sim.as_ref().and_then(|sim| {
            let pos = self.player_pos?;
            let region = sim.world.regions.get(pos.region_idx)?;
            let settlement = region.settlements.first()?;
            Some(settlement.people.first()?.people.clone())
        });
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_else(|| "player".to_string());
        if let Some(line) = crate::sim::god::maybe_prayer(&player_id, dominant.as_deref(), tick) {
            if let Some(ref mut sim) = self.sim {
                sim.log_journal(tick, line);
            }
        }

        // Dream journal entry when Kukri affinity is high
        if self.god_affinity.get(GodName::Kukri) > 0.5 {
            if let Some(ref mut sim) = self.sim {
                let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                    .fork_for(&format!("dream-journal-{}", sim.world.tick));
                let text = crate::sim::journal::dream_text(&mut rng);
                sim.log(sim.world.tick, crate::sim::journal::Voice::Dream, text);
            }
        }

        self.fire_hint(hints::HINT_FIRST_REST);
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
        let weather = self.sim.as_ref().and_then(|sim| {
            let pos = self.player_pos?;
            let region = sim.world.regions.get(pos.region_idx)?;
            let terrain = region.terrain.get(pos.px, pos.py)?;
            Some(Weather::generate(sim.world.seed, sim.world.tick, terrain))
        });
        if let Some(w) = weather {
            if let Some(ref mut rng) = self.player_rng {
                if crate::sim::weather::forced_shelter(w, rng.gen_f64()) {
                    let flavor = crate::sim::weather::weather_travel_flavor(w, true);
                    if let Some(ref mut sim) = self.sim {
                        sim.log(
                            sim.world.tick,
                            crate::sim::journal::Voice::Travel,
                            flavor.to_string(),
                        );
                    }
                    self.status_msg = Some(flavor.to_string());
                    return;
                }
            }
        }
        let weather_mult = weather
            .map(crate::sim::weather::travel_hours_multiplier)
            .unwrap_or(1.0);
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
                let hours = ((terrain.travel_hours() as f64 * weather_mult).round() as i32
                    + bias_mod)
                    .max(1) as u32;
                self.advance_clock(hours);
                self.log_travel(terrain);
                self.check_encounter(terrain);
                self.check_memorial();
                self.check_discovery(region_idx, px, py);
                self.check_quests_on_travel(region_idx);
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
                let hours = ((terrain.travel_hours() as f64 * weather_mult).round() as i32
                    + bias_mod)
                    .max(1) as u32;
                self.advance_clock(hours);
                self.log_travel(terrain);
                self.check_encounter(terrain);
                self.check_memorial();
                self.check_discovery(region_idx, px, py);
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

    fn fire_hint(&mut self, hint_id: &str) {
        if !self.hint_tracker.should_show(hint_id) {
            return;
        }
        if let Some(text) = hints::hint_text(hint_id) {
            self.hint_tracker.mark_shown(hint_id);
            if let Some(ref mut sim) = self.sim {
                use crate::sim::journal::Voice;
                let voice = match hint_id {
                    hints::HINT_FIRST_ENCOUNTER | hints::HINT_FIRST_COLLAPSE => Voice::Encounter,
                    _ => Voice::Travel,
                };
                sim.log(sim.world.tick, voice, text.to_string());
            }
        }
    }

    /// Best-effort sound playback; no-op if audio disabled.
    pub fn play_sound(&self, event: crate::audio::SoundEvent) {
        let cfg = crate::audio::AudioConfig {
            enabled: self.audio_enabled,
            volume: self.audio_volume,
        };
        crate::audio::play(event, cfg);
    }
    pub fn pre_draw(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        if self.flash_frames > 0 {
            self.flash_frames -= 1;
        }
    }

    pub fn trigger_flash(&mut self) {
        if !self.reduced_motion {
            self.flash_frames = 3;
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => match self.screen {
                Screen::TitleScreen => match key.code {
                    crossterm::event::KeyCode::Char('n') => {
                        self.screen = Screen::CharacterCreation;
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.save_entries = crate::save::saves_dir_list();
                        self.screen = Screen::SaveBrowser {
                            scroll: 0,
                            delete_confirm: None,
                        };
                    }
                    crossterm::event::KeyCode::Char('?') => {
                        self.previous_screen = Some(Screen::TitleScreen);
                        self.screen = Screen::Help;
                    }
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    _ => {}
                },
                Screen::SaveBrowser {
                    ref scroll,
                    ref delete_confirm,
                } => {
                    let scroll_val = *scroll;
                    let confirm_val = *delete_confirm;
                    match key.code {
                        crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                            let max_scroll = self.save_entries.len().saturating_sub(1) as u16;
                            self.screen = Screen::SaveBrowser {
                                scroll: scroll_val
                                    .min(max_scroll)
                                    .saturating_add(1)
                                    .min(max_scroll),
                                delete_confirm: confirm_val,
                            };
                        }
                        crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                            self.screen = Screen::SaveBrowser {
                                scroll: scroll_val.saturating_sub(1),
                                delete_confirm: confirm_val,
                            };
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let Some(entry) = self.save_entries.get(scroll_val as usize) {
                                match crate::save::load_game_file(&entry.filename) {
                                    Ok(data) => {
                                        let last_collapse_died = data
                                            .collapse_log
                                            .last()
                                            .map(|c| c.died)
                                            .unwrap_or(false);
                                        self.sim = Some(data.sim);
                                        self.player_start = data.player_start;
                                        self.clock = data.clock;
                                        self.vitals = data.vitals;
                                        self.player_pos = data.player_pos;
                                        self.god_affinity = data.god_affinity;
                                        self.inter_people_bias = data.inter_people_bias;
                                        self.encounters_had = data.encounters_had;
                                        self.collapses_had = data.collapses_had;
                                        self.collapse_log = data.collapse_log;
                                        self.lineage = data.lineage;
                                        self.milestones = data.milestones;
                                        self.hint_tracker = data.hint_tracker;
                                        self.elder = self.milestones.has(
                                            crate::sim::milestones::MilestoneKind::ElderAchieved,
                                        );
                                        if last_collapse_died {
                                            self.continue_as_npc();
                                        }
                                        self.screen = Screen::World;
                                    }
                                    Err(e) => {
                                        self.status_msg = Some(format!("Load failed: {}", e));
                                    }
                                }
                            }
                        }
                        crossterm::event::KeyCode::Char('d') => {
                            if let Some(idx) = confirm_val {
                                if idx == scroll_val as usize {
                                    if let Some(entry) = self.save_entries.get(idx) {
                                        let _ = crate::save::delete_save(&entry.filename);
                                    }
                                    self.save_entries = crate::save::saves_dir_list();
                                    let new_scroll =
                                        if scroll_val as usize >= self.save_entries.len() {
                                            self.save_entries.len().saturating_sub(1) as u16
                                        } else {
                                            scroll_val
                                        };
                                    self.screen = Screen::SaveBrowser {
                                        scroll: new_scroll,
                                        delete_confirm: None,
                                    };
                                } else {
                                    self.screen = Screen::SaveBrowser {
                                        scroll: scroll_val,
                                        delete_confirm: Some(scroll_val as usize),
                                    };
                                }
                            } else if !self.save_entries.is_empty() {
                                self.screen = Screen::SaveBrowser {
                                    scroll: scroll_val,
                                    delete_confirm: Some(scroll_val as usize),
                                };
                            }
                        }
                        crossterm::event::KeyCode::Esc => {
                            self.screen = Screen::TitleScreen;
                        }
                        _ => {}
                    }
                }
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
                        self.screen = Screen::TitleScreen;
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
                    crossterm::event::KeyCode::Char('a') => {
                        if let Some((ri, si)) = self.player_on_settlement() {
                            self.adopt_companion(ri, si);
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
                    crossterm::event::KeyCode::Char('b') => {
                        self.start_build();
                    }
                    crossterm::event::KeyCode::Char('H') => {
                        self.open_encounter_log();
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
                Screen::EncounterLog { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('H')
                    | crossterm::event::KeyCode::Left => {
                        self.exit_encounter_log();
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
                    crossterm::event::KeyCode::Char('h') => {
                        self.high_contrast = !self.high_contrast;
                        self.status_msg = Some(if self.high_contrast {
                            "High contrast mode on".into()
                        } else {
                            "Standard contrast mode on".into()
                        });
                        self.save_settings();
                    }
                    crossterm::event::KeyCode::Char('p') => {
                        self.reduced_motion = !self.reduced_motion;
                        self.status_msg = Some(if self.reduced_motion {
                            "Reduced motion on".into()
                        } else {
                            "Animations enabled".into()
                        });
                        self.save_settings();
                    }
                    crossterm::event::KeyCode::Char('a') => {
                        self.audio_enabled = !self.audio_enabled;
                        self.status_msg = Some(if self.audio_enabled {
                            "Audio enabled".into()
                        } else {
                            "Audio disabled".into()
                        });
                        self.save_settings();
                    }
                    crossterm::event::KeyCode::Char('+') | crossterm::event::KeyCode::Char('=') => {
                        self.audio_volume = (self.audio_volume + 0.1).clamp(0.0, 1.0);
                        self.status_msg =
                            Some(format!("Volume: {:.0}%", self.audio_volume * 100.0));
                        self.save_settings();
                    }
                    crossterm::event::KeyCode::Char('-') => {
                        self.audio_volume = (self.audio_volume - 0.1).clamp(0.0, 1.0);
                        self.status_msg =
                            Some(format!("Volume: {:.0}%", self.audio_volume * 100.0));
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
