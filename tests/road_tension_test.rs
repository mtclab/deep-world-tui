// Inter-people tension on the road (#9): the country of a people you are
// deeply at odds with is not safe to travel — their lawless harry you, so the
// encounter rate rises. The escalation ladder reaches the wilds.
use deep_world_tui::model::{PeopleKind, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

#[test]
fn deep_grudge_makes_the_roads_dangerous() {
    let mut a = app();
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 0,
        py: 0,
    });
    let region_people = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        PeopleKind::from_name(&s.people[0].people)
    };

    // Among your own people, the roads are no more dangerous for tension.
    if region_people == a.inter_people_bias.player_people {
        assert!((a.road_tension_mult() - 1.0).abs() < 1e-9);
        return;
    }

    let before = a.road_tension_mult();
    a.inter_people_bias.mod_toward(region_people, -0.35);
    let after = a.road_tension_mult();

    assert!(after >= before, "a grudge never makes the road safer");
    assert!(
        after > 1.0,
        "a deep grudge raises the danger on their roads (got {after})"
    );
}
