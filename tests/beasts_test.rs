// Wild beasts as grid actors (#637): the land restocks its creatures from its
// wildness, and a felled beast stays felled.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::sim::SimState;

#[test]
fn the_land_stocks_its_wild_beasts() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(7, charts);
    // A handful of days for the daily restock to put beasts on the ground.
    for _ in 0..(24 * 8) {
        sim.step();
    }
    assert!(
        !sim.beasts.is_empty(),
        "the wild country should carry beasts after a few days"
    );
    // Each beast stands on its own tile in a real region.
    for b in &sim.beasts {
        assert!(b.region_idx < sim.world.regions.len());
        assert!(b.hp > 0);
    }
    let mut tiles: std::collections::HashSet<(usize, usize, usize)> =
        std::collections::HashSet::new();
    for b in &sim.beasts {
        assert!(
            tiles.insert((b.region_idx, b.px, b.py)),
            "two beasts share a tile"
        );
    }
}

#[test]
fn beast_stocking_is_deterministic() {
    let charts = load_charts().expect("charts");
    let mut a = SimState::new(31, charts.clone());
    let mut b = SimState::new(31, charts);
    for _ in 0..(24 * 8) {
        a.step();
        b.step();
    }
    let av: Vec<_> = a.beasts.iter().map(|x| (&x.id, x.px, x.py)).collect();
    let bv: Vec<_> = b.beasts.iter().map(|x| (&x.id, x.px, x.py)).collect();
    assert_eq!(
        av, bv,
        "the same seed stocks the same beasts on the same ground"
    );
}
