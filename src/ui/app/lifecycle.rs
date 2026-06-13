use crate::gen::player::generate_player_start;
use crate::model::{
    DeathCause, GameClock, InterPeopleBias, Inventory, ItemType, PeopleKind, PlayerStart,
    PlayerVitals, Terrain,
};
use crate::rng::SeedRng;
use crate::save::{self, LineageRecord};
use crate::sim::hints::HintTracker;
use crate::sim::SimState;

use super::*;

impl App {
    pub fn generate_player(&mut self) {
        if let Some(ref mut rng) = self.player_rng {
            let ps = generate_player_start(rng, &self.charts);
            let pk = PeopleKind::from_name(&ps.person.people);
            self.inter_people_bias = InterPeopleBias::new(pk);
            self.player_start = Some(ps);
        }
    }

    pub fn reroll_player(&mut self) {
        if let Some(ref mut rng) = self.player_rng {
            if let Some(ref mut ps) = self.player_start {
                ps.reroll(rng, &self.charts);
            }
        }
    }

    pub fn accept_player(&mut self) {
        if let Some(mut ps) = self.player_start.take() {
            ps.accepted = true;
            let pk = PeopleKind::from_name(&ps.person.people);
            self.inter_people_bias = InterPeopleBias::new(pk);
            let sim = SimState::new(self.seed, self.charts.clone());
            self.sim = Some(sim);
            if let Some(ref mut sim) = self.sim {
                let quests = crate::sim::quest_gen::generate_initial_quests(
                    self.seed,
                    pk,
                    &sim.world.regions,
                );
                sim.quests = quests;
                // Initialize explored maps for each region
                self.explored = sim
                    .world
                    .regions
                    .iter()
                    .map(|r| crate::model::ExploredMap::new(r.terrain.width, r.terrain.height))
                    .collect();
            }
            let age_band = ps.person.age_band.clone();
            self.player_start = Some(ps);
            self.begin_life_aging(&age_band, 0);
            self.enter_map(0);
        }
    }

    /// Begin tracking age for a new life: starting age from the age band, the
    /// current calendar day as birth, and a rolled lifespan (mischance-weighted).
    fn begin_life_aging(&mut self, age_band: &str, life_salt: u64) {
        self.start_age_years = start_age_from_band(age_band);
        self.birth_day = self.clock.day;
        let mut rng = SeedRng::new(self.seed.wrapping_add(life_salt)).fork_for("lifespan");
        // Base 58-72 years; ~1 in 6 lives is cut short by frailty/mischance.
        let mut span = 58 + rng.gen_range(15);
        if rng.gen_range(6) == 0 {
            span = span.saturating_sub(8 + rng.gen_range(18));
        }
        self.lifespan_years = span.max(self.start_age_years + 2);
        // Every life is born under a star — its luck, rolled once and hidden.
        self.fortune = crate::model::Fortune::roll(self.seed, life_salt);
        self.last_omen_day = 0;
        self.elder = false;
    }

    /// Once in a while a sign shows — the fire's colour, a bird's crossing —
    /// leaning fair or ill with the life's hidden star but never proving it.
    /// Both polarities can show under any star; the omen reads fate, it does
    /// not change it. Called on the turn of a day.
    pub(super) fn maybe_omen(&mut self) {
        let day = self.clock.day;
        // Never on the first day, and no more than one sign in any five.
        if day == 0 || day.saturating_sub(self.last_omen_day) < 5 {
            return;
        }
        // ~1 day in 4 carries a sign — deterministic per seed/day.
        use crate::rng::{mix_u64, unit_from_hash};
        let h = mix_u64(self.seed ^ mix_u64(day as u64));
        if unit_from_hash(h) >= 0.25 {
            return;
        }
        self.last_omen_day = day;
        // Fair or ill, leaned by the star — both possible under either.
        let polarity = unit_from_hash(mix_u64(h ^ 0xD1B54A32D192ED03));
        let bank = if polarity < self.fortune.fair_omen_chance() {
            "OMENS_FAIR"
        } else {
            "OMENS_ILL"
        };
        let lines = crate::banks::bank(bank);
        let mut rng = SeedRng::new(self.seed).fork_for(&format!("omen-{day}"));
        let line = lines[rng.gen_range(lines.len() as u32) as usize].clone();
        if let Some(ref mut sim) = self.sim {
            sim.log(
                sim.world.tick,
                crate::sim::journal::Voice::Dream,
                line.clone(),
            );
        }
        self.status_msg = Some(line);
    }

    /// Restore aging fields from a loaded save; pre-aging saves (lifespan 0)
    /// get a fresh lifespan rolled from the player's band so they still age.
    pub(crate) fn apply_loaded_aging(
        &mut self,
        start_age_years: u32,
        birth_day: u32,
        lifespan_years: u32,
    ) {
        self.start_age_years = start_age_years;
        self.birth_day = birth_day;
        self.lifespan_years = lifespan_years;
        if self.lifespan_years == 0 {
            if let Some(band) = self
                .player_start
                .as_ref()
                .map(|ps| ps.person.age_band.clone())
            {
                self.begin_life_aging(&band, self.clock.day as u64);
            }
        }
    }

    /// The player's current age in years, derived from elapsed calendar days.
    pub fn current_age_years(&self) -> u32 {
        let elapsed = self.clock.day.saturating_sub(self.birth_day);
        self.start_age_years + elapsed / AGING_DAYS_PER_LIFE_YEAR
    }

    /// Advance elderhood and old-age death based on the player's age. Called
    /// once per clock advance.
    pub(super) fn check_aging(&mut self) {
        if self.player_start.is_none() || self.lifespan_years == 0 {
            return;
        }
        let age = self.current_age_years();
        if age >= self.lifespan_years {
            self.die_of_old_age();
            return;
        }
        if !self.elder && age + ELDER_BAND_YEARS >= self.lifespan_years {
            self.elder = true;
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    "My years weigh on me now. I have become an elder.".into(),
                );
            }
        }
    }

    fn die_of_old_age(&mut self) {
        self.die_in_wilds(
            DeathCause::OldAge,
            "Age took me, quiet as dusk. The world remembers, and life goes on.",
        );
    }

    /// End the current life of a given cause: record it, save the lineage, mark
    /// the scar in the journal, and pass to the heir. The shared spine behind
    /// old age and the deaths the wilds deal directly (a beast that does not
    /// let go).
    pub(super) fn die_in_wilds(&mut self, cause: DeathCause, scar: &str) {
        self.death_cause = Some(cause);
        if let Some(data) = self.build_save_data() {
            let _ = save::save_lineage(&data, self.seed);
        }
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(tick, crate::sim::journal::Voice::Scar, scar.into());
        }
        self.continue_as_npc();
    }

    /// Return to the World map at the player's current region.
    pub fn return_to_world(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    /// Recover from any run-out illnesses, then roll for contracting a new one
    /// based on the player's terrain, hunger, shelter, and access to a healer.
    /// The player was previously immune — illness was an NPC-only system.
    pub(super) fn check_player_illness(&mut self) {
        let Some(pos) = self.player_pos else {
            return;
        };
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let terrain = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(pos.region_idx))
            .and_then(|r| r.terrain.get(pos.px, pos.py))
            .unwrap_or(Terrain::Grass);
        let on_settlement = self.player_on_settlement().is_some();
        let has_healer = on_settlement
            && self
                .sim
                .as_ref()
                .and_then(|s| s.world.regions.get(pos.region_idx))
                .and_then(|r| r.settlements.first())
                .map(crate::sim::illness::settlement_has_healer)
                .unwrap_or(false);

        // A Needs proxy from the player's vitals (Food = hunger; Safety from shelter).
        let mut needs = crate::model::Needs::default();
        needs
            .values
            .insert(crate::model::Need::Food, self.vitals.hunger);
        needs.values.insert(
            crate::model::Need::Safety,
            if on_settlement { 0.8 } else { 0.2 },
        );

        // A plague year makes sickness take more readily (#417). Captured
        // before the player borrow below.
        let plague_mult = self
            .current_world_event()
            .map(|e| e.illness_contraction_modifier())
            .unwrap_or(1.0);
        let fortune_mult = self.fortune.bad_multiplier();
        let Some(ref mut ps) = self.player_start else {
            return;
        };
        ps.person.illnesses.retain(|d| !d.is_recovered(tick));
        let count = ps.person.illnesses.len();
        let contracted = crate::sim::illness::tick_illness_luck(
            self.seed,
            tick,
            terrain,
            &needs,
            has_healer,
            count,
            fortune_mult * plague_mult,
        )
        .filter(|d| {
            d.disease != crate::model::Disease::ChildbirthComplication
                || crate::sim::illness::can_contract_childbirth(&ps.person.sex, &ps.person.age_band)
        });
        if let Some(disease) = contracted {
            let label = disease.disease.name();
            ps.person.illnesses.push(disease);
            if let Some(ref mut sim) = self.sim {
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    format!("A sickness takes me — {label}. My body turns against the road."),
                );
            }
            self.status_msg = Some(format!("You have fallen ill: {label}."));
        }
    }

    pub fn restart_game(&mut self) {
        self.sim = None;
        self.player_start = None;
        self.collapse = None;
        self.death_cause = None;
        self.encounter = None;
        self.clock = GameClock::default();
        self.vitals = PlayerVitals::default();
        self.player_pos = None;
        self.screen = Screen::CharacterCreation;
        self.status_msg = None;
        self.running = true;
        self.lineage.clear();
        self.hint_tracker = HintTracker::default();
    }

    pub(crate) fn continue_as_npc(&mut self) {
        let dead_ps = match &self.player_start {
            Some(ps) => ps.clone(),
            None => {
                self.screen = Screen::GameOver;
                return;
            }
        };
        let dead_person = dead_ps.person.clone();
        let settlement_id = dead_person.settlement.clone();

        // Blood before friendship: the eldest grown child of the house is
        // the heir, if there is one. Friends inherit only a childless line.
        let grown_child_idx = self
            .household_children
            .iter()
            .position(|c| self.child_age_years(c) >= 16);
        let (npc_person, heir_settlement_id) = if let Some(ci) = grown_child_idx {
            let child = self.household_children.remove(ci);
            let settlement_id = self
                .player_pos
                .and_then(|pos| {
                    self.sim
                        .as_ref()
                        .and_then(|s| s.world.regions.get(pos.region_idx))
                        .and_then(|r| r.settlements.first())
                        .map(|s| s.id.clone())
                })
                .unwrap_or_else(|| dead_person.settlement.clone());
            let mut heir = dead_person.clone();
            heir.id = format!("heir-{}-{}", self.lineage.len(), self.seed % 0xFFFF);
            heir.name = child.name;
            heir.age_band = "young".into();
            heir.age_years = self.child_age_years(&crate::model::HouseholdChild {
                name: String::new(),
                born_day: child.born_day,
            });
            heir.has_spouse = false;
            heir.children_count = 0;
            heir.illnesses.clear();
            (heir, settlement_id)
        } else {
            // Find a related NPC
            let npc_idx = match self.find_related_npc(&dead_person) {
                Some(idx) => idx,
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };

            // Get the NPC person (and the settlement the heir actually lives
            // in — reputation used to be keyed by the dead's stored
            // settlement string, which can differ from any real settlement
            // id).
            let pos = match self.player_pos {
                Some(p) => p,
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            let region = match self
                .sim
                .as_ref()
                .and_then(|s| s.world.regions.get(pos.region_idx))
            {
                Some(r) => r,
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            let settlement = match region.settlements.first() {
                Some(s) => s,
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            let person = match settlement.people.get(npc_idx) {
                Some(p) => p.clone(),
                None => {
                    self.screen = Screen::GameOver;
                    return;
                }
            };
            (person, settlement.id.clone())
        };
        // A widow(er)'s grief does not pass down; the heir starts unwed.
        self.spouse_id = None;
        self.widowed_day = 0;

        // Add lineage record. The authoritative cause is `death_cause` — both
        // death paths (collapse and old age) set it. The old code read
        // `self.collapse.outcome`, which is stale/None for an old-age death, so
        // OldAge deaths were mislabeled with a leftover collapse outcome or
        // "unknown" and never showed as "old age" in the lineage.
        let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        let cause = self
            .death_cause
            .map(|d| d.label().to_string())
            .or_else(|| self.collapse.as_ref().map(|c| format!("{:?}", c.outcome)))
            .unwrap_or_else(|| "unknown".to_string());

        self.lineage.push(LineageRecord {
            predecessor_name: dead_person.name.clone(),
            predecessor_id: dead_person.id.clone(),
            cause,
            settlement_id: settlement_id.clone(),
            tick,
        });

        // Create new PlayerStart from NPC. The heir keeps a keepsake — a few
        // coins and one thing the dead carried — and the family's standing
        // doesn't vanish with the body (half of it carries to the heir).
        let mut inherited = Inventory::default();
        let keepsake = self.player_start.as_ref().map(|ps| {
            let coins = ps.inventory.get(ItemType::Coin).min(3);
            let item = ps
                .inventory
                .items
                .keys()
                .copied()
                .find(|i| *i != ItemType::Coin && ps.inventory.get(*i) > 0);
            (coins, item)
        });
        if let Some((coins, item)) = keepsake {
            if coins > 0 {
                inherited.add(ItemType::Coin, coins);
            }
            if let Some(it) = item {
                inherited.add(it, 1);
            }
        }
        let new_player_start = PlayerStart {
            person: npc_person.clone(),
            reroll_count: 0,
            point_buy_adjustments: Vec::new(),
            accepted: true,
            inventory: inherited,
            companions: vec![],
        };

        // Add memorial journal entry
        let memorial = format!(
            "{} passed on. You carry their memory forward.",
            dead_person.name
        );
        if let Some(ref mut sim) = self.sim {
            sim.log_journal(sim.world.tick, memorial);
        }

        // +0.15 reputation boost, plus half the dead's standing carries over —
        // the family name is remembered.
        let dead_standing = self
            .sim
            .as_ref()
            .map(|sim| sim.reputation.get(&dead_person.id, &heir_settlement_id))
            .unwrap_or(0.5);
        if let Some(ref mut sim) = self.sim {
            sim.reputation
                .adjust_local(&npc_person.id, &heir_settlement_id, 0.15);
            let carry = (dead_standing - 0.5) * 0.5;
            if carry.abs() > 0.01 {
                sim.reputation
                    .adjust_local(&npc_person.id, &heir_settlement_id, carry);
            }
        }

        // Switch player
        let heir_band = npc_person.age_band.clone();
        self.player_start = Some(new_player_start);
        self.vitals = PlayerVitals::default();
        self.inter_people_bias = InterPeopleBias::new(PeopleKind::from_name(&npc_person.people));
        // The heir is a fresh life: reset aging, salted by lineage depth so each
        // generation rolls its own lifespan.
        self.death_cause = None;
        self.begin_life_aging(&heir_band, self.lineage.len() as u64);

        // Continue on Map screen
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }
}
