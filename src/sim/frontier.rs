//! The frontier and the ungoverned dark (#623): the cause-and-effect chain by
//! which the settled lands shed their restless. A young soul in a village worn
//! by hunger, feud, or want — with little to hold them — may take the road into
//! the open country between the new nations, beyond any town's reach.
//!
//! This module holds the frontier's own state. Slice 1 is only the seed: the
//! count of wanderers the settled lands have lost to the dark, fed by the
//! leave-for-the-road path in `migration`. Later slices turn enough gathered
//! wanderers into bands — living agents that roam, prey, and sometimes settle
//! an outlaw-hold that may, in time, become a town like any other.

use serde::{Deserialize, Serialize};

use crate::rng::SeedRng;
use crate::sim::journal::Voice;

/// Enough wanderers gathered in one country before they make a band of it.
const BAND_FORMS_AT: u32 = 8;
/// A band never musters more than this from one gathering — the rest stay loose
/// in the dark, seed of the next band.
const BAND_MAX_MUSTER: u32 = 14;

/// A living agent of the ungoverned dark (#623 slice 2): a band of the road,
/// mustered from the wanderers the settled lands shed. It holds a patch of wild
/// country and, in slices to come, will roam and prey from it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Band {
    pub id: String,
    /// What the road calls them — "the <word> of <region>".
    pub name: String,
    /// Fighting strength, in souls.
    pub size: u32,
    /// The wild region they hold and range from.
    pub region_idx: usize,
    /// The day they gathered, for reckoning their age.
    pub formed_day: u32,
}

/// The ungoverned country beyond the settled lands, and who has gone into it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Frontier {
    /// Souls who have left the towns for the open country and not joined another
    /// settlement — drifters in the dark. The raw material of the bands.
    #[serde(default)]
    pub wanderers: u32,
    /// The bands that have gathered out of those drifters — living agents of the
    /// dark (#623 slice 2).
    #[serde(default)]
    pub bands: Vec<Band>,
}

impl Frontier {
    /// A soul takes the road into the ungoverned country.
    pub fn take_the_road(&mut self) {
        self.wanderers = self.wanderers.saturating_add(1);
    }
}

/// The names the road gives a band — paired with its country, e.g. "the Ashen
/// of the Reach". A fixed pool, picked deterministically, so a band is named
/// the same every run.
const BAND_EPITHETS: &[&str] = &[
    "Ashen", "Broken", "Roadless", "Hollow", "Grey", "Nameless", "Lean", "Wolfish", "Forsaken",
    "Ragged", "Unsworn", "Cold",
];

/// The frontier's own turn (#623 slice 2): when enough wanderers have gathered
/// in the dark, they muster into a band — a living agent that holds a patch of
/// wild country. Daily, deterministic; the loose remainder stays as wanderers,
/// the seed of the next band.
pub fn tick_frontier(sim: &mut crate::sim::SimState) {
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    if sim.frontier.wanderers < BAND_FORMS_AT {
        return;
    }
    let day = (tick / 24) as u32;
    let seed = sim.world.seed;

    // The wildest country holds them: the region with the fewest living towns,
    // the richest game breaking ties — the emptiest, least-governed ground.
    let Some(region_idx) = wildest_region(sim) else {
        return;
    };

    let muster = sim.frontier.wanderers.min(BAND_MAX_MUSTER);
    sim.frontier.wanderers -= muster;

    let band_n = sim.frontier.bands.len();
    let mut rng = SeedRng::new(seed).fork_for(&format!("band-form-{day}-{band_n}"));
    let epithet = BAND_EPITHETS[(rng.gen_range(BAND_EPITHETS.len() as u32)) as usize];
    let region_name = sim.world.regions[region_idx].name.clone();
    let name = format!("the {epithet} of {region_name}");
    let id = format!("band-{seed}-{day}-{band_n}");

    sim.frontier.bands.push(Band {
        id,
        name: name.clone(),
        size: muster,
        region_idx,
        formed_day: day,
    });
    sim.log(
        tick,
        Voice::Rumor,
        format!(
            "Word on the road: {muster} of the road's lost have banded together in the wild country of {region_name} — they call them {name}."
        ),
    );
}

/// The least-governed region: fewest living settlements, richest game on a tie.
/// `None` only for a world with no regions.
fn wildest_region(sim: &crate::sim::SimState) -> Option<usize> {
    sim.world
        .regions
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let la = a.settlements.iter().filter(|s| s.population > 0).count();
            let lb = b.settlements.iter().filter(|s| s.population > 0).count();
            la.cmp(&lb)
                .then(b.game_richness.partial_cmp(&a.game_richness).unwrap())
        })
        .map(|(i, _)| i)
}
