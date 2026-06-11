// Regression: maintaining a structure must only touch player-built, decaying
// structures — the same predicate used to detect a maintainable structure. The
// update loops filtered on position alone, so a co-located NPC-built structure
// got its maintenance refreshed (and could be charged for / mislabeled).
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::ItemType;
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::ui::app::App;

#[test]
fn maintain_skips_colocated_npc_structure() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    app.advance_clock(5); // get world.tick > 0 so a refresh is distinguishable
    let pos = app.player_pos.expect("pos");
    let (ri, px, py) = (pos.region_idx, pos.px as u32, pos.py as u32);

    let mine = Structure {
        kind: BuildKind::Kota, // decaying, maintainable
        region_idx: ri,
        x: px,
        y: py,
        built_tick: 0,
        last_maintenance_tick: 0,
        name: None,
        is_npc_built: false,
        stash: Default::default(),
    };
    let theirs = Structure {
        kind: BuildKind::Kota,
        region_idx: ri,
        x: px,
        y: py,
        built_tick: 0,
        last_maintenance_tick: 0,
        name: None,
        is_npc_built: true,
        stash: Default::default(), // not maintainable by the player
    };
    {
        let region = &mut app.sim.as_mut().unwrap().world.regions[ri];
        region.structures.push(mine);
        region.structures.push(theirs);
    }
    {
        let ps = app.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Wood, 1);
    }

    app.start_build(); // standing on own structure -> maintains it

    let region = &app.sim.as_ref().unwrap().world.regions[ri];
    let mine = region.structures.iter().find(|s| !s.is_npc_built).unwrap();
    let theirs = region.structures.iter().find(|s| s.is_npc_built).unwrap();
    assert!(
        mine.last_maintenance_tick > 0,
        "player's structure should be maintained"
    );
    assert_eq!(
        theirs.last_maintenance_tick, 0,
        "co-located NPC structure must not be maintained"
    );
}
