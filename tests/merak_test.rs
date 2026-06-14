// The Mëräk exchange (#445): the deep-sea people — coast only, barter
// deep-water goods (fish, deep-glass) for surface make (cloth, tools), no coin.
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
fn the_merak_keep_to_the_tideline() {
    assert!(
        kinds_on(Terrain::Coast).contains(&EncounterKind::MerakTrader),
        "Mëräk should appear on the coast"
    );
    for inland in [
        Terrain::Grass,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Tundra,
    ] {
        assert!(
            !kinds_on(inland).contains(&EncounterKind::MerakTrader),
            "Mëräk must not appear on {inland:?}"
        );
    }
}

fn app_with_merak() -> App {
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
        kind: EncounterKind::MerakTrader,
        terrain: Terrain::Coast,
        species: None,
    });
    a
}

#[test]
fn the_merak_barter_surface_make_for_deep_goods() {
    let mut a = app_with_merak();
    if let Some(ps) = a.player_start.as_mut() {
        ps.inventory.add(ItemType::Tool, 2);
        let coin = ps.inventory.get(ItemType::Coin);
        ps.inventory.remove(ItemType::Coin, coin);
    }
    let food0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(inv.get(ItemType::Tool), 1, "one tool given");
    assert!(inv.get(ItemType::Food) > food0, "received deep-fish");
    assert!(inv.get(ItemType::Glass) >= 1, "received deep-glass");
    assert_eq!(inv.get(ItemType::Coin), 0, "the Mëräk take no coin");
}

#[test]
fn the_merak_refuse_coin() {
    let mut a = app_with_merak();
    if let Some(ps) = a.player_start.as_mut() {
        for it in [ItemType::Tool, ItemType::Cloth] {
            let n = ps.inventory.get(it);
            ps.inventory.remove(it, n);
        }
        ps.inventory.add(ItemType::Coin, 50);
    }
    let food0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    let coin0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(inv.get(ItemType::Food), food0, "coin buys nothing");
    assert_eq!(inv.get(ItemType::Coin), coin0, "coin untouched");
}
