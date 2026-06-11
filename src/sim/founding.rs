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

/// Draw up to `n` souls out of one settlement — the sponsor pays. Population
/// moves soul by soul; person records move with them while the roster can
/// spare them (the people vec is a sample of the population, kept viable).
/// Returns (records moved, souls moved). Stops at the 20-soul floor.
pub fn draw_from_settlement(
    sim: &mut SimState,
    region_idx: usize,
    settlement_idx: usize,
    n: u32,
    rng: &mut SeedRng,
) -> (Vec<Person>, u32) {
    let Some(s) = sim
        .world
        .regions
        .get_mut(region_idx)
        .and_then(|r| r.settlements.get_mut(settlement_idx))
    else {
        return (Vec::new(), 0);
    };
    let mut records = Vec::new();
    let mut souls = 0u32;
    for _ in 0..n {
        if s.population <= 20 {
            break;
        }
        s.population -= 1;
        souls += 1;
        if s.people.len() > 3 {
            let pi = rng.gen_range(s.people.len() as u32) as usize;
            records.push(s.people.remove(pi));
        }
    }
    (records, souls)
}

/// The world builds back without the player (#346), checked each season-turn.
/// Reopenings first — the chronicle is full of them — then, rarely, a founding
/// party into the rich frontier. Disciplines from the §8 audit: the sponsor
/// PAYS (population and stores actually move), and the pace stays slow and
/// rare — the Fall's tail is long, and some ghost towns simply stay empty.
pub fn tick_world_building(sim: &mut SimState, season_idx: u32) {
    let seed = sim.world.seed;
    let tick = sim.world.tick;

    // --- Resettlement: a prosperous neighbor reopens a ghost town. ---
    let ghost_sites: Vec<(usize, usize)> = sim
        .world
        .regions
        .iter()
        .enumerate()
        .flat_map(|(ri, r)| {
            r.settlements
                .iter()
                .enumerate()
                .filter(|(_, s)| s.population == 0)
                .map(move |(si, _)| (ri, si))
        })
        .collect();
    for (gri, gsi) in ghost_sites {
        let ghost_id = sim.world.regions[gri].settlements[gsi].id.clone();
        let mut rng = SeedRng::new(seed).fork_for(&format!("reopen-{season_idx}-{ghost_id}"));
        // Slow and rare; some wounds don't close.
        if rng.gen_range(3) != 0 {
            continue;
        }
        let Some((sri, ssi)) = best_sponsor(sim, gri, Some((gri, gsi))) else {
            continue;
        };
        let want = 10 + rng.gen_range(11); // 10–20 souls
        let (mut records, souls) = draw_from_settlement(sim, sri, ssi, want, &mut rng);
        if souls < 10 {
            // Not enough hands to reopen a town: send them home.
            let s = &mut sim.world.regions[sri].settlements[ssi];
            s.population += souls;
            s.people.append(&mut records);
            continue;
        }
        // Stores travel with the party — the sponsor's larder pays for it.
        let moved_food = {
            let sp = &mut sim.world.regions[sri].settlements[ssi];
            let take = (sp.food_stock * 0.25).min(2.0 * souls as f64).max(0.0);
            sp.food_stock -= take;
            take
        };
        let sponsor_name = sim.world.regions[sri].settlements[ssi].name.clone();
        let (ghost_region_id, dominant) = {
            let r = &sim.world.regions[gri];
            let dom = records
                .first()
                .map(|p| p.people.clone())
                .unwrap_or_default();
            (r.id.clone(), dom)
        };
        let ghost = &mut sim.world.regions[gri].settlements[gsi];
        for p in records.iter_mut() {
            p.settlement = ghost.id.clone();
            p.region = ghost_region_id.clone();
        }
        ghost.population = souls;
        ghost.people = records;
        ghost.size = "hamlet".into();
        ghost.services = crate::gen::world::settlement_services("hamlet", &dominant);
        ghost.food_stock = moved_food;
        ghost.famine_days = 0;
        ghost.description = String::new();
        let ghost_name = ghost.name.clone();
        sim.log(
            tick,
            crate::sim::journal::Voice::Rumor,
            format!(
                "{} sends families to reopen {} — carts on the road, doors coming off \
                 their boards.",
                sponsor_name, ghost_name
            ),
        );
    }

    // --- Land-taking: a founding party into the rich frontier, rarely. ---
    let region_count = sim.world.regions.len();
    for ri in 0..region_count {
        {
            let r = &sim.world.regions[ri];
            let living = r.settlements.iter().filter(|s| s.population > 0).count();
            if r.game_richness <= 0.85 || living > 1 || r.settlements.len() >= 3 {
                continue;
            }
        }
        let mut rng = SeedRng::new(seed).fork_for(&format!("landtake-{season_idx}-{ri}"));
        if rng.gen_range(8) != 0 {
            continue;
        }
        // Open ground well clear of any settled tile.
        let site = {
            let terr = &sim.world.regions[ri].terrain;
            let mut found = None;
            'o: for y in 0..terr.height {
                for x in 0..terr.width {
                    if terr.get(x, y) != Some(Terrain::Grass) {
                        continue;
                    }
                    let near = (y.saturating_sub(8)..(y + 9).min(terr.height)).any(|ty| {
                        (x.saturating_sub(8)..(x + 9).min(terr.width))
                            .any(|tx| terr.get(tx, ty) == Some(Terrain::Settlement))
                    });
                    if !near {
                        found = Some((x, y));
                        break 'o;
                    }
                }
            }
            found
        };
        let Some((x, y)) = site else { continue };
        let Some((sri, ssi)) = best_sponsor(sim, ri, None) else {
            continue;
        };
        let want = 10 + rng.gen_range(5); // 10–14 souls
        let (mut records, souls) = draw_from_settlement(sim, sri, ssi, want, &mut rng);
        if souls < 10 || records.is_empty() {
            let s = &mut sim.world.regions[sri].settlements[ssi];
            s.population += souls;
            s.people.append(&mut records);
            continue;
        }
        let sponsor_name = sim.world.regions[sri].settlements[ssi].name.clone();
        if let Some((new_id, new_name)) = spawn_settlement(sim, ri, x, y, records, &mut rng) {
            // The party is larger than its named record — population counts
            // every soul the sponsor sent, not just the sampled roster.
            if let Some(s) = sim.world.regions[ri]
                .settlements
                .iter_mut()
                .find(|s| s.id == new_id)
            {
                s.population = souls;
                s.food_stock = souls as f64 * 2.0;
            }
            let region_name = sim.world.regions[ri].name.clone();
            sim.log(
                tick,
                crate::sim::journal::Voice::Rumor,
                format!(
                    "{} has sent a founding party into {} — they are calling the new \
                     place {}.",
                    sponsor_name, region_name, new_name
                ),
            );
        }
    }
}

/// The most prosperous settlement in a region or its neighbors that can spare
/// people: population past 30 with better than a meal and a half per head.
/// `exclude` keeps a ghost town from sponsoring itself.
fn best_sponsor(
    sim: &SimState,
    region_idx: usize,
    exclude: Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    let mut idxs = vec![region_idx];
    if let Some(r) = sim.world.regions.get(region_idx) {
        let nb = &r.neighbors;
        for cand in [nb.north, nb.east, nb.south, nb.west].into_iter().flatten() {
            if cand < sim.world.regions.len() && !idxs.contains(&cand) {
                idxs.push(cand);
            }
        }
    }
    let mut best: Option<(usize, usize, f64)> = None;
    for &ri in &idxs {
        for (si, s) in sim.world.regions[ri].settlements.iter().enumerate() {
            if exclude == Some((ri, si)) {
                continue;
            }
            if s.population <= 30 {
                continue;
            }
            let per_head = s.food_stock / s.population as f64;
            if per_head <= 1.5 {
                continue;
            }
            if best.map(|(_, _, b)| per_head > b).unwrap_or(true) {
                best = Some((ri, si, per_head));
            }
        }
    }
    best.map(|(ri, si, _)| (ri, si))
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
