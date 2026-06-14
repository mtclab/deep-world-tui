// The threshold toll (#455, Mythic Phase 2): an old woman keeping a fire at a
// cave-mouth in the high rock, who asks a price for the road past. Pay (Trade)
// for easy passage and the threshold-keeper's favour; refuse (Flee) and take the
// long way. Deniable as a strange hermit.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{
    Encounter, EncounterAction, EncounterKind, GodName, ItemType, PlayerPos, Terrain,
};
use deep_world_tui::ui::app::App;

fn kinds_on(terrain: Terrain) -> Vec<EncounterKind> {
    (0..4000u64)
        .filter_map(|seed| {
            Encounter::roll_biased_weather(terrain, 10, 1, seed, None, 1.0).map(|e| e.kind)
        })
        .collect()
}

#[test]
fn the_threshold_keeper_waits_in_the_high_rock() {
    assert!(
        kinds_on(Terrain::Mountain).contains(&EncounterKind::ThresholdToll),
        "the threshold toll should appear on the mountain"
    );
    assert!(
        !kinds_on(Terrain::Grass).contains(&EncounterKind::ThresholdToll),
        "not out on the open grass"
    );
}

fn app_at_threshold() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(5, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.vitals.energy = 1.0;
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    a.encounter = Some(Encounter {
        kind: EncounterKind::ThresholdToll,
        terrain: Terrain::Mountain,
        species: None,
    });
    a
}

#[test]
fn paying_the_toll_costs_a_good_and_earns_kukris_favour() {
    let mut a = app_at_threshold();
    if let Some(ref mut ps) = a.player_start {
        // Strip herb/food so the coin path is taken predictably.
        for it in [ItemType::Herb, ItemType::Food] {
            let n = ps.inventory.get(it);
            ps.inventory.remove(it, n);
        }
        ps.inventory.add(ItemType::Coin, 10);
    }
    let coin0 = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    let kukri0 = a.god_affinity.get(GodName::Kukri);
    a.resolve_encounter(EncounterAction::Trade);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(
        inv.get(ItemType::Coin),
        coin0 - 3,
        "the toll took three coins"
    );
    assert!(
        a.god_affinity.get(GodName::Kukri) > kukri0,
        "the threshold-keeper marks the payment"
    );
}

#[test]
fn empty_handed_you_are_waved_on() {
    let mut a = app_at_threshold();
    if let Some(ref mut ps) = a.player_start {
        for it in [ItemType::Herb, ItemType::Food, ItemType::Coin] {
            let n = ps.inventory.get(it);
            ps.inventory.remove(it, n);
        }
    }
    a.resolve_encounter(EncounterAction::Trade);
    assert!(a
        .status_msg
        .clone()
        .unwrap_or_default()
        .contains("waves you on"));
}

#[test]
fn refusing_the_toll_costs_the_long_way() {
    let mut a = app_at_threshold();
    a.resolve_encounter(EncounterAction::Flee);
    assert!(
        a.vitals.energy < 0.85,
        "the long way round the mountain costs strength, energy now {}",
        a.vitals.energy
    );
}
