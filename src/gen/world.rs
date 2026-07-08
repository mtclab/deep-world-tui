use crate::charts::Charts;
use crate::model::{Region, Settlement, SettlementService, Terrain, TerrainMap, World};
use crate::rng::SeedRng;

pub fn generate_world(seed: u64, charts: &Charts) -> World {
    generate_world_capped(seed, charts, None)
}

/// Worldgen with an optional per-settlement population cap. `None` = the real
/// game (every soul real, province scale 8.5k–121k). `Some(n)` is a **test
/// affordance**: it clamps each settlement to at most `n` souls so soak/long-run
/// tests step small rosters instead of a whole province, exercising the same
/// entity-first code paths deterministically. Population stays `people.len()`;
/// only the count shrinks.
pub fn generate_world_capped(seed: u64, charts: &Charts, max_pop: Option<usize>) -> World {
    // The crate's own `#[cfg(test)]` build defaults to a small province so the
    // hundreds of lib unit tests step real entities cheaply. This applies ONLY
    // when compiling the lib's tests (`cargo test --lib`); integration test
    // crates link the non-test lib and so keep full province scale — the scale
    // and canon-plausibility suites there still exercise 8.5k–121k souls. The
    // real game (no test cfg) is never capped.
    let max_pop = max_pop.or(if cfg!(test) { Some(300) } else { None });
    let world_rng = SeedRng::new(seed).fork_for("world");
    let mut rng = world_rng;

    let n_regions_str = charts
        .region_count
        .sample(&mut rng)
        .unwrap_or_else(|| "5".into());
    let n_regions: usize = n_regions_str.parse().unwrap_or(5).max(3);

    // The frontier scales with the province (#669): a wider province keeps a
    // wider ungoverned edge — roughly one march per five sectors, the far
    // (highest-index) regions given to the wild, the settled core always the
    // majority. A small province (3 regions) stays all settled — no room to
    // spare a whole region to the wild.
    let n_marches = if n_regions < 4 {
        0
    } else {
        (n_regions / 5).max(1).min(n_regions - 2)
    };
    let mut regions = Vec::with_capacity(n_regions);
    for ri in 0..n_regions {
        let region_rng = SeedRng::new(seed).fork_for(&format!("world:region:{}", ri));
        let is_march = ri >= n_regions - n_marches;
        let region = generate_region(region_rng, ri, charts, is_march);
        regions.push(region);
    }

    let region_cols = compute_grid_cols(n_regions);
    for (i, region) in regions.iter_mut().enumerate() {
        region.neighbors = compute_neighbors(i, n_regions, region_cols);
    }

    seamless_terrain_edges(&mut regions);

    // Entity-first (deep-world-godot#50): now that every settlement's final
    // population is fixed (terrain pass included), give each one a real Person
    // per soul. Parallel — the heaviest part of worldgen at province scale.
    materialize_residents_capped(&mut regions, seed, charts, max_pop);

    // The land decides the master: the province answers to the polity its
    // most common region type belongs to (canon the_37_polities.md).
    let polity = dominant_polity(&regions);

    World {
        seed,
        tick: 0,
        regions,
        charts_version: "0.1.0".into(),
        region_cols,
        polity,
    }
}

/// The polity holding a province, from the region type that occurs most across
/// its regions. Ties break by the first such region — deterministic.
fn dominant_polity(regions: &[crate::model::Region]) -> crate::model::Polity {
    use std::collections::HashMap;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for r in regions {
        *counts.entry(r.region_type.as_str()).or_default() += 1;
    }
    // Walk the regions in order and pick the type with the highest count,
    // first-seen winning ties — avoids HashMap iteration nondeterminism.
    let mut best_type = regions
        .first()
        .map(|r| r.region_type.as_str())
        .unwrap_or("");
    let mut best_count = 0usize;
    for r in regions {
        let c = counts[r.region_type.as_str()];
        if c > best_count {
            best_count = c;
            best_type = r.region_type.as_str();
        }
    }
    crate::model::Polity::for_region_type(best_type)
}

fn compute_grid_cols(n: usize) -> usize {
    let cols = (n as f64).sqrt().ceil() as usize;
    cols.max(1)
}

fn compute_neighbors(index: usize, total: usize, cols: usize) -> crate::model::RegionNeighbors {
    let row = index / cols;
    let col = index % cols;
    let north = if row > 0 {
        Some((row - 1) * cols + col)
    } else {
        None
    };
    let south = {
        let si = (row + 1) * cols + col;
        if si < total {
            Some(si)
        } else {
            None
        }
    };
    let east = {
        if col + 1 < cols {
            let ei = row * cols + col + 1;
            if ei < total {
                Some(ei)
            } else {
                None
            }
        } else {
            None
        }
    };
    let west = if col > 0 {
        Some(row * cols + col - 1)
    } else {
        None
    };
    crate::model::RegionNeighbors {
        north,
        south,
        east,
        west,
    }
}

fn seamless_terrain_edges(regions: &mut [Region]) {
    let n = regions.len();
    // A settlement is a place, not a texture — it must not sit on the literal
    // seam between regions (a big district can now reach an edge). Clear any
    // Settlement on the four border rows/cols to Road first, so both sides of a
    // seam agree (the copy below also maps Settlement->Road).
    for region in regions.iter_mut() {
        let (w, h) = (region.terrain.width, region.terrain.height);
        for x in 0..w {
            for y in [0, h.saturating_sub(1)] {
                if region.terrain.get(x, y) == Some(Terrain::Settlement) {
                    region.terrain.set(x, y, Terrain::Road);
                }
            }
        }
        for y in 0..h {
            for x in [0, w.saturating_sub(1)] {
                if region.terrain.get(x, y) == Some(Terrain::Settlement) {
                    region.terrain.set(x, y, Terrain::Road);
                }
            }
        }
    }
    for pass in 0..2 {
        for i in 0..n {
            if pass == 0 {
                let north = regions[i].neighbors.north;
                if let Some(ni) = north {
                    let w = regions[i].terrain.width;
                    let h = regions[i].terrain.height;
                    let h_other = regions[ni].terrain.height;
                    if h == h_other && regions[ni].terrain.width == w {
                        for x in 0..w {
                            let mut top =
                                regions[ni].terrain.get(x, h - 1).unwrap_or(Terrain::Grass);
                            // A settlement is a place, not a texture: the
                            // seam carries its road onward instead.
                            if top == Terrain::Settlement {
                                top = Terrain::Road;
                            }
                            regions[i].terrain.set(x, 0, top);
                        }
                    }
                }
            } else {
                let east = regions[i].neighbors.east;
                if let Some(ei) = east {
                    let w = regions[i].terrain.width;
                    let h = regions[i].terrain.height;
                    let w_other = regions[ei].terrain.width;
                    if w_other == w && regions[ei].terrain.height == h {
                        for y in 1..h.saturating_sub(1) {
                            let mut right = regions[ei].terrain.get(0, y).unwrap_or(Terrain::Grass);
                            if right == Terrain::Settlement {
                                right = Terrain::Road;
                            }
                            regions[i].terrain.set(w - 1, y, right);
                        }
                    }
                }
            }
        }
    }
}

pub fn region_settlement_count(rng: &mut SeedRng, region_type: &str, charts: &Charts) -> usize {
    let n_str = charts
        .settlements_per_region
        .resolve_and_sample("", region_type, "", "", rng)
        .unwrap_or_else(|| "2".into());
    let n: usize = n_str.parse().unwrap_or(2).max(1);
    n.max(if is_dense_region(region_type) { 2 } else { 1 })
}

fn is_dense_region(region_type: &str) -> bool {
    matches!(region_type, "river_valley" | "coast" | "delta")
}

fn generate_region(mut rng: SeedRng, index: usize, charts: &Charts, is_march: bool) -> Region {
    let region_type = charts.region.sample(&mut rng).unwrap_or_default();

    // A march is ungoverned wilderness: no town holds it (#630). The settled
    // regions are unchanged.
    let n_settlements = if is_march {
        0
    } else {
        region_settlement_count(&mut rng, &region_type, charts)
    };

    let region_id = format!("region-{:04x}", index);
    let region_stem_people = match region_type.as_str() {
        "river_valley" => "vayla",
        "coast" => "merak",
        "forest" => "metsik",
        "upland" => "sepat",
        "steppe" => "khor",
        "delta" => "laakso",
        _ => "arkit",
    };
    let region_name = crate::gen::name::generate_place_stem(&mut rng, region_stem_people, charts)
        .unwrap_or_else(|_| format!("Region {}", index));

    let region_subtype = charts
        .region_subtypes
        .resolve_and_sample("", &region_type, "", "", &mut rng)
        .unwrap_or_else(|| "generic".into());

    // A march reads as the ungoverned wilderness it is, not as a settled region
    // (#630). Its own land still shows through the region_type, but the framing
    // is the wild country past the towns' reach.
    let description = if is_march {
        crate::sim::frontier::march_description(&region_type).to_string()
    } else {
        charts
            .region_descriptions
            .get(&region_type)
            .and_then(|descs| {
                let idx = rng.gen_range(descs.len() as u32) as usize;
                descs.get(idx).cloned()
            })
            .unwrap_or_else(|| "An uncharted tract of land".into())
    };

    let mut settlements = Vec::with_capacity(n_settlements);
    for si in 0..n_settlements {
        let settlement_seed = format!("world:region:{}:settlement:{}", index, si);
        let settlement_rng = SeedRng::new(rng.next_u64()).fork_for(&settlement_seed);
        let settlement = generate_settlement(settlement_rng, si, &region_id, &region_type, charts);
        settlements.push(settlement);
    }

    let terrain = generate_terrain(&mut rng, &region_type, &mut settlements);

    // Opening sky drawn from the region's own tendencies; the front system
    // takes over from day one.
    let weather = crate::model::Weather::generate(
        rng.next_u64(),
        0,
        crate::sim::region_work_terrain(&region_type),
    );

    Region {
        id: region_id,
        name: region_name,
        region_type,
        region_subtype,
        description,
        settlements,
        terrain,
        neighbors: crate::model::RegionNeighbors::default(),
        structures: vec![],
        weather,
        game_richness: 1.0,
        is_march,
        known_fed: None,
        known_fed_as_of: 0,
    }
}

fn generate_terrain(
    rng: &mut SeedRng,
    region_type: &str,
    settlements: &mut [Settlement],
) -> TerrainMap {
    // Canon scale, stage 2 (#378): sectors carry enough ground that a
    // town of thousands shows its full sprawl beside the forests and
    // rivers that feed it.
    let width = 160usize;
    // Taller than wide enough that a Tier-II city has the vertical room to
    // sprawl past a town beside its river, not just the horizontal berth (#480
    // freed the width; this frees some height). Old saves stay their own size —
    // the upscale chain is width-driven and stops at 160 wide.
    let height = 100usize;
    let base = match region_type {
        "river_valley" => Terrain::Grass,
        "coast" => Terrain::Sand,
        "forest" => Terrain::Forest,
        "upland" => Terrain::Mountain,
        "steppe" => Terrain::Steppe,
        "delta" => Terrain::Swamp,
        _ => Terrain::Grass,
    };
    let mut tiles = vec![base; width * height];
    let water_row = match region_type {
        "river_valley" | "delta" => Some(height / 2),
        "coast" => Some(height - 3),
        _ => None,
    };
    if let Some(row) = water_row {
        for x in 0..width {
            tiles[row * width + x] = Terrain::Water;
            if row + 1 < height {
                tiles[(row + 1) * width + x] = Terrain::Water;
            }
            if region_type == "river_valley" && row > 0 && x % 4 == 0 {
                let spread = (rng.next_u64() as usize) % 3;
                for d in 0..=spread {
                    if row > d {
                        tiles[(row - 1 - d) * width + x] = Terrain::Water;
                    }
                    if row + 2 + d < height {
                        tiles[(row + 2 + d) * width + x] = Terrain::Water;
                    }
                }
            }
        }
        // Carve a ford so the river/coast doesn't split the region into two
        // unreachable halves. One passable column (a crossing) is enough to keep
        // the map connected.
        let ford_x = width / 4 + (rng.next_u64() as usize) % (width / 2).max(1);
        for y in 0..height {
            if tiles[y * width + ford_x] == Terrain::Water {
                tiles[y * width + ford_x] = Terrain::Road;
            }
        }
    }
    if region_type == "upland" {
        for y in 0..height {
            for x in 0..width {
                let v = (rng.next_u64() as usize) % 5;
                if v == 0 {
                    tiles[y * width + x] = Terrain::Mountain;
                } else if v == 1 {
                    tiles[y * width + x] = Terrain::Grass;
                }
            }
        }
    }
    if region_type == "steppe" {
        for y in 0..height {
            for x in 0..width {
                let v = (rng.next_u64() as usize) % 10;
                if v == 0 {
                    tiles[y * width + x] = Terrain::Farmland;
                }
            }
        }
    }
    if region_type == "forest" {
        for y in 0..height {
            for x in 0..width {
                let v = (rng.next_u64() as usize) % 4;
                if v == 0 {
                    tiles[y * width + x] = Terrain::Grass;
                }
            }
        }
    }
    if region_type == "coast" {
        for y in 0..height {
            for x in 0..width {
                if y < height - 3 {
                    let v = (rng.next_u64() as usize) % 3;
                    tiles[y * width + x] = if v == 0 {
                        Terrain::Grass
                    } else {
                        Terrain::Sand
                    };
                }
            }
        }
    }
    if region_type == "delta" {
        for y in 0..height {
            for x in 0..width {
                let v = (rng.next_u64() as usize) % 3;
                if v == 0 {
                    tiles[y * width + x] = Terrain::Water;
                } else if v == 1 {
                    tiles[y * width + x] = Terrain::Swamp;
                }
            }
        }
    }
    let n_settle = settlements.len().max(1);
    let spacing = width / n_settle;
    let mut settle_positions: Vec<(usize, usize)> = Vec::new();
    let mut next_free_x = 2usize;
    for (i, settlement) in settlements.iter_mut().enumerate() {
        // Position first (the seam margin keeps towns from being cloned
        // into neighbors), capacity second — sampled where the town will
        // actually stand, so the gen-time ceiling and the daily sim agree.
        let cx = (spacing / 2 + i * spacing).clamp(2, width.saturating_sub(3));
        let cy = if let Some(row) = water_row {
            if row > 3 { row - 2 } else { row + 3 }.clamp(2, height.saturating_sub(3))
        } else {
            (3 + (rng.next_u64() as usize) % (height - 6).max(1)).clamp(2, height.saturating_sub(3))
        };
        let sx = cx.min(width.saturating_sub(10)).max(2).max(next_free_x);
        let sy = cy.min(height.saturating_sub(10)).max(2);
        // The land decides what this spot can carry (canon's hydraulic
        // principles), and the head-count follows: a delta crossroads seeds
        // a town of thousands, a dry upland shelf seeds a steading.
        let is_city =
            crate::gen::town::carrying_capacity(&tiles, width, height, sx + 1, sy + 1, region_type)
                >= 15_000;
        // A real town needs a roof per ~7 souls — a town of a thousand is HUNDREDS
        // of houses, so let the district claim the footprint its head-count wants,
        // up to a generous per-tier ceiling (a wide city gets more). The room the
        // region actually leaves still binds it below (see the footprint below).
        let max_edge = if is_city { 140 } else { 110 };
        let frac = 35 + (rng.next_u64() % 36) as u32; // 35–70% of capacity
        let cap =
            crate::gen::town::carrying_capacity(&tiles, width, height, sx + 1, sy + 1, region_type);
        // The founding assay of the land (#728): recorded now, before this town
        // paints its ground with roofs and before growth can slide its anchor
        // off the water, so the daily sim and lifecycle read a stable ceiling.
        settlement.land_capacity = cap;
        settlement.population = (cap.saturating_mul(frac) / 100).max(8);
        settlement.size =
            crate::model::economy::Settlement::size_for_population(settlement.population)
                .to_string();
        settlement.food_stock = settlement.population as f64 * 2.0;
        // District edge from the head-count: the footprint its population wants,
        // held to the room the region leaves FROM THE FIXED ANCHOR — down to the
        // bottom edge and across to the right, so it never overlaps the next town,
        // runs off an edge, or spills into the sea. The anchor is NOT moved: the
        // capacity was sampled at (sx+1, sy+1), and the daily sim and the canon
        // scale tests read that same fixed corner, so a town always keeps the
        // water it was founded on and its gen-time ceiling matches the runtime
        // one. (A waterside town then sprawls only as far inland as the room
        // allows — it does not wander off its coast to claim a wider district.)
        let mut n =
            (crate::model::economy::Settlement::footprint_for_population(settlement.population)
                as usize)
                .min(max_edge)
                .min(width.saturating_sub(sx + 2))
                .min(height.saturating_sub(sy + 2))
                .max(2);
        n -= n % 2;
        settlement.district = n as u32;
        settlement.map_x = sx as u32;
        settlement.map_y = sy as u32;
        next_free_x = sx + n + 3;
        for dy in 0..n {
            for dx in 0..n {
                let idx = (sy + dy) * width + (sx + dx);
                if !matches!(tiles[idx], Terrain::Water | Terrain::Coast) {
                    tiles[idx] = Terrain::Settlement;
                }
            }
        }
        for dy in 0..n + 2 {
            for dx in 0..n + 2 {
                let ty = sy + dy;
                let tx = sx + dx;
                if ty < height && tx < width && tiles[ty * width + tx] != Terrain::Settlement {
                    tiles[ty * width + tx] = Terrain::Farmland;
                }
            }
        }
        // Roads aim at a street cell (rel 1,1 is always street in the town
        // layout) — aiming at the anchor would dead-end on a house wall.
        settle_positions.push((sx + 1, sy + 1));
    }

    // Lay every district now, BEFORE the roads: the road painter must see
    // the real houses so it can cut its way to the street.
    {
        let mut tmap = TerrainMap {
            width,
            height,
            tiles,
        };
        for s in settlements.iter() {
            let character = crate::gen::building::BuildCharacter::from_people(
                &s.people
                    .first()
                    .map(|p| p.people.clone())
                    .unwrap_or_default(),
            );
            crate::gen::town::lay_town(
                &mut tmap,
                s.map_x as usize,
                s.map_y as usize,
                s.footprint() as usize,
                character,
            );
        }
        tiles = tmap.tiles;
    }

    for window in settle_positions.windows(2) {
        let (x1, y1) = window[0];
        let (x2, y2) = window[1];
        let mut cx = x1;
        let mut cy = y1;
        while cx != x2 || cy != y2 {
            if cx < width && cy < height {
                match tiles[cy * width + cx] {
                    // Streets and the real buildings are left as they stand —
                    // the road meets the town and becomes its streets, it does
                    // not gouge a hole through a house wall.
                    Terrain::Settlement
                    | Terrain::House
                    | Terrain::Wall
                    | Terrain::Floor
                    | Terrain::Door
                    | Terrain::Hearth => {}
                    _ => tiles[cy * width + cx] = Terrain::Road,
                }
            }
            if cx < x2 {
                cx += 1;
            } else if cx > x2 {
                cx -= 1;
            } else if cy < y2 {
                cy += 1;
            } else if cy > y2 {
                cy -= 1;
            }
        }
    }

    // Scatter rural homesteads across the worked country (#458): the open land
    // between the towns is not empty — a few holdings (a dwelling, an
    // outbuilding, a field) stand on arable ground, well clear of the towns,
    // the roads, and the water. Deterministic; placed last so they sit on the
    // settled land without disturbing the towns or the roads that reach them.
    {
        const HW: usize = 14;
        const HH: usize = 11;
        let mut tmap = TerrainMap {
            width,
            height,
            tiles,
        };
        // A forked stream, so scattering holdings never shifts the rest of
        // worldgen (weather and all) off the parent rng.
        let mut hrng = rng.fork_for("homesteads");
        let arable = |t: Terrain| matches!(t, Terrain::Grass | Terrain::Farmland);
        let n_homesteads = 2 + (hrng.next_u64() % 4) as usize; // 2..5
        let mut placed = 0;
        for _ in 0..40 {
            if placed >= n_homesteads {
                break;
            }
            let ax = 2 + (hrng.next_u64() as usize) % width.saturating_sub(HW + 4);
            let ay = 2 + (hrng.next_u64() as usize) % height.saturating_sub(HH + 4);
            // The patch (and a one-tile margin) must be open arable ground —
            // no town, no road, no water, no other holding.
            let mut clear = true;
            'scan: for dy in 0..HH + 2 {
                for dx in 0..HW + 2 {
                    let t = tmap.get(ax + dx, ay + dy);
                    if !t.is_some_and(arable) {
                        clear = false;
                        break 'scan;
                    }
                }
            }
            if !clear {
                continue;
            }
            let seed = crate::rng::mix_u64(
                (ax as u64).wrapping_shl(20) ^ (ay as u64).wrapping_shl(40) ^ 0x40E5_7EAD,
            );
            if !crate::gen::building::lay_homestead(&mut tmap, ax + 1, ay + 1, seed).is_empty() {
                placed += 1;
            }
        }
        tiles = tmap.tiles;
    }

    TerrainMap {
        width,
        height,
        tiles,
    }
}

fn generate_settlement(
    mut rng: SeedRng,
    index: usize,
    region_id: &str,
    region_type: &str,
    charts: &Charts,
) -> Settlement {
    let dense = is_dense_region(region_type);
    let size = loop {
        let s = charts
            .settlement_size
            .resolve_and_sample("", region_type, "", "", &mut rng)
            .unwrap_or_else(|| "hamlet".into());
        if s == "city" && !dense {
            continue;
        }
        break s;
    };

    let pop_str = charts
        .population_tier
        .resolve_and_sample("", region_type, "", &size, &mut rng)
        .unwrap_or_else(|| "40".into());
    let population: u32 = pop_str.parse().unwrap_or(40).max(1);

    let settlement_id = format!("{}-set-{:04x}", region_id, index);
    // Stems come from a region-appropriate naming tradition, not always the
    // Arkit register (places read like the people who named them).
    let stem_people = naming_tradition(region_type);
    let base_name = crate::gen::name::generate_place_stem(&mut rng, stem_people, charts)
        .unwrap_or_else(|_| format!("Settlement {}", index));
    let suffix = charts
        .settlement_suffixes
        .get(region_type)
        .and_then(|suffixes| {
            let idx = rng.gen_range(suffixes.len() as u32) as usize;
            suffixes.get(idx).cloned()
        });
    let name = match suffix {
        Some(s) => format!("{}{}", base_name, s),
        None => base_name,
    };

    let n_persons = population_per_settlement(population);
    let mut people = Vec::with_capacity(n_persons);
    for _ in 0..n_persons {
        let person_rng = rng.fork();
        let person =
            crate::gen::person::generate_person_from(person_rng, region_id, &settlement_id, charts);
        people.push(person);
    }

    // On a people's home ground, now and then a settlement is theirs — a
    // non-human enclave of the canon Five (#454). They keep their own ground:
    // the Mëräk to the tideline, the Khör to the steppe, the Häl to the deep
    // wood. The rng here is this settlement's own and used last, so seeding an
    // enclave does not shift any other settlement's people or place.
    if let Some(home) = enclave_home_people(region_type) {
        if rng.gen_range(3) == 0 {
            for p in people.iter_mut() {
                p.people = home.to_string();
                if let Ok(nm) = crate::gen::name::generate_name(&mut rng, home, &p.sex, charts) {
                    p.name = nm;
                }
            }
        }
    }

    let dominant_people = people.first().map(|p| p.people.clone()).unwrap_or_default();

    Settlement {
        id: settlement_id,
        name,
        size: size.clone(),
        region: region_id.to_string(),
        population,
        description: String::new(),
        people,
        services: settlement_services(&size, &dominant_people),
        politics: crate::model::SettlementPolitics::new(),
        faith: Default::default(),
        // Larders start two days full; the settlement-life tick takes it from here.
        food_stock: population as f64 * 2.0,
        treasury: population.saturating_mul(2),
        goods_stock: std::collections::HashMap::new(),
        farms: Vec::new(),
        buildings: Vec::new(),
        festival_until_day: 0,
        famine_days: 0,
        plague_days: 0,
        map_x: 0,
        map_y: 0,
        district: 0,
        remembered_deed: None,
        land_capacity: 0,
    }
}

/// Old saves carry 40x20 sectors (#372 made them 80x40). Upscale terrain 2x
/// — each tile becomes a 2x2 block, so the world keeps its exact shape —
/// and remap every coordinate that lives on it. Returns true when anything
/// was scaled, so the caller can scale its own player-side coordinates too.
pub fn upscale_world_2x(sim: &mut crate::sim::SimState) -> bool {
    let mut scaled = false;
    for region in sim.world.regions.iter_mut() {
        let (w, h) = (region.terrain.width, region.terrain.height);
        if w == 0 || w >= 160 {
            continue;
        }
        scaled = true;
        let mut tiles = vec![Terrain::Grass; (w * 2) * (h * 2)];
        for y in 0..h {
            for x in 0..w {
                let t = region.terrain.tiles[y * w + x];
                for dy in 0..2 {
                    for dx in 0..2 {
                        tiles[(y * 2 + dy) * (w * 2) + (x * 2 + dx)] = t;
                    }
                }
            }
        }
        region.terrain = TerrainMap {
            width: w * 2,
            height: h * 2,
            tiles,
        };
        for s in region.settlements.iter_mut() {
            s.map_x *= 2;
            s.map_y *= 2;
            // The painted edge covers twice the tiles on the finer grid.
            s.district *= 2;
        }
        for st in region.structures.iter_mut() {
            st.x *= 2;
            st.y *= 2;
        }
    }
    if scaled {
        for site in sim.build_sites.iter_mut() {
            site.x *= 2;
            site.y *= 2;
        }
        for st in sim.structures.iter_mut() {
            st.x *= 2;
            st.y *= 2;
        }
        for d in sim.discoveries.entries.iter_mut() {
            d.x *= 2;
            d.y *= 2;
        }
        for m in sim.memorials.iter_mut() {
            m.x *= 2;
            m.y *= 2;
        }
    }
    scaled
}

/// Backfill settlement anchors for saves written before footprints existed.
/// The old tile↔settlement mapping was "settlement tiles sorted by x", so
/// reassign those tiles as anchors in the same order, then paint each
/// settlement's full footprint so the map matches its size.
pub fn fixup_settlement_anchors(world: &mut crate::model::World) {
    for region in world.regions.iter_mut() {
        if region.settlements.is_empty() {
            continue;
        }
        let (w, h) = (region.terrain.width, region.terrain.height);
        let any_anchor = region
            .settlements
            .iter()
            .any(|s| s.map_x != 0 || s.map_y != 0);
        if !any_anchor {
            let mut tile_pos: Vec<(usize, usize)> = region
                .terrain
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, &t)| t == Terrain::Settlement)
                .map(|(i, _)| (i % w, i / w))
                .collect();
            tile_pos.sort_by_key(|&(x, _)| x);
            if tile_pos.is_empty() {
                continue;
            }
            // Collapse contiguous paint into clusters by bounding box:
            // a tile within two of a cluster's box belongs to that cluster
            // (districts are contiguous; separate towns sit further apart).
            let mut boxes: Vec<(usize, usize, usize, usize)> = Vec::new(); // x0,y0,x1,y1
            let mut anchors: Vec<(usize, usize)> = Vec::new();
            for &(x, y) in &tile_pos {
                let mut claimed = false;
                for b in boxes.iter_mut() {
                    if x + 2 >= b.0 && x <= b.2 + 2 && y + 2 >= b.1 && y <= b.3 + 2 {
                        b.0 = b.0.min(x);
                        b.1 = b.1.min(y);
                        b.2 = b.2.max(x);
                        b.3 = b.3.max(y);
                        claimed = true;
                        break;
                    }
                }
                if !claimed {
                    boxes.push((x, y, x, y));
                    anchors.push((x, y));
                }
            }
            for (i, s) in region.settlements.iter_mut().enumerate() {
                let (x, y) = anchors[i.min(anchors.len() - 1)];
                let n = s.footprint() as usize;
                s.map_x = x.min(w.saturating_sub(n)) as u32;
                s.map_y = y.min(h.saturating_sub(n)) as u32;
            }
        }
        // Lay every district (idempotent; also grows pre-anchor towns).
        let prints: Vec<(usize, usize, usize, crate::gen::building::BuildCharacter)> = region
            .settlements
            .iter()
            .map(|s| {
                let n = (s.footprint() as usize).clamp(2, 48.min(w.saturating_sub(6)));
                let character = crate::gen::building::BuildCharacter::from_people(
                    &s.people
                        .first()
                        .map(|p| p.people.clone())
                        .unwrap_or_default(),
                );
                (s.map_x as usize, s.map_y as usize, n, character)
            })
            .collect();
        for (ax, ay, n, character) in prints {
            crate::gen::town::lay_town(&mut region.terrain, ax, ay, n, character);
        }
    }
}

/// The people of the Five whose home ground this region is, if any — so an
/// enclave is seeded on terrain that is truly theirs (#454): the Mëräk on the
/// coast, the Khör on the steppe, the Häl in the deep forest. Other regions
/// (river valley, upland, delta) are human ground and seed no enclave.
pub fn enclave_home_people(region_type: &str) -> Option<&'static str> {
    match region_type {
        "coast" => Some("merak"),
        "steppe" => Some("khor"),
        "forest" => Some("hal"),
        _ => None,
    }
}

/// The naming tradition a region's places are named in — places read like
/// the people who named them, whoever later lives there.
pub fn naming_tradition(region_type: &str) -> &'static str {
    match region_type {
        "river_valley" => "vayla",
        "coast" => "merak",
        "forest" => "metsik",
        "upland" => "sepat",
        "steppe" => "khor",
        "delta" => "laakso",
        _ => "arkit",
    }
}

pub(crate) fn settlement_services(size: &str, people: &str) -> Vec<SettlementService> {
    let mut svcs = match size {
        "hamlet" => vec![SettlementService::Tavern],
        "village" => vec![SettlementService::Tavern, SettlementService::Temple],
        "town" => vec![SettlementService::Tavern, SettlementService::Temple],
        "city" => vec![SettlementService::Tavern, SettlementService::Temple],
        _ => vec![],
    };
    let pk = crate::model::PeopleKind::from_name(people);
    // Each god-people's signature service. Archive/TradePost/Shrine were fully
    // implemented in use_service but never generated, so they were unreachable.
    match pk {
        crate::model::PeopleKind::Sepat => svcs.push(SettlementService::Forge),
        crate::model::PeopleKind::Ahjo => svcs.push(SettlementService::Hearth),
        crate::model::PeopleKind::Metsik => svcs.push(SettlementService::TrapWorkshop),
        crate::model::PeopleKind::Arkit => svcs.push(SettlementService::Archive),
        crate::model::PeopleKind::Vayla => svcs.push(SettlementService::TradePost),
        crate::model::PeopleKind::Laakso => svcs.push(SettlementService::Shrine),
        // The canon Five each keep their own craft on their own ground (#454):
        // the Tzäkhar deep-forge, the trading floors of the Mëräk and She'ar,
        // the Häl physic-shrine of Keuru's wood, the Khör's word-keepers.
        crate::model::PeopleKind::Tzakhar => svcs.push(SettlementService::Forge),
        crate::model::PeopleKind::Merak => svcs.push(SettlementService::TradePost),
        crate::model::PeopleKind::Shear => svcs.push(SettlementService::TradePost),
        crate::model::PeopleKind::Hal => svcs.push(SettlementService::Shrine),
        crate::model::PeopleKind::Khor => svcs.push(SettlementService::Shrine),
        _ => {}
    }
    svcs
}

// Entity-first epic (deep-world-godot#50): every soul is a real Person. The
// roster is no longer a sample of the population — it IS the population. (Kept as
// a function so the call site reads the same; `materialize_residents` reconciles
// the count after the terrain pass overwrites `population` from land capacity.)
fn population_per_settlement(population: u32) -> usize {
    population.max(1) as usize
}

/// Force every settlement's roster to hold one real `Person` per soul, so
/// `people.len() == population` everywhere (entity-first epic). Worldgen builds a
/// first roster, then the terrain pass overwrites `population` from land
/// capacity — leaving the two out of step; this reconciles them. Used both at
/// the end of worldgen and on load, to top up saves written before
/// materialization. Embarrassingly parallel: each settlement's residents come
/// from its own seeded RNG stream, so the result is identical regardless of
/// thread order.
pub fn materialize_residents(regions: &mut [crate::model::Region], seed: u64, charts: &Charts) {
    materialize_residents_capped(regions, seed, charts, None);
}

/// As [`materialize_residents`], but optionally caps each settlement's roster to
/// `max_pop` souls (test affordance — see [`generate_world_capped`]).
pub fn materialize_residents_capped(
    regions: &mut [crate::model::Region],
    seed: u64,
    charts: &Charts,
    max_pop: Option<usize>,
) {
    use rayon::prelude::*;
    regions.par_iter_mut().enumerate().for_each(|(ri, region)| {
        region
            .settlements
            .par_iter_mut()
            .enumerate()
            .for_each(|(si, s)| {
                let mut target = s.population.max(1) as usize;
                if let Some(cap) = max_pop {
                    target = target.min(cap.max(1));
                }
                if s.people.len() > target {
                    s.people.truncate(target);
                } else if s.people.len() < target {
                    let mut rng = SeedRng::new(seed).fork_for(&format!("materialize:{ri}:{si}"));
                    // Keep an enclave an enclave: if the existing roster is
                    // all one people, new residents share it (#454).
                    let enclave = enclave_people_of(&s.people);
                    let region_id = s.region.clone();
                    let settlement_id = s.id.clone();
                    while s.people.len() < target {
                        let mut p = crate::gen::person::generate_person_from(
                            rng.fork(),
                            &region_id,
                            &settlement_id,
                            charts,
                        );
                        if let Some(home) = &enclave {
                            p.people = home.clone();
                        }
                        s.people.push(p);
                    }
                }
                s.population = s.people.len() as u32;
                // Keep the size tier consistent with the head-count. Capping (or
                // any roster top-up) can move population across a tier boundary;
                // the `size` string was set at worldgen and would otherwise go
                // stale. No-op in the uncapped game (population is unchanged).
                s.size = crate::model::economy::Settlement::size_for_population(s.population)
                    .to_string();
                // Seed the town's common purse (entity-first slice 4): a working
                // wage-fund scaled to its people. Guarded on 0 so worldgen seeds
                // it and pre-slice-4 saves are topped up on load, without wiping
                // a treasury that real trade has since moved.
                if s.treasury == 0 {
                    s.treasury = s.population.saturating_mul(2);
                }
                // Building <-> household (living/organic world): house every soul
                // in a REAL home. The town's homes are laid deterministically, so
                // we group the roster into household-sized chunks and give each a
                // roof — neighbours are real families, a home has known occupants,
                // and a death later leaves one the poorer. Door tile = the home.
                let homes = crate::gen::town::town_buildings(s);
                if !homes.is_empty() && !s.people.is_empty() {
                    let per = s.people.len().div_ceil(homes.len()).max(1);
                    for (i, p) in s.people.iter_mut().enumerate() {
                        let b = &homes[(i / per).min(homes.len() - 1)];
                        // the home's door tile — souls sharing it are a household
                        p.home_x = b.door.0 as u32;
                        p.home_y = b.door.1 as u32;
                    }
                }
            });
    });
}

/// The single people of a roster if every member shares it, else None.
fn enclave_people_of(people: &[crate::model::Person]) -> Option<String> {
    let first = people.first()?.people.clone();
    if people.iter().all(|p| p.people == first) {
        Some(first)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;

    fn make_world(seed: u64) -> World {
        let charts = charts::load_charts().unwrap();
        generate_world(seed, &charts)
    }

    /// Full province scale (bypasses the `cfg!(test)` soak cap) for the few
    /// tests that validate the canon size hierarchy — they need real cities and
    /// towns to exist. Use sparingly; each call is a full 8.5k–121k worldgen.
    fn make_world_full(seed: u64) -> World {
        let charts = charts::load_charts().unwrap();
        generate_world_capped(seed, &charts, Some(usize::MAX))
    }

    #[test]
    fn the_countryside_has_rural_homesteads() {
        // Across a few worlds, the open country between towns holds holdings:
        // building doors that belong to no settlement footprint.
        let mut found = 0;
        for seed in [42u64, 7, 100, 2024, 555, 1] {
            let world = make_world(seed);
            for region in &world.regions {
                for y in 0..region.terrain.height {
                    for x in 0..region.terrain.width {
                        if region.terrain.get(x, y) == Some(Terrain::Door)
                            && !region.settlements.iter().any(|s| s.contains_tile(x, y))
                        {
                            found += 1;
                        }
                    }
                }
            }
        }
        assert!(
            found > 0,
            "the worked country should hold rural homesteads beyond the towns"
        );
    }

    #[test]
    fn a_towns_footprint_scales_with_its_head_count() {
        use crate::model::economy::Settlement;
        // The footprint a town wants grows with its population (a roof per ~7
        // souls): a thousand-soul town wants a real district, a hamlet a handful
        // of plots. (It is then held to the room the region leaves — but no
        // longer starved to a single hut by a late edge placement.)
        let town = Settlement::footprint_for_population(1000) as usize;
        let hamlet = Settlement::footprint_for_population(30) as usize;
        assert!(
            town >= 70,
            "a 1000-soul town wants a big district, got {town}"
        );
        assert!(hamlet <= 20, "a 30-soul hamlet stays small, got {hamlet}");
        assert!(town > hamlet * 3, "a town dwarfs a hamlet");
        // and a real district lays many buildings, not one
        let bld = crate::gen::building::district_buildings(
            2,
            2,
            town.min(96),
            town.min(96),
            7,
            crate::gen::building::BuildCharacter::Plain,
        );
        assert!(
            bld.len() >= 15,
            "a town district holds many buildings, got {}",
            bld.len()
        );
    }

    #[test]
    fn the_five_keep_their_own_craft() {
        use crate::model::SettlementService as S;
        for (people, sig) in [
            ("tzäkhar", S::Forge),
            ("mëräk", S::TradePost),
            ("she'ar", S::TradePost),
            ("häl", S::Shrine),
            ("khör", S::Shrine),
        ] {
            let svcs = settlement_services("village", people);
            assert!(
                svcs.contains(&sig),
                "{people} enclave should keep its {sig:?}, got {svcs:?}"
            );
        }
    }

    #[test]
    fn enclaves_seed_on_their_home_ground() {
        // Across a handful of worlds the Five hold enclaves — each named for
        // what it is and peopled by one of the Five — and at least one sits on
        // its own home ground (the seeded kind), the Mëräk coast, the Khör
        // steppe, the Häl wood.
        let mut total_enclaves = 0;
        let mut on_home_ground = 0;
        for seed in [42u64, 7, 100, 2024, 555] {
            let world = make_world(seed);
            for region in &world.regions {
                for s in &region.settlements {
                    if let Some(pk) = s.enclave_people() {
                        total_enclaves += 1;
                        assert!(pk.is_of_the_five());
                        assert!(
                            s.display_name().contains("enclave"),
                            "an enclave names itself one: {}",
                            s.display_name()
                        );
                        if enclave_home_people(&region.region_type)
                            .map(crate::model::PeopleKind::from_name)
                            == Some(pk)
                        {
                            on_home_ground += 1;
                        }
                    }
                }
            }
        }
        assert!(
            total_enclaves > 0,
            "the Five should hold at least one enclave across five worlds"
        );
        assert!(
            on_home_ground > 0,
            "at least one enclave sits on its own home ground"
        );
    }

    #[test]
    fn generate_world_min_three_regions() {
        let world = make_world(42);
        assert!(
            world.regions.len() >= 3,
            "world has {} regions, need ≥3",
            world.regions.len()
        );
    }

    #[test]
    fn generate_world_each_region_has_settlement() {
        let world = make_world(42);
        for region in &world.regions {
            // A march is ungoverned wilderness by design (#630) — no town.
            if region.is_march {
                continue;
            }
            assert!(
                !region.settlements.is_empty(),
                "region '{}' ({}) has no settlements",
                region.id,
                region.region_type
            );
        }
    }

    #[test]
    fn generate_world_deterministic() {
        let a = make_world(42);
        let b = make_world(42);
        assert_eq!(a.seed, b.seed);
        assert_eq!(a.regions.len(), b.regions.len());
        for (ra, rb) in a.regions.iter().zip(b.regions.iter()) {
            assert_eq!(ra.id, rb.id);
            assert_eq!(ra.name, rb.name);
            assert_eq!(ra.region_type, rb.region_type);
            assert_eq!(ra.settlements.len(), rb.settlements.len());
            for (sa, sb) in ra.settlements.iter().zip(rb.settlements.iter()) {
                assert_eq!(sa.id, sb.id);
                assert_eq!(sa.name, sb.name);
                assert_eq!(sa.population, sb.population);
                assert_eq!(sa.people.len(), sb.people.len());
            }
        }
    }

    #[test]
    fn generate_world_different_seeds_differ() {
        let a = make_world(42);
        let b = make_world(99);
        let a_names: Vec<&str> = a.regions.iter().map(|r| r.name.as_str()).collect();
        let b_names: Vec<&str> = b.regions.iter().map(|r| r.name.as_str()).collect();
        assert_ne!(
            a_names, b_names,
            "worlds from different seeds should differ"
        );
    }

    #[test]
    fn generate_world_region_types_valid() {
        let world = make_world(42);
        let charts = charts::load_charts().unwrap();
        for region in &world.regions {
            assert!(
                charts.region.entries.contains_key(&region.region_type),
                "region_type '{}' not in chart",
                region.region_type
            );
        }
    }

    #[test]
    fn generate_world_settlement_sizes_valid() {
        let world = make_world_full(42);
        // Sizes follow the canon hierarchy now (Rennik's survey), not the
        // sampling chart: tier must match the head-count.
        for region in &world.regions {
            for settlement in &region.settlements {
                assert_eq!(
                    settlement.size,
                    crate::model::economy::Settlement::size_for_population(settlement.population),
                    "settlement size '{}' not in chart",
                    settlement.size
                );
            }
        }
    }

    #[test]
    fn generate_world_dense_regions_higher_population() {
        let mut dense_pop: Vec<u32> = Vec::new();
        let mut sparse_pop: Vec<u32> = Vec::new();
        for seed in 0..20u64 {
            let world = make_world(seed);
            for region in &world.regions {
                for settlement in &region.settlements {
                    match region.region_type.as_str() {
                        "river_valley" | "coast" | "delta" => {
                            dense_pop.push(settlement.population);
                        }
                        "forest" | "upland" | "steppe" => {
                            sparse_pop.push(settlement.population);
                        }
                        _ => {}
                    }
                }
            }
        }
        if dense_pop.is_empty() || sparse_pop.is_empty() {
            return;
        }
        let dense_avg = dense_pop.iter().sum::<u32>() as f64 / dense_pop.len() as f64;
        let sparse_avg = sparse_pop.iter().sum::<u32>() as f64 / sparse_pop.len() as f64;
        assert!(
            dense_avg >= sparse_avg,
            "dense region avg pop ({:.0}) < sparse avg ({:.0})",
            dense_avg,
            sparse_avg
        );
    }

    #[test]
    fn generate_world_persons_have_valid_fields() {
        let world = make_world(42);
        let charts = charts::load_charts().unwrap();
        for region in &world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    assert!(
                        !person.id.is_empty(),
                        "empty person id in {}",
                        settlement.id
                    );
                    assert!(
                        charts.people.entries.contains_key(&person.people),
                        "person people '{}' not in chart",
                        person.people
                    );
                    assert_eq!(person.region, region.id);
                    assert_eq!(person.settlement, settlement.id);
                }
            }
        }
    }

    #[test]
    fn generate_world_seed_stored() {
        let world = make_world(12345);
        assert_eq!(world.seed, 12345);
    }

    #[test]
    fn generate_world_persons_nonempty_settlements() {
        let world = make_world(42);
        for region in &world.regions {
            for settlement in &region.settlements {
                assert!(
                    !settlement.people.is_empty(),
                    "settlement '{}' has no people",
                    settlement.id
                );
            }
        }
    }

    #[test]
    fn cities_only_in_dense_regions() {
        for seed in 0..12u64 {
            let world = make_world_full(seed);
            for region in &world.regions {
                for settlement in &region.settlements {
                    if settlement.size == "city" {
                        assert!(
                            is_dense_region(&region.region_type),
                            "city '{}' in sparse region type '{}'",
                            settlement.id,
                            region.region_type
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn dense_regions_have_more_settlements() {
        let mut dense_counts: Vec<usize> = Vec::new();
        let mut sparse_counts: Vec<usize> = Vec::new();
        for seed in 0..100u64 {
            let world = make_world(seed);
            for region in &world.regions {
                let n = region.settlements.len();
                if is_dense_region(&region.region_type) {
                    dense_counts.push(n);
                } else {
                    sparse_counts.push(n);
                }
            }
        }
        if dense_counts.is_empty() || sparse_counts.is_empty() {
            return;
        }
        let dense_avg = dense_counts.iter().sum::<usize>() as f64 / dense_counts.len() as f64;
        let sparse_avg = sparse_counts.iter().sum::<usize>() as f64 / sparse_counts.len() as f64;
        assert!(
            dense_avg > sparse_avg,
            "dense avg settlements ({:.1}) not > sparse ({:.1})",
            dense_avg,
            sparse_avg
        );
    }

    #[test]
    fn coast_regions_always_at_least_two_settlements() {
        for seed in 0..100u64 {
            let world = make_world(seed);
            for region in &world.regions {
                if region.region_type == "coast" && !region.is_march {
                    assert!(
                        region.settlements.len() >= 2,
                        "coast region '{}' has only {} settlements",
                        region.id,
                        region.settlements.len()
                    );
                }
            }
        }
    }

    #[test]
    fn steppe_regions_often_just_one_settlement() {
        let mut single_count = 0usize;
        let mut total = 0usize;
        // A wider sweep than the early slices used (300 seeds): the larger
        // provinces (#667) throw more steppe sectors, and a bigger sample keeps
        // this sparse-rate estimate steady rather than seed-brittle.
        for seed in 0..300u64 {
            let world = make_world(seed);
            for region in &world.regions {
                if region.region_type == "steppe" && !region.is_march {
                    total += 1;
                    if region.settlements.len() == 1 {
                        single_count += 1;
                    }
                }
            }
        }
        if total == 0 {
            return;
        }
        let ratio = single_count as f64 / total as f64;
        // The settled core: steppe still skews sparse — a lone-settlement sector
        // still turns up. The bar sits below the measured settled-core rate
        // (~7% over 300 seeds since the wider provinces of #667 throw more, and
        // denser, steppe sectors; the march #630 — always the edge — is excluded
        // above, removing the wilderness steppes that once padded this count).
        assert!(
            ratio > 0.05,
            "only {:.0}% of steppe regions have 1 settlement (expect >5%)",
            ratio * 100.0
        );
    }

    #[test]
    fn population_city_gt_town_gt_village_gt_hamlet() {
        let mut size_pop: std::collections::HashMap<String, Vec<u32>> =
            std::collections::HashMap::new();
        for seed in 0..12u64 {
            let world = make_world_full(seed);
            for region in &world.regions {
                for settlement in &region.settlements {
                    size_pop
                        .entry(settlement.size.clone())
                        .or_default()
                        .push(settlement.population);
                }
            }
        }
        let avg = |v: &Vec<u32>| -> f64 {
            if v.is_empty() {
                return 0.0;
            }
            v.iter().sum::<u32>() as f64 / v.len() as f64
        };
        let city_avg = avg(size_pop.get("city").unwrap_or(&vec![]));
        let town_avg = avg(size_pop.get("town").unwrap_or(&vec![]));
        let village_avg = avg(size_pop.get("village").unwrap_or(&vec![]));
        let hamlet_avg = avg(size_pop.get("hamlet").unwrap_or(&vec![]));
        if city_avg > 0.0 {
            assert!(
                city_avg > town_avg,
                "city avg ({:.0}) <= town ({:.0})",
                city_avg,
                town_avg
            );
        }
        if town_avg > 0.0 {
            assert!(
                town_avg > village_avg,
                "town avg ({:.0}) <= village ({:.0})",
                town_avg,
                village_avg
            );
        }
        if village_avg > 0.0 {
            assert!(
                village_avg > hamlet_avg,
                "village avg ({:.0}) <= hamlet ({:.0})",
                village_avg,
                hamlet_avg
            );
        }
    }

    #[test]
    fn determinism_world_partial_eq() {
        let charts = charts::load_charts().unwrap();
        let w1 = generate_world(42, &charts);
        let w2 = generate_world(42, &charts);
        assert_eq!(w1, w2, "same-seed worlds must be deeply equal");
    }

    #[test]
    fn determinism_person_partial_eq() {
        let charts = charts::load_charts().unwrap();
        let mut rng1 = crate::rng::SeedRng::new(42);
        let mut rng2 = crate::rng::SeedRng::new(42);
        let p1 = crate::gen::person::generate_person(&mut rng1, &charts);
        let p2 = crate::gen::person::generate_person(&mut rng2, &charts);
        assert_eq!(p1, p2, "same-seed persons must be deeply equal");
    }

    #[test]
    fn determinism_name_sequence() {
        let charts = charts::load_charts().unwrap();
        let mut rng1 = crate::rng::SeedRng::new(77);
        let mut rng2 = crate::rng::SeedRng::new(77);
        for _ in 0..50 {
            let n1 = crate::gen::name::generate_name(&mut rng1, "metsik", "f", &charts).unwrap();
            let n2 = crate::gen::name::generate_name(&mut rng2, "metsik", "f", &charts).unwrap();
            assert_eq!(n1, n2, "name sequence must be deterministic");
        }
    }

    #[test]
    fn determinism_after_forking() {
        let charts = charts::load_charts().unwrap();
        let _intermediate = generate_world(42, &charts);
        let w1 = generate_world(99, &charts);
        let w2 = generate_world(99, &charts);
        assert_eq!(
            w1, w2,
            "world gen must be deterministic regardless of prior calls"
        );
    }

    #[test]
    fn different_seed_produces_different_person() {
        let charts = charts::load_charts().unwrap();
        let mut rng1 = crate::rng::SeedRng::new(42);
        let mut rng2 = crate::rng::SeedRng::new(99);
        let p1 = crate::gen::person::generate_person(&mut rng1, &charts);
        let p2 = crate::gen::person::generate_person(&mut rng2, &charts);
        assert_ne!(
            p1.id, p2.id,
            "different seeds should produce different person ids"
        );
    }

    #[test]
    fn terrain_map_has_correct_dimensions() {
        let charts = charts::load_charts().unwrap();
        let w = generate_world(42, &charts);
        for region in &w.regions {
            assert_eq!(region.terrain.width, 160, "terrain width must be 160");
            assert_eq!(region.terrain.height, 100, "terrain height must be 100");
            assert_eq!(
                region.terrain.tiles.len(),
                16000,
                "terrain must have 160*100 tiles"
            );
        }
    }

    #[test]
    fn terrain_has_settlements_and_roads() {
        let charts = charts::load_charts().unwrap();
        let w = generate_world(42, &charts);
        for region in &w.regions {
            let settle_count = region
                .terrain
                .tiles
                .iter()
                .filter(|&&t| t == crate::model::Terrain::Settlement)
                .count();
            assert!(
                settle_count >= region.settlements.len(),
                "terrain must have at least as many settlement tiles as settlements"
            );
            if region.settlements.len() > 1 {
                let road_count = region
                    .terrain
                    .tiles
                    .iter()
                    .filter(|&&t| t == crate::model::Terrain::Road)
                    .count();
                assert!(road_count > 0, "multi-settlement regions must have roads");
            }
        }
    }

    #[test]
    fn terrain_river_valley_has_water() {
        let charts = charts::load_charts().unwrap();
        let w = generate_world(42, &charts);
        for region in &w.regions {
            if region.region_type == "river_valley" {
                let water_count = region
                    .terrain
                    .tiles
                    .iter()
                    .filter(|&&t| t == crate::model::Terrain::Water)
                    .count();
                assert!(water_count > 0, "river_valley must have water tiles");
            }
        }
    }

    #[test]
    fn terrain_deterministic() {
        let charts = charts::load_charts().unwrap();
        let w1 = generate_world(42, &charts);
        let w2 = generate_world(42, &charts);
        for (r1, r2) in w1.regions.iter().zip(w2.regions.iter()) {
            assert_eq!(
                r1.terrain, r2.terrain,
                "terrain must be deterministic for same seed"
            );
        }
    }

    #[test]
    fn world_has_region_grid_cols() {
        let charts = charts::load_charts().unwrap();
        let w = generate_world(42, &charts);
        assert!(w.region_cols > 0, "region_cols must be > 0");
        let rows = w.regions.len().div_ceil(w.region_cols);
        assert!(
            rows * w.region_cols >= w.regions.len(),
            "grid must fit all regions"
        );
    }

    #[test]
    fn region_neighbors_consistent() {
        let charts = charts::load_charts().unwrap();
        let w = generate_world(42, &charts);
        let cols = w.region_cols;
        for (i, region) in w.regions.iter().enumerate() {
            let row = i / cols;
            let col = i % cols;
            if row > 0 {
                assert_eq!(
                    region.neighbors.north,
                    Some((row - 1) * cols + col),
                    "north neighbor must match grid"
                );
            }
            if row > 0 {
                if let Some(ni) = region.neighbors.north {
                    let north_region = &w.regions[ni];
                    assert_eq!(
                        north_region.neighbors.south,
                        Some(i),
                        "north's south must point back"
                    );
                }
            }
            if let Some(ei) = region.neighbors.east {
                let east_region = &w.regions[ei];
                assert_eq!(
                    east_region.neighbors.west,
                    Some(i),
                    "east's west must point back"
                );
            }
        }
    }

    #[test]
    fn edge_regions_have_no_out_of_bounds_neighbors() {
        let charts = charts::load_charts().unwrap();
        let w = generate_world(42, &charts);
        let cols = w.region_cols;
        for (i, region) in w.regions.iter().enumerate() {
            let row = i / cols;
            let col = i % cols;
            if col == 0 {
                assert_eq!(region.neighbors.west, None, "leftmost column has no west");
            }
            if col == cols - 1 || i == w.regions.len() - 1 {
                // Last in row or last region
            }
            if row == 0 {
                assert_eq!(region.neighbors.north, None, "top row has no north");
            }
            if let Some(si) = region.neighbors.south {
                assert!(si < w.regions.len(), "south neighbor must be valid index");
            }
        }
    }

    #[test]
    fn seamless_edges_match_neighbors() {
        let charts = charts::load_charts().unwrap();
        let w = generate_world(42, &charts);
        for i in 0..w.regions.len() {
            if let Some(ni) = w.regions[i].neighbors.north {
                let w_width = w.regions[i].terrain.width;
                if w.regions[ni].terrain.width == w_width
                    && w.regions[ni].terrain.height == w.regions[i].terrain.height
                {
                    for x in 0..w_width {
                        let north_bottom = w.regions[ni]
                            .terrain
                            .get(x, w.regions[ni].terrain.height - 1);
                        let my_top = w.regions[i].terrain.get(x, 0);
                        assert_eq!(
                            north_bottom, my_top,
                            "region {} top row must match region {} bottom row at x={}",
                            i, ni, x
                        );
                    }
                }
            }
            if let Some(ei) = w.regions[i].neighbors.east {
                let h = w.regions[i].terrain.height;
                if w.regions[ei].terrain.height == h
                    && w.regions[ei].terrain.width == w.regions[i].terrain.width
                    && h > 2
                {
                    for y in 1..h.saturating_sub(1) {
                        let east_left = w.regions[ei].terrain.get(0, y);
                        let my_right = w.regions[i].terrain.get(w.regions[i].terrain.width - 1, y);
                        assert_eq!(
                            east_left, my_right,
                            "region {} right col must match region {} left col at y={}",
                            i, ei, y
                        );
                    }
                }
            }
        }
    }
}
