use std::fs;
use std::path::Path;

use crate::model::{GameClock, GodAffinity, InterPeopleBias, PlayerPos, PlayerStart, PlayerVitals};
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
}
