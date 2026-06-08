#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Memorial {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub region_idx: usize,
    pub at_tick: u64,
}

static MEMORIAL_FLAVORS: &[&str] = &[
    "A small cairn marks the spot.",
    "The ground remembers.",
    "I left part of myself here.",
    "A quiet place. I do not speak of it often.",
    "The wind is still here.",
];

impl Memorial {
    pub fn generate(seed: u64, tick: u64, region_idx: usize, x: u32, y: u32) -> Self {
        let mut rng = crate::rng::SeedRng::new(seed).fork_for(&format!("memorial-{tick}-{x}-{y}"));
        let idx = rng.gen_range(MEMORIAL_FLAVORS.len() as u32) as usize;
        Memorial {
            text: MEMORIAL_FLAVORS[idx].into(),
            x,
            y,
            region_idx,
            at_tick: tick,
        }
    }

    pub fn glyph() -> char {
        '⚰'
    }

    pub fn at_position(&self, region_idx: usize, px: u32, py: u32) -> bool {
        self.region_idx == region_idx && self.x == px && self.y == py
    }
}

pub fn pick_recovery_region(seed: u64, current_region: usize, total_regions: usize) -> usize {
    if total_regions <= 1 {
        return 0;
    }
    let mut rng =
        crate::rng::SeedRng::new(seed).fork_for(&format!("recovery-{seed}-{current_region}"));
    let mut r = rng.gen_range(total_regions as u32) as usize;
    while r == current_region {
        r = rng.gen_range(total_regions as u32) as usize;
    }
    r
}

pub fn pick_recovery_god(seed: u64) -> crate::model::GodName {
    let gods = [
        crate::model::GodName::Oltzed,
        crate::model::GodName::Keuru,
        crate::model::GodName::Sampsa,
        crate::model::GodName::Masa,
        crate::model::GodName::Kukri,
    ];
    let mut rng = crate::rng::SeedRng::new(seed).fork_for(&format!("recovery-god-{seed}"));
    let idx = rng.gen_range(gods.len() as u32) as usize;
    gods[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::GodName;

    #[test]
    fn memorial_determinism() {
        let m1 = Memorial::generate(42, 100, 0, 5, 5);
        let m2 = Memorial::generate(42, 100, 0, 5, 5);
        assert_eq!(m1.text, m2.text);
    }

    #[test]
    fn memorial_different_params() {
        let m1 = Memorial::generate(42, 100, 0, 5, 5);
        let m2 = Memorial::generate(42, 200, 0, 5, 5);
        assert_ne!(m1.at_tick, m2.at_tick);
    }

    #[test]
    fn recovery_region_not_current() {
        for seed in 0..20 {
            let r = pick_recovery_region(seed, 1, 5);
            assert_ne!(r, 1);
            assert!(r < 5);
        }
    }

    #[test]
    fn recovery_region_single_region() {
        assert_eq!(pick_recovery_region(7, 0, 1), 0);
    }

    #[test]
    fn recovery_god_valid() {
        let g = pick_recovery_god(99);
        assert!(matches!(
            g,
            GodName::Oltzed | GodName::Keuru | GodName::Sampsa | GodName::Masa | GodName::Kukri
        ));
    }

    #[test]
    fn memorial_glyph() {
        assert_eq!(Memorial::glyph(), '⚰');
    }

    #[test]
    fn position_match() {
        let m = Memorial {
            text: "test".into(),
            x: 3,
            y: 4,
            region_idx: 1,
            at_tick: 0,
        };
        assert!(m.at_position(1, 3, 4));
        assert!(!m.at_position(2, 3, 4));
        assert!(!m.at_position(1, 3, 5));
    }
}
