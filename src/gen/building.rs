//! Real buildings on the one world map (#458): a structure is a wall border
//! around a walkable floor, with a doorway you walk in through — not a 1-tile
//! token. Styles vary in size, from a hut to a hall. The primitive (`lay_building`)
//! and the district layout (`district_buildings` / `lay_district`) the town
//! generator lays — load-bearing: every settlement is built from these.

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

/// How a people build, biasing the styles their settlements raise (#454): the
/// Tzäkhar raise grand carved halls, the Häl keep to modest canopy-floor huts,
/// the Khör to long herder-houses. `Plain` is the human default — an even mix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildCharacter {
    #[default]
    Plain,
    /// Favours the largest building that fits (the deep-stone Tzäkhar).
    Grand,
    /// Favours the smallest (the canopy-floor Häl).
    Modest,
    /// Favours the longhouse where it fits (the steppe Khör).
    Long,
}

impl BuildCharacter {
    /// The building character of a settlement's dominant people.
    pub fn from_people(people: &str) -> BuildCharacter {
        match crate::model::PeopleKind::from_name(people) {
            crate::model::PeopleKind::Tzakhar => BuildCharacter::Grand,
            crate::model::PeopleKind::Hal => BuildCharacter::Modest,
            crate::model::PeopleKind::Khor => BuildCharacter::Long,
            _ => BuildCharacter::Plain,
        }
    }
}

/// Pick a fitting style for a plot, varied by the hash and the people's
/// building character.
fn pick_style(
    h: u64,
    avail_w: usize,
    avail_h: usize,
    character: BuildCharacter,
) -> Option<BuildingStyle> {
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
    // A people's character leans the pick; the hash still varies it within the
    // lean so no two settlements are identical.
    match character {
        // Grand: usually the largest that fits, sometimes the next.
        BuildCharacter::Grand => {
            let from_top = (h as usize) % 2; // 0 or 1 back from the largest
            return fits.get(fits.len().saturating_sub(1 + from_top)).copied();
        }
        // Modest: usually the smallest, sometimes the next.
        BuildCharacter::Modest => {
            return fits.get(((h as usize) % 2).min(fits.len() - 1)).copied();
        }
        // Long: a longhouse when it fits, else the hash decides.
        BuildCharacter::Long => {
            if fits.contains(&BuildingStyle::Longhouse) && !h.is_multiple_of(3) {
                return Some(BuildingStyle::Longhouse);
            }
        }
        BuildCharacter::Plain => {}
    }
    Some(fits[(h as usize) % fits.len()])
}

/// The door tile of a building footprint on the given side — the same gap
/// `lay_building` would cut. Pure: lets the layout be computed without painting.
fn building_door(x: usize, y: usize, w: usize, h: usize, side: Side) -> (usize, usize) {
    match side {
        Side::North => (x + w / 2, y),
        Side::South => (x + w / 2, y + h - 1),
        Side::West => (x, y + h / 2),
        Side::East => (x + w - 1, y + h / 2),
    }
}

/// The central market plaza of a district, as `(ox, oy, w, h)` offsets within
/// an `aw`×`ah` area (#458) — or `None` for a holding too small to spare one.
/// The single source of truth: the building generator keeps structures out of
/// it, and the day-time townsfolk gather in it (`npc_street_positions`).
pub fn central_plaza(aw: usize, ah: usize) -> Option<(usize, usize, usize, usize)> {
    if aw.min(ah) < 16 {
        return None;
    }
    let pw = (aw / 3).clamp(4, aw.saturating_sub(2));
    let ph = (ah / 4).clamp(3, ah.saturating_sub(2));
    Some(((aw - pw) / 2, (ah - ph) / 2, pw, ph))
}

/// Compute (but do not paint) the buildings of a district within an area (#458):
/// varied structures on plots with a yard/street margin, each door onto a
/// street. The single source of truth all consumers read — worldgen paints the
/// same buildings via `lay_district`, so service-doors, walls, and NPC streets
/// always agree. Adaptive: small holdings use small plots so even a hamlet gets
/// real buildings with doors. Deterministic per seed; reading order.
pub fn district_buildings(
    ax: usize,
    ay: usize,
    aw: usize,
    ah: usize,
    seed: u64,
    character: BuildCharacter,
) -> Vec<PlacedBuilding> {
    let mut out = Vec::new();
    if aw < 3 || ah < 3 {
        return out;
    }
    // A small holding packs tight (huts on small plots); a real town spreads
    // (varied buildings, the odd open yard). The stride is the plot pitch:
    // building plus a one-tile street, so it scales with the district.
    let span = aw.min(ah);
    let small = span <= 12;
    let stride = if span <= 10 {
        4usize
    } else if span <= 24 {
        6
    } else {
        9
    };
    // A real town reads around an open heart: reserve a central market plaza
    // (it stays walkable Settlement — `lay_district` paints the ground street;
    // we simply keep buildings out of it). Hamlets are too small to spare one.
    let plaza = central_plaza(aw, ah).map(|(ox, oy, w, h)| (ax + ox, ay + oy, w, h));
    // And a real town has a spine: a main street runs the length of the
    // district through the plaza, a clear thoroughfare the lanes branch off
    // (it stays walkable Settlement, like the plaza and the side streets).
    let main_street = if span >= 16 {
        let msw = if span >= 28 { 3 } else { 2 };
        Some((ax + aw / 2 - msw / 2, ay, msw, ah))
    } else {
        None
    };
    // A cross-street meets the main street at the plaza, so a real town reads
    // as a crossroads — a way in from either flank, the quarters set between
    // the four arms. (Walkable Settlement, like the spine.)
    let cross_street = if span >= 16 {
        let csh = if span >= 28 { 3 } else { 2 };
        Some((ax, ay + ah / 2 - csh / 2, aw, csh))
    } else {
        None
    };
    let reserved = [plaza, main_street, cross_street];
    let mut py = 0;
    while py + 4 <= ah {
        let mut px = 0;
        while px + 4 <= aw {
            // Keep the plaza and the main street clear of buildings.
            let (lx, ly) = (ax + px, ay + py);
            if reserved.iter().flatten().any(|&(qx, qy, qw, qh)| {
                lx < qx + qw && lx + stride > qx && ly < qy + qh && ly + stride > qy
            }) {
                px += stride;
                continue;
            }
            // Building fills the plot bar a one-tile street; the +1 inset
            // already leaves a street on the north and west.
            let avail_w = (stride - 1).min(aw - px - 1);
            let avail_h = (stride - 1).min(ah - py - 1);
            let h = crate::rng::mix_u64(
                seed ^ (px as u64).wrapping_shl(20) ^ (py as u64).wrapping_shl(40),
            );
            // In a real town, a scatter of plots stay open yards/gardens.
            if !small && crate::rng::unit_from_hash(h.rotate_left(7)) < 0.18 {
                px += stride;
                continue;
            }
            if let Some(style) = pick_style(h, avail_w, avail_h, character) {
                let (bw, bh) = style.size();
                let (bx, by) = (ax + px + 1, ay + py + 1);
                let side = match h % 4 {
                    0 => Side::South,
                    1 => Side::North,
                    2 => Side::East,
                    _ => Side::West,
                };
                out.push(PlacedBuilding {
                    style,
                    x: bx,
                    y: by,
                    w: bw,
                    h: bh,
                    door: building_door(bx, by, bw, bh, side),
                });
            }
            px += stride;
        }
        py += stride;
    }
    // The tiniest holdings (a steading) still get one dwelling with a door.
    if out.is_empty() && aw >= 3 && ah >= 3 {
        let (bw, bh) = (3usize.min(aw), 3usize.min(ah));
        out.push(PlacedBuilding {
            style: BuildingStyle::Hut,
            x: ax,
            y: ay,
            w: bw,
            h: bh,
            door: building_door(ax, ay, bw, bh, Side::South),
        });
    }
    out
}

/// Lay a district of real buildings within an area (#458): the ground becomes
/// street (walkable Settlement), and the `district_buildings` are painted as
/// walls/floor/door. The river keeps its bed. Returns the placed buildings.
pub fn lay_district(
    terrain: &mut TerrainMap,
    ax: usize,
    ay: usize,
    aw: usize,
    ah: usize,
    seed: u64,
    character: BuildCharacter,
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
    let buildings = district_buildings(ax, ay, aw, ah, seed, character);
    for b in &buildings {
        for dy in 0..b.h {
            for dx in 0..b.w {
                let edge = dx == 0 || dy == 0 || dx == b.w - 1 || dy == b.h - 1;
                terrain.set(
                    b.x + dx,
                    b.y + dy,
                    if edge { Terrain::Wall } else { Terrain::Floor },
                );
            }
        }
        terrain.set(b.door.0, b.door.1, Terrain::Door);
        // A hearth at the heart of the room — every building has its fire.
        terrain.set(b.x + b.w / 2, b.y + b.h / 2, Terrain::Hearth);
    }
    buildings
}

/// A deterministic seed for a town's layout, from its map anchor — so worldgen
/// (which paints) and the consumers (which recompute the same buildings) always
/// agree without storing the layout.
pub fn town_seed(map_x: u32, map_y: u32) -> u64 {
    crate::rng::mix_u64(
        (map_x as u64).wrapping_shl(20) ^ (map_y as u64).wrapping_shl(40) ^ 0x70_11_AA_BB,
    )
}

/// Lay a single rural homestead (#458): a dwelling and an outbuilding around a
/// trodden yard, with a worked field beside them — the scattered holdings of
/// the open country, not a town. The yard is walkable (Settlement); the field
/// is Farmland. Deterministic per seed. Returns the placed buildings, the
/// dwelling first. Refuses (empty) if the patch won't fit.
pub fn lay_homestead(
    terrain: &mut TerrainMap,
    ax: usize,
    ay: usize,
    seed: u64,
) -> Vec<PlacedBuilding> {
    const AW: usize = 14;
    const AH: usize = 11;
    if ax + AW > terrain.width || ay + AH > terrain.height {
        return Vec::new();
    }
    let h = crate::rng::mix_u64(seed ^ 0x40E5_7EAD);
    // The yard: trodden ground around the buildings (water keeps its bed).
    for dy in 0..AH {
        for dx in 0..AW {
            let (tx, ty) = (ax + dx, ay + dy);
            if !matches!(terrain.get(tx, ty), Some(Terrain::Water | Terrain::Coast)) {
                terrain.set(tx, ty, Terrain::Settlement);
            }
        }
    }
    // The field: a worked strip along the bottom.
    for dy in 8..AH {
        for dx in 1..(AW - 1) {
            let (tx, ty) = (ax + dx, ay + dy);
            if !matches!(terrain.get(tx, ty), Some(Terrain::Water | Terrain::Coast)) {
                terrain.set(tx, ty, Terrain::Farmland);
            }
        }
    }
    let mut out = Vec::new();
    // The dwelling: a cottage or, on a prosperous holding, a longhouse.
    let dwelling = if h.is_multiple_of(2) {
        BuildingStyle::Cottage
    } else {
        BuildingStyle::Longhouse
    };
    let (dw, dh) = dwelling.size();
    if let Some(door) = lay_building(terrain, ax + 1, ay + 1, dw, dh, Side::South) {
        out.push(PlacedBuilding {
            style: dwelling,
            x: ax + 1,
            y: ay + 1,
            w: dw,
            h: dh,
            door,
        });
    }
    // The outbuilding: a barn/shed, set apart across the yard.
    let (sw, sh) = BuildingStyle::Hut.size();
    if let Some(door) = lay_building(terrain, ax + 9, ay + 2, sw, sh, Side::West) {
        out.push(PlacedBuilding {
            style: BuildingStyle::Hut,
            x: ax + 9,
            y: ay + 2,
            w: sw,
            h: sh,
            door,
        });
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
        let placed = lay_district(&mut t, 1, 1, 28, 22, 4242, BuildCharacter::Plain);
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
                    // Interior is walkable floor, but for the one hearth tile
                    // at the heart of the room.
                    assert!(
                        matches!(t.get(ix, iy), Some(Terrain::Floor | Terrain::Hearth)),
                        "interior ({ix},{iy}) is floor or hearth"
                    );
                }
            }
        }
        // Every door is reachable: a walkable tile adjoins it — the inner
        // street (Settlement) or the ground beyond the district edge (Grass
        // here; the farmland skirt / road in the real world). What it must
        // never do is open straight into a wall with no way through.
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
                            .is_some_and(|tt| matches!(tt, Terrain::Settlement | Terrain::Grass))
                });
            assert!(
                adj_walkable,
                "door at ({dx},{dy}) opens onto walkable ground"
            );
        }
    }

    #[test]
    fn every_building_has_a_hearth_at_its_heart() {
        let mut t = blank(30, 24);
        let placed = lay_district(&mut t, 1, 1, 28, 22, 4242, BuildCharacter::Plain);
        for b in &placed {
            let (hx, hy) = (b.x + b.w / 2, b.y + b.h / 2);
            assert_eq!(
                t.get(hx, hy),
                Some(Terrain::Hearth),
                "the building at ({},{}) has a hearth at its heart",
                b.x,
                b.y
            );
            assert!(Terrain::Hearth.passable(), "you stand by the hearth");
        }
    }

    #[test]
    fn a_large_town_keeps_a_central_market_plaza_clear() {
        // A real town (span >= 16) reserves an open heart; buildings keep out.
        let (ax, ay, aw, ah) = (0usize, 0usize, 40usize, 36usize);
        let placed = district_buildings(ax, ay, aw, ah, 31, BuildCharacter::Plain);
        assert!(!placed.is_empty(), "a town has buildings around its plaza");
        // The plaza rect, computed exactly as the generator does.
        let pw = (aw / 3).clamp(4, aw - 2);
        let ph = (ah / 4).clamp(3, ah - 2);
        let (qx, qy) = (ax + (aw - pw) / 2, ay + (ah - ph) / 2);
        for b in &placed {
            let overlaps = b.x < qx + pw && b.x + b.w > qx && b.y < qy + ph && b.y + b.h > qy;
            assert!(
                !overlaps,
                "building at ({},{}) {}x{} intrudes on the plaza [{qx},{qy} {pw}x{ph}]",
                b.x, b.y, b.w, b.h
            );
        }
    }

    #[test]
    fn a_large_town_keeps_a_main_street_clear() {
        // A real town's central spine stays free of buildings, full height.
        let (ax, ay, aw, ah) = (0usize, 0usize, 40usize, 36usize);
        let placed = district_buildings(ax, ay, aw, ah, 31, BuildCharacter::Plain);
        let msw = if aw.min(ah) >= 28 { 3 } else { 2 };
        let msx = ax + aw / 2 - msw / 2;
        for b in &placed {
            let overlaps = b.x < msx + msw && b.x + b.w > msx && b.y < ay + ah && b.y + b.h > ay;
            assert!(
                !overlaps,
                "building at ({},{}) {}x{} blocks the main street [x {msx}..{}]",
                b.x,
                b.y,
                b.w,
                b.h,
                msx + msw
            );
        }
    }

    #[test]
    fn a_large_town_keeps_a_cross_street_clear() {
        // The cross-street spans the town's width, free of buildings.
        let (ax, ay, aw, ah) = (0usize, 0usize, 40usize, 36usize);
        let placed = district_buildings(ax, ay, aw, ah, 31, BuildCharacter::Plain);
        let csh = if aw.min(ah) >= 28 { 3 } else { 2 };
        let csy = ay + ah / 2 - csh / 2;
        for b in &placed {
            let overlaps = b.y < csy + csh && b.y + b.h > csy && b.x < ax + aw && b.x + b.w > ax;
            assert!(
                !overlaps,
                "building at ({},{}) {}x{} blocks the cross-street [y {csy}..{}]",
                b.x,
                b.y,
                b.w,
                b.h,
                csy + csh
            );
        }
    }

    #[test]
    fn a_hamlet_is_too_small_for_a_plaza() {
        // A small holding (span < 16) packs tight — no reserved plaza, so the
        // few plots it has still get their buildings.
        let placed = district_buildings(0, 0, 12, 12, 31, BuildCharacter::Plain);
        assert!(!placed.is_empty(), "a hamlet still raises its huts");
    }

    #[test]
    fn a_district_is_deterministic() {
        let mut a = blank(30, 24);
        let mut b = blank(30, 24);
        let pa = lay_district(&mut a, 1, 1, 28, 22, 99, BuildCharacter::Plain);
        let pb = lay_district(&mut b, 1, 1, 28, 22, 99, BuildCharacter::Plain);
        assert_eq!(pa, pb);
        assert_eq!(a.tiles, b.tiles);
    }

    #[test]
    fn building_character_leans_the_styles() {
        let area = |bs: &[PlacedBuilding]| -> f64 {
            if bs.is_empty() {
                return 0.0;
            }
            bs.iter().map(|b| (b.w * b.h) as f64).sum::<f64>() / bs.len() as f64
        };
        // Same ground, same seed — only the people's character differs.
        let grand = district_buildings(0, 0, 40, 40, 7, BuildCharacter::Grand);
        let modest = district_buildings(0, 0, 40, 40, 7, BuildCharacter::Modest);
        let long = district_buildings(0, 0, 40, 40, 7, BuildCharacter::Long);
        assert!(
            area(&grand) > area(&modest),
            "grand builders raise larger buildings than modest ones ({} vs {})",
            area(&grand),
            area(&modest)
        );
        // The Long build longhouses where they fit.
        assert!(
            long.iter()
                .filter(|b| b.style == BuildingStyle::Longhouse)
                .count()
                > grand
                    .iter()
                    .filter(|b| b.style == BuildingStyle::Longhouse)
                    .count(),
            "the Long raise more longhouses than the Grand"
        );
        // Character keeps determinism.
        assert_eq!(
            grand,
            district_buildings(0, 0, 40, 40, 7, BuildCharacter::Grand)
        );
    }

    #[test]
    fn from_people_reads_the_five() {
        assert_eq!(
            BuildCharacter::from_people("tzäkhar"),
            BuildCharacter::Grand
        );
        assert_eq!(BuildCharacter::from_people("häl"), BuildCharacter::Modest);
        assert_eq!(BuildCharacter::from_people("khör"), BuildCharacter::Long);
        assert_eq!(BuildCharacter::from_people("metsik"), BuildCharacter::Plain);
    }

    #[test]
    fn a_homestead_has_a_dwelling_an_outbuilding_and_a_field() {
        let mut t = blank(20, 16);
        let placed = lay_homestead(&mut t, 2, 2, 7);
        assert_eq!(placed.len(), 2, "a dwelling and an outbuilding");
        // Both buildings are real (walls + a door).
        for b in &placed {
            assert_eq!(t.get(b.door.0, b.door.1), Some(Terrain::Door));
        }
        // There is worked field in the holding.
        let mut field = 0;
        for y in 2..18 {
            for x in 2..16 {
                if t.get(x, y) == Some(Terrain::Farmland) {
                    field += 1;
                }
            }
        }
        assert!(field > 0, "a homestead has a worked field");
    }

    #[test]
    fn a_homestead_refuses_to_run_off_the_map() {
        let mut t = blank(10, 8);
        assert!(lay_homestead(&mut t, 0, 0, 1).is_empty());
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
