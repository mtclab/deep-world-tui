// God-vows (#457 item 4): a Blessed devotee may swear a vow to one of the Five
// for a lasting boon, at the cost of forsaking the rest — and the vow breaks if
// the kept god is neglected below Devoted.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{GodName, ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 0,
        py: 0,
    });
    a
}

#[test]
fn a_vow_grants_its_gods_boon() {
    let mut a = app();
    // No vow: the boon modifiers are neutral.
    assert_eq!(a.vow_buy_mult(), 1.0);
    assert_eq!(a.vow_work_energy_mult(), 1.0);
    assert_eq!(a.vow_encounter_mult(), 1.0);
    assert_eq!(a.vow_weather_mult(), 1.0);
    assert_eq!(a.vow_rest_bonus(), 1.0);

    a.god_vow = Some(GodName::Masa);
    assert!(a.vow_buy_mult() < 1.0, "Masa's vow buys the kinder");
    // Only the sworn god's boon runs.
    assert_eq!(a.vow_work_energy_mult(), 1.0);

    a.god_vow = Some(GodName::Oltzed);
    assert!(
        a.vow_work_energy_mult() < 1.0,
        "Oltzed's vow tires you less"
    );

    a.god_vow = Some(GodName::Sampsa);
    assert!(a.vow_encounter_mult() < 1.0, "Sampsa's vow reads the road");

    a.god_vow = Some(GodName::Kukri);
    assert!(a.vow_weather_mult() < 1.0, "Kukri's vow bears the cold");

    a.god_vow = Some(GodName::Keuru);
    assert!(a.vow_rest_bonus() > 1.0, "Keuru's vow rests you deeper");
}

#[test]
fn masas_vow_lowers_the_buy_price() {
    let mut a = app();
    let plain = a.quote_buy_price(ItemType::Coat);
    a.god_vow = Some(GodName::Masa);
    let vowed = a.quote_buy_price(ItemType::Coat);
    assert!(
        vowed <= plain,
        "the vow never raises the price (plain {plain}, vowed {vowed})"
    );
    assert!(vowed < plain, "a meaningful good is cheaper under the vow");
}

#[test]
fn a_vow_needs_a_temple_and_a_blessing() {
    let mut a = app();
    // Out on the open map, no temple, no blessing: nothing to swear.
    a.take_vow();
    assert_eq!(a.god_vow, None, "no vow without a temple and a Blessed god");
}

#[test]
fn a_neglected_vow_breaks_of_itself() {
    let mut a = app();
    // Sworn to Masa, then let the devotion fall below Devoted.
    a.god_vow = Some(GodName::Masa);
    a.god_affinity = Default::default(); // all gods at 0.0, below Devoted (0.60)
    a.check_vow();
    assert_eq!(a.god_vow, None, "a vow kept too thinly breaks");
}

#[test]
fn a_held_vow_endures_while_devotion_holds() {
    let mut a = app();
    a.god_vow = Some(GodName::Masa);
    a.god_affinity.adjust(GodName::Masa, 0.9); // well above Devoted
    a.check_vow();
    assert_eq!(
        a.god_vow,
        Some(GodName::Masa),
        "kept devotion keeps the vow"
    );
}

#[test]
fn renouncing_a_vow_gives_it_back_and_cools_the_god() {
    let mut a = app();
    a.god_vow = Some(GodName::Masa);
    a.god_affinity.adjust(GodName::Masa, 0.9);
    let before = a.god_affinity.get(GodName::Masa);
    a.take_vow(); // with a vow held, this renounces it
    assert_eq!(a.god_vow, None, "the vow is given back");
    assert!(
        a.god_affinity.get(GodName::Masa) < before,
        "the forsaken god is colder for the renouncing"
    );
}
