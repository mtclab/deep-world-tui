//! God-vows (#457 item 4): the deepest reach of faith. A devotee who has come
//! to **Blessed** standing with one of the Five may swear a vow to that god at
//! a temple. A vow is not a stat-nudge: it grants a lasting **boon** that runs
//! while it is held, at a real **cost** — to swear one god you forsake the rest
//! (their regard cools), and the vow **binds**: let the kept god slip below
//! Devoted and it breaks, the grace withdrawn and the lapse marked. The player
//! always chooses it; the gods never coerce.

use super::App;
use crate::model::GodName;

/// The affinity at which a god's grace runs deep enough to swear a vow.
const BLESSED: f64 = 0.85;
/// Below this, a sworn vow lapses — devotion neglected breaks the binding.
const DEVOTED: f64 = 0.60;

/// The boon a god's vow grants, in the player's own words.
fn vow_boon_line(god: GodName) -> &'static str {
    match god {
        GodName::Oltzed => "your hands do not tire at the work — labor costs you the less",
        GodName::Keuru => "you are welcomed at every hearth — your rest restores the deeper",
        GodName::Sampsa => "you read the road before it turns — danger finds you less often",
        GodName::Masa => "the scales fall true for you — every trade goes the kinder",
        GodName::Kukri => "the patient cold cannot wear you — harsh weather bites the less",
    }
}

impl App {
    /// The god the player is vowed to, if the vow still holds (#457).
    pub fn vow_god(&self) -> Option<GodName> {
        self.god_vow
    }

    /// Multiplier on what a market trade costs/pays the player while the Masa
    /// vow is held (#457): goods bought a little cheaper. `1.0` otherwise.
    pub fn vow_buy_mult(&self) -> f64 {
        if self.god_vow == Some(GodName::Masa) {
            0.92
        } else {
            1.0
        }
    }

    /// Multiplier on the energy a piece of work costs while the Oltzed vow is
    /// held (#457): the hands do not tire. `1.0` otherwise.
    pub fn vow_work_energy_mult(&self) -> f64 {
        if self.god_vow == Some(GodName::Oltzed) {
            0.55
        } else {
            1.0
        }
    }

    /// Multiplier on the encounter rate while the Sampsa vow is held (#457):
    /// the road is read before it turns. `1.0` otherwise.
    pub fn vow_encounter_mult(&self) -> f64 {
        if self.god_vow == Some(GodName::Sampsa) {
            0.70
        } else {
            1.0
        }
    }

    /// Multiplier on the harsh-weather wear while the Kukri vow is held (#457):
    /// the patient cold cannot wear the sworn. `1.0` otherwise.
    pub fn vow_weather_mult(&self) -> f64 {
        if self.god_vow == Some(GodName::Kukri) {
            0.70
        } else {
            1.0
        }
    }

    /// Bonus factor on rest's energy restoration while the Keuru vow is held
    /// (#457): welcomed at every hearth. `1.0` otherwise (i.e. no bonus).
    pub fn vow_rest_bonus(&self) -> f64 {
        if self.god_vow == Some(GodName::Keuru) {
            1.5
        } else {
            1.0
        }
    }

    /// The god whose vow the player could swear here and now (#457): the one
    /// they keep at Blessed, when standing at a temple and not already vowed.
    pub fn vowable_god(&self) -> Option<GodName> {
        if self.god_vow.is_some() {
            return None;
        }
        let at_temple = self.current_settlement().is_some_and(|s| {
            s.services
                .contains(&crate::model::SettlementService::Temple)
                || s.services
                    .contains(&crate::model::SettlementService::Shrine)
        });
        if !at_temple {
            return None;
        }
        let god = self.god_affinity.strongest_ally()?;
        if self.god_affinity.get(god) >= BLESSED {
            Some(god)
        } else {
            None
        }
    }

    /// Swear (or renounce) a god-vow at a temple (#457). With a vow held this
    /// renounces it; otherwise it swears one to the Blessed god kept here.
    pub fn take_vow(&mut self) {
        if let Some(broken) = self.god_vow {
            // Renouncing a vow is no small thing — the forsaken god is colder
            // for it, and the journal keeps the turning.
            self.god_vow = None;
            self.god_affinity.adjust(broken, -0.10);
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Faith,
                    "I gave back the vow I swore. The grace withdrew, and the keeping is mine to begin again.".to_string(),
                );
            }
            self.status_msg =
                Some("You renounce your vow. The god's grace withdraws from you.".into());
            return;
        }
        let Some(god) = self.vowable_god() else {
            self.status_msg = Some(
                "A vow is sworn at a temple, by one the god has Blessed — a deep devotion first."
                    .into(),
            );
            return;
        };
        self.god_vow = Some(god);
        // The cost: to swear one of the Five is to forsake the other four, and
        // their regard cools for it. Faith chosen is faith spent elsewhere.
        for other in [
            GodName::Oltzed,
            GodName::Keuru,
            GodName::Sampsa,
            GodName::Masa,
            GodName::Kukri,
        ] {
            if other != god {
                self.god_affinity.adjust(other, -0.05);
            }
        }
        let boon = vow_boon_line(god);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Faith,
                format!(
                    "I swore myself to the god I keep, and forsook the rest. Now {boon}. The binding is mine to keep."
                ),
            );
        }
        self.status_msg = Some(format!("You swear the vow — now {boon}. (V to renounce)"));
    }

    /// Check a held vow against its binding (#457): if the kept god has slipped
    /// below Devoted, the vow breaks — the grace withdrawn, the lapse marked.
    /// Called as devotion can only fall over time (the daily reckoning).
    pub fn check_vow(&mut self) {
        if let Some(god) = self.god_vow {
            if self.god_affinity.get(god) < DEVOTED {
                self.god_vow = None;
                if let Some(ref mut sim) = self.sim {
                    let tick = sim.world.tick;
                    sim.log(
                        tick,
                        crate::sim::journal::Voice::Faith,
                        "The vow I let lapse has broken of itself. I kept the god too thinly, and the grace is gone.".to_string(),
                    );
                }
                self.status_msg =
                    Some("Your vow has broken — the god you kept too thinly withdraws.".into());
            }
        }
    }
}
