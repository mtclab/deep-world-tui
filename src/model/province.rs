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
        let v = self.bond(a, b);
        if v > TIE_THRESHOLD {
            TieKind::Partner
        } else if v < -TIE_THRESHOLD {
            TieKind::Rival
        } else {
            TieKind::Neutral
        }
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
