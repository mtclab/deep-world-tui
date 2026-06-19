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

use crate::model::Terrain;
use crate::rng::SeedRng;
use crate::sim::journal::Voice;

/// A band this strong and this long-lived may lay down its arms and raise a hold.
const HOLD_SETTLE_SIZE: u32 = 12;
/// Days a band must have held its country before it settles — the restless do
/// not put down roots overnight.
const HOLD_SETTLE_AGE_DAYS: u32 = 60;

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
    settle_holds(sim);
    march_tide(sim);
}

/// A hold this size has grown past an outlaw camp into a real town, and
/// governance creeps back over its country — the march is tamed.
const MARCH_TAME_POP: u32 = 40;

/// How a march reads to a traveller (#630): the ungoverned wilderness past the
/// towns' reach, framed by the land it is made of. Deterministic, no roll.
pub fn march_description(region_type: &str) -> &'static str {
    match region_type {
        "forest" => "Wild forest the towns have let go — no road, no hearth, only the deep wood and what moves in it.",
        "upland" => "High, ungoverned country. The stone keeps no law but its own, and the passes belong to whoever holds them.",
        "steppe" => "Open march country, grass to the horizon and no town to claim it — the kind of ground a band can cross unseen.",
        "coast" => "A wild shore beyond the province's reach, its coves known only to those with reason to be unknown.",
        "delta" => "Trackless mire-march where the water hides the ground — and more than the ground.",
        "river_valley" => "A river valley gone back to the wild, its old fields under thorn, its roads swallowed.",
        _ => "Ungoverned march — the open country past the towns' reach, where the Fall's dark never lifted.",
    }
}

/// The Fall's tide at the scale of whole regions (#630 slice 4): the wild and
/// the settled trade ground, slowly. A march where a hold has grown into a real
/// town is **tamed** — the dark recedes, the region joins the settled province.
/// A settled region whose every town has died **falls back** to march — the
/// ungoverned dark returns to the empty country. Seasonal and deterministic, so
/// the line of the frontier moves over a long game, not day to day.
pub fn march_tide(sim: &mut crate::sim::SimState) {
    let day = (sim.world.tick / 24) as u32;
    if day == 0 || !day.is_multiple_of(30) {
        return;
    }
    let mut logs: Vec<String> = Vec::new();
    for region in sim.world.regions.iter_mut() {
        let biggest = region
            .settlements
            .iter()
            .filter(|s| s.population > 0)
            .map(|s| s.population)
            .max();
        if region.is_march {
            // Tamed: a hold has grown into a town the province must reckon with.
            if biggest.is_some_and(|p| p >= MARCH_TAME_POP) {
                region.is_march = false;
                logs.push(format!(
                    "Word on the road: the wild country of {} has a town to its name now — the march is tamed, and the province reaches a little further.",
                    region.name
                ));
            }
        } else if biggest.is_none() {
            // Fallen back: every town in the region is gone, and the dark
            // returns. Only a region that once held towns can fall — a region
            // with no settlement record was never the province's to lose.
            if !region.settlements.is_empty() {
                region.is_march = true;
                region.description = march_description(&region.region_type).to_string();
                logs.push(format!(
                    "Word on the road: {} has fallen back to the wild — its towns are emptied, and the ungoverned dark closes over the country again.",
                    region.name
                ));
            }
        }
    }
    let tick = sim.world.tick;
    for line in logs {
        sim.log(tick, Voice::Rumor, line);
    }
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

/// Where a band seated in `region_idx` finds its prey (#630 slice 2): the
/// richest town in its own country first, and failing that — the case for a
/// band holed up in a march, which has no town of its own — the richest town in
/// a neighbouring region. A band of the marches strikes the settled edge and
/// melts back into the dark. Returns the prey's (region, settlement). Neighbours
/// are scanned in a fixed order, so the raid is deterministic.
/// A band the player would cross while travelling `region_idx` (#630 slice 5):
/// one that holds the region, or one that raids into it from a neighbouring
/// march. Returns its id. Deterministic — bands are scanned in order.
pub fn band_menacing_region(sim: &crate::sim::SimState, region_idx: usize) -> Option<String> {
    // A band based in this very country.
    if let Some(b) = sim
        .frontier
        .bands
        .iter()
        .find(|b| b.region_idx == region_idx)
    {
        return Some(b.id.clone());
    }
    // Else a band that strikes this region from its march.
    sim.frontier
        .bands
        .iter()
        .find(|b| raid_target(sim, b.region_idx).map(|(r, _)| r) == Some(region_idx))
        .map(|b| b.id.clone())
}

/// Break a band the player drove off in the field (#630 slice 5): it loses the
/// better part of its strength, and is scattered outright if that breaks it.
/// Returns the band's name and whether it scattered, for the telling — and
/// `None` if no such band stands (already gone).
pub fn break_band(sim: &mut crate::sim::SimState, band_id: &str) -> Option<(String, bool)> {
    let idx = sim.frontier.bands.iter().position(|b| b.id == band_id)?;
    let (name, size) = {
        let b = &sim.frontier.bands[idx];
        (b.name.clone(), b.size)
    };
    // A fought-off band loses more than half its strength; a small one breaks.
    let loss = (size / 2).max(3);
    if loss >= size {
        sim.frontier.bands.remove(idx);
        Some((name, true))
    } else {
        sim.frontier.bands[idx].size = size - loss;
        Some((name, false))
    }
}

/// The name of the town a band seated in `region_idx` preys on — its own
/// country's, or a neighbour's if it holes up in a town-less march (#630). For
/// the bounty board, so a town raided from the dark can still put coin on the
/// band's head.
pub fn band_prey_town(sim: &crate::sim::SimState, region_idx: usize) -> Option<String> {
    raid_target(sim, region_idx)
        .and_then(|(r, s)| sim.world.regions.get(r)?.settlements.get(s))
        .map(|s| s.name.clone())
}

fn raid_target(sim: &crate::sim::SimState, region_idx: usize) -> Option<(usize, usize)> {
    if let Some(si) = richest_prey(sim, region_idx) {
        return Some((region_idx, si));
    }
    let region = sim.world.regions.get(region_idx)?;
    let n = &region.neighbors;
    [n.north, n.east, n.south, n.west]
        .into_iter()
        .flatten()
        .filter(|&i| i < sim.world.regions.len())
        .find_map(|ni| richest_prey(sim, ni).map(|si| (ni, si)))
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

        match raid_target(sim, region_idx) {
            Some((prey_region, si)) => {
                let from_the_dark = prey_region != region_idx;
                // The raid: spoils scaled by the band's strength and the town's
                // larder, bounded so a single raid stings but never empties a
                // town in a night. The people's sense of safety frays.
                let (town_name, loot_food, looted_goods, killed) = {
                    let s = &mut sim.world.regions[prey_region].settlements[si];
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
                    // A town the raid shrank cannot hold goods past what its
                    // fewer people can: re-clamp to the new cap so the stock
                    // never floats above the population that day (the daily
                    // settlement clamp ran before this frontier turn).
                    let cap = (s.population as f64 * 0.5).max(0.0);
                    for v in s.goods_stock.values_mut() {
                        *v = v.min(cap);
                    }
                    (s.name.clone(), loot_food, looted, killed)
                };
                // Spoils swell the band — success draws more of the road's lost.
                let grow = if loot_food + looted_goods > 1.0 { 1 } else { 0 };
                sim.frontier.bands[bi].size = (size + grow).min(40);
                let melt_back = if from_the_dark {
                    "and rode back into the marches"
                } else {
                    "and melted back into the country"
                };
                if killed {
                    logs.push(format!(
                        "Word on the road: {band_name} fell on {town_name} in the night — stores carried off, a life taken, {melt_back}."
                    ));
                } else {
                    logs.push(format!(
                        "Word on the road: {band_name} raided {town_name} — they took what they could {melt_back}."
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

/// The outlaw-hold (#623 slice 4): a band grown strong and grown old in its
/// country may lay down its arms and raise a hold of its own — a rough, lawless
/// place, but a place, with people and a hearth. Mechanically it is a hamlet
/// like any other, so the living world takes it up from there: it farms, trades,
/// holds a faith, and over generations the Fall's long tail may make an honest
/// town of it. Deterministic; one hold at most per band.
fn settle_holds(sim: &mut crate::sim::SimState) {
    let tick = sim.world.tick;
    let day = (tick / 24) as u32;
    let seed = sim.world.seed;

    // Which bands are ready to settle, by id (snapshot before we mutate world).
    let ready: Vec<(String, String, usize, u32)> = sim
        .frontier
        .bands
        .iter()
        .filter(|b| {
            b.size >= HOLD_SETTLE_SIZE
                && day.saturating_sub(b.formed_day) >= HOLD_SETTLE_AGE_DAYS
                && region_has_room(sim, b.region_idx)
        })
        .map(|b| (b.id.clone(), b.name.clone(), b.region_idx, b.size))
        .collect();

    let mut settled: Vec<String> = Vec::new();
    for (band_id, band_name, region_idx, size) in ready {
        let Some((x, y)) = open_frontier_site(sim, region_idx) else {
            continue;
        };
        let region_id = sim.world.regions[region_idx].id.clone();
        // The hold's founders are the band made flesh — a roster generated from
        // the region's own people-mix, capped to a hamlet's sample.
        let n = size.min(12);
        let mut settlers = Vec::with_capacity(n as usize);
        for k in 0..n {
            let prng = SeedRng::new(seed).fork_for(&format!("hold-{band_id}-{k}"));
            settlers.push(crate::gen::person::generate_person_from(
                prng,
                &region_id,
                "",
                &sim.charts,
            ));
        }
        let mut rng = SeedRng::new(seed).fork_for(&format!("hold-spawn-{band_id}"));
        if let Some((_id, hold_name)) =
            crate::sim::founding::spawn_settlement(sim, region_idx, x, y, settlers, &mut rng)
        {
            settled.push(band_id.clone());
            sim.log(
                tick,
                Voice::Rumor,
                format!(
                    "Word on the road: {band_name} have laid down their arms and raised a hold at {hold_name} — a lawless place, but a place, with smoke from its roofs."
                ),
            );
        }
    }
    sim.frontier.bands.retain(|b| !settled.contains(&b.id));
}

/// A region with room for one more settlement. A band settles a hold in the
/// country it has held and terrorized — it grew strong on the raids there — so
/// the only bar is room, not emptiness (a band in truly empty country starves
/// before it ever grows old enough to settle).
fn region_has_room(sim: &crate::sim::SimState, region_idx: usize) -> bool {
    sim.world
        .regions
        .get(region_idx)
        .map(|r| r.settlements.len() < 3)
        .unwrap_or(false)
}

/// Open ground for a hold: a Grass tile well clear of any settled tile. `None`
/// if the region has no such room. Deterministic — first such tile in scan
/// order, like the founding land-take.
fn open_frontier_site(sim: &crate::sim::SimState, region_idx: usize) -> Option<(usize, usize)> {
    let terr = &sim.world.regions.get(region_idx)?.terrain;
    for y in 0..terr.height {
        for x in 0..terr.width {
            if terr.get(x, y) != Some(Terrain::Grass) {
                continue;
            }
            let near = (y.saturating_sub(8)..(y + 9).min(terr.height)).any(|ty| {
                (x.saturating_sub(8)..(x + 9).min(terr.width))
                    .any(|tx| terr.get(tx, ty) == Some(Terrain::Settlement))
            });
            if !near {
                return Some((x, y));
            }
        }
    }
    None
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
