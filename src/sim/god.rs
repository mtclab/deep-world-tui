use crate::model::GodName;

/// FNV-1a 64-bit, then collapse to u32. Stable, allocation-free, deterministic.
/// (Same shape as `sim::signals::hash_str` if/when that module lands; kept
/// local here to avoid coupling the god module to a sibling that may move.)
fn hash_str(s: &str) -> u32 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    (h as u32) ^ ((h >> 32) as u32)
}

/// Map a people-name to its patron God of the Five.
///
/// Returns `None` for peoples with no patron in the canonical five
/// (e.g. unknown or off-world names). See `AGENTS.md` for the canon table.
pub fn patron_of(people: &str) -> Option<GodName> {
    match people {
        "Metsik" => Some(GodName::Keuru),
        "Sepät" | "Ahjo" => Some(GodName::Oltzed),
        "Arkit" => Some(GodName::Sampsa),
        "Väylä" | "Mëräk" => Some(GodName::Masa),
        "Laakso" | "Tzäkhar" | "She'ar" => Some(GodName::Kukri),
        "Häl" => Some(GodName::Keuru),
        "Khör" => Some(GodName::Sampsa),
        _ => None,
    }
}

/// First-person present-tense prayer line, keyed stably by `person_id`.
///
/// No numbers, labels, or god names are ever printed — only sensory, organic
/// imagery a sleeping traveler would half-remember.
fn prayer_lines(god: GodName) -> &'static [&'static str] {
    match god {
        GodName::Oltzed => &[
            "I dream of hammers on bright metal, and a chariot hung on a chain of stars.",
            "In the dark, a forge-bellows breath; my hands, though still, are full of work.",
            "A wheel turns behind my eyes. I wake with sawdust on my tongue.",
            "A blueprint curls at the foot of my bed. I do not read it. I already know.",
        ],
        GodName::Keuru => &[
            "I wake among voices. Someone is laughing. There is pine resin on the wind.",
            "Moss, bread, the warmth of a stranger's hearth — I drift, and the drift smells of cedar.",
            "I dream I am seated at a long table. No one speaks my name, and I am welcome.",
            "A road turns through a forest of lanterns. I follow it, though I never left the bed.",
        ],
        GodName::Sampsa => &[
            "Pages turn of their own accord. The ink is not yet dry, and yet I know the ending.",
            "A library of bone. I borrow a book and wake with a single line on my palm.",
            "A scholar I have never met reads aloud to me. The room has no walls.",
            "A star moves. I make a note of it. The note is older than I am.",
        ],
        GodName::Masa => &[
            "I count coins in a marketplace that has not been built. Each coin is someone's promise.",
            "A cart passes. The driver nods. I have not met them, and yet I will trust them.",
            "I weigh two sacks of grain, and they balance. The second is full of patience.",
            "A ledger opens in my lap. The numbers are not numbers. They are people I have not yet met.",
        ],
        GodName::Kukri => &[
            "I am alone in a high place. The wind is kind. I do not ask for company.",
            "Snow falls in a room that has no ceiling. I am not cold.",
            "An old voice says nothing. I am content.",
            "I walk a long road and meet no one. The road is patient. So am I.",
        ],
    }
}

/// Pick a prayer line deterministically from `person_id` and the god's bank.
pub fn prayer_flavor(god: GodName, person_id: &str) -> &'static str {
    let bank = prayer_lines(god);
    let idx = (hash_str(person_id) as usize) % bank.len();
    bank[idx]
}

/// Decide whether the player encounters a prayer on this rest tick, and return
/// the flavor line if so. Deterministic: same `world_tick` and `person_id` →
/// same outcome and same line.
///
/// Threshold is gentle — roughly one quiet moment every few rest cycles at a
/// settlement whose dominant people is a patron's.
pub fn maybe_prayer(
    player_id: &str,
    dominant_people: Option<&str>,
    world_tick: u64,
) -> Option<String> {
    let people = dominant_people?;
    let god = patron_of(people)?;
    // Threshold: ~1 in 4 rests. Mix tick + player id so cadence feels human
    // and not metronomic; never less than 0 (always Some chance to fire).
    let mix = hash_str(&format!("prayer:{player_id}:{world_tick}"));
    if !mix.is_multiple_of(4) {
        return None;
    }
    let line = prayer_flavor(god, player_id);
    Some(line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patron_of_canonical_peoples() {
        assert_eq!(patron_of("Metsik"), Some(GodName::Keuru));
        assert_eq!(patron_of("Sepät"), Some(GodName::Oltzed));
        assert_eq!(patron_of("Ahjo"), Some(GodName::Oltzed));
        assert_eq!(patron_of("Arkit"), Some(GodName::Sampsa));
        assert_eq!(patron_of("Väylä"), Some(GodName::Masa));
        assert_eq!(patron_of("Laakso"), Some(GodName::Kukri));
        assert_eq!(patron_of("Tzäkhar"), Some(GodName::Kukri));
        assert_eq!(patron_of("Mëräk"), Some(GodName::Masa));
        assert_eq!(patron_of("She'ar"), Some(GodName::Kukri));
        assert_eq!(patron_of("Häl"), Some(GodName::Keuru));
        assert_eq!(patron_of("Khör"), Some(GodName::Sampsa));
    }

    #[test]
    fn patron_of_unknown_people_returns_none() {
        assert_eq!(patron_of(""), None);
        assert_eq!(patron_of("Unpeople"), None);
        assert_eq!(patron_of("metsik"), None); // case-sensitive on purpose
    }

    #[test]
    fn prayer_flavor_stable_per_id() {
        for god in [
            GodName::Oltzed,
            GodName::Keuru,
            GodName::Sampsa,
            GodName::Masa,
            GodName::Kukri,
        ] {
            let a = prayer_flavor(god, "player-7");
            let b = prayer_flavor(god, "player-7");
            assert_eq!(a, b, "prayer_flavor must be stable for same id + god");
            assert!(!a.is_empty(), "prayer line must not be empty");
        }
    }

    #[test]
    fn prayer_flavor_never_prints_god_name() {
        // The prayer is the player's dream, not a theology lecture.
        for god in [
            GodName::Oltzed,
            GodName::Keuru,
            GodName::Sampsa,
            GodName::Masa,
            GodName::Kukri,
        ] {
            for i in 0..10 {
                let id = format!("test-{i}");
                let line = prayer_flavor(god, &id);
                assert!(!line.contains("Oltzed"), "prayer leaked god name: {line}");
                assert!(!line.contains("Keuru"), "prayer leaked god name: {line}");
                assert!(!line.contains("Sampsa"), "prayer leaked god name: {line}");
                assert!(!line.contains("Masa"), "prayer leaked god name: {line}");
                assert!(!line.contains("Kukri"), "prayer leaked god name: {line}");
            }
        }
    }

    #[test]
    fn prayer_flavor_never_prints_numbers_or_people_names() {
        // The line is a dream fragment — no stats, no names, no digits.
        let people_to_check = [
            "Metsik", "Sepät", "Ahjo", "Arkit", "Väylä", "Laakso", "Tzäkhar", "Mëräk", "She'ar",
            "Häl", "Khör",
        ];
        let banned: &[&str] = &[
            "Metsik", "Sepät", "Ahjo", "Arkit", "Väylä", "Laakso", "0", "1", "2", "3", "4", "5",
            "6", "7", "8", "9",
        ];
        for god in [
            GodName::Oltzed,
            GodName::Keuru,
            GodName::Sampsa,
            GodName::Masa,
            GodName::Kukri,
        ] {
            for people in people_to_check {
                let god2 = patron_of(people).unwrap();
                let _ = god2; // silence unused if we drop the assertion
                for i in 0..3 {
                    let line = prayer_flavor(god, &format!("{people}-{i}"));
                    for token in banned {
                        assert!(
                            !line.contains(token),
                            "prayer leaked token '{token}': {line}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn maybe_prayer_deterministic_per_tick() {
        let a = maybe_prayer("player-1", Some("Metsik"), 42);
        let b = maybe_prayer("player-1", Some("Metsik"), 42);
        assert_eq!(a, b);
    }

    #[test]
    fn maybe_prayer_no_fire_without_settlement_people() {
        assert_eq!(maybe_prayer("player-1", None, 0), None);
    }

    #[test]
    fn maybe_prayer_no_fire_for_unknown_people() {
        assert_eq!(maybe_prayer("player-1", Some("Unpeople"), 0), None);
    }

    #[test]
    fn maybe_prayer_fires_eventually_across_ticks() {
        // Across 100 ticks, we should see at least one fire (1/4 rate each).
        let mut count = 0;
        for t in 0..100 {
            if maybe_prayer("player-X", Some("Laakso"), t).is_some() {
                count += 1;
            }
        }
        assert!(
            count > 10,
            "prayer should fire sometimes (got {count}/100), threshold too tight"
        );
    }
}