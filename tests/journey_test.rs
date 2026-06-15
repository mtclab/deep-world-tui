// The wider world (#456): the playable map is a province slice — the named
// cities of the continent never stand on it — but from a town on the roads you
// can make the days-long journey to one and back, returning with its goods and
// word of the wider world.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 8;
    a
}

fn stand_in_town(a: &mut App) {
    let (mx, my) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: mx,
        py: my,
    });
}

const CITIES: [&str; 5] = [
    "Sampa Crossing",
    "Vessenath",
    "Halkess",
    "Velkarath",
    "Keuramark",
];

#[test]
fn a_journey_reaches_a_named_city_and_comes_home() {
    let mut a = app();
    stand_in_town(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 10);
    let day_before = a.clock.day;
    a.journey_to_city();
    // Several days pass on the road.
    assert!(
        a.clock.day >= day_before + 2,
        "the road takes days ({} -> {})",
        day_before,
        a.clock.day
    );
    // You came home alive, with word of a named city.
    assert!(a.player_start.is_some(), "the traveller comes home");
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        CITIES.iter().any(|c| msg.contains(c)),
        "the journey names a city of the continent: {msg}"
    );
    assert!(msg.contains("long roads"), "it was a real journey: {msg}");
    // ...and word of the wider world comes home with you.
    assert!(
        msg.contains("Word travels"),
        "the journey brings home news: {msg}"
    );
}

#[test]
fn a_journey_needs_provisions_and_a_town() {
    // No provisions: refused, no days lost.
    let mut a = app();
    stand_in_town(&mut a);
    let day = a.clock.day;
    a.journey_to_city();
    assert_eq!(a.clock.day, day, "an unprovisioned journey does not happen");
    assert!(
        a.status_msg
            .clone()
            .unwrap_or_default()
            .contains("provision"),
        "it asks for provisions"
    );

    // Out in the wilds (not on a settlement): refused.
    let mut a = app();
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 10);
    // Park far from any town.
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 0,
        py: 0,
    });
    let day = a.clock.day;
    a.journey_to_city();
    if a.current_settlement().is_none() {
        assert_eq!(a.clock.day, day, "no journey from the open wilds");
    }
}
