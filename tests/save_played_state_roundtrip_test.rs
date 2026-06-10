// Gate test: a *played* game's save must survive the exact compact-RON
// serialize -> deserialize path used by save_game/load_game. The prior
// save_slots_test only round-tripped a fresh game, so the journal
// "Expected option" bug shipped. This drives the real App through gameplay
// (moves that fire encounters + journal entries, gather, services) and asserts
// the accumulated state round-trips on several seeds.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::SettlementService;
use deep_world_tui::ui::app::App;

fn play_and_roundtrip(seed: u64, slot: usize) {
    let charts = load_charts().expect("charts");
    let mut app = App::new(seed, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    // Travel a while to accrue journal entries / fire encounters, gather, and
    // touch a settlement service if one is reachable.
    for _ in 0..6 {
        app.move_player(1, 0);
        app.gather();
    }
    app.move_player(0, 1);
    app.rest();
    app.use_service(SettlementService::Tavern);

    // Distinct per-seed slot so concurrent test threads don't clobber each
    // other or the user's real slot 1.
    app.save_to_slot(slot);

    // Exact production load path: compact-RON read + migrate().
    let loaded = deep_world_tui::save::load_game_file(&deep_world_tui::save::slot_filename(slot))
        .unwrap_or_else(|e| panic!("seed {seed}: played-state save failed to load: {e}"));

    assert_eq!(
        loaded.version,
        deep_world_tui::save_migrations::CURRENT_SAVE_VERSION
    );
}

#[test]
fn played_state_roundtrips_seed_555() {
    play_and_roundtrip(555, 2);
}

#[test]
fn played_state_roundtrips_seed_777() {
    play_and_roundtrip(777, 3);
}

#[test]
fn played_state_roundtrips_seed_42() {
    play_and_roundtrip(42, 4);
}
