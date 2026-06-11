// The town gate lives on the map (#372 PR 5): hostile peoples are turned
// back at the street's edge, a first step in counts the visit, and the old
// settlement menu is no longer the way in — doors, people, and the market
// stall are.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{PeopleKind, PlayerPos, Terrain};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 5; // before the streets fill: the gate, not a bump
    a
}

/// Stand on open ground just west of settlement 0's footprint, facing a
/// street tile (not a roof).
fn stand_at_town_edge(a: &mut App) -> bool {
    let (ax, ay, n) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize, s.footprint() as usize)
    };
    // Row ay+1 is street at the footprint's west edge (odd relative row).
    if n < 2 || ax == 0 {
        return false;
    }
    let outside = (ax - 1, ay + 1);
    let t = a.sim.as_ref().unwrap().world.regions[0]
        .terrain
        .get(outside.0, outside.1);
    if t.map(|t| t.passable()) != Some(true) {
        return false;
    }
    if a.sim.as_ref().unwrap().world.regions[0]
        .terrain
        .get(ax, ay + 1)
        != Some(Terrain::Settlement)
    {
        return false;
    }
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: outside.0,
        py: outside.1,
    });
    true
}

#[test]
fn hostile_guards_turn_you_back_at_the_edge() {
    let mut a = app();
    if !stand_at_town_edge(&mut a) {
        panic!("settlement edge must be reachable on this seed");
    }
    let np = {
        let p = &a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[0];
        PeopleKind::from_name(&p.people)
    };
    a.inter_people_bias.mod_toward(np, -2.0);
    let before = a.player_pos.unwrap();
    a.move_player(1, 0);
    let after = a.player_pos.unwrap();
    assert_eq!(
        (before.px, before.py),
        (after.px, after.py),
        "the guards hold the line"
    );
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("Guards"),
        "they say why: {:?}",
        a.status_msg
    );
}

#[test]
fn a_welcome_step_counts_the_visit_once() {
    let mut a = app();
    if !stand_at_town_edge(&mut a) {
        panic!("settlement edge must be reachable on this seed");
    }
    let v0 = a.milestones.settlements_visited;
    a.move_player(1, 0); // step in
    assert_eq!(
        a.milestones.settlements_visited,
        v0 + 1,
        "walking in is visiting"
    );
    let v1 = a.milestones.settlements_visited;
    // Moving within town (or bumping walls) is not re-visiting.
    a.move_player(0, 1);
    a.move_player(0, -1);
    assert_eq!(a.milestones.settlements_visited, v1, "no double counting");
}
