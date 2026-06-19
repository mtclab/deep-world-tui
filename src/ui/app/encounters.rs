use crate::model::{Collapse, CollapseOutcome, DeathCause, ItemType, PeopleKind};
use crate::save::{self, SaveData};
use crate::save_migrations;
use crate::sim::collapse_log::CollapseEvent;
use crate::sim::hints;
use crate::sim::SimState;

use super::*;

impl App {
    /// Open the faith ledger (#457): where you stand with each of the Five.
    pub fn open_faith(&mut self) {
        self.previous_screen = Some(self.screen.clone());
        self.screen = Screen::Faith { scroll: 0 };
    }

    pub fn exit_faith(&mut self) {
        self.screen = self
            .previous_screen
            .clone()
            .unwrap_or_else(|| self.world_screen());
    }

    /// The encounter-rate multiplier from inter-people tension on the road
    /// (#9): travelling the country of a people you are deeply at odds with,
    /// their lawless harry you — the escalation ladder reaches the wilds.
    /// 1.0 at peace; up to 1.6 under a deep grudge. Neutral toward your own.
    pub fn road_tension_mult(&self) -> f64 {
        let region_people = self.player_pos.and_then(|pos| {
            let sim = self.sim.as_ref()?;
            let dom = sim
                .world
                .regions
                .get(pos.region_idx)?
                .settlements
                .first()?
                .people
                .first()?;
            Some(crate::model::PeopleKind::from_name(&dom.people))
        });
        region_people.map_or(1.0, |rp| {
            if rp == self.inter_people_bias.player_people {
                return 1.0;
            }
            match self.inter_people_bias.effective_bias(rp) {
                b if b < -0.15 => 1.6,
                b if b < -0.05 => 1.2,
                _ => 1.0,
            }
        })
    }

    /// Encounter-rate multiplier from the polity's war (#579 slice 1): while the
    /// province's polity and its rival are at tension the roads are watched and
    /// raided, so the traveller is found more often. 1.0 in peace. Deterministic
    /// per season, on the same clock the war rumor and the levy use.
    pub fn polity_war_mult(&self) -> f64 {
        let Some(sim) = self.sim.as_ref() else {
            return 1.0;
        };
        let day = self.clock.day;
        let season_ord = (day / 30) % 4;
        let year = day / 120;
        if sim.world.polity.in_tension(self.seed, season_ord, year) {
            1.25
        } else {
            1.0
        }
    }

    pub fn check_discovery(&mut self, region_idx: usize, px: usize, py: usize) {
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let px_u32 = px as u32;
        let py_u32 = py as u32;
        let mut found_kind = None;
        if let Some(ref mut sim) = self.sim {
            let disc_id = match sim.discoveries.at_position(region_idx, px_u32, py_u32) {
                Some(d) => d.id.clone(),
                None => return,
            };
            if sim.discoveries.observe(&disc_id, tick, player_id) {
                let kind = sim
                    .discoveries
                    .entries
                    .iter()
                    .find(|d| d.id == disc_id)
                    .map(|d| d.kind);
                if let Some(kind) = kind {
                    sim.log_journal(tick, kind.observe_text().to_string());
                    self.status_msg = Some(format!("I found a {}!", kind.label()));
                    found_kind = Some(kind);
                }
            }
        }
        // Finding a place leaves a mark — discoveries were pure lore before.
        if let Some(kind) = found_kind {
            match kind.observe_effect() {
                crate::model::discovery::DiscoveryEffect::God(god, delta) => {
                    self.god_affinity.adjust(god, delta);
                }
                crate::model::discovery::DiscoveryEffect::Refresh { thirst, energy } => {
                    self.vitals.thirst = (self.vitals.thirst + thirst).min(1.0);
                    self.vitals.energy = (self.vitals.energy + energy).min(1.0);
                }
                crate::model::discovery::DiscoveryEffect::Reveal => {
                    // The land makes sense from here: see twice as far once.
                    self.reveal_around(region_idx, px, py);
                    let wider = crate::model::ExploredMap::reveal_radius_for_elder(self.elder) * 2;
                    if region_idx < self.explored.len() {
                        self.explored[region_idx].reveal(px, py, wider);
                    }
                }
            }
        }
    }

    /// Add an affliction to the player, if there is room for it and it is not
    /// already running. Marks the scar in the journal. Whether it lands is the
    /// caller's roll; this only carries it in.
    pub(crate) fn afflict(&mut self, disease: crate::model::Disease, scar: &str) -> bool {
        let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        let added = if let Some(ref mut ps) = self.player_start {
            const MAX_PLAYER_ILLNESSES: usize = 2;
            if ps.person.illnesses.len() >= MAX_PLAYER_ILLNESSES
                || ps.person.illnesses.iter().any(|d| d.disease == disease)
            {
                false
            } else {
                ps.person
                    .illnesses
                    .push(crate::model::ActiveDisease::new(disease, tick));
                true
            }
        } else {
            false
        };
        if added {
            if let Some(ref mut sim) = self.sim {
                sim.log(tick, crate::sim::journal::Voice::Scar, scar.into());
            }
        }
        added
    }

    pub fn check_milestones(&mut self) {
        use crate::sim::milestones::{core_peoples, faction_key, MilestoneKind};

        let day = self.clock.day;
        let fired = self.milestones.check_day_milestones(day);
        for kind in &fired {
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(tick, kind.voice(), kind.journal_text());
            }
        }

        // Elderhood is age-based now (see check_aging), not a fixed calendar day.

        let has_player_structure = self
            .sim
            .as_ref()
            .is_some_and(|sim| sim.structures.iter().any(|s| !s.is_npc_built));
        if has_player_structure && !self.milestones.has(MilestoneKind::FirstStructureBuilt) {
            self.milestones
                .record(MilestoneKind::FirstStructureBuilt, day);
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    MilestoneKind::FirstStructureBuilt.voice(),
                    MilestoneKind::FirstStructureBuilt.journal_text(),
                );
            }
        }

        if let Some(ref ps) = self.player_start {
            if !ps.companions.is_empty()
                && !self.milestones.has(MilestoneKind::FirstCompanionAdopted)
            {
                self.milestones
                    .record(MilestoneKind::FirstCompanionAdopted, day);
                if let Some(ref mut sim) = self.sim {
                    let tick = sim.world.tick;
                    sim.log(
                        tick,
                        MilestoneKind::FirstCompanionAdopted.voice(),
                        MilestoneKind::FirstCompanionAdopted.journal_text(),
                    );
                }
            }
        }

        let people_endings_to_fire: Vec<PeopleKind> = {
            let player_id = match self.player_start {
                Some(ref ps) => ps.person.id.clone(),
                None => String::new(),
            };
            if player_id.is_empty() {
                Vec::new()
            } else if let Some(ref sim) = self.sim {
                core_peoples()
                    .iter()
                    .copied()
                    .filter(|&people| {
                        let kind = MilestoneKind::PeopleEnding { people };
                        if self.milestones.has(kind) {
                            return false;
                        }
                        let fk = faction_key(people);
                        let total: f64 = sim
                            .reputation
                            .entries
                            .values()
                            .filter(|e| e.person_id == player_id)
                            .map(|e| e.reputation.by_faction.get(fk).copied().unwrap_or(0.5))
                            .sum::<f64>();
                        let count = sim
                            .reputation
                            .entries
                            .values()
                            .filter(|e| e.person_id == player_id)
                            .count();
                        count > 0 && total / count as f64 >= 0.9
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        for people in people_endings_to_fire {
            let kind = MilestoneKind::PeopleEnding { people };
            self.milestones.record(kind, day);
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(tick, kind.voice(), kind.journal_text());
            }
        }
    }

    #[allow(deprecated)]
    pub fn check_collapse(&mut self) {
        if self.vitals.hunger > 0.0 && self.vitals.energy > 0.0 {
            return;
        }
        // A collapse advances the clock for its unconscious hours; that nested
        // advance must not recursively re-trigger another collapse (which would
        // recurse without bound when vitals stay at zero).
        if self.in_collapse {
            return;
        }
        let vitals_before = self.vitals;
        let region_name = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                let region = sim.world.regions.get(pos.region_idx)?;
                Some(region.name.clone())
            })
            .unwrap_or_else(|| "unknown".to_string());
        let weather = self
            .sim
            .as_ref()
            .and_then(|sim| {
                let pos = self.player_pos?;
                let region = sim.world.regions.get(pos.region_idx)?;
                Some(region.weather.name().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());
        let local_rep = self
            .player_start
            .as_ref()
            .and_then(|ps| {
                let pid = &ps.person.id;
                self.sim.as_ref().and_then(|sim| {
                    let pos = self.player_pos?;
                    let region = sim.world.regions.get(pos.region_idx)?;
                    let settlement = region.settlements.first()?;
                    Some(sim.reputation.get(pid, &settlement.id))
                })
            })
            .unwrap_or(0.0);
        let _local_people = self.current_settlement_people();
        let eff_bias = self
            .current_settlement_people()
            .map_or(0.0, |p| self.inter_people_bias.effective_bias(p));
        let collapse = Collapse::roll_biased(self.seed, &self.god_affinity, local_rep, eff_bias);
        let outcome = collapse.outcome;
        let hours = outcome.hours_passed();
        // At the very edge, the life's hidden star gets the last word: a
        // blessed soul is sometimes pulled back from a death that would
        // otherwise have taken it. The cursed get no such reprieve — they die
        // more by meeting more trouble, not by being killed twice.
        let died = if collapse.died {
            let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
            let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x5EED_C0DE));
            crate::rng::unit_from_hash(h) >= self.fortune.death_reprieve_chance()
        } else {
            false
        };
        let rescued_by = collapse.rescued_by;
        self.vitals.hunger = (self.vitals.hunger + outcome.hunger_restore()).min(1.0);
        self.vitals.energy = (self.vitals.energy + outcome.energy_restore()).min(1.0);
        if let Some(ref mut ps) = self.player_start {
            let loss = outcome.coin_loss();
            ps.inventory.remove(ItemType::Coin, loss);
            if let Some(item) = outcome.item_loss() {
                ps.inventory.remove(item, 1);
            }
        }
        if outcome.is_divine() {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(ItemType::Herb, 3);
                ps.inventory.add(ItemType::Food, 2);
            }
        }
        if outcome.is_beast_aided() {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(ItemType::Herb, 1);
            }
        }
        if outcome.is_hostile() {
            self.vitals.hunger = 0.15;
            self.vitals.energy = 0.1;
        }
        self.in_collapse = true;
        self.advance_clock(hours);
        self.in_collapse = false;
        let tick = self.sim.as_ref().map(|sim| sim.world.tick).unwrap_or(0);
        self.collapse_log.push(CollapseEvent {
            tick,
            vitals_before,
            region: region_name,
            weather,
            outcome,
            died,
            rescued_by,
        });
        self.collapse = Some(collapse);
        self.collapses_had += 1;
        self.fire_hint(hints::HINT_FIRST_COLLAPSE);
        if let Some(ref mut sim) = self.sim {
            let voice_text = if died {
                "I collapsed. The dark took me.".into()
            } else if let Some(god) = rescued_by {
                format!("I collapsed. {} held me back from the edge.", god.label())
            } else {
                "I collapsed. The world swam and went dark.".into()
            };
            sim.log(sim.world.tick, crate::sim::journal::Voice::Scar, voice_text);
        }
        if let Some(ref mut sim) = self.sim {
            if let Some(pos) = self.player_pos {
                let memorial = crate::model::memorial::Memorial::generate(
                    sim.world.seed,
                    sim.world.tick,
                    pos.region_idx,
                    pos.px as u32,
                    pos.py as u32,
                );
                sim.memorials.push(memorial);
                if !died {
                    let recovery_region = crate::model::memorial::pick_recovery_region(
                        sim.world.seed,
                        pos.region_idx,
                        sim.world.regions.len(),
                    );
                    let recovery_god = crate::model::memorial::pick_recovery_god(sim.world.seed);
                    self.god_affinity.adjust(recovery_god, 0.01);
                    if let Some(ref mut p) = self.player_pos {
                        p.region_idx = recovery_region;
                    }
                }
            }
        }
        if died {
            let outcome = self
                .collapse
                .map(|c| c.outcome)
                .unwrap_or(CollapseOutcome::Ditch);
            self.death_cause = Some(DeathCause::from_collapse_and_vitals(outcome, self.vitals));
            if self.player_start.is_some() {
                let save_data = SaveData {
                    sim: self
                        .sim
                        .clone()
                        .unwrap_or_else(|| SimState::new(self.seed, self.charts.clone())),
                    player_start: self.player_start.clone(),
                    clock: self.clock,
                    vitals: self.vitals,
                    player_pos: self.player_pos,
                    god_affinity: self.god_affinity,
                    inter_people_bias: self.inter_people_bias.clone(),
                    collapses_had: self.collapses_had,
                    collapse_log: self.collapse_log.clone(),
                    lineage: self.lineage.clone(),
                    milestones: self.milestones.clone(),
                    explored: self.explored.clone(),
                    version: save_migrations::CURRENT_SAVE_VERSION,
                    first_run: false,
                    hint_tracker: self.hint_tracker.clone(),
                    start_age_years: self.start_age_years,
                    birth_day: self.birth_day,
                    lifespan_years: self.lifespan_years,
                    fortune: self.fortune,
                    gift: self.gift,
                    tax_unpaid_seasons: self.tax_unpaid_seasons,
                    last_tax_day: self.last_tax_day,
                    player_farms: self.player_farms.clone(),
                    homestead_settlers: self.homestead_settlers.clone(),
                    homestead_rumored: self.homestead_rumored,
                    founding_check_day: self.founding_check_day,
                    spouse_id: self.spouse_id.clone(),
                    widowed_day: self.widowed_day,
                    household_children: self.household_children.clone(),
                    travel_debt: self.travel_debt,
                    enclaves_seen: self.enclaves_seen.clone(),
                    god_vow: self.god_vow,
                    learned_sense: self.learned_sense,
                };
                let _ = save::save_lineage(&save_data, self.seed);
            }
            self.continue_as_npc();
        } else {
            self.screen = Screen::Collapse;
        }
    }

    pub fn dismiss_collapse(&mut self) {
        self.collapse = None;
        // A floor on waking, not an assignment. check_collapse already applied
        // the outcome-specific restores (a settlement bed / divine rescue
        // recovers far more than a ditch); hardcoding 0.4/0.5 here threw all of
        // that away, so every non-fatal collapse ended identically.
        self.vitals.hunger = self.vitals.hunger.max(0.4);
        self.vitals.energy = self.vitals.energy.max(0.5);
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }
}

#[cfg(test)]
mod war_tests {
    use super::*;
    use crate::charts::load::load_charts;

    #[test]
    fn war_raises_road_danger_only_in_tension() {
        let mut app = App::new(7, load_charts().unwrap());
        app.generate_player();
        app.accept_player();
        app.enter_map(0);
        let polity = app.sim.as_ref().unwrap().world.polity;
        let seed = app.seed;
        // Find a war day and a peace day for this world's polity.
        let war = (0..600u32).find(|&d| polity.in_tension(seed, (d / 30) % 4, d / 120));
        let peace = (0..600u32).find(|&d| !polity.in_tension(seed, (d / 30) % 4, d / 120));
        let (war, peace) = (war.expect("a war season"), peace.expect("a peace season"));
        app.clock.day = peace;
        assert!((app.polity_war_mult() - 1.0).abs() < 1e-9, "peace is quiet");
        app.clock.day = war;
        assert!(
            app.polity_war_mult() > 1.0,
            "war makes the contested roads more dangerous"
        );
    }
}
