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
