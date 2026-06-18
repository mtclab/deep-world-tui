// Festivals as settlement state (#315) and rumors that carry information
// (#316). Festivals used to be a per-visit dice roll with no duration; rumors
// were pure flavor.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::SettlementService;
use deep_world_tui::sim::rumors::informed_rumor;
use deep_world_tui::sim::SimState;
use deep_world_tui::ui::app::App;

#[test]
fn famine_becomes_a_rumor() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    sim.world.regions[0].settlements[0].food_stock = 0.0;
    let name = sim.world.regions[0].settlements[0].name.clone();
    let r = informed_rumor(&sim, 1, 0).expect("a starving settlement is news");
    let mentions_famine = (0..50).any(|salt| {
        informed_rumor(&sim, 1, salt)
            .map(|t| t.contains(&name) && t.contains("empty"))
            .unwrap_or(false)
    });
    assert!(
        mentions_famine,
        "famine in {name} should surface as a rumor (sampled: {r})"
    );
}

#[test]
fn festivals_run_for_days_and_discount_services() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app.clock.hour = 10;

    // Pay the tavern on an ordinary day, then during a festival.
    let coins = |a: &App| {
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(deep_world_tui::model::ItemType::Coin)
    };
    {
        let ps = app.player_start.as_mut().unwrap();
        ps.inventory.add(deep_world_tui::model::ItemType::Coin, 50);
    }
    app.enter_settlement(0, 0);
    let before = coins(&app);
    app.use_service(SettlementService::Tavern);
    let normal_cost = before - coins(&app);

    app.sim.as_mut().unwrap().world.regions[0].settlements[0].festival_until_day =
        app.clock.day + 2;
    app.clock.hour = 10;
    let before = coins(&app);
    app.use_service(SettlementService::Tavern);
    let festival_cost = before - coins(&app);

    assert!(
        festival_cost < normal_cost,
        "festival should halve service costs ({festival_cost} vs {normal_cost})"
    );
    assert!(
        informed_rumor(app.sim.as_ref().unwrap(), app.clock.day, 3).is_some(),
        "a festival is news"
    );
}

#[test]
fn tavern_surfaces_the_rumor_to_the_player() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app.clock.hour = 10;
    app.sim.as_mut().unwrap().world.regions[0].settlements[0].food_stock = 0.0;
    app.enter_settlement(0, 0);
    app.use_service(SettlementService::Tavern);
    let msg = app.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("overhear"),
        "the rumor should reach the player's status line, got: {msg}"
    );
}

#[test]
fn the_nonhuman_peoples_trade_is_rumored_where_they_dwell() {
    // Each people is named where its terrain dominates (#447): Mëräk/coast,
    // Khör/steppe, Häl/forest, Tzäkhar/upland. Across a season-spanning sweep
    // of varied worlds, the words that belong to the present region_types show.
    use deep_world_tui::sim::rumors::informed_rumor;
    use deep_world_tui::sim::SimState;

    let charts = load_charts().expect("charts");
    for seed in [42u64, 555, 777] {
        let sim = SimState::new(seed, charts.clone());
        // Only settled regions speak through their people; a march (#630) has
        // the type but no town, so it cannot rumor its people.
        let types: std::collections::HashSet<String> = sim
            .world
            .regions
            .iter()
            .filter(|r| !r.is_march)
            .map(|r| r.region_type.clone())
            .collect();
        // Collect every rumor the world will tell over a year and four salts.
        let mut all = String::new();
        for day in (1..480).step_by(7) {
            for salt in 0..6u64 {
                if let Some(t) = informed_rumor(&sim, day, salt) {
                    all.push_str(&t);
                    all.push('\n');
                }
            }
        }
        for (rtype, people) in [
            ("coast", "Mëräk"),
            ("steppe", "Khör"),
            ("forest", "Häl"),
            ("upland", "Tzäkhar"),
        ] {
            if types.contains(rtype) {
                assert!(
                    all.contains(people),
                    "seed {seed} has a {rtype} region but never rumored the {people}"
                );
            }
        }
    }
}
