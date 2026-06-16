use crate::model::{
    Collapse, CollapseOutcome, DeathCause, Encounter, EncounterAction, EncounterLogEntry, GodName,
    InterPeopleBias, ItemType, PeopleKind, Terrain, WitnessLevel,
};
use crate::save::{self, SaveData};
use crate::save_migrations;
use crate::sim::collapse_log::CollapseEvent;
use crate::sim::hints;
use crate::sim::SimState;

use super::*;

/// What luck does when you turn your back on a creature. Caution is not
/// immunity: a beast that stands its ground gets a say, and you never know
/// your luck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FleeOutcome {
    /// You broke clean. The common case — but never the only one.
    Clean,
    /// It caught you as you turned: a gash, bleeding, but you got away.
    Wounded,
    /// It ran you down. The ground rushed up. This feeds the collapse funnel.
    RunDown,
}

impl App {
    pub fn open_encounter_log(&mut self) {
        self.previous_screen = Some(self.screen.clone());
        self.screen = Screen::EncounterLog { scroll: 0 };
    }

    pub fn exit_encounter_log(&mut self) {
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

    pub fn check_encounter(&mut self, terrain: Terrain) {
        let pp = Some(self.inter_people_bias.player_people);
        // Weather stirs or settles the land: storms raise encounter chance,
        // clear skies lower it (encounter_rate_modifier was never wired).
        let weather_mult = self
            .player_pos
            .map(|pos| {
                let base = crate::sim::weather::encounter_rate_modifier(
                    self.region_weather(pos.region_idx),
                );
                // Empty land is quiet land: wild terrain spawns scale with game.
                let wild = matches!(
                    terrain,
                    Terrain::Forest | Terrain::Swamp | Terrain::Cave | Terrain::Tundra
                );
                if wild {
                    let richness = self
                        .sim
                        .as_ref()
                        .and_then(|s| s.world.regions.get(pos.region_idx))
                        .map(|r| r.game_richness)
                        .unwrap_or(1.0);
                    base * (0.5 + 0.5 * richness)
                } else {
                    base
                }
            })
            .unwrap_or(1.0);
        // A kept signal fire holds the lit dark off: dusk-to-deep-night
        // encounters near one's own beacon come half as often.
        let night = matches!(
            crate::model::TimeOfDay::from_hour(self.clock.hour),
            crate::model::TimeOfDay::Dusk
                | crate::model::TimeOfDay::Night
                | crate::model::TimeOfDay::DeepNight
        );
        let weather_mult =
            if night && self.own_structure_near(crate::sim::structures::BuildKind::Beacon, 2) {
                weather_mult * 0.5
            } else {
                weather_mult
            };
        // The roads of a people you have wronged or warred are not safe to you.
        let weather_mult = weather_mult * self.road_tension_mult();
        if let Some(enc) = Encounter::roll_biased_weather(
            terrain,
            self.clock.hour,
            self.clock.day,
            self.seed,
            pp,
            weather_mult,
        ) {
            self.encounter = Some(enc);
            // Wildlife gets a face: a terrain- and season-true species.
            if enc.kind == crate::model::EncounterKind::Wildlife {
                let sp_seed = self
                    .seed
                    .wrapping_add(((self.clock.day as u64) << 8) | self.clock.hour as u64);
                let species = crate::model::wildlife::WildSpecies::roll(
                    terrain,
                    self.clock.season(),
                    sp_seed,
                );
                if let Some(ref mut e) = self.encounter {
                    e.species = species;
                }
            }
            self.encounters_had += 1;
            self.fire_hint(hints::HINT_FIRST_ENCOUNTER);
            self.screen = Screen::Encounter;
            self.trigger_flash();
            if let Some(ref enc) = self.encounter {
                let sound = match enc.kind {
                    crate::model::EncounterKind::Storm => crate::audio::SoundEvent::Weather,
                    crate::model::EncounterKind::Bandit | crate::model::EncounterKind::Wildlife => {
                        crate::audio::SoundEvent::Combat
                    }
                    crate::model::EncounterKind::HarvestMarket
                    | crate::model::EncounterKind::MerchantCaravan => {
                        crate::audio::SoundEvent::Trade
                    }
                    _ => crate::audio::SoundEvent::UiClick,
                };
                self.play_sound(sound);
            }
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

    pub fn dismiss_encounter(&mut self) {
        self.resolve_encounter(EncounterAction::Flee);
    }

    /// Roll what fleeing costs you. `danger` is the creature's nerve (0 flees
    /// on sight, 1 stands its ground, 2 hunts). Deterministic per seed/tick so
    /// a given turn always resolves the same way. A guardian companion shortens
    /// the odds — caution that pays — but nothing makes flight free.
    fn flee_outcome(&self, danger: u8) -> FleeOutcome {
        if danger == 0 {
            return FleeOutcome::Clean; // it was already leaving
        }
        let guarded = self
            .player_start
            .as_ref()
            .is_some_and(|ps| ps.companions.iter().any(|c| c.animal.guards()));
        let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        let h = crate::rng::mix_u64(
            self.seed ^ crate::rng::mix_u64(tick ^ (danger as u64).wrapping_shl(56)),
        );
        let roll = crate::rng::unit_from_hash(h); // 0.0..1.0

        // danger 1: a 12% chance of a gash, no catching. danger 2: 22% gash,
        // and a 5% sub-band where it runs you down. A guardian halves both.
        let mut wound_p = 0.06 + 0.08 * danger as f64; // d1=0.14, d2=0.22
        let mut catch_p = if danger >= 2 { 0.05 } else { 0.0 };
        if guarded {
            wound_p *= 0.5;
            catch_p *= 0.5;
        }
        // The life's hidden star leans the odds — the cautious are safer,
        // never safe; the cursed pay more for the same flight.
        wound_p = self.fortune.tilt_bad(wound_p);
        catch_p = self.fortune.tilt_bad(catch_p);
        if roll < catch_p {
            FleeOutcome::RunDown
        } else if roll < wound_p {
            FleeOutcome::Wounded
        } else {
            FleeOutcome::Clean
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

    /// A wound is not only what it costs at the moment. A bite carries what was
    /// on the teeth; dirty ground sours an open gash. Roll, fortune-leaned, for
    /// the wound to turn — a venomous strike to venom, anything else to a
    /// festering infection. Treatment (a healer, herbs) is the counter; luck
    /// decides only whether the turn comes. Returns a note if something took.
    fn roll_wound_affliction(
        &mut self,
        species: Option<crate::model::wildlife::WildSpecies>,
        terrain: Terrain,
        dirty_base: f64,
    ) -> Option<&'static str> {
        let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        let venomous = species.is_some_and(|s| s.venomous());
        // A venomous strike that broke skin is very likely to envenom; an
        // ordinary wound festers by the dirt of the teeth and the ground.
        let (disease, base, scar): (crate::model::Disease, f64, &'static str) = if venomous {
            (
                crate::model::Disease::Venom,
                0.6,
                "The bite burns cold, then hot. Venom — it is in me now.",
            )
        } else {
            let dirty_ground = matches!(terrain, Terrain::Swamp | Terrain::Coast | Terrain::Cave);
            (
                crate::model::Disease::Infection,
                dirty_base + if dirty_ground { 0.12 } else { 0.0 },
                "The wound will not close clean. By morning it is hot and angry — it has turned.",
            )
        };
        let p = self.fortune.tilt_bad(base);
        let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x00AF_F11C));
        if crate::rng::unit_from_hash(h) < p && self.afflict(disease, scar) {
            Some(if venomous {
                " The strike was venomous."
            } else {
                " The wound has turned — it festers."
            })
        } else {
            None
        }
    }

    pub fn resolve_encounter(&mut self, action: EncounterAction) {
        let terrain = self.encounter.map(|e| e.terrain).unwrap_or(Terrain::Grass);
        let species = self.encounter.and_then(|e| e.species);
        let enc_kind = self.encounter.map(|e| e.kind);
        let witness = WitnessLevel::roll(self.seed.wrapping_mul(7919), terrain);
        let rep_mult = witness.reputation_multiplier();
        let enc_mod = match terrain {
            Terrain::Settlement | Terrain::Road => self
                .sim
                .as_ref()
                .and_then(|sim| {
                    let pos = self.player_pos?;
                    let region = sim.world.regions.get(pos.region_idx)?;
                    let settlement = region.settlements.first()?;
                    let person = settlement.people.first()?;
                    Some(InterPeopleBias::encounter_modifier(&person.personality))
                })
                .unwrap_or_default(),
            _ => InterPeopleBias::encounter_modifier(&[]),
        };
        // Weather colors the mood of whoever you meet — rain sours a talk,
        // a clear sky warms it (npc_mood_modifier was never applied).
        let weather_mood = self
            .player_pos
            .map(|pos| self.region_weather(pos.region_idx).npc_mood_modifier())
            .unwrap_or(0.0);
        let people_bias_mod = self.current_settlement_people().map_or(0.0, |npc_people| {
            self.inter_people_bias.effective_bias(npc_people)
                + self.clock.season().bias_modifier()
                + weather_mood
        });
        let god_calm_bonus = if self.god_affinity.get(GodName::Keuru) > 0.4 {
            0.03
        } else {
            0.0
        };
        let god_intimidate_bonus = if self.god_affinity.get(GodName::Oltzed) > 0.4 {
            0.03
        } else {
            0.0
        };
        // Trust bonus from NPC memory (if we know this person)
        let trust_bonus = self.current_settlement_people().map_or(0.0, |_npc_people| {
            if let Some(ref sim) = self.sim {
                let pos = match self.player_pos {
                    Some(p) => p,
                    None => return 0.0,
                };
                let region = sim.world.regions.get(pos.region_idx);
                let settlement = region.and_then(|r| r.settlements.first());
                let person = settlement.and_then(|s| s.people.first());
                if let Some(p) = person {
                    sim.npc_memories
                        .get(&p.id)
                        .map_or(0.0, |m| m.cumulative_trust().clamp(-0.3, 0.3))
                } else {
                    0.0
                }
            } else {
                0.0
            }
        });
        let elder_talk_bonus = if self.elder { 0.10 } else { 0.0 };
        let companion_mood_bonus: f64 = self
            .player_start
            .as_ref()
            .and_then(|ps| ps.companions.first())
            .map(|c| c.mood().encounter_bonus())
            .unwrap_or(0.0);
        let talk_success =
            people_bias_mod + trust_bonus + elder_talk_bonus + companion_mood_bonus > -0.20;
        let trade_bonus = people_bias_mod + trust_bonus + companion_mood_bonus > 0.05;
        let msg = match action {
            EncounterAction::Flee => {
                if enc_mod.flee > 0.05 {
                    "You fled quickly! Your instincts served you.".into()
                } else {
                    "You fled! (cost some energy)".into()
                }
            }
            EncounterAction::Bribe => {
                let base_cost: u32 = 2;
                let effective_cost =
                    ((base_cost as f64) * (1.0 + enc_mod.bribe_cost.abs())).max(1.0) as u32;
                if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.get(ItemType::Coin) >= effective_cost {
                        ps.inventory.remove(ItemType::Coin, effective_cost);
                        format!("You paid {} coins to be left alone.", effective_cost)
                    } else {
                        ps.inventory
                            .remove(ItemType::Coin, ps.inventory.get(ItemType::Coin));
                        "You gave what you had. They seemed satisfied.".into()
                    }
                } else {
                    "You gestured peacefully. They let you pass.".into()
                }
            }
            EncounterAction::Calm => {
                // The still-sense settles a wild thing surely — the Laakso gift
                // (#439). Its cost is charged after the match, below.
                if self.gift.sense().map(|s| s.aids_calm()).unwrap_or(false) {
                    "The quiet in you reaches the beast. It stills, and lets you pass.".into()
                } else if enc_mod.calm + god_calm_bonus > 0.03 {
                    "Your calm presence soothed the beast. It bows its head.".into()
                } else {
                    "The beast settled. It watches you leave.".into()
                }
            }
            EncounterAction::Intimidate => {
                self.play_sound(crate::audio::SoundEvent::Combat);
                if enc_mod.intimidate + god_intimidate_bonus > 0.03 {
                    "You loomed large. They scattered before you.".into()
                } else {
                    "You stared them down. They backed off.".into()
                }
            }
            EncounterAction::Talk => {
                if !talk_success {
                    "They turned away coldly. No wisdom shared.".into()
                } else if self.elder && enc_mod.talk + elder_talk_bonus > 0.08 {
                    "An elder speaks. Even the hostile listen. Wisdom flows freely.".into()
                } else if enc_mod.talk > 0.03 {
                    "The traveler warmed to you quickly. Wisdom flows freely.".into()
                } else if enc_mod.talk < -0.02 {
                    "Words came slow. They barely shared a thing.".into()
                } else {
                    "The traveler shared road wisdom (1h).".into()
                }
            }
            EncounterAction::Trade
                if enc_kind == Some(crate::model::EncounterKind::ThresholdToll) =>
            {
                // Pay the threshold-keeper her price (#455): bread, herb, or coin,
                // her choosing taken from what you carry. Pay, and the road past
                // is easy and the dark kindly — the threshold-keeper Kukri marks
                // it. Come empty-handed and she waves you on, uneasily.
                self.play_sound(crate::audio::SoundEvent::UiClick);
                let paid = if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.remove(ItemType::Herb, 1) {
                        Some("a bundle of herb")
                    } else if ps.inventory.remove(ItemType::Food, 2) {
                        Some("bread from your pack")
                    } else if ps.inventory.remove(ItemType::Coin, 3) {
                        Some("three coins")
                    } else {
                        None
                    }
                } else {
                    None
                };
                match paid {
                    Some(what) => {
                        self.god_affinity.adjust(GodName::Kukri, 0.04);
                        format!(
                            "You set {what} by her fire. She nods once. The road past is easy, \
                             the dark kindly — you tell yourself it was only a hermit. (1h)"
                        )
                    }
                    None => "You have nothing she asks for. She regards you a long moment, then \
                         waves you on. You go quickly, and do not look back. (1h)"
                        .into(),
                }
            }
            EncounterAction::Trade
                if enc_kind == Some(crate::model::EncounterKind::TzakharTrader) =>
            {
                // The Tzäkhar are the deep-stone smiths: they give worked metal
                // for surface food, take no coin, and do not hurry (#447).
                self.play_sound(crate::audio::SoundEvent::Trade);
                if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.remove(ItemType::Food, 2) {
                        ps.inventory.add(ItemType::Iron, 2);
                        ps.inventory.add(ItemType::Tool, 1);
                        "The Tzäkhar weigh your food in broad hands and set out worked iron and a \
                         fine-forged tool on the stone. No coin, no hurry. (1h)"
                            .into()
                    } else if ps.inventory.remove(ItemType::Herb, 2) {
                        ps.inventory.add(ItemType::Iron, 1);
                        "The Tzäkhar take your herbs for the deep larder and hand up a bar of clean \
                         iron. The stone keeps its own count. (1h)"
                            .into()
                    } else {
                        "The Tzäkhar look over your goods and want none of it — and they will not \
                         take coin. They withdraw into the dark. (1h)"
                            .into()
                    }
                } else {
                    "The Tzäkhar wait at the cave-mouth, and you have nothing they want.".into()
                }
            }
            EncounterAction::Trade if enc_kind == Some(crate::model::EncounterKind::HalTrader) => {
                // The Häl bring canopy medicine and fruit down for surface make —
                // cloth and tools, never coin (#447).
                self.play_sound(crate::audio::SoundEvent::Trade);
                if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.remove(ItemType::Tool, 1) {
                        ps.inventory.add(ItemType::Salve, 1);
                        ps.inventory.add(ItemType::Herb, 2);
                        ps.inventory.add(ItemType::Food, 1);
                        "The Häl take your tool up into the green and lower canopy-salve, bundled \
                         herbs, and bright fruit in return. Coin means nothing here. (1h)"
                            .into()
                    } else if ps.inventory.remove(ItemType::Cloth, 1) {
                        ps.inventory.add(ItemType::Herb, 2);
                        ps.inventory.add(ItemType::Food, 2);
                        "The Häl fold your cloth away and set out herbs and canopy-fruit in fair \
                         measure. (1h)"
                            .into()
                    } else {
                        "The Häl regard your goods and want none of it — and coin is nothing to \
                         them. They go back up into the canopy. (1h)"
                            .into()
                    }
                } else {
                    "The Häl wait on the forest floor, and you have nothing to offer.".into()
                }
            }
            EncounterAction::Trade
                if enc_kind == Some(crate::model::EncounterKind::ShearTrader) =>
            {
                // The She'ar give desert game and succulent-physic for what wards
                // the sun — cloth and tools, never coin (#447).
                self.play_sound(crate::audio::SoundEvent::Trade);
                if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.remove(ItemType::Cloth, 1) {
                        ps.inventory.add(ItemType::Food, 2);
                        ps.inventory.add(ItemType::Herb, 1);
                        "The She'ar take your cloth against the sun and lay out dried desert game \
                         and a bundle of succulent-physic. No coin. The rate is the rate. (1h)"
                            .into()
                    } else if ps.inventory.remove(ItemType::Tool, 1) {
                        ps.inventory.add(ItemType::Food, 2);
                        ps.inventory.add(ItemType::Herb, 2);
                        "The She'ar weigh your tool and hand over desert game and succulent-physic \
                         in good measure. (1h)"
                            .into()
                    } else {
                        "The She'ar consider your goods and want none of it — and they will not \
                         take coin. They withdraw into the shimmering heat. (1h)"
                            .into()
                    }
                } else {
                    "The She'ar wait in the long shade, and you have nothing they want.".into()
                }
            }
            EncounterAction::Trade
                if enc_kind == Some(crate::model::EncounterKind::MerakTrader) =>
            {
                // The Mëräk barter deep-water goods for surface make — cloth and
                // tools, never coin, fixed seasonal measure (#445).
                self.play_sound(crate::audio::SoundEvent::Trade);
                if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.remove(ItemType::Tool, 1) {
                        ps.inventory.add(ItemType::Food, 3);
                        ps.inventory.add(ItemType::Glass, 1);
                        "The Mëräk take your tool down into the dark and leave cold deep-fish and \
                         a shard of deep-glass on the stones. No coin, no haggle. (1h)"
                            .into()
                    } else if ps.inventory.remove(ItemType::Cloth, 1) {
                        ps.inventory.add(ItemType::Food, 2);
                        "The Mëräk fold your cloth away and lay out deep-fish in fair measure. \
                         The rate is the rate. (1h)"
                            .into()
                    } else {
                        "The Mëräk regard your goods and want none of it — and they will not \
                         take coin. They slide back beneath the water. (1h)"
                            .into()
                    }
                } else {
                    "The Mëräk wait at the tideline, and you have nothing to offer.".into()
                }
            }
            EncounterAction::Trade if enc_kind == Some(crate::model::EncounterKind::KhorTrader) => {
                // The Khör barter härkä goods for metal — no coin, no haggling,
                // fixed measure (#443). A tool is worth more than raw iron.
                self.play_sound(crate::audio::SoundEvent::Trade);
                if let Some(ref mut ps) = self.player_start {
                    if ps.inventory.remove(ItemType::Tool, 1) {
                        ps.inventory.add(ItemType::Hide, 2);
                        ps.inventory.add(ItemType::Food, 2);
                        "The Khör weigh your tool without a word and lay out härkä-leather and \
                         steppe-butter in fair measure. No coin, no haggle. (1h)"
                            .into()
                    } else if ps.inventory.remove(ItemType::Iron, 1) {
                        ps.inventory.add(ItemType::Hide, 2);
                        ps.inventory.add(ItemType::Food, 1);
                        "The Khör take your iron and hand you härkä-leather and steppe-butter. \
                         No coin changes hands; no word is wasted. (1h)"
                            .into()
                    } else {
                        "The Khör look over your goods and want none of it — and they will not \
                         take coin. They turn back to the cold. (1h)"
                            .into()
                    }
                } else {
                    "The Khör wait, and you have nothing to offer.".into()
                }
            }
            EncounterAction::Trade => {
                if let Some(ref mut ps) = self.player_start {
                    let base_herbs = if trade_bonus { 2 } else { 1 };
                    let herbs = if enc_mod.trade > 0.02 {
                        base_herbs + 1
                    } else {
                        base_herbs
                    };
                    ps.inventory.add(ItemType::Herb, herbs);
                    self.play_sound(crate::audio::SoundEvent::Trade);
                    if herbs >= 3 {
                        "A generous trade — three herbs for your news! (1h)".into()
                    } else if herbs == 2 {
                        "A good trade — two herbs (1h)".into()
                    } else {
                        "Traded news for herbs (1h)".into()
                    }
                } else {
                    "Traded news for herbs (1h)".into()
                }
            }
            EncounterAction::Shelter => "You waited out the storm (1h).".into(),
            EncounterAction::PushThrough => {
                if enc_mod.push_through > 0.03 {
                    "You surged ahead — nothing could slow you!".into()
                } else {
                    "You pushed through regardless!".into()
                }
            }
            EncounterAction::Hunt => {
                if let Some(sp) = species.filter(|s| s.huntable()) {
                    let (base_hide, base_meat) = sp.hunt_yield();
                    // The land's richness scales the meat — a hunted-out valley
                    // gives a lean take. Read before the mutable draw below.
                    let richness = self
                        .player_pos
                        .and_then(|pos| self.sim.as_ref()?.world.regions.get(pos.region_idx))
                        .map(|r| r.game_richness)
                        .unwrap_or(1.0);
                    let meat = ((base_meat as f64) * (0.5 + 0.5 * richness))
                        .round()
                        .max(1.0) as u32;
                    // Luck leans the take: a clean kill gives a bonus portion,
                    // a poor one falls short — the fortunate eat better.
                    let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
                    let luck = crate::rng::unit_from_hash(crate::rng::mix_u64(
                        self.seed ^ crate::rng::mix_u64(tick ^ 0x407E_5A1D),
                    ));
                    let meat = if luck < self.fortune.tilt_good(0.30) {
                        meat + 1
                    } else if luck > 1.0 - self.fortune.tilt_bad(0.20) {
                        meat.saturating_sub(1).max(1)
                    } else {
                        meat
                    };
                    if let Some(ref mut ps) = self.player_start {
                        ps.inventory.add(ItemType::Hide, base_hide);
                        ps.inventory.add(ItemType::Food, meat);
                    }
                    // Hunting draws the valley's game down; it recovers seasonally.
                    if let (Some(pos), Some(sim)) = (self.player_pos, self.sim.as_mut()) {
                        if let Some(region) = sim.world.regions.get_mut(pos.region_idx) {
                            region.game_richness = (region.game_richness - 0.06).max(0.0);
                        }
                    }
                    self.play_sound(crate::audio::SoundEvent::Combat);
                    format!(
                        "You hunt the {} — {} hide, {} meat. (2h)",
                        sp.name(),
                        base_hide,
                        meat
                    )
                } else {
                    "No quarry here worth the taking.".into()
                }
            }
        };
        // Apply the action's data-driven costs (time, energy, hunger) and its
        // god-affinity shift. These lived on EncounterAction but were never
        // wired in — resolution hardcoded a flat 1h + 0.08 energy and ignored
        // hunger and god affinity entirely.
        let hours = action.hours();
        if hours > 0 {
            self.advance_clock(hours);
        }
        self.vitals.energy = (self.vitals.energy - action.energy_cost()).max(0.0);
        self.vitals.hunger = (self.vitals.hunger - action.hunger_cost()).max(0.0);
        if let Some((god, delta)) = action.god_affinity_effect() {
            self.god_affinity.adjust(god, delta);
        }
        let encounter_data = self.encounter.map(|e| e.kind);
        // Fleeing is a gamble, not an exit. A creature that stands its ground
        // or hunts gets a say when you turn your back — and a guardian shortens
        // the odds without ever making flight free. The worst roll feeds the
        // collapse funnel below (`flee_run_down`).
        let mut flee_wound_note: Option<&'static str> = None;
        let mut affliction_note: Option<&'static str> = None;
        let mut flee_run_down = false;
        if action == EncounterAction::Flee {
            let danger = match encounter_data {
                Some(crate::model::EncounterKind::Wildlife) => {
                    species.map(|sp| sp.danger()).unwrap_or(1)
                }
                Some(crate::model::EncounterKind::Bandit) => 1,
                _ => 0,
            };
            match self.flee_outcome(danger) {
                FleeOutcome::Clean => {}
                FleeOutcome::Wounded => {
                    self.vitals.energy = (self.vitals.energy - 0.28).max(0.0);
                    self.vitals.hunger = (self.vitals.hunger - 0.06).max(0.0);
                    flee_wound_note =
                        Some("It caught you as you turned — a gash, bleeding, but you broke free.");
                    // The gash may carry what was on the teeth, or sour in dirty
                    // ground — luck decides whether it turns.
                    affliction_note = self.roll_wound_affliction(species, terrain, 0.15);
                }
                FleeOutcome::RunDown => {
                    self.vitals.energy = 0.0;
                    flee_run_down = true;
                    flee_wound_note =
                        Some("It ran you down. Teeth, weight, the ground rushing up to meet you.");
                    // Affliction for a survived mauling is rolled after the
                    // lethal check below (a dead traveler takes no fever).
                }
            }
        }
        let pos_opt = self.player_pos;
        let npc_people = self
            .current_settlement_people()
            .unwrap_or(PeopleKind::Metsik);
        let player_people = self.inter_people_bias.player_people;
        let pid = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let world_tick = self.sim.as_ref().map(|s| s.world.tick);
        let outside_intervention: Option<String> = match (encounter_data, pos_opt, &pid, world_tick)
        {
            (Some(kind), Some(pos), Some(pid), Some(tick)) if kind.can_have_outside_help() => {
                if let Some(sim) = self.sim.as_ref() {
                    if let Some(region) = sim.world.regions.get(pos.region_idx) {
                        if let Some(settlement) = region.settlements.first() {
                            let rep = sim.reputation.get(pid, &settlement.id);
                            let help_threshold = 0.70_f64;
                            let avoid_threshold = 0.25_f64;
                            if rep >= help_threshold || rep <= avoid_threshold {
                                let mut hasher = self.seed.wrapping_mul(2_654_435_761);
                                hasher ^= tick;
                                hasher ^= match kind {
                                    crate::model::EncounterKind::Wildlife => 0xA1,
                                    crate::model::EncounterKind::Bandit => 0xB2,
                                    _ => 0x00,
                                };
                                let roll = (hasher.rotate_left(13) as f64) / (u32::MAX as f64);
                                // Who shows up at the right moment is luck too:
                                // the fortunate are likelier to have a friend on
                                // the road, the cursed to stand alone.
                                if roll < self.fortune.tilt_good(0.02) {
                                    Some(if rep >= help_threshold {
                                        "A passing trader steps from the road, recognizing you. The bandit recoils.".to_string()
                                    } else {
                                        "The bandit glances at you, then at the road behind. He waves you on, not bothering to clean the act.".to_string()
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(ref mut sim) = self.sim {
            if let Some(_kind) = encounter_data {
                let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                    .fork_for(&format!("encounter-journal-{}", sim.world.tick));
                let voice_text = crate::sim::journal::encounter_text(&mut rng);
                let journal_text = format!("{} — {} — {}", voice_text, action.label(), msg);
                sim.log_journal(sim.world.tick, journal_text);
                if let Some(sp) = species {
                    if sp.uncanny() {
                        // The strange gets written down in its own words —
                        // and stays deniable on the page too.
                        sim.log(
                            sim.world.tick,
                            crate::sim::journal::Voice::Scar,
                            format!("{} I have decided not to wonder about it.", sp.line()),
                        );
                    }
                }
                if let Some(ref note) = outside_intervention {
                    sim.log_journal(sim.world.tick, format!("  * {}", note));
                }
                if let Some(note) = flee_wound_note {
                    sim.log(
                        sim.world.tick,
                        crate::sim::journal::Voice::Scar,
                        note.into(),
                    );
                }
            }
            if rep_mult > 0.0 {
                let rep_delta = match action {
                    EncounterAction::Talk => 0.02,
                    EncounterAction::Trade => 0.03,
                    EncounterAction::Calm => 0.01,
                    EncounterAction::Shelter => 0.01,
                    EncounterAction::Bribe => 0.005,
                    EncounterAction::Flee => 0.0,
                    EncounterAction::Intimidate => -0.01,
                    EncounterAction::PushThrough => -0.005,
                    EncounterAction::Hunt => 0.0,
                };
                if rep_delta != 0.0 {
                    if let (Some(ref pid), Some(pos)) = (&pid, pos_opt) {
                        if let Some(region) = sim.world.regions.get(pos.region_idx) {
                            if let Some(settlement) = region.settlements.first() {
                                let sid = settlement.id.clone();
                                sim.reputation.adjust_local_biased(
                                    pid,
                                    &sid,
                                    rep_delta * rep_mult,
                                    player_people,
                                    npc_people,
                                );
                            }
                        }
                    }
                }
            }
        }
        let mut msg = if let Some(note) = outside_intervention {
            format!("{} {}", msg, note)
        } else {
            msg
        };
        // The still-sense settling a beast is the gift at work — it costs the
        // body like any other gift act (#439).
        if action == EncounterAction::Calm
            && self.gift.sense().map(|s| s.aids_calm()).unwrap_or(false)
        {
            if let Some(note) = self.use_gift() {
                msg.push_str(&note);
            }
        }
        if let Some(ref mut ps) = self.player_start {
            let combat_decay = match action {
                EncounterAction::Flee | EncounterAction::Calm | EncounterAction::Talk => 0.0,
                EncounterAction::Intimidate | EncounterAction::PushThrough => 0.08,
                EncounterAction::Hunt => 0.05,
                EncounterAction::Shelter => 0.02,
                EncounterAction::Bribe | EncounterAction::Trade => 0.01,
            };
            if combat_decay > 0.0 {
                for tool in [ItemType::Iron, ItemType::Wood, ItemType::Stone] {
                    if ps.inventory.has(tool) {
                        ps.inventory.decay(tool, combat_decay);
                    }
                }
            }
        }
        // The näšvyly's miasma (#455): the forest-fever shape keeps to the
        // thickest cover, always upwind, and the air where it stood smells of
        // rot and wet ash — and a wood-fever follows more often than not. The
        // cost lands on the body, deniable (you were deep in cold wet forest,
        // of course you took a fever). Fortune leans whether it takes.
        if species == Some(crate::model::wildlife::WildSpecies::Nashvyly) {
            let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
            let p = self.fortune.tilt_bad(0.35);
            let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x4EA5_7E11));
            if crate::rng::unit_from_hash(h) < p {
                self.afflict(
                    crate::model::Disease::Fever,
                    "The grey shape was upwind, and the rot in the air got into me. By dusk the \
                     fever has me — the wet wood and the cold, surely.",
                );
            }
        }
        // Following the sending (#455): the spectral elk leads you deeper and
        // never tires. PushThrough — give chase — and the wood takes the day
        // from you: chased to exhaustion, and on a poor turn led truly astray,
        // run down to nothing (which hands you to the collapse funnel at the end
        // of this resolution). Deniable: you chased an elk too far and lost the
        // trail. Fleeing breaks clean (the ordinary flee cost already applied).
        if enc_kind == Some(crate::model::EncounterKind::SpectralElk)
            && action == EncounterAction::PushThrough
        {
            self.vitals.energy = (self.vitals.energy - 0.55).max(0.0);
            self.vitals.hunger = (self.vitals.hunger - 0.2).max(0.0);
            let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
            let astray = self.fortune.tilt_bad(0.30);
            let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x735C_0FFE));
            if crate::rng::unit_from_hash(h) < astray {
                self.vitals.energy = 0.0;
            }
            if let Some(ref mut sim) = self.sim {
                sim.log(
                    sim.world.tick,
                    crate::sim::journal::Voice::Scar,
                    "I followed the elk past where I meant to, and the trail simply stopped. \
                     Old light through the trunks, surely. I do not rightly know how I got back."
                        .into(),
                );
            }
        }
        // Refusing the threshold-keeper's price (#455): you take the long way
        // round the mountain — hours and strength lost to the detour and the
        // cold, but no harm done. Deniable: she was only a hermit.
        if enc_kind == Some(crate::model::EncounterKind::ThresholdToll)
            && action == EncounterAction::Flee
        {
            self.advance_clock(2);
            self.vitals.energy = (self.vitals.energy - 0.2).max(0.0);
            self.vitals.hunger = (self.vitals.hunger - 0.1).max(0.0);
            if let Some(ref mut sim) = self.sim {
                sim.log(
                    sim.world.tick,
                    crate::sim::journal::Voice::Scar,
                    "I would not pay her price, and took the long way round the mountain. \
                     Hours lost, and the cold got into me. A hermit, surely."
                        .into(),
                );
            }
        }
        // The mountain stirs (#455): wait it out (Calm) and the slope goes still
        // — you pass safe, telling yourself it was only the mountain settling.
        // Cross beneath it (PushThrough) and the loose scree is treacherous: the
        // day's strength spent, and on a poor turn a turned ankle on the shifting
        // slope. Deniable: the mountain does not say.
        if enc_kind == Some(crate::model::EncounterKind::MountainStir) {
            if action == EncounterAction::Calm {
                if let Some(ref mut sim) = self.sim {
                    sim.log(
                        sim.world.tick,
                        crate::sim::journal::Voice::Scar,
                        "The slope went still when I stopped to look, and stayed still while I \
                         passed. The mountain settling, surely. I did not look back."
                            .into(),
                    );
                }
            } else if action == EncounterAction::PushThrough {
                self.vitals.energy = (self.vitals.energy - 0.3).max(0.0);
                self.vitals.hunger = (self.vitals.hunger - 0.1).max(0.0);
                let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
                let slip = self.fortune.tilt_bad(0.25);
                let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x571A_B1ED));
                if crate::rng::unit_from_hash(h) < slip {
                    self.afflict(
                        crate::model::Disease::Sprain,
                        "The scree gave under me as I crossed beneath it, and my ankle turned. \
                         Loose ground, surely. I limped the rest of the way.",
                    );
                }
            }
        }
        // Record NPC memory for this encounter
        let trust_delta = match action {
            EncounterAction::Talk => 0.02,
            EncounterAction::Trade => 0.03,
            EncounterAction::Calm => 0.01,
            EncounterAction::Shelter => 0.01,
            EncounterAction::Bribe => 0.005,
            EncounterAction::Flee => 0.0,
            EncounterAction::Intimidate => -0.02,
            EncounterAction::PushThrough => -0.01,
            EncounterAction::Hunt => 0.0,
        };
        if let Some(pos) = self.player_pos {
            if let Some(ref sim) = self.sim {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if let Some(settlement) = region.settlements.first() {
                        if let Some(person) = settlement.people.first() {
                            let person_id = person.id.clone();
                            let settlement_name = settlement.name.clone();
                            let tick = (self.clock.day * 24 + self.clock.hour) as u64;
                            if let Some(ref mut sim) = self.sim {
                                sim.npc_memories.entry(person_id).or_default().add(
                                    action,
                                    tick,
                                    settlement_name,
                                    trust_delta,
                                );
                            }
                        }
                    }
                }
            }
        }
        if let Some(kind) = encounter_data {
            self.encounter_log.push(EncounterLogEntry {
                day: self.clock.day,
                hour: self.clock.hour,
                kind,
                terrain,
                action,
                hostile: kind.is_hostile(),
            });
        }
        self.encounter = None;
        let msg_with_witness = match witness {
            WitnessLevel::Unseen | WitnessLevel::Rumored => {
                let base = msg.trim_end_matches('.').to_string();
                format!("{}. {}", base, witness.flavor())
            }
            WitnessLevel::Seen => msg,
        };
        let mut msg_with_witness = match flee_wound_note {
            Some(note) => format!("{} {}", msg_with_witness.trim_end(), note),
            None => msg_with_witness,
        };
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
        // Being run down by a thing big enough to catch you (a run-down only
        // comes from a danger-2 predator) is mortal — not a 1-in-80 stumble.
        // The chance it kills rises the more worn you came in, leans on the
        // life's hidden star, and is a real risk even at full strength. Survive
        // it and you are still emptied, handed to the ordinary collapse funnel.
        if flee_run_down {
            let wear = 1.0 - self.vitals.hunger.clamp(0.0, 1.0); // how spent you came in
            let lethal = self.fortune.tilt_bad(0.18 + 0.17 * wear);
            let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
            let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x1A2B_3C4D));
            if crate::rng::unit_from_hash(h) < lethal {
                self.die_in_wilds(
                    crate::model::DeathCause::Wounds,
                    "The beast did not let go. The wilds keep what they catch.",
                );
                return;
            }
            // Survived the mauling — but a predator's torn flesh is dirty work;
            // the wound may yet turn.
            affliction_note = self.roll_wound_affliction(species, terrain, 0.30);
        }
        if let Some(note) = affliction_note {
            msg_with_witness = format!("{}{}", msg_with_witness.trim_end(), note);
        }
        self.status_msg = Some(msg_with_witness);
        // A wound that emptied you (or a survived run-down) hands you to the
        // collapse funnel — which decides, by luck and your state, whether you
        // rise. (check_collapse self-gates on vitals and overrides the screen
        // with Collapse / continue-as-NPC when it fires.)
        if flee_run_down || self.vitals.energy <= 0.0 {
            self.check_collapse();
        }
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
                    encounters_had: self.encounters_had,
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
                    encounter_log: self.encounter_log.clone(),
                    player_farms: self.player_farms.clone(),
                    homestead_settlers: self.homestead_settlers.clone(),
                    homestead_rumored: self.homestead_rumored,
                    founding_check_day: self.founding_check_day,
                    spouse_id: self.spouse_id.clone(),
                    widowed_day: self.widowed_day,
                    household_children: self.household_children.clone(),
                    travel_debt: self.travel_debt,
                    enclaves_seen: self.enclaves_seen.clone(),
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
