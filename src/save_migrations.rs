use crate::save::SaveData;

pub const CURRENT_SAVE_VERSION: u32 = 2;

fn migrate_v0_to_v1(data: &mut SaveData) {
    data.collapse_log = std::mem::take(&mut data.collapse_log);
    data.lineage = std::mem::take(&mut data.lineage);
    data.version = 1;
}

fn migrate_v1_to_v2(data: &mut SaveData) {
    data.milestones = crate::sim::milestones::MilestoneTracker::new();
    data.version = 2;
}

#[allow(dead_code)]
fn migrate_v2_to_v3(data: &mut SaveData) {
    // Future: add new fields here when v3 format is defined.
    // All new fields must use #[serde(default)] in SaveData.
    data.version = 3;
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
            1 => migrate_v1_to_v2(data),
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

    #[allow(deprecated)]
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
            milestones: crate::sim::milestones::MilestoneTracker::new(),
            explored: Vec::new(),
            version,
            first_run: true,
            hint_tracker: crate::sim::hints::HintTracker::default(),
        }
    }

    #[test]
    fn v0_migrates_to_current() {
        let mut data = make_save(0);
        migrate(&mut data).unwrap();
        assert_eq!(data.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn v1_migrates_to_current() {
        let mut data = make_save(1);
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

    #[test]
    fn v2_roundtrip_through_ron() {
        let data = make_save(CURRENT_SAVE_VERSION);
        let ron_str = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()).unwrap();
        let mut loaded: SaveData = ron::from_str(&ron_str).unwrap();
        assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
        migrate(&mut loaded).unwrap();
        assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn v2_fixture_loads_and_migrates() {
        #[allow(deprecated)]
        let data = make_save(2);
        let ron_str = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()).unwrap();
        let mut loaded: SaveData = ron::from_str(&ron_str).unwrap();
        migrate(&mut loaded).unwrap();
        assert_eq!(loaded.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn migration_preserves_data() {
        let mut data = make_save(0);
        data.encounters_had = 7;
        data.collapses_had = 3;
        migrate(&mut data).unwrap();
        assert_eq!(data.version, CURRENT_SAVE_VERSION);
        assert_eq!(data.encounters_had, 7);
        assert_eq!(data.collapses_had, 3);
    }
}
