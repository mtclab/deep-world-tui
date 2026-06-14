//! Real buildings on the one world map (#458): a structure is a wall border
//! around a walkable floor, with a doorway you walk in through — not a 1-tile
//! token. Styles vary in size, from a hut to a hall. This is the primitive the
//! richer town/enclave layouts will place; not yet wired into worldgen.

use crate::model::{Terrain, TerrainMap};

/// A building style and the floor space it wants. Sizes are the outer footprint
/// (walls included), so the walkable interior is (w-2) x (h-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingStyle {
    /// A one-room hut.
    Hut,
    /// A modest cottage.
    Cottage,
    /// A long, narrow longhouse.
    Longhouse,
    /// A broad hall (tavern, temple, mead-hall).
    Hall,
    /// A large manor / works.
    Manor,
}

impl BuildingStyle {
    /// Outer footprint (w, h) in tiles, walls included.
    pub fn size(self) -> (usize, usize) {
        match self {
            BuildingStyle::Hut => (3, 3),
            BuildingStyle::Cottage => (4, 4),
            BuildingStyle::Longhouse => (4, 7),
            BuildingStyle::Hall => (6, 6),
            BuildingStyle::Manor => (7, 8),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BuildingStyle::Hut => "hut",
            BuildingStyle::Cottage => "cottage",
            BuildingStyle::Longhouse => "longhouse",
            BuildingStyle::Hall => "hall",
            BuildingStyle::Manor => "manor",
        }
    }
}

/// Which wall the doorway breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    North,
    South,
    East,
    West,
}

/// Paint a real building at the top-left `(x, y)` of an outer `w x h` footprint:
/// a `Wall` border around a `Floor` interior, with a single `Door` on the given
/// side. Returns the door tile. Refuses (returns `None`) if it is smaller than
/// 3x3 or would run off the map — the caller decides where it fits.
pub fn lay_building(
    terrain: &mut TerrainMap,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    door: Side,
) -> Option<(usize, usize)> {
    if w < 3 || h < 3 || x + w > terrain.width || y + h > terrain.height {
        return None;
    }
    for dy in 0..h {
        for dx in 0..w {
            let edge = dx == 0 || dy == 0 || dx == w - 1 || dy == h - 1;
            let t = if edge { Terrain::Wall } else { Terrain::Floor };
            terrain.set(x + dx, y + dy, t);
        }
    }
    // The doorway: a single gap centred on the chosen wall.
    let (ddx, ddy) = match door {
        Side::North => (w / 2, 0),
        Side::South => (w / 2, h - 1),
        Side::West => (0, h / 2),
        Side::East => (w - 1, h / 2),
    };
    terrain.set(x + ddx, y + ddy, Terrain::Door);
    Some((x + ddx, y + ddy))
}

/// A building placed in a district: its footprint and its door tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacedBuilding {
    pub style: BuildingStyle,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub door: (usize, usize),
}

/// Pick a fitting style for a plot, varied by the hash.
fn pick_style(h: u64, avail_w: usize, avail_h: usize) -> Option<BuildingStyle> {
    let fits: Vec<BuildingStyle> = [
        BuildingStyle::Hut,
        BuildingStyle::Cottage,
        BuildingStyle::Longhouse,
        BuildingStyle::Hall,
        BuildingStyle::Manor,
    ]
    .into_iter()
    .filter(|s| {
        let (w, hh) = s.size();
        w <= avail_w && hh <= avail_h
    })
    .collect();
    if fits.is_empty() {
        return None;
    }
    Some(fits[(h as usize) % fits.len()])
}

/// Lay a district of real buildings within an area (#458): the ground becomes
/// street (walkable Settlement), and varied buildings sit on plots with a
/// yard/street margin around each, every door opening onto a street. The river
/// keeps its bed. Deterministic per seed. Returns the placed buildings in
/// reading order, so callers can map services/occupants onto their doors.
pub fn lay_district(
    terrain: &mut TerrainMap,
    ax: usize,
    ay: usize,
    aw: usize,
    ah: usize,
    seed: u64,
) -> Vec<PlacedBuilding> {
    let (mw, mh) = (terrain.width, terrain.height);
    let aw = aw.min(mw.saturating_sub(ax));
    let ah = ah.min(mh.saturating_sub(ay));
    // The ground between buildings is the street — walkable. Water keeps its bed.
    for dy in 0..ah {
        for dx in 0..aw {
            let (tx, ty) = (ax + dx, ay + dy);
            if !matches!(terrain.get(tx, ty), Some(Terrain::Water | Terrain::Coast)) {
                terrain.set(tx, ty, Terrain::Settlement);
            }
        }
    }
    let stride = 9usize; // plot: a building (≤7) + a yard/street margin
    let mut out = Vec::new();
    let mut py = 0;
    while py + 4 <= ah {
        let mut px = 0;
        while px + 4 <= aw {
            let avail_w = (stride - 2).min(aw - px - 2);
            let avail_h = (stride - 2).min(ah - py - 2);
            let h = crate::rng::mix_u64(
                seed ^ (px as u64).wrapping_shl(20) ^ (py as u64).wrapping_shl(40),
            );
            // A scatter of plots stay open yards/gardens, not built on.
            if crate::rng::unit_from_hash(h.rotate_left(7)) < 0.18 {
                px += stride;
                continue;
            }
            if let Some(style) = pick_style(h, avail_w, avail_h) {
                let (bw, bh) = style.size();
                let (bx, by) = (ax + px + 1, ay + py + 1);
                let side = match h % 4 {
                    0 => Side::South,
                    1 => Side::North,
                    2 => Side::East,
                    _ => Side::West,
                };
                if let Some(door) = lay_building(terrain, bx, by, bw, bh, side) {
                    out.push(PlacedBuilding {
                        style,
                        x: bx,
                        y: by,
                        w: bw,
                        h: bh,
                        door,
                    });
                }
            }
            px += stride;
        }
        py += stride;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(w: usize, h: usize) -> TerrainMap {
        TerrainMap {
            width: w,
            height: h,
            tiles: vec![Terrain::Grass; w * h],
        }
    }

    #[test]
    fn a_building_has_walls_a_walkable_interior_and_one_door() {
        let mut t = blank(12, 12);
        let (dx, dy) = lay_building(&mut t, 2, 2, 5, 4, Side::South).expect("fits");
        // The door is on the south wall, passable.
        assert_eq!((dx, dy), (2 + 2, 2 + 3));
        assert_eq!(t.get(dx, dy), Some(Terrain::Door));
        assert!(Terrain::Door.passable());
        // The interior is floor and walkable.
        for iy in 3..5 {
            for ix in 3..6 {
                assert_eq!(t.get(ix, iy), Some(Terrain::Floor), "interior ({ix},{iy})");
                assert!(Terrain::Floor.passable());
            }
        }
        // The border is wall (and impassable) — except the one door.
        let mut walls = 0;
        let mut doors = 0;
        for ey in 2..6 {
            for ex in 2..7 {
                let edge = ex == 2 || ey == 2 || ex == 6 || ey == 5;
                if edge {
                    match t.get(ex, ey) {
                        Some(Terrain::Wall) => walls += 1,
                        Some(Terrain::Door) => doors += 1,
                        other => panic!("border ({ex},{ey}) is {other:?}"),
                    }
                }
            }
        }
        assert_eq!(doors, 1, "exactly one doorway");
        assert!(walls > 0 && !Terrain::Wall.passable());
    }

    #[test]
    fn a_building_refuses_to_run_off_the_map_or_be_too_small() {
        let mut t = blank(8, 8);
        assert!(lay_building(&mut t, 6, 6, 5, 5, Side::North).is_none());
        assert!(lay_building(&mut t, 0, 0, 2, 2, Side::North).is_none());
    }

    #[test]
    fn a_district_lays_varied_buildings_with_doors_onto_streets() {
        let mut t = blank(30, 24);
        let placed = lay_district(&mut t, 1, 1, 28, 22, 4242);
        assert!(
            placed.len() >= 3,
            "a district should hold several buildings"
        );
        // Buildings don't overlap (each interior floor belongs to one building).
        let mut seen = std::collections::HashSet::new();
        for b in &placed {
            for iy in (b.y + 1)..(b.y + b.h - 1) {
                for ix in (b.x + 1)..(b.x + b.w - 1) {
                    assert!(seen.insert((ix, iy)), "buildings overlap at ({ix},{iy})");
                    assert_eq!(t.get(ix, iy), Some(Terrain::Floor));
                }
            }
        }
        // Every door is reachable: at least one passable street tile adjoins it.
        for b in &placed {
            let (dx, dy) = b.door;
            assert_eq!(t.get(dx, dy), Some(Terrain::Door));
            let adj_walkable = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)]
                .iter()
                .any(|(ox, oy)| {
                    let (nx, ny) = (dx as i32 + ox, dy as i32 + oy);
                    nx >= 0
                        && ny >= 0
                        && t.get(nx as usize, ny as usize)
                            .is_some_and(|tt| tt == Terrain::Settlement)
                });
            assert!(adj_walkable, "door at ({dx},{dy}) opens onto a street");
        }
    }

    #[test]
    fn a_district_is_deterministic() {
        let mut a = blank(30, 24);
        let mut b = blank(30, 24);
        let pa = lay_district(&mut a, 1, 1, 28, 22, 99);
        let pb = lay_district(&mut b, 1, 1, 28, 22, 99);
        assert_eq!(pa, pb);
        assert_eq!(a.tiles, b.tiles);
    }

    #[test]
    fn styles_have_sane_sizes() {
        for s in [
            BuildingStyle::Hut,
            BuildingStyle::Cottage,
            BuildingStyle::Longhouse,
            BuildingStyle::Hall,
            BuildingStyle::Manor,
        ] {
            let (w, h) = s.size();
            assert!(w >= 3 && h >= 3, "{} too small", s.name());
        }
    }
}
