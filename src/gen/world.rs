use crate::charts::Charts;
use crate::model::{Region, Settlement, World};
use crate::rng::SeedRng;

pub fn generate_world(seed: u64, charts: &Charts) -> World {
    let world_rng = SeedRng::new(seed).fork_for(&format!("world:{}", seed));
    let mut rng = world_rng;

    let n_regions_str = charts
        .region_count
        .sample(&mut rng)
        .unwrap_or_else(|| "5".into());
    let n_regions: usize = n_regions_str.parse().unwrap_or(5).max(3);

    let mut regions = Vec::with_capacity(n_regions);
    for ri in 0..n_regions {
        let region_rng = SeedRng::new(seed).fork_for(&format!("world:{}:region:{}", seed, ri));
        let region = generate_region(region_rng, ri, charts);
        regions.push(region);
    }

    World {
        seed,
        tick: 0,
        regions,
        charts_version: "0.1.0".into(),
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
    let region_name = crate::gen::name::generate_name(&mut rng, "laakso", "f", charts)
        .unwrap_or_else(|_| format!("Region {}", index));

    let descriptions: &[&str] = match region_type.as_str() {
        "river_valley" => &[
            "Fertile lowlands hugging a broad river",
            "Rich alluvial plains with winding waterways",
            "A well-watered corridor between hills",
        ],
        "coast" => &[
            "A windswept coastline with rocky harbours",
            "Sheltered bays and tidal flats",
            "Salt-washed shores where fisher-folk dwell",
        ],
        "forest" => &[
            "Dense old-growth forest with scattered clearings",
            "Shadowed woods where ancient roots run deep",
            "A vast woodland broken by narrow trails",
        ],
        "upland" => &[
            "Rocky highlands with thin soil and hardy folk",
            "Wind-swept ridges above the treeline",
            "Crags and meadows alternating under grey skies",
        ],
        "steppe" => &[
            "Grass ocean stretching to a pale horizon",
            "Endless plains where herders follow the seasons",
            "Open grassland with scattered watering holes",
        ],
        "delta" => &[
            "Labyrinthine waterways and reed-bound islets",
            "A shifting mosaic of silt and tide",
            "Marshy lowlands where river meets sea",
        ],
        _ => &["An uncharted tract of land"],
    };
    let desc_idx = rng.gen_range(descriptions.len() as u32) as usize;
    let description = descriptions[desc_idx].to_string();

    let mut settlements = Vec::with_capacity(n_settlements);
    for si in 0..n_settlements {
        let settlement_seed = format!("world:region:{}:settlement:{}", index, si);
        let settlement_rng = SeedRng::new(rng.next_u64()).fork_for(&settlement_seed);
        let settlement = generate_settlement(settlement_rng, si, &region_id, &region_type, charts);
        settlements.push(settlement);
    }

    Region {
        id: region_id,
        name: region_name,
        region_type,
        description,
        settlements,
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
    let name = crate::gen::name::generate_name(&mut rng, "arkit", "f", charts)
        .unwrap_or_else(|_| format!("Settlement {}", index));

    let n_persons = population_per_settlement(population);
    let mut people = Vec::with_capacity(n_persons);
    for _ in 0..n_persons {
        let person_rng = rng.fork();
        let person =
            crate::gen::person::generate_person_from(person_rng, region_id, &settlement_id, charts);
        people.push(person);
    }

    Settlement {
        id: settlement_id,
        name,
        size,
        region: region_id.to_string(),
        population,
        description: String::new(),
        people,
    }
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
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
        let charts = charts::load_charts("data/charts.ron").unwrap();
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
}
