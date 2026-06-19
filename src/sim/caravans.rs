//! Caravans as actors on the road (#641 slice 1): a `Caravan` in the sim is an
//! abstract record — origin, destination, goods, a departure and an arrival
//! tick — with no place on any map and never drawn. The no-blobs rule (every
//! entity its own actor with its own glyph) applies to moving groups too. This
//! puts a caravan on the grid as a small **train of individuals** — a lead
//! drover, the trader, a pack animal, a guard — strung back along the road,
//! their position interpolated along the route by the clock. Deterministic: a
//! pure read of the caravan record and the world's settlement places, no roll.

use crate::model::economy::Caravan;

/// Who a member of a caravan train is, for its glyph and reading on the road.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaravanRole {
    /// The lead drover at the head of the train.
    Drover,
    /// The trader whose goods these are — bump them to deal on the road.
    Trader,
    /// A pack animal, laden.
    Pack,
    /// A hired guard at the rear.
    Guard,
}

/// The roster of a caravan train, head first, trailing back along the road. As
/// many members as the train is long, capped — a small train, not a column.
const TRAIN: [CaravanRole; 4] = [
    CaravanRole::Drover,
    CaravanRole::Trader,
    CaravanRole::Pack,
    CaravanRole::Guard,
];

/// Where a settlement stands: (region_idx, map_x, map_y). `None` if no such id.
fn settlement_place(sim: &crate::sim::SimState, id: &str) -> Option<(usize, usize, usize)> {
    for (ri, region) in sim.world.regions.iter().enumerate() {
        if let Some(s) = region.settlements.iter().find(|s| s.id == id) {
            return Some((ri, s.map_x as usize, s.map_y as usize));
        }
    }
    None
}

/// How far along its route a caravan is at `tick`, 0.0 (just left) to 1.0
/// (arriving). A zero-length journey reads as arrived.
fn route_progress(c: &Caravan, tick: u64) -> f64 {
    if c.arrival_tick <= c.departure_tick {
        return 1.0;
    }
    let span = (c.arrival_tick - c.departure_tick) as f64;
    (tick.saturating_sub(c.departure_tick) as f64 / span).clamp(0.0, 1.0)
}

/// The border tile of `region_idx` toward the direction of neighbour `other`,
/// running straight out from the town at (`tx`,`ty`). `None` if `other` is not
/// a direct neighbour (a longer haul across a third region waits for a later
/// slice).
fn border_toward(
    sim: &crate::sim::SimState,
    region_idx: usize,
    other: usize,
    tx: usize,
    ty: usize,
) -> Option<(usize, usize)> {
    let region = sim.world.regions.get(region_idx)?;
    let (w, h) = (region.terrain.width, region.terrain.height);
    let n = &region.neighbors;
    if n.north == Some(other) {
        Some((tx.min(w.saturating_sub(1)), 0))
    } else if n.south == Some(other) {
        Some((tx.min(w.saturating_sub(1)), h.saturating_sub(1)))
    } else if n.east == Some(other) {
        Some((w.saturating_sub(1), ty.min(h.saturating_sub(1))))
    } else if n.west == Some(other) {
        Some((0, ty.min(h.saturating_sub(1))))
    } else {
        None
    }
}

/// The head of a caravan as seen in `region_idx` at `tick`: a point on the road
/// within this region and the direction it travels, or `None` if the caravan is
/// not on this region's ground now. A caravan is shown in exactly one region at
/// a time — its origin region while the journey's first half runs (the train
/// pulling away toward the border), its destination region for the second half
/// (coming in off the border toward the town). A same-region run shows for the
/// whole journey.
fn head_in_region(
    sim: &crate::sim::SimState,
    c: &Caravan,
    region_idx: usize,
    tick: u64,
) -> Option<(f64, f64, (f64, f64))> {
    let (o_ri, ox, oy) = settlement_place(sim, &c.origin)?;
    let (d_ri, dx, dy) = settlement_place(sim, &c.destination)?;
    let t = route_progress(c, tick);

    let lerp = |a: usize, b: usize, f: f64| a as f64 + (b as f64 - a as f64) * f;
    let dir = |fromx: f64, fromy: f64, tox: f64, toy: f64| {
        let (vx, vy) = (tox - fromx, toy - fromy);
        let len = (vx * vx + vy * vy).sqrt();
        if len < 1e-6 {
            (1.0, 0.0)
        } else {
            (vx / len, vy / len)
        }
    };

    if o_ri == region_idx && d_ri == region_idx {
        // A short haul inside one region: straight from town to town.
        let hx = lerp(ox, dx, t);
        let hy = lerp(oy, dy, t);
        return Some((hx, hy, dir(ox as f64, oy as f64, dx as f64, dy as f64)));
    }
    if o_ri == region_idx {
        // The first leg: pulling out of the home town toward the border. Shown
        // only while the journey's first half runs.
        if t >= 0.5 {
            return None;
        }
        let (bx, by) = border_toward(sim, region_idx, d_ri, ox, oy)?;
        let f = t / 0.5;
        let hx = lerp(ox, bx, f);
        let hy = lerp(oy, by, f);
        return Some((hx, hy, dir(ox as f64, oy as f64, bx as f64, by as f64)));
    }
    if d_ri == region_idx {
        // The last leg: in off the border, on toward the destination town.
        if t < 0.5 {
            return None;
        }
        let (bx, by) = border_toward(sim, region_idx, o_ri, dx, dy)?;
        let f = (t - 0.5) / 0.5;
        let hx = lerp(bx, dx, f);
        let hy = lerp(by, dy, f);
        return Some((hx, hy, dir(bx as f64, by as f64, dx as f64, dy as f64)));
    }
    None
}

/// The train of a caravan as it stands on `region_idx`'s grid at `tick`: each
/// member its own tile and role, head first and trailing back up the road. An
/// empty vector if the caravan is not visible here, or its head falls off the
/// map. Members that would land off the map or stack on the head are dropped,
/// so a train near a border reads as the part of it that is on this ground.
pub fn caravan_train_tiles(
    sim: &crate::sim::SimState,
    caravan_id: &str,
    region_idx: usize,
    tick: u64,
) -> Vec<((usize, usize), CaravanRole)> {
    let Some(c) = sim.caravans.iter().find(|c| c.id == caravan_id) else {
        return Vec::new();
    };
    let Some((hx, hy, (dx, dy))) = head_in_region(sim, c, region_idx, tick) else {
        return Vec::new();
    };
    let Some(region) = sim.world.regions.get(region_idx) else {
        return Vec::new();
    };
    let (w, h) = (region.terrain.width, region.terrain.height);
    if w == 0 || h == 0 {
        return Vec::new();
    }

    let mut out: Vec<((usize, usize), CaravanRole)> = Vec::new();
    let mut taken: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for (i, role) in TRAIN.iter().enumerate() {
        // Each member trails one tile further back up the road from the head.
        let mx = hx - dx * i as f64;
        let my = hy - dy * i as f64;
        let (tx, ty) = (mx.round(), my.round());
        if tx < 0.0 || ty < 0.0 || tx >= w as f64 || ty >= h as f64 {
            continue;
        }
        let tile = (tx as usize, ty as usize);
        if taken.insert(tile) {
            out.push((tile, *role));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::economy::Caravan;
    use crate::sim::SimState;

    fn make_sim() -> SimState {
        SimState::new(7, crate::charts::load_charts().unwrap())
    }

    #[test]
    fn an_intra_region_caravan_strings_a_train_between_its_towns() {
        let mut sim = make_sim();
        // Find a region with two settlements to run a caravan between.
        let ri = sim
            .world
            .regions
            .iter()
            .position(|r| r.settlements.len() >= 2)
            .expect("a region with two towns");
        let (o_id, d_id, ox, oy, dx, dy) = {
            let r = &sim.world.regions[ri];
            let o = &r.settlements[0];
            let d = &r.settlements[1];
            (
                o.id.clone(),
                d.id.clone(),
                o.map_x as usize,
                o.map_y as usize,
                d.map_x as usize,
                d.map_y as usize,
            )
        };
        sim.caravans.push(Caravan {
            id: "carav-1".into(),
            origin: o_id,
            destination: d_id,
            goods: vec![],
            departure_tick: 0,
            arrival_tick: 100,
            travel_cost: 0,
        });

        // At the journey's start the train sits on the origin town.
        let start = caravan_train_tiles(&sim, "carav-1", ri, 0);
        assert!(!start.is_empty(), "the train is drawn at the start");
        assert_eq!(start[0].0, (ox, oy), "the head leaves from the origin town");
        assert_eq!(start[0].1, CaravanRole::Drover, "a drover leads");

        // Halfway, the head is somewhere between the two towns (not on either).
        let mid = caravan_train_tiles(&sim, "carav-1", ri, 50);
        assert!(!mid.is_empty(), "the train is on the road midway");
        if ox != dx || oy != dy {
            assert_ne!(mid[0].0, (ox, oy), "midway it has left the origin");
            assert_ne!(mid[0].0, (dx, dy), "midway it has not yet arrived");
        }

        // Arriving, the head is on the destination town.
        let end = caravan_train_tiles(&sim, "carav-1", ri, 100);
        assert_eq!(end[0].0, (dx, dy), "the head arrives at the destination");
    }

    #[test]
    fn a_caravan_is_not_drawn_in_a_region_it_does_not_touch() {
        let mut sim = make_sim();
        // Two towns in one region; ask about a different region.
        let ri = sim
            .world
            .regions
            .iter()
            .position(|r| r.settlements.len() >= 2)
            .expect("a region with two towns");
        let other = (0..sim.world.regions.len())
            .find(|&i| i != ri)
            .expect("another region");
        let (o_id, d_id) = {
            let r = &sim.world.regions[ri];
            (r.settlements[0].id.clone(), r.settlements[1].id.clone())
        };
        sim.caravans.push(Caravan {
            id: "carav-2".into(),
            origin: o_id,
            destination: d_id,
            goods: vec![],
            departure_tick: 0,
            arrival_tick: 100,
            travel_cost: 0,
        });
        assert!(
            caravan_train_tiles(&sim, "carav-2", other, 50).is_empty(),
            "a caravan running inside one region is not drawn in another"
        );
    }
}
