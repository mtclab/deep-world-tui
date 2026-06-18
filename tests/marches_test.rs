// The marches (#630 slice 1): the province keeps an ungoverned wilderness edge
// — a region with no town holding it, where the frontier's bands and holds and
// the deep wild's dreads will concentrate.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::world::generate_world;

#[test]
fn a_large_province_keeps_a_march_at_its_edge() {
    let charts = load_charts().expect("charts");
    let world = generate_world(7, &charts);
    if world.regions.len() >= 4 {
        // The last region is the march.
        let march = world.regions.last().unwrap();
        assert!(march.is_march, "the edge region is a march");
        assert!(
            march.settlements.is_empty(),
            "a march is ungoverned — no town holds it"
        );
        // The core stays settled: region 0 has its towns, and the province as a
        // whole keeps more than one settlement.
        assert!(
            !world.regions[0].settlements.is_empty(),
            "the settled core is unchanged"
        );
        let total: usize = world.regions.iter().map(|r| r.settlements.len()).sum();
        assert!(total > 1, "the province still has its settlements");
        // Exactly the edge region is wild; the interior is not.
        let marches = world.regions.iter().filter(|r| r.is_march).count();
        assert_eq!(marches, 1, "one march, at the edge");
    }
}

#[test]
fn marches_are_deterministic() {
    let charts = load_charts().expect("charts");
    let a = generate_world(31, &charts);
    let b = generate_world(31, &charts);
    let am: Vec<bool> = a.regions.iter().map(|r| r.is_march).collect();
    let bm: Vec<bool> = b.regions.iter().map(|r| r.is_march).collect();
    assert_eq!(am, bm, "the same seed marks the same marches");
}
