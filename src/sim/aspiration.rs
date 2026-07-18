//! Purposeful agents — aspirations pursued over a life (deep-world-godot#53).
//!
//! A needs-driven agent keeps itself alive; a *person* also wants things and
//! works toward them across years. When a soul's needs are met (it is not in
//! survival crisis), it pursues a standing **aspiration** — to master a trade, to
//! marry — a multi-day project advanced a little at a time, resolving into a real
//! life event. From these, life stories emerge: "a weaver's lad who apprenticed
//! to the smith and wed the trader's daughter."
//!
//! This is a daily settlement pass, separate from the needs selector. O(n): the
//! cross-roster facts (which trades have a master to learn from, who is eligible
//! to marry) are gathered once per settlement; each soul advances its own goal.
//! Deterministic — roster order, seeded thresholds, no wall-clock.

use crate::model::economy::Settlement;
use crate::model::relation::{InterNpcRelation, RelationKind};
use crate::model::Person;
use crate::rng::SeedRng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A standing life-goal. Progress runs 0.0 → 1.0; on completion the aspiration
/// resolves (a trade learned, a marriage made) and clears.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Aspiration {
    /// Apprentice to a master of `trade` and, in time, become one.
    LearnTrade { trade: String, progress: f64 },
    /// Court an eligible neighbour and wed.
    Marry { progress: f64 },
}

impl Aspiration {
    /// A short read of what a soul is working toward, for the dialogue panel.
    pub fn label(&self) -> String {
        match self {
            Aspiration::LearnTrade { trade, .. } => format!("learning the {trade}'s trade"),
            Aspiration::Marry { .. } => "looking to marry".into(),
        }
    }
}

/// The skilled trades a youth might aspire to master (the ones worth an
/// apprenticeship). Mirrors lifecycle's notion of a trade worth passing on.
const SKILLED_TRADES: &[&str] = &[
    "smith",
    "weaver",
    "carpenter",
    "healer",
    "scribe",
    "potter",
    "brewer",
    "mason",
    "tanner",
    "miner",
    "fisher",
    "trader",
];

fn is_marriageable(p: &Person) -> bool {
    !p.has_spouse && matches!(p.age_band.as_str(), "youth" | "adult")
}

/// Settled enough to have ambitions — not in a survival crisis. A soul that is
/// hungry or unsafe tends to that first (the needs selector); only a fed,
/// reasonably secure life looks to the longer road.
fn settled_enough(p: &Person) -> bool {
    use crate::model::Need;
    p.needs.get(Need::Food) > 0.5 && p.needs.get(Need::Safety) > 0.5
}

/// Advance every resident's life-aspiration one day. Returns the life events
/// (trade mastered, marriage made) for the caller to put on the road.
pub fn tick_settlement_aspirations(s: &mut Settlement, seed: u64, tick: u64) -> Vec<String> {
    let mut events: Vec<String> = Vec::new();
    let n = s.people.len();
    if n == 0 {
        return events;
    }
    // The trades that have a living master here to learn from.
    let trades_present: HashSet<String> = s.people.iter().map(|p| p.profession.clone()).collect();
    // Belief colours the person (belief-colours-choices #56-H): where a temple or
    // shrine stands, the devout give of what they have — a coin of alms to the
    // commons. Culture shaping the economy, soul by soul. Coins tithed this pass
    // are added to the treasury after the loop (it is read elsewhere here).
    use crate::model::economy::SettlementService;
    let has_temple = s
        .services
        .iter()
        .any(|sv| matches!(sv, SettlementService::Temple | SettlementService::Shrine));
    let mut tithes: u32 = 0;
    // The unwed of marriageable years, in roster order — the pool courtship draws
    // a match from. First-come pairing keeps it deterministic.
    let mut eligible: Vec<usize> = (0..n).filter(|&i| is_marriageable(&s.people[i])).collect();
    // Marriages decided this pass (both partners), applied after the loop so a
    // soul is not paired twice.
    let mut weddings: Vec<(usize, usize)> = Vec::new();
    let mut claimed: HashSet<usize> = HashSet::new();

    for i in 0..n {
        if !settled_enough(&s.people[i]) {
            continue;
        }
        // A devout soul, settled and with a little to spare, gives a coin of alms
        // where there is a temple to give it to (#56-H).
        if has_temple
            && s.people[i].coins >= 3
            && s.people[i].personality.iter().any(|t| t == "devout")
        {
            s.people[i].coins -= 1;
            tithes += 1;
        }
        // Assign an aspiration to a settled soul that has none, by its station.
        if s.people[i].aspiration.is_none() {
            let p = &s.people[i];
            let band = p.age_band.as_str();
            let unskilled = !SKILLED_TRADES.contains(&p.profession.as_str());
            let rng = SeedRng::new(seed)
                .fork_for(&format!("aspire:{}:{}", p.id, tick / 24))
                .next_u64();
            let pick = if matches!(band, "youth") && unskilled {
                // a youth without a trade looks to learn one present in town
                let masters: Vec<&String> = trades_present
                    .iter()
                    .filter(|t| SKILLED_TRADES.contains(&t.as_str()))
                    .collect();
                if !masters.is_empty() {
                    // Modulo in u64 before narrowing: usize is 32-bit on wasm32.
                    let t = masters[(rng % masters.len() as u64) as usize].clone();
                    Some(Aspiration::LearnTrade {
                        trade: t,
                        progress: 0.0,
                    })
                } else {
                    None
                }
            } else if is_marriageable(p) {
                Some(Aspiration::Marry { progress: 0.0 })
            } else {
                None
            };
            s.people[i].aspiration = pick;
        }

        // Advance whatever the soul is pursuing.
        match s.people[i].aspiration.clone() {
            Some(Aspiration::LearnTrade { trade, progress }) => {
                if trades_present.contains(&trade) {
                    let np = progress + 0.05; // ~20 settled days to master
                    if np >= 1.0 {
                        s.people[i].profession = trade.clone();
                        s.people[i].aspiration = None;
                        events.push(format!(
                            "{} of {} has come into the {trade}'s trade.",
                            s.people[i].name, s.name
                        ));
                    } else {
                        s.people[i].aspiration = Some(Aspiration::LearnTrade {
                            trade,
                            progress: np,
                        });
                    }
                }
                // No master left in town: the pursuit stalls (no progress) until
                // one appears or the soul gives up (handled by reassignment when
                // cleared elsewhere).
            }
            Some(Aspiration::Marry { progress }) => {
                let np = progress + 0.05;
                if np >= 1.0 {
                    // Court the first eligible neighbour not already spoken for.
                    if let Some(&j) = eligible
                        .iter()
                        .find(|&&j| j != i && !claimed.contains(&j) && !claimed.contains(&i))
                    {
                        claimed.insert(i);
                        claimed.insert(j);
                        weddings.push((i, j));
                    } else {
                        // no match to be had yet — keep courting
                        s.people[i].aspiration = Some(Aspiration::Marry { progress: 0.6 });
                    }
                } else {
                    s.people[i].aspiration = Some(Aspiration::Marry { progress: np });
                }
            }
            None => {}
        }
    }

    // Make the marriages: both wed, each holds a Spouse bond, ambitions met.
    for (a, b) in weddings {
        let (aid, bid) = (s.people[a].id.clone(), s.people[b].id.clone());
        wed(&mut s.people[a], &bid, tick);
        wed(&mut s.people[b], &aid, tick);
        events.push(format!(
            "{} and {} of {} were wed.",
            s.people[a].name, s.people[b].name, s.name
        ));
    }
    // A wed soul leaves the eligible pool (defensive — pairing already guarded).
    eligible.retain(|i| !claimed.contains(i));
    // The day's alms swell the temple's common purse (#56-H).
    s.treasury = s.treasury.saturating_add(tithes);
    events
}

fn wed(p: &mut Person, spouse_id: &str, tick: u64) {
    p.has_spouse = true;
    p.aspiration = None;
    if !p
        .relations
        .iter()
        .any(|r| r.target_person_id == spouse_id && r.kind == RelationKind::Spouse)
    {
        p.relations.push(InterNpcRelation {
            kind: RelationKind::Spouse,
            target_person_id: spouse_id.to_string(),
            intensity: 0.8,
            formed_at_tick: tick,
            reason: "wed".into(),
        });
    }
}
