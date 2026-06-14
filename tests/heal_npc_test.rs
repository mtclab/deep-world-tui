// Treating the sick (#454): the herbalist's work in the world. The remedies
// that tend your own sickness (#451) ease a villager's too — and a healer is
// remembered: standing rises, and the river-keeper marks the mercy.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::{ActiveDisease, Disease};
use deep_world_tui::model::{GodName, ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    a
}

/// Make person 0 of settlement 0 sick with a festered case; return their id.
fn sicken_first(a: &mut App) -> String {
    let now = a.sim.as_ref().map_or(0, |s| s.world.tick);
    let sim = a.sim.as_mut().unwrap();
    let p = &mut sim.world.regions[0].settlements[0].people[0];
    p.illnesses.clear();
    let mut d = ActiveDisease::new(Disease::Fever, now);
    d.worsen(48);
    p.illnesses.push(d);
    p.id.clone()
}

#[test]
fn tending_a_villager_eases_them_and_spends_a_remedy() {
    let mut a = app(42);
    sicken_first(&mut a);
    if let Some(ref mut ps) = a.player_start {
        ps.inventory.add(ItemType::Herb, 1);
    }
    let sev0 =
        a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[0].illnesses[0].severity;
    a.heal_npc(0, 0, 0);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(inv.get(ItemType::Herb), 0, "the remedy was spent");
    let sev1 =
        a.sim.as_ref().unwrap().world.regions[0].settlements[0].people[0].illnesses[0].severity;
    assert!(sev1 < sev0, "the villager's case eased: {sev1} !< {sev0}");
}

#[test]
fn healing_builds_standing_and_marks_masa() {
    let mut a = app(42);
    let pid = a.player_start.as_ref().unwrap().person.id.clone();
    let sid = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .id
        .clone();
    sicken_first(&mut a);
    if let Some(ref mut ps) = a.player_start {
        ps.inventory.add(ItemType::Herb, 1);
    }
    let rep0 = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
    let masa0 = a.god_affinity.get(GodName::Masa);
    a.heal_npc(0, 0, 0);
    let rep1 = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
    assert!(rep1 > rep0, "a healer's standing rises: {rep1} !> {rep0}");
    assert!(
        a.god_affinity.get(GodName::Masa) > masa0,
        "the river-keeper marks the mercy"
    );
}

#[test]
fn no_remedy_no_healing() {
    let mut a = app(42);
    sicken_first(&mut a);
    if let Some(ref mut ps) = a.player_start {
        for it in [ItemType::Herb, ItemType::Salve, ItemType::Bandage] {
            let n = ps.inventory.get(it);
            ps.inventory.remove(it, n);
        }
    }
    a.heal_npc(0, 0, 0);
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("nothing to treat"),
        "empty-handed should say so, got: {msg}"
    );
}

#[test]
fn the_hale_have_nothing_to_tend() {
    let mut a = app(42);
    // ensure person 0 is not sick
    a.sim.as_mut().unwrap().world.regions[0].settlements[0].people[0]
        .illnesses
        .clear();
    if let Some(ref mut ps) = a.player_start {
        ps.inventory.add(ItemType::Herb, 1);
    }
    a.heal_npc(0, 0, 0);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Herb),
        1,
        "no remedy spent on the hale"
    );
}
