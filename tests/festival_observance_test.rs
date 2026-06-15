// Keeping a festival in earnest (#457): when a settlement's holy day is
// underway, deliberate observance deepens devotion to that festival's god and
// mends standing with its people — more than the passing nod of arrival.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{FestivalKind, PlayerPos};
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

fn set_festival(a: &mut App, underway: bool) {
    let day = a.clock.day;
    let s = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0];
    s.festival_until_day = if underway { day + 3 } else { 0 };
}

#[test]
fn keeping_a_festival_deepens_devotion() {
    let mut a = app();
    stand_in_town(&mut a);
    set_festival(&mut a, true);

    let people = a.current_settlement_people().expect("standing in a town");
    let god = FestivalKind::for_people(people).patron_god();
    let before = a.god_affinity.get(god);
    let day_before = a.clock.day;

    a.observe_festival();

    assert!(
        a.god_affinity.get(god) > before,
        "observance deepens devotion ({before} -> {})",
        a.god_affinity.get(god)
    );
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(msg.contains("keep the"), "the status names the observance: {msg}");
    assert!(
        a.clock.day >= day_before,
        "the festival takes some hours, not negative time"
    );
}

#[test]
fn no_festival_means_nothing_to_keep() {
    let mut a = app();
    stand_in_town(&mut a);
    set_festival(&mut a, false);

    a.observe_festival();

    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("no festival"),
        "with no holy day underway, there is nothing to keep: {msg}"
    );
}
