use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{GameClock, Inventory, ItemType, PlayerVitals};

#[allow(dead_code)]
struct SeedResult {
    seed: u64,
    died: bool,
    death_tick: Option<u64>,
    final_hunger: f64,
    final_energy: f64,
    gathers: u32,
    rests: u32,
    eats: u32,
}

enum AiAction {
    Gather,
    Eat,
    Rest,
}

fn choose_ai_action(vitals: &PlayerVitals, inventory: &Inventory) -> AiAction {
    if vitals.hunger < 0.2 && inventory.get(ItemType::Food) > 0 {
        return AiAction::Eat;
    }
    if vitals.hunger < 0.15 && inventory.get(ItemType::Herb) > 0 {
        return AiAction::Eat;
    }
    if vitals.energy < 0.2 {
        return AiAction::Rest;
    }
    if vitals.hunger > 0.6 && inventory.get(ItemType::Food) < 3 {
        return AiAction::Gather;
    }
    if inventory.get(ItemType::Food) < 1 {
        return AiAction::Gather;
    }
    if vitals.energy < 0.5 {
        return AiAction::Rest;
    }
    AiAction::Gather
}

fn run_seed(seed: u64, _charts: &deep_world_tui::charts::Charts) -> SeedResult {
    let mut vitals = PlayerVitals::default();
    let mut clock = GameClock::default();
    let mut inventory = Inventory::default();
    inventory.add(ItemType::Food, 3);
    inventory.add(ItemType::Herb, 1);

    let max_ticks = 600u64;
    let mut died = false;
    let mut death_tick: Option<u64> = None;
    let mut gathers = 0u32;
    let mut rests = 0u32;
    let mut eats = 0u32;

    for tick in 0..max_ticks {
        let action = choose_ai_action(&vitals, &inventory);
        match action {
            AiAction::Gather => {
                let mut rng =
                    deep_world_tui::rng::SeedRng::new(seed).fork_for(&format!("gather-{}", tick));
                let gathered = simulate_gather(&mut rng);
                for item in gathered {
                    inventory.add(item, 1);
                }
                gathers += 1;
            }
            AiAction::Eat => {
                if inventory.remove(ItemType::Food, 1) {
                    vitals.hunger = (vitals.hunger + 0.25).min(1.0);
                    eats += 1;
                } else if inventory.remove(ItemType::Herb, 1) {
                    vitals.hunger = (vitals.hunger + 0.15).min(1.0);
                    eats += 1;
                }
            }
            AiAction::Rest => {
                vitals.energy = (vitals.energy + 0.20).min(1.0);
                vitals.hunger = (vitals.hunger - 0.02).max(0.0);
                rests += 1;
            }
        }

        let season = clock.season();
        vitals.tick(1, &mut inventory, season);

        if vitals.hunger <= 0.0 && vitals.energy <= 0.0 {
            died = true;
            death_tick = Some(tick);
            break;
        }

        clock.advance(1);
    }

    SeedResult {
        seed,
        died,
        death_tick,
        final_hunger: vitals.hunger,
        final_energy: vitals.energy,
        gathers,
        rests,
        eats,
    }
}

fn simulate_gather(rng: &mut deep_world_tui::rng::SeedRng) -> Vec<ItemType> {
    let roll = rng.gen_f64();
    if roll < 0.30 {
        vec![ItemType::Food]
    } else if roll < 0.50 {
        vec![ItemType::Herb]
    } else if roll < 0.60 {
        vec![ItemType::Wood]
    } else {
        vec![]
    }
}

#[test]
fn ai_survival_reasonable() {
    let charts = load_charts("data/charts.ron").expect("charts should load");
    let mut survived = 0u32;
    for seed in 1..=15u64 {
        let result = run_seed(seed, &charts);
        if !result.died {
            survived += 1;
        }
    }
    assert!(
        survived >= 5,
        "AI should survive at least 5/15 seeds, got {}/15",
        survived,
    );
}

#[test]
fn starvation_works() {
    let mut vitals = PlayerVitals::default();
    let mut clock = GameClock::default();
    let mut inventory = Inventory::default();
    for _ in 0..500u64 {
        let season = clock.season();
        vitals.tick(1, &mut inventory, season);
        if vitals.hunger <= 0.0 && vitals.energy <= 0.0 {
            return;
        }
        clock.advance(1);
    }
    panic!("vitals never hit 0 in 500 ticks without food");
}

#[test]
fn gather_produces_resources() {
    let mut rng = deep_world_tui::rng::SeedRng::new(42).fork_for("gather-test");
    let mut food_count = 0u32;
    let mut herb_count = 0u32;
    for _ in 0..100 {
        let items = simulate_gather(&mut rng);
        for item in items {
            match item {
                ItemType::Food => food_count += 1,
                ItemType::Herb => herb_count += 1,
                _ => {}
            }
        }
    }
    assert!(food_count > 0, "gathering should sometimes produce food");
    assert!(herb_count > 0, "gathering should sometimes produce herbs");
}
