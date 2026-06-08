use std::fs;
use std::path::Path;

use crate::model::{GameClock, GodAffinity, InterPeopleBias, PlayerPos, PlayerStart, PlayerVitals};
use crate::sim::collapse_log::CollapseEvent;
use crate::sim::SimState;

#[derive(serde::Serialize, serde::Deserialize)]
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
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CompactSave {
    pub seed: u64,
    pub player_choices: Vec<PlayerChoice>,
    pub tick: u64,
}

pub fn save_game(data: &SaveData, filename: &str) -> Result<(), String> {
    let path = Path::new(filename);
    let ron_string = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(path, ron_string).map_err(|e| format!("Failed to write file: {}", e))
}

pub fn load_game(filename: &str) -> Result<SaveData, String> {
    let path = Path::new(filename);
    let contents = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    ron::from_str(&contents).map_err(|e| format!("Failed to deserialize: {}", e))
}

pub fn save_compact(data: &CompactSave, filename: &str) -> Result<(), String> {
    let path = Path::new(filename);
    let ron_string = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize compact: {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(path, ron_string).map_err(|e| format!("Failed to write compact: {}", e))
}

pub fn load_compact(filename: &str) -> Result<CompactSave, String> {
    let path = Path::new(filename);
    let contents =
        fs::read_to_string(path).map_err(|e| format!("Failed to read compact: {}", e))?;
    ron::from_str(&contents).map_err(|e| format!("Failed to deserialize compact: {}", e))
}

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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::model::PeopleKind;

    #[test]
    fn save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.ron");
        let path_str = path.to_str().unwrap();
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
        };
        save_game(&data, path_str).expect("save should succeed");
        let loaded = load_game(path_str).expect("load should succeed");
        assert_eq!(
            data.sim.world.tick, loaded.sim.world.tick,
            "tick should match"
        );
        assert_eq!(
            data.sim.world.regions.len(),
            loaded.sim.world.regions.len(),
            "region count should match"
        );
    }

    #[test]
    fn load_nonexistent_file() {
        let result = load_game("/tmp/deep-world-tui-nonexistent.ron");
        assert!(result.is_err(), "loading nonexistent file should fail");
    }

    #[test]
    fn compact_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("compact.ron");
        let path_str = path.to_str().unwrap();
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
        save_compact(&data, path_str).expect("compact save should succeed");
        let loaded = load_compact(path_str).expect("compact load should succeed");
        assert_eq!(loaded.seed, 12345);
        assert_eq!(loaded.tick, 42);
        assert_eq!(loaded.player_choices, choices);
    }

    #[test]
    fn compact_load_nonexistent_file() {
        let result = load_compact("/tmp/deep-world-tui-nonexistent-compact.ron");
        assert!(result.is_err(), "loading nonexistent compact should fail");
    }

    #[test]
    fn compact_save_is_smaller_than_full() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
}
