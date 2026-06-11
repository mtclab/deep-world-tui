use super::*;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Animal {
    Dog,
    Hound,
    Horse,
    Donkey,
    Ox,
    Falcon,
    Crow,
    Goat,
    CaveLurker,
    RiverEel,
    RockLizard,
    ForestOwl,
    TundraFox,
    DesertCaravanDog,
    HighlandGoat,
    MarshCrane,
    /// Canon herd-beast of the Metsik border-clans and the Porokansa.
    Reindeer,
}

impl Animal {
    pub fn glyph(self) -> char {
        match self {
            Animal::Dog => 'd',
            Animal::Hound => 'h',
            Animal::Horse => 'H',
            Animal::Donkey => 'D',
            Animal::Ox => 'O',
            Animal::Falcon => 'f',
            Animal::Crow => 'c',
            Animal::Goat => 'g',
            Animal::CaveLurker => 'L',
            Animal::RiverEel => 'e',
            Animal::RockLizard => 'l',
            Animal::ForestOwl => 'W',
            Animal::TundraFox => 'F',
            Animal::DesertCaravanDog => 'C',
            Animal::HighlandGoat => 'G',
            Animal::MarshCrane => 'M',
            Animal::Reindeer => 'R',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Animal::Dog => "dog",
            Animal::Hound => "hound",
            Animal::Horse => "horse",
            Animal::Donkey => "donkey",
            Animal::Ox => "ox",
            Animal::Falcon => "falcon",
            Animal::Crow => "crow",
            Animal::Goat => "goat",
            Animal::CaveLurker => "cave_lurker",
            Animal::RiverEel => "river_eel",
            Animal::RockLizard => "rock_lizard",
            Animal::ForestOwl => "forest_owl",
            Animal::TundraFox => "tundra_fox",
            Animal::DesertCaravanDog => "desert_caravan_dog",
            Animal::HighlandGoat => "highland_goat",
            Animal::MarshCrane => "marsh_crane",
            Animal::Reindeer => "reindeer",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Animal::Dog => "loyal guardian and keen-nosed gatherer",
            Animal::Hound => "faithful hound, warns of danger",
            Animal::Horse => "swift mount for distant roads",
            Animal::Donkey => "sturdy carrier, lightens the load",
            Animal::Ox => "strong back for heavy loads",
            Animal::Falcon => "sharp-eyed scout from the sky",
            Animal::Crow => "bird of the road, bearer of rumors",
            Animal::Goat => "patient provider of milk",
            Animal::CaveLurker => "silent shadow that knows the deep paths",
            Animal::RiverEel => "slippery hunter of shallows and weirs",
            Animal::RockLizard => "cold-blooded climber of crags and ruins",
            Animal::ForestOwl => "watchful spirit of the midnight canopy",
            Animal::TundraFox => "white-furred ghost of the northern wastes",
            Animal::DesertCaravanDog => "hardy trail-runner bred for sand and heat",
            Animal::HighlandGoat => "sure-footed provider for the high places",
            Animal::MarshCrane => "tall wader that knows the safe passages",
            Animal::Reindeer => {
                "A steady-eyed herd beast; milk, carry-strength, and a back that knows the snow."
            }
        }
    }

    pub fn gathering_bonus(self) -> f64 {
        match self {
            Animal::Dog | Animal::Hound => 0.15,
            _ => 0.0,
        }
    }

    pub fn travel_speed_multiplier(self) -> f64 {
        match self {
            Animal::Horse => 0.7,
            _ => 1.0,
        }
    }

    pub fn carry_capacity_bonus(self) -> u32 {
        match self {
            Animal::Ox => 10,
            Animal::Donkey => 8,
            Animal::Reindeer => 6,
            _ => 0,
        }
    }

    pub fn scouting_bonus(self) -> f64 {
        match self {
            Animal::Falcon | Animal::Crow => 0.2,
            _ => 0.0,
        }
    }

    pub fn milk_production(self) -> u32 {
        match self {
            Animal::Goat => 1,
            Animal::HighlandGoat => 1,
            Animal::Reindeer => 1,
            _ => 0,
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            Animal::Dog => 8,
            Animal::Hound => 10,
            Animal::Horse => 25,
            Animal::Donkey => 12,
            Animal::Ox => 15,
            Animal::Falcon => 12,
            Animal::Crow => 5,
            Animal::Goat => 6,
            Animal::CaveLurker => 18,
            Animal::RiverEel => 4,
            Animal::RockLizard => 7,
            Animal::ForestOwl => 10,
            Animal::TundraFox => 14,
            Animal::DesertCaravanDog => 9,
            Animal::HighlandGoat => 8,
            Animal::MarshCrane => 5,
            Animal::Reindeer => 13,
        }
    }

    pub fn food_per_tick(self) -> u32 {
        match self {
            Animal::Dog | Animal::Hound | Animal::Crow => 1,
            Animal::Horse | Animal::Ox | Animal::Donkey => 2,
            Animal::Falcon => 1,
            Animal::Goat => 1,
            Animal::CaveLurker => 1,
            Animal::RiverEel => 0,
            Animal::RockLizard => 0,
            Animal::ForestOwl => 1,
            Animal::TundraFox => 1,
            Animal::DesertCaravanDog => 1,
            Animal::HighlandGoat => 1,
            Animal::MarshCrane => 0,
            Animal::Reindeer => 1,
        }
    }

    pub fn rest_per_tick(self) -> u32 {
        match self {
            Animal::Dog | Animal::Hound | Animal::Crow => 1,
            Animal::Horse | Animal::Ox | Animal::Donkey => 2,
            Animal::Falcon => 1,
            Animal::Goat => 1,
            Animal::CaveLurker => 1,
            Animal::RiverEel => 0,
            Animal::RockLizard => 1,
            Animal::ForestOwl => 1,
            Animal::TundraFox => 1,
            Animal::DesertCaravanDog => 1,
            Animal::HighlandGoat => 1,
            Animal::MarshCrane => 0,
            Animal::Reindeer => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompanionMood {
    Content,
    Restless,
    Unhappy,
}

impl CompanionMood {
    pub fn label(self) -> &'static str {
        match self {
            CompanionMood::Content => "content",
            CompanionMood::Restless => "restless",
            CompanionMood::Unhappy => "unhappy",
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            CompanionMood::Content => "Your companion pads along quietly, at ease.",
            CompanionMood::Restless => {
                "Your companion paces, sniffing the wind. Something is on their mind."
            }
            CompanionMood::Unhappy => "Your companion hangs back, refusing to meet your eyes.",
        }
    }

    pub fn encounter_bonus(self) -> f64 {
        match self {
            CompanionMood::Content => 0.05,
            CompanionMood::Restless => 0.0,
            CompanionMood::Unhappy => -0.10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompanionAction {
    Hunt,
    Gather,
    Scout,
}

impl CompanionAction {
    pub fn label(self) -> &'static str {
        match self {
            CompanionAction::Hunt => "hunted",
            CompanionAction::Gather => "gathered",
            CompanionAction::Scout => "scouted",
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            CompanionAction::Hunt => "returned from the hunt with game in their jaws.",
            CompanionAction::Gather => "came back carrying something useful in their mouth.",
            CompanionAction::Scout => {
                "circled back after checking the path ahead. The way seems clear."
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Companion {
    pub animal: Animal,
    pub name: String,
    pub food_need: f64,
    pub rest_need: f64,
    pub acquired_tick: u64,
    pub loyalty: f64,
}

impl Companion {
    pub fn new(animal: Animal, name: String, acquired_tick: u64) -> Self {
        Companion {
            animal,
            name,
            food_need: 0.0,
            rest_need: 0.0,
            acquired_tick,
            loyalty: 0.5,
        }
    }

    pub fn mood(&self) -> CompanionMood {
        if self.food_need >= 70.0 || self.rest_need >= 70.0 || self.loyalty < 0.2 {
            CompanionMood::Unhappy
        } else if self.food_need >= 40.0 || self.rest_need >= 40.0 {
            CompanionMood::Restless
        } else {
            CompanionMood::Content
        }
    }

    pub fn decay_needs(&mut self, ticks: u64) {
        // Per-animal upkeep: a horse eats twice what a dog does, and a river
        // eel asks for nothing. These rates existed on Animal but decay used
        // flat constants, so every companion cost the same to keep.
        let food_rate = 0.25 * self.animal.food_per_tick() as f64;
        let rest_rate = 0.15 * self.animal.rest_per_tick() as f64;
        self.food_need = (self.food_need + ticks as f64 * food_rate).min(100.0);
        self.rest_need = (self.rest_need + ticks as f64 * rest_rate).min(100.0);
    }

    pub fn feed(&mut self, amount: f64) {
        self.food_need = (self.food_need - amount * 20.0).max(0.0);
        self.loyalty = (self.loyalty + 0.03).min(1.0);
    }

    pub fn rest(&mut self, amount: f64) {
        self.rest_need = (self.rest_need - amount * 25.0).max(0.0);
    }

    pub fn is_starving(&self) -> bool {
        self.food_need >= 80.0
    }

    pub fn is_exhausted(&self) -> bool {
        self.rest_need >= 80.0
    }

    pub fn is_alive(&self) -> bool {
        !(self.food_need >= 100.0 || self.rest_need >= 100.0)
    }

    pub fn autonomous_action(&self, seed: u64) -> Option<CompanionAction> {
        if self.mood() == CompanionMood::Unhappy {
            return None;
        }
        let val = (seed.wrapping_mul(2654435761) >> 32) as u32 % 100;
        if val < 15 {
            Some(CompanionAction::Hunt)
        } else if val < 30 {
            Some(CompanionAction::Gather)
        } else if val < 40 {
            Some(CompanionAction::Scout)
        } else {
            None
        }
    }

    pub fn apply_action(&mut self, action: CompanionAction) -> &'static str {
        match action {
            CompanionAction::Hunt => {
                self.food_need = (self.food_need - 30.0).max(0.0);
                self.loyalty = (self.loyalty + 0.02).min(1.0);
            }
            CompanionAction::Gather => {
                self.food_need = (self.food_need - 15.0).max(0.0);
            }
            CompanionAction::Scout => {
                self.rest_need = (self.rest_need + 10.0).min(100.0);
            }
        }
        action.flavor()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn animal_properties() {
        assert_eq!(Animal::Dog.name(), "dog");

        assert!(Animal::Dog.gathering_bonus() > 0.0);

        assert!(Animal::Horse.travel_speed_multiplier() < 1.0);

        assert!(Animal::Ox.carry_capacity_bonus() > 0);

        assert!(Animal::Falcon.scouting_bonus() > 0.0);

        assert!(Animal::Goat.milk_production() > 0);
    }

    #[test]

    fn animal_costs_and_needs() {
        assert!(Animal::Horse.cost() > Animal::Dog.cost());

        assert!(Animal::Dog.food_per_tick() > 0);

        assert!(Animal::Dog.rest_per_tick() > 0);
    }

    #[test]

    fn companion_creation() {
        let companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        assert_eq!(companion.animal, Animal::Dog);

        assert_eq!(companion.name, "Rex");

        assert_eq!(companion.acquired_tick, 100);

        assert_eq!(companion.food_need, 0.0);
    }

    #[test]

    fn companion_need_decay() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.decay_needs(10);

        assert!(companion.food_need > 0.0);

        assert!(companion.rest_need > 0.0);

        assert!(!companion.is_starving());
    }

    #[test]

    fn companion_feeding() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.food_need = 50.0;

        companion.feed(1.0);

        assert!(companion.food_need < 50.0);
    }

    #[test]

    fn companion_starvation_death() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.food_need = 100.0;

        assert!(!companion.is_alive());
    }

    #[test]

    fn companion_mood_content() {
        let companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        assert_eq!(companion.mood(), CompanionMood::Content);

        assert!(companion.mood().encounter_bonus() > 0.0);
    }

    #[test]

    fn companion_mood_restless() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.food_need = 45.0;

        assert_eq!(companion.mood(), CompanionMood::Restless);

        assert_eq!(companion.mood().encounter_bonus(), 0.0);
    }

    #[test]

    fn companion_mood_unhappy() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.food_need = 75.0;

        assert_eq!(companion.mood(), CompanionMood::Unhappy);

        assert!(companion.mood().encounter_bonus() < 0.0);
    }

    #[test]

    fn companion_mood_unhappy_low_loyalty() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.loyalty = 0.1;

        assert_eq!(companion.mood(), CompanionMood::Unhappy);
    }

    #[test]

    fn companion_autonomous_action_deterministic() {
        let companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        let a1 = companion.autonomous_action(42);

        let a2 = companion.autonomous_action(42);

        assert_eq!(a1, a2);
    }

    #[test]

    fn companion_autonomous_action_unhappy_none() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.food_need = 80.0;

        assert!(companion.autonomous_action(42).is_none());
    }

    #[test]

    fn companion_apply_hunt_action() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.food_need = 50.0;

        companion.apply_action(CompanionAction::Hunt);

        assert!(companion.food_need < 50.0);

        assert!(companion.loyalty > 0.5);
    }

    #[test]

    fn companion_apply_scout_action() {
        let mut companion = Companion::new(Animal::Dog, "Rex".into(), 100);

        companion.rest_need = 10.0;

        companion.apply_action(CompanionAction::Scout);

        assert!(companion.rest_need > 10.0);
    }
}
