use crate::model::{Need, World};

pub mod effects;
pub mod needs_dependent;
pub mod relationships;
pub mod reputation;

const FOOD_DECAY_RATE: f64 = 0.08;
const MONEY_DECAY_RATE: f64 = 0.04;
const CARE_DECAY_RATE: f64 = 0.02;
const PRESENCE_DECAY_RATE: f64 = 0.02;
const SAFETY_DECAY_RATE: f64 = 0.01;

pub fn tick_needs(world: &mut World, dt: f64) {
    let rates: [(Need, f64); 5] = [
        (Need::Food, FOOD_DECAY_RATE),
        (Need::Money, MONEY_DECAY_RATE),
        (Need::Care, CARE_DECAY_RATE),
        (Need::Presence, PRESENCE_DECAY_RATE),
        (Need::Safety, SAFETY_DECAY_RATE),
    ];
    for region in &mut world.regions {
        for settlement in &mut region.settlements {
            for person in &mut settlement.people {
                for (need, rate) in &rates {
                    person.needs.decay(*need, rate * dt);
                }
            }
        }
    }
}

pub fn tick(world: &mut World) {
    tick_needs(world, 1.0);
    world.tick += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::gen::world::generate_world;

    #[test]
    fn tick_needs_food_highest_decay() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        let food_before = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let safety_before = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Safety);
        tick_needs(&mut world, 1.0);
        let food_after = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Food);
        let safety_after = world.regions[0].settlements[0].people[0]
            .needs
            .get(Need::Safety);
        let food_drop = food_before - food_after;
        let safety_drop = safety_before - safety_after;
        assert!(
            food_drop > safety_drop,
            "food decay ({:.4}) should exceed safety ({:.4})",
            food_drop,
            safety_drop
        );
    }

    #[test]
    fn tick_needs_exact_values() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    person.needs = crate::model::Needs::default();
                }
            }
        }
        tick_needs(&mut world, 1.0);
        let p = &world.regions[0].settlements[0].people[0];
        assert!(
            (p.needs.get(Need::Food) - (0.8 - FOOD_DECAY_RATE)).abs() < f64::EPSILON,
            "food after 1 tick: expected {}, got {}",
            0.8 - FOOD_DECAY_RATE,
            p.needs.get(Need::Food)
        );
        assert!(
            (p.needs.get(Need::Safety) - (0.8 - SAFETY_DECAY_RATE)).abs() < f64::EPSILON,
            "safety after 1 tick: expected {}, got {}",
            0.8 - SAFETY_DECAY_RATE,
            p.needs.get(Need::Safety)
        );
    }

    #[test]
    fn tick_needs_clamped_at_zero() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        for _ in 0..200 {
            tick_needs(&mut world, 1.0);
        }
        for region in &world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    for need in &[
                        Need::Food,
                        Need::Money,
                        Need::Care,
                        Need::Presence,
                        Need::Safety,
                    ] {
                        assert!(
                            person.needs.get(*need) >= 0.0,
                            "{} went negative: {}",
                            need,
                            person.needs.get(*need)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn tick_needs_10_ticks_food_below_half() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        for region in &mut world.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    person.needs = crate::model::Needs::default();
                }
            }
        }
        for _ in 0..10 {
            tick_needs(&mut world, 1.0);
        }
        for region in &world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    let food = person.needs.get(Need::Food);
                    let safety = person.needs.get(Need::Safety);
                    assert!(food < 0.5, "food after 10 ticks: {} (expect < 0.5)", food);
                    assert!(
                        safety > 0.5,
                        "safety after 10 ticks: {} (expect > 0.5)",
                        safety
                    );
                }
            }
        }
    }

    #[test]
    fn tick_deterministic() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut a = generate_world(42, &charts);
        let mut b = generate_world(42, &charts);
        for _ in 0..5 {
            tick_needs(&mut a, 1.0);
            tick_needs(&mut b, 1.0);
        }
        let pa = &a.regions[0].settlements[0].people[0];
        let pb = &b.regions[0].settlements[0].people[0];
        assert_eq!(pa.needs, pb.needs, "tick_needs must be deterministic");
    }

    #[test]
    fn tick_increments_tick_counter() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut world = generate_world(42, &charts);
        assert_eq!(world.tick, 0);
        tick(&mut world);
        assert_eq!(world.tick, 1);
        tick(&mut world);
        assert_eq!(world.tick, 2);
    }
}
