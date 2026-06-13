//! The world calendar: seasons become *events*, not just multipliers (#417).
//! A season can carry a market fair, a hard winter, or a plague year — each
//! deterministic from the world seed + season + year, announced in rumor and
//! journal, visible in exactly one mechanic, and gone when the season turns.

use crate::model::weather::Season;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldEvent {
    /// A market fair: goods cheaper, the town livelier.
    MarketFair,
    /// A hard winter declared: the cold bites deeper and the stores run lean.
    HardWinter,
    /// A plague year: sickness takes more readily for the season.
    PlagueYear,
}

impl Season {
    fn ordinal(self) -> u32 {
        match self {
            Season::Thaw => 0,
            Season::Green => 1,
            Season::Frost => 2,
        }
    }
}

impl WorldEvent {
    /// The event (if any) gripping the world this season. Deterministic from
    /// seed + season + year, so a given world always runs the same calendar.
    /// At most one per season, and most seasons carry none.
    pub fn current(seed: u64, season: Season, year: u32) -> Option<WorldEvent> {
        let mut h = seed ^ crate::rng::fnv1a_hash("world-calendar");
        h = crate::rng::mix_u64(h ^ (((season.ordinal() as u64) << 32) | year as u64));
        let r = crate::rng::unit_from_hash(h);
        // A plague is rare and can fall in any season; otherwise a seasonal
        // event sometimes lands — a hard winter in the Frost, a fair in the
        // growing seasons. Most seasons (r >= 0.33) are ordinary.
        if r < 0.10 {
            Some(WorldEvent::PlagueYear)
        } else if r < 0.33 {
            match season {
                Season::Frost => Some(WorldEvent::HardWinter),
                Season::Thaw | Season::Green => Some(WorldEvent::MarketFair),
            }
        } else {
            None
        }
    }

    /// Multiplier on market buy prices. A fair makes goods cheaper; nothing
    /// else moves the stalls.
    pub fn buy_price_modifier(self) -> f64 {
        match self {
            WorldEvent::MarketFair => 0.80,
            _ => 1.0,
        }
    }

    /// Extra multiplier on the harsh-weather vitals penalty. A hard winter
    /// bites deeper than an ordinary Frost.
    pub fn weather_decay_modifier(self) -> f64 {
        match self {
            WorldEvent::HardWinter => 1.30,
            _ => 1.0,
        }
    }

    /// Multiplier on the chance of taking ill. A plague year raises it.
    pub fn illness_contraction_modifier(self) -> f64 {
        match self {
            WorldEvent::PlagueYear => 1.6,
            _ => 1.0,
        }
    }

    /// The rumor that announces the event on the wind.
    pub fn rumor(self) -> &'static str {
        match self {
            WorldEvent::MarketFair => {
                "There's a great fair this season — goods cheap, the roads full of carts."
            }
            WorldEvent::HardWinter => {
                "They're calling it a hard winter. Lay in what you can; the stores will run thin."
            }
            WorldEvent::PlagueYear => {
                "A sickness is going round this year. They say it takes the careless first."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_is_deterministic() {
        let seed = 9090u64;
        for (s, y) in [(Season::Thaw, 0), (Season::Frost, 3), (Season::Green, 7)] {
            assert_eq!(
                WorldEvent::current(seed, s, y),
                WorldEvent::current(seed, s, y)
            );
        }
    }

    #[test]
    fn hard_winter_only_in_frost_and_fair_only_in_growth() {
        // Over many years no season carries a mismatched seasonal event.
        for y in 0..400u32 {
            if WorldEvent::current(7, Season::Frost, y) == Some(WorldEvent::MarketFair) {
                panic!("a fair fell in the Frost");
            }
            for s in [Season::Thaw, Season::Green] {
                if WorldEvent::current(7, s, y) == Some(WorldEvent::HardWinter) {
                    panic!("a hard winter fell outside the Frost");
                }
            }
        }
    }

    #[test]
    fn events_are_occasional_not_constant() {
        let mut events = 0;
        for y in 0..300u32 {
            for s in [Season::Thaw, Season::Green, Season::Frost] {
                if WorldEvent::current(42, s, y).is_some() {
                    events += 1;
                }
            }
        }
        // Roughly a third of seasons; never always, never never.
        assert!(
            events > 150 && events < 450,
            "calendar frequency off: {events}/900"
        );
    }

    #[test]
    fn each_event_moves_exactly_its_own_mechanic() {
        assert!(WorldEvent::MarketFair.buy_price_modifier() < 1.0);
        assert_eq!(WorldEvent::MarketFair.weather_decay_modifier(), 1.0);
        assert!(WorldEvent::HardWinter.weather_decay_modifier() > 1.0);
        assert!(WorldEvent::PlagueYear.illness_contraction_modifier() > 1.0);
    }
}
