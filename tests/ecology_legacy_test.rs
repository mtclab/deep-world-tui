// Wildlife ecology (#320) and legacy depth (#323): the land can be trapped
// out and recovers with the seasons; an heir keeps a keepsake and half the
// family's standing.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::SimState;
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

#[test]
fn trapping_draws_the_land_down_and_an_empty_land_gives_nothing() {
    let mut a = app(42);
    a.clock.hour = 10;
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Trap, 1);
        ps.companions.clear();
    }
    // Stand on wild grass clear of any town (houses can wall a straight
    // walk now, so place rather than wander).
    {
        let spot = {
            let region = &a.sim.as_ref().unwrap().world.regions[0];
            let mut found = None;
            'o: for y in 0..region.terrain.height {
                for x in 0..region.terrain.width {
                    if region.terrain.get(x, y) == Some(deep_world_tui::model::Terrain::Grass)
                        && !region.settlements.iter().any(|s| s.contains_tile(x, y))
                    {
                        found = Some((x, y));
                        break 'o;
                    }
                }
            }
            found.expect("wild grass exists")
        };
        a.player_pos = Some(deep_world_tui::model::PlayerPos {
            region_idx: 0,
            px: spot.0,
            py: spot.1,
        });
    }
    assert!(a.player_on_settlement().is_none(), "clear of town");
    let before = a.sim.as_ref().unwrap().world.regions[0].game_richness;
    a.rest_hours(8);
    let after = a.sim.as_ref().unwrap().world.regions[0].game_richness;
    assert!(
        after < before,
        "trapping draws richness ({before} -> {after})"
    );

    // Trap out the valley entirely: the snare comes up empty.
    a.sim.as_mut().unwrap().world.regions[0].game_richness = 0.1;
    let food_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    a.clock.hour = 10;
    a.rest_hours(8);
    let food_after = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    assert!(
        food_after <= food_before + 1, // (goat-free roster; allow auto-eat noise)
        "a trapped-out land should not feed the snare"
    );
}

#[test]
fn richness_recovers_with_time() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    sim.world.regions[0].game_richness = 0.2;
    for _ in 0..30 {
        sim.world.tick = ((sim.world.tick / 24) + 1) * 24 - 1;
        sim.step();
    }
    assert!(
        sim.world.regions[0].game_richness > 0.3,
        "a month should let the land breed back, got {}",
        sim.world.regions[0].game_richness
    );
}

#[test]
fn an_heir_keeps_a_keepsake_and_the_family_standing() {
    let mut a = app(42);
    // Give the dying life some standing and some goods.
    let pid = a.player_start.as_ref().unwrap().person.id.clone();
    let sid = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .id
        .clone();
    a.sim
        .as_mut()
        .unwrap()
        .reputation
        .adjust_settlement(&pid, &sid, 0.4);
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Coin, 10);
        ps.inventory.add(ItemType::Herb, 5);
    }
    // Force an old-age death -> heir handoff.
    a.start_age_years = 80;
    a.birth_day = a.clock.day;
    a.lifespan_years = 80;
    a.vitals.hunger = 1.0;
    a.vitals.thirst = 1.0;
    a.vitals.energy = 1.0;
    a.advance_clock(1);

    let heir = a.player_start.as_ref().expect("heir exists");
    assert!(
        heir.inventory.get(ItemType::Coin) > 0 || heir.inventory.get(ItemType::Herb) > 0,
        "the heir should keep a keepsake"
    );
    let heir_rep = a
        .sim
        .as_ref()
        .unwrap()
        .reputation
        .get(&heir.person.id, &sid);
    assert!(
        heir_rep > 0.65,
        "half the family standing should carry (got {heir_rep})"
    );
}
