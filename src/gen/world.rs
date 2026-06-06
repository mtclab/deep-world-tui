/// World generation (pure: seed + charts → World).
/// Stub for issue #4.
use crate::charts::Charts;
use crate::model::World;

pub fn generate_world(_rng: &mut crate::rng::SeedRng, _charts: &Charts) -> World {
    World::default()
}
