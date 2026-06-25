//! Entity-first agency: the utility-driven needs selector (slice 8,
//! deep-world-godot#50).
//!
//! Each resident, each settlement tick, scores its drives and acts on the **most
//! pressing unmet** one with the best **affordable** action — not a single
//! hard-coded hunger pipe. The five drives (Food, Money, Care, Presence, Safety)
//! are weighted Maslow-style: survival (Food, Safety) outweighs care and company;
//! Money is *instrumental* (you earn to eat), so it never wins on its own — it is
//! served inside the Food column's "take work" rung.
//!
//! The hunger ladder (slices 3-5) is now just the Food column, generalised: it
//! gains foraging off the land and begging from a neighbour, and its bottom rung
//! — leaving — forks by **disposition** into lawful migration to a fed town or a
//! turn to banditry. Two equally desperate souls diverge by character.
//!
//! Determinism: residents act in roster order, all amounts fixed, the only
//! "randomness" is the agent's own seeded traits. O(n): every cross-roster fact
//! (the richest neighbour, whether the town has a healer/company/shelter, a
//! reachable fed town) is precomputed once into [`TownContext`].

use crate::model::economy::{BuildingType, Settlement, SettlementService};
use crate::model::Need;

/// Per-settlement facts the per-agent decision needs, gathered once so the agent
/// loop stays O(1) per soul.
pub struct TownContext {
    pub ration: f64,
    pub food_price: u32,
    pub wage: u32,
    /// The region's wild plenty (game_richness): high land feeds a forager.
    pub region_richness: f64,
    /// A healer lives here (the Care column can be served).
    pub has_healer: bool,
    /// A tavern or temple — somewhere to find company (Presence).
    pub has_company: bool,
    /// A built shelter to weather a threat behind (Safety).
    pub has_shelter: bool,
    /// The land is dangerous now (beasts/raiders abroad) — Safety is pressed.
    pub under_threat: bool,
    /// A reachable, well-fed settlement in the region to flee or migrate to.
    /// `None` means there is nowhere better — leaving can only mean the road.
    pub migrate_target: Option<usize>,
}

/// How a soul that left this settlement left it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Departure {
    /// Took to the open country — feeds the frontier's bands.
    Bandit,
    /// Sought a better town — joins `to` (a settlement index in this region).
    Migrate { to: usize },
}

/// Does this soul, pushed to the wall, turn to crime rather than seek a lawful
/// way out? Read from disposition (`personality`): the hard and bitter take what
/// they can; the loyal and gentle would sooner move on or go without. Net of the
/// two leanings; ties fall on the lawful side. Deterministic.
fn turns_to_crime(personality: &[String]) -> bool {
    let mut score: i32 = 0;
    for t in personality {
        match t.as_str() {
            "devious" | "reckless" | "bitter" | "mercenary" | "ruthless" | "suspicious"
            | "sharp" => score += 1,
            "loyal" | "earnest" | "gentle" | "warm" | "cautious" | "devout" | "proud" => score -= 1,
            _ => {}
        }
    }
    score > 0
}

/// The single most-pressing unmet drive, weighted by survival priority. Returns
/// `None` when the soul is content (every drive comfortable) — it simply lives.
/// Money is deliberately absent: it is instrumental, earned to buy food, so it
/// is served inside the Food column rather than competing as a terminal drive.
fn most_pressing(p: &crate::model::Person) -> Option<Need> {
    // Only a drive below the comfort line competes; weight makes survival win.
    let urg = |need: Need, w: f64| -> f64 {
        let v = p.needs.get(need);
        if v >= 0.7 {
            0.0
        } else {
            w * (0.7 - v)
        }
    };
    let candidates = [
        (Need::Food, urg(Need::Food, 1.0)),
        (Need::Safety, urg(Need::Safety, 0.9)),
        (Need::Care, urg(Need::Care, 0.7)),
        (Need::Presence, urg(Need::Presence, 0.5)),
    ];
    candidates
        .into_iter()
        .filter(|(_, u)| *u > 0.0)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(need, _)| need)
}

/// Run one decision for every resident. Mutates the granary, the treasury, and
/// each soul's purse and needs in place; returns the leavers — `(roster index,
/// how they left)` — for the caller to remove and route (migrants into the
/// destination roster, bandits into the frontier). Returns the food drawn from
/// the granary so the caller can account consumption.
pub fn step_agents(s: &mut Settlement, ctx: &TownContext) -> (Vec<(usize, Departure)>, f64) {
    let mut stock = s.food_stock;
    let mut treasury = s.treasury;
    let mut eaten = 0.0;
    // The wealthiest neighbour's purse is the pool that charity and theft draw
    // from; the transfer is settled after the loop so we never alias two souls.
    let richest = s
        .people
        .iter()
        .enumerate()
        .max_by_key(|(_, p)| p.coins)
        .map(|(i, p)| (i, p.coins));
    let (richest_idx, mut richest_left) = match richest {
        Some((i, c)) => (Some(i), c),
        None => (None, 0),
    };
    let mut taken_from_richest: u32 = 0;
    let mut departures: Vec<(usize, Departure)> = Vec::new();

    let leave = |personality: &[String]| -> Departure {
        // Lawful flight if there is somewhere better and the soul is not
        // criminally inclined; otherwise the road.
        match ctx.migrate_target {
            Some(to) if !turns_to_crime(personality) => Departure::Migrate { to },
            _ => Departure::Bandit,
        }
    };

    for i in 0..s.people.len() {
        let need = match most_pressing(&s.people[i]) {
            Some(n) => n,
            None => continue, // content — just lives
        };
        match need {
            Need::Food => {
                // --- the Food column: eat now, else get the means, else leave ---
                // (Foraging/hunting is not a separate rung here: the land's yield
                // is already in the granary — the settlement's farms, gathering,
                // and trapping fill `food_stock` before this. A free per-agent
                // forage would double-count it and let a province-scale town
                // ignore the hinterland limit that drives famine and trade.)
                if stock > 0.0 {
                    // Eat the daily ration; the last mouths split what is left so
                    // the granary empties cleanly to 0 and the famine trigger
                    // (food_stock <= 0) still fires.
                    let bite = ctx.ration.min(stock);
                    stock -= bite;
                    eaten += bite;
                    s.people[i]
                        .needs
                        .satisfy(Need::Food, 0.10 * (bite / ctx.ration).min(1.0));
                } else if s.people[i].coins >= ctx.food_price {
                    s.people[i].coins -= ctx.food_price;
                    treasury = treasury.saturating_add(ctx.food_price);
                    s.people[i].needs.satisfy(Need::Food, 0.10);
                } else if treasury >= ctx.wage {
                    // take work — earn coin to buy next time (Money served here)
                    treasury -= ctx.wage;
                    s.people[i].coins = s.people[i].coins.saturating_add(ctx.wage);
                    s.people[i].needs.satisfy(Need::Money, 0.10);
                    s.people[i].needs.decay(Need::Food, 0.05);
                } else if richest_idx.is_some_and(|r| r != i) && richest_left >= 1 {
                    // beg a coin from the well-off neighbour (charity, no crime)
                    richest_left -= 1;
                    taken_from_richest += 1;
                    s.people[i].coins = s.people[i].coins.saturating_add(1);
                    s.people[i].needs.decay(Need::Food, 0.05);
                } else if richest_idx.is_some_and(|r| r != i)
                    && richest_left >= ctx.food_price
                    && turns_to_crime(&s.people[i].personality)
                {
                    // steal from the neighbour — the crime rung
                    let amt = ctx.food_price.max(3);
                    let amt = amt.min(richest_left);
                    richest_left -= amt;
                    taken_from_richest += amt;
                    s.people[i].coins = s.people[i].coins.saturating_add(amt);
                } else if s.people[i].needs.get(Need::Food) < 0.1 {
                    // truly starving, every option spent — leave
                    let d = leave(&s.people[i].personality);
                    departures.push((i, d));
                } else {
                    s.people[i].needs.decay(Need::Food, 0.05);
                }
            }
            Need::Safety => {
                if !ctx.under_threat {
                    s.people[i].needs.satisfy(Need::Safety, 0.10);
                } else if ctx.has_shelter {
                    s.people[i].needs.satisfy(Need::Safety, 0.08);
                } else if s.people[i].needs.get(Need::Safety) < 0.15 {
                    // no wall to hide behind and the dark is close — flee
                    let d = leave(&s.people[i].personality);
                    departures.push((i, d));
                } else {
                    s.people[i].needs.decay(Need::Safety, 0.04);
                }
            }
            Need::Care => {
                if ctx.has_healer {
                    s.people[i].needs.satisfy(Need::Care, 0.10);
                } else {
                    s.people[i].needs.decay(Need::Care, 0.03);
                }
            }
            Need::Presence => {
                if ctx.has_company || s.people[i].has_spouse {
                    s.people[i].needs.satisfy(Need::Presence, 0.10);
                } else {
                    s.people[i].needs.decay(Need::Presence, 0.03);
                }
            }
            Need::Money => {}
        }
    }

    // Settle the charity/theft pool against the one neighbour it came from.
    if let Some(r) = richest_idx {
        let r_coins = &mut s.people[r].coins;
        *r_coins = r_coins.saturating_sub(taken_from_richest);
    }
    s.food_stock = stock;
    s.treasury = treasury;
    (departures, eaten)
}

/// Build the per-settlement context from the town and its region. `has_*` read
/// the town's services/buildings/professions; `migrate_target` is the best-fed
/// *other* settlement in the region (a place worth fleeing to), precomputed by
/// the caller and passed through. Region facts (richness, threat) come from the
/// caller too, which holds the region borrow.
pub fn town_context(
    s: &Settlement,
    region_richness: f64,
    under_threat: bool,
    migrate_target: Option<usize>,
    ration: f64,
) -> TownContext {
    let has_healer = s.profession_count("healer") > 0 || s.profession_count("priest") > 0;
    let has_company = s.services.contains(&SettlementService::Tavern)
        || s.services.contains(&SettlementService::Temple);
    let has_shelter = s.has_building(BuildingType::Shelter);
    TownContext {
        ration,
        food_price: 1,
        wage: 1,
        region_richness,
        has_healer,
        has_company,
        has_shelter,
        under_threat,
        migrate_target,
    }
}
