use crate::rng::SeedRng;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryKind {
    StandingStone,
    Wreck,
    StrangeTree,
    Cairn,
    Spring,
    BurningTree,
    SleepingBear,
    SunkenVillage,
    ErredMarker,
    FrostShrine,
    WhisperingPool,
    BoneCircle,
}

impl DiscoveryKind {
    pub fn label(self) -> &'static str {
        match self {
            DiscoveryKind::StandingStone => "standing stone",
            DiscoveryKind::Wreck => "wreckage",
            DiscoveryKind::StrangeTree => "strange tree",
            DiscoveryKind::Cairn => "cairn",
            DiscoveryKind::Spring => "spring",
            DiscoveryKind::BurningTree => "burning tree",
            DiscoveryKind::SleepingBear => "sleeping bear",
            DiscoveryKind::SunkenVillage => "sunken village",
            DiscoveryKind::ErredMarker => "erred marker",
            DiscoveryKind::FrostShrine => "frost shrine",
            DiscoveryKind::WhisperingPool => "whispering pool",
            DiscoveryKind::BoneCircle => "bone circle",
        }
    }

    pub fn observe_text(self) -> &'static str {
        match self {
            DiscoveryKind::StandingStone => {
                "An ancient stone stands alone, carved with sigils older than memory."
            }
            DiscoveryKind::Wreck => {
                "Tumbled beams and rusted iron. Someone's road ended here long ago."
            }
            DiscoveryKind::StrangeTree => {
                "A tree twisted beyond reason, its bark etched with patterns no wind made."
            }
            DiscoveryKind::Cairn => {
                "A pile of stones stacked with care. A traveler's marker, or a grave."
            }
            DiscoveryKind::Spring => {
                "Clear water rises from the earth. The ground here remembers rain that never fell."
            }
            DiscoveryKind::BurningTree => {
                "The tree smolders without flame. A pillar of warmth in the cold."
            }
            DiscoveryKind::SleepingBear => {
                "A great shape breathes slow in the undergrowth. Best to pass quietly."
            }
            DiscoveryKind::SunkenVillage => {
                "Rooflines below the waterline. A village that sank, still holding its silence."
            }
            DiscoveryKind::ErredMarker => {
                "A boundary stone, inscribed with a name that no longer belongs to any people."
            }
            DiscoveryKind::FrostShrine => {
                "Ice clings to a small altar. The cold here is reverent, not harsh."
            }
            DiscoveryKind::WhisperingPool => {
                "The pool murmurs. Not wind — something under the surface speaks softly."
            }
            DiscoveryKind::BoneCircle => {
                "Bones arranged with purpose. A warning, a ward, or a covenant."
            }
        }
    }

    pub fn glyph(self) -> char {
        match self {
            DiscoveryKind::StandingStone => '▮',
            DiscoveryKind::Wreck => '✗',
            DiscoveryKind::StrangeTree => '♧',
            DiscoveryKind::Cairn => '▴',
            DiscoveryKind::Spring => '≋',
            DiscoveryKind::BurningTree => '♦',
            DiscoveryKind::SleepingBear => '▣',
            DiscoveryKind::SunkenVillage => '∻',
            DiscoveryKind::ErredMarker => '§',
            DiscoveryKind::FrostShrine => '❄',
            DiscoveryKind::WhisperingPool => '◎',
            DiscoveryKind::BoneCircle => '⊛',
        }
    }

    pub fn all() -> &'static [DiscoveryKind] {
        &[
            DiscoveryKind::StandingStone,
            DiscoveryKind::Wreck,
            DiscoveryKind::StrangeTree,
            DiscoveryKind::Cairn,
            DiscoveryKind::Spring,
            DiscoveryKind::BurningTree,
            DiscoveryKind::SleepingBear,
            DiscoveryKind::SunkenVillage,
            DiscoveryKind::ErredMarker,
            DiscoveryKind::FrostShrine,
            DiscoveryKind::WhisperingPool,
            DiscoveryKind::BoneCircle,
        ]
    }
}

impl fmt::Display for DiscoveryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Discovery {
    pub id: String,
    pub region_idx: usize,
    pub kind: DiscoveryKind,
    pub label: String,
    pub observed: bool,
    pub observed_at_tick: Option<u64>,
    pub witness_person_ids: Vec<String>,
    pub x: u32,
    pub y: u32,
}

impl Discovery {
    pub fn new(id: String, region_idx: usize, kind: DiscoveryKind, x: u32, y: u32) -> Self {
        let label = kind.label().to_string();
        Discovery {
            id,
            region_idx,
            kind,
            label,
            observed: false,
            observed_at_tick: None,
            witness_person_ids: vec![],
            x,
            y,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DiscoveryStore {
    pub entries: Vec<Discovery>,
}

impl DiscoveryStore {
    pub fn new() -> Self {
        DiscoveryStore { entries: vec![] }
    }

    pub fn observe(&mut self, discovery_id: &str, tick: u64, witness_id: String) -> bool {
        if let Some(d) = self.entries.iter_mut().find(|d| d.id == discovery_id) {
            if d.observed {
                return false;
            }
            d.observed = true;
            d.observed_at_tick = Some(tick);
            d.witness_person_ids.push(witness_id);
            return true;
        }
        false
    }

    pub fn is_observed(&self, discovery_id: &str) -> bool {
        self.entries
            .iter()
            .any(|d| d.id == discovery_id && d.observed)
    }

    pub fn for_region(&self, region_idx: usize) -> Vec<&Discovery> {
        self.entries
            .iter()
            .filter(|d| d.region_idx == region_idx)
            .collect()
    }

    pub fn at_position(&self, region_idx: usize, x: u32, y: u32) -> Option<&Discovery> {
        self.entries
            .iter()
            .find(|d| d.region_idx == region_idx && d.x == x && d.y == y)
    }
}

pub fn generate_region_discoveries(
    rng: &mut SeedRng,
    region_idx: usize,
    terrain_width: usize,
    terrain_height: usize,
) -> Vec<Discovery> {
    let count = 1 + rng.gen_range(3) as usize;
    let kinds = DiscoveryKind::all();
    let mut discoveries = Vec::new();
    for i in 0..count {
        let kind = kinds[rng.gen_range(kinds.len() as u32) as usize];
        let x = rng.gen_range(terrain_width.max(1) as u32);
        let y = rng.gen_range(terrain_height.max(1) as u32);
        let id = format!("disc_{}_{}", region_idx, i);
        discoveries.push(Discovery::new(id, region_idx, kind, x, y));
    }
    discoveries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SeedRng;

    #[test]
    fn discovery_kind_roundtrip() {
        for kind in DiscoveryKind::all() {
            let label = kind.label();
            let glyph = kind.glyph();
            assert!(!label.is_empty());
            assert_ne!(glyph, '\0');
        }
    }

    #[test]
    fn discovery_observe_once() {
        let mut store = DiscoveryStore::new();
        let d = Discovery::new("disc_0_0".into(), 0, DiscoveryKind::Cairn, 5, 3);
        store.entries.push(d);
        assert!(!store.is_observed("disc_0_0"));
        let first = store.observe("disc_0_0", 100, "player".into());
        assert!(first);
        assert!(store.is_observed("disc_0_0"));
        let second = store.observe("disc_0_0", 200, "player".into());
        assert!(!second);
        let d = store.entries.iter().find(|d| d.id == "disc_0_0").unwrap();
        assert_eq!(d.observed_at_tick, Some(100));
    }

    #[test]
    fn generate_region_discoveries_deterministic() {
        let mut r1 = SeedRng::new(42);
        let mut r2 = SeedRng::new(42);
        let d1 = generate_region_discoveries(&mut r1, 0, 20, 15);
        let d2 = generate_region_discoveries(&mut r2, 0, 20, 15);
        assert_eq!(d1.len(), d2.len());
        for (a, b) in d1.iter().zip(d2.iter()) {
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
        }
    }

    #[test]
    fn generate_region_discoveries_count_range() {
        let mut rng = SeedRng::new(99);
        for _ in 0..50 {
            let ds = generate_region_discoveries(&mut rng, 0, 30, 20);
            assert!(!ds.is_empty() && ds.len() <= 3);
        }
    }

    #[test]
    fn discovery_store_for_region() {
        let mut store = DiscoveryStore::new();
        store
            .entries
            .push(Discovery::new("d0".into(), 0, DiscoveryKind::Cairn, 1, 1));
        store
            .entries
            .push(Discovery::new("d1".into(), 0, DiscoveryKind::Spring, 2, 3));
        store
            .entries
            .push(Discovery::new("d2".into(), 1, DiscoveryKind::Wreck, 5, 5));
        assert_eq!(store.for_region(0).len(), 2);
        assert_eq!(store.for_region(1).len(), 1);
        assert_eq!(store.for_region(2).len(), 0);
    }
}
