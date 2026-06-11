// Household (#363): a living marriage and a fed larder bring children on the
// ten-day calendar; the eldest grown child is the heir before any friend is,
// through the same legacy machinery (keepsake, half standing, the house and
// what's in it).
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::HouseholdChild;
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 10;
    a
}

#[test]
fn a_fed_house_hears_small_feet() {
    let mut a = app();
    // A real spouse — a made-up id would read as a death on the first
    // day-tick and widow the house before any child came.
    let real = a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[0]
        .id
        .clone();
    a.spouse_id = Some(real);
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(deep_world_tui::model::ItemType::Food, 300);
        ps.inventory
            .add(deep_world_tui::model::ItemType::Water, 300);
        ps.companions.clear();
    }
    // The ten-day checks, walked directly (the gate wiring is shared with
    // the founding check, which has its own integration test).
    for day in (10..=400).step_by(10) {
        a.clock.day = day;
        a.tick_household();
        if !a.household_children.is_empty() {
            break;
        }
    }
    assert!(
        !a.household_children.is_empty(),
        "a wed, fed house should hear small feet within a few years"
    );
    let told = a
        .sim
        .as_ref()
        .unwrap()
        .journal
        .iter()
        .any(|e| e.text.contains("Born to us"));
    assert!(told, "the birth reaches the record");
}

#[test]
fn an_unwed_house_stays_quiet() {
    let mut a = app();
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(deep_world_tui::model::ItemType::Food, 300);
        ps.inventory
            .add(deep_world_tui::model::ItemType::Water, 300);
        ps.companions.clear();
    }
    for day in (10..=400).step_by(10) {
        a.clock.day = day;
        a.tick_household();
    }
    assert!(
        a.household_children.is_empty(),
        "no marriage, no household births"
    );
}

#[test]
fn the_eldest_grown_child_inherits() {
    let mut a = app();
    a.clock.day = 60; // 16+ life-years at 3 days/year
    a.household_children.push(HouseholdChild {
        name: "Vieno".into(),
        born_day: 0,
    });
    a.household_children.push(HouseholdChild {
        name: "Toivo".into(),
        born_day: 55, // still small
    });
    // Die of old age; the heir machinery runs.
    a.start_age_years = 80;
    a.birth_day = a.clock.day;
    a.lifespan_years = 80;
    a.vitals.hunger = 1.0;
    a.vitals.thirst = 1.0;
    a.vitals.energy = 1.0;
    a.spouse_id = Some("beloved".into());
    a.advance_clock(1);
    let heir = a.player_start.as_ref().unwrap().person.clone();
    assert_eq!(heir.name, "Vieno", "blood before friendship");
    assert_eq!(
        a.household_children.len(),
        1,
        "the heir leaves the children's roster"
    );
    assert_eq!(a.household_children[0].name, "Toivo");
    assert!(a.spouse_id.is_none(), "grief does not pass down");
}

#[test]
fn a_childless_line_falls_to_friends_as_before() {
    let mut a = app();
    a.start_age_years = 80;
    a.birth_day = a.clock.day;
    a.lifespan_years = 80;
    a.vitals.hunger = 1.0;
    a.vitals.thirst = 1.0;
    a.vitals.energy = 1.0;
    a.advance_clock(1);
    assert!(
        a.player_start.is_some(),
        "the friend-heir path still stands"
    );
    assert!(a.household_children.is_empty());
}
