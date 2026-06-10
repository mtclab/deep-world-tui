// Living settlements: farms plant/grow/harvest into a food stock, professions
// produce and protect, builders raise buildings, and scarcity moves market
// prices. The Farm/CropType and Building/BuildingType systems were fully
// modeled but never instantiated before this.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::BuildingType;
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::SimState;
use deep_world_tui::ui::app::App;

/// Advance the sim by whole days (settlement life runs on day boundaries).
fn run_days(sim: &mut SimState, days: u64) {
    for _ in 0..days {
        // Jump to the next day boundary so each step lands on tick % 24 == 0.
        sim.world.tick = ((sim.world.tick / 24) + 1) * 24 - 1;
        sim.step();
    }
}

fn staffed_sim(seed: u64, professions: &[&str]) -> SimState {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(seed, charts);
    let settlement = &mut sim.world.regions[0].settlements[0];
    for (i, prof) in professions.iter().enumerate() {
        if let Some(p) = settlement.people.get_mut(i) {
            p.profession = prof.to_string();
        }
    }
    sim
}

#[test]
fn farms_plant_grow_and_fill_the_stores() {
    let mut sim = staffed_sim(42, &["farmer", "farmer", "fisher", "herder"]);
    let initial = sim.world.regions[0].settlements[0].food_stock;
    run_days(&mut sim, 12);
    let s = &sim.world.regions[0].settlements[0];
    assert!(
        !s.farms.is_empty() || s.food_stock > 0.0,
        "farmers should have planted farms"
    );
    // Producers + harvests must outrun pure consumption from the initial stock.
    let consumed_only = (initial - s.population as f64 * 0.15 * 12.0).max(0.0);
    assert!(
        s.food_stock > consumed_only,
        "production should add to the stores (stock {} vs consumption-only floor {})",
        s.food_stock,
        consumed_only
    );
}

#[test]
fn builders_raise_buildings() {
    let mut sim = staffed_sim(
        42,
        &["carpenter", "labourer", "forester", "miner", "weaver"],
    );
    assert!(sim.world.regions[0].settlements[0].buildings.is_empty());
    run_days(&mut sim, 25); // Shelter needs 48 build-ticks; 5 builders/day
    let s = &sim.world.regions[0].settlements[0];
    assert!(
        s.has_building(BuildingType::Shelter),
        "a staffed settlement should complete its first building (progress: {:?})",
        s.buildings.first().map(|b| b.build_progress)
    );
}

#[test]
fn scarcity_moves_food_prices() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app.clock.hour = 10;
    app.enter_settlement(0, 0); // markets are in town

    app.sim.as_mut().unwrap().world.regions[0].settlements[0].food_stock = 0.0;
    let starving_price = app.quote_buy_price(ItemType::Food);
    let pop = app.sim.as_ref().unwrap().world.regions[0].settlements[0].population;
    app.sim.as_mut().unwrap().world.regions[0].settlements[0].food_stock = pop as f64 * 5.0;
    let plenty_price = app.quote_buy_price(ItemType::Food);
    assert!(
        starving_price > plenty_price,
        "lean stores ({starving_price}) should price food above full stores ({plenty_price})"
    );
}

#[test]
fn settlement_life_is_deterministic() {
    let mut a = staffed_sim(99, &["farmer", "carpenter"]);
    let mut b = staffed_sim(99, &["farmer", "carpenter"]);
    run_days(&mut a, 10);
    run_days(&mut b, 10);
    let sa = &a.world.regions[0].settlements[0];
    let sb = &b.world.regions[0].settlements[0];
    assert_eq!(sa.food_stock, sb.food_stock);
    assert_eq!(sa.farms.len(), sb.farms.len());
    assert_eq!(sa.buildings.len(), sb.buildings.len());
}
