// Numbered manual save slots: each slot is an independent file, saved compactly.

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::save::{load_game, slot_filename, SAVE_SLOT_COUNT};
use deep_world_tui::ui::app::App;

fn fresh_app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut app = App::new(seed, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app
}

#[test]
fn slots_are_independent_files() {
    for s in 1..=SAVE_SLOT_COUNT {
        let _ = std::fs::remove_file(format!("saves/{}", slot_filename(s)));
    }

    // Two different runs into two different slots.
    let mut a = fresh_app(111);
    a.save_to_slot(2);
    let mut b = fresh_app(222);
    b.save_to_slot(4);

    // Both slot files exist and load with their own seed/world.
    let in_2 = load_game(&slot_filename(2)).expect("slot 2 loads");
    let in_4 = load_game(&slot_filename(4)).expect("slot 4 loads");
    assert_eq!(in_2.sim.world.seed, 111);
    assert_eq!(in_4.sim.world.seed, 222);

    // Slot 4 untouched by saving slot 2 again.
    a.save_to_slot(2);
    let in_4_again = load_game(&slot_filename(4)).expect("slot 4 still loads");
    assert_eq!(in_4_again.sim.world.seed, 222);

    for s in 1..=SAVE_SLOT_COUNT {
        let _ = std::fs::remove_file(format!("saves/{}", slot_filename(s)));
    }
}
