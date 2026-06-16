// Ask an NPC the way (#528 conversations): a local points you toward the
// nearest settlement and a notable place, gated by how they regard you.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{PeopleKind, PlayerPos};
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

fn stand_at_first_settlement(a: &mut App) -> PeopleKind {
    let (mx, my, people) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (
            s.map_x as usize,
            s.map_y as usize,
            PeopleKind::from_name(&s.people[0].people),
        )
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: mx,
        py: my,
    });
    people
}

#[test]
fn asking_the_way_points_somewhere() {
    let mut a = app();
    stand_at_first_settlement(&mut a);
    a.status_msg = None;
    a.ask_directions(0, 0, 0);
    let msg = a.status_msg.expect("asking the way yields a reply");
    assert!(
        !msg.contains("only shrugs"),
        "a welcomed asker is given the road: {msg}"
    );
    // Region 0 has more than one settlement in seed 42, so a bearing is named.
    assert!(
        msg.contains("points the way") || msg.contains("as far as the road goes"),
        "the reply is a direction or honest end-of-road: {msg}"
    );
}

#[test]
fn a_cold_welcome_points_nowhere() {
    let mut a = app();
    let people = stand_at_first_settlement(&mut a);
    a.inter_people_bias.mod_toward(people, -0.5);
    a.status_msg = None;
    a.ask_directions(0, 0, 0);
    let msg = a.status_msg.expect("reply");
    assert!(
        msg.contains("only shrugs"),
        "the distrusted get no road: {msg}"
    );
}
