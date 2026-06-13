//! Wild species: the Wildlife encounter was one anonymous creature for the
//! whole continent. Now the land has its own animals — boreal, terrain-true,
//! season-true — and a few things from after the Fall that a sober witness
//! could still explain away (proximity is never confirmed; that rule binds
//! beasts too). Roster grows version by version; this is the founding stock.

use super::{Season, Terrain};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WildSpecies {
    Wolf,
    BrownBear,
    Elk,
    ForestReindeer,
    Lynx,
    Wolverine,
    Boar,
    RedFox,
    Adder,
    EagleOwl,
    Capercaillie,
    Hare,
    Beaver,
    RingedSeal,
    MireCrane,
    // After the Fall, some things in the far places are not quite right —
    // and every sighting has a sober explanation, if you want one.
    HollowStag,
    MireLight,
    CaveBreather,
    // Tranche 2 (#393): the dry lands, the steppe, the under-places, the
    // high ledges. All mundane — the strange must stay strange.
    SandOx,
    KethVaal,
    GlassBeetle,
    HeatShimmer,
    SteppeBison,
    SteppeLark,
    CaveBat,
    GhostGoat,
}

impl WildSpecies {
    pub fn all() -> &'static [WildSpecies] {
        use WildSpecies::*;
        &[
            Wolf,
            BrownBear,
            Elk,
            ForestReindeer,
            Lynx,
            Wolverine,
            Boar,
            RedFox,
            Adder,
            EagleOwl,
            Capercaillie,
            Hare,
            Beaver,
            RingedSeal,
            MireCrane,
            HollowStag,
            MireLight,
            CaveBreather,
            SandOx,
            KethVaal,
            GlassBeetle,
            HeatShimmer,
            SteppeBison,
            SteppeLark,
            CaveBat,
            GhostGoat,
        ]
    }

    pub fn name(self) -> &'static str {
        use WildSpecies::*;
        match self {
            Wolf => "wolf",
            BrownBear => "brown bear",
            Elk => "elk",
            ForestReindeer => "forest reindeer",
            Lynx => "lynx",
            Wolverine => "wolverine",
            Boar => "boar",
            RedFox => "red fox",
            Adder => "adder",
            EagleOwl => "eagle-owl",
            Capercaillie => "capercaillie",
            Hare => "hare",
            Beaver => "beaver",
            RingedSeal => "ringed seal",
            MireCrane => "mire crane",
            HollowStag => "hollow stag",
            MireLight => "mire-light",
            CaveBreather => "something in the dark",
            SandOx => "sand-ox",
            KethVaal => "keth-vaal lizard",
            GlassBeetle => "glass-beetle",
            HeatShimmer => "heat-shimmer beast",
            SteppeBison => "steppe bison",
            SteppeLark => "steppe-lark",
            CaveBat => "cave bat",
            GhostGoat => "ghost-goat",
        }
    }

    /// Where this animal is at home. An encounter never rolls a seal in a
    /// spruce forest.
    pub fn habitats(self) -> &'static [Terrain] {
        use Terrain as T;
        use WildSpecies::*;
        match self {
            Wolf => &[T::Forest, T::Tundra, T::Grass],
            BrownBear => &[T::Forest],
            Elk => &[T::Forest, T::Swamp],
            ForestReindeer => &[T::Forest, T::Tundra],
            Lynx => &[T::Forest],
            Wolverine => &[T::Tundra, T::Mountain, T::Forest],
            Boar => &[T::Forest, T::Farmland, T::Grass],
            RedFox => &[T::Grass, T::Forest, T::Farmland],
            Adder => &[T::Grass, T::Sand, T::Swamp],
            EagleOwl => &[T::Forest, T::Mountain],
            Capercaillie => &[T::Forest],
            Hare => &[T::Grass, T::Tundra, T::Farmland],
            Beaver => &[T::Swamp, T::Coast],
            RingedSeal => &[T::Coast],
            MireCrane => &[T::Swamp],
            HollowStag => &[T::Forest, T::Tundra],
            MireLight => &[T::Swamp],
            CaveBreather => &[T::Cave],
            SandOx => &[T::Sand, T::DeepDesert],
            KethVaal => &[T::Sand, T::DeepDesert],
            GlassBeetle => &[T::Sand, T::DeepDesert],
            HeatShimmer => &[T::DeepDesert],
            SteppeBison => &[T::Grass, T::Tundra],
            SteppeLark => &[T::Grass, T::Tundra],
            CaveBat => &[T::Cave],
            GhostGoat => &[T::Mountain],
        }
    }

    /// Seasonal presence weight (0 = absent). Adders sleep through the
    /// Frost; the capercaillie lek belongs to the Thaw.
    pub fn season_weight(self, season: Season) -> u32 {
        use Season::*;
        use WildSpecies::*;
        match (self, season) {
            (Adder, Frost) => 0,
            (Adder, Thaw) => 2,
            (BrownBear, Frost) => 1, // a woken bear is rare and bad news
            (BrownBear, _) => 3,
            (Capercaillie, Thaw) => 4,
            (Wolf, Frost) => 5, // hunger makes the packs bold
            (MireCrane, Frost) => 0,
            (HollowStag | MireLight | CaveBreather, _) => 1, // always rare
            (SteppeLark, Frost) => 0,                        // gone south with the waterfowl
            (SteppeBison, Frost) => 2,                       // the herds draw in tight
            (HeatShimmer, _) => 1,                           // an ambush predator is rarely seen
            (CaveBat, _) => 5,                               // the under-places are mostly bats
            _ => 3,
        }
    }

    /// 0 = flees on sight, 1 = stands its ground, 2 = dangerous.
    pub fn danger(self) -> u8 {
        use WildSpecies::*;
        match self {
            Hare | RedFox | Capercaillie | Beaver | RingedSeal | MireCrane | EagleOwl
            | ForestReindeer | KethVaal | GlassBeetle | SteppeLark | CaveBat | GhostGoat => 0,
            Elk | Lynx | Adder | HollowStag | MireLight | CaveBreather | SandOx | HeatShimmer => 1,
            Wolf | BrownBear | Boar | Wolverine | SteppeBison => 2,
        }
    }

    /// Whether the creature can be hunted for hide and meat at all. Danger-2
    /// animals are a fight, not a harvest; the uncanny are not taken for trade.
    pub fn huntable(self) -> bool {
        self.danger() <= 1 && !self.uncanny()
    }

    /// Base harvest of a clean kill: (hide, meat). Bigger game gives more meat;
    /// every huntable creature gives at least one hide. Before luck and the
    /// land's richness lean it. The uncanny and danger-2 give nothing here.
    pub fn hunt_yield(self) -> (u32, u32) {
        use WildSpecies::*;
        if !self.huntable() {
            return (0, 0);
        }
        match self {
            // Small game: a hide and a little meat.
            Hare | RedFox | Capercaillie | MireCrane | EagleOwl | GlassBeetle | SteppeLark
            | CaveBat => (1, 1),
            // Mid game: a hide and a good portion.
            Beaver | RingedSeal | KethVaal | GhostGoat | Lynx | Adder => (1, 2),
            // Large game: a fuller hide and a haul of meat.
            Elk | ForestReindeer | SandOx => (2, 3),
            // Anything else huntable: a modest take.
            _ => (1, 1),
        }
    }

    /// A creature whose bite carries venom, not just teeth. A strike that lands
    /// courses the blood — the wound is the least of it.
    pub fn venomous(self) -> bool {
        matches!(self, WildSpecies::Adder)
    }

    /// Not natural — post-Fall strangeness, always written deniable.
    pub fn uncanny(self) -> bool {
        matches!(
            self,
            WildSpecies::HollowStag | WildSpecies::MireLight | WildSpecies::CaveBreather
        )
    }

    /// The encounter line. Uncanny lines stay deniable: a sober witness
    /// could explain every one of them away.
    pub fn line(self) -> &'static str {
        use WildSpecies::*;
        match self {
            Wolf => "Wolves on the trail ahead — lean, patient, counting you.",
            BrownBear => "A brown bear rises from the bilberry scrub, deciding what you are.",
            Elk => "A bull elk stands square in the way, antlers low. He was here first.",
            ForestReindeer => "Forest reindeer drift between the trunks, unhurried as weather.",
            Lynx => "A lynx watches from a deadfall, tufted ears reading you.",
            Wolverine => "A wolverine claims the kill between you and the path, and will not move.",
            Boar => "A boar sow stops rooting. Striped piglets scatter behind her — bad sign.",
            RedFox => {
                "A red fox trots the verge with something in its mouth, supremely unbothered."
            }
            Adder => "An adder lies coiled on the warm stone, exactly where your foot was going.",
            EagleOwl => "An eagle-owl opens its eyes in the dusk — two orange lamps, soundless.",
            Capercaillie => "A capercaillie cock bursts from the heather, all thunder and temper.",
            Hare => "A hare bolts, stops dead, and stares back at you sideways.",
            Beaver => "A beaver slaps the water flat — the whole pond knows about you now.",
            RingedSeal => "A ringed seal watches from beyond the break, patient as a buoy.",
            MireCrane => "Cranes wade the mire, grey and deliberate, voices like old hinges.",
            HollowStag => {
                "A stag stands wrong at the tree-line — antlers too wide, eyes too still. \
                 It watches without feeding, and does not run. Old light, you tell yourself."
            }
            MireLight => {
                "A pale light keeps pace with you over the mire, just past arm's reach. \
                 Marsh gas, the sensible say. It stops when you stop."
            }
            CaveBreather => {
                "From deeper in the dark: breathing. Slow, large, even. Air moving through \
                 stone, surely. The dark does not say."
            }
            SandOx => {
                "A sand-ox stands hock-deep in the lee of a dune, chewing what the desert \
                 grudged it. It does not yield the shade."
            }
            KethVaal => {
                "A keth-vaal lizard flows off the hot stone and is gone — water owed nobody."
            }
            GlassBeetle => {
                "Glass-beetles cross the sand in single file, backs throwing the light like sparks."
            }
            HeatShimmer => {
                "Something low keeps to the shimmer at the edge of sight, patient as thirst. \
                 It waits where the heat bends."
            }
            SteppeBison => {
                "The steppe bison lift their heads as one — a wall of winter-coated muscle \
                 deciding whether you matter."
            }
            SteppeLark => "A steppe-lark climbs singing out of the grass, straight up, all spring.",
            CaveBat => "Bats pour past your ears toward the night-mouth, a river of soft leather.",
            GhostGoat => {
                "A ghost-goat watches from a ledge no path serves, pale against the rock, \
                 chewing, unimpressed."
            }
        }
    }

    /// Deterministic species roll for the tile and day. None when nothing of
    /// the roster lives on this ground.
    pub fn roll(terrain: Terrain, season: Season, seed: u64) -> Option<WildSpecies> {
        let candidates: Vec<(WildSpecies, u32)> = Self::all()
            .iter()
            .filter(|s| s.habitats().contains(&terrain))
            .map(|&s| (s, s.season_weight(season)))
            .filter(|&(_, w)| w > 0)
            .collect();
        let total: u32 = candidates.iter().map(|&(_, w)| w).sum();
        if total == 0 {
            return None;
        }
        let hash = seed.wrapping_mul(0x9E3779B97F4A7C15) ^ (terrain as u64).wrapping_mul(7919);
        let mut pick = (hash % total as u64) as u32;
        for (s, w) in candidates {
            if pick < w {
                return Some(s);
            }
            pick -= w;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn danger_two_and_uncanny_are_not_huntable() {
        // A bear is a fight, not a harvest; the uncanny aren't taken for trade.
        assert!(
            !WildSpecies::BrownBear.huntable(),
            "bear must not be huntable"
        );
        assert!(!WildSpecies::Wolf.huntable(), "wolf must not be huntable");
        assert!(
            !WildSpecies::HollowStag.huntable(),
            "uncanny (HollowStag) must not be huntable"
        );
        assert!(WildSpecies::Hare.huntable(), "hare must be huntable");
        assert!(
            WildSpecies::Elk.huntable(),
            "elk (danger 1) must be huntable"
        );
    }

    #[test]
    fn hunt_yield_matches_huntability_and_size() {
        for s in WildSpecies::all() {
            let (hide, meat) = s.hunt_yield();
            if s.huntable() {
                assert!(hide >= 1 && meat >= 1, "{s:?} huntable but yields nothing");
            } else {
                assert_eq!((hide, meat), (0, 0), "{s:?} not huntable but yields");
            }
        }
        // Big game gives more than small.
        assert!(WildSpecies::Elk.hunt_yield().1 > WildSpecies::Hare.hunt_yield().1);
    }

    #[test]
    fn no_seals_in_the_spruce() {
        for s in WildSpecies::all() {
            for t in s.habitats() {
                assert!(
                    t.passable() || *t == Terrain::Mountain || *t == Terrain::Water,
                    "{:?} habitat {:?} must be real ground",
                    s,
                    t
                );
            }
        }
        assert!(!WildSpecies::RingedSeal
            .habitats()
            .contains(&Terrain::Forest));
        assert!(!WildSpecies::BrownBear.habitats().contains(&Terrain::Coast));
    }

    #[test]
    fn rolls_are_terrain_true_and_deterministic() {
        for seed in 0..200u64 {
            if let Some(s) = WildSpecies::roll(Terrain::Forest, Season::Green, seed) {
                assert!(s.habitats().contains(&Terrain::Forest));
                assert_eq!(
                    Some(s),
                    WildSpecies::roll(Terrain::Forest, Season::Green, seed),
                    "same seed, same beast"
                );
            }
        }
    }

    #[test]
    fn adders_sleep_through_the_frost() {
        for seed in 0..400u64 {
            if let Some(s) = WildSpecies::roll(Terrain::Grass, Season::Frost, seed) {
                assert_ne!(s, WildSpecies::Adder, "no adders on frost ground");
            }
        }
    }

    #[test]
    fn the_under_places_are_mostly_bats() {
        // Cave's only resident used to be the uncanny CaveBreather — every
        // sighting underground was strange. The strange must stay strange.
        let mut uncanny = 0;
        let mut total = 0;
        for seed in 0..1000u64 {
            if let Some(s) = WildSpecies::roll(Terrain::Cave, Season::Green, seed) {
                total += 1;
                if s.uncanny() {
                    uncanny += 1;
                }
            }
        }
        assert!(total > 0);
        assert!(
            (uncanny as f64) < (total as f64) * 0.30,
            "mundane majority underground: {uncanny}/{total}"
        );
    }

    #[test]
    fn the_dry_lands_have_their_own_animals() {
        // Before tranche 2, Sand rolled only the adder and DeepDesert nothing.
        assert!(WildSpecies::roll(Terrain::DeepDesert, Season::Green, 7).is_some());
        for seed in 0..200u64 {
            if let Some(s) = WildSpecies::roll(Terrain::DeepDesert, Season::Frost, seed) {
                assert!(s.habitats().contains(&Terrain::DeepDesert));
            }
        }
    }

    #[test]
    fn the_lark_goes_south_for_the_frost() {
        for seed in 0..400u64 {
            if let Some(s) = WildSpecies::roll(Terrain::Grass, Season::Frost, seed) {
                assert_ne!(s, WildSpecies::SteppeLark, "gone with the waterfowl");
            }
        }
    }

    #[test]
    fn the_uncanny_stays_rare() {
        let mut uncanny = 0;
        let mut total = 0;
        for seed in 0..1000u64 {
            if let Some(s) = WildSpecies::roll(Terrain::Forest, Season::Green, seed) {
                total += 1;
                if s.uncanny() {
                    uncanny += 1;
                }
            }
        }
        assert!(total > 0);
        assert!(
            (uncanny as f64) < (total as f64) * 0.12,
            "the strange must stay strange: {uncanny}/{total}"
        );
    }
}
