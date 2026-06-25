// Localized threat (deep-world-godot#56-G): fear has a locus. A real predator
// prowling near a town gnaws at *its* people's sense of safety — not the whole
// region at once — and a shelterless place empties before it.
use deep_world_tui::model::wildlife::WildSpecies;
use deep_world_tui::model::Need;
use deep_world_tui::sim::agency::{step_agents, town_context, Departure};
use deep_world_tui::sim::beasts::WildBeast;
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

#[test]
fn fear_gnaws_at_a_threatened_town() {
    let mut sim = SimState::new_capped(42, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.9);
        p.needs.set(Need::Safety, 0.6); // uneasy, not yet pressing
    }
    let before = s.people[0].needs.get(Need::Safety);
    // Threatened, no shelter: fear should erode.
    let ctx = town_context(s, 1.0, /*under_threat*/ true, None, 0.15, 0);
    step_agents(s, &ctx);
    assert!(
        s.people[0].needs.get(Need::Safety) < before,
        "a danger near the town draws its people's safety down ({} -> {})",
        before,
        s.people[0].needs.get(Need::Safety)
    );
}

#[test]
fn a_safe_town_keeps_its_nerve() {
    let mut sim = SimState::new_capped(7, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    for p in s.people.iter_mut() {
        p.needs.set(Need::Food, 0.9);
        p.needs.set(Need::Safety, 0.6);
    }
    let before = s.people[0].needs.get(Need::Safety);
    let ctx = town_context(s, 1.0, /*under_threat*/ false, None, 0.15, 0);
    step_agents(s, &ctx);
    assert!(
        s.people[0].needs.get(Need::Safety) >= before,
        "with no danger about, the town keeps its nerve"
    );
}

#[test]
fn the_terrified_flee_a_shelterless_threatened_town() {
    let mut sim = SimState::new_capped(99, charts(), Some(40));
    let s = &mut sim.world.regions[0].settlements[0];
    s.buildings.clear(); // no shelter — nowhere to hide
    for p in s.people.iter_mut() {
        p.age_band = "adult".into();
        p.personality = vec!["loyal".into()]; // lawful — they flee, not turn outlaw
        p.needs.set(Need::Food, 0.9);
        p.needs.set(Need::Safety, 0.1); // terror — the pressing need
    }
    // Threatened, no shelter, a kinder town to run to.
    let ctx = town_context(s, 1.0, true, Some(1), 0.15, 0);
    let (departures, _) = step_agents(s, &ctx);
    assert!(
        !departures.is_empty()
            && departures
                .iter()
                .all(|(_, d)| matches!(d, Departure::Migrate { to: 1 })),
        "the terrified flee a place that cannot protect them"
    );
}

#[test]
fn a_prowling_beast_frightens_only_the_town_it_stalks() {
    // A wolf set beside town 0; a daily tick should press town 0's safety while a
    // distant town keeps calm — threat has a locus.
    let mut sim = SimState::new_capped(2024, charts(), Some(60));
    if sim.world.regions[0].settlements.len() < 2 {
        return; // need two towns to contrast; skip otherwise
    }
    // Soothe everyone first so any change is the beast's doing.
    for s in sim.world.regions[0].settlements.iter_mut() {
        for p in s.people.iter_mut() {
            p.needs.set(Need::Safety, 0.6);
            p.needs.set(Need::Food, 0.9);
        }
    }
    let (near_x, near_y) = {
        let s0 = &sim.world.regions[0].settlements[0];
        (s0.map_x, s0.map_y)
    };
    sim.beasts = vec![WildBeast {
        id: "wolf-1".into(),
        species: WildSpecies::Wolf,
        region_idx: 0,
        px: near_x as usize,
        py: near_y as usize,
        hp: 4,
    }];
    let safe0_before = sim.world.regions[0].settlements[0].people[0]
        .needs
        .get(Need::Safety);
    // Run the daily settlement pass.
    sim.world.tick = 23;
    sim.step();
    let safe0_after = sim.world.regions[0].settlements[0].people[0]
        .needs
        .get(Need::Safety);
    assert!(
        safe0_after < safe0_before,
        "the town the wolf stalks loses its nerve ({safe0_before} -> {safe0_after})"
    );
}
