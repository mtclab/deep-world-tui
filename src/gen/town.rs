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
pub fn trade_factor(tiles: &[Terrain], w: usize, h: usize, x: usize, y: usize) -> f64 {
    let mut roads = 0;
    let mut harbor = false;
    for ty in y.saturating_sub(12)..(y + 13).min(h) {
        for tx in x.saturating_sub(12)..(x + 13).min(w) {
            match tiles[ty * w + tx] {
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
pub fn carrying_capacity(
    tiles: &[Terrain],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    region_type: &str,
) -> u32 {
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
            if matches!(tiles[ty * w + tx], Terrain::Water | Terrain::Coast) {
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
            match tiles[ty * w + tx] {
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
    let cap = base * water_factor * arable_factor * trade_factor(tiles, w, h, x, y);
    (cap as u32).max(12)
}

/// The real buildings of a settlement (#458): the single source of truth that
/// worldgen paints and every consumer reads, recomputed deterministically from
/// the anchor so service-doors, walls, and NPC streets always agree.
pub fn town_buildings(settlement: &Settlement) -> Vec<crate::gen::building::PlacedBuilding> {
    let n = settlement.footprint() as usize;
    let character = crate::gen::building::BuildCharacter::from_people(
        &settlement
            .people
            .first()
            .map(|p| p.people.clone())
            .unwrap_or_default(),
    );
    crate::gen::building::district_buildings(
        settlement.map_x as usize,
        settlement.map_y as usize,
        n,
        n,
        crate::gen::building::town_seed(settlement.map_x, settlement.map_y),
        character,
    )
}

/// Paint a settlement's district onto the map: real buildings (walls, floors,
/// doors) on walkable streets (#458), worked land skirting the edge. Safe at
/// map edges (clamped). The anchor seeds the layout so consumers can recompute
/// the same buildings. `character` is the dominant people's building character,
/// which must match what `town_buildings` derives so painted and computed agree.
pub fn lay_town(
    terrain: &mut TerrainMap,
    anchor_x: usize,
    anchor_y: usize,
    footprint: usize,
    character: crate::gen::building::BuildCharacter,
) {
    crate::gen::building::lay_district(
        terrain,
        anchor_x,
        anchor_y,
        footprint,
        footprint,
        crate::gen::building::town_seed(anchor_x as u32, anchor_y as u32),
        character,
    );
    // A great town stands walled (#449/#458): the buildings already keep a
    // clear lane inside the footprint edge (`wall_border`), so the perimeter
    // can be raised as a wall — with gates where the main street and the
    // cross-street reach the edge, opening onto the lane that rings the town.
    // The streets lead from each gate to the heart, so it stays reachable.
    if crate::gen::building::wall_border(footprint, footprint) > 0 {
        let (w, h) = (terrain.width, terrain.height);
        let gate = 3usize; // = the street width at this scale
        let g0 = footprint / 2 - gate / 2; // main/cross street offset
        let (x1, y1) = (anchor_x + footprint - 1, anchor_y + footprint - 1);
        for d in 0..footprint {
            let on_gate = (g0..g0 + gate).contains(&d);
            for (tx, ty) in [
                (anchor_x + d, anchor_y), // top edge   — main-street gate
                (anchor_x + d, y1),       // bottom edge — main-street gate
                (anchor_x, anchor_y + d), // left edge  — cross-street gate
                (x1, anchor_y + d),       // right edge — cross-street gate
            ] {
                if tx >= w || ty >= h {
                    continue;
                }
                if matches!(terrain.tiles[ty * w + tx], Terrain::Water | Terrain::Coast) {
                    continue; // the water keeps its bed; a town gates the ford itself
                }
                terrain.tiles[ty * w + tx] = if on_gate {
                    Terrain::Door
                } else {
                    Terrain::Wall
                };
            }
        }
    }
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
                        | Terrain::Hearth
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
    // How many leading tiles are the plaza (day-time gathering ground).
    let mut plaza_tiles = 0usize;
    if day_time {
        let in_building = |x: usize, y: usize| {
            buildings
                .iter()
                .any(|b| x >= b.x && x < b.x + b.w && y >= b.y && y < b.y + b.h)
        };
        // The townsfolk gather in the market plaza by day (#458): its open
        // tiles come first, so the square fills before the side lanes, and a
        // town reads as a place with a centre — not folk scattered evenly down
        // every street.
        let plaza = crate::gen::building::central_plaza(n, n)
            .map(|(ox, oy, w, h)| (ax + ox, ay + oy, w, h));
        let in_plaza = |x: usize, y: usize| {
            plaza.is_some_and(|(qx, qy, qw, qh)| x >= qx && x < qx + qw && y >= qy && y < qy + qh)
        };
        let mut lanes: Vec<(usize, usize)> = Vec::new();
        for dy in 0..n {
            for dx in 0..n {
                let (x, y) = (ax + dx, ay + dy);
                if in_building(x, y) {
                    continue;
                }
                if in_plaza(x, y) {
                    tiles.push((x, y));
                } else {
                    lanes.push((x, y));
                }
            }
        }
        plaza_tiles = tiles.len();
        tiles.extend(lanes);
    } else {
        // Indoors, people gather by the fire: collect the interior floors with
        // their distance to their building's hearth (its centre), and take the
        // warmest first — so the household fills the hearth-rooms before the
        // far corners.
        let mut scored: Vec<((usize, usize), usize)> = Vec::new();
        for b in &buildings {
            let (cx, cy) = (b.x + b.w / 2, b.y + b.h / 2);
            for iy in (b.y + 1)..(b.y + b.h.saturating_sub(1)) {
                for ix in (b.x + 1)..(b.x + b.w.saturating_sub(1)) {
                    scored.push(((ix, iy), cx.abs_diff(ix) + cy.abs_diff(iy)));
                }
            }
        }
        // Stable sort on distance keeps it deterministic per (anchor, layout).
        scored.sort_by_key(|&(_, d)| d);
        tiles = scored.into_iter().map(|(t, _)| t).collect();
        // Keep only the warmest tiles — as many as there are souls — so they
        // cluster at the hearths instead of scattering through every room.
        tiles.truncate(settlement.people.len().max(1));
    }
    if tiles.is_empty() {
        return Vec::new();
    }
    let mut taken: Vec<usize> = Vec::new();
    let mut out = Vec::new();
    // By day, fill contiguously from a daily offset *within the plaza* so the
    // souls cluster in the square (and spill into the lanes only once it is
    // full); by night, the warmest hearth tiles, lightly rotated. Either way
    // deterministic per (person, day).
    let start = if day_time && plaza_tiles > 0 {
        day as usize % plaza_tiles
    } else {
        0
    };
    for pi in 0..settlement.people.len().min(tiles.len()) {
        let mut slot = if day_time {
            (start + pi) % tiles.len()
        } else {
            (pi * 5 + day as usize) % tiles.len()
        };
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
            faith: Default::default(),
            food_stock: 0.0,
            goods_stock: Default::default(),
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            map_x,
            map_y,
            district,
            remembered_deed: None,
        }
    }

    #[test]
    fn townsfolk_gather_in_the_market_plaza_by_day() {
        // A town with a plaza (footprint >= 16) gathers its souls in the square
        // at midday, not scattered down every lane.
        let mut s = test_settlement(5, 5, 24);
        s.people = (0..8).map(|_| crate::model::Person::default()).collect();
        let (px, py, pw, ph) = crate::gen::building::central_plaza(24, 24).expect("a plaza");
        let (qx, qy) = (5 + px, 5 + py);
        let pos = npc_street_positions(&s, 1, 12); // day 1, noon
        assert!(!pos.is_empty(), "the townsfolk are out by day");
        let in_plaza = pos
            .iter()
            .filter(|(_, x, y)| *x >= qx && *x < qx + pw && *y >= qy && *y < qy + ph)
            .count();
        assert!(
            in_plaza * 4 >= pos.len() * 3,
            "most townsfolk gather in the plaza by day ({in_plaza}/{})",
            pos.len()
        );
    }

    #[test]
    fn a_great_town_is_walled_with_enterable_gates() {
        let mut t = TerrainMap {
            width: 60,
            height: 60,
            tiles: vec![Terrain::Grass; 3600],
        };
        let fp = 32usize;
        lay_town(
            &mut t,
            10,
            10,
            fp,
            crate::gen::building::BuildCharacter::Plain,
        );
        let get = |x: usize, y: usize| t.tiles[y * 60 + x];
        let (mut walls, mut gates) = (0, 0);
        for d in 0..fp {
            for (x, y) in [
                (10 + d, 10),
                (10 + d, 10 + fp - 1),
                (10, 10 + d),
                (10 + fp - 1, 10 + d),
            ] {
                match get(x, y) {
                    Terrain::Wall => walls += 1,
                    Terrain::Door => gates += 1,
                    _ => {}
                }
                // No building ever fronts the wall — the lane keeps them clear.
                assert!(
                    !matches!(get(x, y), Terrain::Floor | Terrain::Hearth | Terrain::House),
                    "a building sits on the wall ring at ({x},{y})"
                );
            }
        }
        assert!(walls > 0, "a great town stands walled");
        assert!(
            gates >= 4,
            "gates where the streets meet the edge ({gates})"
        );
        // A gate opens onto walkable ground within — the town is enterable.
        let gx = 10 + fp / 2 - 1;
        assert_eq!(get(gx, 10), Terrain::Door, "the gate is a door");
        assert!(
            get(gx, 11).passable(),
            "the gate opens onto the lane within"
        );
    }

    #[test]
    fn a_small_town_keeps_no_wall() {
        let mut t = TerrainMap {
            width: 40,
            height: 40,
            tiles: vec![Terrain::Grass; 1600],
        };
        lay_town(
            &mut t,
            5,
            5,
            20,
            crate::gen::building::BuildCharacter::Plain,
        );
        let mut walls = 0;
        for d in 0..20 {
            for (x, y) in [(5 + d, 5), (5 + d, 24), (5, 5 + d), (24, 5 + d)] {
                if t.tiles[y * 40 + x] == Terrain::Wall {
                    walls += 1;
                }
            }
        }
        assert_eq!(walls, 0, "a small town keeps no wall");
    }

    #[test]
    fn central_plaza_only_for_real_towns() {
        assert!(crate::gen::building::central_plaza(12, 12).is_none());
        let (_, _, w, h) = crate::gen::building::central_plaza(40, 36).expect("a plaza");
        assert!(w >= 4 && h >= 3, "the plaza has real extent ({w}x{h})");
    }

    #[test]
    fn every_building_door_opens_onto_a_walkable_street() {
        let mut terrain = TerrainMap {
            width: 60,
            height: 60,
            tiles: vec![Terrain::Grass; 3600],
        };
        for n in [6usize, 12, 24, 40] {
            lay_town(
                &mut terrain,
                5,
                5,
                n,
                crate::gen::building::BuildCharacter::Plain,
            );
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
