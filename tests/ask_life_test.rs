// Ask an NPC about their life (#548/#528): they name the bonds that matter,
// reading the living social web.
use deep_world_tui::model::relation::{InterNpcRelation, RelationKind};
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
fn an_npc_names_their_bonds() {
    let mut a = app();
    // Settlement 0 needs at least two people; give person 0 a sworn-friend bond
    // to person 1.
    let n = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .people
        .len();
    if n < 2 {
        return; // nothing to test in a one-soul settlement
    }
    let friend_name = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        s.people[1].name.clone()
    };
    let friend_id = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        s.people[1].id.clone()
    };
    {
        let s = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0];
        s.people[0].relations = vec![InterNpcRelation {
            kind: RelationKind::SwornFriend,
            target_person_id: friend_id,
            intensity: 0.9,
            formed_at_tick: 0,
            reason: "the long nearness of neighbours".into(),
        }];
    }
    a.status_msg = None;
    a.ask_about_life(0, 0, 0);
    let msg = a.status_msg.expect("a reply");
    assert!(msg.contains("sworn friend"), "names the bond kind: {msg}");
    assert!(msg.contains(&friend_name), "names the friend: {msg}");
}

#[test]
fn the_distrusted_get_no_confidences() {
    let mut a = app();
    let people = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        PeopleKind::from_name(&s.people[0].people)
    };
    a.inter_people_bias.mod_toward(people, -0.5);
    a.status_msg = None;
    a.ask_about_life(0, 0, 0);
    assert!(
        a.status_msg
            .as_deref()
            .unwrap_or("")
            .contains("keeps their own counsel"),
        "rebuffed"
    );
}
