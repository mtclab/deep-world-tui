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

/// The chance an heir of a gifted parent is gifted too — far above the base
/// rate (the gift runs in the blood), but well short of certain: most often
/// the line still goes quiet.
const HEIR_GIFT_RATE: f64 = 0.35;
/// When a gifted parent's gift does pass, the chance it is the same sense.
const HEIR_SAME_SENSE_RATE: f64 = 0.75;

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

    /// Whether this sense masters a given craft — the work it can do with the
    /// gift (and pay the body for). Iron-ear answers metal; root-eye answers
    /// the green. Still-sense and scale-hand have no craft in the present
    /// recipe set (their gifts will matter elsewhere).
    pub fn aids_craft(self, recipe: &crate::model::economy::CraftRecipe) -> bool {
        use crate::model::economy::ItemType::*;
        match self {
            CraftSense::IronEar => {
                recipe.output == Tool || recipe.inputs.iter().any(|(i, _)| *i == Iron)
            }
            CraftSense::RootEye => {
                matches!(recipe.output, Bandage | Salve)
                    || recipe.inputs.iter().any(|(i, _)| *i == Herb)
            }
            CraftSense::StillSense | CraftSense::ScaleHand => false,
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

    /// Roll an heir's gift, leaned by the parent's (#429). The gift runs in
    /// the blood: a gifted parent gives a far better than even-odds chance of
    /// a gifted child — usually the same sense — but the line can still go
    /// quiet ("the children do not hear"). A craftless line rolls the ordinary
    /// rare chance, so a gift can still arise from nowhere. Deterministic.
    pub fn roll_heir(seed: u64, life_salt: u64, parent: Gift) -> Self {
        let mut rng = SeedRng::new(seed.wrapping_add(life_salt)).fork_for("gift-heir");
        match parent.sense() {
            Some(parent_sense) => {
                if rng.gen_f64() < HEIR_GIFT_RATE {
                    // The gift passes — usually the parent's own sense.
                    if rng.gen_f64() < HEIR_SAME_SENSE_RATE {
                        Gift(Some(parent_sense))
                    } else {
                        Gift(Some(CraftSense::from_index(rng.next_u64())))
                    }
                } else {
                    // The line goes quiet.
                    Gift(None)
                }
            }
            None => {
                // A craftless line: the ordinary rare chance, as any life.
                if rng.gen_f64() < GIFT_RATE {
                    Gift(Some(CraftSense::from_index(rng.next_u64())))
                } else {
                    Gift(None)
                }
            }
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
    fn the_senses_master_their_own_craft() {
        use crate::model::economy::craft_recipes;
        use crate::model::economy::ItemType;
        let r = craft_recipes();
        let tool = r.iter().find(|c| c.output == ItemType::Tool).unwrap();
        let salve = r.iter().find(|c| c.output == ItemType::Salve).unwrap();
        // Iron-ear answers the forge; root-eye answers the green.
        assert!(CraftSense::IronEar.aids_craft(tool));
        assert!(!CraftSense::IronEar.aids_craft(salve));
        assert!(CraftSense::RootEye.aids_craft(salve));
        assert!(!CraftSense::RootEye.aids_craft(tool));
        // The deep and the tide have no craft in the present recipe set.
        assert!(!CraftSense::StillSense.aids_craft(tool));
        assert!(!CraftSense::ScaleHand.aids_craft(salve));
    }

    #[test]
    fn the_gift_runs_in_the_blood_but_the_line_can_go_quiet() {
        let parent = Gift::of(CraftSense::IronEar);
        let n = 20_000u64;
        let (mut gifted, mut same) = (0u64, 0u64);
        for s in 0..n {
            let heir = Gift::roll_heir(1, s, parent);
            if heir.has() {
                gifted += 1;
                if heir.sense() == Some(CraftSense::IronEar) {
                    same += 1;
                }
            }
        }
        let rate = gifted as f64 / n as f64;
        // Far above the base 2.5%, but most lines still go quiet.
        assert!((0.28..0.42).contains(&rate), "heir gift rate off: {rate}");
        assert!(rate < 0.5, "the line must be able to go quiet");
        // When it passes, usually the parent's own sense.
        let same_rate = same as f64 / gifted as f64;
        assert!(
            (0.65..0.85).contains(&same_rate),
            "same-sense rate off: {same_rate}"
        );
    }

    #[test]
    fn a_craftless_line_keeps_the_ordinary_rare_chance() {
        let n = 30_000u64;
        let mut gifted = 0;
        for s in 0..n {
            if Gift::roll_heir(2, s, Gift::NONE).has() {
                gifted += 1;
            }
        }
        let rate = gifted as f64 / n as f64;
        assert!(
            (0.015..0.035).contains(&rate),
            "craftless-line rate off: {rate}"
        );
    }

    #[test]
    fn heir_gift_is_deterministic() {
        let p = Gift::of(CraftSense::RootEye);
        assert_eq!(Gift::roll_heir(5, 9, p), Gift::roll_heir(5, 9, p));
    }

    #[test]
    fn craftless_is_the_default() {
        assert!(!Gift::default().has());
        assert_eq!(Gift::default().name(), "craftless");
    }
}
