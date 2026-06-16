// Trade flow (#540): goods drift across a region from where they are plentiful
// toward each town's fair share — supply finding demand, with no player.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::SimState;

#[test]
fn a_good_drifts_to_a_town_that_makes_none() {
    let charts = load_charts().expect("charts");
    // Find a region with >=2 living settlements, where some settlement makes no
    // Iron (no miner, no smith) — any Iron it ends with came by trade, not by
    // its own hands.
    let (mut sim, ri, donor, receiver) = (0..50u64)
        .find_map(|seed| {
            let sim = SimState::new(seed, charts.clone());
            for (ri, region) in sim.world.regions.iter().enumerate() {
                let living: Vec<usize> = region
                    .settlements
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.population > 0)
                    .map(|(i, _)| i)
                    .collect();
                if living.len() < 2 {
                    continue;
                }
                let receiver = living.iter().copied().find(|&i| {
                    let s = &region.settlements[i];
                    s.profession_count("miner") == 0 && s.profession_count("smith") == 0
                });
                if let Some(receiver) = receiver {
                    let donor = living.iter().copied().find(|&i| i != receiver).unwrap();
                    return Some((sim.clone(), ri, donor, receiver));
                }
            }
            None
        })
        .expect("a region with a non-Iron-making settlement");

    // Clear all Iron in the region, then hand the donor a big stock.
    for s in sim.world.regions[ri].settlements.iter_mut() {
        s.goods_stock.insert(ItemType::Iron, 0.0);
    }
    sim.world.regions[ri].settlements[donor]
        .goods_stock
        .insert(ItemType::Iron, 40.0);

    // A few days of pure simulation.
    for _ in 0..(24 * 5) {
        sim.step();
    }

    let got = sim.world.regions[ri].settlements[receiver].good(ItemType::Iron);
    assert!(
        got > 0.0,
        "Iron reached a town that forges none — it came by trade ({got})"
    );
    // The donor shed some of its surplus toward its fair share.
    let donor_left = sim.world.regions[ri].settlements[donor].good(ItemType::Iron);
    assert!(
        donor_left < 40.0,
        "the donor shipped out some of its surplus ({donor_left})"
    );
}
