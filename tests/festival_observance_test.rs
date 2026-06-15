// Keeping a festival in earnest (#457): when a settlement's holy day is
// underway, deliberate observance deepens devotion to that festival's god and
// mends standing with its people — more than the passing nod of arrival.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{FestivalKind, ItemType, PlayerPos, SettlementService};
use deep_world_tui::ui::app::App;

fn total_affinity(a: &App) -> f64 {
    let g = &a.god_affinity;
    g.oltzed + g.keuru + g.sampsa + g.masa + g.kukri
}

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
    assert!(
        msg.contains("keep the"),
        "the status names the observance: {msg}"
    );
    assert!(
        a.clock.day >= day_before,
        "the festival takes some hours, not negative time"
    );
}

#[test]
fn deep_devotion_earns_the_gods_grace() {
    let mut a = app();
    stand_in_town(&mut a);
    set_festival(&mut a, true);

    let people = a.current_settlement_people().expect("standing in a town");
    let god = FestivalKind::for_people(people).patron_god();
    // Already "Devoted" — grace should answer.
    a.god_affinity.adjust(god, 0.8);
    a.vitals.thirst = 0.2; // passive observance never touches thirst; grace sets it to 1.0

    a.observe_festival();

    assert!(
        (a.vitals.thirst - 1.0).abs() < 1e-9,
        "the god's grace steadies the body whole (thirst {})",
        a.vitals.thirst
    );
}

#[test]
fn shallow_devotion_gets_no_grace() {
    let mut a = app();
    stand_in_town(&mut a);
    set_festival(&mut a, true);
    a.vitals.thirst = 0.2; // affinity starts near zero — below the Devoted threshold

    a.observe_festival();

    assert!(
        a.vitals.thirst < 0.5,
        "without deep devotion the grace does not come (thirst {})",
        a.vitals.thirst
    );
}

#[test]
fn devotion_ranks_climb_with_affinity() {
    use deep_world_tui::sim::god::devotion_rank;
    assert_eq!(devotion_rank(0.10), None);
    assert_eq!(devotion_rank(0.35), Some("a Keeper"));
    assert_eq!(devotion_rank(0.65), Some("Devoted"));
    assert_eq!(devotion_rank(0.90), Some("Blessed"));
}

#[test]
fn an_offering_spends_food_and_deepens_devotion() {
    let mut a = app();
    stand_in_town(&mut a);
    // a place of devotion, and food to give
    {
        let s = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0];
        if !s.services.contains(&SettlementService::Temple) {
            s.services.push(SettlementService::Temple);
        }
    }
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 3);
    let food_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    let aff_before = total_affinity(&a);

    a.make_offering();

    let food_after = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    assert_eq!(
        food_before - food_after,
        1,
        "the offering gives up one Food"
    );
    assert!(
        total_affinity(&a) > aff_before,
        "the offering deepens devotion to the god you keep"
    );
}

#[test]
fn an_offering_needs_food() {
    let mut a = app();
    stand_in_town(&mut a);
    {
        let s = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0];
        if !s.services.contains(&SettlementService::Temple) {
            s.services.push(SettlementService::Temple);
        }
    }
    // strip any starting food
    {
        let inv = &mut a.player_start.as_mut().unwrap().inventory;
        while inv.remove(ItemType::Food, 1) {}
    }
    a.make_offering();
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(msg.contains("needs Food"), "nothing to lay down: {msg}");
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
