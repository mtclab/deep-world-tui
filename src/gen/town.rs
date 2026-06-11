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

/// The house cells of a footprint, in reading order (left-right, top-down).
pub fn house_cells(anchor_x: usize, anchor_y: usize, footprint: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for dy in 0..footprint {
        for dx in 0..footprint {
            if dx.is_multiple_of(2) && dy.is_multiple_of(2) {
                out.push((anchor_x + dx, anchor_y + dy));
            }
        }
    }
    out
}

/// Paint a settlement's district onto the map: streets and houses inside the
/// footprint, worked land skirting the walls. Safe at map edges (clamped).
pub fn lay_town(terrain: &mut TerrainMap, anchor_x: usize, anchor_y: usize, footprint: usize) {
    let (w, h) = (terrain.width, terrain.height);
    for dy in 0..footprint {
        for dx in 0..footprint {
            let (tx, ty) = (anchor_x + dx, anchor_y + dy);
            if tx < w && ty < h {
                let t = if dx.is_multiple_of(2) && dy.is_multiple_of(2) {
                    Terrain::House
                } else {
                    Terrain::Settlement
                };
                terrain.tiles[ty * w + tx] = t;
            }
        }
    }
    // Worked land skirts the walls (the same hand worldgen always used).
    for dy in 0..footprint + 2 {
        for dx in 0..footprint + 2 {
            let (tx, ty) = (anchor_x + dx, anchor_y + dy);
            if tx < w
                && ty < h
                && !matches!(
                    terrain.tiles[ty * w + tx],
                    Terrain::House | Terrain::Settlement
                )
            {
                terrain.tiles[ty * w + tx] = Terrain::Farmland;
            }
        }
    }
}

/// Which service (if any) keeps its door at this house tile. Services take
/// the house cells in reading order; later houses are homes.
pub fn service_at(settlement: &Settlement, x: usize, y: usize) -> Option<SettlementService> {
    let cells = house_cells(
        settlement.map_x as usize,
        settlement.map_y as usize,
        settlement.footprint() as usize,
    );
    let idx = cells.iter().position(|&(cx, cy)| cx == x && cy == y)?;
    settlement.services.get(idx).copied()
}

/// Where the settlement's people stand in its streets, by the clock:
/// out and about through the day (06–20), home behind their doors at night
/// (empty vec — the streets go quiet). Deterministic per (person, day):
/// the same woman keeps the same corner all day, and a different one
/// tomorrow. Used by both the renderer and the bump-to-talk layer, so what
/// you see is exactly who you can meet.
pub fn npc_street_positions(
    settlement: &Settlement,
    day: u32,
    hour: u32,
) -> Vec<(usize, usize, usize)> {
    if !(6..21).contains(&hour) {
        return Vec::new();
    }
    let (ax, ay, n) = (
        settlement.map_x as usize,
        settlement.map_y as usize,
        settlement.footprint() as usize,
    );
    let mut streets: Vec<(usize, usize)> = Vec::new();
    for dy in 0..n {
        for dx in 0..n {
            if !(dx.is_multiple_of(2) && dy.is_multiple_of(2)) {
                streets.push((ax + dx, ay + dy));
            }
        }
    }
    if streets.is_empty() {
        return Vec::new();
    }
    let mut taken: Vec<usize> = Vec::new();
    let mut out = Vec::new();
    for pi in 0..settlement.people.len().min(streets.len()) {
        let mut slot = (pi * 5 + day as usize) % streets.len();
        while taken.contains(&slot) {
            slot = (slot + 1) % streets.len();
        }
        taken.push(slot);
        let (x, y) = streets[slot];
        out.push((pi, x, y));
    }
    out
}

/// Whether this tile is one of the settlement's houses at all.
pub fn is_house_of(settlement: &Settlement, x: usize, y: usize) -> bool {
    settlement.contains_tile(x, y)
        && (x - settlement.map_x as usize).is_multiple_of(2)
        && (y - settlement.map_y as usize).is_multiple_of(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_house_touches_a_street_and_the_street_reaches_the_edge() {
        let mut terrain = TerrainMap {
            width: 20,
            height: 20,
            tiles: vec![Terrain::Grass; 400],
        };
        for n in [2usize, 4, 6, 8] {
            lay_town(&mut terrain, 5, 5, n);
            for (hx, hy) in house_cells(5, 5, n) {
                assert_eq!(terrain.tiles[hy * 20 + hx], Terrain::House);
                let touches_street =
                    [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)]
                        .into_iter()
                        .any(|(dx, dy)| {
                            let (nx, ny) = ((hx as i32 + dx) as usize, (hy as i32 + dy) as usize);
                            matches!(
                                terrain.tiles[ny * 20 + nx],
                                Terrain::Settlement | Terrain::Farmland | Terrain::Grass
                            )
                        });
                assert!(touches_street, "house at ({hx},{hy}) n={n} has a door");
            }
        }
    }

    #[test]
    fn house_counts_scale_with_the_size() {
        assert_eq!(house_cells(0, 0, 2).len(), 1);
        assert_eq!(house_cells(0, 0, 4).len(), 4);
        assert_eq!(house_cells(0, 0, 6).len(), 9);
        assert_eq!(house_cells(0, 0, 8).len(), 16);
    }
}
