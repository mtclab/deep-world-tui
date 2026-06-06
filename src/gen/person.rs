/// Person generation (pure: seed + charts → Person).
/// Stub for issue #4.
use crate::charts::Charts;
use crate::model::Person;

pub fn generate_person(_rng: &mut crate::rng::SeedRng, _charts: &Charts, _context: &str) -> Person {
    Person::default()
}
