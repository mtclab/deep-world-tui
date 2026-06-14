// Town layout (#458): a settlement is streets and real buildings on the one
// map — every building reachable from its street through its door, every
// service at its own door, all of it derived (anchor + footprint + services),
// nothing persisted.
use deep_world_tui::gen::town::{service_at, town_buildings};
use deep_world_tui::model::Terrain;
use deep_world_tui::sim::SimState;
use deep_world_tui::charts::load::load_charts;

#[test]
fn every_town_has_real_buildings_on_the_map() {
    let charts = load_charts().expect("charts");
    let sim = SimState::new(42, charts);
    for region in &sim.world.regions {
        for s in &region.settlements {
            let buildings = town_buildings(s);
            assert!(!buildings.is_empty(), "{} has at least one building", s.name);
            // Each building's door is painted as a door, and most walls stand
            // (a road meets the town as its street, it does not gouge walls).
            let mut doors = 0;
            for b in &buildings {
                if region.terrain.get(b.door.0, b.door.1) == Some(Terrain::Door) {
                    doors += 1;
                }
            }
            assert!(
                doors * 2 >= buildings.len(),
                "{} ({}): {}/{} doors survive",
                s.name,
                s.size,
                doors,
                buildings.len()
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
    let buildings = town_buildings(s);
    // The first building carries the first service, in order.
    if let Some(first_service) = s.services.first() {
        let (dx, dy) = buildings[0].door;
        assert_eq!(
            service_at(s, dx, dy),
            Some(*first_service),
            "the first door is the {:?}",
            first_service
        );
    }
    // A tile outside any building's door serves nothing.
    assert_eq!(service_at(s, 0, 0), None);
}

#[test]
fn streets_make_the_town_one_place() {
    // Standing anywhere in the footprint resolves to the settlement.
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
