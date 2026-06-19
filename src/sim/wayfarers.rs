//! Wandering folk on the road (#649 slice 2): the travellers, bards, pilgrims,
//! and hermits that the old encounter system rolled into a popup are now actors
//! on the grid — someone you see ahead on the road and walk up to for a word.
//! No random roll, no menu screen: what you see is who you can meet. Restocked
//! per region from the daily sim like the wild beasts, deterministic, and they
//! move on after a few days so the roads are never crowded with the same faces.

use serde::{Deserialize, Serialize};

use crate::model::Terrain;
use crate::rng::SeedRng;

/// Who is on the road — a wanderer, met for the word they carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WayfarerKind {
    /// A fellow traveller, full of road-news.
    Traveler,
    /// A wandering bard — a song lifts the heart on a long road.
    Bard,
    /// A pilgrim bound for a far shrine, their talk turned to the gods.
    Pilgrim,
    /// A hermit at the edge of the settled land, sparing with words.
    Hermit,
}

impl WayfarerKind {
    /// A short word for the meeting, for the status line.
    pub fn label(self) -> &'static str {
        match self {
            WayfarerKind::Traveler => "a traveller",
            WayfarerKind::Bard => "a wandering bard",
            WayfarerKind::Pilgrim => "a pilgrim",
            WayfarerKind::Hermit => "a hermit",
        }
    }
}

/// A single wanderer standing somewhere on a region's road (#649).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wayfarer {
    pub id: String,
    pub kind: WayfarerKind,
    pub region_idx: usize,
    pub px: usize,
    pub py: usize,
    /// The day they appeared, for moving them on.
    pub spawned_day: u32,
}

/// How long a wanderer lingers on a region's roads before moving on.
const WAYFARER_DAYS: u32 = 3;

/// How many wanderers a region should hold: the settled, road-laced country
/// carries travellers; the empty wild rarely does. At most one or two — the
/// road is travelled, not thronged.
fn target_wayfarers(has_road: bool, settlements: usize) -> usize {
    if !has_road && settlements == 0 {
        0
    } else if settlements >= 2 {
        2
    } else {
        1
    }
}

/// The daily turn for the road's wanderers (#649 slice 2): move on those who
/// have lingered their days, then restock each region toward its target,
/// spawning a terrain-true wanderer on an open road (or, failing a road, open
/// grass). Deterministic.
pub fn tick_wayfarers(sim: &mut crate::sim::SimState) {
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = (tick / 24) as u32;

    // The road moves them on after a few days.
    sim.wayfarers
        .retain(|w| day.saturating_sub(w.spawned_day) < WAYFARER_DAYS);

    let seed = sim.world.seed;
    let region_count = sim.world.regions.len();
    for ri in 0..region_count {
        let (has_road, settlements) = {
            let r = &sim.world.regions[ri];
            let has_road = r.terrain.tiles.iter().any(|t| matches!(t, Terrain::Road));
            (
                has_road,
                r.settlements.iter().filter(|s| s.population > 0).count(),
            )
        };
        let target = target_wayfarers(has_road, settlements);
        let here = sim.wayfarers.iter().filter(|w| w.region_idx == ri).count();
        if here >= target {
            continue;
        }
        let n = sim.wayfarers.len();
        let mut rng = SeedRng::new(seed).fork_for(&format!("wayfarer-{ri}-{day}-{n}"));
        let kind = match rng.next_u64() % 4 {
            0 => WayfarerKind::Traveler,
            1 => WayfarerKind::Bard,
            2 => WayfarerKind::Pilgrim,
            _ => WayfarerKind::Hermit,
        };
        let Some((px, py)) = road_tile(sim, ri, &mut rng) else {
            continue;
        };
        sim.wayfarers.push(Wayfarer {
            id: format!("wayfarer-{seed}-{day}-{n}"),
            kind,
            region_idx: ri,
            px,
            py,
            spawned_day: day,
        });
    }
}

/// An open road tile in a region (grass if the region has no road laid), clear
/// of any wanderer already standing there. `None` if there is no open footing.
fn road_tile(
    sim: &crate::sim::SimState,
    region_idx: usize,
    rng: &mut SeedRng,
) -> Option<(usize, usize)> {
    let terr = &sim.world.regions.get(region_idx)?.terrain;
    let mut roads: Vec<(usize, usize)> = Vec::new();
    let mut grass: Vec<(usize, usize)> = Vec::new();
    for y in 0..terr.height {
        for x in 0..terr.width {
            let occupied = sim
                .wayfarers
                .iter()
                .any(|w| w.region_idx == region_idx && w.px == x && w.py == y);
            if occupied {
                continue;
            }
            match terr.get(x, y) {
                Some(Terrain::Road) => roads.push((x, y)),
                Some(Terrain::Grass) => grass.push((x, y)),
                _ => {}
            }
        }
    }
    let pool = if !roads.is_empty() { &roads } else { &grass };
    if pool.is_empty() {
        return None;
    }
    Some(pool[(rng.next_u64() % pool.len() as u64) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::SimState;

    fn make_sim() -> SimState {
        SimState::new(7, crate::charts::load_charts().unwrap())
    }

    #[test]
    fn the_settled_roads_carry_wanderers_and_each_stands_on_their_own_tile() {
        let mut sim = make_sim();
        // Run a season of days so the roads stock their wanderers.
        for d in 1..=20u64 {
            sim.world.tick = d * 24;
            tick_wayfarers(&mut sim);
        }
        assert!(
            !sim.wayfarers.is_empty(),
            "the settled, road-laced province carries travellers"
        );
        // No two wanderers share a tile.
        let mut seen = std::collections::HashSet::new();
        for w in &sim.wayfarers {
            assert!(
                seen.insert((w.region_idx, w.px, w.py)),
                "each wanderer stands on their own tile"
            );
        }
    }

    #[test]
    fn wanderers_move_on_after_a_few_days() {
        let mut sim = make_sim();
        sim.world.tick = 24;
        tick_wayfarers(&mut sim);
        let first: Vec<String> = sim.wayfarers.iter().map(|w| w.id.clone()).collect();
        assert!(!first.is_empty(), "a wanderer appeared on day 1");
        // Days later, the first wanderers have moved on (their ids are gone).
        for d in 2..=8u64 {
            sim.world.tick = d * 24;
            tick_wayfarers(&mut sim);
        }
        for id in &first {
            assert!(
                !sim.wayfarers.iter().any(|w| &w.id == id),
                "the wanderer {id} moved on after their days"
            );
        }
    }
}
