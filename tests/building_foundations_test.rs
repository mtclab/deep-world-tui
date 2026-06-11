// Building foundations (#342): the player chooses what to raise, the land
// has a say, real construction needs a tool and working hands — no more
// self-assembling Homes while the builder walks away.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::BuildKind;
use deep_world_tui::ui::app::App;

fn app_on(terrain_want: Terrain) -> Option<App> {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let sim = a.sim.as_ref().unwrap();
    let terr = &sim.world.regions[0].terrain;
    for y in 0..terr.height {
        for x in 0..terr.width {
            if terr.get(x, y) == Some(terrain_want) {
                a.player_pos = Some(PlayerPos {
                    region_idx: 0,
                    px: x,
                    py: y,
                });
                return Some(a);
            }
        }
    }
    None
}

#[test]
fn land_fitness_is_enforced() {
    assert!(
        !BuildKind::Kota.stands_on(Terrain::Forest),
        "kota wants open ground"
    );
    assert!(BuildKind::Kota.stands_on(Terrain::Tundra));
    assert!(!BuildKind::Tarp.stands_on(Terrain::Water));
    assert!(
        BuildKind::Cabin.stands_on(Terrain::Settlement),
        "residency land"
    );
    // Live: try to raise a kota in a forest.
    if let Some(mut a) = app_on(Terrain::Forest) {
        {
            let ps = a.player_start.as_mut().unwrap();
            ps.inventory.add(ItemType::Branches, 12);
            ps.inventory.add(ItemType::Stone, 4);
            ps.inventory.add(ItemType::Tinder, 2);
        }
        a.start_build_kind(Some(BuildKind::Kota));
        let msg = a.status_msg.clone().unwrap_or_default();
        assert!(
            msg.contains("cannot stand"),
            "forest must refuse a kota, got: {msg}"
        );
    }
}

#[test]
fn big_builds_need_a_tool_and_working_hands() {
    let mut a = app_on(Terrain::Grass).expect("grass tile");
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Wood, 40);
        ps.inventory.add(ItemType::Nails, 20);
        ps.inventory.add(ItemType::Stone, 12);
        // A fed builder: collapse recovery would relocate the player away
        // from the site (which is itself correct behavior).
        ps.inventory.add(ItemType::Food, 60);
        ps.inventory.add(ItemType::Water, 60);
    }
    // No tool: refused.
    a.start_build_kind(Some(BuildKind::Cabin));
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("Tool"),
        "cabin without a tool must be refused"
    );
    // With a tool: site opens, but time alone builds nothing.
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Tool, 1);
    a.start_build_kind(Some(BuildKind::Cabin));
    assert_eq!(a.sim.as_ref().unwrap().build_sites.len(), 1);
    a.advance_clock(20); // a day passes unworked
    assert_eq!(
        a.sim.as_ref().unwrap().build_sites.len(),
        1,
        "an unworked cabin site must not self-assemble"
    );
    let done_before = a.sim.as_ref().unwrap().build_sites[0].hours_done;
    assert_eq!(done_before, 0, "no ghost progress");
    // Work it like a real builder: a day's labor, then sleep. (Working
    // around the clock ends in an exhaustion collapse that carries the
    // builder off to recovery — the sim is right about that.)
    for _ in 0..9 {
        a.work_site();
        a.rest_hours(10);
    }
    a.advance_clock(1); // completion tick
    let region_has_cabin = a.sim.as_ref().unwrap().world.regions[0]
        .structures
        .iter()
        .any(|s| s.kind == BuildKind::Cabin && !s.is_npc_built);
    assert!(region_has_cabin, "worked site should finish into a cabin");
}

#[test]
fn quick_camps_still_pitch_instantly() {
    let mut a = app_on(Terrain::Grass).expect("grass tile");
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Branches, 4);
    a.start_build_kind(Some(BuildKind::LeanTo));
    let built = a.sim.as_ref().unwrap().world.regions[0]
        .structures
        .iter()
        .any(|s| s.kind == BuildKind::LeanTo);
    let site_open = !a.sim.as_ref().unwrap().build_sites.is_empty();
    assert!(
        built || site_open,
        "a lean-to should pitch immediately or open a fast site, got: {:?}",
        a.status_msg
    );
}
