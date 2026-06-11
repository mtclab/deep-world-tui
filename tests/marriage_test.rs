// Marriage (#362): an ordinary oath in an ordinary town — no chosen ones.
// It asks trust earned over time, a family that doesn't bar the door, the
// town's regard, a roof of your own, and a feast. Grief closes the door for
// a season when the lifecycle takes a spouse.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{EncounterAction, ItemType, PeopleKind, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::ui::app::App;

fn app_in_town() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.enter_settlement(0, 0);
    a.clock.hour = 10;
    // Stand on settlement 0's own ground so courting addresses ITS people.
    let (ax, ay) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(deep_world_tui::model::PlayerPos {
        region_idx: 0,
        px: ax,
        py: ay,
    });
    // Make sure there's a single person to court.
    {
        let p = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0].people[0];
        p.has_spouse = false;
    }
    a
}

fn give_home(a: &mut App) {
    let tick = a.sim.as_ref().unwrap().world.tick;
    a.sim.as_mut().unwrap().world.regions[0]
        .structures
        .push(Structure {
            kind: BuildKind::Cabin,
            region_idx: 0,
            x: 1,
            y: 1,
            built_tick: 0,
            last_maintenance_tick: tick,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        });
}

fn earn_trust(a: &mut App) {
    // NpcMemory keeps the last 10 interactions, so talk alone (0.01 each)
    // can never reach the courting threshold — gifts and help (0.02+) can.
    // That asymmetry is the design: time, gifts, and help.
    for _ in 0..10 {
        a.record_npc_memory(0, 0, EncounterAction::Trade, 0.02);
    }
}

fn stock_feast(a: &mut App) {
    let ps = a.player_start.as_mut().unwrap();
    ps.inventory.add(ItemType::Food, 30);
    ps.inventory.add(ItemType::Coin, 20);
}

#[test]
fn the_oath_asks_everything_it_should() {
    let mut a = app_in_town();
    // A stranger is refused.
    a.court(0);
    assert!(
        a.spouse_id.is_none(),
        "no trust, no oath: {:?}",
        a.status_msg
    );
    earn_trust(&mut a);
    // Trust without a roof is refused.
    a.court(0);
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("roof"),
        "an oath needs a roof: {:?}",
        a.status_msg
    );
    give_home(&mut a);
    // A roof without a feast is refused.
    {
        let ps = a.player_start.as_mut().unwrap();
        let f = ps.inventory.get(ItemType::Food);
        ps.inventory.remove(ItemType::Food, f);
    }
    a.court(0);
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("feast"),
        "the hall must eat: {:?}",
        a.status_msg
    );
    stock_feast(&mut a);
    a.court(0);
    assert!(a.spouse_id.is_some(), "wed: {:?}", a.status_msg);
    // The spouse is marked, the record keeps the day.
    let p = &a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[0];
    assert!(p.has_spouse, "the spouse is wed too");
    let told = a
        .sim
        .as_ref()
        .unwrap()
        .journal
        .iter()
        .any(|e| e.text.contains("spoke the oath"));
    assert!(told, "the oath reaches the record");
    // And a second courting is refused flat.
    a.court(1);
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("already"),
        "one oath at a time"
    );
}

#[test]
fn a_hostile_family_bars_the_door() {
    let mut a = app_in_town();
    earn_trust(&mut a);
    give_home(&mut a);
    stock_feast(&mut a);
    let np = {
        let p = &a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[0];
        PeopleKind::from_name(&p.people)
    };
    a.inter_people_bias.mod_toward(np, -2.0);
    a.court(0);
    assert!(
        a.spouse_id.is_none(),
        "hostile peoples bar the door: {:?}",
        a.status_msg
    );
}

#[test]
fn grief_closes_the_door_for_a_season() {
    let mut a = app_in_town();
    earn_trust(&mut a);
    give_home(&mut a);
    stock_feast(&mut a);
    a.court(0);
    let spouse = a.spouse_id.clone().expect("wed");
    // The lifecycle takes them.
    {
        let s = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0];
        s.people.retain(|p| p.id != spouse);
    }
    // Eat well so the day-tick is about grief, not hunger.
    {
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Food, 60);
        ps.inventory.add(ItemType::Water, 60);
    }
    a.exit_settlement();
    a.rest_hours(12);
    a.rest_hours(12);
    a.advance_clock(4);
    assert!(a.spouse_id.is_none(), "widowed");
    assert!(a.widowed_day > 0, "the day is kept");
    let told = a
        .sim
        .as_ref()
        .unwrap()
        .journal
        .iter()
        .any(|e| e.text.contains("two bowls"));
    assert!(told, "grief reaches the record");
    // The door stays closed a while.
    a.enter_settlement(0, 0);
    {
        let p = &mut a.sim.as_mut().unwrap().world.regions[0].settlements[0].people[1];
        p.has_spouse = false;
    }
    a.record_npc_memory(0, 1, EncounterAction::Talk, 0.2);
    a.court(1);
    assert!(
        a.status_msg.clone().unwrap_or_default().contains("grief"),
        "a season of grief first: {:?}",
        a.status_msg
    );
}

#[test]
fn a_shared_roof_rests_better() {
    let charts = load_charts().expect("charts");
    let mut wed = App::new(42, charts.clone());
    let mut single = App::new(42, charts);
    for a in [&mut wed, &mut single] {
        a.generate_player();
        a.accept_player();
        a.running = true;
        a.enter_map(0);
        a.clock.hour = 20;
        // Stand on an own cabin on grass.
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
        a.player_pos = Some(deep_world_tui::model::PlayerPos {
            region_idx: 0,
            px,
            py,
        });
        let tick = a.sim.as_ref().unwrap().world.tick;
        a.sim.as_mut().unwrap().world.regions[0]
            .structures
            .push(Structure {
                kind: BuildKind::Cabin,
                region_idx: 0,
                x: px as u32,
                y: py as u32,
                built_tick: 0,
                last_maintenance_tick: tick,
                name: None,
                is_npc_built: false,
                stash: Default::default(),
            });
        a.vitals.energy = 0.2;
        let ps = a.player_start.as_mut().unwrap();
        ps.inventory.add(ItemType::Food, 30);
        ps.inventory.add(ItemType::Water, 30);
        ps.companions.clear();
    }
    wed.spouse_id = Some("someone".into());
    wed.rest_hours(8);
    single.rest_hours(8);
    assert!(
        wed.vitals.energy >= single.vitals.energy,
        "a shared roof never rests worse ({} vs {})",
        wed.vitals.energy,
        single.vitals.energy
    );
}
