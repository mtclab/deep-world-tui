// Goods prosperity (#540): a town grows only when fed AND furnished — bread
// keeps it alive, but tools and cloth let it thrive. A goods-starved town holds.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::SimState;

#[test]
fn a_furnished_town_grows_where_a_starved_one_holds() {
    let charts = load_charts().expect("charts");
    // A settlement that makes no Tool or Cloth (no smith/weaver) and is below
    // its carrying capacity, so growth is possible and its only goods are what
    // we give it.
    let (base, ri, si) = (0..120u64)
        .find_map(|seed| {
            let sim = SimState::new(seed, charts.clone());
            for (ri, region) in sim.world.regions.iter().enumerate() {
                for (si, s) in region.settlements.iter().enumerate() {
                    let makes_goods = s.profession_count("smith") + s.profession_count("weaver");
                    if s.population >= 20 && makes_goods == 0 {
                        return Some((sim.clone(), ri, si));
                    }
                }
            }
            None
        })
        .expect("a settlement that makes no tools or cloth");

    let pop0 = base.world.regions[ri].settlements[si].population;
    let mut furnished = base.clone();
    let mut starved = base.clone();
    for sim in [&mut furnished, &mut starved] {
        let s = &mut sim.world.regions[ri].settlements[si];
        // Keep both well-fed so food is never the limiter.
        s.food_stock = s.population as f64 * 3.0;
    }
    {
        let s = &mut furnished.world.regions[ri].settlements[si];
        s.goods_stock.insert(ItemType::Tool, s.population as f64);
        s.goods_stock.insert(ItemType::Cloth, s.population as f64);
    }
    {
        let s = &mut starved.world.regions[ri].settlements[si];
        s.goods_stock.insert(ItemType::Tool, 0.0);
        s.goods_stock.insert(ItemType::Cloth, 0.0);
    }

    // A short window before regional trade can refurnish the starved town.
    for _ in 0..(24 * 3) {
        furnished.step();
        starved.step();
    }

    let pf = furnished.world.regions[ri].settlements[si].population;
    let ps = starved.world.regions[ri].settlements[si].population;
    assert!(
        pf > ps,
        "the furnished town grew past the starved one (start {pop0}; furnished {pf} > starved {ps})"
    );
}
