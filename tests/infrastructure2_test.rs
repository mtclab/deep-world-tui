// Infrastructure II (§7 leftovers): a well waters the dry lands, a waymarker
// keeps the ground around it known, a palisade quiets the night — and all of
// it decays, because that lesson never stops applying.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::ui::app::App;

fn app_on_grass() -> (App, usize, usize) {
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
    (a, px, py)
}

fn put(a: &mut App, kind: BuildKind, x: u32, y: u32) {
    let tick = a.sim.as_ref().unwrap().world.tick;
    a.sim.as_mut().unwrap().world.regions[0]
        .structures
        .push(Structure {
            kind,
            region_idx: 0,
            x,
            y,
            built_tick: 0,
            last_maintenance_tick: tick,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
}

#[test]
fn a_well_waters_the_rest() {
    let (mut dry, _, _) = app_on_grass();
    let (mut welled, wx, wy) = app_on_grass();
    put(&mut welled, BuildKind::Well, wx as u32, wy as u32);
    for a in [&mut dry, &mut welled] {
        a.vitals.thirst = 0.3;
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Food, 20);
        // No waterskin: the well is the only water.
        let w = ps.inventory.get(ItemType::Water);
        ps.inventory.remove(ItemType::Water, w);
        ps.companions.clear();
    }
    dry.rest_hours(8);
    welled.rest_hours(8);
    assert!(
        welled.vitals.thirst > dry.vitals.thirst,
        "the well slakes what the dry rest cannot ({} vs {})",
        welled.vitals.thirst,
        dry.vitals.thirst
    );
}

#[test]
fn a_waymarker_keeps_the_ground_known() {
    let (mut plain, px, py) = app_on_grass();
    let (mut marked, _, _) = app_on_grass();
    // A cairn two tiles ahead of the walker's next step.
    if px + 3 >= marked.sim.as_ref().unwrap().world.regions[0].terrain.width {
        return; // edge-bound seed: vacuous
    }
    put(
        &mut marked,
        BuildKind::Waymarker,
        (px + 3) as u32,
        py as u32,
    );
    for a in [&mut plain, &mut marked] {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Food, 20);
        ps.inventory.add(ItemType::Water, 20);
    }
    plain.move_player(1, 0);
    marked.move_player(1, 0);
    let count = |a: &App| {
        a.explored
            .first()
            .map(|m| {
                let mut n = 0;
                let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
                for y in 0..terr.height {
                    for x in 0..terr.width {
                        if m.is_explored(x, y) {
                            n += 1;
                        }
                    }
                }
                n
            })
            .unwrap_or(0)
    };
    assert!(
        count(&marked) > count(&plain),
        "the cairn shows more ground ({} vs {})",
        count(&marked),
        count(&plain)
    );
}

#[test]
fn the_quiet_inside_the_line_and_the_rot_outside_it() {
    // The palisade halves the night's encounter risk (type-level effects are
    // covered in rest; here the §8 discipline: it decays like everything).
    assert!(BuildKind::Well.decay_years().is_some(), "wells silt");
    assert!(
        BuildKind::Waymarker.decay_years().is_some(),
        "cairns topple"
    );
    assert!(
        BuildKind::Palisade.decay_years().is_some(),
        "timber rots — the lesson never stops applying"
    );
    // And none of them is shelter: a fence is not a roof.
    assert!(BuildKind::Palisade.stands_on(Terrain::Grass));
    assert!(!BuildKind::Well.stands_on(Terrain::Water));
    assert!(!BuildKind::Waymarker.stands_on(Terrain::Settlement));
}
