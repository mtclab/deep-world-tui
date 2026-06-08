use crate::charts::Charts;
use crate::model::{Needs, NpcActivity, NpcSchedule, Person};
use crate::rng::SeedRng;

pub fn generate_schedule(profession: &str) -> NpcSchedule {
    let blocks = match profession.to_lowercase().as_str() {
        "priest" | "acolyte" | "monk" => [
            NpcActivity::Sleep,
            NpcActivity::Worship,
            NpcActivity::Work,
            NpcActivity::Socialize,
            NpcActivity::Worship,
            NpcActivity::Sleep,
        ],
        "smith" | "blacksmith" | "weaponsmith" | "armorsmith" => [
            NpcActivity::Sleep,
            NpcActivity::Craft,
            NpcActivity::Craft,
            NpcActivity::Socialize,
            NpcActivity::Idle,
            NpcActivity::Sleep,
        ],
        "merchant" | "trader" | "shopkeeper" => [
            NpcActivity::Sleep,
            NpcActivity::Work,
            NpcActivity::Work,
            NpcActivity::Travel,
            NpcActivity::Socialize,
            NpcActivity::Sleep,
        ],
        "farmer" | "herder" | "shepherd" => [
            NpcActivity::Sleep,
            NpcActivity::Work,
            NpcActivity::Work,
            NpcActivity::Idle,
            NpcActivity::Socialize,
            NpcActivity::Sleep,
        ],
        "scholar" | "scribe" | "librarian" => [
            NpcActivity::Sleep,
            NpcActivity::Work,
            NpcActivity::Work,
            NpcActivity::Socialize,
            NpcActivity::Work,
            NpcActivity::Sleep,
        ],
        _ => NpcSchedule::default().blocks,
    };
    NpcSchedule { blocks }
}

pub fn generate_person(rng: &mut SeedRng, charts: &Charts) -> Person {
    let mut person_rng = rng.fork();
    let sub_seed = person_rng.next_u64();

    let people = charts.people.sample(&mut person_rng).unwrap_or_default();
    let region = charts.region.sample(&mut person_rng).unwrap_or_default();
    let settlement_size = charts
        .settlement_size
        .resolve_and_sample(&people, &region, "", "", &mut person_rng)
        .unwrap_or_default();
    let social_class = charts
        .social_class
        .sample(&mut person_rng)
        .unwrap_or_default();

    let profession = charts
        .profession
        .resolve_and_sample(
            &people,
            &region,
            &social_class,
            &settlement_size,
            &mut person_rng,
        )
        .unwrap_or_default();

    let craft_affinity = charts
        .craft_affinity
        .resolve_and_sample(
            &people,
            &region,
            &social_class,
            &settlement_size,
            &mut person_rng,
        )
        .unwrap_or_default();

    let mut personality = Vec::new();
    let n_traits = 2 + (person_rng.gen_range(2) as usize);
    for _ in 0..n_traits {
        if let Some(t) = charts.personality_traits.sample(&mut person_rng) {
            if !personality.contains(&t) {
                personality.push(t);
            }
        }
    }

    let age_band = charts.age_band.sample(&mut person_rng).unwrap_or_default();
    let sex = charts.sex.sample(&mut person_rng).unwrap_or_default();
    let has_spouse = charts
        .has_spouse
        .sample(&mut person_rng)
        .map(|v| v == "yes")
        .unwrap_or(false);
    let children_str = charts
        .children_count
        .sample(&mut person_rng)
        .unwrap_or_default();
    let children_count: u32 = children_str.parse().unwrap_or(0);
    let has_debt = charts
        .has_debt
        .sample(&mut person_rng)
        .map(|v| v == "yes")
        .unwrap_or(false);

    let name = crate::gen::name::generate_name(&mut person_rng, &people, &sex, charts)
        .unwrap_or_else(|_| "Unnamed".into());

    let schedule = generate_schedule(&profession);

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
        region,
        settlement: String::new(),
        has_spouse,
        children_count,
        has_debt,
        schedule,
        relations: vec![],
    }
}

pub fn generate_person_from(
    mut person_rng: SeedRng,
    region: &str,
    settlement: &str,
    charts: &Charts,
) -> Person {
    let sub_seed = person_rng.next_u64();

    let people = charts.people.sample(&mut person_rng).unwrap_or_default();
    let settlement_size = charts
        .settlement_size
        .resolve_and_sample(&people, region, "", "", &mut person_rng)
        .unwrap_or_default();
    let social_class = charts
        .social_class
        .sample(&mut person_rng)
        .unwrap_or_default();

    let profession = charts
        .profession
        .resolve_and_sample(
            &people,
            region,
            &social_class,
            &settlement_size,
            &mut person_rng,
        )
        .unwrap_or_default();

    let craft_affinity = charts
        .craft_affinity
        .resolve_and_sample(
            &people,
            region,
            &social_class,
            &settlement_size,
            &mut person_rng,
        )
        .unwrap_or_default();

    let mut personality = Vec::new();
    let n_traits = 2 + (person_rng.gen_range(2) as usize);
    for _ in 0..n_traits {
        if let Some(t) = charts.personality_traits.sample(&mut person_rng) {
            if !personality.contains(&t) {
                personality.push(t);
            }
        }
    }

    let age_band = charts.age_band.sample(&mut person_rng).unwrap_or_default();
    let sex = charts.sex.sample(&mut person_rng).unwrap_or_default();
    let has_spouse = charts
        .has_spouse
        .sample(&mut person_rng)
        .map(|v| v == "yes")
        .unwrap_or(false);
    let children_str = charts
        .children_count
        .sample(&mut person_rng)
        .unwrap_or_default();
    let children_count: u32 = children_str.parse().unwrap_or(0);
    let has_debt = charts
        .has_debt
        .sample(&mut person_rng)
        .map(|v| v == "yes")
        .unwrap_or(false);

    let name = crate::gen::name::generate_name(&mut person_rng, &people, &sex, charts)
        .unwrap_or_else(|_| "Unnamed".into());

    let schedule = generate_schedule(&profession);

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
        region: region.to_string(),
        settlement: settlement.to_string(),
        has_spouse,
        children_count,
        has_debt,
        schedule,
        relations: vec![],
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

    fn generate_n(n: usize, seed: u64) -> (Vec<Person>, crate::charts::Charts) {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(seed);
        let people: Vec<Person> = (0..n).map(|_| generate_person(&mut rng, &charts)).collect();
        (people, charts)
    }

    #[test]
    fn distribution_profession_caps() {
        let (people, _) = generate_n(10_000, 42);
        let mut prof_counts = std::collections::HashMap::new();
        for p in &people {
            *prof_counts.entry(p.profession.clone()).or_insert(0usize) += 1;
        }
        let total = people.len() as f64;
        for (prof, count) in &prof_counts {
            let pct = *count as f64 / total * 100.0;
            assert!(
                pct < 40.0,
                "profession '{}' at {:.1}% exceeds 40% cap",
                prof,
                pct
            );
        }
        let farmer_labourer =
            *prof_counts.get("farmer").unwrap_or(&0) + *prof_counts.get("labourer").unwrap_or(&0);
        assert!(
            farmer_labourer as f64 / total > 0.30,
            "farmer+labourer only {:.1}%, expect >30%",
            farmer_labourer as f64 / total * 100.0
        );
        let soldier = *prof_counts.get("soldier").unwrap_or(&0);
        assert!(
            soldier as f64 / total < 0.10,
            "soldier at {:.1}%, expect <10%",
            soldier as f64 / total * 100.0
        );
        let scribe = *prof_counts.get("scribe").unwrap_or(&0);
        assert!(
            scribe as f64 / total < 0.05,
            "scribe at {:.1}%, expect <5%",
            scribe as f64 / total * 100.0
        );
    }

    #[test]
    fn distribution_per_people_profession_shift() {
        let (people, _charts) = generate_n(10_000, 77);
        let mut sepat_prof = std::collections::HashMap::new();
        let mut other_prof = std::collections::HashMap::new();
        for p in &people {
            if p.people == "sepat" {
                *sepat_prof.entry(p.profession.clone()).or_insert(0usize) += 1;
            } else {
                *other_prof.entry(p.profession.clone()).or_insert(0usize) += 1;
            }
        }
        let sepat_total = sepat_prof.values().sum::<usize>() as f64;
        let other_total = other_prof.values().sum::<usize>() as f64;
        let sepat_smith = *sepat_prof.get("smith").unwrap_or(&0) as f64 / sepat_total;
        let other_smith = *other_prof.get("smith").unwrap_or(&0) as f64 / other_total;
        assert!(
            sepat_smith > other_smith * 2.0,
            "sepat smith rate ({:.3}) not 2x other rate ({:.3})",
            sepat_smith,
            other_smith
        );
        let sepat_miner = *sepat_prof.get("miner").unwrap_or(&0) as f64 / sepat_total;
        let other_miner = *other_prof.get("miner").unwrap_or(&0) as f64 / other_total;
        assert!(
            sepat_miner > other_miner * 2.0,
            "sepat miner rate ({:.3}) not 2x other rate ({:.3})",
            sepat_miner,
            other_miner
        );
    }

    #[test]
    fn distribution_all_fields_populated_no_empty() {
        let (people, _) = generate_n(1_000, 42);
        for p in &people {
            assert!(!p.id.is_empty(), "empty id");
            assert!(!p.name.is_empty(), "empty name");
            assert!(!p.people.is_empty(), "empty people");
            assert!(!p.region.is_empty(), "empty region");
            assert!(!p.sex.is_empty(), "empty sex");
            assert!(!p.profession.is_empty(), "empty profession");
            assert!(!p.social_class.is_empty(), "empty social_class");
            assert!(!p.craft_affinity.is_empty(), "empty craft_affinity");
        }
    }
}
