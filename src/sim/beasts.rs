//! Wild beasts as actors on the grid (#637): the Wildlife encounter was a
//! random roll that cut away to a menu. Now the land's creatures stand on the
//! map — each beast its own actor, on its own tile, with its own species and
//! its own toughness — to be seen, avoided, hunted, or fled. Persistent in the
//! sim (a felled beast stays felled; the land's game grows back over seasons),
//! spawned from each region's wildness, thickest in the deep wild of a march.

use serde::{Deserialize, Serialize};

use crate::model::wildlife::WildSpecies;
use crate::rng::SeedRng;

/// A single wild creature standing somewhere in a region (#637).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WildBeast {
    pub id: String,
    pub species: WildSpecies,
    pub region_idx: usize,
    pub px: usize,
    pub py: usize,
    /// Blows it can take before it falls — its toughness, from its danger.
    pub hp: u32,
}

/// How much fight a creature has in it, from its danger rating: a hare drops to
/// one blow, a wolf takes a few, a bear is a real fight.
pub fn beast_hp(species: WildSpecies) -> u32 {
    match species.danger() {
        0 => 1,
        1 => 4,
        _ => 8,
    }
}

/// How many beasts a region should hold, from its wildness: richer game and the
/// ungoverned deep wild of a march carry more.
fn target_beasts(richness: f64, is_march: bool) -> usize {
    let base = (richness * 4.0).round() as usize;
    if is_march { base + 3 } else { base }.min(8)
}

/// Keep each region stocked with its wild beasts (#637): top up toward the
/// region's target on the day's turn, spawning each on its own wild tile with a
/// terrain- and season-true species (the dreads weigh heavier in a march, #630).
/// Felled beasts are not replaced until the land's turn comes round again, so a
/// hunted-out country stays quiet a while. Deterministic.
pub fn tick_beasts(sim: &mut crate::sim::SimState) {
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    let day = (tick / 24) as u32;
    let seed = sim.world.seed;
    let season = crate::model::Season::from_day(day);

    let region_count = sim.world.regions.len();
    for ri in 0..region_count {
        let (richness, is_march, terrain) = {
            let r = &sim.world.regions[ri];
            (
                r.game_richness,
                r.is_march,
                crate::sim::region_work_terrain(&r.region_type),
            )
        };
        let target = target_beasts(richness, is_march);
        let here = sim.beasts.iter().filter(|b| b.region_idx == ri).count();
        if here >= target {
            continue;
        }
        // Spawn one toward the target each turn — the land restocks slowly.
        let n = sim.beasts.len();
        let mut rng = SeedRng::new(seed).fork_for(&format!("beast-{ri}-{day}-{n}"));
        let uncanny_boost = if is_march { 6 } else { 1 };
        let Some(species) =
            WildSpecies::roll_biased(terrain, season, rng.next_u64(), uncanny_boost)
        else {
            continue;
        };
        let Some((px, py)) = wild_tile(sim, ri, &mut rng) else {
            continue;
        };
        sim.beasts.push(WildBeast {
            id: format!("beast-{seed}-{day}-{n}"),
            species,
            region_idx: ri,
            px,
            py,
            hp: beast_hp(species),
        });
    }
}

/// A random wild tile (grass/forest) in a region, clear of any beast already
/// there. `None` if the region has no open wild ground.
fn wild_tile(
    sim: &crate::sim::SimState,
    region_idx: usize,
    rng: &mut SeedRng,
) -> Option<(usize, usize)> {
    let terr = &sim.world.regions.get(region_idx)?.terrain;
    let mut wild: Vec<(usize, usize)> = Vec::new();
    for y in 0..terr.height {
        for x in 0..terr.width {
            if matches!(
                terr.get(x, y),
                Some(crate::model::Terrain::Grass | crate::model::Terrain::Forest)
            ) && !sim
                .beasts
                .iter()
                .any(|b| b.region_idx == region_idx && b.px == x && b.py == y)
            {
                wild.push((x, y));
            }
        }
    }
    if wild.is_empty() {
        return None;
    }
    Some(wild[(rng.next_u64() % wild.len() as u64) as usize])
}
