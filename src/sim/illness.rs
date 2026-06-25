use crate::model::{ActiveDisease, Disease, Need, Needs, Terrain};
use crate::rng::SeedRng;

const ILLNESS_HEALTH_THRESHOLD: f64 = 0.5;
const ILLNESS_LOW_HEALTH_PROB: f64 = 0.003;
const BASE_CONTRACTION_PROB: f64 = 0.0005;
const SHELTER_THRESHOLD: f64 = 0.3;
const SHELTER_ILLNESS_MULTIPLIER: f64 = 1.5;
const HEALER_MISSING_MULTIPLIER: f64 = 1.5;
const MAX_ILLNESSES_PER_PERSON: usize = 2;

pub fn tick_illness(
    person_seed: u64,
    tick: u64,
    terrain: Terrain,
    needs: &Needs,
    _low_health_ticks: u64,
    has_healer: bool,
    existing_count: usize,
) -> Option<ActiveDisease> {
    tick_illness_luck(
        person_seed,
        tick,
        terrain,
        needs,
        has_healer,
        existing_count,
        1.0,
    )
}

/// As `tick_illness`, but `luck_mult` leans the contraction odds (a life's
/// hidden star: < 1 for the blessed, > 1 for the cursed). NPCs pass 1.0; the
/// player passes their fortune's bad-outcome multiplier — so the unlucky
/// sicken sooner in the very same swamp.
#[allow(clippy::too_many_arguments)]
pub fn tick_illness_luck(
    person_seed: u64,
    tick: u64,
    terrain: Terrain,
    needs: &Needs,
    has_healer: bool,
    existing_count: usize,
    luck_mult: f64,
) -> Option<ActiveDisease> {
    if existing_count >= MAX_ILLNESSES_PER_PERSON {
        return None;
    }
    let mut rng = SeedRng::new(person_seed).fork_for(&format!("illness-{tick}"));
    let mut prob = BASE_CONTRACTION_PROB;
    let food = needs.get(Need::Food);
    let safety = needs.get(Need::Safety);
    if food < ILLNESS_HEALTH_THRESHOLD {
        prob += ILLNESS_LOW_HEALTH_PROB * (1.0 - food);
    }
    if safety < SHELTER_THRESHOLD {
        prob *= SHELTER_ILLNESS_MULTIPLIER;
    }
    if !has_healer {
        prob *= HEALER_MISSING_MULTIPLIER;
    }
    prob += Disease::Fever.contraction_probability(terrain) / 200.0;
    prob *= luck_mult.max(0.0);
    let roll = rng.gen_range(10000) as f64 / 10000.0;
    if roll >= prob {
        return None;
    }
    let disease = pick_disease(&mut rng, terrain);
    Some(ActiveDisease::new(disease, tick))
}

fn pick_disease(rng: &mut SeedRng, terrain: Terrain) -> Disease {
    let weights: Vec<(Disease, f64)> = vec![
        (
            Disease::Fever,
            Disease::Fever.contraction_probability(terrain),
        ),
        (
            Disease::Infection,
            Disease::Infection.contraction_probability(terrain),
        ),
        (
            Disease::Sprain,
            Disease::Sprain.contraction_probability(terrain),
        ),
        (
            Disease::Exhaustion,
            Disease::Exhaustion.contraction_probability(terrain),
        ),
        (
            Disease::Plague,
            Disease::Plague.contraction_probability(terrain),
        ),
        (
            Disease::WinterCough,
            Disease::WinterCough.contraction_probability(terrain),
        ),
        (
            Disease::MarshFever,
            Disease::MarshFever.contraction_probability(terrain),
        ),
        (
            Disease::BloodAche,
            Disease::BloodAche.contraction_probability(terrain),
        ),
        (
            Disease::ForgeBlindness,
            Disease::ForgeBlindness.contraction_probability(terrain),
        ),
        (
            Disease::ChildbirthComplication,
            Disease::ChildbirthComplication.contraction_probability(terrain),
        ),
    ];
    let total: f64 = weights.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_range(10000) as f64 / 10000.0 * total;
    for (disease, w) in &weights {
        roll -= *w;
        if roll <= 0.0 {
            return *disease;
        }
    }
    Disease::Fever
}

/// Childbirth complications only afflict those who can actually give birth —
/// the disease was previously contractible by anyone (a flavor break).
pub fn can_contract_childbirth(sex: &str, age_band: &str) -> bool {
    sex.eq_ignore_ascii_case("f") && matches!(age_band, "youth" | "young" | "adult")
}

pub fn apply_illness_effects(person: &mut crate::model::Person, current_tick: u64) {
    person.illnesses.retain(|d| !d.is_recovered(current_tick));
}

pub fn illness_productivity_modifier(person: &crate::model::Person) -> f64 {
    if person.illnesses.is_empty() {
        return 1.0;
    }
    0.7
}

pub fn settlement_has_healer(settlement: &crate::model::Settlement) -> bool {
    settlement
        .services
        .iter()
        .any(|s| matches!(s, crate::model::SettlementService::Temple))
        || settlement
            .people
            .iter()
            .any(|p| p.profession == "healer" || p.profession == "herbalist")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Need, Needs};

    fn test_needs(food: f64, safety: f64) -> Needs {
        // Only food and safety matter to illness; the rest stay at 0.0 as the
        // old map-with-absent-keys did (get returned 0.0 for the unset needs).
        let mut n = Needs { values: [0.0; 5] };
        n.set(Need::Food, food);
        n.set(Need::Safety, safety);
        n
    }

    #[test]
    fn healthy_person_rarely_gets_ill() {
        let needs = test_needs(0.8, 0.8);
        let mut ill_count = 0;
        for tick in 0..200u64 {
            if tick_illness(42, tick, Terrain::Swamp, &needs, 0, true, 0).is_some() {
                ill_count += 1;
            }
        }
        assert!(
            ill_count <= 5,
            "healthy person got ill {} times in 200 ticks",
            ill_count
        );
    }

    #[test]
    fn low_health_increases_illness() {
        let needs = test_needs(0.2, 0.5);
        let mut ill_count = 0;
        for tick in 0..200u64 {
            if tick_illness(42, tick, Terrain::Swamp, &needs, 35, false, 0).is_some() {
                ill_count += 1;
            }
        }
        assert!(
            ill_count >= 1,
            "low health + no healer: expected some illness, got {}",
            ill_count
        );
    }

    #[test]
    fn no_healer_increases_rate() {
        let needs = test_needs(0.4, 0.5);
        let mut with_healer = 0;
        let mut without_healer = 0;
        for tick in 0..200u64 {
            if tick_illness(42, tick, Terrain::Forest, &needs, 0, true, 0).is_some() {
                with_healer += 1;
            }
            if tick_illness(42, tick, Terrain::Forest, &needs, 0, false, 0).is_some() {
                without_healer += 1;
            }
        }
        assert!(without_healer >= with_healer);
    }

    #[test]
    fn max_two_illnesses() {
        let needs = test_needs(0.1, 0.1);
        let result = tick_illness(42, 100, Terrain::Swamp, &needs, 40, false, 2);
        assert!(result.is_none());
    }

    #[test]
    fn illness_deterministic() {
        let needs = test_needs(0.2, 0.3);
        let r1 = tick_illness(99, 50, Terrain::Swamp, &needs, 35, false, 0);
        let r2 = tick_illness(99, 50, Terrain::Swamp, &needs, 35, false, 0);
        assert_eq!(r1.map(|d| d.disease), r2.map(|d| d.disease));
    }

    #[test]
    fn productivity_modifier_no_illness() {
        let person = crate::model::Person {
            id: "p1".into(),
            name: "Test".into(),
            people: "metsik".into(),
            sex: "f".into(),
            age_band: "adult".into(),
            profession: "farmer".into(),
            social_class: "low".into(),
            craft_affinity: "none".into(),
            personality: vec![],
            bias: "neutral".into(),
            needs: Needs::default(),
            region: "r".into(),
            settlement: "s".into(),
            has_spouse: false,
            children_count: 0,
            has_debt: false,
            coins: 0,
            schedule: crate::model::NpcSchedule::default(),
            illnesses: vec![],
            relations: vec![],
            wants: vec![],
            gift: Default::default(),
            aspiration: None,
            crimes: 0,
            age_years: 0,
        };
        assert_eq!(illness_productivity_modifier(&person), 1.0);
    }
}
