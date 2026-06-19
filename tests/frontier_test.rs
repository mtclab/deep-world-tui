// The frontier seed (#623): a village worn by hunger or feud sheds its young
// and unattached to the open road — they leave the settled lands entirely and
// become the frontier's wanderers, the raw material of the bands to come.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::Need;
use deep_world_tui::sim::frontier::Band;
use deep_world_tui::sim::SimState;

/// A region index that holds a living settlement — a band seated here has prey.
fn region_with_a_town(sim: &SimState) -> usize {
    sim.world
        .regions
        .iter()
        .position(|r| r.settlements.iter().any(|s| s.population > 0))
        .expect("some region has a living town")
}

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
fn gathered_wanderers_muster_into_a_band() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(5150, charts);
    // Enough lost souls have gathered in the dark to make a band of them.
    sim.frontier.wanderers = 12;
    // Step a day so the frontier takes its turn (it acts on the day boundary).
    for _ in 0..24 {
        sim.step();
    }
    assert_eq!(
        sim.frontier.bands.len(),
        1,
        "a band gathers from the wanderers"
    );
    let band = &sim.frontier.bands[0];
    assert!(band.size >= 8, "a band musters real numbers: {}", band.size);
    assert!(
        sim.frontier.wanderers < 12,
        "the muster drew down the loose wanderers"
    );
    assert!(band.region_idx < sim.world.regions.len());
}

#[test]
fn band_formation_is_deterministic() {
    let charts = load_charts().expect("charts");
    let mut a = SimState::new(909, charts.clone());
    let mut b = SimState::new(909, charts);
    a.frontier.wanderers = 20;
    b.frontier.wanderers = 20;
    for _ in 0..48 {
        a.step();
        b.step();
    }
    let an: Vec<_> = a.frontier.bands.iter().map(|x| &x.name).collect();
    let bn: Vec<_> = b.frontier.bands.iter().map(|x| &x.name).collect();
    assert_eq!(
        an, bn,
        "the same seed musters the same bands by the same names"
    );
}

#[test]
fn a_band_preys_on_the_town_in_its_country() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(8400, charts);
    let region_idx = region_with_a_town(&sim);
    sim.frontier.bands.push(Band {
        id: "band-test-1".into(),
        name: "the Test of the Wild".into(),
        size: 12,
        region_idx,
        formed_day: 0,
    });
    // One frontier turn (the day boundary).
    for _ in 0..24 {
        sim.step();
    }
    // The raid is talked of on the road, by the band's name.
    let preyed = sim.journal.iter().any(|e| {
        e.text.contains("the Test of the Wild")
            && (e.text.contains("raided") || e.text.contains("fell on"))
    });
    assert!(
        preyed,
        "a band seated over a town raids it, and the road hears"
    );
}

#[test]
fn a_band_in_empty_country_wears_down_and_scatters() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(8401, charts);
    // No prey anywhere — empty every town, so the band cannot raid its own
    // country nor a neighbour's (#630 slice 2 lets a band strike the settled
    // edge, so true starvation needs the whole reach barren). A lone band then
    // only roams and the hungry road wears it down: a size-1 band loses its last
    // soul that very turn and scatters.
    for r in sim.world.regions.iter_mut() {
        for s in r.settlements.iter_mut() {
            s.population = 0;
            s.people.clear();
        }
    }
    sim.frontier.bands.push(Band {
        id: "band-test-2".into(),
        name: "the Doomed of Nowhere".into(),
        size: 1,
        region_idx: 0,
        formed_day: 0,
    });
    // One frontier turn (the day boundary).
    for _ in 0..24 {
        sim.step();
    }
    assert!(
        !sim.frontier.bands.iter().any(|b| b.id == "band-test-2"),
        "a lone band with no prey anywhere in reach scatters"
    );
}

#[test]
fn a_band_in_a_march_raids_the_settled_edge() {
    let charts = load_charts().expect("charts");
    // A world with a march (needs >= 4 regions). Find the march and confirm a
    // neighbouring region holds a town to strike.
    let mut sim = SimState::new(7, charts);
    let march = sim.world.regions.iter().position(|r| r.is_march);
    if let Some(march_idx) = march {
        let n = &sim.world.regions[march_idx].neighbors;
        let has_neighbor_town = [n.north, n.east, n.south, n.west]
            .into_iter()
            .flatten()
            .any(|ni| {
                sim.world
                    .regions
                    .get(ni)
                    .map(|r| r.settlements.iter().any(|s| s.population > 0))
                    .unwrap_or(false)
            });
        if has_neighbor_town {
            sim.frontier.bands.push(Band {
                id: "band-march-1".into(),
                name: "the Marchers".into(),
                size: 12,
                region_idx: march_idx,
                formed_day: 0,
            });
            for _ in 0..24 {
                sim.step();
            }
            // The band holed in the town-less march struck the settled edge and
            // rode back into the dark — the road tells it.
            let raided = sim
                .journal
                .iter()
                .any(|e| e.text.contains("the Marchers") && e.text.contains("marches"));
            assert!(
                raided,
                "a band in a march raids a neighbouring town from the dark"
            );
        }
    }
}

#[test]
fn a_strong_old_band_settles_a_hold() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(8402, charts);
    // A region with a town (so the band can prey, survive, and grow old) that
    // still has room for one more settlement.
    let region_idx = sim
        .world
        .regions
        .iter()
        .position(|r| r.settlements.iter().any(|s| s.population > 0) && r.settlements.len() < 3)
        .expect("a region with a town and room");
    let holds_before = sim.world.regions[region_idx].settlements.len();
    sim.frontier.bands.push(Band {
        id: "band-hold-1".into(),
        name: "the Ragged of the Reach".into(),
        size: 14,
        region_idx,
        formed_day: 0,
    });
    // Past the settle age (60 days), preying all the while.
    for _ in 0..(24 * 65) {
        sim.step();
    }
    let band_gone = !sim.frontier.bands.iter().any(|b| b.id == "band-hold-1");
    let holds_after = sim.world.regions[region_idx].settlements.len();
    assert!(band_gone, "the band that settled is no longer roaming");
    assert!(
        holds_after > holds_before,
        "a hold was raised in the band's country ({holds_before} -> {holds_after})"
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

#[test]
fn a_band_stands_as_individual_members_on_the_grid() {
    use deep_world_tui::sim::frontier::{band_member_tiles, Band, BAND_MEMBERS_SHOWN};
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(7, charts);
    let region_idx = region_with_a_town(&sim);
    sim.frontier.bands.push(Band {
        id: "band-members-1".into(),
        name: "the Many".into(),
        size: 7,
        region_idx,
        formed_day: 0,
    });
    let tiles = band_member_tiles(&sim, "band-members-1", region_idx);
    assert_eq!(tiles.len(), 7, "a band of 7 shows 7 outlaws, not one blob");
    // No two members share a tile.
    let uniq: std::collections::HashSet<_> = tiles.iter().collect();
    assert_eq!(
        uniq.len(),
        tiles.len(),
        "each outlaw stands on their own tile"
    );
    // A huge band is capped to what the grid shows.
    sim.frontier.bands[0].size = 99;
    assert_eq!(
        band_member_tiles(&sim, "band-members-1", region_idx).len(),
        BAND_MEMBERS_SHOWN,
        "a great band is capped to the shown members"
    );
}
