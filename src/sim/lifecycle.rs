//! NPC lifecycle: the world's people are born, age through their bands, and
//! die — of age, faster when ill. Before this, populations only relocated;
//! nobody had ever been born or died of age anywhere.
use crate::model::Person;
use crate::rng::SeedRng;
use crate::sim::{SimState, Voice};

/// One NPC life-year per calendar year (90 days, `Season::YEAR_DAYS`).
///
/// The PLAYER ages on a compressed clock (3 game days per life-year, so a whole
/// life fits a session). That conceit is the player's alone. Before the
/// entity-first epic it was harmless to the world: `population` was a bulk number
/// that never aged, and only a ~400-soul sample ran the lifecycle. Once
/// materialization made EVERY inhabitant a real, aging soul, that same 3-day
/// clock churned the whole province through ~30 generations a calendar year and
/// emptied it (8.5k→<300 in a year). So the world's people now age at the
/// world's own calendar pace — the province stays a living, stable society across
/// the many short lives the player and their heirs live within it.
pub const TICKS_PER_LIFE_YEAR: u64 = 90 * 24;

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

/// Who takes up a dead master's skilled trade (#623 slice 6). Adoption before
/// chance: if the master kept an apprentice (a relation of their own) who still
/// lives among `people` and does not already hold the trade, the trade — and,
/// in a real sense, the line — passes to that apprentice; the bool marks it an
/// adoption. Failing an apprentice, the trade falls to any young plain hand
/// (labourer/farmer) who can take it up. `None` if no one can. Pure and
/// deterministic; `people` must not include the dead.
fn trade_successor(dead: &Person, people: &[Person]) -> Option<(usize, bool)> {
    let apprentice_id = dead
        .relations
        .iter()
        .find(|r| r.kind == crate::model::relation::RelationKind::Apprentice)
        .map(|r| r.target_person_id.as_str());
    if let Some(appr_id) = apprentice_id {
        if let Some(idx) = people
            .iter()
            .position(|q| q.id == appr_id && q.profession != dead.profession)
        {
            return Some((idx, true));
        }
    }
    people
        .iter()
        .position(|q| {
            matches!(q.age_band.as_str(), "youth" | "adult")
                && matches!(q.profession.as_str(), "labourer" | "farmer")
        })
        .map(|idx| (idx, false))
}

/// Pass a dead soul's purse to the nearest of kin still in town (households-as-
/// economic-units #56-F): a spouse first, then a child, sibling, or parent, so a
/// family's wealth — and its chance to rise — outlives any one of them. With no
/// kin to take it, the coin escheats to the common purse (treasury). Deterministic
/// (first matching relation, roster order).
pub fn bequeath(dead: &Person, heirs: &mut [Person], treasury: &mut u32) {
    if dead.coins == 0 && dead.wares == 0 {
        return;
    }
    use crate::model::relation::RelationKind;
    let heir_id = dead
        .relations
        .iter()
        .find(|r| r.kind == RelationKind::Spouse)
        .or_else(|| {
            dead.relations.iter().find(|r| {
                matches!(
                    r.kind,
                    RelationKind::Child | RelationKind::Sibling | RelationKind::Parent
                )
            })
        })
        .map(|r| r.target_person_id.clone());
    match heir_id.and_then(|id| heirs.iter_mut().find(|p| p.id == id)) {
        // Coin and the goods of the trade alike pass to the heir (#54 slice 5).
        Some(h) => {
            h.coins = h.coins.saturating_add(dead.coins);
            h.wares = h.wares.saturating_add(dead.wares);
        }
        // With no kin, the purse escheats to the commons, and the town sells off
        // the goods left behind (a coin a piece) into the same common purse.
        None => {
            *treasury = treasury
                .saturating_add(dead.coins)
                .saturating_add(dead.wares)
        }
    }
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
        // Snapshot the land so we can read its carrying capacity per settlement
        // without fighting the settlement borrow below.
        let rtype = region.region_type.clone();
        let terr_tiles = region.terrain.tiles.clone();
        let (terr_w, terr_h) = (region.terrain.width, region.terrain.height);
        for settlement in region.settlements.iter_mut() {
            let sample = settlement.people.len().max(1);
            let scale =
                ((settlement.population.max(1) as f64 / sample as f64).round() as u32).max(1);
            let settlement_name = settlement.name.clone();
            // What the land can feed (entity-first epic): every birth is now a
            // real resident, so without a ceiling a young town grows without
            // bound (births >> deaths) and the roster runs away. The land caps
            // it — a settlement at capacity sees births balance deaths, no more.
            // Read the settlement's founding assay (#728), not a fresh sample of
            // terrain it has since built over (which collapsed the ceiling ~10x
            // and, mistaking a fed town for one past capacity, culled its
            // births into decline). A `0` is assayed once and cached.
            if settlement.land_capacity == 0 {
                settlement.land_capacity = crate::gen::town::carrying_capacity(
                    &terr_tiles,
                    terr_w,
                    terr_h,
                    settlement.map_x as usize + 1,
                    settlement.map_y as usize + 1,
                    &rtype,
                );
            }
            let carrying = settlement.land_capacity as usize;

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
                    && roll > 1.0 - 0.22
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
                // Inheritance (households-as-economic-units #56-F): a life's purse
                // does not vanish into the ground.
                bequeath(&dead, &mut settlement.people, &mut settlement.treasury);
                if is_skilled(&dead.profession) {
                    if let Some((idx, adopted)) = trade_successor(&dead, &settlement.people) {
                        let heir_name = settlement.people[idx].name.clone();
                        settlement.people[idx].profession = dead.profession.clone();
                        events.push(if adopted {
                            format!(
                                "{} of {} has died with no child to leave the trade to — {}, the apprentice, takes up the {}'s work and the master's name with it.",
                                dead.name, settlement.name, heir_name, dead.profession
                            )
                        } else {
                            format!(
                                "{} of {} has died. {} takes up the {}'s work.",
                                dead.name, settlement.name, heir_name, dead.profession
                            )
                        });
                        continue;
                    }
                }
                events.push(format!(
                    "{} of {} has died, full of years.",
                    dead.name, settlement.name
                ));
            }

            // Births: every birth is a real new resident (entity-first epic), at
            // the natural rate, and only while the land can feed another mouth.
            // No forced replacement — a population must be free to fall as well as
            // rise (some years bury more than they bear); guaranteeing it never
            // declines would make every town grow monotonically to its carrying
            // cap, ballooning the province. The fed-town-holds property is a
            // separate BALANCE pass (the deferred growth_decline test).
            for b in 0..births {
                if settlement.people.len() >= carrying.max(1) {
                    break;
                }
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
            settlement.population = settlement.people.len() as u32;
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
    use super::{childbirth_complication_chance, trade_successor};
    use crate::model::relation::{InterNpcRelation, RelationKind};
    use crate::model::Fortune;
    use crate::rng::{fnv1a_hash, SeedRng};

    fn a_person(profession: &str, age_band: &str) -> crate::model::Person {
        let charts = crate::charts::load::load_charts().expect("charts");
        let mut rng = SeedRng::new(1).fork_for(&format!("test-person-{profession}-{age_band}"));
        let mut p = crate::gen::person::generate_person(&mut rng, &charts);
        p.profession = profession.into();
        p.age_band = age_band.into();
        p.relations.clear();
        p
    }

    #[test]
    fn a_master_leaves_the_trade_to_their_apprentice() {
        let mut master = a_person("smith", "elder");
        let apprentice = a_person("labourer", "adult");
        // The master kept this apprentice — a relation of their own.
        master.relations.push(InterNpcRelation {
            kind: RelationKind::Apprentice,
            target_person_id: apprentice.id.clone(),
            intensity: 0.6,
            formed_at_tick: 0,
            reason: "apprenticed at the bench".into(),
        });
        let bystander = a_person("farmer", "adult");
        // People list: bystander first, apprentice second — adoption must beat
        // the plain-hand fallback even though the bystander could take it.
        let people = vec![bystander, apprentice.clone()];
        let (idx, adopted) = trade_successor(&master, &people).expect("a successor");
        assert!(adopted, "the apprentice is adopted into the trade");
        assert_eq!(
            people[idx].id, apprentice.id,
            "it is the apprentice who inherits"
        );
    }

    #[test]
    fn with_no_apprentice_a_young_hand_takes_the_trade() {
        let master = a_person("weaver", "elder"); // no apprentice relation
        let young = a_person("labourer", "youth");
        let people = vec![young.clone()];
        let (idx, adopted) = trade_successor(&master, &people).expect("a successor");
        assert!(!adopted, "no apprentice — this is chance, not adoption");
        assert_eq!(people[idx].id, young.id);
    }

    #[test]
    fn a_trade_with_no_one_to_take_it_passes_to_no_one() {
        let master = a_person("scribe", "elder");
        // Only skilled elders about — no young plain hand, no apprentice.
        let people = vec![a_person("smith", "elder")];
        assert!(trade_successor(&master, &people).is_none());
    }

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
