// Ask an NPC who is worth knowing here (#528 conversations): a local names the
// folk a stranger would seek — gifted crafter, healer, smith, scribe, trader.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{PeopleKind, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(7, charts);
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
fn asking_who_to_know_gives_a_reply() {
    let mut a = app();
    stand_at_first_settlement(&mut a);
    a.status_msg = None;
    a.ask_folk(0, 0, 0);
    let msg = a.status_msg.expect("a reply");
    assert!(
        !msg.contains("names no one"),
        "a welcomed asker is not rebuffed: {msg}"
    );
    assert!(
        msg.contains("who to seek") || msg.contains("Plain folk here"),
        "either names folk or honestly says there are none: {msg}"
    );
}

#[test]
fn a_cold_welcome_names_no_one() {
    let mut a = app();
    let people = stand_at_first_settlement(&mut a);
    a.inter_people_bias.mod_toward(people, -0.5);
    a.status_msg = None;
    a.ask_folk(0, 0, 0);
    let msg = a.status_msg.expect("a reply");
    assert!(
        msg.contains("names no one"),
        "the distrusted learn nothing: {msg}"
    );
}
