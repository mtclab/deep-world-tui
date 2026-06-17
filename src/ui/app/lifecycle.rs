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
        // Born under a star, and born with a gift or (almost always) without —
        // both rolled once from the life-seed, both hidden (#426).
        self.gift = crate::model::Gift::roll(self.seed, life_salt);
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
        // In a plague year the crowds are the danger (#457): a settlement, the
        // safe haven in any ordinary season, becomes the most likely place to
        // catch it. The wild — empty of people — is the safer place to wait the
        // plague out, but it has no healer. The choice is the play.
        let plague = self.current_world_event() == Some(crate::model::WorldEvent::PlagueYear);
        let crowd_mult = if plague && on_settlement { 2.2 } else { 1.0 };
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
            fortune_mult * plague_mult * crowd_mult,
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

    /// Disease is the great leveller of the post-Fall age: in a world with no
    /// medicine, an untreated fever, a wound gone bad, a plague year, or a birth
    /// gone wrong can take you. Once per day each active illness rolls for it —
    /// deadlier the longer it festers and in a plague year, gentler when you are
    /// fed, sheltered, or near a healer who can tend it; the life's hidden star
    /// leans the edge. Treatment is the counter, never immunity.
    pub(super) fn check_illness_mortality(&mut self) {
        use crate::sim::structures::BuildKind;
        if self.death_cause.is_some() {
            return;
        }
        let Some(pos) = self.player_pos else {
            return;
        };
        let on_settlement = self.player_on_settlement().is_some();
        let has_healer = on_settlement
            && self
                .sim
                .as_ref()
                .and_then(|s| s.world.regions.get(pos.region_idx))
                .and_then(|r| r.settlements.first())
                .map(crate::sim::illness::settlement_has_healer)
                .unwrap_or(false);
        let sheltered = on_settlement
            || [
                BuildKind::Kota,
                BuildKind::Cabin,
                BuildKind::Longhouse,
                BuildKind::Home,
                BuildKind::Laavu,
            ]
            .iter()
            .any(|k| self.own_structure_near(*k, 1));
        // A plague year makes every fever deadlier, not only commoner (#417).
        let plague_mult = self
            .current_world_event()
            .map(|e| e.illness_contraction_modifier())
            .unwrap_or(1.0);
        let hunger = self.vitals.hunger;
        let energy = self.vitals.energy;

        let Some(ref ps) = self.player_start else {
            return;
        };
        // The worst case in the body sets the day's risk.
        let mut worst: Option<(crate::model::Disease, f64)> = None;
        for d in &ps.person.illnesses {
            let base = d.disease.daily_mortality();
            if base <= 0.0 {
                continue;
            }
            let mut p = base * d.severity.clamp(0.5, 1.5);
            if has_healer {
                p *= 0.30;
            } else if sheltered {
                p *= 0.65;
            }
            if hunger > 0.6 {
                p *= 0.65;
            } else if hunger < 0.3 {
                p *= 1.7;
            }
            if energy < 0.2 {
                p *= 1.3;
            }
            p *= plague_mult;
            if worst.is_none_or(|(_, q)| p > q) {
                worst = Some((d.disease, p));
            }
        }
        let Some((disease, p)) = worst else {
            return;
        };
        let p = self.fortune.tilt_bad(p).clamp(0.0, 0.95);
        let h = crate::rng::mix_u64(
            self.seed
                ^ crate::rng::mix_u64(
                    (self.clock.day as u64) ^ (disease as u64).wrapping_shl(48) ^ 0x5117_0DEA,
                ),
        );
        if crate::rng::unit_from_hash(h) < p {
            let scar = match disease {
                crate::model::Disease::Plague => {
                    "The plague took me. It takes whole houses; why not mine."
                }
                crate::model::Disease::ChildbirthComplication => {
                    "The birth went wrong, and there was no one near who could set it right."
                }
                crate::model::Disease::Venom => {
                    "The venom went all through me. No herb to hand, and the dark came up."
                }
                _ => {
                    "The fever would not break. By the third night it had me. So the age thins us."
                }
            };
            self.die_in_wilds(crate::model::DeathCause::Sickness, scar);
        }
    }

    /// The post-Fall peace is thin and unevenly kept. The new nations are still
    /// forming; between and around them lie wide ungoverned spaces no watch
    /// patrols — and they are full of the men the Fall made: broken soldiers,
    /// displaced bands, ordinary desperation turned to the knife. A night spent
    /// unsheltered in the open country risks a raid at any time; when the
    /// province's polity and its rival are at open tension (#415) the roads bleed
    /// far worse. A raid costs goods, and the worn and the unlucky their lives. A
    /// palisade or a guardian companion shortens the odds; a settlement's walls
    /// and watch end the risk entirely.
    pub(super) fn check_turmoil(&mut self) {
        use crate::sim::structures::BuildKind;
        if self.death_cause.is_some() {
            return;
        }
        if self.player_pos.is_none() || self.player_on_settlement().is_some() {
            return;
        }
        let season_ord = (self.clock.day / 30) % 4;
        let year = self.clock.day / 120;
        let at_war = self
            .sim
            .as_ref()
            .map(|s| s.world.polity.in_tension(self.seed, season_ord, year))
            .unwrap_or(false);
        let guarded = self
            .player_start
            .as_ref()
            .is_some_and(|ps| ps.companions.iter().any(|c| c.animal.guards()));
        let palisade = self.own_structure_near(BuildKind::Palisade, 2);
        // The lawless baseline of an unstable age — higher where war has loosed
        // the roads entirely.
        let mut raid_p = 0.003;
        if at_war {
            raid_p *= 3.0;
        }
        if guarded {
            raid_p *= 0.5;
        }
        if palisade {
            raid_p *= 0.4;
        }
        raid_p = self.fortune.tilt_bad(raid_p);
        let h = crate::rng::mix_u64(
            self.seed ^ crate::rng::mix_u64((self.clock.day as u64) ^ 0x4A1D_5EED),
        );
        if crate::rng::unit_from_hash(h) >= raid_p {
            return;
        }
        // A raid lands. It always takes something; whether it takes more depends
        // on how worn you were and how the night falls.
        if let Some(ref mut ps) = self.player_start {
            let coin = ps.inventory.get(ItemType::Coin);
            ps.inventory
                .remove(ItemType::Coin, (coin / 3).max(1).min(coin));
            ps.inventory.remove(ItemType::Food, 2);
        }
        let wear = 1.0 - self.vitals.hunger.clamp(0.0, 1.0);
        let mut lethal = self.fortune.tilt_bad(0.18 + 0.15 * wear);
        if guarded {
            lethal *= 0.5;
        }
        let h2 = crate::rng::mix_u64(
            self.seed ^ crate::rng::mix_u64((self.clock.day as u64) ^ 0x9E37_7A1D),
        );
        if crate::rng::unit_from_hash(h2) < lethal {
            self.die_in_wilds(
                DeathCause::Wounds,
                "Raiders in the night, out of the ungoverned dark between the new nations. The Fall left men who answer to no one, and the open country is theirs.",
            );
        } else {
            self.vitals.energy = (self.vitals.energy - 0.3).max(0.0);
            self.vitals.hunger = (self.vitals.hunger - 0.1).max(0.0);
            self.status_msg = Some(
                "Raiders struck in the night — you drove them off, but they took what they could."
                    .into(),
            );
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    "Raiders in the dark. I kept my life and little else. The roads are not the roads they were.".into(),
                );
            }
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
        // How deep the line runs (the lineage record for this death is already
        // pushed): a long, storied line opens doors wider than a first heir's
        // (#588 slice 3).
        let line_depth = self.lineage.len();
        if let Some(ref mut sim) = self.sim {
            sim.reputation
                .adjust_local(&npc_person.id, &heir_settlement_id, 0.15);
            let carry = (dead_standing - 0.5) * 0.5;
            if carry.abs() > 0.01 {
                sim.reputation
                    .adjust_local(&npc_person.id, &heir_settlement_id, carry);
            }
            // The forebear's mark on the province seeds the heir's standing
            // wherever it was made (#588 slice 1): every town that holds a
            // remembered_deed of the dead — the places kept fed through a lean
            // year, run supplies to through the war — opens its door to the
            // heir, who arrives there a half-friend, not a stranger.
            // The line's renown compounds (#588 slice 3): a third-generation
            // heir of a remembered line arrives better regarded than a first.
            let bonus = inherited_standing_bonus(line_depth);
            let remembering = towns_remembering(&sim.world, &heir_settlement_id, &dead_person.name);
            for sid in remembering {
                sim.reputation.adjust_local(&npc_person.id, &sid, bonus);
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
        // The gift runs in the blood (#429): the heir's gift is leaned by the
        // parent's. Captured before begin_life_aging re-rolls a plain one.
        let parent_gift = self.gift;
        let heir_salt = self.lineage.len() as u64;
        self.begin_life_aging(&heir_band, heir_salt);
        self.gift = crate::model::Gift::roll_heir(self.seed, heir_salt, parent_gift);

        // Continue on Map screen
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }
}

/// The settlements across the province that hold a `remembered_deed` naming the
/// forebear (#588 slice 1), other than the town the heir is taking up in — the
/// places a long life marked, where the heir arrives a half-friend, not a
/// stranger. Pure: reads the deeds laid down by play.
/// How much standing the heir inherits in each town that remembers a forebear
/// (#588 slice 3), growing with how deep the line runs: a base half-friend's
/// regard, lifted a little for each generation the line has held, capped so a
/// dynasty is renowned but never simply revered for its name alone.
fn inherited_standing_bonus(line_depth: usize) -> f64 {
    let extra = (line_depth.saturating_sub(1)).min(4) as f64 * 0.05;
    0.2 + extra
}

fn towns_remembering(
    world: &crate::model::World,
    exclude_settlement_id: &str,
    forebear_name: &str,
) -> Vec<String> {
    world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .filter(|s| {
            s.id != exclude_settlement_id
                && s.remembered_deed
                    .as_deref()
                    .is_some_and(|d| d.contains(forebear_name))
        })
        .map(|s| s.id.clone())
        .collect()
}

#[cfg(test)]
mod legacy_tests {
    use super::towns_remembering;
    use crate::charts::load::load_charts;
    use crate::gen::world::generate_world;

    #[test]
    fn inherited_standing_compounds_with_the_line_but_is_capped() {
        use super::inherited_standing_bonus;
        // A first heir gets the base; a deeper line a little more, capped.
        let first = inherited_standing_bonus(1);
        let third = inherited_standing_bonus(3);
        let ancient = inherited_standing_bonus(50);
        assert!((first - 0.2).abs() < 1e-9, "first heir gets the base");
        assert!(third > first, "a deeper line inherits more");
        assert!(
            ancient <= 0.45,
            "renown is capped — never revered for the name alone"
        );
        assert_eq!(
            inherited_standing_bonus(0),
            0.2,
            "no prior line, just the base"
        );
    }

    #[test]
    fn the_heir_is_remembered_where_the_forebear_was() {
        let charts = load_charts().unwrap();
        let mut world = generate_world(42, &charts);
        // Mark two towns with a deed of "Aino", and the death-town with another.
        let mut marked = Vec::new();
        let mut death_town = String::new();
        for region in world.regions.iter_mut() {
            for s in region.settlements.iter_mut() {
                if marked.len() < 2 {
                    s.remembered_deed =
                        Some("Aino, the stranger who kept us fed through the lean year".into());
                    marked.push(s.id.clone());
                } else if death_town.is_empty() {
                    s.remembered_deed = Some("Aino, who ran supplies to us through the war".into());
                    death_town = s.id.clone();
                }
            }
        }
        assert_eq!(marked.len(), 2);
        assert!(!death_town.is_empty());
        // The heir takes up in the death-town; the other two remember the line.
        let found = towns_remembering(&world, &death_town, "Aino");
        assert_eq!(found.len(), 2, "both marked towns remember the forebear");
        assert!(!found.contains(&death_town), "the death-town is excluded");
        // A forebear no town named seeds nothing.
        assert!(towns_remembering(&world, &death_town, "Nobody").is_empty());
    }
}
