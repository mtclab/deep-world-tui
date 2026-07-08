// Canon scale (#378): the world the map is a slice of carries millions, and
// nothing here is authored — the land decides what it can carry (the
// Archive's hydraulic principles), the head-count follows the land, the
// tier follows the head-count, and a town past its land's capacity lives on
// grain moved by road and water.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::town::{carrying_capacity, trade_factor};
use deep_world_tui::model::{Settlement, Terrain, TerrainMap};
use deep_world_tui::sim::SimState;

fn flat(t: Terrain) -> TerrainMap {
    TerrainMap {
        width: 40,
        height: 40,
        tiles: vec![t; 1600],
    }
}

#[test]
fn the_tiers_are_the_archives_tiers() {
    assert_eq!(Settlement::size_for_population(30), "steading");
    assert_eq!(Settlement::size_for_population(100), "hamlet");
    assert_eq!(Settlement::size_for_population(1_000), "village");
    assert_eq!(Settlement::size_for_population(5_000), "town");
    assert_eq!(Settlement::size_for_population(20_000), "city");
    assert_eq!(Settlement::size_for_population(80_000), "city");
}

#[test]
fn the_land_decides_what_it_carries() {
    // A riverside plain against a dry mountain shelf.
    let mut river = flat(Terrain::Grass);
    for y in 0..40 {
        river.tiles[y * 40 + 20] = Terrain::Water;
    }
    let mountain = flat(Terrain::Mountain);
    let plain_cap = carrying_capacity(
        &river.tiles,
        river.width,
        river.height,
        18,
        20,
        "river_valley",
    );
    let shelf_cap = carrying_capacity(
        &mountain.tiles,
        mountain.width,
        mountain.height,
        20,
        20,
        "upland",
    );
    assert!(
        plain_cap > shelf_cap * 10,
        "where rivers run, people settle ({plain_cap} vs {shelf_cap})"
    );
    // Off-water density: one-fifth the riverine standard or less.
    let dry = flat(Terrain::Grass);
    let dry_cap = carrying_capacity(&dry.tiles, dry.width, dry.height, 20, 20, "river_valley");
    assert!(
        dry_cap * 4 <= plain_cap,
        "off-water is a fraction of riverine ({dry_cap} vs {plain_cap})"
    );
}

#[test]
fn generated_worlds_settle_to_canon_plausibility() {
    let charts = load_charts().expect("charts");
    let sim = SimState::new(42, charts);
    let mut village_or_better = 0;
    for region in &sim.world.regions {
        for s in &region.settlements {
            let cap = carrying_capacity(
                &region.terrain.tiles,
                region.terrain.width,
                region.terrain.height,
                s.map_x as usize + 1,
                s.map_y as usize + 1,
                &region.region_type,
            );
            assert!(
                s.population <= cap,
                "{} ({}, {}) seeded under its land's ceiling ({} <= {})",
                s.name,
                region.region_type,
                s.size,
                s.population,
                cap
            );
            if s.population >= 500 {
                village_or_better += 1;
            }
            if region.region_type == "upland" {
                assert!(
                    s.population < 3_000,
                    "no towns on dry shelves: {} holds {}",
                    s.name,
                    s.population
                );
            }
        }
    }
    assert!(
        village_or_better >= 1,
        "the watered lands carry at least village weight"
    );
}

#[test]
fn the_road_feeds_what_the_fields_cannot() {
    let charts = load_charts().expect("charts");
    let mut fed = SimState::new(42, charts.clone());
    let mut cut = SimState::new(42, charts);
    for sim in [&mut fed, &mut cut] {
        let r = &mut sim.world.regions[0];
        let s = &mut r.settlements[0];
        // A town living past what its FIELDS can feed: its land carries only
        // half its roster, so the rest must come by road. Same fields for both
        // (identical land capacity, #728) — the granary difference is the road's
        // doing alone. Without the road bringing grain it runs the stores dry.
        s.land_capacity = (s.people.len() / 2).max(1) as u32;
        s.food_stock = s.people.len() as f64 * 1.5;
    }
    // Cut: strip every road and water tile near the cut town (no reach).
    {
        let r = &mut cut.world.regions[0];
        let (ax, ay) = (
            r.settlements[0].map_x as usize,
            r.settlements[0].map_y as usize,
        );
        let (w, h) = (r.terrain.width, r.terrain.height);
        for y in ay.saturating_sub(28)..(ay + 29).min(h) {
            for x in ax.saturating_sub(28)..(ax + 29).min(w) {
                if matches!(
                    r.terrain.tiles[y * w + x],
                    Terrain::Road | Terrain::Water | Terrain::Coast
                ) {
                    r.terrain.tiles[y * w + x] = Terrain::Mountain;
                }
            }
        }
        let tf = trade_factor(
            &r.terrain.tiles,
            r.terrain.width,
            r.terrain.height,
            ax + 1,
            ay + 1,
        );
        assert!(tf < 1.4, "the cut town has no reach ({tf})");
    }
    for _ in 0..20 {
        fed.world.tick += 23;
        fed.step();
        cut.world.tick += 23;
        cut.step();
    }
    // The road feeds what the fields cannot: grain rides in on the caravans, so
    // the connected town keeps a fuller granary than the one cut off behind the
    // mountains — same seed, same fields, only the road differs.
    let fed_food = fed.world.regions[0].settlements[0].food_stock;
    let cut_food = cut.world.regions[0].settlements[0].food_stock;
    assert!(
        fed_food > cut_food,
        "grain rides the road or it does not arrive (granary {fed_food:.0} fed vs {cut_food:.0} cut)"
    );
}
