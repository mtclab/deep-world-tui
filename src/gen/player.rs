use crate::charts::Charts;
use crate::model::{Adjustment, CraftAffinity, Inventory, ItemType, Player, PlayerStart};
use crate::rng::SeedRng;

pub fn generate_player_start(rng: &mut SeedRng, charts: &Charts) -> PlayerStart {
    let person = crate::gen::person::generate_person(rng, charts);
    let mut inventory = Inventory::default();
    inventory.add(ItemType::Food, 3);
    inventory.add(ItemType::Coin, 5);
    PlayerStart {
        person,
        reroll_count: 0,
        point_buy_adjustments: vec![],
        accepted: false,
        inventory,
    }
}

impl PlayerStart {
    pub fn reroll(&mut self, rng: &mut SeedRng, charts: &Charts) {
        self.person = crate::gen::person::generate_person(rng, charts);
        self.reroll_count += 1;
    }

    pub fn apply_adjustment(&mut self, adj: Adjustment) {
        match adj {
            Adjustment::SwapProfession(p) => {
                self.person.profession = p;
            }
            Adjustment::SetCraft(c) => {
                self.person.craft_affinity = c.to_chart_key().to_string();
            }
            Adjustment::AddPerk(_) => {
                self.point_buy_adjustments.push(adj);
            }
            Adjustment::CutHouseholdTie => {
                self.person.has_spouse = false;
                self.person.children_count = 0;
                self.point_buy_adjustments.push(Adjustment::CutHouseholdTie);
            }
        }
    }
}

#[allow(dead_code)]
fn player_from_person(person: &crate::model::Person) -> Player {
    Player {
        id: person.id.clone(),
        name: person.name.clone(),
        people: person.people.clone(),
        sex: person.sex.clone(),
        age_band: person.age_band.clone(),
        profession: person.profession.clone(),
        social_class: person.social_class.clone(),
        craft_affinity: CraftAffinity::from_chart_key(&person.craft_affinity).unwrap_or_default(),
        personality: person.personality.clone(),
        region: person.region.clone(),
        settlement: person.settlement.clone(),
        perks: vec![],
        household_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts;

    #[test]
    fn generate_player_start_populated() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(42);
        let ps = generate_player_start(&mut rng, &charts);
        assert!(!ps.person.id.is_empty());
        assert!(!ps.person.name.is_empty());
        assert!(!ps.person.people.is_empty());
        assert!(!ps.person.profession.is_empty());
        assert_eq!(ps.reroll_count, 0);
        assert!(ps.point_buy_adjustments.is_empty());
        assert!(!ps.accepted);
    }

    #[test]
    fn reroll_changes_person() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(42);
        let mut ps = generate_player_start(&mut rng, &charts);
        let _original_name = ps.person.name.clone();
        let original_id = ps.person.id.clone();
        ps.reroll(&mut rng, &charts);
        assert_eq!(ps.reroll_count, 1);
        assert_ne!(ps.person.id, original_id, "id should change after reroll");
    }

    #[test]
    fn reroll_deterministic() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut a = SeedRng::new(42);
        let mut b = SeedRng::new(42);
        let mut psa = generate_player_start(&mut a, &charts);
        let mut psb = generate_player_start(&mut b, &charts);
        assert_eq!(psa.person.id, psb.person.id);
        psa.reroll(&mut a, &charts);
        psb.reroll(&mut b, &charts);
        assert_eq!(psa.person.id, psb.person.id);
    }

    #[test]
    fn apply_swap_profession() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(42);
        let mut ps = generate_player_start(&mut rng, &charts);
        ps.apply_adjustment(Adjustment::SwapProfession("trader".into()));
        assert_eq!(ps.person.profession, "trader");
    }

    #[test]
    fn apply_set_craft() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(42);
        let mut ps = generate_player_start(&mut rng, &charts);
        ps.apply_adjustment(Adjustment::SetCraft(CraftAffinity::Current));
        assert_eq!(ps.person.craft_affinity, "current");
    }

    #[test]
    fn apply_cut_household_tie() {
        let charts = charts::load_charts("data/charts.ron").unwrap();
        let mut rng = SeedRng::new(42);
        let mut ps = generate_player_start(&mut rng, &charts);
        ps.person.has_spouse = true;
        ps.person.children_count = 3;
        ps.apply_adjustment(Adjustment::CutHouseholdTie);
        assert!(!ps.person.has_spouse);
        assert_eq!(ps.person.children_count, 0);
    }
}
