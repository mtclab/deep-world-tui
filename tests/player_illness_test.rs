// Player illness was an NPC-only system — the player could never get sick.
// Now the player contracts disease (speeding vitals decay) and recovers over
// time. These cover the two wired effects.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ActiveDisease, Disease};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    // Remove starting food so auto-eat doesn't mask the decay comparison.
    if let Some(ps) = a.player_start.as_mut() {
        let food = ps.inventory.get(deep_world_tui::model::ItemType::Food);
        ps.inventory
            .remove(deep_world_tui::model::ItemType::Food, food);
    }
    a.vitals.hunger = 1.0;
    a.vitals.thirst = 1.0;
    a.vitals.energy = 1.0;
    a
}

#[test]
fn a_sick_player_decays_faster() {
    let mut healthy = app();
    let mut sick = app();
    let t = sick_tick(&sick);
    sick.player_start
        .as_mut()
        .unwrap()
        .person
        .illnesses
        // MarshFever: vitals decay modifier 1.35, recovery 96 ticks (survives).
        .push(ActiveDisease::new(Disease::MarshFever, t));

    healthy.advance_clock(5);
    sick.advance_clock(5);

    assert!(
        sick.vitals.hunger < healthy.vitals.hunger,
        "sick hunger {} should be below healthy {}",
        sick.vitals.hunger,
        healthy.vitals.hunger
    );
}

fn sick_tick(a: &App) -> u64 {
    a.sim.as_ref().map_or(0, |s| s.world.tick)
}

#[test]
fn a_recovered_illness_is_cleared() {
    let mut a = app();
    // Contracted long ago; Exhaustion recovers after 24 ticks.
    a.player_start
        .as_mut()
        .unwrap()
        .person
        .illnesses
        .push(ActiveDisease::new(Disease::Exhaustion, 0));
    a.sim.as_mut().unwrap().world.tick = 100;

    a.advance_clock(1);

    assert!(
        a.player_start.as_ref().unwrap().person.illnesses.is_empty(),
        "an illness past its recovery window should be cleared"
    );
}
