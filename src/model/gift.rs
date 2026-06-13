//! The Gift: the rare innate craft-sensitivity of the Deep World (#426, epic
//! #424). Like a life's Fortune, it is rolled once from the life-seed and
//! hidden — but unlike Fortune, almost no one carries it. Canon: roughly one
//! person in forty has a sensitivity; it shows in childhood or never (there is
//! no late gift); and the craft it grants always costs the body to use (the
//! Conservation Principle — later issues). Here we lay only the gift itself:
//! who carries it, and which sense.

use crate::model::GodName;
use crate::rng::SeedRng;
use serde::{Deserialize, Serialize};

/// The fraction of lives born with any sensitivity at all. The canon ~2.5%:
/// the craftless are the overwhelming, ordinary norm.
const GIFT_RATE: f64 = 0.025;

/// The four craft-senses, each bound to a god and a people's craft. A gifted
/// life has exactly one; the vast majority have none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftSense {
    /// Iron-ear: the Sepät forge-sense, the metal's song. Oltzed's.
    IronEar,
    /// Root-eye: the Metsik forest-sense, the green's deep order. Keuru's.
    RootEye,
    /// Still-sense: the Laakso deep-sense, the quiet under things. Kukri's.
    StillSense,
    /// Scale-hand: the Väylä water-sense, the weight and tide. Masa's.
    ScaleHand,
}

impl CraftSense {
    pub fn name(self) -> &'static str {
        match self {
            CraftSense::IronEar => "iron-ear",
            CraftSense::RootEye => "root-eye",
            CraftSense::StillSense => "still-sense",
            CraftSense::ScaleHand => "scale-hand",
        }
    }

    /// The god whose domain the sense belongs to.
    pub fn god(self) -> GodName {
        match self {
            CraftSense::IronEar => GodName::Oltzed,
            CraftSense::RootEye => GodName::Keuru,
            CraftSense::StillSense => GodName::Kukri,
            CraftSense::ScaleHand => GodName::Masa,
        }
    }

    fn from_index(i: u64) -> CraftSense {
        match i % 4 {
            0 => CraftSense::IronEar,
            1 => CraftSense::RootEye,
            2 => CraftSense::StillSense,
            _ => CraftSense::ScaleHand,
        }
    }
}

/// A life's gift: one craft-sense, or — almost always — none. Hidden; rolled
/// once from the life-seed; innate and unchosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Gift(Option<CraftSense>);

impl Gift {
    /// A craftless life — the ordinary case. Equivalent to the default.
    pub const NONE: Gift = Gift(None);

    /// Roll a life's gift from its seed. About one in forty carries a sense;
    /// the rest carry none. Deterministic — same seed + salt, same gift.
    pub fn roll(seed: u64, life_salt: u64) -> Self {
        let mut rng = SeedRng::new(seed.wrapping_add(life_salt)).fork_for("gift");
        if rng.gen_f64() < GIFT_RATE {
            Gift(Some(CraftSense::from_index(rng.next_u64())))
        } else {
            Gift(None)
        }
    }

    /// A deliberate gift, for tests and for callers that want a known sense.
    pub fn of(sense: CraftSense) -> Self {
        Gift(Some(sense))
    }

    /// The sense carried, if any.
    pub fn sense(self) -> Option<CraftSense> {
        self.0
    }

    /// Whether this life carries any gift at all.
    pub fn has(self) -> bool {
        self.0.is_some()
    }

    /// The gift's name, or "craftless" for the ordinary majority.
    pub fn name(self) -> &'static str {
        match self.0 {
            Some(s) => s.name(),
            None => "craftless",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_gift() {
        assert_eq!(Gift::roll(42, 7), Gift::roll(42, 7));
    }

    #[test]
    fn the_gift_is_rare_and_the_craftless_are_the_norm() {
        let mut gifted = 0;
        let n = 20_000u64;
        for s in 0..n {
            if Gift::roll(s, 0).has() {
                gifted += 1;
            }
        }
        let rate = gifted as f64 / n as f64;
        // Around 2.5%, comfortably between 1.5% and 3.5% — rare, never common.
        assert!(
            (0.015..0.035).contains(&rate),
            "gift rate off: {rate} ({gifted}/{n})"
        );
    }

    #[test]
    fn all_four_senses_appear_and_map_to_a_god() {
        use std::collections::HashSet;
        let mut seen: HashSet<&str> = HashSet::new();
        for s in 0..50_000u64 {
            if let Some(sense) = Gift::roll(s, 0).sense() {
                seen.insert(sense.name());
                let _ = sense.god();
            }
        }
        assert_eq!(seen.len(), 4, "all four senses should occur: {seen:?}");
    }

    #[test]
    fn craftless_is_the_default() {
        assert!(!Gift::default().has());
        assert_eq!(Gift::default().name(), "craftless");
    }
}
