// Street life (#372 PR 4 / #458): the settlement's people stand in its streets
// by day and indoors by night, deterministically — and what you see is who you
// can meet: stepping into someone greets them. At night the streets go quiet
// because everyone has gone inside, where you can still walk in and meet them.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::town::npc_street_positions;
use deep_world_tui::model::PlayerPos;
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 10;
    a
}

#[test]
fn the_streets_fill_by_day_and_empty_by_night() {
    let a = app();
    let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
    let day = npc_street_positions(s, 5, 10);
    let night = npc_street_positions(s, 5, 23);
    assert!(!day.is_empty(), "people stand in the street at mid-morning");
    assert!(!night.is_empty(), "people are home, indoors, at night");
    // Deterministic: the same woman keeps her corner all day, her room all night.
    assert_eq!(day, npc_street_positions(s, 5, 14));
    assert_eq!(night, npc_street_positions(s, 5, 2));
    // By day everyone stands on street ground, nobody inside a wall.
    for &(_, x, y) in &day {
        assert!(
            !deep_world_tui::gen::town::is_house_of(s, x, y),
            "({x},{y}) is street, not roof"
        );
        assert!(s.contains_tile(x, y));
    }
    // By night everyone is on an interior floor — inside a building, in town.
    let region = &a.sim.as_ref().unwrap().world.regions[0];
    for &(_, x, y) in &night {
        assert!(
            deep_world_tui::gen::town::is_house_of(s, x, y),
            "({x},{y}) is indoors at night"
        );
        assert_eq!(
            region.terrain.get(x, y),
            Some(deep_world_tui::model::Terrain::Floor),
            "({x},{y}) is a walkable interior floor"
        );
    }
}

#[test]
fn bumping_into_someone_greets_them() {
    let mut a = app();
    let (pi, nx, ny) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        let mut ps = npc_street_positions(s, a.clock.day, a.clock.hour);
        ps.pop().expect("someone is out")
    };
    // Stand beside them on the street (east side; fall back to west).
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: nx + 1,
        py: ny,
    });
    a.move_player(-1, 0);
    match a.screen {
        deep_world_tui::ui::app::Screen::Talk { person_idx, .. } => {
            assert_eq!(person_idx, pi, "you greet exactly who you walked into")
        }
        _ => panic!("a bump should open talk"),
    }
}
