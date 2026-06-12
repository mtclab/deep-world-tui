// Canon crops (#391): the Bronze Road four reach the fields. Winter-rye holds
// through the Frost, drought-millet owns the dry lands, flax pays in cloth,
// and a settlement's farmers plant only what feeds the stores.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::CropType;
use deep_world_tui::model::{ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::ui::app::App;

fn app_on_grass() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
    let mut found = None;
    'o: for y in 0..terr.height {
        for x in 0..terr.width {
            if terr.get(x, y) == Some(Terrain::Grass) {
                found = Some((x, y));
                break 'o;
            }
        }
    }
    let (px, py) = found.expect("grass");
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px,
        py,
    });
    a.clock.hour = 10;
    a
}

fn give_cabin_here(a: &mut App) {
    let pos = a.player_pos.unwrap();
    a.sim.as_mut().unwrap().world.regions[pos.region_idx]
        .structures
        .push(Structure {
            kind: BuildKind::Cabin,
            region_idx: pos.region_idx,
            x: pos.px as u32,
            y: pos.py as u32,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
}

#[test]
fn winter_rye_holds_through_the_frost() {
    let mut a = app_on_grass();
    give_cabin_here(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 10);
    a.plant_crop(Some(CropType::WinterRye));
    assert_eq!(a.player_farms.len(), 1, "the rye goes in");
    // Frost arrives. Everything else dies standing; the rye is why anyone
    // plants it.
    a.clock.day = 60; // the frost arrives with the next day's clock
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Water, 10);
    a.rest_hours(12);
    assert_eq!(a.clock.season(), deep_world_tui::model::Season::Frost);
    assert_eq!(a.player_farms.len(), 1, "the rye holds");
    // A barley field beside it would have died: prove the discrimination.
    assert!(!CropType::Grain.survives_frost());
    assert!(CropType::WinterRye.survives_frost());
}

#[test]
fn winter_rye_may_be_planted_in_the_frost_by_name() {
    let mut a = app_on_grass();
    give_cabin_here(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 10);
    a.clock.day = 61;
    assert_eq!(a.clock.season(), deep_world_tui::model::Season::Frost);
    // The land's own pick is refused in Frost…
    a.plant();
    assert_eq!(a.player_farms.len(), 0, "no unnamed planting in the frost");
    // …but the named autumn field goes in.
    a.plant_crop(Some(CropType::WinterRye));
    assert_eq!(a.player_farms.len(), 1, "winter-rye is the exception");
}

#[test]
fn flax_pays_in_cloth_not_food() {
    let mut a = app_on_grass();
    give_cabin_here(&mut a);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 10);
    a.plant_crop(Some(CropType::Flax));
    assert_eq!(a.player_farms.len(), 1);
    // Ripen the field by hand and harvest.
    let ticks = CropType::Flax.growth_ticks();
    let now = a.sim.as_ref().unwrap().world.tick;
    a.sim.as_mut().unwrap().world.tick = now + ticks * 2;
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Water, 10);
    a.rest_hours(1);
    let cloth_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Cloth);
    let food_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    a.harvest();
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert!(
        inv.get(ItemType::Cloth) > cloth_before,
        "the flax harvest is fiber"
    );
    assert_eq!(
        inv.get(ItemType::Food),
        food_before,
        "no meal comes off a fiber field"
    );
}

#[test]
fn settlements_plant_only_what_feeds_the_stores() {
    // Across every terrain a settlement might farm, the picked crop is food.
    for t in [
        Terrain::Farmland,
        Terrain::Grass,
        Terrain::Forest,
        Terrain::Sand,
        Terrain::Tundra,
        Terrain::Swamp,
        Terrain::Coast,
    ] {
        let best = CropType::all()
            .into_iter()
            .filter(|c| c.is_food())
            .max_by(|a, b| {
                a.regional_suitability(t)
                    .partial_cmp(&b.regional_suitability(t))
                    .unwrap()
            })
            .unwrap();
        assert!(best.is_food(), "{t:?} picks {best:?}");
        assert_ne!(best, CropType::Flax);
    }
    // And the dry lands belong to the millet — the insurance crop.
    let dry = CropType::all()
        .into_iter()
        .filter(|c| c.is_food())
        .max_by(|a, b| {
            a.regional_suitability(Terrain::Sand)
                .partial_cmp(&b.regional_suitability(Terrain::Sand))
                .unwrap()
        })
        .unwrap();
    assert_eq!(dry, CropType::DroughtMillet);
}

#[test]
fn crop_names_speak_canon() {
    assert_eq!(CropType::Grain.name(), "flood-barley");
    assert_eq!(CropType::from_name("rye"), Some(CropType::WinterRye));
    assert_eq!(CropType::from_name("millet"), Some(CropType::DroughtMillet));
    assert_eq!(CropType::from_name("flax"), Some(CropType::Flax));
    assert_eq!(CropType::from_name("nonsense"), None);
}
