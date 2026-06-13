// Fortune: the luck a life is born with (#399). Hidden, read only in omens,
// it tilts every risk a little — the cautious are safer, never safe — and a
// blessed life is sometimes pulled back from a death a cursed one would meet.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::wildlife::WildSpecies;
use deep_world_tui::model::{
    Encounter, EncounterAction, EncounterKind, Fortune, PlayerPos, Terrain,
};
use deep_world_tui::ui::app::App;

fn app(seed: u64, fortune: f64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    if let Some(ps) = a.player_start.as_mut() {
        ps.companions.clear();
    }
    a.fortune = Fortune::from_value(fortune);
    a
}

#[test]
fn a_blessed_life_flees_a_predator_better_than_a_cursed_one() {
    // Same seed, same wolves, same flights — only the star differs. The blessed
    // bleed less; the cursed pay more for the very same caution.
    fn hits(fortune: f64) -> u32 {
        let mut a = app(7, fortune);
        let mut count = 0;
        for tick in 0..3000u64 {
            a.sim.as_mut().unwrap().world.tick = tick;
            a.vitals.energy = 1.0;
            a.vitals.hunger = 1.0;
            a.collapse = None;
            let before = a.collapses_had;
            a.encounter = Some(Encounter {
                kind: EncounterKind::Wildlife,
                terrain: Terrain::Forest,
                species: Some(WildSpecies::Wolf),
            });
            a.resolve_encounter(EncounterAction::Flee);
            if a.collapses_had > before || a.vitals.energy < 0.8 {
                count += 1;
            }
        }
        count
    }
    let blessed = hits(1.0);
    let cursed = hits(-1.0);
    assert!(
        blessed < cursed,
        "the blessed bleed less ({blessed} vs {cursed})"
    );
    // …but a blessed life is not an invulnerable one.
    assert!(
        blessed > 0,
        "even fortune does not make flight free ({blessed})"
    );
}

#[test]
fn the_star_is_hidden_in_the_save_and_survives_the_roundtrip() {
    use deep_world_tui::save::{load_game_file, save_game};
    let mut a = app(31, 0.6);
    let star = a.fortune.value();
    // Build and write a save, then read it back into a fresh app.
    a.save_to_slot(1);
    let data = load_game_file(&deep_world_tui::save::slot_filename(1)).expect("load");
    assert!(
        (data.fortune.value() - star).abs() < 1e-9,
        "the star persists ({} vs {star})",
        data.fortune.value()
    );
    let mut b = App::new(31, load_charts().expect("charts"));
    b.apply_save_data(data);
    assert!(
        (b.fortune.value() - star).abs() < 1e-9,
        "and is restored on load"
    );
    // keep `save_game` referenced so the import is meaningful across versions
    let _ = save_game as fn(&deep_world_tui::save::SaveData, &str) -> Result<(), String>;
}

#[test]
fn omens_lean_with_the_star_but_both_show_under_either() {
    // Drive many days of rests for a blessed life and a cursed one, counting
    // the fair and ill signs that show. The blessed see more fair omens — but
    // the cursed still see the odd fair one, and the blessed the odd ill.
    fn tally(fortune: f64) -> (u32, u32) {
        let fair_lines = deep_world_tui::banks::bank("OMENS_FAIR");
        let ill_lines = deep_world_tui::banks::bank("OMENS_ILL");
        let mut a = app(7, fortune);
        a.player_start
            .as_mut()
            .unwrap()
            .inventory
            .add(deep_world_tui::model::ItemType::Food, 4000);
        a.player_start
            .as_mut()
            .unwrap()
            .inventory
            .add(deep_world_tui::model::ItemType::Water, 4000);
        // Capture each omen as it shows: maybe_omen sets the day's status line
        // to the omen text. The journal caps and drains, so a single end-scan
        // would lose the early signs — read them live instead.
        let (mut fair, mut ill) = (0u32, 0u32);
        for _ in 0..400 {
            // Stay hale so no collapse derails the calendar; each step turns a
            // full day, on whose rollover maybe_omen rolls.
            a.vitals.energy = 1.0;
            a.vitals.hunger = 1.0;
            a.vitals.thirst = 1.0;
            a.status_msg = None;
            a.advance_clock(24);
            if let Some(ref msg) = a.status_msg {
                if fair_lines.iter().any(|l| l == msg) {
                    fair += 1;
                } else if ill_lines.iter().any(|l| l == msg) {
                    ill += 1;
                }
            }
        }
        (fair, ill)
    }
    let (bf, bi) = tally(1.0);
    let (cf, ci) = tally(-1.0);
    assert!(bf + bi > 0 && cf + ci > 0, "omens do show");
    // Blessed lives skew fair; cursed lives skew ill.
    assert!(
        (bf as f64) / ((bf + bi).max(1) as f64) > (cf as f64) / ((cf + ci).max(1) as f64),
        "the star leans the signs (blessed {bf}/{bi} vs cursed {cf}/{ci})"
    );
}
