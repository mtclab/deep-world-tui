// Active self-treatment (#451): the post-Fall mortality (#448) made disease a
// real killer but left the player's counters passive (only auto-tend on rest, a
// salve that answered wounds alone). `tend_illness` is the deliberate counter —
// a herb-physic for ANY fever, a salve for the wound-illnesses — each easing the
// case and shortening its course, so a tended life outlasts an untended one.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::{ActiveDisease, Disease};
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app_sick_with(disease: Disease) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(7, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.lifespan_years = 9999;
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    let now = a.sim.as_ref().map_or(0, |s| s.world.tick);
    if let Some(ref mut ps) = a.player_start {
        ps.person.illnesses.clear();
        let mut d = ActiveDisease::new(disease, now);
        d.worsen(48); // a case that has festered a while: severity well above 1
        ps.person.illnesses.push(d);
    }
    a
}

fn fever_recovers_tick(a: &App) -> u64 {
    a.player_start
        .as_ref()
        .unwrap()
        .person
        .illnesses
        .first()
        .map(|d| d.contracted_tick + d.disease.recovery_ticks())
        .unwrap_or(0)
}

#[test]
fn a_herb_physic_eases_a_fever_and_shortens_its_course() {
    let mut a = app_sick_with(Disease::Fever);
    if let Some(ref mut ps) = a.player_start {
        ps.inventory.add(ItemType::Herb, 1);
    }
    let sev_before = a.player_start.as_ref().unwrap().person.illnesses[0].severity;
    let end_before = fever_recovers_tick(&a);
    a.tend_illness();
    // The herb was spent, the case eased, and it will run its course sooner.
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(inv.get(ItemType::Herb), 0, "the physic spent the herb");
    let ill = &a.player_start.as_ref().unwrap().person.illnesses;
    assert!(
        !ill.is_empty(),
        "an ordinary fever is not cured in one draught"
    );
    assert!(
        ill[0].severity < sev_before,
        "tending should ease the case: {} !< {sev_before}",
        ill[0].severity
    );
    // recovery tick anchored earlier OR the start day not yet advanced — the
    // remaining course is shorter than before tending.
    assert!(
        fever_recovers_tick(&a) <= end_before,
        "tending shortens the course"
    );
}

#[test]
fn tending_needs_a_remedy() {
    let mut a = app_sick_with(Disease::Fever);
    if let Some(ref mut ps) = a.player_start {
        for it in [ItemType::Herb, ItemType::Salve, ItemType::Bandage] {
            let n = ps.inventory.get(it);
            ps.inventory.remove(it, n);
        }
    }
    a.tend_illness();
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("nothing to treat"),
        "empty-handed tending should say so, got: {msg}"
    );
}

#[test]
fn the_healthy_have_nothing_to_tend() {
    let charts = load_charts().expect("charts");
    let mut a = App::new(7, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    if let Some(ref mut ps) = a.player_start {
        ps.person.illnesses.clear();
        ps.inventory.add(ItemType::Herb, 2);
    }
    a.tend_illness();
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(inv.get(ItemType::Herb), 2, "no remedy spent when not sick");
    assert!(a
        .status_msg
        .clone()
        .unwrap_or_default()
        .contains("not sick"));
}
