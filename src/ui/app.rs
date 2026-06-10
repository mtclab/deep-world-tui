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
    SaveSlots {
        scroll: u16,
    },
    RestPrompt {
        hours: u32,
    },
    CharacterCreation,
    World {
        region_idx: usize,
    },
    Overmap {
        region_idx: usize,
    },
    Inventory,
    Craft {
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
    pub explored: Vec<crate::model::ExploredMap>,
    pub elder: bool,
    /// Age in years at the start of the current life (from age_band).
    pub start_age_years: u32,
    /// Calendar day on which the current life began (for elapsed-age math).
    pub birth_day: u32,
    /// Rolled maximum age for the current life; death of old age at/after this.
    pub lifespan_years: u32,
    pub tick_count: u64,
    pub flash_frames: u8,
    pub perf_slow_frames: u32,
    pub perf_last_render_us: u64,
    /// Re-entrancy guard: a collapse advances the clock for its unconscious
    /// hours, which must not recursively re-trigger another collapse.
    in_collapse: bool,
    seed: u64,
    charts: Charts,
    player_rng: Option<SeedRng>,
}

/// Calendar days that elapse per year of the player's life. Aging is decoupled
/// from the literal hour/day calendar so a full life is reachable in normal play.
const AGING_DAYS_PER_LIFE_YEAR: u32 = 3;
/// Years before death at which the player becomes an elder.
const ELDER_BAND_YEARS: u32 = 8;
/// Longest single rest the player may take (a full night).
const MAX_REST_HOURS: u32 = 12;
/// Default rest duration the picker opens on.
const DEFAULT_REST_HOURS: u32 = 6;

/// Starting age (years) for a generated age band.
fn start_age_from_band(band: &str) -> u32 {
    match band.to_ascii_lowercase().as_str() {
        "youth" | "young" => 18,
        "elder" | "old" => 58,
        _ => 32, // adult / unknown
    }
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
            explored: Vec::new(),
            elder: false,
            start_age_years: 0,
            birth_day: 0,
            lifespan_years: 0,
            tick_count: 0,
            flash_frames: 0,
            perf_slow_frames: 0,
            perf_last_render_us: 0,
            in_collapse: false,
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
                // Initialize explored maps for each region
                self.explored = sim
                    .world
                    .regions
                    .iter()
                    .map(|r| crate::model::ExploredMap::new(r.terrain.width, r.terrain.height))
                    .collect();
            }
            let age_band = ps.person.age_band.clone();
            self.player_start = Some(ps);
            self.begin_life_aging(&age_band, 0);
            self.enter_map(0);
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

        // Roll leadership events for the settlement
        if let Some(ref mut sim) = self.sim {
            if let Some(pos) = self.player_pos {
                if let Some(region) = sim.world.regions.get_mut(pos.region_idx) {
                    if let Some(settlement) = region.settlements.get_mut(settlement_idx) {
                        let seed = self
                            .seed
                            .wrapping_add(self.clock.day as u64)
                            .wrapping_add(settlement_idx as u64);
                        if let Some(event) = settlement.politics.roll_leadership_event(seed) {
                            self.status_msg = Some(event.flavor().to_string());
                        }
                    }
                }
            }
        }

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
        let festival_now = self
            .sim
            .as_ref()
            .and_then(|sim| sim.world.regions.get(region_idx))
            .and_then(|r| r.settlements.get(settlement_idx))
            .is_some_and(|s| s.in_festival(self.clock.day));
        if festival_now {
            {
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
        self.screen = self.world_screen();
    }

    /// Today's sky over a region (the weather-front system's state).
    pub fn region_weather(&self, region_idx: usize) -> Weather {
        self.sim
            .as_ref()
            .and_then(|s| s.world.regions.get(region_idx))
            .map(|r| r.weather)
            .unwrap_or(Weather::Clear)
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
        self.screen = self.world_screen();
    }

    #[allow(deprecated)]
    fn build_save_data(&self) -> Option<SaveData> {
        let sim = self.sim.as_ref()?;
        Some(SaveData {
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
            explored: self.explored.clone(),
            version: save_migrations::CURRENT_SAVE_VERSION,
            first_run: false,
            hint_tracker: self.hint_tracker.clone(),
            start_age_years: self.start_age_years,
            birth_day: self.birth_day,
            lifespan_years: self.lifespan_years,
            encounter_log: self.encounter_log.clone(),
        })
    }

    /// Save into a numbered manual slot (1-based).
    pub fn save_to_slot(&mut self, slot: usize) {
        let Some(data) = self.build_save_data() else {
            return;
        };
        match save::save_game(&data, &save::slot_filename(slot)) {
            Ok(()) => self.status_msg = Some(format!("Saved to slot {slot}")),
            Err(e) => self.status_msg = Some(format!("Save failed: {}", e)),
        }
    }

    pub fn save_game(&mut self) {
        // Back-compat default slot (used by the legacy single-save path/tests).
        self.save_to_slot(1);
    }

    /// Begin tracking age for a new life: starting age from the age band, the
    /// current calendar day as birth, and a rolled lifespan (mischance-weighted).
    fn begin_life_aging(&mut self, age_band: &str, life_salt: u64) {
        self.start_age_years = start_age_from_band(age_band);
        self.birth_day = self.clock.day;
        let mut rng = SeedRng::new(self.seed.wrapping_add(life_salt)).fork_for("lifespan");
        // Base 58-72 years; ~1 in 6 lives is cut short by frailty/mischance.
        let mut span = 58 + rng.gen_range(15);
        if rng.gen_range(6) == 0 {
            span = span.saturating_sub(8 + rng.gen_range(18));
        }
        self.lifespan_years = span.max(self.start_age_years + 2);
        self.elder = false;
    }

    /// Restore aging fields from a loaded save; pre-aging saves (lifespan 0)
    /// get a fresh lifespan rolled from the player's band so they still age.
    pub(crate) fn apply_loaded_aging(
        &mut self,
        start_age_years: u32,
        birth_day: u32,
        lifespan_years: u32,
    ) {
        self.start_age_years = start_age_years;
        self.birth_day = birth_day;
        self.lifespan_years = lifespan_years;
        if self.lifespan_years == 0 {
            if let Some(band) = self
                .player_start
                .as_ref()
                .map(|ps| ps.person.age_band.clone())
            {
                self.begin_life_aging(&band, self.clock.day as u64);
            }
        }
    }

    /// Best travel-speed multiplier among the player's companions (a horse
    /// quickens the road); 1.0 with none.
    fn companion_travel_mult(&self) -> f64 {
        self.player_start
            .as_ref()
            .map(|ps| {
                ps.companions
                    .iter()
                    .map(|c| c.animal.travel_speed_multiplier())
                    .fold(1.0, f64::min)
            })
            .unwrap_or(1.0)
    }

    /// The player's current age in years, derived from elapsed calendar days.
    pub fn current_age_years(&self) -> u32 {
        let elapsed = self.clock.day.saturating_sub(self.birth_day);
        self.start_age_years + elapsed / AGING_DAYS_PER_LIFE_YEAR
    }

    /// Advance elderhood and old-age death based on the player's age. Called
    /// once per clock advance.
    fn check_aging(&mut self) {
        if self.player_start.is_none() || self.lifespan_years == 0 {
            return;
        }
        let age = self.current_age_years();
        if age >= self.lifespan_years {
            self.die_of_old_age();
            return;
        }
        if !self.elder && age + ELDER_BAND_YEARS >= self.lifespan_years {
            self.elder = true;
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    "My years weigh on me now. I have become an elder.".into(),
                );
            }
        }
    }

    fn die_of_old_age(&mut self) {
        self.death_cause = Some(DeathCause::OldAge);
        if let Some(data) = self.build_save_data() {
            let _ = save::save_lineage(&data, self.seed);
        }
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Scar,
                "Age took me, quiet as dusk. The world remembers, and life goes on.".into(),
            );
        }
        self.continue_as_npc();
    }

    /// Return to the World map at the player's current region.
    pub fn return_to_world(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    /// Open the manual save-slot picker.
    pub fn open_save_slots(&mut self) {
        self.save_entries = save::saves_dir_list();
        self.screen = Screen::SaveSlots { scroll: 0 };
    }

    /// Open the rest-duration picker.
    pub fn open_rest_prompt(&mut self) {
        self.screen = Screen::RestPrompt {
            hours: DEFAULT_REST_HOURS,
        };
    }

    pub const MAX_REST_HOURS: u32 = MAX_REST_HOURS;

    /// The seed driving player-facing RNG (encounters, collapses, tension).
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Restore the full App state from a save. The single restore path for
    /// every load entry point — slot 1, the save browser, anything else —
    /// so loads can't silently diverge (the browser used its own field list
    /// and dropped `explored`; neither path restored `seed`, so a loaded game
    /// rolled encounters/collapses from whatever seed this session started
    /// with instead of the world it was saved in).
    pub fn apply_save_data(&mut self, data: SaveData) {
        let last_collapse_died = data.collapse_log.last().map(|c| c.died).unwrap_or(false);
        // Re-anchor all player-facing RNG (encounters, collapses, tension
        // events) to the saved world's seed.
        self.seed = data.sim.world.seed;
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
        self.explored = data.explored;
        self.milestones = data.milestones;
        self.encounter_log = data.encounter_log;
        self.apply_loaded_aging(data.start_age_years, data.birth_day, data.lifespan_years);
        self.elder = self
            .milestones
            .has(crate::sim::milestones::MilestoneKind::ElderAchieved);
        if last_collapse_died {
            self.continue_as_npc();
        } else {
            self.screen = self.world_screen();
        }
    }

    pub fn load_game(&mut self) {
        match save::load_game(&save::slot_filename(1)) {
            Ok(data) => {
                self.apply_save_data(data);
                self.status_msg = Some("Loaded slot 1".into());
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

    pub fn reveal_around(&mut self, region_idx: usize, px: usize, py: usize) {
        let mut radius = crate::model::ExploredMap::reveal_radius_for_elder(self.elder);
        // A scouting animal (falcon, crow) rides ahead and widens what the
        // player sees. scouting_bonus existed on Animal but was never applied.
        let scout = self
            .player_start
            .as_ref()
            .map(|ps| {
                ps.companions
                    .iter()
                    .map(|c| c.animal.scouting_bonus())
                    .fold(0.0, f64::max)
            })
            .unwrap_or(0.0);
        radius += (scout * 5.0).round() as usize;
        if region_idx < self.explored.len() {
            self.explored[region_idx].reveal(px, py, radius);
        }
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
        self.reveal_around(region_idx, px, py);
        self.screen = Screen::World { region_idx };
    }

    fn find_settlement_pos(&self, region_idx: usize) -> (usize, usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                let w = region.terrain.width.max(1);
                // Prefer a settlement tile.
                if let Some(pos) = region
                    .terrain
                    .tiles
                    .iter()
                    .position(|&t| t == Terrain::Settlement)
                {
                    return (pos % w, pos / w);
                }
                // Otherwise any passable tile — never strand the player in water
                // or on a mountain (e.g. a settlement-less upland region).
                if let Some(pos) = region.terrain.tiles.iter().position(|&t| t.passable()) {
                    return (pos % w, pos / w);
                }
            }
        }
        (20, 10)
    }

    pub fn exit_map(&mut self) {
        self.screen = self.world_screen();
    }

    pub(crate) fn world_screen(&self) -> Screen {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        Screen::World { region_idx }
    }

    pub fn enter_overmap(&mut self) {
        let region_idx = match &self.screen {
            Screen::World { region_idx } => *region_idx,
            _ => 0,
        };
        self.screen = Screen::Overmap { region_idx };
    }

    pub fn exit_overmap(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
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
            // Weather affects the harvest too, not just travel — a storm or
            // whiteout thins what the land gives. gather_modifier was previously
            // applied only to NPC farm growth, never to player gathering.
            let weather = self
                .player_pos
                .map(|pos| self.region_weather(pos.region_idx))
                .unwrap_or(Weather::Clear);
            let mult = season.gather_multiplier() * weather.gather_modifier();
            let pp = self.inter_people_bias.player_people;
            let people_bonus = Terrain::people_gather_bonus(pp, terrain);
            let base = 1 + people_bonus;
            let tool_bonus = if let Some(ref ps) = self.player_start {
                // A crafted Tool beats improvised iron/wood/stone.
                if ps.inventory.has(ItemType::Tool) && !ps.inventory.is_broken(ItemType::Tool) {
                    2
                } else {
                    let best_tool = [ItemType::Iron, ItemType::Wood, ItemType::Stone]
                        .into_iter()
                        .filter(|t| ps.inventory.has(*t) && !ps.inventory.is_broken(*t))
                        .max_by_key(|t| t.base_price());
                    if best_tool.is_some() {
                        1
                    } else {
                        0
                    }
                }
            } else {
                0
            };
            if tool_bonus == 2 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.use_tool(ItemType::Tool);
                }
            }
            // A gathering animal (e.g. a hound) makes the player more productive.
            let companion_gather = self
                .player_start
                .as_ref()
                .map(|ps| {
                    ps.companions
                        .iter()
                        .map(|c| c.animal.gathering_bonus())
                        .fold(0.0, f64::max)
                })
                .unwrap_or(0.0);
            let count =
                ((base + tool_bonus) as f64 * mult * (1.0 + companion_gather)).floor() as u32;
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
        self.screen = Screen::World { region_idx };
    }

    pub fn enter_craft(&mut self) {
        self.screen = Screen::Craft { scroll: 0 };
    }

    pub fn exit_craft(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    pub fn open_encounter_log(&mut self) {
        self.previous_screen = Some(self.screen.clone());
        self.screen = Screen::EncounterLog { scroll: 0 };
    }

    pub fn exit_encounter_log(&mut self) {
        self.screen = self
            .previous_screen
            .clone()
            .unwrap_or_else(|| self.world_screen());
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
        self.screen = Screen::World { region_idx };
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

    /// Supply effect of arrived caravans on the current settlement's price for
    /// an item (more goods in town → lower price). 1.0 when no caravan applies.
    fn caravan_price_modifier(&self, item: ItemType) -> f64 {
        let Some(name) = self.current_settlement().map(|s| s.name.clone()) else {
            return 1.0;
        };
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let mut m = 1.0;
        if let Some(ref sim) = self.sim {
            for c in &sim.caravans {
                if c.destination == name && c.has_arrived(tick) {
                    m *= c.price_modifier(item, tick);
                }
            }
        }
        m
    }

    pub fn buy_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot buy that".into());
            return;
        }
        // Single source of truth with the displayed quote.
        let price = self.quote_buy_price(item);
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
        // Single source of truth with the displayed quote (spread-clamped so
        // selling never pays more than buying costs).
        let price = self.quote_sell_price(item);
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

    pub fn politics_price_modifier(&self) -> f64 {
        if let (Some(ref sim), Some(pos)) = (&self.sim, self.player_pos) {
            if let Some(region) = sim.world.regions.get(pos.region_idx) {
                if let Some(settlement) = region.settlements.first() {
                    return settlement.politics.price_modifier();
                }
            }
        }
        1.0
    }

    /// What the market charges the player. Includes ALL the modifiers the
    /// transaction applies (politics + caravan supply too — the quotes used to
    /// omit those, so the displayed price could differ from the charged one).
    pub fn quote_buy_price(&self, item: ItemType) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|sp| self.inter_people_bias.price_modifier(sp))
            .unwrap_or(1.0);
        let rep_mod =
            crate::sim::signals::reputation_price_modifier(self.reputation_in_current_settlement());
        let m = inter_mod
            * rep_mod
            * self.politics_price_modifier()
            * self.caravan_price_modifier(item)
            * self.food_scarcity_modifier(item);
        ((base as f64 * m).ceil() as u32).max(1)
    }

    /// Hunger in the settlement moves the price of what can be eaten: lean
    /// stores raise Food/Herb prices, full ones lower them.
    fn food_scarcity_modifier(&self, item: ItemType) -> f64 {
        if !matches!(item, ItemType::Food | ItemType::Herb) {
            return 1.0;
        }
        self.current_settlement()
            .map(|s| s.food_scarcity_modifier())
            .unwrap_or(1.0)
    }

    /// What the market pays the player — always below the buy price, merchants
    /// take a margin. Without the clamp, high reputation inverted the spread
    /// (buy at 0.6x base, sell at 1.4x) and buy->sell was an infinite-coin loop.
    pub fn quote_sell_price(&self, item: ItemType) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|bp| 2.0 - self.inter_people_bias.price_modifier(bp))
            .unwrap_or(1.0);
        let rep_mod = 2.0
            - crate::sim::signals::reputation_price_modifier(
                self.reputation_in_current_settlement(),
            );
        let m = inter_mod
            * rep_mod
            * self.politics_price_modifier()
            * self.caravan_price_modifier(item)
            * self.food_scarcity_modifier(item);
        let raw = ((base as f64 * m).floor() as u32).max(1);
        let buy = self.quote_buy_price(item);
        if buy > 1 {
            raw.clamp(1, buy - 1)
        } else {
            1
        }
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
        // Harsh weather wears the body down faster too (need_decay_modifier
        // was defined per-weather but never applied to the player).
        let weather_mult = self
            .player_pos
            .map(|pos| self.region_weather(pos.region_idx).need_decay_modifier())
            .unwrap_or(1.0);
        let mut departed: Vec<String> = Vec::new();
        if let Some(ref mut ps) = self.player_start {
            // A sick player wears down faster (worst active disease sets the
            // rate), and untreated illness slowly worsens.
            for d in ps.person.illnesses.iter_mut() {
                d.worsen(hours);
            }
            let illness_mult = ps
                .person
                .illnesses
                .iter()
                .map(|d| d.vitals_modifier())
                .fold(1.0_f64, f64::max);
            self.vitals.tick_with_illness(
                hours,
                &mut ps.inventory,
                season,
                illness_mult * weather_mult,
            );
            for companion in &mut ps.companions {
                companion.decay_needs(hours as u64);
            }
            // A companion neglected until its needs max out leaves (or dies).
            // Nothing enforced this, so is_alive() was dead and starved
            // companions lingered forever at full need, Unhappy and idle.
            ps.companions.retain(|c| {
                if c.is_alive() {
                    true
                } else {
                    departed.push(c.name.clone());
                    false
                }
            });
        }
        for name in departed {
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    format!(
                        "{} could bear it no longer and slipped away. I kept nothing.",
                        name
                    ),
                );
            }
            self.status_msg = Some(format!("{} left — too long neglected.", name));
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
        self.check_aging();
        self.check_player_illness();
    }

    /// Recover from any run-out illnesses, then roll for contracting a new one
    /// based on the player's terrain, hunger, shelter, and access to a healer.
    /// The player was previously immune — illness was an NPC-only system.
    fn check_player_illness(&mut self) {
        let Some(pos) = self.player_pos else {
            return;
        };
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let terrain = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(pos.region_idx))
            .and_then(|r| r.terrain.get(pos.px, pos.py))
            .unwrap_or(Terrain::Grass);
        let on_settlement = self.player_on_settlement().is_some();
        let has_healer = on_settlement
            && self
                .sim
                .as_ref()
                .and_then(|s| s.world.regions.get(pos.region_idx))
                .and_then(|r| r.settlements.first())
                .map(crate::sim::illness::settlement_has_healer)
                .unwrap_or(false);

        // A Needs proxy from the player's vitals (Food = hunger; Safety from shelter).
        let mut needs = crate::model::Needs::default();
        needs
            .values
            .insert(crate::model::Need::Food, self.vitals.hunger);
        needs.values.insert(
            crate::model::Need::Safety,
            if on_settlement { 0.8 } else { 0.2 },
        );

        let Some(ref mut ps) = self.player_start else {
            return;
        };
        ps.person.illnesses.retain(|d| !d.is_recovered(tick));
        let count = ps.person.illnesses.len();
        let contracted = crate::sim::illness::tick_illness(
            self.seed, tick, terrain, &needs, 0, has_healer, count,
        )
        .filter(|d| {
            d.disease != crate::model::Disease::ChildbirthComplication
                || crate::sim::illness::can_contract_childbirth(&ps.person.sex, &ps.person.age_band)
        });
        if let Some(disease) = contracted {
            let label = disease.disease.name();
            ps.person.illnesses.push(disease);
            if let Some(ref mut sim) = self.sim {
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    format!("A sickness takes me — {label}. My body turns against the road."),
                );
            }
            self.status_msg = Some(format!("You have fallen ill: {label}."));
        }
    }

    fn log_travel(&mut self, terrain: Terrain) {
        if let Some(ref mut sim) = self.sim {
            let tod = self.clock.time_of_day();
            let weather = self
                .player_pos
                .and_then(|pos| sim.world.regions.get(pos.region_idx))
                .map(|r| r.weather)
                .unwrap_or(Weather::Clear);
            let _ = terrain;
            let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                .fork_for(&format!("travel-journal-{}", sim.world.tick));
            let text = crate::sim::journal::travel_text(&mut rng, tod, weather);
            sim.log(sim.world.tick, crate::sim::journal::Voice::Travel, text);
        }
    }

    pub fn check_encounter(&mut self, terrain: Terrain) {
        let pp = Some(self.inter_people_bias.player_people);
        // Weather stirs or settles the land: storms raise encounter chance,
        // clear skies lower it (encounter_rate_modifier was never wired).
        let weather_mult = self
            .player_pos
            .map(|pos| {
                let base = crate::sim::weather::encounter_rate_modifier(
                    self.region_weather(pos.region_idx),
                );
                // Empty land is quiet land: wild terrain spawns scale with game.
                let wild = matches!(
                    terrain,
                    Terrain::Forest | Terrain::Swamp | Terrain::Cave | Terrain::Tundra
                );
                if wild {
                    let richness = self
                        .sim
                        .as_ref()
                        .and_then(|s| s.world.regions.get(pos.region_idx))
                        .map(|r| r.game_richness)
                        .unwrap_or(1.0);
                    base * (0.5 + 0.5 * richness)
                } else {
                    base
                }
            })
            .unwrap_or(1.0);
        if let Some(enc) = Encounter::roll_biased_weather(
            terrain,
            self.clock.hour,
            self.clock.day,
            self.seed,
            pp,
            weather_mult,
        ) {
            self.encounter = Some(enc);
            self.encounters_had += 1;
            self.fire_hint(hints::HINT_FIRST_ENCOUNTER);
            self.screen = Screen::Encounter;
            self.trigger_flash();
            if let Some(ref enc) = self.encounter {
                let sound = match enc.kind {
                    crate::model::EncounterKind::Storm => crate::audio::SoundEvent::Weather,
                    crate::model::EncounterKind::Bandit | crate::model::EncounterKind::Wildlife => {
                        crate::audio::SoundEvent::Combat
                    }
                    crate::model::EncounterKind::HarvestMarket
                    | crate::model::EncounterKind::MerchantCaravan => {
                        crate::audio::SoundEvent::Trade
                    }
                    _ => crate::audio::SoundEvent::UiClick,
                };
                self.play_sound(sound);
            }
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
        let mut found_kind = None;
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
                    found_kind = Some(kind);
                }
            }
        }
        // Finding a place leaves a mark — discoveries were pure lore before.
        if let Some(kind) = found_kind {
            match kind.observe_effect() {
                crate::model::discovery::DiscoveryEffect::God(god, delta) => {
                    self.god_affinity.adjust(god, delta);
                }
                crate::model::discovery::DiscoveryEffect::Refresh { thirst, energy } => {
                    self.vitals.thirst = (self.vitals.thirst + thirst).min(1.0);
                    self.vitals.energy = (self.vitals.energy + energy).min(1.0);
                }
                crate::model::discovery::DiscoveryEffect::Reveal => {
                    // The land makes sense from here: see twice as far once.
                    self.reveal_around(region_idx, px, py);
                    let wider = crate::model::ExploredMap::reveal_radius_for_elder(self.elder) * 2;
                    if region_idx < self.explored.len() {
                        self.explored[region_idx].reveal(px, py, wider);
                    }
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

        // Inputs for the newer quest kinds: discoveries seen, dealings had,
        // and where the player has raised structures.
        let (observed, dealings, structure_regions) = {
            let sim = self.sim.as_ref().unwrap();
            let observed = sim
                .discoveries
                .entries
                .iter()
                .filter(|d| d.observed)
                .count() as u32;
            let dealings: u32 = sim.npc_memories.values().map(|m| m.count() as u32).sum();
            let mut regions: Vec<usize> = sim
                .world
                .regions
                .iter()
                .enumerate()
                .filter(|(_, r)| r.structures.iter().any(|st| !st.is_npc_built))
                .map(|(i, _)| i)
                .collect();
            regions.dedup();
            (observed, dealings, regions)
        };

        let result = crate::sim::quest_gen::check_quests(
            &mut self.sim.as_mut().unwrap().quests,
            &crate::sim::quest_gen::QuestContext {
                inventory: &inventory,
                current_region_idx: region_idx,
                aided_npcs: &aided_npcs,
                local_reputation: local_rep,
                current_day,
                observed_discoveries: observed,
                dealings,
                player_structure_regions: &structure_regions,
            },
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

        // FetchItem quests are deliveries ("needed within the walls") — the
        // goods are handed over on completion. Collect what to consume before
        // the quests are removed below; without this you keep the items and the
        // same stack can satisfy repeated fetch quests (reward farming).
        let fetch_deliveries: Vec<(ItemType, u32)> = result
            .completed
            .iter()
            .filter_map(|&idx| match &self.sim.as_ref()?.quests.get(idx)?.kind {
                crate::model::quest::QuestKind::FetchItem { item, count } => Some((*item, *count)),
                crate::model::quest::QuestKind::DeliverTo { item, count, .. } => {
                    Some((*item, *count))
                }
                _ => None,
            })
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

        // Hand over the delivered goods.
        if let Some(ref mut ps) = self.player_start {
            for (item, count) in &fetch_deliveries {
                ps.inventory.remove(*item, *count);
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

        // Keep the quest board alive: when active quests run low, the world posts
        // new needs. Salted by world tick (not day): a day-only salt regenerated
        // the IDENTICAL quest batch after completing one within the same day,
        // letting the same fetch quest be re-completed for repeat rewards.
        let player_people = self.inter_people_bias.player_people;
        let day = self.clock.day;
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let salt = self
            .seed
            .wrapping_add(tick.wrapping_mul(1009))
            .wrapping_add(7);
        if let Some(ref mut sim) = self.sim {
            if sim.quests.len() < 2 {
                let mut fresh = crate::sim::quest_gen::generate_quests(
                    salt,
                    player_people,
                    &sim.world.regions,
                    day,
                );
                let seen = sim
                    .discoveries
                    .entries
                    .iter()
                    .filter(|d| d.observed)
                    .count() as u32;
                for q in fresh.iter_mut() {
                    if let crate::model::quest::QuestKind::VisitDiscovery { baseline } = &mut q.kind
                    {
                        if *baseline == u32::MAX {
                            *baseline = seen;
                        }
                    }
                }
                sim.quests.extend(fresh);
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

    /// Maintain a weathering structure under the player (resets its decay clock).
    /// Returns true if a maintainable structure was here (handled), false if not.
    fn maintain_structure_here(&mut self) -> bool {
        let pos = match self.player_pos {
            Some(p) => p,
            None => return false,
        };
        let (px, py) = (pos.px as u32, pos.py as u32);
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let here = |st: &crate::sim::structures::Structure| {
            st.x == px && st.y == py && !st.is_npc_built && st.kind.decay_years().is_some()
        };
        let found = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(pos.region_idx))
            .is_some_and(|r| r.structures.iter().any(here));
        if !found {
            return false;
        }
        if !self
            .player_start
            .as_ref()
            .is_some_and(|ps| ps.inventory.has(ItemType::Wood))
        {
            self.status_msg = Some("Need 1 Wood to maintain this structure.".into());
            return true;
        }
        let mut label = "structure";
        if let Some(ref mut sim) = self.sim {
            if let Some(region) = sim.world.regions.get_mut(pos.region_idx) {
                // Match the `here` detect predicate exactly — only player-built,
                // decaying structures are maintainable. Filtering on position
                // alone would refresh (and mislabel/charge for) a co-located
                // NPC-built or non-decaying structure.
                for st in region.structures.iter_mut().filter(|st| {
                    st.x == px && st.y == py && !st.is_npc_built && st.kind.decay_years().is_some()
                }) {
                    st.last_maintenance_tick = tick;
                    label = st.kind.label();
                }
            }
            for st in sim.structures.iter_mut().filter(|st| {
                st.region_idx == pos.region_idx
                    && st.x == px
                    && st.y == py
                    && !st.is_npc_built
                    && st.kind.decay_years().is_some()
            }) {
                st.last_maintenance_tick = tick;
            }
        }
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(ItemType::Wood, 1);
        }
        self.advance_clock(2);
        self.status_msg = Some(format!("Maintained {label} (1 Wood, 2h)"));
        true
    }

    pub fn start_build(&mut self) {
        let pos = match self.player_pos {
            Some(p) => p,
            None => {
                self.status_msg = Some("No position".into());
                return;
            }
        };
        // Standing on your own weathering structure? Maintain it instead of
        // trying to build a new one on the same tile.
        if self.maintain_structure_here() {
            return;
        }
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
        // Weather colors the mood of whoever you meet — rain sours a talk,
        // a clear sky warms it (npc_mood_modifier was never applied).
        let weather_mood = self
            .player_pos
            .map(|pos| self.region_weather(pos.region_idx).npc_mood_modifier())
            .unwrap_or(0.0);
        let people_bias_mod = self.current_settlement_people().map_or(0.0, |npc_people| {
            self.inter_people_bias.effective_bias(npc_people)
                + self.clock.season().bias_modifier()
                + weather_mood
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
        let companion_mood_bonus: f64 = self
            .player_start
            .as_ref()
            .and_then(|ps| ps.companions.first())
            .map(|c| c.mood().encounter_bonus())
            .unwrap_or(0.0);
        let talk_success =
            people_bias_mod + trust_bonus + elder_talk_bonus + companion_mood_bonus > -0.20;
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
        // Apply the action's data-driven costs (time, energy, hunger) and its
        // god-affinity shift. These lived on EncounterAction but were never
        // wired in — resolution hardcoded a flat 1h + 0.08 energy and ignored
        // hunger and god affinity entirely.
        let hours = action.hours();
        if hours > 0 {
            self.advance_clock(hours);
        }
        self.vitals.energy = (self.vitals.energy - action.energy_cost()).max(0.0);
        self.vitals.hunger = (self.vitals.hunger - action.hunger_cost()).max(0.0);
        if let Some((god, delta)) = action.god_affinity_effect() {
            self.god_affinity.adjust(god, delta);
        }
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
        self.screen = Screen::World { region_idx };
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

        // Elderhood is age-based now (see check_aging), not a fixed calendar day.

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
        // A collapse advances the clock for its unconscious hours; that nested
        // advance must not recursively re-trigger another collapse (which would
        // recurse without bound when vitals stay at zero).
        if self.in_collapse {
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
                Some(region.weather.name().to_string())
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
        self.in_collapse = true;
        self.advance_clock(hours);
        self.in_collapse = false;
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
                    explored: self.explored.clone(),
                    version: save_migrations::CURRENT_SAVE_VERSION,
                    first_run: false,
                    hint_tracker: self.hint_tracker.clone(),
                    start_age_years: self.start_age_years,
                    birth_day: self.birth_day,
                    lifespan_years: self.lifespan_years,
                    encounter_log: self.encounter_log.clone(),
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
        // A floor on waking, not an assignment. check_collapse already applied
        // the outcome-specific restores (a settlement bed / divine rescue
        // recovers far more than a ditch); hardcoding 0.4/0.5 here threw all of
        // that away, so every non-fatal collapse ended identically.
        self.vitals.hunger = self.vitals.hunger.max(0.4);
        self.vitals.energy = self.vitals.energy.max(0.5);
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
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

    pub(crate) fn continue_as_npc(&mut self) {
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

        // Get the NPC person (and the settlement the heir actually lives in —
        // reputation used to be keyed by the dead's stored settlement string,
        // which can differ from any real settlement id).
        let (npc_person, heir_settlement_id) = {
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
            let person = match settlement.people.get(npc_idx) {
                Some(p) => p.clone(),
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            (person, settlement.id.clone())
        };

        // Add lineage record. The authoritative cause is `death_cause` — both
        // death paths (collapse and old age) set it. The old code read
        // `self.collapse.outcome`, which is stale/None for an old-age death, so
        // OldAge deaths were mislabeled with a leftover collapse outcome or
        // "unknown" and never showed as "old age" in the lineage.
        let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        let cause = self
            .death_cause
            .map(|d| d.label().to_string())
            .or_else(|| self.collapse.as_ref().map(|c| format!("{:?}", c.outcome)))
            .unwrap_or_else(|| "unknown".to_string());

        self.lineage.push(LineageRecord {
            predecessor_name: dead_person.name.clone(),
            predecessor_id: dead_person.id.clone(),
            cause,
            settlement_id: settlement_id.clone(),
            tick,
        });

        // Create new PlayerStart from NPC. The heir keeps a keepsake — a few
        // coins and one thing the dead carried — and the family's standing
        // doesn't vanish with the body (half of it carries to the heir).
        let mut inherited = Inventory::default();
        let keepsake = self.player_start.as_ref().map(|ps| {
            let coins = ps.inventory.get(ItemType::Coin).min(3);
            let item = ps
                .inventory
                .items
                .keys()
                .copied()
                .find(|i| *i != ItemType::Coin && ps.inventory.get(*i) > 0);
            (coins, item)
        });
        if let Some((coins, item)) = keepsake {
            if coins > 0 {
                inherited.add(ItemType::Coin, coins);
            }
            if let Some(it) = item {
                inherited.add(it, 1);
            }
        }
        let new_player_start = PlayerStart {
            person: npc_person.clone(),
            reroll_count: 0,
            point_buy_adjustments: Vec::new(),
            accepted: true,
            inventory: inherited,
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

        // +0.15 reputation boost, plus half the dead's standing carries over —
        // the family name is remembered.
        let dead_standing = self
            .sim
            .as_ref()
            .map(|sim| sim.reputation.get(&dead_person.id, &heir_settlement_id))
            .unwrap_or(0.5);
        if let Some(ref mut sim) = self.sim {
            sim.reputation
                .adjust_local(&npc_person.id, &heir_settlement_id, 0.15);
            let carry = (dead_standing - 0.5) * 0.5;
            if carry.abs() > 0.01 {
                sim.reputation
                    .adjust_local(&npc_person.id, &heir_settlement_id, carry);
            }
        }

        // Switch player
        let heir_band = npc_person.age_band.clone();
        self.player_start = Some(new_player_start);
        self.vitals = PlayerVitals::default();
        self.inter_people_bias = InterPeopleBias::new(PeopleKind::from_name(&npc_person.people));
        // The heir is a fresh life: reset aging, salted by lineage depth so each
        // generation rolls its own lifespan.
        self.death_cause = None;
        self.begin_life_aging(&heir_band, self.lineage.len() as u64);

        // Continue on Map screen
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
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
        // Festival days: the doors are open and everything is half price.
        if self
            .current_settlement()
            .is_some_and(|s| s.in_festival(self.clock.day))
        {
            cost = (cost / 2).max(1);
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
                // Taverns are where word travels. Prefer a rumor grounded in
                // the actual world (famine, caravans, festivals, construction)
                // — actionable information — and fall back to flavor.
                let day = self.clock.day;
                let mut heard: Option<String> = None;
                if let Some(ref mut sim) = self.sim {
                    let tick = sim.world.tick;
                    let text =
                        crate::sim::rumors::informed_rumor(sim, day, tick).unwrap_or_else(|| {
                            let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                                .fork_for(&format!("tavern-rumor-{tick}"));
                            crate::sim::journal::rumor_text(&mut rng)
                        });
                    sim.log(tick, crate::sim::journal::Voice::Rumor, text.clone());
                    heard = Some(text);
                }
                self.status_msg = Some(match heard {
                    Some(r) => {
                        format!("Rested at tavern ({} coins). You overhear: \"{}\"", cost, r)
                    }
                    None => format!("Rested at tavern (+energy, +hunger, 2h, {} coins)", cost),
                });
            }
            SettlementService::Temple => {
                self.vitals.hunger = (self.vitals.hunger + 0.5).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.3).min(1.0);
                self.advance_clock(3);
                // The temple healers also tend the sick — clear active illness.
                let cured = self
                    .player_start
                    .as_mut()
                    .map(|ps| {
                        let had = !ps.person.illnesses.is_empty();
                        ps.person.illnesses.clear();
                        had
                    })
                    .unwrap_or(false);
                self.status_msg = Some(if cured {
                    format!("Blessed and healed at temple (illness cured, 3h, {cost} coins)")
                } else {
                    format!("Blessed at temple (+hunger, +energy, 3h, {cost} coins)")
                });
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

    /// A full night's rest (8h). Kept for the legacy/default path.
    pub fn rest(&mut self) {
        self.rest_hours(8);
    }

    /// Rest for a chosen number of hours (a short spurt up to a full night),
    /// clamped to [1, MAX_REST_HOURS]. Effects and risk scale with the duration;
    /// quality still depends on where you rest (settlement, shelter, the cold).
    pub fn rest_hours(&mut self, hours: u32) {
        use crate::sim::rest::{tile_rest_quality, RestQuality};

        let hours = hours.clamp(1, MAX_REST_HOURS);
        let h = hours as f64;
        let tod = crate::model::TimeOfDay::from_hour(self.clock.hour);
        let was_deep_night = tod == crate::model::TimeOfDay::DeepNight;
        let on_settlement = self.player_on_settlement().is_some();
        // A structure on this tile raises the rest tier — your own walls count.
        // Tier drives stamina rate, encounter risk, and journal flavor; the
        // tile_rest_quality shelter flags were previously hardcoded false, so a
        // built Home still rested like open ground (and deep night forced
        // "out in the cold" even inside it).
        let structure_tier = self.structure_at_player().map(|s| match s.kind {
            crate::sim::structures::BuildKind::Tarp => RestQuality::Campfire,
            crate::sim::structures::BuildKind::LeanTo
            | crate::sim::structures::BuildKind::TarpTent => RestQuality::LeanTo,
            crate::sim::structures::BuildKind::Laavu | crate::sim::structures::BuildKind::Kota => {
                RestQuality::SettlementFloor
            }
            crate::sim::structures::BuildKind::Cabin
            | crate::sim::structures::BuildKind::Longhouse
            | crate::sim::structures::BuildKind::Home => RestQuality::Inn,
        });
        let sheltered = structure_tier.is_some() || on_settlement;
        let base_quality = if was_deep_night && !sheltered {
            RestQuality::OutInCold
        } else {
            tile_rest_quality(on_settlement, false, false, false)
        };
        let quality = match structure_tier {
            Some(t) if t.stamina_per_hour() > base_quality.stamina_per_hour() => t,
            _ => base_quality,
        };
        let stamina_gain = quality.stamina_per_hour() * h;
        let morale_gain = quality.morale_per_hour() * h;
        let encounter_risk = quality.encounter_risk_per_hour() * h;

        let quality_label = crate::sim::journal::rest_quality_label(
            on_settlement,
            quality == RestQuality::Inn,
            false,
            false,
        );

        self.advance_clock(hours);
        self.vitals.rest(hours);
        let mut scouted = false;
        if let Some(ref mut ps) = self.player_start {
            for companion in &mut ps.companions {
                companion.rest(h / 8.0);
                let action_seed = self
                    .seed
                    .wrapping_add((self.clock.day as u64 * 24 + self.clock.hour as u64) * 137);
                if let Some(action) = companion.autonomous_action(action_seed) {
                    // The flavor promises a yield — deliver it to the player.
                    match action {
                        crate::model::CompanionAction::Hunt => {
                            ps.inventory.add(crate::model::ItemType::Food, 1)
                        }
                        crate::model::CompanionAction::Gather => {
                            ps.inventory.add(crate::model::ItemType::Herb, 1)
                        }
                        crate::model::CompanionAction::Scout => scouted = true,
                    }
                    let flavor = companion.apply_action(action);
                    self.status_msg = Some(format!("{} {}", companion.name, flavor));
                }
                // Companion eats from shared inventory when hungry
                if companion.food_need > 50.0 && ps.inventory.has(crate::model::ItemType::Food) {
                    ps.inventory.remove(crate::model::ItemType::Food, 1);
                    companion.feed(0.5);
                }
                // A goat is a walking larder: a proper rest stop yields milk.
                // milk_production existed on Animal but was never collected.
                let milk = companion.animal.milk_production();
                if milk > 0
                    && hours >= 4
                    && companion.mood() != crate::model::CompanionMood::Unhappy
                {
                    ps.inventory.add(crate::model::ItemType::Food, milk);
                }
            }
        }
        // A scouting companion reveals the ground around the player.
        if scouted {
            if let Some(pos) = self.player_pos {
                self.reveal_around(pos.region_idx, pos.px, pos.py);
            }
        }
        // Rest is when wounds get tended and snares get checked.
        if let Some(ref mut ps) = self.player_start {
            // Tending an illness with a bandage eases it and shortens its course.
            if !ps.person.illnesses.is_empty() && ps.inventory.remove(ItemType::Bandage, 1) {
                for d in ps.person.illnesses.iter_mut() {
                    d.tend();
                }
                self.status_msg = Some("You dress your sickness with a bandage.".into());
            }
            // A set trap yields food over a proper rest in the wild — if the
            // land still carries game. Trapping draws the valley down.
            let richness = self
                .player_pos
                .and_then(|pos| {
                    self.sim
                        .as_ref()
                        .and_then(|s| s.world.regions.get(pos.region_idx))
                })
                .map(|r| r.game_richness)
                .unwrap_or(1.0);
            if hours >= 4
                && !on_settlement
                && richness > 0.3
                && ps.inventory.has(ItemType::Trap)
                && !ps.inventory.is_broken(ItemType::Trap)
            {
                ps.inventory.add(ItemType::Food, 1);
                ps.inventory.use_tool(ItemType::Trap);
                if let Some(pos) = self.player_pos {
                    if let Some(region) = self
                        .sim
                        .as_mut()
                        .and_then(|s| s.world.regions.get_mut(pos.region_idx))
                    {
                        region.game_richness = (region.game_richness - 0.02).max(0.0);
                    }
                }
            } else if hours >= 4
                && !on_settlement
                && richness <= 0.3
                && ps.inventory.has(ItemType::Trap)
            {
                self.status_msg = Some("The snare sits empty. This land is trapped out.".into());
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
        let dur_frac = h / 8.0;
        if structure_bonus > 0.0 {
            self.vitals.energy = (self.vitals.energy + structure_bonus * dur_frac).min(1.0);
            self.vitals.hunger = (self.vitals.hunger - structure_bonus * 0.3 * dur_frac).max(0.0);
        }
        self.vitals.energy = (self.vitals.energy + stamina_gain / 8.0).min(1.0);
        self.god_affinity
            .adjust(GodName::Kukri, 0.02 + morale_gain * 0.1);
        if quality == RestQuality::Inn {
            self.vitals.energy = (self.vitals.energy + 0.05 * dur_frac).min(1.0);
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
        self.play_sound(crate::audio::SoundEvent::Ambient);
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
            Some(region.weather)
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
        let companion_travel = self.companion_travel_mult();
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
                let hours = ((terrain.travel_hours() as f64 * weather_mult * companion_travel)
                    .round() as i32
                    + bias_mod)
                    .max(1) as u32;
                self.advance_clock(hours);
                self.log_travel(terrain);
                self.reveal_around(region_idx, px, py);
                self.check_encounter(terrain);
                self.check_memorial();
                self.check_discovery(region_idx, px, py);
                self.check_quests_on_travel(region_idx);
                if self.encounter.is_none() {
                    self.screen = Screen::World { region_idx };
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
                let hours = ((terrain.travel_hours() as f64 * weather_mult * companion_travel)
                    .round() as i32
                    + bias_mod)
                    .max(1) as u32;
                self.advance_clock(hours);
                self.log_travel(terrain);
                self.reveal_around(region_idx, px, py);
                self.check_encounter(terrain);
                self.check_memorial();
                self.check_discovery(region_idx, px, py);
                if self.encounter.is_none() {
                    self.screen = Screen::World { region_idx };
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
                        let mut settle_positions: Vec<(usize, usize)> = Vec::new();
                        for (i, &t) in region.terrain.tiles.iter().enumerate() {
                            if t == Terrain::Settlement {
                                let sx = i % region.terrain.width;
                                let sy = i / region.terrain.width;
                                settle_positions.push((sx, sy));
                            }
                        }
                        settle_positions.sort_by_key(|(x, _)| *x);
                        if let Some(idx) = settle_positions
                            .iter()
                            .position(|&(sx, sy)| sx == pos.px && sy == pos.py)
                        {
                            return Some((pos.region_idx, idx.min(region.settlements.len() - 1)));
                        }
                        return Some((pos.region_idx, 0));
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
        if let AppEvent::Key(key) = &event {
            if key.code == crossterm::event::KeyCode::Esc {
                self.play_sound(crate::audio::SoundEvent::UiCancel);
            }
        }
        match event {
            AppEvent::Key(key) => match self.screen {
                Screen::TitleScreen => {
                    super::input::title::handle_title_input(self, key);
                }
                Screen::SaveBrowser { .. } => {
                    super::input::save_browser::handle_save_browser_input(self, key);
                }
                Screen::SaveSlots { scroll } => {
                    super::input::save_slots::handle_save_slots_input(self, key, scroll);
                }
                Screen::RestPrompt { hours } => {
                    super::input::rest_prompt::handle_rest_prompt_input(self, key, hours);
                }
                Screen::CharacterCreation => {
                    super::input::character_creation::handle_character_creation_input(self, key);
                }
                Screen::World { region_idx } => {
                    super::input::world::handle_world_input(self, key, region_idx);
                }
                Screen::Overmap { region_idx } => {
                    super::input::overmap::handle_overmap_input(self, key, region_idx);
                }
                Screen::Inventory => {
                    super::input::inventory::handle_inventory_input(self, key);
                }
                Screen::Craft { scroll } => {
                    let new_scroll = super::input::craft::handle_craft_input(self, key, scroll);
                    if let Screen::Craft { .. } = self.screen {
                        self.screen = Screen::Craft { scroll: new_scroll };
                    }
                }
                Screen::Location {
                    scroll,
                    region_idx,
                    settlement_idx,
                } => {
                    let new_scroll = super::input::location::handle_location_input(
                        self,
                        key,
                        scroll,
                        region_idx,
                        settlement_idx,
                    );
                    if let Screen::Location {
                        region_idx,
                        settlement_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Location {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                        };
                    }
                }
                Screen::Npc {
                    scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => {
                    let new_scroll = super::input::npc::handle_npc_input(
                        self,
                        key,
                        scroll,
                        region_idx,
                        settlement_idx,
                        person_idx,
                    );
                    if let Screen::Npc {
                        region_idx,
                        settlement_idx,
                        person_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Npc {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                            person_idx,
                        };
                    }
                }
                Screen::Journal { scroll } => {
                    let new_scroll = super::input::journal::handle_journal_input(self, key, scroll);
                    if let Screen::Journal { .. } = self.screen {
                        self.screen = Screen::Journal { scroll: new_scroll };
                    }
                }
                Screen::EncounterLog { scroll } => {
                    let new_scroll =
                        super::input::encounter_log::handle_encounter_log_input(self, key, scroll);
                    if let Screen::EncounterLog { .. } = self.screen {
                        self.screen = Screen::EncounterLog { scroll: new_scroll };
                    }
                }
                Screen::Talk {
                    scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => {
                    let new_scroll = super::input::talk::handle_talk_input(
                        self,
                        key,
                        scroll,
                        region_idx,
                        settlement_idx,
                        person_idx,
                    );
                    if let Screen::Talk {
                        region_idx,
                        settlement_idx,
                        person_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Talk {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                            person_idx,
                        };
                    }
                }
                Screen::Market {
                    scroll,
                    region_idx: _,
                    settlement_idx: _,
                } => {
                    let new_scroll = super::input::market::handle_market_input(self, key, scroll);
                    if let Screen::Market {
                        region_idx,
                        settlement_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Market {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                        };
                    }
                }
                Screen::Encounter => {
                    super::input::encounter::handle_encounter_input(self, key);
                }
                Screen::Collapse => {
                    super::input::collapse::handle_collapse_input(self, key);
                }
                Screen::GameOver => {
                    super::input::game_over::handle_game_over_input(self, key);
                }
                Screen::Help => {
                    super::input::help::handle_help_input(self, key);
                }
                Screen::Settings => {
                    super::input::settings::handle_settings_input(self, key);
                }
            },
            AppEvent::Tick => {}
        }
    }
}
