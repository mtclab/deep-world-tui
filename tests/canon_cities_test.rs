// Canon scale stage 3 (#378): the named cities of the continent reach the
// province by road and by rumor — and the rare Tier-II city, where the land
// can carry one, rightly dominates its sector.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::{SimState, CANON_CITIES};

#[test]
fn the_long_roads_carry_named_manifests() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    // Run a season of days: a third of caravans ride the long roads.
    let mut from_canon = 0;
    for _ in 0..60 {
        sim.world.tick = ((sim.world.tick / 24) + 1) * 24 - 1;
        sim.step();
        for c in &sim.caravans {
            if CANON_CITIES.iter().any(|(name, _)| *name == c.origin) {
                from_canon += 1;
            }
        }
    }
    assert!(
        from_canon > 0,
        "some caravans must come from the named cities of the continent"
    );
}

#[test]
fn a_city_head_count_earns_a_city_sprawl() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    sim.world.regions[0].settlements.truncate(1);
    {
        let s = &mut sim.world.regions[0].settlements[0];
        s.map_x = 4;
        s.map_y = 4;
        s.district = 48;
        s.population = 20_000;
        s.food_stock = 40_000.0;
    }
    sim.world.tick = 23;
    sim.step();
    let s = &sim.world.regions[0].settlements[0];
    assert_eq!(s.size, "city");
    assert!(
        s.district > 48,
        "a Tier-II city sprawls past the town clamp (district {})",
        s.district
    );
    assert!(s.district <= 72, "and stays within its sector");
}

#[test]
fn the_register_is_canon() {
    // The names are the Archive's (great_cities_of_the_ages.md) — no
    // invented metropolises.
    let names: Vec<&str> = CANON_CITIES.iter().map(|(n, _)| *n).collect();
    for required in [
        "Sampa Crossing",
        "Vessenath",
        "Halkess",
        "Velkarath",
        "Keuramark",
    ] {
        assert!(
            names.contains(&required),
            "{required} belongs to the register"
        );
    }
}
