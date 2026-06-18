// The frontier seed (#623): a village worn by hunger or feud sheds its young
// and unattached to the open road — they leave the settled lands entirely and
// become the frontier's wanderers, the raw material of the bands to come.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::Need;
use deep_world_tui::sim::SimState;

/// Rig a settlement into pressure and make its people the kind the road takes:
/// young, unattached, their own safety worn thin.
fn press_first_settlement(sim: &mut SimState) -> u32 {
    let s = &mut sim.world.regions[0].settlements[0];
    s.famine_days = 10;
    for p in s.people.iter_mut() {
        p.age_band = "youth".into();
        p.has_spouse = false;
        p.needs.values.insert(Need::Safety, 0.05);
    }
    s.population
}

#[test]
fn a_pressed_village_sheds_its_young_to_the_road() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(20240, charts);
    let pop_before = press_first_settlement(&mut sim);
    assert!(pop_before > 0);
    // Several migration turns (interval 30) under standing pressure.
    for t in [30u64, 60, 90, 120, 150] {
        deep_world_tui::sim::migration::tick_migration(&mut sim, t);
        // Keep the town pressed so the read stays a real one across turns.
        sim.world.regions[0].settlements[0].famine_days = 10;
    }
    assert!(
        sim.frontier.wanderers > 0,
        "a pressed village should send some of its young to the frontier"
    );
    assert!(
        sim.world.regions[0].settlements[0].population < pop_before,
        "those who left the settled lands are gone from the town"
    );
}

#[test]
fn the_road_taking_is_deterministic() {
    let charts = load_charts().expect("charts");
    let mut a = SimState::new(771, charts.clone());
    let mut b = SimState::new(771, charts);
    press_first_settlement(&mut a);
    press_first_settlement(&mut b);
    for t in [30u64, 60, 90] {
        deep_world_tui::sim::migration::tick_migration(&mut a, t);
        deep_world_tui::sim::migration::tick_migration(&mut b, t);
        a.world.regions[0].settlements[0].famine_days = 10;
        b.world.regions[0].settlements[0].famine_days = 10;
    }
    assert_eq!(
        a.frontier.wanderers, b.frontier.wanderers,
        "the same seed sheds the same souls to the road"
    );
}
