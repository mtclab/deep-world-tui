//! Rumors that carry real information. The tavern channel used to deal only
//! in generic flavor lines; now it reports the actual state of the world —
//! famine and plenty (tradable!), caravans on the road, festivals, buildings
//! going up — so an attentive player can act on what they overhear.
use crate::sim::SimState;

/// Word that one of the canon non-human peoples is trading in a given land this
/// season — so the barter encounters (#443/#445/#447) can be sought out, not
/// only stumbled into. Each people is named where its terrain dominates: the
/// Mëräk on the coast, the Khör on the steppe, the Häl in the forest, the
/// Tzäkhar in the uplands. (The She'ar have no steppe region in the province
/// map, so no rumor points at empty ground.) Deterministic per land + season;
/// fires only about a third of seasons so the news is worth carrying.
fn nonhuman_trade_rumor(
    region_type: &str,
    seed: u64,
    region_id: &str,
    season_ord: u32,
    year: u32,
) -> Option<&'static str> {
    let line = match region_type {
        "coast" => {
            "Word on the shore of {}: the Mëräk are surfacing at the tideline this \
                    season — cloth and tools for deep-fish and deep-glass, never coin."
        }
        "steppe" => {
            "They say the Khör have driven the härkä-herds through {} — metal buys \
                     leather and steppe-butter at their cairns, and they do not haggle."
        }
        "forest" => {
            "Travelers out of {} swear the Häl have come down from the canopy to \
                     trade — salve, herbs, and fruit for good cloth and tools."
        }
        "upland" => {
            "Miners from {} tell of the Tzäkhar at the cave-mouths, trading worked \
                     iron and fine tools for plain food — patient folk, no use for coin."
        }
        _ => return None,
    };
    let h = seed.wrapping_mul(2_654_435_761)
        ^ crate::rng::mix_u64(
            region_id
                .bytes()
                .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64))
                ^ (season_ord as u64).wrapping_shl(40)
                ^ (year as u64).wrapping_shl(52),
        );
    if h.is_multiple_of(3) {
        Some(line)
    } else {
        None
    }
}

/// A rumor grounded in the current world state, deterministic per salt.
/// Returns None when the world has nothing worth repeating.
pub fn informed_rumor(sim: &SimState, day: u32, salt: u64) -> Option<String> {
    let mut candidates: Vec<String> = Vec::new();
    // A gifted crafter is a rare highlight (#441), not a census line. With every
    // soul now a real resident (entity-first epic), nearly every town holds one,
    // and surfacing them all floods the rumor pool and crowds out everything else
    // (the non-human trades, the wars). Carry at most one province-wide per call.
    let mut gift_rumored = false;

    for region in &sim.world.regions {
        for s in &region.settlements {
            let per_head = s.food_stock / s.population.max(1) as f64;
            if per_head < 0.5 {
                candidates.push(format!(
                    "They say the stores run empty in {} — bread is dear there.",
                    s.name
                ));
            } else if per_head > 4.0 {
                candidates.push(format!(
                    "The granaries overflow in {}; food sells for a song.",
                    s.name
                ));
            }
            if s.in_festival(day) {
                candidates.push(format!(
                    "There's a festival in {} — cheap beds and open doors.",
                    s.name
                ));
            }
            if let Some(b) = s.buildings.iter().find(|b| !b.is_complete()) {
                candidates.push(format!(
                    "They're raising a {} in {}.",
                    b.building_type.name(),
                    s.name
                ));
            }
            // A real gifted crafter in town, surfaced as word on the road
            // (#431/#441) — an actual person who carries the gift, not a reroll.
            if !gift_rumored {
                if let Some(p) = s.people.iter().find(|p| p.gift.has()) {
                    if let Some(sense) = p.gift.sense() {
                        candidates.push(sense.npc_rumor(&p.name, &s.name));
                        gift_rumored = true;
                    }
                }
            }
        }
        // The non-human peoples trading in this land this season (#447).
        if let Some(tmpl) = nonhuman_trade_rumor(
            &region.region_type,
            sim.world.seed,
            &region.id,
            (day / 30) % 4,
            day / 120,
        ) {
            candidates.push(tmpl.replace("{}", &region.name));
        }
    }
    let tick = sim.world.tick;
    for c in &sim.caravans {
        if c.is_in_transit(tick) {
            candidates.push(format!(
                "A caravan is on the road to {}, heavy with goods.",
                c.destination
            ));
        }
    }

    // The province's polity and its rival at open tension: war on the wind,
    // roads watched and closed, the levy raised (#415). Deterministic.
    let polity = sim.world.polity;
    let season_ord = (day / 30) % 4;
    let year = day / 120;
    if polity.in_tension(sim.world.seed, season_ord, year) {
        let rival = polity.rival();
        candidates.push(format!(
            "They say {} and {} are at it again — the roads east are watched, some closed.",
            polity.name(),
            rival.name()
        ));
        candidates.push(format!(
            "Word is {} has raised the levy for the war. Hard season to keep a hearth.",
            polity.name()
        ));
    }

    // The season's world-event, if any, is on every tongue (#417).
    if let Some(event) = crate::model::WorldEvent::current(
        sim.world.seed,
        crate::model::Season::from_day(day),
        day / 90,
    ) {
        candidates.push(event.rumor().to_string());
    }

    if candidates.is_empty() {
        return None;
    }
    let idx = (salt.wrapping_mul(2_654_435_761) >> 16) as usize % candidates.len();
    Some(candidates.swap_remove(idx))
}
