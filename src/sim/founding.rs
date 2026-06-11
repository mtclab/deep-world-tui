//! Founding: the world recognizes a fed place. Settlers come FROM somewhere —
//! drawn out of real settlements and subtracted from them (people are
//! conserved) — and a new hamlet is named by the region's naming tradition,
//! never the founder's. The founding is remembered in the record; the player
//! gets fame, not naming rights. Reused by re-population/land-taking (#346):
//! the same machinery serves whether the hand on the first stake is the
//! player's or a sponsor-polity's.

use crate::model::economy::Settlement;
use crate::model::{Person, Terrain};
use crate::rng::SeedRng;
use crate::sim::SimState;

/// Draw up to `n` settlers out of the region's (and its neighbors')
/// settlements. Each drawn person leaves their people-roster AND their
/// settlement's population count — nothing from nothing. Settlements near
/// their own floor are left alone (a hamlet is not drained to found a hamlet).
pub fn draw_settlers(
    sim: &mut SimState,
    region_idx: usize,
    n: u32,
    rng: &mut SeedRng,
) -> Vec<Person> {
    let neighbor_idxs = {
        let Some(region) = sim.world.regions.get(region_idx) else {
            return Vec::new();
        };
        let nb = &region.neighbors;
        let mut idxs = vec![region_idx];
        for cand in [nb.north, nb.east, nb.south, nb.west].into_iter().flatten() {
            if cand < sim.world.regions.len() && !idxs.contains(&cand) {
                idxs.push(cand);
            }
        }
        idxs
    };
    let mut drawn = Vec::new();
    for _ in 0..n {
        // Sources: settlements still comfortably peopled.
        let mut sources: Vec<(usize, usize)> = Vec::new();
        for &ri in &neighbor_idxs {
            for (si, s) in sim.world.regions[ri].settlements.iter().enumerate() {
                if s.population > 20 && !s.people.is_empty() {
                    sources.push((ri, si));
                }
            }
        }
        if sources.is_empty() {
            break;
        }
        let (ri, si) = sources[rng.gen_range(sources.len() as u32) as usize];
        let s = &mut sim.world.regions[ri].settlements[si];
        let pi = rng.gen_range(s.people.len() as u32) as usize;
        let person = s.people.remove(pi);
        s.population = s.population.saturating_sub(1);
        drawn.push(person);
    }
    drawn
}

/// Raise a new hamlet at a tile: named by the region's naming tradition,
/// peopled by the settlers handed in (their mix carried as-is — god-peoples
/// stay minorities), inserted so the tile↔settlement x-order mapping holds,
/// and the ground marked. Returns (settlement_id, name).
pub fn spawn_settlement(
    sim: &mut SimState,
    region_idx: usize,
    x: usize,
    y: usize,
    mut settlers: Vec<Person>,
    rng: &mut SeedRng,
) -> Option<(String, String)> {
    if settlers.is_empty() || region_idx >= sim.world.regions.len() {
        return None;
    }
    let (region_id, region_type) = {
        let r = &sim.world.regions[region_idx];
        (r.id.clone(), r.region_type.clone())
    };
    let stem_people = crate::gen::world::naming_tradition(&region_type);
    let base_name = crate::gen::name::generate_place_stem(rng, stem_people, &sim.charts)
        .unwrap_or_else(|_| "Unnamed".into());
    let suffix = sim
        .charts
        .settlement_suffixes
        .get(&region_type)
        .and_then(|suffixes| {
            let idx = rng.gen_range(suffixes.len() as u32) as usize;
            suffixes.get(idx).cloned()
        });
    let name = match suffix {
        Some(s) => format!("{}{}", base_name, s),
        None => base_name,
    };
    let region = &mut sim.world.regions[region_idx];
    let settlement_id = format!("{}-set-f{:04x}", region_id, region.settlements.len());
    for p in settlers.iter_mut() {
        p.settlement = settlement_id.clone();
        p.region = region_id.clone();
    }
    // The settlers' own mix decides the hamlet's character.
    let dominant_people = {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for p in &settlers {
            *counts.entry(p.people.as_str()).or_default() += 1;
        }
        counts
            .into_iter()
            .max_by_key(|&(_, c)| c)
            .map(|(p, _)| p.to_string())
            .unwrap_or_default()
    };
    let population = settlers.len() as u32;
    let settlement = Settlement {
        id: settlement_id.clone(),
        name: name.clone(),
        size: "hamlet".into(),
        region: region_id,
        population,
        description: String::new(),
        people: settlers,
        services: crate::gen::world::settlement_services("hamlet", &dominant_people),
        politics: crate::model::SettlementPolitics::new(),
        food_stock: population as f64 * 2.0,
        farms: Vec::new(),
        buildings: Vec::new(),
        festival_until_day: 0,
        famine_days: 0,
    };
    // The tile↔settlement mapping sorts Settlement tiles by x; insert the new
    // settlement at its x-rank so every existing index keeps meaning itself.
    let insert_at = {
        let mut rank = 0usize;
        for (i, &t) in region.terrain.tiles.iter().enumerate() {
            if t == Terrain::Settlement && i % region.terrain.width < x {
                rank += 1;
            }
        }
        rank.min(region.settlements.len())
    };
    region.settlements.insert(insert_at, settlement);
    // Mark the ground: the site itself, with worked land around it (the same
    // hand worldgen uses).
    if y < region.terrain.height && x < region.terrain.width {
        region.terrain.tiles[y * region.terrain.width + x] = Terrain::Settlement;
    }
    for dy in 0..3usize {
        for dx in 0..3usize {
            let (tx, ty) = (x + dx, y + dy);
            if ty < region.terrain.height
                && tx < region.terrain.width
                && region.terrain.tiles[ty * region.terrain.width + tx] != Terrain::Settlement
            {
                region.terrain.tiles[ty * region.terrain.width + tx] = Terrain::Farmland;
            }
        }
    }
    Some((settlement_id, name))
}
