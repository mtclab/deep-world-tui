use crate::model::{PeopleKind, World};
use crate::rng::SeedRng;

const MIGRATION_INTERVAL: u64 = 30;
const JOB_SEEKING_STABILITY_THRESHOLD: f64 = 0.3;
const MARRIAGE_BIAS_THRESHOLD: f64 = 0.4;
const FLIGHT_REPUTATION_THRESHOLD: f64 = 0.15;
const FLIGHT_CHANCE: f64 = 0.1;

#[derive(Debug, Clone)]
struct MigrationCandidate {
    person_id: String,
    source_settlement_idx: usize,
    source_region_idx: usize,
}

#[derive(Debug, Clone)]
struct SettlementRef {
    region_idx: usize,
    settlement_idx: usize,
}

fn find_settlement_refs(world: &World) -> Vec<SettlementRef> {
    let mut refs = Vec::new();
    for (ri, region) in world.regions.iter().enumerate() {
        for (si, _) in region.settlements.iter().enumerate() {
            refs.push(SettlementRef {
                region_idx: ri,
                settlement_idx: si,
            });
        }
    }
    refs
}

fn find_adjacent_settlements(world: &World, region_idx: usize) -> Vec<SettlementRef> {
    let mut adjacent = Vec::new();
    let region = &world.regions[region_idx];

    // Settlements in the same region
    for si in 0..region.settlements.len() {
        adjacent.push(SettlementRef {
            region_idx,
            settlement_idx: si,
        });
    }

    // Settlements in neighboring regions
    let neighbors = &region.neighbors;
    for ni in [
        neighbors.north,
        neighbors.south,
        neighbors.east,
        neighbors.west,
    ]
    .iter()
    .flatten()
    {
        if *ni < world.regions.len() {
            for si in 0..world.regions[*ni].settlements.len() {
                adjacent.push(SettlementRef {
                    region_idx: *ni,
                    settlement_idx: si,
                });
            }
        }
    }

    adjacent
}

fn profession_demand(settlement: &crate::model::Settlement, profession: &str) -> f64 {
    let profession_lower = profession.to_lowercase();
    let has_matching_service = settlement.services.iter().any(|s| {
        s.label().to_lowercase().contains(&profession_lower)
            || profession_lower.contains(&s.label().to_lowercase())
    });

    let current_count = settlement
        .people
        .iter()
        .filter(|p| p.profession.to_lowercase() == profession_lower)
        .count();

    let demand_base = if has_matching_service { 3 } else { 1 };
    (demand_base as f64) - (current_count as f64 * 0.5)
}

pub fn tick_migration(sim: &mut crate::sim::SimState, tick: u64) {
    if tick == 0 || !tick.is_multiple_of(MIGRATION_INTERVAL) {
        return;
    }

    let seed = sim.world.seed;
    let mut migrants: Vec<(MigrationCandidate, SettlementRef, String)> = Vec::new();

    let settlement_refs = find_settlement_refs(&sim.world);
    if settlement_refs.len() < 2 {
        return;
    }

    for (sri, sref) in settlement_refs.iter().enumerate() {
        let settlement = &sim.world.regions[sref.region_idx].settlements[sref.settlement_idx];
        let adjacent = find_adjacent_settlements(&sim.world, sref.region_idx);

        for person in settlement.people.iter() {
            if person.age_band != "adult" && person.age_band != "elder" {
                continue;
            }

            let mut person_rng =
                SeedRng::new(seed).fork_for(&format!("migration-{}-{}-{}", person.id, sri, tick));

            // Job-seeking: low stability + matching profession demand
            if person.needs.get(crate::model::Need::Safety) < JOB_SEEKING_STABILITY_THRESHOLD {
                if let Some(target) = find_job_seeking_target(
                    &mut person_rng,
                    &sim.world,
                    &adjacent,
                    &person.profession,
                    &person.people,
                    sref.region_idx,
                    sref.settlement_idx,
                ) {
                    let target_settlement =
                        &sim.world.regions[target.region_idx].settlements[target.settlement_idx];
                    let people_kind = PeopleKind::from_name(&person.people);
                    let reason = format!(
                        "A {} {} crossed the ridge to {}, citing steady work.",
                        people_kind.label(),
                        person.profession,
                        target_settlement.name,
                    );
                    migrants.push((
                        MigrationCandidate {
                            person_id: person.id.clone(),
                            source_settlement_idx: sref.settlement_idx,
                            source_region_idx: sref.region_idx,
                        },
                        target,
                        reason,
                    ));
                    continue;
                }
            }

            // Marriage: unmarried adult with mutual bias in adjacent settlement
            if !person.has_spouse && person.age_band == "adult" {
                if let Some((partner_target, reason)) = check_marriage_migration(
                    &sim.world,
                    &adjacent,
                    person,
                    sref.region_idx,
                    sref.settlement_idx,
                    seed,
                    tick,
                ) {
                    migrants.push((
                        MigrationCandidate {
                            person_id: person.id.clone(),
                            source_settlement_idx: sref.settlement_idx,
                            source_region_idx: sref.region_idx,
                        },
                        partner_target,
                        reason,
                    ));
                    continue;
                }
            }

            // Flight: bad reputation
            let local_rep = sim.reputation.get(&person.id, &settlement.id);
            if local_rep < FLIGHT_REPUTATION_THRESHOLD {
                let roll = person_rng.gen_f64();
                if roll < FLIGHT_CHANCE {
                    if let Some(target) = find_flight_target(
                        &sim.world,
                        &adjacent,
                        &person.people,
                        sref.region_idx,
                        sref.settlement_idx,
                    ) {
                        let target_settlement = &sim.world.regions[target.region_idx].settlements
                            [target.settlement_idx];
                        let people_kind = PeopleKind::from_name(&person.people);
                        let reason = format!(
                            "A {} {} fled to {}, seeking a fresh start.",
                            people_kind.label(),
                            person.profession,
                            target_settlement.name,
                        );
                        migrants.push((
                            MigrationCandidate {
                                person_id: person.id.clone(),
                                source_settlement_idx: sref.settlement_idx,
                                source_region_idx: sref.region_idx,
                            },
                            target,
                            reason,
                        ));
                    }
                }
            }
        }
    }

    // Apply migrations - reverse sort by source region/settlement to preserve indices
    migrants.sort_by(
        |a: &(MigrationCandidate, SettlementRef, String),
         b: &(MigrationCandidate, SettlementRef, String)| {
            b.0.source_region_idx
                .cmp(&a.0.source_region_idx)
                .then(b.0.source_settlement_idx.cmp(&a.0.source_settlement_idx))
        },
    );

    let mut migrated_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (cand, target, reason) in &migrants {
        if migrated_ids.contains(&cand.person_id) {
            continue;
        }

        let source_region_idx = cand.source_region_idx;
        let source_settlement_idx = cand.source_settlement_idx;
        let target_region_idx = target.region_idx;
        let target_settlement_idx = target.settlement_idx;

        // Find and remove the person from source settlement
        let person_id = cand.person_id.clone();
        let person_pos = sim.world.regions[source_region_idx].settlements[source_settlement_idx]
            .people
            .iter()
            .position(|p| p.id == person_id);

        let person = match person_pos {
            Some(idx) => sim.world.regions[source_region_idx].settlements[source_settlement_idx]
                .people
                .remove(idx),
            None => continue,
        };

        let target_settlement_id = sim.world.regions[target_region_idx].settlements
            [target_settlement_idx]
            .id
            .clone();
        let target_region_id = sim.world.regions[target_region_idx].id.clone();

        // Update person's location
        let mut moved_person = person;
        moved_person.region = target_region_id;
        moved_person.settlement = target_settlement_id;

        // Add to target settlement
        sim.world.regions[target_region_idx].settlements[target_settlement_idx]
            .people
            .push(moved_person);

        migrated_ids.insert(cand.person_id.clone());

        // Journal the migration
        sim.log_journal(tick, reason.clone());

        // Update population counts (derived from people.len())
        sim.world.regions[source_region_idx].settlements[source_settlement_idx].population =
            sim.world.regions[source_region_idx].settlements[source_settlement_idx]
                .people
                .len() as u32;
        sim.world.regions[target_region_idx].settlements[target_settlement_idx].population =
            sim.world.regions[target_region_idx].settlements[target_settlement_idx]
                .people
                .len() as u32;
    }
}

fn find_job_seeking_target(
    rng: &mut SeedRng,
    world: &World,
    adjacent: &[SettlementRef],
    profession: &str,
    person_people: &str,
    source_region_idx: usize,
    source_settlement_idx: usize,
) -> Option<SettlementRef> {
    let person_kind = PeopleKind::from_name(person_people);

    let mut best_target: Option<(SettlementRef, f64)> = None;

    for sref in adjacent {
        if sref.region_idx == source_region_idx && sref.settlement_idx == source_settlement_idx {
            continue;
        }

        let settlement = &world.regions[sref.region_idx].settlements[sref.settlement_idx];
        let demand = profession_demand(settlement, profession);
        if demand <= 0.0 {
            continue;
        }

        let dominant_people = settlement
            .people
            .first()
            .map(|p| PeopleKind::from_name(&p.people))
            .unwrap_or(PeopleKind::Metsik);
        let bias = person_kind.bias_toward(dominant_people);

        let score = demand + bias;
        if best_target.is_none() || score > best_target.as_ref().unwrap().1 {
            best_target = Some((sref.clone(), score));
        }
    }

    if let Some((target, score)) = best_target {
        let move_chance = (0.3 + score * 0.2).min(0.8);
        if rng.gen_f64() < move_chance {
            return Some(target);
        }
    }

    None
}

fn check_marriage_migration(
    world: &World,
    adjacent: &[SettlementRef],
    person: &crate::model::Person,
    source_region_idx: usize,
    source_settlement_idx: usize,
    seed: u64,
    tick: u64,
) -> Option<(SettlementRef, String)> {
    let person_kind = PeopleKind::from_name(&person.people);

    for sref in adjacent {
        if sref.region_idx == source_region_idx && sref.settlement_idx == source_settlement_idx {
            continue;
        }

        let settlement = &world.regions[sref.region_idx].settlements[sref.settlement_idx];

        for partner in &settlement.people {
            if partner.age_band != "adult" || partner.has_spouse {
                continue;
            }

            let partner_kind = PeopleKind::from_name(&partner.people);
            let bias_to_partner = person_kind.bias_toward(partner_kind);
            let bias_from_partner = partner_kind.bias_toward(person_kind);

            if bias_to_partner > MARRIAGE_BIAS_THRESHOLD
                && bias_from_partner > MARRIAGE_BIAS_THRESHOLD
            {
                let mut partner_rng = SeedRng::new(seed)
                    .fork_for(&format!("marriage-{}-{}-{}", person.id, partner.id, tick));
                let roll = partner_rng.gen_f64();
                let chance =
                    (bias_to_partner.min(bias_from_partner) - MARRIAGE_BIAS_THRESHOLD) * 2.0;
                if roll < chance {
                    let reason = format!(
                        "A {} {} and a {} {} were joined across settlements, bound by mutual regard.",
                        person_kind.label(),
                        person.profession,
                        partner_kind.label(),
                        partner.profession,
                    );
                    return Some((sref.clone(), reason));
                }
            }
        }
    }

    None
}

fn find_flight_target(
    world: &World,
    adjacent: &[SettlementRef],
    person_people: &str,
    source_region_idx: usize,
    source_settlement_idx: usize,
) -> Option<SettlementRef> {
    let person_kind = PeopleKind::from_name(person_people);

    let mut best_target: Option<(SettlementRef, f64)> = None;

    for sref in adjacent {
        if sref.region_idx == source_region_idx && sref.settlement_idx == source_settlement_idx {
            continue;
        }

        let settlement = &world.regions[sref.region_idx].settlements[sref.settlement_idx];
        let dominant_people = settlement
            .people
            .first()
            .map(|p| PeopleKind::from_name(&p.people))
            .unwrap_or(PeopleKind::Metsik);
        let bias = person_kind.bias_toward(dominant_people);

        if best_target.is_none() || bias > best_target.as_ref().unwrap().1 {
            best_target = Some((sref.clone(), bias));
        }
    }

    best_target.map(|(target, _)| target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;
    use crate::sim::SimState;

    fn make_sim(seed: u64) -> SimState {
        let charts = charts::load_charts().unwrap();
        SimState::new(seed, charts)
    }

    #[test]
    fn migration_deterministic_across_1000_ticks() {
        let mut sim_a = make_sim(42);
        let mut sim_b = make_sim(42);
        for _ in 0..1000 {
            sim_a.step();
            sim_b.step();
        }
        assert_eq!(sim_a.world.tick, sim_b.world.tick);
        assert_eq!(sim_a.world.regions.len(), sim_b.world.regions.len());
        for (ra, rb) in sim_a.world.regions.iter().zip(sim_b.world.regions.iter()) {
            assert_eq!(ra.settlements.len(), rb.settlements.len());
            for (sa, sb) in ra.settlements.iter().zip(rb.settlements.iter()) {
                assert_eq!(
                    sa.people.len(),
                    sb.people.len(),
                    "population mismatch in {}",
                    sa.name
                );
                for (pa, pb) in sa.people.iter().zip(sb.people.iter()) {
                    assert_eq!(pa.id, pb.id, "person id mismatch");
                    assert_eq!(pa.region, pb.region, "region mismatch for {}", pa.id);
                    assert_eq!(
                        pa.settlement, pb.settlement,
                        "settlement mismatch for {}",
                        pa.id
                    );
                }
            }
        }
    }

    #[test]
    fn job_seeking_migration_runs() {
        let mut sim = make_sim(12345);
        // Give someone low safety to trigger job-seeking
        if sim.world.regions.is_empty() || sim.world.regions[0].settlements.len() < 2 {
            return;
        }
        if sim.world.regions[0].settlements[0].people.is_empty()
            || sim.world.regions[0].settlements[1].people.is_empty()
        {
            return;
        }

        let region = &mut sim.world.regions[0];
        region.settlements[0].people[0]
            .needs
            .values
            .insert(crate::model::Need::Safety, 0.1);

        // Run enough ticks for migration checks
        for _ in 0..150 {
            sim.step();
        }

        // Verify structure integrity: population counts match people.len()
        for region in &sim.world.regions {
            for settlement in &region.settlements {
                assert_eq!(
                    settlement.population,
                    settlement.people.len() as u32,
                    "population should be derived from people.len()"
                );
            }
        }
    }

    #[test]
    fn no_orphaning_after_migration() {
        let mut sim = make_sim(77);
        for _ in 0..200 {
            sim.step();
        }

        // Collect all person IDs; verify no duplicates
        let mut all_person_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for region in &sim.world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    assert!(
                        !all_person_ids.contains(&person.id),
                        "person {} appears in multiple settlements",
                        person.id
                    );
                    all_person_ids.insert(person.id.clone());
                }
            }
        }

        // Verify person.settlement matches actual location
        for region in &sim.world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    assert_eq!(
                        person.settlement, settlement.id,
                        "person {} says they're in {} but found in {}",
                        person.id, person.settlement, settlement.id
                    );
                    assert_eq!(
                        person.region, region.id,
                        "person {} says they're in region {} but found in {}",
                        person.id, person.region, region.id
                    );
                }
            }
        }

        // All population counts match people.len()
        for region in &sim.world.regions {
            for settlement in &region.settlements {
                assert_eq!(
                    settlement.population,
                    settlement.people.len() as u32,
                    "population count mismatch in settlement {}",
                    settlement.id
                );
            }
        }
    }

    #[test]
    fn flight_migration_low_reputation() {
        let mut sim = make_sim(999);
        if sim.world.regions.is_empty() || sim.world.regions[0].settlements.is_empty() {
            return;
        }
        let person_id = sim.world.regions[0].settlements[0].people[0].id.clone();
        let settlement_id = sim.world.regions[0].settlements[0].id.clone();

        for _ in 0..20 {
            sim.reputation
                .adjust_local(&person_id, &settlement_id, -0.1);
        }

        let rep_before = sim.reputation.get(&person_id, &settlement_id);
        assert!(
            rep_before <= 0.05,
            "reputation should be near floor: got {}",
            rep_before
        );

        for _ in 0..200 {
            sim.step();
        }

        for region in &sim.world.regions {
            for settlement in &region.settlements {
                assert_eq!(
                    settlement.population,
                    settlement.people.len() as u32,
                    "population should match people count in {}",
                    settlement.name
                );
            }
        }
    }

    #[test]
    fn marriage_migration_with_bias() {
        let mut sim = make_sim(555);
        for _ in 0..300 {
            sim.step();
        }

        // Verify no person is in two places at once
        let mut person_settlements: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for region in &sim.world.regions {
            for settlement in &region.settlements {
                for person in &settlement.people {
                    if let Some(existing) = person_settlements.get(&person.id) {
                        assert_eq!(
                            existing, &settlement.id,
                            "person {} found in both {} and {}",
                            person.id, existing, settlement.id
                        );
                    }
                    person_settlements.insert(person.id.clone(), settlement.id.clone());
                }
            }
        }

        // All population counts match
        for region in &sim.world.regions {
            for settlement in &region.settlements {
                assert_eq!(settlement.population, settlement.people.len() as u32);
            }
        }
    }

    #[test]
    fn migration_preserves_total_population() {
        let sim_before = make_sim(42);
        let total_before: usize = sim_before
            .world
            .regions
            .iter()
            .flat_map(|r| r.settlements.iter())
            .map(|s| s.people.len())
            .sum();

        // Call migration directly at a boundary: it must move people without
        // creating or losing any. (The full sim no longer conserves headcount
        // — the lifecycle system births and buries people by design.)
        let mut sim = make_sim(42);
        for boundary in 1..=16u64 {
            tick_migration(&mut sim, boundary * 30);
        }
        let total_after: usize = sim
            .world
            .regions
            .iter()
            .flat_map(|r| r.settlements.iter())
            .map(|s| s.people.len())
            .sum();

        assert_eq!(
            total_before, total_after,
            "migration must preserve total population"
        );
    }

    #[test]
    fn profession_demand_basic() {
        let person = crate::model::Person {
            profession: "smith".to_string(),
            ..Default::default()
        };
        let settlement = crate::model::Settlement {
            id: "test-set".into(),
            name: "Test".into(),
            size: "village".into(),
            region: "test".into(),
            population: 5,
            description: String::new(),
            people: vec![person],
            services: vec![crate::model::SettlementService::Forge],
            politics: crate::model::SettlementPolitics::new(),
            food_stock: 0.0,
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
        };
        let demand = profession_demand(&settlement, "smith");
        assert!(
            demand > 0.0,
            "demand for smith with Forge service should be positive: got {}",
            demand
        );
    }

    #[test]
    fn profession_demand_saturated() {
        let mut people = Vec::new();
        for i in 0..10 {
            people.push(crate::model::Person {
                id: format!("smith-{}", i),
                profession: "smith".to_string(),
                ..Default::default()
            });
        }
        let settlement = crate::model::Settlement {
            id: "test-set".into(),
            name: "Test".into(),
            size: "village".into(),
            region: "test".into(),
            population: 10,
            description: String::new(),
            people,
            services: vec![crate::model::SettlementService::Forge],
            politics: crate::model::SettlementPolitics::new(),
            food_stock: 0.0,
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
        };
        let demand = profession_demand(&settlement, "smith");
        assert!(
            demand <= 0.0,
            "demand for smith with 10 smiths should be non-positive: got {}",
            demand
        );
    }
}
