// Settlement footprints: a settlement is not a point. Its painted ground
// scales with its size (hamlet 1x1 … city 4x4), every street of it resolves
// to the same settlement, the footprint grows on promotion, and pre-anchor
// saves are backfilled so old worlds keep working.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{PlayerPos, Terrain};
use deep_world_tui::sim::SimState;
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

#[test]
fn footprints_match_size_and_are_painted() {
    let charts = load_charts().expect("charts");
    let sim = SimState::new(42, charts);
    for region in &sim.world.regions {
        for s in &region.settlements {
            let n = s.footprint() as usize;
            assert_eq!(
                n, s.district as usize,
                "{} containment matches the painted district",
                s.name
            );
            assert!(n >= 2, "{} has ground", s.name);
            for dy in 0..n {
                for dx in 0..n {
                    let t = region
                        .terrain
                        .get(s.map_x as usize + dx, s.map_y as usize + dy);
                    assert!(
                        matches!(
                            t,
                            Some(Terrain::Settlement)
                                | Some(Terrain::House)
                                | Some(Terrain::Water)
                                | Some(Terrain::Coast)
                        ),
                        "{} ({}) tile ({},{}) must be street, roof, or stream, got {:?}",
                        s.name,
                        s.size,
                        s.map_x as usize + dx,
                        s.map_y as usize + dy,
                        t
                    );
                }
            }
        }
    }
}

#[test]
fn every_street_resolves_to_its_own_town() {
    let mut a = app();
    let prints: Vec<(usize, usize, usize, usize)> = {
        let region = &a.sim.as_ref().unwrap().world.regions[0];
        region
            .settlements
            .iter()
            .enumerate()
            .map(|(si, s)| {
                (
                    si,
                    s.map_x as usize,
                    s.map_y as usize,
                    s.footprint() as usize,
                )
            })
            .collect()
    };
    assert!(!prints.is_empty());
    for (si, ax, ay, n) in prints {
        for dy in 0..n {
            for dx in 0..n {
                a.player_pos = Some(PlayerPos {
                    region_idx: 0,
                    px: ax + dx,
                    py: ay + dy,
                });
                assert_eq!(
                    a.player_on_settlement(),
                    Some((0, si)),
                    "tile ({},{}) belongs to settlement {}",
                    ax + dx,
                    ay + dy,
                    si
                );
            }
        }
    }
}

#[test]
fn promotion_paints_new_houses_past_the_old_wall() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    // Alone in its valley: growth is neighbor-aware, so the test clears
    // the rest of the region's roster to give the city room.
    sim.world.regions[0].settlements.truncate(1);
    let (ax, ay, old_n) = {
        let s = &mut sim.world.regions[0].settlements[0];
        // A small painted district with a city's head-count arriving: the
        // canon hierarchy says 15k+ is a city, and the map must follow.
        s.district = 8;
        s.population = 20_000;
        (s.map_x as usize, s.map_y as usize, 8usize)
    };
    sim.world.tick = 23;
    sim.step(); // day boundary: settlement life runs
    let region = &sim.world.regions[0];
    let s = &region.settlements[0];
    assert_eq!(s.size, "city", "promotion happened");
    let n = s.footprint() as usize;
    assert!(
        n > old_n,
        "a city stands wider than what it grew from (district {})",
        s.district
    );
    let region = &sim.world.regions[0];
    let s = &region.settlements[0];
    let (ax2, ay2) = (s.map_x as usize, s.map_y as usize);
    for dy in 0..n {
        for dx in 0..n {
            let t = region.terrain.get(ax2 + dx, ay2 + dy);
            // A stream through town keeps its bed; everything else is
            // street or roof.
            assert!(
                matches!(
                    t,
                    Some(Terrain::Settlement)
                        | Some(Terrain::House)
                        | Some(Terrain::Water)
                        | Some(Terrain::Coast)
                ),
                "grown ground painted at ({},{}): {:?}",
                ax2 + dx,
                ay2 + dy,
                t
            );
        }
    }
    let _ = (ax, ay);
}

#[test]
fn pre_anchor_saves_are_backfilled() {
    let charts = load_charts().expect("charts");
    let mut sim = SimState::new(42, charts);
    // Forge an old-save world: anchors zeroed, footprints unpainted beyond
    // what the old single-tile painter left. (Zeroing alone is the signal.)
    let expected: Vec<(usize, usize)> = sim.world.regions[0]
        .settlements
        .iter()
        .map(|s| (s.map_x as usize, s.map_y as usize))
        .collect();
    for s in sim.world.regions[0].settlements.iter_mut() {
        s.map_x = 0;
        s.map_y = 0;
    }
    deep_world_tui::gen::world::fixup_settlement_anchors(&mut sim.world);
    let got: Vec<(usize, usize)> = sim.world.regions[0]
        .settlements
        .iter()
        .map(|s| (s.map_x as usize, s.map_y as usize))
        .collect();
    // The old mapping was x-rank order, and worldgen places settlements
    // left-to-right, so the backfilled anchors land inside the original
    // footprints (clamping may shift them off the exact corner).
    for ((gx, gy), (ex, ey)) in got.iter().zip(expected.iter()) {
        assert!(
            (*gx as i64 - *ex as i64).abs() <= 4 && (*gy as i64 - *ey as i64).abs() <= 4,
            "backfilled anchor ({gx},{gy}) should sit by the original ({ex},{ey})"
        );
    }
    // And the ground is painted for each footprint.
    let region = &sim.world.regions[0];
    for s in &region.settlements {
        let n = s.footprint() as usize;
        for dy in 0..n {
            for dx in 0..n {
                let t = region
                    .terrain
                    .get(s.map_x as usize + dx, s.map_y as usize + dy);
                assert!(matches!(
                    t,
                    Some(Terrain::Settlement)
                        | Some(Terrain::House)
                        | Some(Terrain::Water)
                        | Some(Terrain::Coast)
                ));
            }
        }
    }
}
