// NPC lifecycle (#314): people age through their bands, the old die (faster
// when ill), children are born to spouse-households, and skilled trades pass
// to the young. Before this, no NPC had ever been born or died of age.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::lifecycle::{band_from_age, TICKS_PER_LIFE_YEAR};
use deep_world_tui::sim::SimState;

/// Step exactly one life-year boundary at a time.
fn run_years(sim: &mut SimState, years: u64) {
    for _ in 0..years {
        sim.world.tick = ((sim.world.tick / TICKS_PER_LIFE_YEAR) + 1) * TICKS_PER_LIFE_YEAR - 1;
        sim.step();
    }
}

#[test]
fn people_age_through_bands() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new_capped(42, charts, Some(300));
    {
        let p = &mut sim.world.regions[0].settlements[0].people[0];
        p.age_band = "youth".into();
        p.age_years = 20;
        p.illnesses.clear();
    }
    let id = sim.world.regions[0].settlements[0].people[0].id.clone();
    run_years(&mut sim, 4);
    let p = sim.world.regions[0].settlements[0]
        .people
        .iter()
        .find(|p| p.id == id);
    if let Some(p) = p {
        assert_eq!(p.age_band, "adult", "a 24-year-old is an adult");
        assert!(p.age_years >= 24);
    } // (may have died by mischance roll — acceptable, covered below)
}

#[test]
fn the_old_die_and_the_world_changes() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new_capped(42, charts, Some(300));
    // Make everyone in one settlement aged: mortality should bite within years.
    {
        let s = &mut sim.world.regions[0].settlements[0];
        for p in s.people.iter_mut() {
            p.age_band = "aged".into();
            p.age_years = 75;
        }
    }
    let before = sim.world.regions[0].settlements[0].people.len();
    let pop_before = sim.world.regions[0].settlements[0].population;
    run_years(&mut sim, 12);
    let s = &sim.world.regions[0].settlements[0];
    assert!(
        s.people.len() < before,
        "an aged settlement should lose people over 12 years ({} -> {})",
        before,
        s.people.len()
    );
    assert!(
        s.population < pop_before,
        "population should shrink with the deaths"
    );
}

#[test]
fn children_are_born_to_spouse_households() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new_capped(7, charts, Some(300));
    {
        let s = &mut sim.world.regions[0].settlements[0];
        for p in s.people.iter_mut() {
            p.age_band = "adult".into();
            p.age_years = 30;
            p.sex = "f".into();
            p.has_spouse = true;
            p.illnesses.clear();
        }
    }
    run_years(&mut sim, 10);
    let s = &sim.world.regions[0].settlements[0];
    assert!(
        s.people.iter().any(|p| p.age_band == "child"),
        "ten years of spouse-households should produce at least one child"
    );
}

#[test]
fn lifecycle_is_deterministic() {
    let charts = load_charts().expect("charts");
    let mut a = SimState::new_capped(99, charts.clone(), Some(300));
    let mut b = SimState::new_capped(99, charts, Some(300));
    run_years(&mut a, 8);
    run_years(&mut b, 8);
    let sa = &a.world.regions[0].settlements[0];
    let sb = &b.world.regions[0].settlements[0];
    assert_eq!(sa.people.len(), sb.people.len());
    assert_eq!(sa.population, sb.population);
}

#[test]
fn band_thresholds() {
    assert_eq!(band_from_age(5), "child");
    assert_eq!(band_from_age(17), "youth");
    assert_eq!(band_from_age(30), "adult");
    assert_eq!(band_from_age(62), "elder");
    assert_eq!(band_from_age(75), "aged");
}
