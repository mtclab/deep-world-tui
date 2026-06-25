// Life-stage epic (deep-world-godot#52): the young and the old are dependents the
// able-bodied carry, and they act differently from a working adult. A child never
// works, steals, or turns bandit — the commons keep it. An elder is too frail for
// the open road — it migrates to a kinder town or stays, never to banditry — and
// it needs more tending.
use deep_world_tui::model::Need;
use deep_world_tui::sim::agency::{life_stage, step_agents, town_context, Departure, Stage};
use deep_world_tui::sim::SimState;

fn charts() -> deep_world_tui::charts::Charts {
    deep_world_tui::charts::load::load_charts().expect("charts")
}

fn town(seed: u64) -> SimState {
    SimState::new_capped(seed, charts(), Some(40))
}

#[test]
fn life_stage_reads_the_age_bands() {
    assert_eq!(life_stage("child"), Stage::Dependent);
    assert_eq!(life_stage("youth"), Stage::Able);
    assert_eq!(life_stage("adult"), Stage::Able);
    assert_eq!(life_stage("elder"), Stage::Elder);
    assert_eq!(life_stage("aged"), Stage::Elder);
}

#[test]
fn children_never_work_steal_or_turn_bandit() {
    let mut sim = town(42);
    let s = &mut sim.world.regions[0].settlements[0];
    // A town of starving children, an empty granary and an empty common purse —
    // and even a coin in each pocket, to prove they do not trade or steal.
    s.food_stock = 0.0;
    s.treasury = 0;
    for p in s.people.iter_mut() {
        p.age_band = "child".into();
        p.needs.set(Need::Food, 0.05);
        p.coins = 5;
        p.personality = vec!["devious".into(), "ruthless".into()]; // would steal, if it could
    }
    let coins_before: u32 = s.people.iter().map(|p| p.coins).sum();
    // A fed neighbour exists, to prove children still do not leave on their own.
    let ctx = town_context(s, 1.0, false, Some(1), 0.15, 100);
    let (departures, _) = step_agents(s, &ctx);
    assert!(departures.is_empty(), "no child leaves on its own");
    assert_eq!(
        s.people.iter().map(|p| p.coins).sum::<u32>(),
        coins_before,
        "children neither work, buy, nor steal"
    );
    assert!(
        s.people.iter().all(|p| p.relations.is_empty()),
        "and they sow no feuds"
    );
    // They simply went hungry — the mark of a town that cannot feed its young.
    assert!(s.people[0].needs.get(Need::Food) < 0.05);
}

#[test]
fn the_town_feeds_its_children_from_the_common_purse() {
    let mut sim = town(7);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0; // the granary has failed...
    s.treasury = 20; // ...but the town has common coin
    for p in s.people.iter_mut() {
        p.age_band = "adult".into();
        p.needs.set(Need::Food, 0.9); // adults sated, so only the child acts
    }
    s.people[0].age_band = "child".into();
    s.people[0].needs.set(Need::Food, 0.3);
    let food_before = s.people[0].needs.get(Need::Food);
    let ctx = town_context(s, 1.0, false, None, 0.15, 50);
    step_agents(s, &ctx);
    assert!(
        s.people[0].needs.get(Need::Food) > food_before,
        "the child was fed from the commons"
    );
    assert!(s.treasury < 20, "the common purse paid for it");
}

#[test]
fn an_elder_takes_to_a_kinder_town_but_never_to_the_road() {
    // With nowhere better, a destitute elder stays and suffers — it does not turn
    // bandit, however hard its heart.
    let mut sim = town(99);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0;
    s.treasury = 0;
    for p in s.people.iter_mut() {
        p.age_band = "elder".into();
        p.needs.set(Need::Food, 0.05);
        p.coins = 0;
        p.personality = vec!["bitter".into(), "ruthless".into()]; // would-be brigand
    }
    let wanderers_before = sim.frontier.wanderers;
    let s = &mut sim.world.regions[0].settlements[0];
    let ctx = town_context(s, 1.0, false, None, 0.15, 10); // no migrate target
    let (departures, _) = step_agents(s, &ctx);
    assert!(
        departures
            .iter()
            .all(|(_, d)| !matches!(d, Departure::Bandit)),
        "no elder takes to the road as a brigand"
    );
    assert!(
        departures.is_empty(),
        "with nowhere better, the elder stays"
    );
    assert_eq!(sim.frontier.wanderers, wanderers_before);
}

#[test]
fn a_frail_elder_with_a_kinder_town_migrates() {
    let mut sim = town(123);
    let s = &mut sim.world.regions[0].settlements[0];
    s.food_stock = 0.0;
    s.treasury = 0;
    for p in s.people.iter_mut() {
        p.age_band = "aged".into();
        p.needs.set(Need::Food, 0.05);
        p.coins = 0;
        p.personality = vec!["bitter".into()];
    }
    let ctx = town_context(s, 1.0, false, Some(1), 0.15, 10); // a fed town to flee to
    let (departures, _) = step_agents(s, &ctx);
    assert!(
        !departures.is_empty(),
        "the frail flee a town that cannot feed them"
    );
    assert!(
        departures
            .iter()
            .all(|(_, d)| matches!(d, Departure::Migrate { to: 1 })),
        "and they go lawfully to the kinder town, never to the road"
    );
}
