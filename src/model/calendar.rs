//! The world calendar: seasons become *events*, not just multipliers (#417).
//! A season can carry a market fair, a hard winter, or a plague year — each
//! deterministic from the world seed + season + year, announced in rumor and
//! journal, visible in exactly one mechanic, and gone when the season turns.

use crate::model::weather::Season;
use crate::model::GodName;
use serde::{Deserialize, Serialize};

/// The fixed holy days of the Five across the 90-day year (#457): each god's
/// day falls on the same day-of-year, every year, so a devotee can plan for it.
/// On a god's holy day, acts of devotion to that god weigh heavier. One per
/// god, spread across the three seasons. Days are 0-based day-of-year.
pub const HOLY_DAYS: [(u32, GodName); 5] = [
    (9, GodName::Oltzed),  // Thaw: the first fires of the working year
    (24, GodName::Masa),   // Thaw: the road-and-ledger day
    (44, GodName::Keuru),  // Green: midsummer, the green height
    (59, GodName::Kukri),  // Green→Frost: the ancestor vigil at the turn
    (74, GodName::Sampsa), // Frost: the star-reckoning under cold skies
];

/// The day-of-year (0-based) of an absolute game `day`.
fn day_of_year(day: u32) -> u32 {
    (day.max(1) - 1) % Season::YEAR_DAYS
}

/// The god whose holy day falls on this absolute `day`, if any (#457).
pub fn holy_day_god(day: u32) -> Option<GodName> {
    let doy = day_of_year(day);
    HOLY_DAYS.iter().find(|(d, _)| *d == doy).map(|(_, g)| *g)
}

/// The next holy day from `day`: its god and how many days off (0 = today is
/// one). Always returns one — the year always comes round again.
pub fn next_holy_day(day: u32) -> (GodName, u32) {
    let doy = day_of_year(day);
    for delta in 0..Season::YEAR_DAYS {
        let d = (doy + delta) % Season::YEAR_DAYS;
        if let Some((_, g)) = HOLY_DAYS.iter().find(|(hd, _)| *hd == d) {
            return (*g, delta);
        }
    }
    // The table is non-empty, so a holy day is always found within the year.
    (GodName::Oltzed, 0)
}

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
    fn holy_days_recur_on_the_same_day_of_year() {
        // The Oltzed day is day-of-year 9 (0-based) → absolute day 10, and again
        // a full year on.
        assert_eq!(holy_day_god(10), Some(GodName::Oltzed));
        assert_eq!(holy_day_god(10 + Season::YEAR_DAYS), Some(GodName::Oltzed));
        // An ordinary day keeps no god's holy day.
        assert_eq!(holy_day_god(3), None);
    }

    #[test]
    fn every_god_keeps_exactly_one_holy_day() {
        let mut found = std::collections::HashSet::new();
        for doy in 0..Season::YEAR_DAYS {
            if let Some(g) = holy_day_god(doy + 1) {
                found.insert(g);
            }
        }
        assert_eq!(found.len(), 5, "all five gods keep a holy day");
    }

    #[test]
    fn next_holy_day_is_today_when_one_falls_now() {
        let (g, off) = next_holy_day(10); // Oltzed's day
        assert_eq!(g, GodName::Oltzed);
        assert_eq!(off, 0);
    }

    #[test]
    fn next_holy_day_counts_forward_and_always_finds_one() {
        // From the day after Oltzed's (day 11), the next is Masa's (doy 24).
        let (g, off) = next_holy_day(11);
        assert_eq!(g, GodName::Masa);
        assert_eq!(off, 24 - 10);
        // From any day of any year, a holy day is found within the year.
        for d in 1..(Season::YEAR_DAYS * 3) {
            let (_, off) = next_holy_day(d);
            assert!(off < Season::YEAR_DAYS);
        }
    }

    #[test]
    fn each_event_moves_exactly_its_own_mechanic() {
        assert!(WorldEvent::MarketFair.buy_price_modifier() < 1.0);
        assert_eq!(WorldEvent::MarketFair.weather_decay_modifier(), 1.0);
        assert!(WorldEvent::HardWinter.weather_decay_modifier() > 1.0);
        assert!(WorldEvent::PlagueYear.illness_contraction_modifier() > 1.0);
    }
}
