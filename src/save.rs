use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::model::{GameClock, PlayerPos, PlayerStart, PlayerVitals};
use crate::sim::SimState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub sim: SimState,
    pub player_start: Option<PlayerStart>,
    #[serde(default)]
    pub clock: GameClock,
    #[serde(default)]
    pub vitals: PlayerVitals,
    #[serde(default)]
    pub player_pos: Option<PlayerPos>,
}

pub fn save_game(data: &SaveData, path: &str) -> Result<()> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    let ron_str =
        ron::ser::to_string_pretty(data, Default::default()).context("serialize save data")?;
    fs::write(p, ron_str).context("write save file")?;
    Ok(())
}

pub fn load_game(path: &str) -> Result<SaveData> {
    let contents = fs::read_to_string(path).context("read save file")?;
    let data: SaveData = ron::from_str(&contents).context("deserialize save data")?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::gen::player::generate_player_start;
    use crate::rng::SeedRng;
    use crate::sim::SimState;

    #[test]
    fn round_trip_save_load() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let sim = SimState::new(42, charts.clone());
        let mut rng = SeedRng::new(42);
        let player = Some(generate_player_start(&mut rng, &charts));
        let data = SaveData {
            sim,
            player_start: player,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            player_pos: None,
        };
        let path = "/tmp/dw_test_save.ron";
        save_game(&data, path).unwrap();
        let loaded = load_game(path).unwrap();
        assert_eq!(loaded.sim.world.seed, data.sim.world.seed);
        assert_eq!(loaded.sim.world.tick, data.sim.world.tick);
        assert_eq!(loaded.player_start.is_some(), data.player_start.is_some());
    }

    #[test]
    fn save_creates_parent_dirs() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let sim = SimState::new(99, charts);
        let data = SaveData {
            sim,
            player_start: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            player_pos: None,
        };
        let path = "/tmp/dw_test_nested/sub/dir/save.ron";
        save_game(&data, path).unwrap();
        assert!(Path::new(path).exists());
    }

    #[test]
    fn load_nonexistent_fails() {
        assert!(load_game("/tmp/dw_nonexistent.ron").is_err());
    }
}
