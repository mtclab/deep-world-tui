// Town layout (#372 PR 2): a settlement is streets and houses on the one
// map — every house reachable from its street, every service at its own
// door, all of it derived (anchor + footprint + services), nothing persisted.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::town::{house_cells, service_at};
use deep_world_tui::model::Terrain;
use deep_world_tui::sim::SimState;

#[test]
fn every_town_has_roofs_on_the_map() {
    let charts = load_charts().expect("charts");
    let sim = SimState::new(42, charts);
    for region in &sim.world.regions {
        for s in &region.settlements {
            let cells = house_cells(s.map_x as usize, s.map_y as usize, s.footprint() as usize);
            assert!(!cells.is_empty(), "{} has at least one roof", s.name);
            let mut roofs = 0;
            for (hx, hy) in &cells {
                if region.terrain.get(*hx, *hy) == Some(Terrain::House) {
                    roofs += 1;
                }
            }
            // A road may have cut one plot into street; most roofs stand.
            assert!(
                roofs * 2 >= cells.len(),
                "{} ({}): {}/{} roofs survive the roads",
                s.name,
                s.size,
                roofs,
                cells.len()
            );
        }
    }
}

#[test]
fn services_keep_their_own_doors() {
    let charts = load_charts().expect("charts");
    let sim = SimState::new(42, charts);
    let region = &sim.world.regions[0];
    let s = &region.settlements[0];
    let cells = house_cells(s.map_x as usize, s.map_y as usize, s.footprint() as usize);
    // The first house cell carries the first service, in order.
    if let Some(first_service) = s.services.first() {
        let (hx, hy) = cells[0];
        assert_eq!(
            service_at(s, hx, hy),
            Some(*first_service),
            "the first roof is the {:?}",
            first_service
        );
    }
    // A tile outside the footprint serves nothing.
    assert_eq!(service_at(s, 0, 0), None);
}

#[test]
fn streets_make_the_town_one_place() {
    // Standing on a street or under a roof both resolve to the settlement.
    use deep_world_tui::model::PlayerPos;
    use deep_world_tui::ui::app::App;
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let (ax, ay, n) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize, s.footprint() as usize)
    };
    for dy in 0..n {
        for dx in 0..n {
            a.player_pos = Some(PlayerPos {
                region_idx: 0,
                px: ax + dx,
                py: ay + dy,
            });
            assert_eq!(
                a.player_on_settlement(),
                Some((0, 0)),
                "tile ({},{}) is part of town",
                ax + dx,
                ay + dy
            );
        }
    }
}
