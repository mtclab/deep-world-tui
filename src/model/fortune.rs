//! Fortune: the luck a life is born with. A hidden constant in roughly
//! [-1, 1], rolled once from the life-seed and never shown as a number. It
//! tilts every risk a little — the cautious are safer, never safe — and you
//! learn your own luck only by living it, read in omens that hint without
//! proving. The omen reads fate; it does not change it.

use crate::rng::SeedRng;
use serde::{Deserialize, Serialize};

/// How hard fortune leans on a probability. At the extremes (±1) a risk is
/// shifted ±30% of itself — enough to feel, never enough to make caution
/// certain or mischance inevitable.
const TILT: f64 = 0.30;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fortune(f64);

impl Default for Fortune {
    fn default() -> Self {
        Fortune(0.0) // an unrolled life is plain middling luck
    }
}

impl Fortune {
    /// Roll a life's fortune from its seed. The average of three uniforms is
    /// centre-weighted: most lives are middling, the blessed and the cursed
    /// rare. Deterministic — same seed + salt, same star.
    pub fn roll(seed: u64, life_salt: u64) -> Self {
        let mut rng = SeedRng::new(seed.wrapping_add(life_salt)).fork_for("fortune");
        let a = rng.gen_f64();
        let b = rng.gen_f64();
        let c = rng.gen_f64();
        // mean of three uniforms in [0,1) -> [0,1) centred on 0.5; map to [-1,1)
        Fortune(((a + b + c) / 3.0) * 2.0 - 1.0)
    }

    /// A fortune of a known value, clamped to the honest range. For tests and
    /// for any caller that wants a deliberate star rather than a rolled one.
    pub fn from_value(v: f64) -> Self {
        Fortune(v.clamp(-1.0, 1.0))
    }

    /// The raw value, for tests and omen biasing. Not for display.
    pub fn value(self) -> f64 {
        self.0
    }

    /// The multiplier fortune applies to a *bad* outcome's odds — good fortune
    /// below 1, ill fortune above. For callers that compute a probability
    /// internally and want to lean it without re-deriving the tilt.
    pub fn bad_multiplier(self) -> f64 {
        1.0 - self.0 * TILT
    }

    /// Tilt the probability of a *bad* outcome. Good fortune lowers it, ill
    /// fortune raises it; the result stays in [0, 1] and never collapses to a
    /// certainty either way.
    pub fn tilt_bad(self, p: f64) -> f64 {
        (p * self.bad_multiplier()).clamp(0.0, 1.0)
    }

    /// Tilt the probability of a *good* outcome — the mirror of `tilt_bad`.
    pub fn tilt_good(self, p: f64) -> f64 {
        (p * (1.0 + self.0 * TILT)).clamp(0.0, 1.0)
    }

    /// At the edge of a fatal collapse, the chance a blessed life is pulled
    /// back from it. Middling and cursed lives get none — fortune only ever
    /// helps here; the unlucky already die more by meeting more trouble, not
    /// by being killed twice over.
    pub fn death_reprieve_chance(self) -> f64 {
        (self.0.max(0.0)) * 0.5
    }

    /// The chance that, when an omen shows, it shows *fair*. Centred on a coin
    /// toss and leaned by fortune, but never certain in either direction — a
    /// blessed life still sees the odd ill omen, and the cursed the odd fair
    /// one. The omen hints; it does not prove.
    pub fn fair_omen_chance(self) -> f64 {
        (0.5 + self.0 * 0.35).clamp(0.15, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_star() {
        assert_eq!(Fortune::roll(42, 7), Fortune::roll(42, 7));
        assert_ne!(Fortune::roll(42, 7).value(), Fortune::roll(42, 8).value());
    }

    #[test]
    fn fortune_stays_in_range_and_centres() {
        let mut sum = 0.0;
        let mut extremes = 0;
        let n = 5000;
        for s in 0..n {
            let f = Fortune::roll(s as u64, 0).value();
            assert!((-1.0..1.0).contains(&f), "fortune {f} out of range");
            sum += f;
            if f.abs() > 0.7 {
                extremes += 1;
            }
        }
        let mean = sum / n as f64;
        assert!(
            mean.abs() < 0.05,
            "fortune should centre on 0 (mean {mean})"
        );
        // Centre-weighted: the blessed/cursed tails are rare.
        assert!(
            (extremes as f64) < (n as f64) * 0.10,
            "extreme luck should be rare ({extremes}/{n})"
        );
    }

    #[test]
    fn good_fortune_lowers_bad_odds_and_lifts_good() {
        let blessed = Fortune(1.0);
        let cursed = Fortune(-1.0);
        assert!(blessed.tilt_bad(0.20) < 0.20);
        assert!(cursed.tilt_bad(0.20) > 0.20);
        assert!(blessed.tilt_good(0.20) > 0.20);
        assert!(cursed.tilt_good(0.20) < 0.20);
        // never a certainty, never an impossibility
        assert!(cursed.tilt_bad(0.50) < 1.0);
        assert!(blessed.tilt_bad(0.50) > 0.0);
    }

    #[test]
    fn omen_polarity_leans_but_never_locks() {
        let blessed = Fortune(1.0);
        let cursed = Fortune(-1.0);
        assert!(blessed.fair_omen_chance() > cursed.fair_omen_chance());
        // both polarities remain possible under either star
        assert!(blessed.fair_omen_chance() < 1.0 && blessed.fair_omen_chance() > 0.0);
        assert!(cursed.fair_omen_chance() < 1.0 && cursed.fair_omen_chance() > 0.0);
    }
}
