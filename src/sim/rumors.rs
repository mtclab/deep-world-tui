//! Rumors that carry real information. The tavern channel used to deal only
//! in generic flavor lines; now it reports the actual state of the world —
//! famine and plenty (tradable!), caravans on the road, festivals, buildings
//! going up — so an attentive player can act on what they overhear.
use crate::sim::SimState;

/// A rumor grounded in the current world state, deterministic per salt.
/// Returns None when the world has nothing worth repeating.
pub fn informed_rumor(sim: &SimState, day: u32, salt: u64) -> Option<String> {
    let mut candidates: Vec<String> = Vec::new();

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
            // A rare gifted crafter, surfaced as word on the road (#431).
            // Deterministic per settlement; the gift is as rare here as anywhere.
            let g = crate::model::Gift::roll(sim.world.seed, crate::rng::fnv1a_hash(&s.id));
            if let Some(sense) = g.sense() {
                if let Some(p) = s.people.iter().find(|p| {
                    matches!(
                        p.profession.as_str(),
                        "smith" | "herbalist" | "healer" | "weaver" | "trader"
                    )
                }) {
                    candidates.push(sense.npc_rumor(&p.name, &s.name));
                }
            }
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
