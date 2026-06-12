//! Text banks loaded from data/ RON files (embedded via include_str! so the
//! binary stays self-contained, same pattern as charts.ron). One map per
//! file: bank name → lines. Content edits are data edits, not code edits.

use std::collections::HashMap;
use std::sync::OnceLock;

const JOURNAL_BANKS_RON: &str = include_str!("../data/journal_banks.ron");
const VOICE_BANKS_RON: &str = include_str!("../data/voice_banks.ron");

fn load() -> &'static HashMap<String, Vec<String>> {
    static BANKS: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    BANKS.get_or_init(|| {
        let mut all: HashMap<String, Vec<String>> =
            ron::from_str(JOURNAL_BANKS_RON).expect("data/journal_banks.ron parses");
        let voice: HashMap<String, Vec<String>> =
            ron::from_str(VOICE_BANKS_RON).expect("data/voice_banks.ron parses");
        for (k, v) in voice {
            assert!(all.insert(k.clone(), v).is_none(), "duplicate bank {k}");
        }
        for (k, v) in &all {
            assert!(!v.is_empty(), "bank {k} is empty");
        }
        all
    })
}

/// A named line bank. Panics on a missing name: every call site names a bank
/// that ships in data/, and the presence test sweeps them all.
pub fn bank(name: &str) -> &'static [String] {
    load()
        .get(name)
        .unwrap_or_else(|| panic!("unknown text bank {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_banks_parse_and_are_nonempty() {
        let all = load();
        assert!(!all.is_empty());
        for (k, v) in all {
            assert!(!v.is_empty(), "bank {k} empty");
            for line in v {
                assert!(!line.trim().is_empty(), "blank line in {k}");
            }
        }
    }
}
