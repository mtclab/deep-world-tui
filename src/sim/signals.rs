use serde::{Deserialize, Serialize};

const REP_MIN: f64 = 0.0;
const REP_MAX: f64 = 1.0;

const HOSTILE_MAX: f64 = 0.20;
const COLD_MAX: f64 = 0.35;
const NEUTRAL_MAX: f64 = 0.55;
const WARM_MAX: f64 = 0.75;

const PRICE_FLOOR: f64 = 0.6;
const PRICE_CEIL: f64 = 1.5;
const PRICE_SLOPE: f64 = 1.0;
const PRICE_INTERCEPT: f64 = 1.5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReputationBand {
    Hostile,
    Cold,
    Neutral,
    Warm,
    Revered,
}

impl ReputationBand {
    pub fn label(self) -> &'static str {
        match self {
            ReputationBand::Hostile => "hostile",
            ReputationBand::Cold => "cold",
            ReputationBand::Neutral => "neutral",
            ReputationBand::Warm => "warm",
            ReputationBand::Revered => "revered",
        }
    }
}

pub fn band_for(rep: f64) -> ReputationBand {
    let r = rep.clamp(REP_MIN, REP_MAX);
    if r < HOSTILE_MAX {
        ReputationBand::Hostile
    } else if r < COLD_MAX {
        ReputationBand::Cold
    } else if r < NEUTRAL_MAX {
        ReputationBand::Neutral
    } else if r < WARM_MAX {
        ReputationBand::Warm
    } else {
        ReputationBand::Revered
    }
}

const BODY_LANGUAGE: &[(ReputationBand, &[&str])] = &[
    (
        ReputationBand::Hostile,
        &[
            "scowls and turns her shoulder",
            "won't meet your eye",
            "backs toward the door",
            "narrows her gaze as you approach",
            "spits to the side at your shadow",
        ],
    ),
    (
        ReputationBand::Cold,
        &[
            "measures you in silence",
            "gives a curt nod",
            "eyes you with no warmth",
            "stands stiff, hand near the latch",
        ],
    ),
    (
        ReputationBand::Neutral,
        &[
            "nods as you pass",
            "watches you from the doorway",
            "looks up briefly from her work",
            "sets aside what she was mending",
        ],
    ),
    (
        ReputationBand::Warm,
        &[
            "greets you by the road",
            "gestures to the stool by the fire",
            "smiles and sets out a cup",
            "calls you by name as you enter",
        ],
    ),
    (
        ReputationBand::Revered,
        &[
            "rises when you appear",
            "beckons you closer with both hands",
            "ushers you in ahead of the line",
            "leaves her work to attend you first",
        ],
    ),
];

pub fn body_language(rep: f64, npc_id_hash: u32) -> &'static str {
    let band = band_for(rep);
    let bank = BODY_LANGUAGE
        .iter()
        .find(|(b, _)| *b == band)
        .map(|(_, v)| *v)
        .unwrap_or(&["nods"]);
    bank[(npc_id_hash as usize) % bank.len()]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EngagementLevel {
    Refuses,
    Reluctant,
    Neutral,
    Willing,
    Eager,
}

pub fn engagement_for(rep: f64) -> EngagementLevel {
    let r = rep.clamp(REP_MIN, REP_MAX);
    if r < 0.18 {
        EngagementLevel::Refuses
    } else if r < 0.32 {
        EngagementLevel::Reluctant
    } else if r < 0.62 {
        EngagementLevel::Neutral
    } else if r < 0.82 {
        EngagementLevel::Willing
    } else {
        EngagementLevel::Eager
    }
}

pub fn engagement_narration(level: EngagementLevel, npc_name: &str) -> String {
    match level {
        EngagementLevel::Refuses => format!(
            "{} does not look up. Whatever you came for, the answer is no.",
            npc_name
        ),
        EngagementLevel::Reluctant => format!(
            "{} pauses a long moment, then nods once. You may speak, but briefly.",
            npc_name
        ),
        EngagementLevel::Neutral => format!("{} turns toward you and waits.", npc_name),
        EngagementLevel::Willing => format!(
            "{} greets you and steps closer. How can she help?",
            npc_name
        ),
        EngagementLevel::Eager => format!(
            "{} rises at the sound of your step, and comes to meet you.",
            npc_name
        ),
    }
}

pub fn reputation_price_modifier(rep: f64) -> f64 {
    let r = rep.clamp(REP_MIN, REP_MAX);
    (PRICE_INTERCEPT - PRICE_SLOPE * r).clamp(PRICE_FLOOR, PRICE_CEIL)
}

const SETTLEMENT_WELCOME: &[(ReputationBand, &str)] = &[
    (ReputationBand::Hostile, "unwelcome"),
    (ReputationBand::Cold, "watchfully"),
    (ReputationBand::Neutral, "evenly"),
    (ReputationBand::Warm, "graciously"),
    (ReputationBand::Revered, "warmly"),
];

pub fn settlement_welcome_adverb(rep: f64) -> &'static str {
    let band = band_for(rep);
    SETTLEMENT_WELCOME
        .iter()
        .find(|(b, _)| *b == band)
        .map(|(_, w)| *w)
        .unwrap_or("evenly")
}

const SETTLEMENT_NOTE: &[(ReputationBand, &str)] = &[
    (
        ReputationBand::Hostile,
        "The doors shutter as you pass. The headwoman will not see you.",
    ),
    (
        ReputationBand::Cold,
        "Eyes follow you across the square, but no one calls you over.",
    ),
    (
        ReputationBand::Neutral,
        "A few nod. The well-worn paths accept another pair of feet.",
    ),
    (
        ReputationBand::Warm,
        "The headwoman nods from across the square. Neighbors offer you a stool.",
    ),
    (
        ReputationBand::Revered,
        "The headwoman crosses the square to greet you. Children trail behind to listen.",
    ),
];

pub fn settlement_welcome_note(rep: f64) -> &'static str {
    let band = band_for(rep);
    SETTLEMENT_NOTE
        .iter()
        .find(|(b, _)| *b == band)
        .map(|(_, w)| *w)
        .unwrap_or("")
}

pub fn rep_allows_trade(rep: f64) -> bool {
    let band = band_for(rep);
    !matches!(band, ReputationBand::Hostile)
}

pub fn rep_allows_service(rep: f64) -> bool {
    let r = rep.clamp(REP_MIN, REP_MAX);
    r >= 0.18
}

pub fn hash_str(s: &str) -> u32 {
    let mut h: u64 = 1469598103934665603;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    (h as u32) ^ ((h >> 32) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_boundaries_strict() {
        assert_eq!(band_for(0.0), ReputationBand::Hostile);
        assert_eq!(band_for(0.199), ReputationBand::Hostile);
        assert_eq!(band_for(0.20), ReputationBand::Cold);
        assert_eq!(band_for(0.349), ReputationBand::Cold);
        assert_eq!(band_for(0.35), ReputationBand::Neutral);
        assert_eq!(band_for(0.549), ReputationBand::Neutral);
        assert_eq!(band_for(0.55), ReputationBand::Warm);
        assert_eq!(band_for(0.749), ReputationBand::Warm);
        assert_eq!(band_for(0.75), ReputationBand::Revered);
        assert_eq!(band_for(1.0), ReputationBand::Revered);
    }

    #[test]
    fn band_clamped_outside_range() {
        assert_eq!(band_for(-0.5), ReputationBand::Hostile);
        assert_eq!(band_for(2.0), ReputationBand::Revered);
    }

    #[test]
    fn price_modifier_anchors() {
        assert!((reputation_price_modifier(0.25) - 1.25).abs() < 1e-9);
        assert!((reputation_price_modifier(0.75) - 0.75).abs() < 1e-9);
        assert!((reputation_price_modifier(0.5) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn price_modifier_in_design_range() {
        for i in 0..=10 {
            let rep = i as f64 / 10.0;
            let m = reputation_price_modifier(rep);
            assert!(
                (PRICE_FLOOR..=PRICE_CEIL).contains(&m),
                "rep {rep} -> modifier {m} out of [{PRICE_FLOOR}, {PRICE_CEIL}]"
            );
        }
    }

    #[test]
    fn price_modifier_clamped_at_extremes() {
        assert!((reputation_price_modifier(0.0) - PRICE_CEIL).abs() < 1e-9);
        assert!((reputation_price_modifier(1.0) - PRICE_FLOOR).abs() < 1e-9);
    }

    #[test]
    fn body_language_deterministic() {
        let a = body_language(0.10, 7);
        let b = body_language(0.10, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn body_language_distinct_per_band() {
        let h = body_language(0.05, 1);
        let c = body_language(0.27, 1);
        let n = body_language(0.45, 1);
        let w = body_language(0.65, 1);
        let r = body_language(0.90, 1);
        let all: Vec<&str> = vec![h, c, n, w, r];
        let unique: std::collections::HashSet<&str> = all.iter().copied().collect();
        assert_eq!(
            unique.len(),
            5,
            "bands must give distinct lines (got {all:?})"
        );
    }

    #[test]
    fn body_language_stable_across_clamps() {
        let a = body_language(-0.5, 3);
        let b = body_language(0.0, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn engagement_thresholds_strict() {
        assert_eq!(engagement_for(0.0), EngagementLevel::Refuses);
        assert_eq!(engagement_for(0.179), EngagementLevel::Refuses);
        assert_eq!(engagement_for(0.18), EngagementLevel::Reluctant);
        assert_eq!(engagement_for(0.32), EngagementLevel::Neutral);
        assert_eq!(engagement_for(0.62), EngagementLevel::Willing);
        assert_eq!(engagement_for(0.82), EngagementLevel::Eager);
        assert_eq!(engagement_for(1.0), EngagementLevel::Eager);
    }

    #[test]
    fn engagement_narration_never_prints_level_name() {
        let names = ["Aila", "Mikko", "Tova", "Eetu"];
        for level in [
            EngagementLevel::Refuses,
            EngagementLevel::Reluctant,
            EngagementLevel::Neutral,
            EngagementLevel::Willing,
            EngagementLevel::Eager,
        ] {
            for n in names {
                let s = engagement_narration(level, n);
                assert!(s.contains(n));
                for forbidden in [
                    "Refuses",
                    "Reluctant",
                    "Neutral",
                    "Willing",
                    "Eager",
                    "EngagementLevel",
                ] {
                    assert!(
                        !s.contains(forbidden),
                        "narration leaked level name {forbidden:?} in {s:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn settlement_welcome_adverb_covers_bands() {
        let ads = [
            settlement_welcome_adverb(0.05),
            settlement_welcome_adverb(0.27),
            settlement_welcome_adverb(0.45),
            settlement_welcome_adverb(0.65),
            settlement_welcome_adverb(0.90),
        ];
        let set: std::collections::HashSet<&str> = ads.iter().copied().collect();
        assert_eq!(set.len(), 5, "each band should have a distinct adverb");
    }

    #[test]
    fn trade_and_service_gates_consistent() {
        assert!(!rep_allows_trade(0.10));
        assert!(rep_allows_trade(0.30));
        assert!(!rep_allows_service(0.10));
        assert!(rep_allows_service(0.30));
    }

    #[test]
    fn hash_str_stable() {
        assert_eq!(hash_str("Aila"), hash_str("Aila"));
        assert_ne!(hash_str("Aila"), hash_str("Bail"));
    }

    #[test]
    fn body_language_distinct_per_band_extra() {
        let mut seen: std::collections::HashSet<&'static str> = Default::default();
        seen.insert(body_language(0.05, 1));
        seen.insert(body_language(0.25, 1));
        seen.insert(body_language(0.45, 1));
        seen.insert(body_language(0.65, 1));
        seen.insert(body_language(0.95, 1));
        assert!(seen.len() >= 4, "most bands should sound different");
    }

    #[test]
    fn engagement_narration_never_prints_level_names() {
        for level in [
            EngagementLevel::Refuses,
            EngagementLevel::Reluctant,
            EngagementLevel::Neutral,
            EngagementLevel::Willing,
            EngagementLevel::Eager,
        ] {
            for name in ["Aila", "Bail", "Sigr"] {
                let s = engagement_narration(level, name);
                for leak in [
                    "Refuses",
                    "Reluctant",
                    "Neutral",
                    "Willing",
                    "Eager",
                    "level=",
                    "engagement",
                    "reputation",
                ] {
                    assert!(
                        !s.contains(leak),
                        "narration leaked design token '{leak}': {s}"
                    );
                }
            }
        }
    }

    #[test]
    fn settlement_welcome_strings_never_print_reputation() {
        for rep in [0.0, 0.10, 0.20, 0.30, 0.45, 0.50, 0.55, 0.70, 0.80, 1.0] {
            let adv = settlement_welcome_adverb(rep);
            let note = settlement_welcome_note(rep);
            for s in [adv.to_string(), note.to_string()] {
                for leak in [
                    "reputation",
                    "trusted",
                    "liked",
                    "suspect",
                    "shunned",
                    "repel",
                    "morale",
                    "Hostile",
                    "Cold",
                    "Neutral",
                    "Warm",
                    "Revered",
                ] {
                    assert!(!s.contains(leak), "welcome string leaked '{leak}': {s}");
                }
            }
        }
    }

    #[test]
    fn body_language_never_prints_reputation() {
        for rep in [0.0, 0.1, 0.3, 0.5, 0.7, 0.9, 1.0] {
            for hash in [0u32, 1, 42, 9999, u32::MAX] {
                let s = body_language(rep, hash);
                for leak in [
                    "reputation",
                    "trusted",
                    "liked",
                    "suspect",
                    "shunned",
                    "Hostile",
                    "Revered",
                    "level",
                    "pct",
                    "%",
                ] {
                    assert!(!s.contains(leak), "body language leaked '{leak}': {s}");
                }
            }
        }
    }
}
