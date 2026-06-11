// The AI-play action API (#PlayerChoice/CompactSave): the types existed with
// serde support but nothing ever applied, recorded, or replayed a choice.
// Now apply_choice is the headless action surface, and a recorded session
// replays deterministically on a fresh world with the same seed.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::save::{load_compact, save_compact, CompactSave, PlayerChoice};
use deep_world_tui::ui::app::App;

fn fresh(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

fn session() -> Vec<PlayerChoice> {
    vec![
        PlayerChoice::Gather,
        PlayerChoice::Rest,
        PlayerChoice::EnterSettlement {
            region_idx: 0,
            settlement_idx: 0,
        },
        PlayerChoice::BuyItem {
            item: "Food".into(),
        },
        PlayerChoice::UseService {
            service: "tavern".into(),
        },
        PlayerChoice::ExitSettlement,
        PlayerChoice::Gather,
        PlayerChoice::Rest,
    ]
}

fn fingerprint(a: &App) -> (u32, u32, usize, usize, u32, u32) {
    let inv = a.player_inventory();
    (
        a.clock.day,
        a.clock.hour,
        a.player_pos.map(|p| p.px).unwrap_or(0),
        a.player_pos.map(|p| p.py).unwrap_or(0),
        inv.get(deep_world_tui::model::ItemType::Coin),
        inv.get(deep_world_tui::model::ItemType::Food),
    )
}

#[test]
fn a_recorded_session_replays_deterministically() {
    let mut a = fresh(2718);
    let mut b = fresh(2718);
    for c in session() {
        a.apply_choice(&c);
        b.apply_choice(&c);
    }
    assert_eq!(
        fingerprint(&a),
        fingerprint(&b),
        "same seed + same choices must land on the same world state"
    );
}

#[test]
fn compact_save_roundtrips_and_replays() {
    let compact = CompactSave {
        seed: 2718,
        player_choices: session(),
        tick: 0,
    };
    save_compact(&compact, "ai_session_test.ron").expect("write");
    let loaded = load_compact("ai_session_test.ron").expect("read");
    assert_eq!(loaded.player_choices.len(), session().len());

    let mut live = fresh(loaded.seed);
    for c in &loaded.player_choices {
        live.apply_choice(c);
    }
    let mut direct = fresh(2718);
    for c in session() {
        direct.apply_choice(&c);
    }
    assert_eq!(
        fingerprint(&live),
        fingerprint(&direct),
        "a session loaded from disk replays to the same state"
    );
}

#[test]
fn every_choice_variant_is_applicable() {
    // No variant may be a silent no-op due to a parse gap.
    let mut a = fresh(99);
    for c in [
        PlayerChoice::Gather,
        PlayerChoice::Rest,
        PlayerChoice::Build,
        PlayerChoice::DismissCollapse,
        PlayerChoice::ExitSettlement,
        PlayerChoice::CraftRecipe { recipe_idx: 0 },
        PlayerChoice::UseService {
            service: "temple".into(),
        },
        PlayerChoice::BuyItem {
            item: "Herb".into(),
        },
        PlayerChoice::SellItem {
            item: "Herb".into(),
        },
        PlayerChoice::StealItem {
            item: "Herb".into(),
        },
        PlayerChoice::ResolveEncounter {
            action: "talk".into(),
        },
        PlayerChoice::Talk { person_idx: 0 },
        PlayerChoice::EnterSettlement {
            region_idx: 0,
            settlement_idx: 0,
        },
        PlayerChoice::TravelTo {
            region_idx: 0,
            px: 5,
            py: 5,
        },
    ] {
        a.apply_choice(&c); // must not panic
    }
    assert!(
        App::item_from_name("food").is_some() && App::item_from_name("FOOD").is_some(),
        "item parsing is case-insensitive"
    );
}
