// Ask how a town stands with the province (#560 slice 4): the person names the
// town it holds its strongest standing with, reading the province-ties web.
use deep_world_tui::model::{PeopleKind, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let (mx, my) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: mx,
        py: my,
    });
    a
}

#[test]
fn an_npc_names_a_trade_partner() {
    let mut a = app();
    let town = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .name
        .clone();
    a.sim
        .as_mut()
        .unwrap()
        .province_ties
        .nudge(&town, "Farhaven", 0.8);
    a.status_msg = None;
    a.ask_province(0, 0, 0);
    let msg = a.status_msg.expect("a reply");
    assert!(msg.contains("Farhaven"), "names the partner town: {msg}");
    assert!(msg.contains("good trade"), "reads as a partnership: {msg}");
}

#[test]
fn the_distrusted_learn_nothing_of_the_province() {
    let mut a = app();
    let people = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        PeopleKind::from_name(&s.people[0].people)
    };
    a.inter_people_bias.mod_toward(people, -0.5);
    a.status_msg = None;
    a.ask_province(0, 0, 0);
    assert!(
        a.status_msg
            .as_deref()
            .unwrap_or("")
            .contains("keeps the town's dealings"),
        "rebuffed"
    );
}
