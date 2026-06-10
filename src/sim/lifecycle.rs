//! NPC lifecycle: the world's people are born, age through their bands, and
//! die — of age, faster when ill. Before this, populations only relocated;
//! nobody had ever been born or died of age anywhere.
use crate::model::Person;
use crate::rng::SeedRng;
use crate::sim::{SimState, Voice};

/// One NPC life-year per 3 game days — the same clock the player ages on.
pub const TICKS_PER_LIFE_YEAR: u64 = 72;

/// Sample-roster cap per settlement (the roster represents a larger population).
const MAX_SAMPLE_PEOPLE: usize = 40;

pub fn age_from_band(band: &str) -> u32 {
    match band {
        "child" => 8,
        "youth" | "young" => 17,
        "elder" | "old" => 62,
        "aged" => 72,
        _ => 32, // adult / unknown
    }
}

pub fn band_from_age(age: u32) -> &'static str {
    if age < 14 {
        "child"
    } else if age < 22 {
        "youth"
    } else if age < 58 {
        "adult"
    } else if age < 70 {
        "elder"
    } else {
        "aged"
    }
}

/// Professions worth passing on when their holder dies.
fn is_skilled(profession: &str) -> bool {
    matches!(
        profession,
        "healer" | "herbalist" | "smith" | "trader" | "scribe" | "priest" | "weaver" | "carpenter"
    )
}

pub fn tick_lifecycle(sim: &mut SimState) {
    let tick = sim.world.tick;
    if tick == 0 || !tick.is_multiple_of(TICKS_PER_LIFE_YEAR) {
        return;
    }
    let year = tick / TICKS_PER_LIFE_YEAR;
    let seed = sim.world.seed;
    let charts = sim.charts.clone();
    let mut events: Vec<String> = Vec::new();
    let mut died: Vec<(String, String, String)> = Vec::new();

    for region in sim.world.regions.iter_mut() {
        let region_id = region.id.clone();
        for settlement in region.settlements.iter_mut() {
            let sample = settlement.people.len().max(1);
            let scale =
                ((settlement.population.max(1) as f64 / sample as f64).round() as u32).max(1);

            let mut deaths: Vec<usize> = Vec::new();
            let mut births: u32 = 0;
            for (i, p) in settlement.people.iter_mut().enumerate() {
                if p.age_years == 0 {
                    p.age_years = age_from_band(&p.age_band);
                }
                p.age_years += 1;
                let band = band_from_age(p.age_years);
                if band != p.age_band {
                    p.age_band = band.to_string();
                }
                let mut rng = SeedRng::new(seed).fork_for(&format!("life-{year}-{}", p.id));
                let roll = rng.gen_range(1000) as f64 / 1000.0;
                let ill = if p.illnesses.is_empty() { 0.0 } else { 0.08 };
                let death_chance = match band {
                    "aged" => 0.18 + ill,
                    "elder" => 0.05 + ill,
                    "child" => 0.01 + ill,
                    _ => 0.004 + ill,
                };
                if roll < death_chance {
                    deaths.push(i);
                } else if p.has_spouse
                    && p.sex == "f"
                    && matches!(band, "youth" | "adult")
                    && roll > 1.0 - 0.10
                {
                    births += 1;
                }
            }

            // Deaths: remove from the roster, shrink the population, pass the
            // trade on where someone young can take it up.
            for &i in deaths.iter().rev() {
                let dead = settlement.people.remove(i);
                died.push((dead.id.clone(), dead.name.clone(), settlement.name.clone()));
                settlement.population = settlement
                    .population
                    .saturating_sub(scale)
                    .max(settlement.people.len() as u32);
                if is_skilled(&dead.profession) {
                    if let Some(heir) = settlement.people.iter_mut().find(|q| {
                        matches!(q.age_band.as_str(), "youth" | "adult")
                            && matches!(q.profession.as_str(), "labourer" | "farmer")
                    }) {
                        heir.profession = dead.profession.clone();
                        events.push(format!(
                            "{} of {} has died. {} takes up the {}'s work.",
                            dead.name, settlement.name, heir.name, dead.profession
                        ));
                        continue;
                    }
                }
                events.push(format!(
                    "{} of {} has died, full of years.",
                    dead.name, settlement.name
                ));
            }

            // Births: a new child joins the roster (capped — the roster is a
            // sample) and the population grows either way.
            for b in 0..births {
                settlement.population += scale;
                if settlement.people.len() < MAX_SAMPLE_PEOPLE {
                    let rng =
                        SeedRng::new(seed).fork_for(&format!("birth-{year}-{}-{b}", settlement.id));
                    let mut child: Person = crate::gen::person::generate_person_from(
                        rng,
                        &region_id,
                        &settlement.id,
                        &charts,
                    );
                    child.age_band = "child".into();
                    child.age_years = 1;
                    child.has_spouse = false;
                    child.children_count = 0;
                    // A child of the settlement's people, not a random draw.
                    if let Some(parent) = settlement.people.first() {
                        child.people = parent.people.clone();
                    }
                    child.settlement = settlement.id.clone();
                    events.push(format!("A child was born in {}.", settlement.name));
                    settlement.people.push(child);
                }
            }
        }
    }

    // The world talks about a few of these; the journal isn't a census.
    for e in events.into_iter().take(4) {
        sim.log(tick, Voice::Rumor, e);
    }
    // The player's people are not statistics. A friend's death cuts.
    for (id, name, place) in died {
        let trust = sim
            .npc_memories
            .get(&id)
            .map(|m| m.cumulative_trust())
            .unwrap_or(0.0);
        if trust >= 0.15 {
            sim.log(
                tick,
                Voice::Scar,
                format!("{name} of {place} is gone. I knew them. The road is poorer."),
            );
        }
    }
}
