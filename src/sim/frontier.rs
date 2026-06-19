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
    prey_on_caravans(sim);
    waylay_travellers(sim);
    settle_holds(sim);
    march_tide(sim);
}

/// The bands fall on the travellers, not only the carts (#641): a migrant family
/// crossing a band's country may be waylaid — shaken, their sense of safety
/// frayed, arriving the warier for it. No blood (the souls are conserved on the
/// road and counted), but the danger of the marches is felt by those who cross
/// them. Deterministic per band, party, and day; one party per band a turn.
fn waylay_travellers(sim: &mut crate::sim::SimState) {
    if sim.migrant_parties.is_empty() {
        return;
    }
    let tick = sim.world.tick;
    let day = (tick / 24) as u32;
    let seed = sim.world.seed;

    let mut waylaid: Vec<usize> = Vec::new();
    for band in sim.frontier.bands.iter() {
        for (pi, party) in sim.migrant_parties.iter().enumerate() {
            if waylaid.contains(&pi) {
                continue;
            }
            if crate::sim::migration::migrant_party_tiles(sim, &party.id, band.region_idx, tick)
                .is_empty()
            {
                continue;
            }
            let mut rng =
                SeedRng::new(seed).fork_for(&format!("waylay-{}-{}-{day}", band.id, party.id));
            if rng.gen_f64() < (0.10 + 0.015 * band.size as f64).min(0.5) {
                waylaid.push(pi);
                break;
            }
        }
    }

    let mut struck = false;
    for pi in &waylaid {
        if let Some(party) = sim.migrant_parties.get_mut(*pi) {
            for person in party.people.iter_mut() {
                person.needs.decay(crate::model::Need::Safety, 0.2);
            }
            struck = true;
        }
    }
    if struck {
        sim.log(
            tick,
            Voice::Rumor,
            "Word on the road: travellers were set upon crossing the wild country — shaken, but they kept the road.".to_string(),
        );
    }
}

/// The bands fall on the roads, not just the towns (#641 slice 4): a band that
/// holds country a caravan is crossing may ride it down — its goods carried
/// off, the train left limping on as a wreck. Bands gather thickest in the
/// marches (#623/#630), so the ungoverned edges are where a caravan is likeliest
/// to be taken — the marches' danger falling on the travellers who cross them.
/// Deterministic per band, caravan, and day; one caravan per band a turn.
fn prey_on_caravans(sim: &mut crate::sim::SimState) {
    let tick = sim.world.tick;
    let day = (tick / 24) as u32;
    let seed = sim.world.seed;

    // Gather the raids first (immutable reads of bands + caravans + the grid),
    // then apply them — a band cannot raid through a borrow of itself.
    let mut raids: Vec<(usize, String, usize)> = Vec::new();
    for (bi, band) in sim.frontier.bands.iter().enumerate() {
        for (ci, c) in sim.caravans.iter().enumerate() {
            if c.raided || !c.is_in_transit(tick) {
                continue;
            }
            // The caravan must actually stand on the band's ground now.
            if crate::sim::caravans::caravan_train_tiles(sim, &c.id, band.region_idx, tick)
                .is_empty()
            {
                continue;
            }
            let mut rng =
                SeedRng::new(seed).fork_for(&format!("caravan-prey-{}-{}-{day}", band.id, c.id));
            // A band preys readily; the stronger the surer of the kill.
            let chance = (0.15 + 0.02 * band.size as f64).min(0.6);
            if rng.gen_f64() < chance {
                raids.push((ci, band.name.clone(), bi));
                break; // one caravan a turn — they melt back into the country
            }
        }
    }

    let mut logs: Vec<String> = Vec::new();
    for (ci, band_name, bi) in raids {
        if let Some(c) = sim.caravans.get_mut(ci) {
            if c.raided {
                continue; // already taken by another band this turn
            }
            c.raided = true;
            c.goods.clear();
            logs.push(format!(
                "Word on the road: {band_name} fell on a caravan bound for {} — its goods carried off, the train left limping.",
                c.destination
            ));
        }
        // Spoils swell the band, as a town-raid does.
        if let Some(b) = sim.frontier.bands.get_mut(bi) {
            b.size = (b.size + 1).min(40);
        }
    }
    for line in logs {
        sim.log(tick, Voice::Rumor, line);
    }
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

/// How dangerous a region reads to a traveller (#637 slice 5 — the marches as
/// survival): off the land's standing wildness (a march, and rich game, carry
/// more) and what actually stands on its ground *now* — the size of any
/// outlaw bands, the danger of the beasts, the uncanny weighing heavier. The
/// player reads this before committing to the deep wild: a march is where you
/// go to fight and maybe not come back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerTier {
    /// Governed, quiet country — the towns' reach.
    Settled,
    /// Watch yourself: wild ground, or a settled region with something on it.
    Wary,
    /// Real danger stands here — bands, big game, the strange.
    Perilous,
    /// The deep wild at its worst. Go armed, or do not go.
    Deadly,
}

impl DangerTier {
    /// A word for the country, for the map header.
    pub fn label(self) -> &'static str {
        match self {
            DangerTier::Settled => "settled",
            DangerTier::Wary => "wary",
            DangerTier::Perilous => "perilous",
            DangerTier::Deadly => "deadly",
        }
    }
}

/// Read the danger of a region from its wildness and what stands on it now
/// (#637 slice 5). Deterministic — a pure read of world state, no roll.
pub fn region_danger(sim: &crate::sim::SimState, region_idx: usize) -> DangerTier {
    let Some(region) = sim.world.regions.get(region_idx) else {
        return DangerTier::Settled;
    };
    // The land's standing menace: the ungoverned dark of a march, leaned
    // heavier where the game is rich (a richer wild holds more that bites).
    let mut threat: u32 = 0;
    if region.is_march {
        threat += 6;
    }
    if region.game_richness >= 1.3 {
        threat += 2;
    }
    // The outlaws on the ground — each soul a blade.
    for band in sim
        .frontier
        .bands
        .iter()
        .filter(|b| b.region_idx == region_idx)
    {
        threat += band.size;
    }
    // The beasts standing here, the dangerous and the uncanny weighing more.
    for beast in sim.beasts.iter().filter(|b| b.region_idx == region_idx) {
        threat += 1 + beast.species.danger() as u32;
        if beast.species.uncanny() {
            threat += 2;
        }
    }
    match threat {
        0..=2 => DangerTier::Settled,
        3..=7 => DangerTier::Wary,
        8..=15 => DangerTier::Perilous,
        _ => DangerTier::Deadly,
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
/// Where a band stands on the region grid (#637): a deterministic wild tile in
/// the country it holds, clear of any settlement, derived from the band's id so
/// it sits in the same place every render. `None` if the region has no open
/// wild ground. The player can see it and choose to walk into it.
pub fn band_field_tile(
    sim: &crate::sim::SimState,
    band_id: &str,
    region_idx: usize,
) -> Option<(usize, usize)> {
    let region = sim.world.regions.get(region_idx)?;
    let terr = &region.terrain;
    let h = crate::rng::fnv1a_hash(band_id);
    // Wild ground a band would hold: grass or forest, never a settled tile.
    let mut candidates = 0u64;
    // First pass: count, so the pick is uniform and stable.
    for y in 0..terr.height {
        for x in 0..terr.width {
            if matches!(
                terr.get(x, y),
                Some(crate::model::Terrain::Grass | crate::model::Terrain::Forest)
            ) {
                candidates += 1;
            }
        }
    }
    if candidates == 0 {
        return None;
    }
    let mut target = h % candidates;
    for y in 0..terr.height {
        for x in 0..terr.width {
            if matches!(
                terr.get(x, y),
                Some(crate::model::Terrain::Grass | crate::model::Terrain::Forest)
            ) {
                if target == 0 {
                    return Some((x, y));
                }
                target -= 1;
            }
        }
    }
    None
}

/// How many of a band's members show as individual actors on the grid — a band
/// is people, not a blob, so you see (and cut down) them one by one (#637).
pub const BAND_MEMBERS_SHOWN: usize = 12;

/// The tiles a band's members stand on (#637): individual outlaws clustered on
/// the wild ground the band holds, the leader on the anchor tile and the rest
/// ringed around it, as many as the band has strength for (capped). Each is its
/// own actor you can walk into and fell. Deterministic; clear of settlements.
pub fn band_member_tiles(
    sim: &crate::sim::SimState,
    band_id: &str,
    region_idx: usize,
) -> Vec<(usize, usize)> {
    let Some((ax, ay)) = band_field_tile(sim, band_id, region_idx) else {
        return Vec::new();
    };
    let Some(region) = sim.world.regions.get(region_idx) else {
        return Vec::new();
    };
    let terr = &region.terrain;
    let size = sim
        .frontier
        .bands
        .iter()
        .find(|b| b.id == band_id)
        .map(|b| b.size as usize)
        .unwrap_or(0);
    let want = size.min(BAND_MEMBERS_SHOWN);
    let walkable = |x: usize, y: usize| {
        matches!(
            terr.get(x, y),
            Some(crate::model::Terrain::Grass | crate::model::Terrain::Forest)
        )
    };
    // Spiral outward from the anchor by Chebyshev ring (the leader's tile first,
    // then the ground around it ring by ring), taking wild tiles until the band
    // is placed or we have searched far enough. Deterministic, clustered, and
    // robust where the anchor sits near settled or watered ground.
    let mut tiles = Vec::with_capacity(want);
    if walkable(ax, ay) {
        tiles.push((ax, ay));
    }
    let mut radius = 1i32;
    while tiles.len() < want && radius <= 8 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                // Only the ring at exactly this radius (Chebyshev), in a fixed
                // row-major order — deterministic.
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                if tiles.len() >= want {
                    break;
                }
                let x = ax as i32 + dx;
                let y = ay as i32 + dy;
                if x < 0 || y < 0 {
                    continue;
                }
                let (ux, uy) = (x as usize, y as usize);
                if walkable(ux, uy) && !tiles.contains(&(ux, uy)) {
                    tiles.push((ux, uy));
                }
            }
        }
        radius += 1;
    }
    tiles
}

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

/// One blow struck against a band in a stand-up fight (#637): cut `blow` from
/// its strength, and scatter it if that breaks it. Returns the band's name,
/// whether it scattered, and its remaining size. `None` if no such band stands.
/// The fine-grained counterpart of `break_band` — a single exchange, not a rout.
pub fn strike_band(
    sim: &mut crate::sim::SimState,
    band_id: &str,
    blow: u32,
) -> Option<(String, bool, u32)> {
    let idx = sim.frontier.bands.iter().position(|b| b.id == band_id)?;
    let name = sim.frontier.bands[idx].name.clone();
    let size = sim.frontier.bands[idx].size;
    if blow >= size {
        sim.frontier.bands.remove(idx);
        Some((name, true, 0))
    } else {
        let remaining = size - blow;
        sim.frontier.bands[idx].size = remaining;
        Some((name, false, remaining))
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

#[cfg(test)]
mod danger_tests {
    use super::*;
    use crate::sim::beasts::WildBeast;
    use crate::sim::SimState;

    fn make_sim() -> SimState {
        SimState::new(7, crate::charts::load_charts().unwrap())
    }

    #[test]
    fn a_band_falls_on_a_caravan_crossing_its_country() {
        // #641 slice 4: a band that holds country a caravan is crossing rides it
        // down — its goods carried off, the train left limping.
        use crate::model::economy::Caravan;
        let mut sim = make_sim();
        let ri = sim
            .world
            .regions
            .iter()
            .position(|r| r.settlements.len() >= 2)
            .expect("a region with two towns");
        let (o_name, d_name) = {
            let r = &sim.world.regions[ri];
            (r.settlements[0].name.clone(), r.settlements[1].name.clone())
        };
        // A strong band holds the country the caravan crosses.
        sim.frontier.bands.push(Band {
            id: "band-road-1".into(),
            name: "the Toll of the Reach".into(),
            size: 20,
            region_idx: ri,
            formed_day: 0,
        });
        sim.caravans.push(Caravan {
            id: "carav-prey-1".into(),
            origin: o_name,
            destination: d_name,
            goods: vec![(crate::model::ItemType::Iron, 5)],
            departure_tick: 0,
            arrival_tick: 100_000, // long road, so it stays in transit
            travel_cost: 0,
            raided: false,
        });

        // Over a handful of days the band takes it (a deterministic roll a day).
        for d in 1..=30u64 {
            sim.world.tick = d * 24;
            prey_on_caravans(&mut sim);
            if sim.caravans[0].raided {
                break;
            }
        }
        assert!(sim.caravans[0].raided, "the band rode the caravan down");
        assert!(
            sim.caravans[0].goods.is_empty(),
            "its goods are carried off"
        );
    }

    #[test]
    fn a_quiet_settled_region_reads_settled() {
        let mut sim = make_sim();
        // A region with a living town and nothing on its ground.
        let ri = sim
            .world
            .regions
            .iter()
            .position(|r| !r.is_march && r.settlements.iter().any(|s| s.population > 0))
            .expect("a settled region");
        sim.world.regions[ri].is_march = false;
        sim.frontier.bands.retain(|b| b.region_idx != ri);
        sim.beasts.retain(|b| b.region_idx != ri);
        sim.world.regions[ri].game_richness = 1.0;
        assert_eq!(region_danger(&sim, ri), DangerTier::Settled);
    }

    #[test]
    fn the_march_reads_wary_empty_and_climbs_with_what_stands_on_it() {
        let mut sim = make_sim();
        let ri = 0;
        sim.world.regions[ri].is_march = true;
        sim.world.regions[ri].game_richness = 1.0;
        sim.frontier.bands.retain(|b| b.region_idx != ri);
        sim.beasts.retain(|b| b.region_idx != ri);
        // Empty march: wild ground, but nothing on it yet.
        assert_eq!(region_danger(&sim, ri), DangerTier::Wary);

        // A band rides in — the country turns perilous.
        sim.frontier.bands.push(Band {
            id: "band-danger-1".into(),
            name: "the Grey".into(),
            size: 5,
            region_idx: ri,
            formed_day: 0,
        });
        assert_eq!(region_danger(&sim, ri), DangerTier::Perilous);

        // And with the dreads of the deep wild thick on it, deadly.
        for n in 0..4 {
            sim.beasts.push(WildBeast {
                id: format!("beast-danger-{n}"),
                species: crate::model::wildlife::WildSpecies::HollowStag,
                region_idx: ri,
                px: n,
                py: 0,
                hp: 4,
            });
        }
        assert_eq!(region_danger(&sim, ri), DangerTier::Deadly);
    }
}
