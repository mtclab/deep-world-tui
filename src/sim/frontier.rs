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

/// The frontier's own turn (#623): the dark gathers its bands (slice 2) and the
/// bands that already hold the wild country roam and prey (slice 3). Daily and
/// deterministic.
pub fn tick_frontier(sim: &mut crate::sim::SimState) {
    let tick = sim.world.tick;
    if !tick.is_multiple_of(24) {
        return;
    }
    form_bands(sim);
    bands_act(sim);
}

/// When enough wanderers have gathered, they muster into a band — a living
/// agent that holds a patch of wild country. The loose remainder stays as
/// wanderers, the seed of the next band.
fn form_bands(sim: &mut crate::sim::SimState) {
    if sim.frontier.wanderers < BAND_FORMS_AT {
        return;
    }
    let tick = sim.world.tick;
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

/// The largest living settlement in a region, if any — a band's nearest prey.
fn richest_prey(sim: &crate::sim::SimState, region_idx: usize) -> Option<usize> {
    let region = sim.world.regions.get(region_idx)?;
    region
        .settlements
        .iter()
        .enumerate()
        .filter(|(_, s)| s.population > 0)
        .max_by_key(|(_, s)| s.population)
        .map(|(i, _)| i)
}

/// Each band's day in the wild (#623 slice 3): it preys on the nearest town if
/// its country holds one — taking food and goods, fraying the people's safety,
/// and growing fat on the spoils — or, finding only empty country, roams to a
/// neighbouring region and is worn down by the hungry road. A band ground down
/// to nothing scatters. All deterministic per band and day.
fn bands_act(sim: &mut crate::sim::SimState) {
    let tick = sim.world.tick;
    let day = (tick / 24) as u32;
    let seed = sim.world.seed;
    let mut logs: Vec<String> = Vec::new();

    for bi in 0..sim.frontier.bands.len() {
        let (band_id, band_name, region_idx, size) = {
            let b = &sim.frontier.bands[bi];
            (b.id.clone(), b.name.clone(), b.region_idx, b.size)
        };
        let mut rng = SeedRng::new(seed).fork_for(&format!("band-act-{band_id}-{day}"));

        match richest_prey(sim, region_idx) {
            Some(si) => {
                // The raid: spoils scaled by the band's strength and the town's
                // larder, bounded so a single raid stings but never empties a
                // town in a night. The people's sense of safety frays.
                let (town_name, loot_food, looted_goods, killed) = {
                    let s = &mut sim.world.regions[region_idx].settlements[si];
                    let loot_food = (s.food_stock * 0.15).min(size as f64 * 1.5);
                    s.food_stock = (s.food_stock - loot_food).max(0.0);
                    let mut looted = 0.0f64;
                    for v in s.goods_stock.values_mut() {
                        let take = (*v * 0.2).min(size as f64 * 0.5);
                        *v -= take;
                        looted += take;
                    }
                    for p in s.people.iter_mut() {
                        p.needs.decay(crate::model::Need::Safety, 0.15);
                    }
                    // A hard raid on a small place can cost a life — the band is
                    // not gentle. Rare, and never the last soul.
                    let killed = if s.people.len() > 2 && rng.gen_f64() < 0.10 {
                        s.people.pop();
                        s.population = s.population.saturating_sub(1).max(s.people.len() as u32);
                        true
                    } else {
                        false
                    };
                    (s.name.clone(), loot_food, looted, killed)
                };
                // Spoils swell the band — success draws more of the road's lost.
                let grow = if loot_food + looted_goods > 1.0 { 1 } else { 0 };
                sim.frontier.bands[bi].size = (size + grow).min(40);
                if killed {
                    logs.push(format!(
                        "Word on the road: {band_name} fell on {town_name} in the night — stores carried off, and a life taken."
                    ));
                } else {
                    logs.push(format!(
                        "Word on the road: {band_name} raided {town_name} — they took what they could and melted back into the country."
                    ));
                }
            }
            None => {
                // Empty country: the band roams to a neighbour and the hungry
                // road wears it down a little.
                if let Some(next) = roam_target(sim, region_idx, &mut rng) {
                    sim.frontier.bands[bi].region_idx = next;
                }
                sim.frontier.bands[bi].size = size.saturating_sub(1);
            }
        }
    }

    // A band ground down to nothing scatters back into the dark.
    let mut scattered: Vec<String> = Vec::new();
    sim.frontier.bands.retain(|b| {
        if b.size == 0 {
            scattered.push(b.name.clone());
            false
        } else {
            true
        }
    });
    for name in scattered {
        logs.push(format!(
            "Word on the road: {name} has scattered — hunger and the road undid them, and the country is quiet again."
        ));
    }

    for line in logs {
        sim.log(tick, Voice::Rumor, line);
    }
}

/// A neighbouring region for a roaming band to drift into, chosen
/// deterministically. `None` if the region has no mapped neighbours.
fn roam_target(sim: &crate::sim::SimState, region_idx: usize, rng: &mut SeedRng) -> Option<usize> {
    let region = sim.world.regions.get(region_idx)?;
    let n = &region.neighbors;
    let nbs: Vec<usize> = [n.north, n.east, n.south, n.west]
        .into_iter()
        .flatten()
        .filter(|&i| i < sim.world.regions.len())
        .collect();
    if nbs.is_empty() {
        return None;
    }
    Some(nbs[rng.gen_range(nbs.len() as u32) as usize])
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
