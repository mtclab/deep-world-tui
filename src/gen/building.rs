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
    /// A lean-to / tarp — the barest shelter, the poorest and the passing-through.
    LeanTo,
    /// A tent — a cloth dwelling, nomads and the poor.
    Tent,
    /// A one-room hut.
    Hut,
    /// A stable — beasts below, the hand who tends them above.
    Stable,
    /// A modest cottage.
    Cottage,
    /// A long, narrow longhouse.
    Longhouse,
    /// A broad hall (tavern, temple, mead-hall).
    Hall,
    /// A large manor / works — the seat of the well-off.
    Manor,
}

impl BuildingStyle {
    /// Outer footprint (w, h) in tiles, walls included.
    pub fn size(self) -> (usize, usize) {
        // Outer footprint incl. walls. Interiors are (w-2)x(h-2) walkable Floor
        // — sized so every building is a real room you walk into (Fallout 1/2 /
        // Gothic: enter through the door, stay on the one map), not a 1-cell hut.
        // The lesser shelters (lean-to, tent) are small by nature — a bedroll,
        // not a hall — but still a walkable space with a way in.
        match self {
            BuildingStyle::LeanTo => (3, 3),
            BuildingStyle::Tent => (3, 4),
            BuildingStyle::Hut => (5, 5),
            BuildingStyle::Stable => (6, 5),
            BuildingStyle::Cottage => (6, 6),
            BuildingStyle::Longhouse => (5, 8),
            BuildingStyle::Hall => (9, 9),
            BuildingStyle::Manor => (11, 12),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BuildingStyle::LeanTo => "lean-to",
            BuildingStyle::Tent => "tent",
            BuildingStyle::Hut => "hut",
            BuildingStyle::Stable => "stable",
            BuildingStyle::Cottage => "cottage",
            BuildingStyle::Longhouse => "longhouse",
            BuildingStyle::Hall => "hall",
            BuildingStyle::Manor => "manor",
        }
    }

    /// Whether this style is a lesser shelter (no hearth, the poor's lodging).
    pub fn is_shelter(self) -> bool {
        matches!(self, BuildingStyle::LeanTo | BuildingStyle::Tent)
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

/// The organic shape a building's footprint takes inside its `w x h` plot —
/// real structures are not clean rectangles. Chosen deterministically per plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Footprint {
    /// A rectangle with the hard corners cut — the everyday irregular house.
    Chamfered,
    /// An elliptical longhouse / round dwelling (curved walls).
    Oval,
    /// An L: the main body plus a wing bitten out of one corner.
    Ell,
}

/// True where the `w x h` plot belongs to the building's organic footprint.
/// Convex (chamfered/oval) shapes keep the interior connected; the L stays
/// connected because the bite is a single corner quadrant.
fn footprint_mask(w: usize, h: usize, shape: Footprint, seed: u64) -> Vec<bool> {
    let mut inside = vec![true; w * h];
    let at = |dx: usize, dy: usize| dy * w + dx;
    match shape {
        Footprint::Chamfered => {
            // cut the corners; bigger buildings get a 2-cell chamfer
            let c = if w >= 7 && h >= 7 { 2 } else { 1 };
            for dy in 0..h {
                for dx in 0..w {
                    let cx = dx.min(w - 1 - dx);
                    let cy = dy.min(h - 1 - dy);
                    if cx + cy < c {
                        inside[at(dx, dy)] = false;
                    }
                }
            }
        }
        Footprint::Oval => {
            let (rx, ry) = ((w as f64 - 1.0) / 2.0, (h as f64 - 1.0) / 2.0);
            let (cx, cy) = (rx, ry);
            for dy in 0..h {
                for dx in 0..w {
                    let nx = (dx as f64 - cx) / rx.max(0.5);
                    let ny = (dy as f64 - cy) / ry.max(0.5);
                    if nx * nx + ny * ny > 1.05 {
                        inside[at(dx, dy)] = false;
                    }
                }
            }
        }
        Footprint::Ell => {
            // bite a roughly-quarter rectangle out of one corner (seed picks which)
            let bw = (w / 2).max(1);
            let bh = (h / 2).max(1);
            let right = seed & 1 == 0;
            let bottom = seed & 2 == 0;
            for dy in 0..h {
                for dx in 0..w {
                    let in_x = if right { dx >= w - bw } else { dx < bw };
                    let in_y = if bottom { dy >= h - bh } else { dy < bh };
                    if in_x && in_y {
                        inside[at(dx, dy)] = false;
                    }
                }
            }
        }
    }
    inside
}

/// Paint a real building at the top-left `(x, y)` of an outer `w x h` plot: an
/// organic footprint (chamfered / oval / L, deterministic per plot — never a
/// clean box), drawn as a `Wall` ring around its `Floor` interior, with a single
/// `Door`. Cells outside the footprint are left as-is (a natural yard). Returns
/// the door tile. Refuses (`None`) below 3x3 or off the map.
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
    let seed = {
        // deterministic per plot; no new caller args needed
        let mut s = 0xcbf29ce484222325u64;
        for v in [x, y, w, h] {
            s = (s ^ v as u64).wrapping_mul(0x100000001b3);
        }
        s
    };
    let shape = match seed % 3 {
        0 => Footprint::Chamfered,
        1 => Footprint::Oval,
        _ => Footprint::Ell,
    };
    let inside = footprint_mask(w, h, shape, seed);
    let at = |dx: usize, dy: usize| dy * w + dx;
    let is_in = |dx: i64, dy: i64| {
        dx >= 0
            && dy >= 0
            && (dx as usize) < w
            && (dy as usize) < h
            && inside[at(dx as usize, dy as usize)]
    };

    let mut floor_count = 0usize;
    for dy in 0..h {
        for dx in 0..w {
            if !inside[at(dx, dy)] {
                continue; // outside the organic footprint -> left as yard
            }
            // a wall where the footprint meets the outside (or the plot edge)
            let boundary = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(ox, oy)| !is_in(dx as i64 + ox, dy as i64 + oy));
            let t = if boundary {
                Terrain::Wall
            } else {
                floor_count += 1;
                Terrain::Floor
            };
            terrain.set(x + dx, y + dy, t);
        }
    }

    // Tiny or thin shapes can carve away the whole interior — fall back to the
    // plain rectangle so the building is always walkable.
    if floor_count == 0 {
        for dy in 0..h {
            for dx in 0..w {
                let edge = dx == 0 || dy == 0 || dx == w - 1 || dy == h - 1;
                terrain.set(
                    x + dx,
                    y + dy,
                    if edge { Terrain::Wall } else { Terrain::Floor },
                );
            }
        }
    }

    // The doorway: the boundary wall on the chosen side that has interior floor
    // just inside it (so you always walk straight into the room).
    let door_tile = place_door(terrain, x, y, w, h, door);
    Some(door_tile)
}

/// Set a `Door` on the chosen side: scan that edge from its midpoint outward for
/// a `Wall` cell with a `Floor` immediately inside, and open it. Falls back to
/// the side's centre.
fn place_door(
    terrain: &mut TerrainMap,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    side: Side,
) -> (usize, usize) {
    let floor_inside = |tx: usize, ty: usize, ix: i64, iy: i64| -> bool {
        let (nx, ny) = (tx as i64 + ix, ty as i64 + iy);
        nx >= 0
            && ny >= 0
            && (nx as usize) < terrain.width
            && (ny as usize) < terrain.height
            && terrain.get(nx as usize, ny as usize) == Some(Terrain::Floor)
    };
    // candidate offsets along the side, nearest the centre first
    let span = |n: usize| {
        let mid = n / 2;
        (0..n).map(move |k| {
            if k % 2 == 0 {
                mid + k / 2
            } else {
                mid.saturating_sub(k / 2 + 1)
            }
        })
    };
    let mut found: Option<(usize, usize)> = None;
    match side {
        Side::North | Side::South => {
            let ty = if side == Side::North { y } else { y + h - 1 };
            let iy = if side == Side::North { 1 } else { -1 };
            for dx in span(w) {
                let tx = x + dx;
                if terrain.get(tx, ty) == Some(Terrain::Wall) && floor_inside(tx, ty, 0, iy) {
                    found = Some((tx, ty));
                    break;
                }
            }
        }
        Side::West | Side::East => {
            let tx = if side == Side::West { x } else { x + w - 1 };
            let ix = if side == Side::West { 1 } else { -1 };
            for dy in span(h) {
                let ty = y + dy;
                if terrain.get(tx, ty) == Some(Terrain::Wall) && floor_inside(tx, ty, ix, 0) {
                    found = Some((tx, ty));
                    break;
                }
            }
        }
    }
    let (ddx, ddy) = found.unwrap_or(match side {
        Side::North => (x + w / 2, y),
        Side::South => (x + w / 2, y + h - 1),
        Side::West => (x, y + h / 2),
        Side::East => (x + w - 1, y + h / 2),
    });
    terrain.set(ddx, ddy, Terrain::Door);
    (ddx, ddy)
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
        // Grand: the largest *broad* building that fits — halls and manors, not
        // the narrow longhouse (that is the Khör's Long character).
        BuildCharacter::Grand => {
            let broad: Vec<BuildingStyle> = fits
                .iter()
                .copied()
                .filter(|s| *s != BuildingStyle::Longhouse)
                .collect();
            let pool = if broad.is_empty() { &fits } else { &broad };
            let from_top = (h as usize) % 2; // 0 or 1 back from the largest
            return pool.get(pool.len().saturating_sub(1 + from_top)).copied();
        }
        // Modest: the smallest that fits — the canopy-floor Häl keep it humble.
        BuildCharacter::Modest => {
            return fits.first().copied();
        }
        // Long: a longhouse when it fits, else the hash decides.
        BuildCharacter::Long => {
            if fits.contains(&BuildingStyle::Longhouse) && !h.is_multiple_of(3) {
                return Some(BuildingStyle::Longhouse);
            }
        }
        // Plain: lean to the larger half that fits, so a town reads as real
        // buildings (halls, cottages) — varied within the lean by the hash.
        BuildCharacter::Plain => {
            let lo = fits.len() / 2;
            return fits.get(lo + (h as usize) % (fits.len() - lo)).copied();
        }
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
    if aw.min(ah) < 28 {
        return None;
    }
    let pw = (aw / 3).clamp(4, aw.saturating_sub(2));
    let ph = (ah / 4).clamp(3, ah.saturating_sub(2));
    Some(((aw - pw) / 2, (ah - ph) / 2, pw, ph))
}

/// The four furnishing glyphs (table, chest, bed-pallet, shelf) a people raise,
/// keyed to their building character (#458 per-people texture). All single-width
/// (box-drawing / geometric-shape / math), so the map grid stays true.
fn furnishing_glyphs(character: BuildCharacter) -> [char; 4] {
    match character {
        // Carved deep-stone: heavy slab table, stone store, plinth bed, niche.
        BuildCharacter::Grand => ['╦', '▤', '▬', '▦'],
        // Canopy-floor woven: light board, basket, woven mat, hanging net.
        BuildCharacter::Modest => ['┬', '▫', '~', '≋'],
        // Steppe herder: trestle, rolled store, hide pallet, rack.
        BuildCharacter::Long => ['╤', '▱', '∾', '≡'],
        // The even human hearth.
        BuildCharacter::Plain => ['╤', '▪', '=', '≡'],
    }
}

/// The furnishings inside a building (#458 interiors): a few pieces — table,
/// chest, bed-pallet, shelf — placed deterministically on the interior floor so
/// a building reads as a lived-in room, not an empty box. Returns
/// `(x, y, glyph)` in absolute tile coords; never the hearth at the heart, the
/// walls, or the doorway. Single-width glyphs only (so the map grid stays true).
/// Bigger rooms hold more. The renderer paints these over interior floor tiles.
pub fn building_furnishings(
    b: &PlacedBuilding,
    seed: u64,
    character: BuildCharacter,
) -> Vec<(usize, usize, char)> {
    let mut out = Vec::new();
    if b.w < 3 || b.h < 3 {
        return out;
    }
    let (hx, hy) = (b.x + b.w / 2, b.y + b.h / 2); // the hearth, kept clear
                                                   // Candidate spots: the four interior corners (inset one from the walls),
                                                   // each with its own piece of furniture. The glyph set varies with the
                                                   // people's building character, so a Tzäkhar deephold reads of carved
                                                   // stone, a Häl canopy-floor of woven things, a Khör longhouse of hides.
    let g = furnishing_glyphs(character);
    let spots = [
        (b.x + 1, b.y + 1, g[0]),             // a table / worktop
        (b.x + b.w - 2, b.y + 1, g[1]),       // a chest / store
        (b.x + 1, b.y + b.h - 2, g[2]),       // a bed-pallet
        (b.x + b.w - 2, b.y + b.h - 2, g[3]), // shelves
    ];
    let area = b.w.saturating_sub(2) * b.h.saturating_sub(2);
    let n = (area / 4).clamp(1, spots.len());
    let h =
        crate::rng::mix_u64(seed ^ (b.x as u64).wrapping_shl(20) ^ (b.y as u64).wrapping_shl(40));
    // Rotate which corner the furnishing run starts at, so not every room is
    // laid out identically; deterministic per building.
    let start = (h % spots.len() as u64) as usize;
    for k in 0..n {
        let (fx, fy, g) = spots[(start + k) % spots.len()];
        if (fx, fy) == (hx, hy) {
            continue; // never over the hearth
        }
        if fx > b.x && fx < b.x + b.w - 1 && fy > b.y && fy < b.y + b.h - 1 && (fx, fy) != b.door {
            out.push((fx, fy, g));
        }
    }
    out
}

/// The width of the wall border a great town keeps clear at its footprint edge
/// (#458/#449): `0` for an unwalled town, else a wall ring plus a walkable lane
/// (the pomerium) just inside it, so buildings never front the wall and their
/// outward doors open onto street. Shared geometry: `district_buildings` keeps
/// buildings out of it; `lay_town` raises the wall on it.
pub fn wall_border(aw: usize, ah: usize) -> usize {
    if aw.min(ah) >= 28 {
        2
    } else {
        0
    }
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
    // Plot pitch = building + a one-tile street. Sized so a real room fits each
    // plot: small holdings still get a 5x5 (3x3 floor) dwelling, towns get
    // halls/manors. Bigger than the old 4-tile pitch, which only ever fit huts.
    // Plot pitch = building + a one-tile street. 6 fits Huts (5x5) and Cottages
    // (6x6) — real walkable rooms — through villages and ordinary towns without
    // crowding the central plaza/streets; cities widen it so grand Halls (9x9)
    // and Manors (11x12) can stand.
    // Buildings are packed at their REAL sizes (variable plots), not on a uniform
    // grid — so a town is hundreds of small family homes with the odd hall or
    // longhouse among them, dense and varied, not a checkerboard of identical
    // huts (owner: living/organic world, households + ties).
    // A real town reads around an open heart: reserve a central market plaza
    // (it stays walkable Settlement — `lay_district` paints the ground street;
    // we simply keep buildings out of it). Hamlets are too small to spare one.
    let plaza = central_plaza(aw, ah).map(|(ox, oy, w, h)| (ax + ox, ay + oy, w, h));
    // And a real town has a spine: a main street runs the length of the
    // district through the plaza, a clear thoroughfare the lanes branch off
    // (it stays walkable Settlement, like the plaza and the side streets).
    let main_street = if span >= 28 {
        let msw = 3;
        Some((ax + aw / 2 - msw / 2, ay, msw, ah))
    } else {
        None
    };
    // A cross-street meets the main street at the plaza, so a real town reads
    // as a crossroads — a way in from either flank, the quarters set between
    // the four arms. (Walkable Settlement, like the spine.)
    let cross_street = if span >= 28 {
        let csh = 3;
        Some((ax, ay + ah / 2 - csh / 2, aw, csh))
    } else {
        None
    };
    let reserved = [plaza, main_street, cross_street];
    // A great town keeps a clear border at its edge — the wall ring and the
    // lane just inside it — so no building fronts the wall. The plot grid
    // shifts in by `border-1` and its far side is capped by `border`, so the
    // buildings sit a clear lane inside the wall `lay_town` raises here; both
    // read the same `wall_border`, so painted and computed agree.
    let border = wall_border(aw, ah);
    let far = border.max(1);
    let start = border.saturating_sub(1);
    let mut py = start;
    // Row-by-row VARIABLE packing: place each building at its real size, then step
    // past it (building + a one-tile street), so a row holds many small homes and
    // the odd larger one tight together — real density, varied, never a grid.
    while py + 4 <= ah.saturating_sub(far) {
        let mut px = start;
        let mut row_h = 4usize; // how far down the next row starts
        while px + 4 <= aw.saturating_sub(far) {
            let (lx, ly) = (ax + px, ay + py);
            let h = crate::rng::mix_u64(
                seed ^ (px as u64).wrapping_shl(20) ^ (py as u64).wrapping_shl(40),
            );
            // Plaza / main / cross streets stay clear: jump past the block.
            let mut jumped = false;
            for &(qx, qy, qw, qh) in reserved.iter().flatten() {
                if lx >= qx && lx < qx + qw && ly >= qy && ly < qy + qh {
                    px = (qx + qw).saturating_sub(ax).max(px + 1);
                    jumped = true;
                    break;
                }
            }
            if jumped {
                continue;
            }
            // A scatter of plots stay open yards/gardens.
            if !small && crate::rng::unit_from_hash(h.rotate_left(7)) < 0.15 {
                px += 2;
                continue;
            }
            // Room left for THIS plot: building + its one-tile inset, capped a lane
            // inside the district's far edge.
            let avail_w = aw.saturating_sub(far).saturating_sub(px + 1);
            let avail_h = ah.saturating_sub(far).saturating_sub(py + 1);
            if avail_w < 3 || avail_h < 3 {
                break;
            }
            if let Some(style) = pick_style(h, avail_w, avail_h, character) {
                // The common dwellings (hut/cottage) become the poor's lesser
                // shelter for a share of plots — a tent or a bare lean-to — and the
                // odd holding keeps a stable. Grand styles stay the seats of those
                // who can raise them. Deterministic per plot.
                let chosen = if matches!(style, BuildingStyle::Hut | BuildingStyle::Cottage) {
                    let lot = crate::rng::unit_from_hash(h.rotate_left(11));
                    if lot < 0.12 {
                        BuildingStyle::LeanTo
                    } else if lot < 0.30 {
                        BuildingStyle::Tent
                    } else if lot > 0.90 {
                        let (sw, sh) = BuildingStyle::Stable.size();
                        if sw <= avail_w && sh <= avail_h {
                            BuildingStyle::Stable
                        } else {
                            style
                        }
                    } else {
                        style
                    }
                } else {
                    style
                };
                let (bw, bh) = chosen.size();
                let (bx, by) = (ax + px + 1, ay + py + 1);
                // Don't let a wide building spill into a reserved street.
                let clips = reserved.iter().flatten().any(|&(qx, qy, qw, qh)| {
                    bx < qx + qw && bx + bw > qx && by < qy + qh && by + bh > qy
                });
                if clips {
                    px += 1;
                    continue;
                }
                let side = match h % 4 {
                    0 => Side::South,
                    1 => Side::North,
                    2 => Side::East,
                    _ => Side::West,
                };
                out.push(PlacedBuilding {
                    style: chosen,
                    x: bx,
                    y: by,
                    w: bw,
                    h: bh,
                    door: building_door(bx, by, bw, bh, side),
                });
                px += bw + 2; // inset(1) + building + street(1)
                row_h = row_h.max(bh + 2);
            } else {
                px += 1;
            }
        }
        py += row_h;
    }
    // The tiniest holdings (a steading) still get one dwelling with a door —
    // inset one tile from the anchor (like every plot building) so the anchor
    // stays walkable street (the homestead ground), not a wall, and sized as
    // big as the patch allows (up to a 5x5 room), never below 3x3.
    if out.is_empty() && aw >= 4 && ah >= 4 {
        let (bw, bh) = ((aw - 1).clamp(3, 5), (ah - 1).clamp(3, 5));
        let (bx, by) = (ax + 1, ay + 1);
        out.push(PlacedBuilding {
            style: BuildingStyle::Hut,
            x: bx,
            y: by,
            w: bw,
            h: bh,
            door: building_door(bx, by, bw, bh, Side::South),
        });
    } else if out.is_empty() && aw >= 3 && ah >= 3 {
        // A patch too small to inset: a bare 3x3 shelter on the spot.
        out.push(PlacedBuilding {
            style: BuildingStyle::Hut,
            x: ax,
            y: ay,
            w: 3,
            h: 3,
            door: building_door(ax, ay, 3, 3, Side::South),
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
        // A hearth at the heart of the room — but only where the room is big
        // enough to spare the tile, so the interior keeps its walkable Floor
        // (a 1-cell hut would otherwise be all hearth, no floor).
        if b.w > 3 && b.h > 3 {
            terrain.set(b.x + b.w / 2, b.y + b.h / 2, Terrain::Hearth);
        }
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
        // Footprints are now organic (chamfered/oval/L), not clean boxes; assert
        // the invariants that hold for any shape, across positions + sizes (so all
        // three shapes are exercised) + every door side.
        for (px, py, w, h) in [(2, 2, 5, 4), (1, 1, 6, 6), (3, 2, 9, 5), (2, 3, 7, 8)] {
            for side in [Side::North, Side::South, Side::East, Side::West] {
                let mut t = blank(20, 20);
                let (dx, dy) = lay_building(&mut t, px, py, w, h, side).expect("fits");
                // The door is a passable Door, the only one in the plot, with a
                // Floor immediately inside (you walk straight in).
                assert_eq!(
                    t.get(dx, dy),
                    Some(Terrain::Door),
                    "door tile at {px},{py} {side:?}"
                );
                assert!(Terrain::Door.passable());
                let mut floors = 0;
                let mut walls = 0;
                let mut doors = 0;
                for ey in py..py + h {
                    for ex in px..px + w {
                        match t.get(ex, ey) {
                            Some(Terrain::Floor) => floors += 1,
                            Some(Terrain::Wall) => walls += 1,
                            Some(Terrain::Door) => doors += 1,
                            _ => {} // cut corner -> yard, fine
                        }
                    }
                }
                assert!(floors > 0, "walkable interior at {px},{py} {w}x{h}");
                assert!(walls > 0 && !Terrain::Wall.passable());
                assert_eq!(doors, 1, "exactly one doorway at {px},{py} {side:?}");
                // the door opens onto a floor (reachable interior)
                let touches_floor = [(-1i64, 0), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .any(|(ox, oy)| {
                        let (nx, ny) = (dx as i64 + ox, dy as i64 + oy);
                        nx >= 0
                            && ny >= 0
                            && t.get(nx as usize, ny as usize) == Some(Terrain::Floor)
                    });
                assert!(touches_floor, "door opens onto floor at {px},{py} {side:?}");
            }
        }
    }

    #[test]
    fn footprints_are_organic_not_clean_rectangles() {
        // At least one plot should leave a corner as yard (a non-rectangular
        // footprint) — the whole point of the change.
        let mut any_cut = false;
        for (px, py, w, h) in [(2, 2, 6, 6), (3, 3, 7, 5), (1, 1, 9, 8), (2, 4, 5, 7)] {
            let mut t = blank(20, 20);
            lay_building(&mut t, px, py, w, h, Side::South);
            for &(cx, cy) in &[
                (px, py),
                (px + w - 1, py),
                (px, py + h - 1),
                (px + w - 1, py + h - 1),
            ] {
                match t.get(cx, cy) {
                    Some(Terrain::Wall) | Some(Terrain::Floor) | Some(Terrain::Door) => {}
                    _ => any_cut = true, // a corner left as yard
                }
            }
        }
        assert!(
            any_cut,
            "organic footprints should cut at least one corner to yard"
        );
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
            // The lesser shelters (tent, lean-to) are too small for a hearth —
            // a bedroll, not a fire. Every proper room keeps its hearth.
            if b.w <= 3 || b.h <= 3 {
                continue;
            }
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
    fn furnishings_sit_inside_the_room() {
        let b = PlacedBuilding {
            style: BuildingStyle::Manor,
            x: 2,
            y: 2,
            w: 7,
            h: 8,
            door: (5, 9),
        };
        let f = building_furnishings(&b, 99, BuildCharacter::Plain);
        assert!(!f.is_empty(), "a manor is furnished");
        let (hx, hy) = (b.x + b.w / 2, b.y + b.h / 2);
        for &(x, y, _) in &f {
            assert!(
                x > b.x && x < b.x + b.w - 1 && y > b.y && y < b.y + b.h - 1,
                "furnishing ({x},{y}) is inside the walls"
            );
            assert_ne!((x, y), (hx, hy), "never over the hearth");
            assert_ne!((x, y), b.door, "never over the door");
        }
        assert_eq!(
            building_furnishings(&b, 99, BuildCharacter::Plain),
            f,
            "deterministic"
        );
    }

    #[test]
    fn a_peoples_character_shows_in_the_furnishings() {
        let b = PlacedBuilding {
            style: BuildingStyle::Manor,
            x: 2,
            y: 2,
            w: 7,
            h: 8,
            door: (5, 9),
        };
        let glyphs = |c| -> Vec<char> {
            building_furnishings(&b, 7, c)
                .iter()
                .map(|&(_, _, g)| g)
                .collect()
        };
        let plain = glyphs(BuildCharacter::Plain);
        let grand = glyphs(BuildCharacter::Grand);
        // Same room, same seed: the pieces sit in the same spots but read of a
        // different people — the deep-stone Tzäkhar furnish unlike the humans.
        assert_eq!(plain.len(), grand.len(), "same count of pieces");
        assert_ne!(
            plain, grand,
            "a people's character shows in what they raise"
        );
    }

    #[test]
    fn a_tiny_hut_has_no_room_to_furnish() {
        let b = PlacedBuilding {
            style: BuildingStyle::Hut,
            x: 0,
            y: 0,
            w: 3,
            h: 3,
            door: (1, 2),
        };
        // The 3x3's single interior tile is the hearth — nothing to add.
        assert!(building_furnishings(&b, 1, BuildCharacter::Plain).is_empty());
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
