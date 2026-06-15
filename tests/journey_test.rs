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

#[test]
fn the_first_great_journey_marks_the_life() {
    use deep_world_tui::sim::milestones::MilestoneKind;
    let mut a = app();
    stand_in_town(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 20);
    assert!(!a.milestones.has(MilestoneKind::WalkedToGreatCity));
    a.journey_to_city();
    assert!(
        a.milestones.has(MilestoneKind::WalkedToGreatCity),
        "the first journey to a great city marks the life"
    );
}

#[test]
fn the_long_haul_turns_a_profit() {
    // Carry bulk goods to the city and they sell at a premium for coin (#456).
    let mut a = app();
    stand_in_town(&mut a);
    {
        let inv = &mut a.player_start.as_mut().unwrap().inventory;
        inv.add(ItemType::Food, 10);
        // Cloth and Stone are haulable and no city sends them home, so they
        // are a clean test of the sale (unlike Hide/Iron, which Vessenath ships).
        inv.add(ItemType::Cloth, 3);
        inv.add(ItemType::Stone, 2);
    }
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);

    a.journey_to_city();

    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert!(
        inv.get(ItemType::Coin) > coin_before,
        "the haul comes home as coin ({} -> {})",
        coin_before,
        inv.get(ItemType::Coin)
    );
    assert_eq!(inv.get(ItemType::Cloth), 0, "the hauled cloth was sold");
    assert_eq!(inv.get(ItemType::Stone), 0, "the hauled stone was sold");
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(msg.contains("sold at"), "the trade is reported: {msg}");
}
