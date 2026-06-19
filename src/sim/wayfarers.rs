//! Wandering folk on the road (#649 slice 2): the travellers, bards, pilgrims,
//! and hermits that the old encounter system rolled into a popup are now actors
//! on the grid — someone you see ahead on the road and walk up to for a word.
//! No random roll, no menu screen: what you see is who you can meet. Restocked
//! per region from the daily sim like the wild beasts, deterministic, and they
//! move on after a few days so the roads are never crowded with the same faces.

use serde::{Deserialize, Serialize};

use crate::model::{PeopleKind, Terrain};
use crate::rng::SeedRng;

/// Who is on the road — a wanderer met for the word they carry, or a trader of
/// one of the peoples (a Khör, a Mëräk, a Häl) down from their own country to
/// barter.
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
    /// A trader of one of the peoples (Khör, Mëräk, Häl), met on the road to
    /// barter in kind — they take no coin. Named by their own people, never an
    /// umbrella term.
    Trader(PeopleKind),
    /// A child lost from a nearby town — walk them back toward the hearth.
    LostChild,
    /// Someone caught out in the killing cold — a little food may keep them.
    WinterSurvivor,
    /// A town's funeral on the road — stand aside and pay your respects.
    FuneralProcession,
    /// A farmer's beasts broken loose — a hand turning them costs you nothing.
    EscapedLivestock,
    /// A keeper at the edge of the wild marches who asks a price for the road
    /// past — bread or herb, your choosing. A strange hermit, surely (#455).
    ThresholdKeeper,
}

impl WayfarerKind {
    /// A short word for the meeting, for the status line. Traders are named by
    /// their own people at the call site (via `trader_people`), not here.
    pub fn label(self) -> &'static str {
        match self {
            WayfarerKind::Traveler => "a traveller",
            WayfarerKind::Bard => "a wandering bard",
            WayfarerKind::Pilgrim => "a pilgrim",
            WayfarerKind::Hermit => "a hermit",
            WayfarerKind::Trader(_) => "a trader",
            WayfarerKind::LostChild => "a lost child",
            WayfarerKind::WinterSurvivor => "someone caught in the cold",
            WayfarerKind::FuneralProcession => "a funeral procession",
            WayfarerKind::EscapedLivestock => "strayed livestock",
            WayfarerKind::ThresholdKeeper => "a keeper at the threshold",
        }
    }

    /// The people this wayfarer trades for, if they are a trader.
    pub fn trader_people(self) -> Option<PeopleKind> {
        match self {
            WayfarerKind::Trader(pk) => Some(pk),
            _ => None,
        }
    }
}

/// Which people, if any, sends a trader to the roads of a region of this kind
/// (#649 slice 2b): the canon home-terrains — the Mëräk from the coasts and
/// deltas, the Khör from the open and high country, the Häl from the deep woods.
/// The deep-cave Tzäkhar and deep-desert She'ar keep to their own country and
/// are met in their enclaves, not on the province's roads.
fn road_trader_people(region_type: &str) -> Option<PeopleKind> {
    match region_type {
        "coast" | "delta" => Some(PeopleKind::Merak),
        "steppe" | "upland" => Some(PeopleKind::Khor),
        "forest" | "river_valley" => Some(PeopleKind::Hal),
        _ => None,
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
fn target_wayfarers(has_road: bool, settlements: usize, is_march: bool) -> usize {
    if settlements >= 2 {
        2
    } else if has_road || settlements > 0 || is_march {
        // The marches carry no road and no town, but a keeper waits at the edge.
        1
    } else {
        0
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
    let season = crate::model::Season::from_day(day);

    // The road moves them on after a few days.
    sim.wayfarers
        .retain(|w| day.saturating_sub(w.spawned_day) < WAYFARER_DAYS);

    let seed = sim.world.seed;
    let region_count = sim.world.regions.len();
    for ri in 0..region_count {
        let (has_road, settlements, trader_people, is_march) = {
            let r = &sim.world.regions[ri];
            let has_road = r.terrain.tiles.iter().any(|t| matches!(t, Terrain::Road));
            (
                has_road,
                r.settlements.iter().filter(|s| s.population > 0).count(),
                road_trader_people(&r.region_type),
                r.is_march,
            )
        };
        let target = target_wayfarers(has_road, settlements, is_march);
        let here = sim.wayfarers.iter().filter(|w| w.region_idx == ri).count();
        if here >= target {
            continue;
        }
        let n = sim.wayfarers.len();
        let mut rng = SeedRng::new(seed).fork_for(&format!("wayfarer-{ri}-{day}-{n}"));
        // Most who walk the road are wandering folk; now and then the country's
        // own people sends a trader down to barter; rarer still, the road throws
        // up someone in need — a lost child near a town, a soul caught in the
        // winter cold, a passing funeral, a farmer's strayed beasts. The
        // need-moments are gated to where they make sense (#649 slice 4).
        let settled = settlements > 0;
        let winter = season == crate::model::Season::Frost;
        // The ungoverned marches carry no wandering folk and no caravans — only,
        // now and then, the keeper at the edge who asks a price for the road
        // past (#455). Settled country gets the ordinary roll.
        let kind = if is_march {
            if rng.next_u64().is_multiple_of(3) {
                WayfarerKind::ThresholdKeeper
            } else {
                WayfarerKind::Hermit // else the wild's edge holds only a hermit
            }
        } else {
            match rng.next_u64() % 8 {
                0 => WayfarerKind::Traveler,
                1 => WayfarerKind::Bard,
                2 => WayfarerKind::Pilgrim,
                3 => WayfarerKind::Hermit,
                4 => trader_people
                    .map(WayfarerKind::Trader)
                    .unwrap_or(WayfarerKind::Traveler),
                5 if settled => WayfarerKind::LostChild,
                6 if winter => WayfarerKind::WinterSurvivor,
                7 if settled => {
                    if rng.next_u64().is_multiple_of(2) {
                        WayfarerKind::FuneralProcession
                    } else {
                        WayfarerKind::EscapedLivestock
                    }
                }
                // Off the gate, just a traveller on the road.
                _ => WayfarerKind::Traveler,
            }
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
