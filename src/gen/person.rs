use crate::charts::Charts;
use crate::model::{Needs, Person};
use crate::rng::SeedRng;

pub fn generate_person(seed_rng: &mut SeedRng, charts: &Charts) -> Person {
    let mut person_rng = seed_rng.fork_for("person");
    let sub_seed = person_rng.next_u64();
    let people_rng = person_rng.fork_for("people");
    let region_rng = person_rng.fork_for("region");
    let settlement_rng = person_rng.fork_for("settlement");
    let class_rng = person_rng.fork_for("class");
    let profession_rng = person_rng.fork_for("profession");
    let craft_rng = person_rng.fork_for("craft");
    let personality_rng = person_rng.fork_for("personality");
    let age_rng = person_rng.fork_for("age");
    let sex_rng = person_rng.fork_for("sex");
    let spouse_rng = person_rng.fork_for("spouse");
    let children_rng = person_rng.fork_for("children");
    let debt_rng = person_rng.fork_for("debt");
    let mut name_rng = person_rng.fork_for("name");

    let mut people_rng = people_rng;
    let people = charts.people.sample(&mut people_rng).unwrap_or_default();

    let mut region_rng = region_rng;
    let region = charts.region.sample(&mut region_rng).unwrap_or_default();

    let mut settlement_rng = settlement_rng;
    let settlement_size = charts
        .settlement_size
        .sample(&mut settlement_rng)
        .unwrap_or_default();

    let mut class_rng = class_rng;
    let social_class = charts
        .social_class
        .sample(&mut class_rng)
        .unwrap_or_default();

    let mut profession_rng = profession_rng;
    let profession = charts
        .profession
        .resolve_and_sample(
            &people,
            &region,
            &social_class,
            &settlement_size,
            &mut profession_rng,
        )
        .unwrap_or_default();

    let mut craft_rng = craft_rng;
    let craft_affinity = charts
        .craft_affinity
        .resolve_and_sample(
            &people,
            &region,
            &social_class,
            &settlement_size,
            &mut craft_rng,
        )
        .unwrap_or_default();

    let mut personality_rng = personality_rng;
    let mut personality = Vec::new();
    let n_traits = 2 + (personality_rng.gen_range(2) as usize);
    for _ in 0..n_traits {
        if let Some(t) = charts.personality_traits.sample(&mut personality_rng) {
            if !personality.contains(&t) {
                personality.push(t);
            }
        }
    }

    let mut age_rng = age_rng;
    let age_band = charts.age_band.sample(&mut age_rng).unwrap_or_default();

    let mut sex_rng = sex_rng;
    let sex = charts.sex.sample(&mut sex_rng).unwrap_or_default();

    let mut spouse_rng = spouse_rng;
    let has_spouse = charts
        .has_spouse
        .sample(&mut spouse_rng)
        .map(|v| v == "yes")
        .unwrap_or(false);

    let mut children_rng = children_rng;
    let children_str = charts
        .children_count
        .sample(&mut children_rng)
        .unwrap_or_default();
    let children_count: u32 = children_str.parse().unwrap_or(0);

    let mut debt_rng = debt_rng;
    let has_debt = charts
        .has_debt
        .sample(&mut debt_rng)
        .map(|v| v == "yes")
        .unwrap_or(false);

    let name = crate::gen::name::generate_name(&mut name_rng, &people, &sex, charts)
        .unwrap_or_else(|_| "Unnamed".into());

    Person {
        id: format!("person-{:016x}", sub_seed),
        name,
        people,
        sex,
        age_band,
        profession,
        social_class,
        craft_affinity,
        personality,
        bias: String::new(),
        needs: Needs::default(),
        region: region.clone(),
        settlement: String::new(),
        has_spouse,
        children_count,
        has_debt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;

    #[test]
    fn generate_person_all_fields_populated() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(42);
        let p = generate_person(&mut rng, &charts);
        assert!(!p.id.is_empty(), "id empty");
        assert!(!p.name.is_empty(), "name empty");
        assert!(!p.people.is_empty(), "people empty");
        assert!(!p.region.is_empty(), "region empty");
        assert!(!p.sex.is_empty(), "sex empty");
        assert!(!p.age_band.is_empty(), "age_band empty");
        assert!(!p.profession.is_empty(), "profession empty");
        assert!(!p.social_class.is_empty(), "social_class empty");
        assert!(!p.craft_affinity.is_empty(), "craft_affinity empty");
        assert!(!p.personality.is_empty(), "personality empty");
    }

    #[test]
    fn generate_person_fields_reference_valid_chart_keys() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(42);
        let p = generate_person(&mut rng, &charts);
        assert!(
            charts.people.entries.contains_key(&p.people),
            "people '{}' not in chart",
            p.people
        );
        assert!(
            charts.region.entries.contains_key(&p.region),
            "region '{}' not in chart",
            p.region
        );
        assert!(
            charts.profession.base.entries.contains_key(&p.profession),
            "profession '{}' not in chart",
            p.profession
        );
        assert!(
            charts.social_class.entries.contains_key(&p.social_class),
            "social_class '{}' not in chart",
            p.social_class
        );
        assert!(
            charts
                .craft_affinity
                .base
                .entries
                .contains_key(&p.craft_affinity),
            "craft_affinity '{}' not in chart",
            p.craft_affinity
        );
    }

    #[test]
    fn generate_person_deterministic() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut a = SeedRng::new(42);
        let mut b = SeedRng::new(42);
        let pa = generate_person(&mut a, &charts);
        let pb = generate_person(&mut b, &charts);
        assert_eq!(pa.id, pb.id);
        assert_eq!(pa.name, pb.name);
        assert_eq!(pa.people, pb.people);
        assert_eq!(pa.profession, pb.profession);
        assert_eq!(pa.craft_affinity, pb.craft_affinity);
    }

    #[test]
    fn generate_person_no_thread_rng() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(123);
        let _ = generate_person(&mut rng, &charts);
    }

    #[test]
    fn generate_multiple_people_deterministic() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut a = SeedRng::new(99);
        let mut b = SeedRng::new(99);
        for _ in 0..10 {
            let pa = generate_person(&mut a, &charts);
            let pb = generate_person(&mut b, &charts);
            assert_eq!(pa.id, pb.id, "id mismatch");
            assert_eq!(pa.name, pb.name, "name mismatch");
        }
    }
}
