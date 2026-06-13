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

/// Base chance a birth carries a complication before the mother's fortune leans
/// it. The complication lingers as illness; luck rides this roll like the rest.
const BASE_CHILDBIRTH_COMPLICATION_PROB: f64 = 0.05;

/// The chance a birth carries a complication, leaned by the mother's own hidden
/// fortune. The blessed come through clean more often; the cursed take it.
pub(crate) fn childbirth_complication_chance(mother: crate::model::Fortune) -> f64 {
    mother.tilt_bad(BASE_CHILDBIRTH_COMPLICATION_PROB)
}

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
            let settlement_name = settlement.name.clone();

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
                    // A hard birth: the existing childbirth-complication roll,
                    // leaned by the mother's own hidden fortune (rolled from her
                    // life-seed). The blessed come through clean more often; the
                    // cursed take the complication, which lingers as illness and
                    // raises her odds in the years after. Deterministic.
                    let mother_fortune =
                        crate::model::Fortune::roll(seed, crate::rng::fnv1a_hash(&p.id));
                    let comp_p = childbirth_complication_chance(mother_fortune);
                    let mut crng =
                        SeedRng::new(seed).fork_for(&format!("childbirth-{year}-{}", p.id));
                    let comp_roll = crng.gen_range(10000) as f64 / 10000.0;
                    if comp_roll < comp_p
                        && !p
                            .illnesses
                            .iter()
                            .any(|d| d.disease == crate::model::Disease::ChildbirthComplication)
                    {
                        p.illnesses.push(crate::model::ActiveDisease::new(
                            crate::model::Disease::ChildbirthComplication,
                            tick,
                        ));
                        events.push(format!(
                            "{} of {} has come through a hard birth.",
                            p.name, settlement_name
                        ));
                    }
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

#[cfg(test)]
mod tests {
    use super::childbirth_complication_chance;
    use crate::model::Fortune;
    use crate::rng::fnv1a_hash;

    #[test]
    fn cursed_mother_takes_more_complications() {
        let cursed = childbirth_complication_chance(Fortune::from_value(-1.0));
        let plain = childbirth_complication_chance(Fortune::from_value(0.0));
        let blessed = childbirth_complication_chance(Fortune::from_value(1.0));
        assert!(
            cursed > plain && plain > blessed,
            "complication should rise with ill fortune: cursed={cursed} plain={plain} blessed={blessed}"
        );
    }

    #[test]
    fn complication_chance_stays_a_chance() {
        for v in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let p = childbirth_complication_chance(Fortune::from_value(v));
            assert!(p > 0.0 && p < 1.0, "chance {p} out of (0,1) at fortune {v}");
        }
    }

    #[test]
    fn mother_fortune_is_deterministic_from_her_id() {
        // The same mother (same seed + id) is born under the same star, so her
        // complication odds are stable run to run; a different id, a different star.
        let seed = 42u64;
        let a = Fortune::roll(seed, fnv1a_hash("person-7")).value();
        let b = Fortune::roll(seed, fnv1a_hash("person-7")).value();
        let other = Fortune::roll(seed, fnv1a_hash("person-8")).value();
        assert_eq!(a, b);
        assert_ne!(a, other);
    }
}
