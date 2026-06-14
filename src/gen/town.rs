//! Town layout (#372 PR 2): a settlement's footprint is a district, not a
//! smear of one terrain. Streets run between roofed houses; the tavern, the
//! temple, the forge each stand at their own door. The layout is derived —
//! anchor + footprint + services decide everything — so nothing new needs
//! persisting and every caller (worldgen, promotion, old-save fixup,
//! foundings) lays the same town.
//!
//! The pattern: within the n x n footprint, cells whose relative x AND y are
//! both even carry a house; every other cell is street. Every house touches
//! a street, the streets connect to each other and to the footprint's edge,
//! and the house count scales with the size tier (hamlet 1, village 4,
//! town 9, city 16).

use crate::model::economy::Settlement;
use crate::model::{SettlementService, Terrain, TerrainMap};

/// How much trade reach a spot has: roads carry grain, harbors carry more.
/// 1.0 = subsistence only; >= 1.4 = a real hinterland connection.
pub fn trade_factor(terrain: &TerrainMap, x: usize, y: usize) -> f64 {
    let (w, h) = (terrain.width, terrain.height);
    let mut roads = 0;
    let mut harbor = false;
    for ty in y.saturating_sub(12)..(y + 13).min(h) {
        for tx in x.saturating_sub(12)..(x + 13).min(w) {
            match terrain.tiles[ty * w + tx] {
                Terrain::Road => roads += 1,
                Terrain::Water | Terrain::Coast => harbor = true,
                _ => {}
            }
        }
    }
    1.0 + if roads >= 3 { 0.6 } else { 0.0 } + if harbor { 0.4 } else { 0.0 }
}

/// Carrying capacity of a spot, computed from the canon's own hydraulic
/// principles (population_scale_and_settlement_hierarchy.md): people settle
/// where water runs (off-water densities one-fifth the riverine standard),
/// arable hinterland feeds the head-count, terrain sets the base, and trade
/// reach moves surplus grain. Nothing here is authored per-settlement: the
/// land decides what it can carry.
pub fn carrying_capacity(terrain: &TerrainMap, x: usize, y: usize, region_type: &str) -> u32 {
    let (w, h) = (terrain.width, terrain.height);
    let base: f64 = match region_type {
        "delta" => 3_500.0,
        "river_valley" => 3_000.0,
        "coast" => 2_000.0,
        "steppe" => 700.0,
        "forest" => 600.0,
        "upland" => 400.0,
        _ => 1_000.0,
    };
    // Water within reach: the river-corridor principle.
    let mut water = 0;
    for ty in y.saturating_sub(12)..(y + 13).min(h) {
        for tx in x.saturating_sub(12)..(x + 13).min(w) {
            if matches!(terrain.tiles[ty * w + tx], Terrain::Water | Terrain::Coast) {
                water += 1;
            }
        }
    }
    let water_factor = match water {
        0 => 0.2,
        1..=5 => 1.0,
        _ => 1.5,
    };
    // Arable hinterland: grass and worked land within walking reach. The
    // town's own paint (streets, roofs) is excluded from the measure — the
    // ground was arable before the town stood on it, and a settlement must
    // not erode its own ceiling by existing.
    let mut arable = 0usize;
    let mut total = 0usize;
    for ty in y.saturating_sub(14)..(y + 15).min(h) {
        for tx in x.saturating_sub(14)..(x + 15).min(w) {
            match terrain.tiles[ty * w + tx] {
                Terrain::Settlement | Terrain::House => {}
                Terrain::Grass | Terrain::Farmland => {
                    arable += 1;
                    total += 1;
                }
                _ => total += 1,
            }
        }
    }
    let arable_factor = 0.4 + 1.6 * (arable as f64 / total.max(1) as f64);
    let cap = base * water_factor * arable_factor * trade_factor(terrain, x, y);
    (cap as u32).max(12)
}

/// The real buildings of a settlement (#458): the single source of truth that
/// worldgen paints and every consumer reads, recomputed deterministically from
/// the anchor so service-doors, walls, and NPC streets always agree.
pub fn town_buildings(settlement: &Settlement) -> Vec<crate::gen::building::PlacedBuilding> {
    let n = settlement.footprint() as usize;
    crate::gen::building::district_buildings(
        settlement.map_x as usize,
        settlement.map_y as usize,
        n,
        n,
        crate::gen::building::town_seed(settlement.map_x, settlement.map_y),
    )
}

/// Paint a settlement's district onto the map: real buildings (walls, floors,
/// doors) on walkable streets (#458), worked land skirting the edge. Safe at
/// map edges (clamped). The anchor seeds the layout so consumers can recompute
/// the same buildings.
pub fn lay_town(terrain: &mut TerrainMap, anchor_x: usize, anchor_y: usize, footprint: usize) {
    crate::gen::building::lay_district(
        terrain,
        anchor_x,
        anchor_y,
        footprint,
        footprint,
        crate::gen::building::town_seed(anchor_x as u32, anchor_y as u32),
    );
    // Worked land skirts the walls — and the water keeps its bed.
    let (w, h) = (terrain.width, terrain.height);
    for dy in 0..footprint + 2 {
        for dx in 0..footprint + 2 {
            let (tx, ty) = (anchor_x + dx, anchor_y + dy);
            if tx < w
                && ty < h
                && !matches!(
                    terrain.tiles[ty * w + tx],
                    Terrain::House
                        | Terrain::Settlement
                        | Terrain::Wall
                        | Terrain::Floor
                        | Terrain::Door
                        | Terrain::Water
                        | Terrain::Coast
                )
            {
                terrain.tiles[ty * w + tx] = Terrain::Farmland;
            }
        }
    }
}

/// Which service (if any) keeps its door at this tile. Services take the
/// buildings in reading order; later buildings are homes.
pub fn service_at(settlement: &Settlement, x: usize, y: usize) -> Option<SettlementService> {
    let buildings = town_buildings(settlement);
    let idx = buildings.iter().position(|b| b.door == (x, y))?;
    settlement.services.get(idx).copied()
}

/// Where the settlement's people are, by the clock (#458): out in the streets
/// through the day (06–20), and **indoors by night** — on the floors of their
/// own buildings, by the hearth, where you can still walk in and meet them.
/// Deterministic per (person, day): the same woman keeps the same corner all
/// day and the same room all night, a different one tomorrow. Used by both the
/// renderer and the bump-to-talk layer, so what you see is exactly who you can
/// meet.
pub fn npc_street_positions(
    settlement: &Settlement,
    day: u32,
    hour: u32,
) -> Vec<(usize, usize, usize)> {
    let (ax, ay, n) = (
        settlement.map_x as usize,
        settlement.map_y as usize,
        settlement.footprint() as usize,
    );
    let buildings = town_buildings(settlement);
    let day_time = (6..21).contains(&hour);
    // By day, the candidate tiles are the open streets between buildings; by
    // night, the interior floors of the buildings themselves.
    let mut tiles: Vec<(usize, usize)> = Vec::new();
    if day_time {
        let in_building = |x: usize, y: usize| {
            buildings
                .iter()
                .any(|b| x >= b.x && x < b.x + b.w && y >= b.y && y < b.y + b.h)
        };
        for dy in 0..n {
            for dx in 0..n {
                let (x, y) = (ax + dx, ay + dy);
                if !in_building(x, y) {
                    tiles.push((x, y));
                }
            }
        }
    } else {
        for b in &buildings {
            for iy in (b.y + 1)..(b.y + b.h.saturating_sub(1)) {
                for ix in (b.x + 1)..(b.x + b.w.saturating_sub(1)) {
                    tiles.push((ix, iy));
                }
            }
        }
    }
    if tiles.is_empty() {
        return Vec::new();
    }
    let mut taken: Vec<usize> = Vec::new();
    let mut out = Vec::new();
    for pi in 0..settlement.people.len().min(tiles.len()) {
        let mut slot = (pi * 5 + day as usize) % tiles.len();
        while taken.contains(&slot) {
            slot = (slot + 1) % tiles.len();
        }
        taken.push(slot);
        let (x, y) = tiles[slot];
        out.push((pi, x, y));
    }
    out
}

/// Whether this tile is part of one of the settlement's buildings.
pub fn is_house_of(settlement: &Settlement, x: usize, y: usize) -> bool {
    town_buildings(settlement)
        .iter()
        .any(|b| x >= b.x && x < b.x + b.w && y >= b.y && y < b.y + b.h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_settlement(map_x: u32, map_y: u32, district: u32) -> Settlement {
        Settlement {
            id: "t".into(),
            name: "Test".into(),
            size: "town".into(),
            region: "r".into(),
            population: 100,
            description: String::new(),
            people: Vec::new(),
            services: Vec::new(),
            politics: crate::model::SettlementPolitics::new(),
            food_stock: 0.0,
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            map_x,
            map_y,
            district,
        }
    }

    #[test]
    fn every_building_door_opens_onto_a_walkable_street() {
        let mut terrain = TerrainMap {
            width: 60,
            height: 60,
            tiles: vec![Terrain::Grass; 3600],
        };
        for n in [6usize, 12, 24, 40] {
            lay_town(&mut terrain, 5, 5, n);
            let s = test_settlement(5, 5, n as u32);
            let buildings = town_buildings(&s);
            assert!(!buildings.is_empty(), "n={n} lays at least one building");
            for b in &buildings {
                let (dx, dy) = b.door;
                assert_eq!(terrain.tiles[dy * 60 + dx], Terrain::Door, "door painted");
                let opens = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)]
                    .into_iter()
                    .any(|(ox, oy)| {
                        let (nx, ny) = ((dx as i32 + ox) as usize, (dy as i32 + oy) as usize);
                        matches!(
                            terrain.tiles[ny * 60 + nx],
                            Terrain::Settlement | Terrain::Farmland
                        )
                    });
                assert!(opens, "door at ({dx},{dy}) n={n} opens onto a street");
            }
        }
    }

    #[test]
    fn building_counts_scale_with_the_size() {
        // A bigger district holds more buildings than a small holding.
        let small = town_buildings(&test_settlement(5, 5, 6)).len();
        let big = town_buildings(&test_settlement(5, 5, 40)).len();
        assert!(small >= 1, "even a hamlet gets a dwelling");
        assert!(big > small, "a city holds more buildings than a hamlet");
    }

    #[test]
    fn service_doors_match_building_doors() {
        let mut s = test_settlement(5, 5, 24);
        s.services = vec![SettlementService::Tavern, SettlementService::Temple];
        let buildings = town_buildings(&s);
        for (i, svc) in s.services.iter().enumerate() {
            let (dx, dy) = buildings[i].door;
            assert_eq!(service_at(&s, dx, dy), Some(*svc));
        }
    }
}
