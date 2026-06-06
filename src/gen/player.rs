/// Player start generation.
/// Stub for issue #4.
use crate::charts::Charts;
use crate::model::Player;
use crate::rng::SeedRng;

pub fn generate_player(_rng: &mut SeedRng, _charts: &Charts) -> Player {
    Player::default()
}
