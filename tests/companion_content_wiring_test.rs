// Regression for the dead companion/rest content wired in this PR:
// - per-animal upkeep rates (food_per_tick/rest_per_tick) now drive decay_needs
// - milk_production yields Food on a proper rest
// - scouting_bonus widens the reveal radius
// - a structure on the tile raises the rest quality tier (deep night inside
//   your own walls is no longer "out in the cold")
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::companion::{Animal, Companion};
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::structures::{BuildKind, Structure};
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

#[test]
fn upkeep_scales_per_animal() {
    let mut horse = Companion::new(Animal::Horse, "H".into(), 0);
    let mut dog = Companion::new(Animal::Dog, "D".into(), 0);
    let mut eel = Companion::new(Animal::RiverEel, "E".into(), 0);
    horse.decay_needs(10);
    dog.decay_needs(10);
    eel.decay_needs(10);
    assert!(
        horse.food_need > dog.food_need,
        "a horse ({}) should hunger faster than a dog ({})",
        horse.food_need,
        dog.food_need
    );
    assert_eq!(eel.food_need, 0.0, "a river eel asks for nothing");
}

#[test]
fn goat_yields_milk_on_proper_rest() {
    let mut a = app();
    a.clock.hour = 10; // not deep night
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.companions.clear();
        ps.companions
            .push(Companion::new(Animal::Goat, "G".into(), 0));
        let food = ps.inventory.get(ItemType::Food);
        ps.inventory.remove(ItemType::Food, food);
    }
    a.rest_hours(8);
    let food = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    assert!(
        food >= 1,
        "a content goat should yield milk (Food), got {food}"
    );
}

#[test]
fn falcon_widens_reveal_radius() {
    let mut with_falcon = app();
    let mut without = app();
    with_falcon
        .player_start
        .as_mut()
        .unwrap()
        .companions
        .push(Companion::new(Animal::Falcon, "F".into(), 0));

    // Reveal from the same fresh spot in both and compare explored counts.
    let pos = without.player_pos.unwrap();
    let count = |a: &App| {
        let m = &a.explored[pos.region_idx];
        (0..40)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .filter(|&(x, y)| m.is_explored(x, y))
            .count()
    };
    let blank = deep_world_tui::model::ExploredMap::new(40, 40);
    with_falcon.explored[pos.region_idx] = blank.clone();
    without.explored[pos.region_idx] = blank;
    with_falcon.reveal_around(pos.region_idx, 20, 20);
    without.reveal_around(pos.region_idx, 20, 20);
    assert!(
        count(&with_falcon) > count(&without),
        "falcon should widen the revealed area ({} vs {})",
        count(&with_falcon),
        count(&without)
    );
}

#[test]
fn deep_night_in_own_home_is_not_out_in_the_cold() {
    let mut a = app();
    a.clock.hour = 2; // deep night
    let pos = a.player_pos.unwrap();
    a.sim.as_mut().unwrap().world.regions[pos.region_idx]
        .structures
        .push(Structure {
            kind: BuildKind::Home,
            region_idx: pos.region_idx,
            x: pos.px as u32,
            y: pos.py as u32,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
    a.rest_hours(8);
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        !msg.contains("Cold. I do not sleep."),
        "resting in your own Home at deep night must not be out-in-the-cold, got: {msg}"
    );
}
