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
//! The hunger ladder (slices 3-5) is now just the Food column, generalised. Its
//! bottom rung — leaving — forks by **disposition** into lawful migration to a
//! fed town or a turn to banditry; two equally desperate souls diverge by
//! character.
//!
//! Slice 9 runs need-satisfaction through the **social fabric**, and makes health
//! real: a soul begs of its kin and sworn friends before it would rob a stranger,
//! and the giving deepens the bond; one with a hard heart and no kin to ask robs
//! a well-off stranger, and the robbed remember it as a feud — so scarcity sows
//! the province's grudges. A healer does not just reassure: it tends the worst
//! illness, easing it and hastening recovery (#449).
//!
//! Determinism: residents act in roster order, all amounts fixed, the only
//! "randomness" is the agent's own seeded traits. O(n): every cross-roster fact
//! (the richest neighbour, the town's healer/company/shelter, a reachable fed
//! town) is precomputed once; coin moves through a flat purse array so a giver
//! and taker are never aliased; bonds and feuds are settled after the loop.

use crate::model::economy::{BuildingType, Settlement, SettlementService};
use crate::model::relation::{InterNpcRelation, RelationKind};
use crate::model::Need;
use std::collections::HashMap;

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
    /// The current tick — stamps any bond or feud the day's acts create.
    pub tick: u64,
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
    let n = s.people.len();
    let mut stock = s.food_stock;
    let mut treasury = s.treasury;
    let mut eaten = 0.0;
    // Coin lives in a flat array for the transactional part so a giver and a
    // taker are never two aliased borrows of `people`; written back at the end.
    let mut purses: Vec<u32> = s.people.iter().map(|p| p.coins).collect();
    // id -> roster index (first-seen order = deterministic), to resolve a soul's
    // kin and friends to the neighbours actually present.
    let mut idx_of: HashMap<String, usize> = HashMap::with_capacity(n);
    for (i, p) in s.people.iter().enumerate() {
        idx_of.entry(p.id.clone()).or_insert(i);
    }
    // The wealthiest neighbour — the mark a thief robs (if not their own kin).
    let richest = purses
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| **c)
        .map(|(i, c)| (i, *c));
    let mut departures: Vec<(usize, Departure)> = Vec::new();
    // Social consequences of the day's acts, applied after the loop: charity
    // deepens a bond (gratitude), theft sows a feud (resentment).
    let mut gratitude: Vec<(usize, usize)> = Vec::new(); // (beggar, benefactor)
    let mut new_feuds: Vec<(usize, usize)> = Vec::new(); // (victim, thief)

    let leave = |personality: &[String]| -> Departure {
        match ctx.migrate_target {
            Some(to) if !turns_to_crime(personality) => Departure::Migrate { to },
            _ => Departure::Bandit,
        }
    };

    for i in 0..n {
        let need = match most_pressing(&s.people[i]) {
            Some(nd) => nd,
            None => continue, // content — just lives
        };
        match need {
            Need::Food => {
                // The Food column: eat now, else get the means, else leave.
                // (Foraging is already in the granary — the town's farms and
                // gathering fill `food_stock` before this; a free per-agent
                // forage would double-count it and break the hinterland limit
                // that drives famine and trade.)
                if stock > 0.0 {
                    let bite = ctx.ration.min(stock);
                    stock -= bite;
                    eaten += bite;
                    s.people[i]
                        .needs
                        .satisfy(Need::Food, 0.10 * (bite / ctx.ration).min(1.0));
                } else if purses[i] >= ctx.food_price {
                    purses[i] -= ctx.food_price;
                    treasury = treasury.saturating_add(ctx.food_price);
                    s.people[i].needs.satisfy(Need::Food, 0.10);
                } else if treasury >= ctx.wage {
                    treasury -= ctx.wage;
                    purses[i] = purses[i].saturating_add(ctx.wage);
                    s.people[i].needs.satisfy(Need::Money, 0.10);
                    s.people[i].needs.decay(Need::Food, 0.05);
                } else if let Some(b) = bonded_benefactor(&s.people[i], &idx_of, &purses, i) {
                    // beg of kin or a sworn friend — they share what they have,
                    // and the giving deepens the tie
                    purses[b] -= 1;
                    purses[i] = purses[i].saturating_add(1);
                    s.people[i].needs.decay(Need::Food, 0.05);
                    gratitude.push((i, b));
                } else if turns_to_crime(&s.people[i].personality)
                    && richest.is_some_and(|(r, c)| {
                        r != i
                            && c >= ctx.food_price
                            && !has_bond_with(&s.people[i], &s.people[r].id)
                    })
                {
                    // steal from a well-off stranger — never from one's own kin —
                    // and the robbed remember it
                    let (r, _) = richest.unwrap();
                    let amt = ctx.food_price.max(3).min(purses[r]);
                    purses[r] -= amt;
                    purses[i] = purses[i].saturating_add(amt);
                    new_feuds.push((r, i));
                } else if s.people[i].needs.get(Need::Food) < 0.1 {
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
                    let d = leave(&s.people[i].personality);
                    departures.push((i, d));
                } else {
                    s.people[i].needs.decay(Need::Safety, 0.04);
                }
            }
            Need::Care => {
                if ctx.has_healer {
                    // A healer actually tends the worst illness — easing it and
                    // hastening recovery (#449 illness) — not just a number.
                    if let Some(worst) = s.people[i].illnesses.iter_mut().max_by(|a, b| {
                        a.severity
                            .partial_cmp(&b.severity)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }) {
                        worst.tend();
                    }
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

    // Write the purses back and settle the social ledger.
    for (i, p) in s.people.iter_mut().enumerate() {
        p.coins = purses[i];
    }
    for (beggar, benefactor) in gratitude {
        let bid = s.people[benefactor].id.clone();
        deepen_bond(&mut s.people[beggar], &bid);
    }
    for (victim, thief) in new_feuds {
        let tid = s.people[thief].id.clone();
        sow_feud(&mut s.people[victim], &tid, ctx.tick);
    }
    s.food_stock = stock;
    s.treasury = treasury;
    (departures, eaten)
}

/// A kin- or friend-neighbour of `i`, present in town and with a coin to spare —
/// the one a soul begs of before it would rob a stranger. First match in the
/// soul's own relation order (deterministic). `None` if it has no one to turn to.
fn bonded_benefactor(
    p: &crate::model::Person,
    idx_of: &HashMap<String, usize>,
    purses: &[u32],
    self_idx: usize,
) -> Option<usize> {
    for r in &p.relations {
        if !r.kind.is_bond() {
            continue;
        }
        if let Some(&b) = idx_of.get(&r.target_person_id) {
            if b != self_idx && purses[b] >= 1 {
                return Some(b);
            }
        }
    }
    None
}

/// Does this soul hold a warm tie to the person with id `other`?
fn has_bond_with(p: &crate::model::Person, other: &str) -> bool {
    p.relations
        .iter()
        .any(|r| r.kind.is_bond() && r.target_person_id == other)
}

/// Charity received deepens the receiver's bond to the giver (gratitude). Only an
/// existing bond is strengthened — generosity is remembered, not invented.
fn deepen_bond(p: &mut crate::model::Person, benefactor_id: &str) {
    if let Some(r) = p
        .relations
        .iter_mut()
        .find(|r| r.kind.is_bond() && r.target_person_id == benefactor_id)
    {
        r.intensity = (r.intensity + 0.05).min(1.0);
    }
}

/// Being robbed sows a feud toward the thief — unless a tie to them already
/// exists (we do not overwrite kinship). Scarcity breeds the province's feuds.
fn sow_feud(victim: &mut crate::model::Person, thief_id: &str, tick: u64) {
    if victim
        .relations
        .iter()
        .any(|r| r.target_person_id == thief_id)
    {
        return;
    }
    victim.relations.push(InterNpcRelation {
        kind: RelationKind::Feud,
        target_person_id: thief_id.to_string(),
        intensity: 0.35,
        formed_at_tick: tick,
        reason: "robbed in a hungry season".into(),
    });
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
    tick: u64,
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
        tick,
    }
}
