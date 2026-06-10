// Weather fronts (#317): a region's sky is daily state with persistence and
// neighbor drift, replacing the hourly hash where sun could follow blizzard
// within the hour.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

fn run_days(sim: &mut SimState, days: u64) -> Vec<deep_world_tui::model::Weather> {
    let mut seq = Vec::new();
    for _ in 0..days {
        sim.world.tick = ((sim.world.tick / 24) + 1) * 24 - 1;
        sim.step();
        seq.push(sim.world.regions[0].weather);
    }
    seq
}

#[test]
fn weather_persists_across_days() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    let seq = run_days(&mut sim, 30);
    let holds = seq.windows(2).filter(|w| w[0] == w[1]).count();
    let changes = seq.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(
        holds >= 5,
        "with ~55% persistence the sky should hold across days sometimes (held {holds}/29)"
    );
    assert!(
        changes >= 2,
        "the sky should also change over a month (changed {changes}/29)"
    );
}

#[test]
fn fronts_are_deterministic() {
    let charts = load_charts().expect("charts");
    let mut a = SimState::new(7, charts.clone());
    let mut b = SimState::new(7, charts);
    assert_eq!(run_days(&mut a, 20), run_days(&mut b, 20));
}
