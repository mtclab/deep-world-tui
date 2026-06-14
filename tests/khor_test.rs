// The Khör rendezvous (#443): the first non-human people — cold-terrain only,
// barter härkä goods for metal, take no coin, do not haggle.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::ItemType;
use deep_world_tui::model::{Encounter, EncounterAction, EncounterKind, PlayerPos, Terrain};
use deep_world_tui::ui::app::App;

fn kinds_on(terrain: Terrain) -> Vec<EncounterKind> {
    (0..4000u64)
        .filter_map(|seed| {
            Encounter::roll_biased_weather(terrain, 10, 1, seed, None, 1.0).map(|e| e.kind)
        })
        .collect()
}

#[test]
fn the_khor_keep_to_the_cold() {
    let tundra = kinds_on(Terrain::Tundra);
    let mountain = kinds_on(Terrain::Mountain);
    assert!(
        tundra.contains(&EncounterKind::KhorTrader),
        "Khör should appear on the tundra"
    );
    assert!(
        mountain.contains(&EncounterKind::KhorTrader),
        "Khör should appear in the mountains"
    );
    // Never in the warm lowlands.
    for warm in [Terrain::Grass, Terrain::Forest, Terrain::Farmland] {
        assert!(
            !kinds_on(warm).contains(&EncounterKind::KhorTrader),
            "Khör must not appear on {warm:?}"
        );
    }
}

fn app_with_khor() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(7, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    a.encounter = Some(Encounter {
        kind: EncounterKind::KhorTrader,
        terrain: Terrain::Tundra,
        species: None,
    });
    a
}

#[test]
fn the_khor_barter_metal_for_hide_and_meat() {
    let mut a = app_with_khor();
    if let Some(ps) = a.player_start.as_mut() {
        ps.inventory.add(ItemType::Iron, 3);
        // Strip coin so we know none of it moves.
        let coin = ps.inventory.get(ItemType::Coin);
        ps.inventory.remove(ItemType::Coin, coin);
    }
    let hide0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Hide);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(inv.get(ItemType::Iron), 2, "one iron given");
    assert!(inv.get(ItemType::Hide) > hide0, "received härkä-leather");
    assert!(inv.get(ItemType::Food) >= 1, "received steppe-butter");
    assert_eq!(inv.get(ItemType::Coin), 0, "the Khör take no coin");
}

#[test]
fn the_khor_refuse_a_buyer_with_no_metal() {
    let mut a = app_with_khor();
    if let Some(ps) = a.player_start.as_mut() {
        let iron = ps.inventory.get(ItemType::Iron);
        ps.inventory.remove(ItemType::Iron, iron);
        let tool = ps.inventory.get(ItemType::Tool);
        ps.inventory.remove(ItemType::Tool, tool);
        ps.inventory.add(ItemType::Coin, 50);
    }
    let hide0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Hide);
    let coin0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(
        inv.get(ItemType::Hide),
        hide0,
        "coin buys nothing from the Khör"
    );
    assert_eq!(inv.get(ItemType::Coin), coin0, "coin untouched");
}
