// Items & afflictions depth (#310):
// - Tool/Bandage/Trap are real items; the stand-in recipe outputs are fixed
// - a Tool out-gathers improvised tools and wears with use
// - bandages tend illness during rest (eases severity, shortens the course)
// - a set Trap yields food on a wild rest
// - Disease.severity is finally read: untreated illness worsens
// - discoveries leave a mark (a spring quenches)
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::discovery::{Discovery, DiscoveryKind};
use deep_world_tui::model::{craft_recipes, ActiveDisease, Disease, ItemType};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

#[test]
fn recipes_output_real_items() {
    let r = craft_recipes();
    assert_eq!(r[0].name, "Bandage");
    assert_eq!(r[0].output, ItemType::Bandage);
    assert_eq!(r[1].name, "Tool");
    assert_eq!(r[1].output, ItemType::Tool);
    assert!(
        r.iter()
            .any(|x| x.name == "Metsik Trap" && x.output == ItemType::Trap),
        "trap recipe outputs a Trap"
    );
}

#[test]
fn a_tool_outgathers_bare_hands_and_wears() {
    let mut bare = app(42);
    let mut tooled = app(42);
    for a in [&mut bare, &mut tooled] {
        a.clock.hour = 10;
        let ps = a.player_start.as_mut().unwrap();
        for it in [ItemType::Iron, ItemType::Wood, ItemType::Stone] {
            let n = ps.inventory.get(it);
            ps.inventory.remove(it, n);
        }
    }
    tooled
        .player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Tool, 1);

    // Stand both on the same gatherable tile (find one).
    let tile = {
        let sim = bare.sim.as_ref().unwrap();
        let terr = &sim.world.regions[0].terrain;
        let mut found = None;
        'scan: for y in 0..terr.height {
            for x in 0..terr.width {
                if let Some(t) = terr.get(x, y) {
                    if let Some(item) = ItemType::gather_from(t) {
                        found = Some((x, y, item));
                        break 'scan;
                    }
                }
            }
        }
        found.expect("gatherable tile")
    };
    let (px, py, item) = tile;
    for a in [&mut bare, &mut tooled] {
        a.player_pos = Some(deep_world_tui::model::PlayerPos {
            region_idx: 0,
            px,
            py,
        });
    }
    bare.gather();
    tooled.gather();
    let got = |a: &App| a.player_start.as_ref().unwrap().inventory.get(item);
    assert!(
        got(&tooled) > got(&bare),
        "a Tool should out-gather bare hands ({} vs {})",
        got(&tooled),
        got(&bare)
    );
    let dur = tooled
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .durability(ItemType::Tool);
    assert!(dur < 1.0, "the Tool should wear with use, got {dur}");
}

#[test]
fn bandage_tends_illness_during_rest() {
    let mut a = app(42);
    a.clock.hour = 10;
    let t = a.sim.as_ref().unwrap().world.tick;
    {
        let ps = a.player_start.as_mut().unwrap();
        let mut d = ActiveDisease::new(Disease::MarshFever, t);
        d.severity = 1.4;
        ps.person.illnesses.push(d);
        ps.inventory.add(ItemType::Bandage, 1);
    }
    a.rest_hours(8);
    let ps = a.player_start.as_ref().unwrap();
    assert_eq!(
        ps.inventory.get(ItemType::Bandage),
        0,
        "the bandage is consumed"
    );
    if let Some(d) = ps.person.illnesses.first() {
        assert!(
            d.severity < 1.4,
            "tending should ease severity, got {}",
            d.severity
        );
    } // (it may also have fully recovered — equally fine)
}

#[test]
fn trap_yields_food_on_wild_rest() {
    let mut a = app(42);
    a.clock.hour = 10;
    {
        let ps = a.player_start.as_mut().unwrap();
        let n = ps.inventory.get(ItemType::Food);
        ps.inventory.remove(ItemType::Food, n);
        ps.inventory.add(ItemType::Trap, 1);
        ps.companions.clear();
    }
    // Stand on wild grass clear of any town (houses can wall a straight
    // walk now, so place rather than wander).
    {
        let spot = {
            let region = &a.sim.as_ref().unwrap().world.regions[0];
            let mut found = None;
            'o: for y in 0..region.terrain.height {
                for x in 0..region.terrain.width {
                    if region.terrain.get(x, y) == Some(deep_world_tui::model::Terrain::Grass)
                        && !region.settlements.iter().any(|s| s.contains_tile(x, y))
                    {
                        found = Some((x, y));
                        break 'o;
                    }
                }
            }
            found.expect("wild grass exists")
        };
        a.player_pos = Some(deep_world_tui::model::PlayerPos {
            region_idx: 0,
            px: spot.0,
            py: spot.1,
        });
    }
    assert!(a.player_on_settlement().is_none(), "clear of town");
    let before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    a.rest_hours(8);
    let after = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Food);
    assert!(
        after > before,
        "a set trap should yield food over a wild rest ({before} -> {after})"
    );
}

#[test]
fn untreated_illness_worsens() {
    let mut d = ActiveDisease::new(Disease::Fever, 0);
    let mild = d.vitals_modifier();
    d.worsen(48);
    assert!(
        d.vitals_modifier() > mild,
        "severity should sharpen the bite"
    );
    d.tend();
    assert!(d.severity < 1.24, "tending eases severity");
}

#[test]
fn a_spring_quenches() {
    let mut a = app(42);
    let pos = a.player_pos.unwrap();
    a.vitals.thirst = 0.2;
    a.sim
        .as_mut()
        .unwrap()
        .discoveries
        .entries
        .push(Discovery::new(
            "spring-test".into(),
            pos.region_idx,
            DiscoveryKind::Spring,
            pos.px as u32,
            pos.py as u32,
        ));
    a.check_discovery(pos.region_idx, pos.px, pos.py);
    assert!(
        a.vitals.thirst > 0.2,
        "finding a spring should quench, got {}",
        a.vitals.thirst
    );
}
