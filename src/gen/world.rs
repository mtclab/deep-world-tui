use crate::charts::Charts;
use crate::model::{Region, Settlement, SettlementService, Terrain, TerrainMap, World};
use crate::rng::SeedRng;

pub fn generate_world(seed: u64, charts: &Charts) -> World {
    let world_rng = SeedRng::new(seed).fork_for("world");
    let mut rng = world_rng;

    let n_regions_str = charts
        .region_count
        .sample(&mut rng)
        .unwrap_or_else(|| "5".into());
    let n_regions: usize = n_regions_str.parse().unwrap_or(5).max(3);

    let mut regions = Vec::with_capacity(n_regions);
    for ri in 0..n_regions {
        let region_rng = SeedRng::new(seed).fork_for(&format!("world:region:{}", ri));
        let region = generate_region(region_rng, ri, charts);
        regions.push(region);
    }

    let region_cols = compute_grid_cols(n_regions);
    for (i, region) in regions.iter_mut().enumerate() {
        region.neighbors = compute_neighbors(i, n_regions, region_cols);
    }

    seamless_terrain_edges(&mut regions);

    World {
        seed,
        tick: 0,
        regions,
        charts_version: "0.1.0".into(),
        region_cols,
    }
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
                            let top = regions[ni].terrain.get(x, h - 1).unwrap_or(Terrain::Grass);
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
                            let right = regions[ei].terrain.get(0, y).unwrap_or(Terrain::Grass);
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

fn generate_region(mut rng: SeedRng, index: usize, charts: &Charts) -> Region {
    let region_type = charts.region.sample(&mut rng).unwrap_or_default();

    let n_settlements = region_settlement_count(&mut rng, &region_type, charts);

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

    let description = charts
        .region_descriptions
        .get(&region_type)
        .and_then(|descs| {
            let idx = rng.gen_range(descs.len() as u32) as usize;
            descs.get(idx).cloned()
        })
        .unwrap_or_else(|| "An uncharted tract of land".into());

    let mut settlements = Vec::with_capacity(n_settlements);
    for si in 0..n_settlements {
        let settlement_seed = format!("world:region:{}:settlement:{}", index, si);
        let settlement_rng = SeedRng::new(rng.next_u64()).fork_for(&settlement_seed);
        let settlement = generate_settlement(settlement_rng, si, &region_id, &region_type, charts);
        settlements.push(settlement);
    }

    let terrain = generate_terrain(&mut rng, &region_type, &settlements);

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
    }
}

fn generate_terrain(
    rng: &mut SeedRng,
    region_type: &str,
    settlements: &[Settlement],
) -> TerrainMap {
    let width = 40usize;
    let height = 20usize;
    let base = match region_type {
        "river_valley" => Terrain::Grass,
        "coast" => Terrain::Sand,
        "forest" => Terrain::Forest,
        "upland" => Terrain::Mountain,
        "steppe" => Terrain::Grass,
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
    for (i, _) in settlements.iter().enumerate() {
        let sx = (spacing / 2 + i * spacing).min(width - 1);
        let sy = if let Some(row) = water_row {
            if row > 3 { row - 2 } else { row + 3 }.min(height - 1)
        } else {
            3 + (rng.next_u64() as usize) % (height - 6).max(1)
        };
        tiles[sy * width + sx] = Terrain::Settlement;
        for dy in 0..3 {
            for dx in 0..3 {
                let ty = sy + dy;
                let tx = sx + dx;
                if ty < height && tx < width && tiles[ty * width + tx] != Terrain::Settlement {
                    tiles[ty * width + tx] = Terrain::Farmland;
                }
            }
        }
        settle_positions.push((sx, sy));
    }

    for window in settle_positions.windows(2) {
        let (x1, y1) = window[0];
        let (x2, y2) = window[1];
        let mut cx = x1;
        let mut cy = y1;
        while cx != x2 || cy != y2 {
            if cx < width && cy < height && tiles[cy * width + cx] != Terrain::Settlement {
                tiles[cy * width + cx] = Terrain::Road;
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
        // Larders start two days full; the settlement-life tick takes it from here.
        food_stock: population as f64 * 2.0,
        farms: Vec::new(),
        buildings: Vec::new(),
        festival_until_day: 0,
        famine_days: 0,
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
        _ => {}
    }
    svcs
}

fn population_per_settlement(population: u32) -> usize {
    let sample_size = match population {
        0..=30 => population as usize,
        31..=80 => (population as f64 * 0.25).round() as usize,
        81..=200 => (population as f64 * 0.10).round() as usize,
        _ => (population as f64 * 0.05).round() as usize,
    };
    sample_size.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;

    fn make_world(seed: u64) -> World {
        let charts = charts::load_charts().unwrap();
        generate_world(seed, &charts)
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
        let world = make_world(42);
        let charts = charts::load_charts().unwrap();
        for region in &world.regions {
            for settlement in &region.settlements {
                assert!(
                    charts
                        .settlement_size
                        .base
                        .entries
                        .contains_key(&settlement.size),
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
        for seed in 0..100u64 {
            let world = make_world(seed);
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
                if region.region_type == "coast" {
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
        for seed in 0..100u64 {
            let world = make_world(seed);
            for region in &world.regions {
                if region.region_type == "steppe" {
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
        assert!(
            ratio > 0.1,
            "only {:.0}% of steppe regions have 1 settlement (expect >10%)",
            ratio * 100.0
        );
    }

    #[test]
    fn population_city_gt_town_gt_village_gt_hamlet() {
        let mut size_pop: std::collections::HashMap<String, Vec<u32>> =
            std::collections::HashMap::new();
        for seed in 0..50u64 {
            let world = make_world(seed);
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
            assert_eq!(region.terrain.width, 40, "terrain width must be 40");
            assert_eq!(region.terrain.height, 20, "terrain height must be 20");
            assert_eq!(
                region.terrain.tiles.len(),
                800,
                "terrain must have 800 tiles"
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
