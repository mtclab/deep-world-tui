// Ask an NPC for news (#528 conversations): word of the world comes through a
// person, gated by how they regard you — a cold welcome gives nothing.
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

/// Stand the player at region 0's first settlement; return its first person's
/// people and id.
fn stand_at_first_settlement(a: &mut App) -> (PeopleKind, String) {
    let (mx, my, people, _id) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        let p = &s.people[0];
        (
            s.map_x as usize,
            s.map_y as usize,
            PeopleKind::from_name(&p.people),
            p.id.clone(),
        )
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: mx,
        py: my,
    });
    (people, _id)
}

#[test]
fn asking_for_news_gives_an_answer() {
    let mut a = app();
    stand_at_first_settlement(&mut a);
    a.status_msg = None;
    a.ask_news(0, 0, 0);
    let msg = a.status_msg.expect("asking yields a reply");
    // A welcomed asker hears something — either word, or honest quiet — never
    // the cold-shoulder line.
    assert!(
        !msg.contains("no word for the likes of you"),
        "a neutral/welcomed asker is not rebuffed: {msg}"
    );
}

#[test]
fn a_cold_welcome_gives_no_news() {
    let mut a = app();
    let (people, _id) = stand_at_first_settlement(&mut a);
    // Sour the standing with this people well past the rebuff line.
    a.inter_people_bias.mod_toward(people, -0.5);
    a.status_msg = None;
    a.ask_news(0, 0, 0);
    let msg = a.status_msg.expect("asking yields a reply");
    assert!(
        msg.contains("no word for the likes of you"),
        "a deeply-distrusted asker is rebuffed: {msg}"
    );
}

#[test]
fn the_same_person_tells_the_same_tale_that_day() {
    let mut a = app();
    stand_at_first_settlement(&mut a);
    a.ask_news(0, 0, 0);
    let first = a.status_msg.clone();
    a.ask_news(0, 0, 0);
    let second = a.status_msg.clone();
    assert_eq!(first, second, "news is stable per person per day");
}
