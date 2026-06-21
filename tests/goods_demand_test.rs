// Goods as real demand (#540): raising a building draws on Wood and Iron, and
// a town short of them builds slower — a real consequence to a goods shortage.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::SimState;

/// Progress of the furthest-along unfinished building in a settlement.
fn build_progress(sim: &SimState, ri: usize, si: usize) -> f64 {
    sim.world.regions[ri].settlements[si]
        .buildings
        .iter()
        .filter(|b| !b.is_complete())
        .map(|b| b.build_progress)
        .fold(0.0, f64::max)
}

#[test]
fn a_town_short_of_materials_builds_slower() {
    let charts = load_charts().expect("charts");
    // A settlement with hands to build (labourer) but no trade that makes Wood
    // or Iron — so the only materials it has are the ones we hand it; nothing
    // refills them. Since the working economy (#671) has foresters fell Wood and
    // carpenters/miners/smiths make Wood/Iron, the no-refill town must keep none
    // of those — only labourers (who make Stone, not Wood/Iron).
    let (base, ri, si) = (0..120u64)
        .find_map(|seed| {
            let sim = SimState::new(seed, charts.clone());
            for (ri, region) in sim.world.regions.iter().enumerate() {
                for (si, s) in region.settlements.iter().enumerate() {
                    let hands = s.profession_count("labourer");
                    let makers = s.profession_count("carpenter")
                        + s.profession_count("miner")
                        + s.profession_count("smith")
                        + s.profession_count("forester");
                    if s.population > 0 && hands > 0 && makers == 0 {
                        return Some((sim.clone(), ri, si));
                    }
                }
            }
            None
        })
        .expect("a settlement that builds but makes no Wood/Iron");

    let mut stocked = base.clone();
    let mut starved = base.clone();
    {
        let s = &mut stocked.world.regions[ri].settlements[si];
        s.goods_stock.insert(ItemType::Wood, 20.0);
        s.goods_stock.insert(ItemType::Iron, 20.0);
    }
    {
        let s = &mut starved.world.regions[ri].settlements[si];
        s.goods_stock.insert(ItemType::Wood, 0.0);
        s.goods_stock.insert(ItemType::Iron, 0.0);
    }

    // A few days of building.
    for _ in 0..(24 * 4) {
        stocked.step();
        starved.step();
    }

    let p_stocked = build_progress(&stocked, ri, si);
    let p_starved = build_progress(&starved, ri, si);
    assert!(
        p_stocked > p_starved,
        "the stocked town builds faster than the starved one ({p_stocked} > {p_starved})"
    );
}
