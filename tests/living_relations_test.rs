// Living relationships (#548): the people's bonds form and deepen on their own
// — the social web grows and shifts, not just fades.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

#[test]
fn new_bonds_form_during_the_simulation() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new_capped(42, charts, Some(300));
    let start_tick = sim.world.tick;

    // A long stretch of pure simulation — no player.
    for _ in 0..(24 * 100) {
        sim.step();
    }

    // Somewhere, a bond was struck during the soak (formed_at_tick in-window).
    let formed_during = sim.world.regions.iter().any(|r| {
        r.settlements.iter().any(|s| {
            s.people.iter().any(|p| {
                p.relations
                    .iter()
                    .any(|rel| rel.formed_at_tick > start_tick)
            })
        })
    });
    assert!(
        formed_during,
        "the social web grew — at least one bond formed on its own"
    );
}

#[test]
fn the_web_stays_bounded() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new_capped(7, charts, Some(300));
    for _ in 0..(24 * 150) {
        sim.step();
    }
    for region in &sim.world.regions {
        for s in &region.settlements {
            for p in &s.people {
                assert!(
                    p.relations.len() <= 8,
                    "{} holds {} relations — over the cap",
                    p.name,
                    p.relations.len()
                );
            }
        }
    }
}
