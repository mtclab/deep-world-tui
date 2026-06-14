// Completing the canon Five non-human peoples in play (#447): after the Khör
// (#443) and the Mëräk (#445), the last three barter in-kind on their own
// ground and take no coin — the Tzäkhar (deep stone), the Häl (forest canopy),
// the She'ar (desert edge). Each gives what their world makes for what the
// surface makes.
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
fn the_deep_people_keep_to_the_stone() {
    assert!(
        kinds_on(Terrain::Cave).contains(&EncounterKind::TzakharTrader),
        "Tzäkhar should appear at the cave-mouth"
    );
    for elsewhere in [
        Terrain::Grass,
        Terrain::Coast,
        Terrain::Sand,
        Terrain::Forest,
    ] {
        assert!(
            !kinds_on(elsewhere).contains(&EncounterKind::TzakharTrader),
            "Tzäkhar must not appear on {elsewhere:?}"
        );
    }
}

#[test]
fn the_canopy_people_keep_to_the_forest() {
    assert!(
        kinds_on(Terrain::Forest).contains(&EncounterKind::HalTrader),
        "Häl should appear in the forest"
    );
    for elsewhere in [
        Terrain::Grass,
        Terrain::Mountain,
        Terrain::Cave,
        Terrain::Coast,
    ] {
        assert!(
            !kinds_on(elsewhere).contains(&EncounterKind::HalTrader),
            "Häl must not appear on {elsewhere:?}"
        );
    }
}

#[test]
fn the_desert_people_keep_to_the_sand() {
    assert!(
        kinds_on(Terrain::Sand).contains(&EncounterKind::ShearTrader),
        "She'ar should appear on the sand"
    );
    assert!(
        kinds_on(Terrain::DeepDesert).contains(&EncounterKind::ShearTrader),
        "She'ar should appear in deep desert"
    );
    for elsewhere in [
        Terrain::Grass,
        Terrain::Forest,
        Terrain::Cave,
        Terrain::Coast,
    ] {
        assert!(
            !kinds_on(elsewhere).contains(&EncounterKind::ShearTrader),
            "She'ar must not appear on {elsewhere:?}"
        );
    }
}

fn app_with(kind: EncounterKind, terrain: Terrain) -> App {
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
        kind,
        terrain,
        species: None,
    });
    a
}

fn strip_coin(a: &mut App) {
    if let Some(ps) = a.player_start.as_mut() {
        let coin = ps.inventory.get(ItemType::Coin);
        ps.inventory.remove(ItemType::Coin, coin);
    }
}

#[test]
fn the_tzakhar_give_metal_for_food() {
    let mut a = app_with(EncounterKind::TzakharTrader, Terrain::Cave);
    strip_coin(&mut a);
    if let Some(ps) = a.player_start.as_mut() {
        ps.inventory.add(ItemType::Food, 2);
    }
    let iron0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Iron);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert!(inv.get(ItemType::Iron) > iron0, "received worked iron");
    assert!(inv.get(ItemType::Tool) >= 1, "received a tool");
    assert_eq!(inv.get(ItemType::Coin), 0, "the Tzäkhar take no coin");
}

#[test]
fn the_hal_give_medicine_for_make() {
    let mut a = app_with(EncounterKind::HalTrader, Terrain::Forest);
    strip_coin(&mut a);
    if let Some(ps) = a.player_start.as_mut() {
        ps.inventory.add(ItemType::Tool, 1);
    }
    let salve0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Salve);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert!(inv.get(ItemType::Salve) > salve0, "received canopy-salve");
    assert!(inv.get(ItemType::Herb) >= 2, "received herbs");
    assert_eq!(inv.get(ItemType::Coin), 0, "the Häl take no coin");
}

#[test]
fn the_shear_give_game_for_cloth() {
    let mut a = app_with(EncounterKind::ShearTrader, Terrain::Sand);
    strip_coin(&mut a);
    if let Some(ps) = a.player_start.as_mut() {
        ps.inventory.add(ItemType::Cloth, 1);
    }
    let food0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert!(inv.get(ItemType::Food) > food0, "received desert game");
    assert!(inv.get(ItemType::Herb) >= 1, "received succulent-physic");
    assert_eq!(inv.get(ItemType::Coin), 0, "the She'ar take no coin");
}

#[test]
fn the_five_refuse_coin() {
    for (kind, terrain) in [
        (EncounterKind::TzakharTrader, Terrain::Cave),
        (EncounterKind::HalTrader, Terrain::Forest),
        (EncounterKind::ShearTrader, Terrain::Sand),
    ] {
        let mut a = app_with(kind, terrain);
        if let Some(ps) = a.player_start.as_mut() {
            for it in [
                ItemType::Food,
                ItemType::Herb,
                ItemType::Tool,
                ItemType::Cloth,
            ] {
                let n = ps.inventory.get(it);
                ps.inventory.remove(it, n);
            }
            ps.inventory.add(ItemType::Coin, 50);
        }
        let coin0 = a
            .player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin);
        a.resolve_encounter(EncounterAction::Trade);
        let inv = &a.player_start.as_ref().unwrap().inventory;
        assert_eq!(
            inv.get(ItemType::Coin),
            coin0,
            "{kind:?} must not take coin"
        );
    }
}
