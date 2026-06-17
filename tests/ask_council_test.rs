// Ask who holds the council (#556 living politics): the person names the
// settlement's dominant faction and how firmly it sits, reading the politics
// layer that drifts and turns over on its own.
use deep_world_tui::model::economy::Faction;
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
fn an_npc_names_who_holds_the_council() {
    let mut a = app();
    // Set a clear ruling faction so the reply is deterministic.
    {
        let s = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0];
        s.politics.trader_standing = 0.6;
        s.politics.crafter_standing = 0.25;
        s.politics.elder_standing = 0.15;
    }
    a.status_msg = None;
    a.ask_council(0, 0, 0);
    let msg = a.status_msg.expect("a reply");
    assert!(
        msg.contains(Faction::Traders.label()),
        "names the ruling faction: {msg}"
    );
    assert!(
        msg.contains("hold the council"),
        "reads as council talk: {msg}"
    );
}

#[test]
fn the_distrusted_learn_nothing_of_the_council() {
    let mut a = app();
    let people = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        PeopleKind::from_name(&s.people[0].people)
    };
    a.inter_people_bias.mod_toward(people, -0.5);
    a.status_msg = None;
    a.ask_council(0, 0, 0);
    assert!(
        a.status_msg
            .as_deref()
            .unwrap_or("")
            .contains("no business of your kind"),
        "rebuffed"
    );
}
