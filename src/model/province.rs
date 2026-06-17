//! A living province (#560): settlements relate to one another, not only within
//! their own walls. Towns that trade much grow into partners; towns that
//! compete for the same scarce goods grow into rivals. The ties form and fade in
//! the daily sim, conserving nothing but bounded, and they shape the caravans,
//! the prices, and the talk — whether the player is there or not.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How two settlements stand with each other, read off the signed bond strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TieKind {
    /// Busy roads between them, easier trade.
    Partner,
    /// Bad blood — carts do not cross.
    Rival,
    /// No particular standing either way.
    Neutral,
}

impl TieKind {
    pub fn label(self) -> &'static str {
        match self {
            TieKind::Partner => "partners",
            TieKind::Rival => "rivals",
            TieKind::Neutral => "neither close nor at odds",
        }
    }
}

/// The web of standings among the province's settlements. A single signed
/// strength per unordered pair: positive is partnership, negative is rivalry.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProvinceTies {
    /// Keyed by the pair of settlement names, always stored low-name-first so
    /// the relation is symmetric. Value in [-1.0, 1.0].
    pub bonds: HashMap<(String, String), f64>,
    /// The last standing the world announced for a pair (#560 slice 4), so a
    /// forming partnership or hardening rivalry is talked of once, not every
    /// day. A pair that lapses back to neutral is cleared, free to be announced
    /// afresh if it forms again.
    #[serde(default)]
    pub announced: HashMap<(String, String), i8>,
}

/// Above this a pair reads as partners; below its negation, as rivals.
const TIE_THRESHOLD: f64 = 0.3;

impl ProvinceTies {
    /// Order a pair so (A,B) and (B,A) name the same bond.
    fn key(a: &str, b: &str) -> (String, String) {
        if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }

    /// The signed strength of the bond between two towns (0.0 if none yet).
    pub fn bond(&self, a: &str, b: &str) -> f64 {
        if a == b {
            return 0.0;
        }
        self.bonds.get(&Self::key(a, b)).copied().unwrap_or(0.0)
    }

    /// Move a pair's bond by `delta`, clamped to [-1.0, 1.0]. A bond that lands
    /// back near zero is dropped so the map does not grow without bound.
    pub fn nudge(&mut self, a: &str, b: &str, delta: f64) {
        if a == b {
            return;
        }
        let k = Self::key(a, b);
        let v = (self.bonds.get(&k).copied().unwrap_or(0.0) + delta).clamp(-1.0, 1.0);
        if v.abs() < 0.01 {
            self.bonds.remove(&k);
        } else {
            self.bonds.insert(k, v);
        }
    }

    /// Fade every bond toward zero — a partnership or rivalry not kept up by
    /// fresh trade or fresh friction slowly lapses to neutral.
    pub fn decay(&mut self, rate: f64) {
        self.bonds.retain(|_, v| {
            *v *= 1.0 - rate;
            v.abs() >= 0.01
        });
    }

    /// How two towns stand with each other right now.
    pub fn tie(&self, a: &str, b: &str) -> TieKind {
        Self::tie_of(self.bond(a, b))
    }

    fn tie_of(v: f64) -> TieKind {
        if v > TIE_THRESHOLD {
            TieKind::Partner
        } else if v < -TIE_THRESHOLD {
            TieKind::Rival
        } else {
            TieKind::Neutral
        }
    }

    /// The town the given settlement holds its strongest standing with, partner
    /// or rival — what an NPC there would name first if asked how their town
    /// stands with the province (#560 slice 4). `None` for a town with no real
    /// tie either way.
    pub fn strongest_tie(&self, town: &str) -> Option<(String, TieKind)> {
        self.bonds
            .iter()
            .filter(|((a, b), _)| a == town || b == town)
            .filter(|(_, v)| v.abs() > TIE_THRESHOLD)
            .max_by(|x, y| x.1.abs().partial_cmp(&y.1.abs()).unwrap())
            .map(|((a, b), v)| {
                let other = if a == town { b } else { a };
                (other.clone(), Self::tie_of(*v))
            })
    }

    /// Pairs whose standing has newly crossed into partnership or rivalry since
    /// the last call (#560 slice 4) — the world's fresh trade-news. Records each
    /// so it is reported once; a pair that has lapsed back to neutral is
    /// forgotten so it can be announced again if it re-forms.
    pub fn newly_crossed(&mut self) -> Vec<(String, String, TieKind)> {
        let mut news = Vec::new();
        for (pair, v) in &self.bonds {
            let code = match Self::tie_of(*v) {
                TieKind::Partner => 1,
                TieKind::Rival => -1,
                TieKind::Neutral => 0,
            };
            if code == 0 {
                continue;
            }
            if self.announced.get(pair).copied() != Some(code) {
                let kind = if code == 1 {
                    TieKind::Partner
                } else {
                    TieKind::Rival
                };
                news.push((pair.0.clone(), pair.1.clone(), kind));
            }
        }
        // Update the record: drop lapsed pairs, set the rest to their current
        // standing.
        self.announced
            .retain(|pair, _| self.bonds.get(pair).map(|v| v.abs()).unwrap_or(0.0) > TIE_THRESHOLD);
        for (a, b, kind) in &news {
            let code = if *kind == TieKind::Partner { 1 } else { -1 };
            self.announced.insert((a.clone(), b.clone()), code);
        }
        news
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bonds_are_symmetric() {
        let mut t = ProvinceTies::default();
        t.nudge("Brookhollow", "Stonefen", 0.4);
        assert!((t.bond("Stonefen", "Brookhollow") - 0.4).abs() < 1e-9);
        assert_eq!(t.tie("Stonefen", "Brookhollow"), TieKind::Partner);
    }

    #[test]
    fn rivalry_reads_negative() {
        let mut t = ProvinceTies::default();
        t.nudge("A-town", "B-town", -0.5);
        assert_eq!(t.tie("A-town", "B-town"), TieKind::Rival);
    }

    #[test]
    fn small_bonds_are_neutral_and_dropped() {
        let mut t = ProvinceTies::default();
        t.nudge("A", "B", 0.2);
        assert_eq!(t.tie("A", "B"), TieKind::Neutral);
        // Decay it away; the entry is removed once it lapses near zero.
        for _ in 0..200 {
            t.decay(0.1);
        }
        assert!(t.bonds.is_empty(), "lapsed bond dropped");
    }

    #[test]
    fn a_town_has_no_bond_with_itself() {
        let mut t = ProvinceTies::default();
        t.nudge("Self", "Self", 0.9);
        assert_eq!(t.bond("Self", "Self"), 0.0);
        assert!(t.bonds.is_empty());
    }

    #[test]
    fn crossings_announce_once_then_clear_on_lapse() {
        let mut t = ProvinceTies::default();
        t.nudge("A", "B", 0.5);
        let first = t.newly_crossed();
        assert_eq!(first.len(), 1, "the new partnership is announced");
        assert_eq!(first[0].2, TieKind::Partner);
        // Not announced again while it holds.
        assert!(t.newly_crossed().is_empty(), "no repeat while it stands");
        // Lapse it to neutral; the record clears.
        for _ in 0..400 {
            t.decay(0.05);
        }
        assert!(
            t.newly_crossed().is_empty(),
            "nothing to announce once neutral"
        );
        // Re-form it — announced afresh.
        t.nudge("A", "B", 0.5);
        assert_eq!(t.newly_crossed().len(), 1, "a re-formed tie is news again");
    }

    #[test]
    fn strongest_tie_names_the_sharpest_standing() {
        let mut t = ProvinceTies::default();
        t.nudge("Home", "Mild", 0.4);
        t.nudge("Home", "Bitter", -0.7);
        let (other, kind) = t.strongest_tie("Home").expect("a tie");
        assert_eq!(other, "Bitter");
        assert_eq!(kind, TieKind::Rival);
        assert!(t.strongest_tie("Nowhere").is_none());
    }

    #[test]
    fn bonds_are_clamped() {
        let mut t = ProvinceTies::default();
        for _ in 0..100 {
            t.nudge("X", "Y", 0.5);
        }
        assert!(t.bond("X", "Y") <= 1.0);
        for _ in 0..100 {
            t.nudge("X", "Y", -0.5);
        }
        assert!(t.bond("X", "Y") >= -1.0);
    }
}
