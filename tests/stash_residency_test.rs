// Stash + residency (#344): a house keeps what you put in it — through death
// (the heir inherits the building and its contents); building on a village's
// ground needs the village's consent; a resident pays neighbor's prices.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos, SettlementService, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
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

fn cabin_at(a: &mut App, terrain_want: Terrain) -> bool {
    let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
    let mut found = None;
    'o: for y in 0..terr.height {
        for x in 0..terr.width {
            if terr.get(x, y) == Some(terrain_want) {
                found = Some((x, y));
                break 'o;
            }
        }
    }
    let Some((px, py)) = found else { return false };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });
    a.sim.as_mut().unwrap().world.regions[0]
        .structures
        .push(Structure {
            kind: BuildKind::Cabin,
            region_idx: 0,
            x: px as u32,
            y: py as u32,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
    true
}

#[test]
fn the_house_keeps_things_and_the_heir_inherits_them() {
    let mut a = app();
    assert!(cabin_at(&mut a, Terrain::Grass));
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Iron, 5);
    a.stash_item(ItemType::Iron, 5);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Iron),
        0
    );
    a.take_item(ItemType::Iron, 2);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Iron),
        2
    );
    // Die of old age; the heir walks to the cabin and opens the stash.
    a.start_age_years = 80;
    a.birth_day = a.clock.day;
    a.lifespan_years = 80;
    a.vitals.hunger = 1.0;
    a.vitals.thirst = 1.0;
    a.vitals.energy = 1.0;
    let cabin_pos = (a.player_pos.unwrap().px, a.player_pos.unwrap().py);
    a.advance_clock(1);
    assert!(a.player_start.is_some(), "an heir continues");
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: cabin_pos.0,
        py: cabin_pos.1,
    });
    a.take_item(ItemType::Iron, 3);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Iron),
        3,
        "the heir inherits the house and what's in it"
    );
}

#[test]
fn the_council_must_grant_the_ground() {
    let mut a = app();
    // Stand on settlement ground with materials + tool.
    let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
    let mut found = None;
    'o: for y in 0..terr.height {
        for x in 0..terr.width {
            if terr.get(x, y) == Some(Terrain::Settlement) {
                found = Some((x, y));
                break 'o;
            }
        }
    }
    let (px, py) = found.expect("settlement tile");
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Wood, 40);
        ps.inventory.add(ItemType::Nails, 20);
        ps.inventory.add(ItemType::Stone, 12);
        ps.inventory.add(ItemType::Tool, 1);
    }
    // Hostile: refused.
    let people = a
        .current_settlement_people()
        .unwrap_or(deep_world_tui::model::PeopleKind::Metsik);
    a.inter_people_bias.mod_toward(people, -2.0);
    a.start_build_kind(Some(BuildKind::Cabin));
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("council"),
        "hostile bias must be refused ground, got {:?}",
        a.status_msg
    );
    // Mend it: granted.
    a.inter_people_bias.mod_toward(people, 4.0);
    a.start_build_kind(Some(BuildKind::Cabin));
    assert!(
        !a.sim.as_ref().unwrap().build_sites.is_empty(),
        "with regard, the ground is granted: {:?}",
        a.status_msg
    );
}

#[test]
fn a_resident_pays_neighbors_prices() {
    let mut plain = app();
    let mut resident = app();
    for a in [&mut plain, &mut resident] {
        a.player_start
            .as_mut()
            .unwrap()
            .inventory
            .add(ItemType::Coin, 30);
        a.enter_settlement(0, 0);
    }
    assert!(cabin_at(&mut resident, Terrain::Settlement));
    let coins = |a: &App| {
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin)
    };
    let b0 = coins(&plain);
    plain.use_service(SettlementService::Temple);
    let plain_cost = b0 - coins(&plain);
    let b1 = coins(&resident);
    resident.use_service(SettlementService::Temple);
    let res_cost = b1 - coins(&resident);
    assert!(
        res_cost < plain_cost,
        "a finished house in town is worth a coin off ({res_cost} vs {plain_cost})"
    );
}
