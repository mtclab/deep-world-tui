use super::*;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PlayerVitals {
    pub hunger: f64,
    pub thirst: f64,
    pub energy: f64,
}

impl Default for PlayerVitals {
    fn default() -> Self {
        PlayerVitals {
            hunger: 1.0,
            thirst: 1.0,
            energy: 1.0,
        }
    }
}

impl PlayerVitals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self, hours: u32, inventory: &mut Inventory, season: Season) {
        self.tick_with_illness(hours, inventory, season, 1.0);
    }

    /// Like `tick`, but scales hunger/thirst/energy decay by an illness
    /// multiplier (>= 1.0) — a sick traveller wears down faster.
    pub fn tick_with_illness(
        &mut self,
        hours: u32,
        inventory: &mut Inventory,
        season: Season,
        illness_mult: f64,
    ) {
        let m = season.need_decay_multiplier() * illness_mult.max(1.0);
        let hunger_rate = 0.05 * m;
        let thirst_rate = 0.06 * m;
        let energy_rate = 0.02 * m;
        for _ in 0..hours {
            self.hunger -= hunger_rate;
            self.thirst -= thirst_rate;
            self.energy -= energy_rate;
            if self.hunger <= 0.3 && inventory.remove(ItemType::Food, 1) {
                self.hunger = (self.hunger + 0.3).min(1.0);
            }
            if self.thirst <= 0.3 && inventory.remove(ItemType::Water, 1) {
                self.thirst = (self.thirst + 0.4).min(1.0);
            }
        }
        self.hunger = self.hunger.max(0.0);
        self.thirst = self.thirst.max(0.0);
        self.energy = self.energy.max(0.0);
    }

    /// Restore energy proportional to hours rested (≈+0.6 over a full 8h).
    pub fn rest(&mut self, hours: u32) {
        self.energy = (self.energy + 0.075 * hours as f64).min(1.0);
    }

    pub fn is_starving(self) -> bool {
        self.hunger <= 0.0
    }

    pub fn is_dehydrated(self) -> bool {
        self.thirst <= 0.0
    }

    pub fn is_exhausted(self) -> bool {
        self.energy <= 0.1
    }

    pub fn hunger_label(self) -> &'static str {
        if self.hunger >= 0.7 {
            "full"
        } else if self.hunger >= 0.4 {
            "hungry"
        } else if self.hunger > 0.0 {
            "starving"
        } else {
            "famished"
        }
    }

    pub fn thirst_label(self) -> &'static str {
        if self.thirst >= 0.7 {
            "quenched"
        } else if self.thirst >= 0.4 {
            "thirsty"
        } else if self.thirst > 0.0 {
            "parched"
        } else {
            "dehydrated"
        }
    }

    pub fn energy_label(self) -> &'static str {
        if self.energy >= 0.7 {
            "energized"
        } else if self.energy >= 0.4 {
            "tired"
        } else {
            "exhausted"
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
    >(
        value: &T,
    ) {
        let ser = ron::ser::to_string(value).unwrap();
        let de: T = ron::from_str(&ser).unwrap();
        assert_eq!(*value, de);
    }

    #[test]

    fn player_vitals_tick_hunger_decay() {
        let mut v = PlayerVitals::new();

        let mut inv = Inventory::default();

        v.tick(5, &mut inv, Season::Thaw);

        assert!(v.hunger < 1.0, "hunger should decrease");

        assert!(v.energy < 1.0, "energy should decrease");
    }

    #[test]

    fn player_vitals_auto_eat() {
        let mut v = PlayerVitals::new();

        v.hunger = 0.2;

        let mut inv = Inventory::default();

        inv.add(ItemType::Food, 3);

        v.tick(1, &mut inv, Season::Thaw);

        assert!(v.hunger > 0.2, "should auto-eat when hungry");

        assert_eq!(inv.get(ItemType::Food), 2);
    }

    #[test]

    fn player_vitals_rest_restores_energy() {
        let mut v = PlayerVitals::new();

        v.energy = 0.1;

        v.rest(8);

        assert!(v.energy > 0.5, "rest should restore energy");
    }

    #[test]

    fn player_vitals_labels() {
        let full = PlayerVitals {
            hunger: 0.8,

            thirst: 0.8,

            energy: 0.8,
        };

        assert_eq!(full.hunger_label(), "full");

        assert_eq!(full.thirst_label(), "quenched");

        assert_eq!(full.energy_label(), "energized");

        let hungry = PlayerVitals {
            hunger: 0.5,

            thirst: 0.5,

            energy: 0.5,
        };

        assert_eq!(hungry.hunger_label(), "hungry");

        assert_eq!(hungry.thirst_label(), "thirsty");

        assert_eq!(hungry.energy_label(), "tired");
    }

    #[test]

    fn player_vitals_auto_drink() {
        let mut v = PlayerVitals::new();

        v.thirst = 0.2;

        let mut inv = Inventory::default();

        inv.add(ItemType::Water, 3);

        v.tick(1, &mut inv, Season::Thaw);

        assert!(v.thirst > 0.2, "should auto-drink when thirsty");

        assert_eq!(inv.get(ItemType::Water), 2);
    }

    #[test]

    fn thirst_depletes_faster_than_hunger() {
        let mut v = PlayerVitals::new();

        let mut inv = Inventory::default();

        v.tick(10, &mut inv, Season::Thaw);

        assert!(
            v.thirst < v.hunger,
            "thirst should deplete faster than hunger"
        );
    }

    #[test]

    fn is_dehydrated() {
        let v = PlayerVitals {
            hunger: 1.0,

            thirst: 0.0,

            energy: 1.0,
        };

        assert!(v.is_dehydrated());

        let v_ok = PlayerVitals {
            hunger: 1.0,

            thirst: 0.5,

            energy: 1.0,
        };

        assert!(!v_ok.is_dehydrated());
    }

    #[test]

    fn thirst_labels() {
        let v = PlayerVitals {
            hunger: 1.0,

            thirst: 0.9,

            energy: 1.0,
        };

        assert_eq!(v.thirst_label(), "quenched");

        let v2 = PlayerVitals {
            hunger: 1.0,

            thirst: 0.5,

            energy: 1.0,
        };

        assert_eq!(v2.thirst_label(), "thirsty");

        let v3 = PlayerVitals {
            hunger: 1.0,

            thirst: 0.15,

            energy: 1.0,
        };

        assert_eq!(v3.thirst_label(), "parched");

        let v4 = PlayerVitals {
            hunger: 1.0,

            thirst: 0.0,

            energy: 1.0,
        };

        assert_eq!(v4.thirst_label(), "dehydrated");
    }

    #[test]

    fn player_vitals_serialization() {
        let v = PlayerVitals::new();

        roundtrip(&v);
    }

    #[test]

    fn exposure_when_thirst_zero() {
        let vitals = PlayerVitals {
            hunger: 0.5,

            thirst: 0.0,

            energy: 0.8,
        };

        let cause = DeathCause::from_collapse_and_vitals(CollapseOutcome::Ditch, vitals);

        assert_eq!(cause, DeathCause::Exposure);
    }
}
