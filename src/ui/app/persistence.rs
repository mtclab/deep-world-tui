use crate::model::{EncounterAction, ItemType, SettlementService};
use crate::save::{self, SaveData};
use crate::save_migrations;

use super::*;

impl App {
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

    #[allow(deprecated)]
    pub(super) fn build_save_data(&self) -> Option<SaveData> {
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
            fortune: self.fortune,
            gift: self.gift,
            tax_unpaid_seasons: self.tax_unpaid_seasons,
            last_tax_day: self.last_tax_day,
            encounter_log: self.encounter_log.clone(),
            player_farms: self.player_farms.clone(),
            homestead_settlers: self.homestead_settlers.clone(),
            homestead_rumored: self.homestead_rumored,
            founding_check_day: self.founding_check_day,
            spouse_id: self.spouse_id.clone(),
            widowed_day: self.widowed_day,
            household_children: self.household_children.clone(),
            travel_debt: self.travel_debt,
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

    /// Open the manual save-slot picker.
    pub fn open_save_slots(&mut self) {
        self.save_entries = save::saves_dir_list();
        self.screen = Screen::SaveSlots { scroll: 0 };
    }

    /// Parse an item by its display name (case-insensitive) — the string form
    /// used by the recorded-choice API.
    pub fn item_from_name(name: &str) -> Option<ItemType> {
        ItemType::tradeable_items()
            .into_iter()
            .chain([ItemType::Coin])
            .find(|i| i.name().eq_ignore_ascii_case(name))
    }

    /// Apply one recorded player choice — the headless action API used by
    /// AI play, session recording, and deterministic replay. The
    /// PlayerChoice/CompactSave types existed with serialization support but
    /// nothing ever applied or recorded them.
    pub fn apply_choice(&mut self, choice: &crate::save::PlayerChoice) {
        use crate::save::PlayerChoice as C;
        match choice {
            C::TravelTo { region_idx, px, py } => {
                // Recorded as the resulting tile; replays as a relative step
                // when adjacent, else a direct reposition within the region.
                if let Some(pos) = self.player_pos {
                    if pos.region_idx == *region_idx {
                        let dx = *px as i32 - pos.px as i32;
                        let dy = *py as i32 - pos.py as i32;
                        if dx.abs() <= 1 && dy.abs() <= 1 {
                            self.move_player(dx, dy);
                            return;
                        }
                    }
                }
                self.enter_map(*region_idx);
            }
            C::EnterSettlement {
                region_idx,
                settlement_idx,
            } => self.enter_settlement(*region_idx, *settlement_idx),
            C::ExitSettlement => self.exit_settlement(),
            C::Gather => self.gather(),
            C::Rest => self.rest(),
            C::TendSelf => self.tend_illness(),
            C::ForageHerbs => self.forage_herbs(),
            C::UseService { service } => {
                let svc = match service.to_ascii_lowercase().as_str() {
                    "tavern" => Some(SettlementService::Tavern),
                    "temple" => Some(SettlementService::Temple),
                    "forge" => Some(SettlementService::Forge),
                    "hearth" => Some(SettlementService::Hearth),
                    "trapworkshop" | "trap" => Some(SettlementService::TrapWorkshop),
                    "archive" => Some(SettlementService::Archive),
                    "tradepost" => Some(SettlementService::TradePost),
                    "shrine" => Some(SettlementService::Shrine),
                    _ => None,
                };
                if let Some(svc) = svc {
                    self.use_service(svc);
                }
            }
            C::CraftRecipe { recipe_idx } => {
                self.enter_craft();
                self.craft_recipe(*recipe_idx);
                self.exit_craft();
            }
            C::ResolveEncounter { action } => {
                let act = match action.to_ascii_lowercase().as_str() {
                    "flee" => Some(EncounterAction::Flee),
                    "bribe" => Some(EncounterAction::Bribe),
                    "calm" => Some(EncounterAction::Calm),
                    "intimidate" => Some(EncounterAction::Intimidate),
                    "talk" => Some(EncounterAction::Talk),
                    "trade" => Some(EncounterAction::Trade),
                    "shelter" => Some(EncounterAction::Shelter),
                    "push" | "pushthrough" => Some(EncounterAction::PushThrough),
                    _ => None,
                };
                if let Some(a) = act {
                    self.resolve_encounter(a);
                }
            }
            C::DismissCollapse => self.dismiss_collapse(),
            C::BuyItem { item } => {
                if let Some(i) = Self::item_from_name(item) {
                    self.buy_item(i);
                }
            }
            C::SellItem { item } => {
                if let Some(i) = Self::item_from_name(item) {
                    self.sell_item(i);
                }
            }
            C::StealItem { item } => {
                if let Some(i) = Self::item_from_name(item) {
                    self.steal_item(i);
                }
            }
            C::Build { kind } => {
                let wanted = kind
                    .as_deref()
                    .and_then(crate::sim::structures::BuildKind::from_name);
                self.start_build_kind(wanted);
            }
            C::StashItem { item, count } => {
                if let Some(i) = Self::item_from_name(item) {
                    self.stash_item(i, *count);
                }
            }
            C::TakeItem { item, count } => {
                if let Some(i) = Self::item_from_name(item) {
                    self.take_item(i, *count);
                }
            }
            C::Plant => self.plant(),
            C::PlantCrop { crop } => {
                self.plant_crop(crate::model::economy::CropType::from_name(crop))
            }
            C::Harvest => self.harvest(),
            C::Talk { person_idx } => {
                if let Some(pos) = self.player_pos {
                    self.enter_talk(pos.region_idx, 0, *person_idx);
                }
            }
            C::Court { person_idx } => self.court(*person_idx),
        }
    }

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
        // Saves from before the 80x40 sectors get upscaled 2x (the world
        // keeps its exact shape); player-side coordinates follow below.
        let mut upscale_factor = 1usize;
        if let Some(ref mut sim) = self.sim {
            // Old saves double until they reach the current sector size
            // (40x20 and 80x40 both arrive at 160x80, shape intact).
            while crate::gen::world::upscale_world_2x(sim) {
                upscale_factor *= 2;
            }
        }
        let upscaled = upscale_factor > 1;
        // Saves from before settlement footprints carry point-settlements;
        // give them anchors and paint their ground.
        if let Some(ref mut sim) = self.sim {
            crate::gen::world::fixup_settlement_anchors(&mut sim.world);
        }
        self.player_start = data.player_start;
        self.clock = data.clock;
        self.vitals = data.vitals;
        self.player_pos = data.player_pos;
        if upscaled {
            if let Some(ref mut pos) = self.player_pos {
                pos.px *= upscale_factor;
                pos.py *= upscale_factor;
            }
        }
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
        self.player_farms = data.player_farms;
        if upscaled {
            for f in self.player_farms.iter_mut() {
                f.x *= upscale_factor as u32;
                f.y *= upscale_factor as u32;
            }
            for e in self.explored.iter_mut() {
                *e = e.upscale(upscale_factor);
            }
        }
        self.homestead_settlers = data.homestead_settlers;
        self.homestead_rumored = data.homestead_rumored;
        self.founding_check_day = data.founding_check_day;
        self.spouse_id = data.spouse_id;
        self.widowed_day = data.widowed_day;
        self.household_children = data.household_children;
        self.travel_debt = data.travel_debt;
        // The life's star, restored. A pre-aging save (lifespan 0) has none
        // saved; apply_loaded_aging re-rolls the life below, fortune with it.
        self.fortune = data.fortune;
        self.gift = data.gift;
        // Gift-strain is a fresh-day matter; it does not survive a reload.
        self.gift_strain = 0.0;
        self.gift_overworked_days = 0;
        self.gift_revealed = false;
        self.tax_unpaid_seasons = data.tax_unpaid_seasons;
        self.last_tax_day = data.last_tax_day;
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
}
