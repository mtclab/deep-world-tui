use crate::model::{PeopleKind, World};
use crate::rng::SeedRng;
use crate::sim::journal::Voice;
use serde::{Deserialize, Serialize};

const MIGRATION_INTERVAL: u64 = 30;
/// How long a migrant party is on the road before it reaches its new town
/// (#641 slice 3): a day and a half at the half-hour-walk scale — long enough
/// to be seen crossing the country, short enough not to pile up.
const MIGRANT_TRAVEL_TICKS: u64 = 36;

/// A household on the move between towns (#641 slice 3): a migration is no
/// longer an instant teleport in the roster. The migrant leaves their town and
/// walks the road as a party — seen on the grid as individuals — until they
/// reach their new home, when they join its people. The carried `people` are
/// off every settlement roster for the journey (counted only here), so the
/// world conserves its souls without doubling them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrantParty {
    pub id: String,
    /// The souls on the road — removed from their old town, not yet in the new.
    pub people: Vec<crate::model::Person>,
    /// The town they are bound for, by id (robust to roster index shifts).
    pub dest_settlement_id: String,
    pub origin_region: usize,
    pub origin_x: usize,
    pub origin_y: usize,
    pub dest_region: usize,
    pub dest_x: usize,
    pub dest_y: usize,
    pub departure_tick: u64,
    pub arrival_tick: u64,
}

impl MigrantParty {
    /// How far along the road the party is at `tick`, 0.0..=1.0.
    pub fn progress(&self, tick: u64) -> f64 {
        if self.arrival_tick <= self.departure_tick {
            return 1.0;
        }
        let span = (self.arrival_tick - self.departure_tick) as f64;
        (tick.saturating_sub(self.departure_tick) as f64 / span).clamp(0.0, 1.0)
    }
}
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
    // The fourth field marks a routine job-seeking move (#614 polish): these are
    // the bulk of migration and were flooding the player's journal one line per
    // person. They are now tallied into a single seasonal summary, while the
    // rarer, character-driven marriage and flight moves keep their own lines.
    let mut migrants: Vec<(MigrationCandidate, SettlementRef, String, bool)> = Vec::new();

    let settlement_refs = find_settlement_refs(&sim.world);
    if settlement_refs.len() < 2 {
        return;
    }

    // Souls who leave the settled lands entirely — not for another town, but
    // for the open road (#623). Gathered here, removed and counted below.
    let mut leavers: Vec<(MigrationCandidate, String)> = Vec::new();

    for (sri, sref) in settlement_refs.iter().enumerate() {
        let settlement = &sim.world.regions[sref.region_idx].settlements[sref.settlement_idx];
        let adjacent = find_adjacent_settlements(&sim.world, sref.region_idx);

        // A village worn by hunger or feud, or simply crowded past its land,
        // pushes its restless toward the road (#623). One pressure read per
        // town, deterministic; the harder-pressed shed the more.
        let town_pressure = {
            let famine = settlement.famine_days > 0;
            let feud = crate::model::relation::feud_load(&settlement.people) >= 0.4;
            famine || feud
        };

        for person in settlement.people.iter() {
            // The road takes the young and unattached first (#623): a youth or
            // young adult with no spouse, in a pressed town, whose own safety
            // has worn thin, may leave the settled lands for the ungoverned
            // dark. Rare and deterministic — a village sheds a few, not a flood.
            if town_pressure
                && !person.has_spouse
                && matches!(person.age_band.as_str(), "youth" | "adult")
                && person.needs.get(crate::model::Need::Safety) < 0.35
            {
                let mut leave_rng =
                    SeedRng::new(seed).fork_for(&format!("road-{}-{}-{}", person.id, sri, tick));
                if leave_rng.gen_f64() < 0.06 {
                    leavers.push((
                        MigrationCandidate {
                            person_id: person.id.clone(),
                            source_settlement_idx: sref.settlement_idx,
                            source_region_idx: sref.region_idx,
                        },
                        person.name.clone(),
                    ));
                    continue;
                }
            }

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
                        true,
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
                        false,
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
                            false,
                        ));
                    }
                }
            }
        }
    }

    // Apply migrations - reverse sort by source region/settlement to preserve indices
    migrants.sort_by(
        |a: &(MigrationCandidate, SettlementRef, String, bool),
         b: &(MigrationCandidate, SettlementRef, String, bool)| {
            b.0.source_region_idx
                .cmp(&a.0.source_region_idx)
                .then(b.0.source_settlement_idx.cmp(&a.0.source_settlement_idx))
        },
    );

    let mut migrated_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut routine_count = 0u32;

    for (cand, target, reason, routine) in &migrants {
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

        let target_settlement =
            &sim.world.regions[target_region_idx].settlements[target_settlement_idx];
        let target_settlement_id = target_settlement.id.clone();
        let target_region_id = sim.world.regions[target_region_idx].id.clone();
        let (dest_x, dest_y) = (
            target_settlement.map_x as usize,
            target_settlement.map_y as usize,
        );
        let (origin_x, origin_y) = {
            let s = &sim.world.regions[source_region_idx].settlements[source_settlement_idx];
            (s.map_x as usize, s.map_y as usize)
        };

        // Update person's location to their new town now — but they do not join
        // its roster yet (#641 slice 3): they walk the road as a party and
        // arrive in `complete_migrant_arrivals`. The roster move is no longer
        // instant; the world sees them cross the country.
        let mut moved_person = person;
        moved_person.region = target_region_id;
        moved_person.settlement = target_settlement_id.clone();

        sim.migrant_parties.push(MigrantParty {
            id: format!("migrant-{}-{}", cand.person_id, tick),
            people: vec![moved_person],
            dest_settlement_id: target_settlement_id,
            origin_region: source_region_idx,
            origin_x,
            origin_y,
            dest_region: target_region_idx,
            dest_x,
            dest_y,
            departure_tick: tick,
            arrival_tick: tick + MIGRANT_TRAVEL_TICKS,
        });

        migrated_ids.insert(cand.person_id.clone());

        // Routine job-seeking moves are tallied for one summary line below; the
        // rarer marriage and flight moves are journaled in their own words. Word
        // travels when they set out, even as their feet are still on the road.
        if *routine {
            routine_count += 1;
        } else {
            sim.log_journal(tick, reason.clone());
        }

        // The source town empties now (they have left); the destination's count
        // rises only when they actually arrive.
        sim.world.regions[source_region_idx].settlements[source_settlement_idx].population =
            sim.world.regions[source_region_idx].settlements[source_settlement_idx]
                .people
                .len() as u32;
    }

    // One seasonal summary stands in for the many routine job-seeking moves, so
    // the journal carries the world's churn without drowning in it (#614 polish).
    if routine_count > 0 {
        let line = if routine_count == 1 {
            "Word on the road: a tradesperson crossed the ridges this season, chasing steady work."
                .to_string()
        } else {
            format!(
                "Word on the road: {routine_count} tradespeople crossed the ridges this season, chasing steady work."
            )
        };
        sim.log(tick, Voice::Rumor, line);
    }

    // The road takes the restless (#623): remove those who left the settled
    // lands, shrink their town, and add them to the frontier's wanderers — the
    // raw material of the bands to come. One named line carries the season's
    // leaving, so a village emptying of its young is felt, not droned.
    let mut left_names: Vec<String> = Vec::new();
    for (cand, name) in &leavers {
        if migrated_ids.contains(&cand.person_id) {
            continue;
        }
        let s =
            &mut sim.world.regions[cand.source_region_idx].settlements[cand.source_settlement_idx];
        if let Some(idx) = s.people.iter().position(|p| p.id == cand.person_id) {
            s.people.remove(idx);
            s.population = s.population.saturating_sub(1).max(s.people.len() as u32);
            migrated_ids.insert(cand.person_id.clone());
            sim.frontier.take_the_road();
            left_names.push(name.clone());
        }
    }
    if let Some(first) = left_names.first() {
        let line = if left_names.len() == 1 {
            format!("Word on the road: {first} left the village for the open country, done with the life the land offered.")
        } else {
            format!(
                "Word on the road: {first} and {} others took the road into the ungoverned country this season.",
                left_names.len() - 1
            )
        };
        sim.log(tick, Voice::Rumor, line);
    }
}

/// Land the migrant parties whose road is run (#641 slice 3): each party that
/// has reached its arrival tick joins its people to their new town, raising its
/// count, and leaves the road. If the destination town has died while they
/// walked, the party is dropped — the road did not deliver them. Called every
/// tick so a party arrives the moment its journey is done.
pub fn complete_migrant_arrivals(sim: &mut crate::sim::SimState, tick: u64) {
    if sim.migrant_parties.is_empty() {
        return;
    }
    let arrived: Vec<MigrantParty> = {
        let mut still = Vec::new();
        let mut done = Vec::new();
        for party in sim.migrant_parties.drain(..) {
            if party.arrival_tick <= tick {
                done.push(party);
            } else {
                still.push(party);
            }
        }
        sim.migrant_parties = still;
        done
    };
    for party in arrived {
        // Find the destination town by id (its roster index may have shifted).
        let place = sim.world.regions.iter().position(|r| {
            r.settlements
                .iter()
                .any(|s| s.id == party.dest_settlement_id)
        });
        let Some(ri) = place else {
            continue; // the town is gone; the road kept them
        };
        let si = sim.world.regions[ri]
            .settlements
            .iter()
            .position(|s| s.id == party.dest_settlement_id)
            .unwrap();
        let settlement = &mut sim.world.regions[ri].settlements[si];
        for person in party.people {
            settlement.people.push(person);
        }
        settlement.population = settlement.people.len() as u32;
    }
}

/// Where a migrant party stands on `region_idx`'s grid at `tick`: each soul on
/// the road its own tile, head first and trailing back, their place
/// interpolated along the route (#641 slice 3). Empty if the party is not on
/// this region's ground now, or its head falls off the map.
pub fn migrant_party_tiles(
    sim: &crate::sim::SimState,
    party_id: &str,
    region_idx: usize,
    tick: u64,
) -> Vec<(usize, usize)> {
    let Some(party) = sim.migrant_parties.iter().find(|p| p.id == party_id) else {
        return Vec::new();
    };
    let t = party.progress(tick);
    let Some((hx, hy, (dx, dy))) = crate::sim::caravans::road_point_in_region(
        sim,
        region_idx,
        (party.origin_region, party.origin_x, party.origin_y),
        (party.dest_region, party.dest_x, party.dest_y),
        t,
    ) else {
        return Vec::new();
    };
    let Some(region) = sim.world.regions.get(region_idx) else {
        return Vec::new();
    };
    let (w, h) = (region.terrain.width, region.terrain.height);
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut taken: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    // The party walks as a small file: a soul per member on the road, trailing
    // one tile back up the route from the head.
    for i in 0..party.people.len().max(1) {
        let mx = hx - dx * i as f64;
        let my = hy - dy * i as f64;
        let (tx, ty) = (mx.round(), my.round());
        if tx < 0.0 || ty < 0.0 || tx >= w as f64 || ty >= h as f64 {
            continue;
        }
        let tile = (tx as usize, ty as usize);
        if taken.insert(tile) {
            out.push(tile);
        }
    }
    out
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
    fn a_migrant_party_walks_the_road_then_joins_its_new_town() {
        // #641 slice 3: a migration is not an instant teleport — the party is on
        // the road for its travel time, seen on the grid, then arrives.
        let mut sim = make_sim(7);
        let (dest_id, ri, si, before) = {
            let r = &sim.world.regions[0];
            // A destination town with two settlements in the region for a tile.
            (
                r.settlements[1].id.clone(),
                0usize,
                1usize,
                r.settlements[1].people.len(),
            )
        };
        let (ox, oy, dx, dy) = {
            let r = &sim.world.regions[0];
            (
                r.settlements[0].map_x as usize,
                r.settlements[0].map_y as usize,
                r.settlements[1].map_x as usize,
                r.settlements[1].map_y as usize,
            )
        };
        let mut traveller = crate::model::Person {
            id: "traveller-1".into(),
            ..Default::default()
        };
        traveller.settlement = dest_id.clone();
        sim.migrant_parties.push(MigrantParty {
            id: "party-1".into(),
            people: vec![traveller],
            dest_settlement_id: dest_id.clone(),
            origin_region: 0,
            origin_x: ox,
            origin_y: oy,
            dest_region: 0,
            dest_x: dx,
            dest_y: dy,
            departure_tick: 0,
            arrival_tick: 36,
        });

        // Mid-journey: still on the road, drawn on the grid, not yet in town.
        complete_migrant_arrivals(&mut sim, 18);
        assert_eq!(
            sim.migrant_parties.len(),
            1,
            "still travelling at the midpoint"
        );
        assert!(
            !migrant_party_tiles(&sim, "party-1", ri, 18).is_empty(),
            "the party stands on the road, seen on the grid"
        );
        assert_eq!(
            sim.world.regions[ri].settlements[si].people.len(),
            before,
            "they have not joined the town yet"
        );

        // The road run: they arrive and join their new town.
        complete_migrant_arrivals(&mut sim, 36);
        assert!(sim.migrant_parties.is_empty(), "the party has arrived");
        assert!(
            sim.world.regions[ri].settlements[si]
                .people
                .iter()
                .any(|p| p.id == "traveller-1"),
            "the migrant has joined their new town's people"
        );
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
                // The people vec is a SAMPLE of the population on the canon
                // scale: the roll never exceeds the head-count.
                assert!(
                    settlement.population >= settlement.people.len() as u32,
                    "sample exceeds head-count in settlement {}",
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
                assert!(
                    settlement.population >= settlement.people.len() as u32,
                    "sample exceeds head-count in {}",
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
        // Souls in towns PLUS souls on the road (#641 slice 3): a migrant party
        // holds its people off every roster while it walks, so the count must
        // include them or the road would look like it swallowed people.
        let in_towns: usize = sim
            .world
            .regions
            .iter()
            .flat_map(|r| r.settlements.iter())
            .map(|s| s.people.len())
            .sum();
        let on_the_road: usize = sim.migrant_parties.iter().map(|p| p.people.len()).sum();
        let total_after = in_towns + on_the_road;

        assert_eq!(
            total_before, total_after,
            "migration must preserve total population (towns + the road)"
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
            faith: Default::default(),
            food_stock: 0.0,
            goods_stock: Default::default(),
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            plague_days: 0,
            map_x: 0,
            map_y: 0,
            district: 0,
            remembered_deed: None,
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
            faith: Default::default(),
            food_stock: 0.0,
            goods_stock: Default::default(),
            farms: Vec::new(),
            buildings: Vec::new(),
            festival_until_day: 0,
            famine_days: 0,
            plague_days: 0,
            map_x: 0,
            map_y: 0,
            district: 0,
            remembered_deed: None,
        };
        let demand = profession_demand(&settlement, "smith");
        assert!(
            demand <= 0.0,
            "demand for smith with 10 smiths should be non-positive: got {}",
            demand
        );
    }
}
