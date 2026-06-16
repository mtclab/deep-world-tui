use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::model::{GameClock, GodAffinity, InterPeopleBias, PlayerPos, PlayerStart, PlayerVitals};
use crate::save_migrations::CURRENT_SAVE_VERSION;
use crate::sim::collapse_log::CollapseEvent;
use crate::sim::hints::HintTracker;
use crate::sim::milestones::MilestoneTracker;
use crate::sim::SimState;

const SAVES_DIR: &str = "saves";

fn sanitize_save_path(filename: &str) -> Result<PathBuf, String> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(format!("Path traversal blocked: {}", filename));
    }
    let path = Path::new(filename);
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("Invalid filename: {}", filename))?;
    if file_name.is_empty() {
        return Err(format!("Empty filename: {}", filename));
    }
    Ok(PathBuf::from(SAVES_DIR).join(file_name))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LineageRecord {
    pub predecessor_name: String,
    pub predecessor_id: String,
    pub cause: String,
    pub settlement_id: String,
    pub tick: u64,
}

#[allow(dead_code)]
fn current_save_version() -> u32 {
    CURRENT_SAVE_VERSION
}

fn default_true() -> bool {
    true
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SaveData {
    pub sim: SimState,
    pub player_start: Option<PlayerStart>,
    pub clock: GameClock,
    pub vitals: PlayerVitals,
    pub player_pos: Option<PlayerPos>,
    pub god_affinity: GodAffinity,
    pub inter_people_bias: InterPeopleBias,
    pub encounters_had: u32,
    pub collapses_had: u32,
    #[serde(default)]
    pub collapse_log: Vec<CollapseEvent>,
    #[serde(default)]
    pub lineage: Vec<LineageRecord>,
    #[serde(default)]
    pub milestones: MilestoneTracker,
    #[serde(default)]
    pub explored: Vec<crate::model::ExploredMap>,
    #[serde(default)]
    pub version: u32,
    #[serde(default = "default_true")]
    #[deprecated = "unused — hint_tracker controls onboarding; kept for save compat"]
    pub first_run: bool,
    #[serde(default)]
    pub hint_tracker: HintTracker,
    #[serde(default)]
    pub start_age_years: u32,
    #[serde(default)]
    pub birth_day: u32,
    #[serde(default)]
    pub lifespan_years: u32,
    /// The luck this life was born with. Old saves default to middling.
    #[serde(default)]
    pub fortune: crate::model::Fortune,
    /// The craft-gift this life was born with. Old saves default to craftless.
    #[serde(default)]
    pub gift: crate::model::Gift,
    /// Seasons of unpaid hearth-tax the player owes the polity.
    #[serde(default)]
    pub tax_unpaid_seasons: u32,
    #[serde(default)]
    pub last_tax_day: u32,
    // The player's encounter history (day/hour/kind/action) lived in App but
    // was never saved, so the log reset on every load.
    #[serde(default)]
    pub encounter_log: crate::model::EncounterLog,
    /// The player's worked fields.
    #[serde(default)]
    pub player_farms: Vec<crate::model::economy::PlayerFarm>,
    /// Settlers camped by the homestead, waiting to become a hamlet.
    #[serde(default)]
    pub homestead_settlers: Vec<crate::model::Person>,
    #[serde(default)]
    pub homestead_rumored: bool,
    #[serde(default)]
    pub founding_check_day: u32,
    /// Marriage: the spouse's Person id, and the day the player was widowed.
    #[serde(default)]
    pub spouse_id: Option<String>,
    #[serde(default)]
    pub widowed_day: u32,
    #[serde(default)]
    pub household_children: Vec<crate::model::HouseholdChild>,
    #[serde(default)]
    pub travel_debt: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum PlayerChoice {
    TravelTo {
        region_idx: usize,
        px: u32,
        py: u32,
    },
    EnterSettlement {
        region_idx: usize,
        settlement_idx: usize,
    },
    ExitSettlement,
    Gather,
    Rest,
    UseService {
        service: String,
    },
    CraftRecipe {
        recipe_idx: usize,
    },
    ResolveEncounter {
        action: String,
    },
    DismissCollapse,
    BuyItem {
        item: String,
    },
    SellItem {
        item: String,
    },
    StealItem {
        item: String,
    },
    Build {
        kind: Option<String>,
    },
    Talk {
        person_idx: usize,
    },
    Plant,
    /// Plant a named crop (flax, winter-rye, …). `Plant` keeps the land's
    /// own pick; old recordings parse unchanged.
    PlantCrop {
        crop: String,
    },
    Harvest,
    StashItem {
        item: String,
        count: u32,
    },
    TakeItem {
        item: String,
        count: u32,
    },
    Court {
        person_idx: usize,
    },
    /// Sit down and treat your own sickness with what you carry (#451).
    TendSelf,
    /// Forage the ground for medicinal herbs (#456).
    ForageHerbs,
    /// Set out on the long roads to one of the continent's named cities (#456).
    JourneyToCity,
    /// Sit a while in prayer — devotion, the gods being withdrawn (#457).
    Pray,
    /// Keep a settlement's festival in earnest — deeper devotion (#457).
    ObserveFestival,
    /// Lay an offering at a shrine/temple — devotion that costs food (#457).
    MakeOffering,
    /// Set out on a pilgrimage from a holy site — the long road of devotion (#457).
    Pilgrimage,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CompactSave {
    pub seed: u64,
    pub player_choices: Vec<PlayerChoice>,
    pub tick: u64,
}

/// Number of manual save slots exposed to the player.
pub const SAVE_SLOT_COUNT: usize = 5;

/// Filename for a numbered manual save slot (1-based).
pub fn slot_filename(slot: usize) -> String {
    format!("slot_{slot}.ron")
}

pub fn save_game(data: &SaveData, filename: &str) -> Result<(), String> {
    let path = sanitize_save_path(filename)?;
    // Compact RON (no pretty-printing) — ~4x smaller than the indented form.
    let ron_string =
        ron::ser::to_string(data).map_err(|e| format!("Failed to serialize: {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(&path, ron_string).map_err(|e| format!("Failed to write file: {}", e))
}

pub fn save_lineage(data: &SaveData, seed: u64) -> Result<(), String> {
    let filename = format!("lineage_{}.ron", seed);
    save_game(data, &filename)
}

pub fn load_game(filename: &str) -> Result<SaveData, String> {
    load_game_file(filename)
}

pub fn load_game_file(filename: &str) -> Result<SaveData, String> {
    let path = sanitize_save_path(filename)?;
    let contents = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut data: SaveData =
        ron::from_str(&contents).map_err(|e| format!("Failed to deserialize: {}", e))?;
    crate::save_migrations::migrate(&mut data)?;
    Ok(data)
}

#[derive(Debug, Clone)]
pub struct SaveEntry {
    pub filename: String,
    pub character_name: Option<String>,
    pub people: Option<String>,
    pub day: u32,
    pub timestamp: Option<u64>,
}

pub fn saves_dir_list() -> Vec<SaveEntry> {
    let dir = Path::new(SAVES_DIR);
    if !dir.exists() {
        return Vec::new();
    }
    let mut entries: Vec<SaveEntry> = Vec::new();
    let Ok(files) = fs::read_dir(dir) else {
        return Vec::new();
    };
    for file in files.flatten() {
        let path = file.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ron") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let mtime = file
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        match load_game_file(&filename) {
            Ok(data) => {
                let name = data.player_start.as_ref().map(|ps| ps.person.name.clone());
                let people = data
                    .player_start
                    .as_ref()
                    .map(|ps| ps.person.people.to_string());
                let day = data.clock.day;
                entries.push(SaveEntry {
                    filename,
                    character_name: name,
                    people,
                    day,
                    timestamp: mtime,
                });
            }
            Err(_) => {
                entries.push(SaveEntry {
                    filename,
                    character_name: None,
                    people: None,
                    day: 0,
                    timestamp: mtime,
                });
            }
        }
    }
    entries.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
    entries
}

pub fn delete_save(filename: &str) -> Result<(), String> {
    let path = sanitize_save_path(filename)?;
    fs::remove_file(&path).map_err(|e| format!("Failed to delete save: {}", e))
}

pub fn save_compact(data: &CompactSave, filename: &str) -> Result<(), String> {
    let path = sanitize_save_path(filename)?;
    let ron_string = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize compact: {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(&path, ron_string).map_err(|e| format!("Failed to write compact: {}", e))
}

pub fn load_compact(filename: &str) -> Result<CompactSave, String> {
    let path = sanitize_save_path(filename)?;
    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read compact: {}", e))?;
    ron::from_str(&contents).map_err(|e| format!("Failed to deserialize compact: {}", e))
}

#[allow(deprecated)]
pub fn restore_from_compact(
    compact: &CompactSave,
    charts: &crate::charts::Charts,
) -> Result<SaveData, String> {
    let mut sim = SimState::new(compact.seed, charts.clone());
    for _ in 0..compact.tick {
        sim.step();
    }
    Ok(SaveData {
        sim,
        player_start: None,
        clock: GameClock::default(),
        vitals: PlayerVitals::default(),
        player_pos: None,
        god_affinity: GodAffinity::new(),
        inter_people_bias: InterPeopleBias::new(crate::model::PeopleKind::Metsik),
        encounters_had: 0,
        collapses_had: 0,
        collapse_log: Vec::new(),
        lineage: Vec::new(),
        milestones: MilestoneTracker::new(),
        explored: Vec::new(),
        version: CURRENT_SAVE_VERSION,
        first_run: true,
        hint_tracker: HintTracker::default(),
        start_age_years: 0,
        birth_day: 0,
        lifespan_years: 0,
        fortune: Default::default(),
        gift: Default::default(),
        tax_unpaid_seasons: 0,
        last_tax_day: 0,
        encounter_log: Default::default(),
        player_farms: Vec::new(),
        homestead_settlers: Vec::new(),
        homestead_rumored: false,
        founding_check_day: 0,
        spouse_id: None,
        widowed_day: 0,
        household_children: Vec::new(),
        travel_debt: 0.0,
    })
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::model::PeopleKind;

    #[test]
    fn save_load_roundtrip() {
        let charts = charts::load_charts().unwrap();
        let sim = SimState::new(42, charts);
        let data = SaveData {
            sim,
            player_start: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            player_pos: None,
            god_affinity: GodAffinity::new(),
            inter_people_bias: InterPeopleBias::new(PeopleKind::Metsik),
            encounters_had: 0,
            collapses_had: 0,
            collapse_log: Vec::new(),
            lineage: Vec::new(),
            milestones: MilestoneTracker::new(),
            explored: Vec::new(),
            version: CURRENT_SAVE_VERSION,
            first_run: true,
            hint_tracker: HintTracker::default(),
            start_age_years: 0,
            birth_day: 0,
            lifespan_years: 0,
            fortune: Default::default(),
            gift: Default::default(),
            tax_unpaid_seasons: 0,
            last_tax_day: 0,
            encounter_log: Default::default(),
            player_farms: Vec::new(),
            homestead_settlers: Vec::new(),
            homestead_rumored: false,
            founding_check_day: 0,
            spouse_id: None,
            widowed_day: 0,
            household_children: Vec::new(),
            travel_debt: 0.0,
        };
        save_game(&data, "test_save.ron").expect("save should succeed");
        let loaded = load_game("test_save.ron").expect("load should succeed");
        assert_eq!(
            data.sim.world.tick, loaded.sim.world.tick,
            "tick should match"
        );
        assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
        assert_eq!(
            data.sim.world.regions.len(),
            loaded.sim.world.regions.len(),
            "region count should match"
        );
        let _ = std::fs::remove_file("saves/test_save.ron");
    }

    #[test]
    fn load_nonexistent_file() {
        let result = load_game("nonexistent_test_file.ron");
        assert!(result.is_err(), "loading nonexistent file should fail");
    }

    #[test]
    fn compact_save_load_roundtrip() {
        let choices = vec![
            PlayerChoice::TravelTo {
                region_idx: 0,
                px: 5,
                py: 3,
            },
            PlayerChoice::EnterSettlement {
                region_idx: 0,
                settlement_idx: 1,
            },
            PlayerChoice::Gather,
            PlayerChoice::Rest,
            PlayerChoice::ExitSettlement,
            PlayerChoice::UseService {
                service: "tavern".into(),
            },
            PlayerChoice::CraftRecipe { recipe_idx: 2 },
            PlayerChoice::ResolveEncounter {
                action: "flee".into(),
            },
            PlayerChoice::DismissCollapse,
        ];
        let data = CompactSave {
            seed: 12345,
            player_choices: choices.clone(),
            tick: 42,
        };
        save_compact(&data, "test_compact.ron").expect("compact save should succeed");
        let loaded = load_compact("test_compact.ron").expect("compact load should succeed");
        assert_eq!(loaded.seed, 12345);
        assert_eq!(loaded.tick, 42);
        assert_eq!(loaded.player_choices, choices);
        let _ = std::fs::remove_file("saves/test_compact.ron");
    }

    #[test]
    fn compact_load_nonexistent_file() {
        let result = load_compact("nonexistent_compact.ron");
        assert!(result.is_err(), "loading nonexistent compact should fail");
    }

    #[test]
    fn compact_save_is_smaller_than_full() {
        let charts = charts::load_charts().unwrap();
        let sim = SimState::new(42, charts);
        let full_data = SaveData {
            sim,
            player_start: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            player_pos: None,
            god_affinity: GodAffinity::new(),
            inter_people_bias: InterPeopleBias::new(PeopleKind::Metsik),
            encounters_had: 0,
            collapses_had: 0,
            collapse_log: Vec::new(),
            lineage: Vec::new(),
            milestones: MilestoneTracker::new(),
            explored: Vec::new(),
            version: CURRENT_SAVE_VERSION,
            first_run: true,
            hint_tracker: HintTracker::default(),
            start_age_years: 0,
            birth_day: 0,
            lifespan_years: 0,
            fortune: Default::default(),
            gift: Default::default(),
            tax_unpaid_seasons: 0,
            last_tax_day: 0,
            encounter_log: Default::default(),
            player_farms: Vec::new(),
            homestead_settlers: Vec::new(),
            homestead_rumored: false,
            founding_check_day: 0,
            spouse_id: None,
            widowed_day: 0,
            household_children: Vec::new(),
            travel_debt: 0.0,
        };
        let compact_data = CompactSave {
            seed: 42,
            player_choices: vec![
                PlayerChoice::TravelTo {
                    region_idx: 0,
                    px: 5,
                    py: 3,
                },
                PlayerChoice::Gather,
                PlayerChoice::Rest,
            ],
            tick: 10,
        };
        let full_size = ron::ser::to_string_pretty(&full_data, ron::ser::PrettyConfig::default())
            .unwrap()
            .len();
        let compact_size =
            ron::ser::to_string_pretty(&compact_data, ron::ser::PrettyConfig::default())
                .unwrap()
                .len();
        assert!(
            compact_size < full_size,
            "compact ({}) should be smaller than full ({})",
            compact_size,
            full_size
        );
    }

    #[test]
    fn restore_from_compact_regenerates_world() {
        let charts = charts::load_charts().unwrap();
        let compact = CompactSave {
            seed: 99,
            player_choices: vec![],
            tick: 5,
        };
        let restored = restore_from_compact(&compact, &charts).expect("restore should succeed");
        assert_eq!(restored.sim.world.tick, 5, "tick should advance to 5");
        let fresh = SimState::new(99, charts);
        assert_eq!(
            restored.sim.world.regions.len(),
            fresh.world.regions.len(),
            "same seed should produce same number of regions"
        );
        assert_eq!(
            restored.sim.world.regions[0].name, fresh.world.regions[0].name,
            "same seed should produce same region names"
        );
    }

    #[test]
    fn restore_deterministic_from_same_seed() {
        let charts = charts::load_charts().unwrap();
        let compact = CompactSave {
            seed: 77,
            player_choices: vec![],
            tick: 10,
        };
        let r1 = restore_from_compact(&compact, &charts).expect("restore 1");
        let r2 = restore_from_compact(&compact, &charts).expect("restore 2");
        assert_eq!(r1.sim.world.tick, r2.sim.world.tick);
        assert_eq!(r1.sim.world.regions.len(), r2.sim.world.regions.len());
        assert_eq!(r1.sim.world.regions[0].name, r2.sim.world.regions[0].name);
    }

    #[test]
    fn path_traversal_blocked() {
        let result = sanitize_save_path("../../etc/passwd");
        assert!(result.is_err(), "path traversal with .. should be blocked");
        let result2 = sanitize_save_path("../secret.key");
        assert!(
            result2.is_err(),
            "parent directory traversal should be blocked"
        );
        let result3 = sanitize_save_path("/etc/shadow");
        assert!(result3.is_err(), "absolute path should be blocked");
        let ok = sanitize_save_path("my_save.ron");
        assert!(ok.is_ok(), "plain filename should be allowed");
        assert_eq!(ok.unwrap(), PathBuf::from("saves/my_save.ron"));
    }
}
