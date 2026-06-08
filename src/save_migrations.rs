use crate::save::SaveData;

pub const CURRENT_SAVE_VERSION: u32 = 1;

fn migrate_v0_to_v1(data: &mut SaveData) {
    data.collapse_log = std::mem::take(&mut data.collapse_log);
    data.lineage = std::mem::take(&mut data.lineage);
    data.version = 1;
}

pub fn migrate(data: &mut SaveData) -> Result<(), String> {
    if data.version > CURRENT_SAVE_VERSION {
        return Err(format!(
            "Save is from a newer version of Deep World (v{} > v{})",
            data.version, CURRENT_SAVE_VERSION
        ));
    }
    while data.version < CURRENT_SAVE_VERSION {
        match data.version {
            0 => migrate_v0_to_v1(data),
            // Future migrations go here:
            // 1 => migrate_v1_to_v2(data),
            v => return Err(format!("Unknown migration step from v{}", v)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::model::{GameClock, GodAffinity, InterPeopleBias, PeopleKind, PlayerVitals};
    use crate::sim::SimState;

    fn make_save(version: u32) -> SaveData {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        SaveData {
            sim: SimState::new(42, charts),
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
            version,
        }
    }

    #[test]
    fn v0_migrates_to_current() {
        let mut data = make_save(0);
        migrate(&mut data).unwrap();
        assert_eq!(data.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn current_version_is_noop() {
        let mut data = make_save(CURRENT_SAVE_VERSION);
        migrate(&mut data).unwrap();
        assert_eq!(data.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn future_version_returns_error() {
        let mut data = make_save(999);
        let err = migrate(&mut data).unwrap_err();
        assert!(err.contains("newer version"));
    }
}
