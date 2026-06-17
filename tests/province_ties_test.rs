// A living province (#560): town-to-town ties form on their own in the daily
// sim — caravans deepen partnerships, shared trades grate toward rivalry — so
// the province is a web of standings, not a set of unrelated towns.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::province::TieKind;
use deep_world_tui::sim::SimState;

#[test]
fn the_provinces_towns_grow_close_or_at_odds() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);

    // Long enough for caravans to run and trades to compete.
    for _ in 0..(24 * 200) {
        sim.step();
    }

    // Some real standing has formed somewhere in the province — a partnership or
    // a rivalry, not all flat neutral.
    let names: Vec<String> = sim
        .world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .map(|s| s.name.clone())
        .collect();
    let mut partners = false;
    let mut rivals = false;
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            match sim.province_ties.tie(&names[i], &names[j]) {
                TieKind::Partner => partners = true,
                TieKind::Rival => rivals = true,
                TieKind::Neutral => {}
            }
        }
    }
    assert!(
        partners || rivals,
        "the province's towns formed standings (partners={partners}, rivals={rivals})"
    );
}
