use crate::model::{craft_recipes, GodName, Inventory, ItemType, Terrain, Weather};
use crate::sim::hints;

use super::*;

/// Base chance a craft is botched (materials wasted, no output) before fortune
/// leans it. Low enough that crafting is reliable, high enough that the cursed
/// feel it — luck rides this roll like every other.
const BASE_CRAFT_BOTCH_PROB: f64 = 0.10;

/// How much the craftless reduce their botch chance: the undivided, un-taxed
/// hand is steadier at ordinary work than the gifted reaching outside their
/// sense (#430). The craftless are not lesser.
const CRAFTLESS_RELIABILITY: f64 = 0.55;

/// Strain added by one gift-aided craft. About three a day reaches the
/// flame-fever threshold (#427).
const GIFT_STRAIN_PER_CRAFT: f64 = 0.34;
/// The day's gift-strain (fortune-leaned) at which the flame-fever takes.
const GIFT_FLAME_THRESHOLD: f64 = 1.0;
/// Most illnesses the player carries at once (mirrors the encounter cap).
const MAX_PLAYER_ILLNESSES: usize = 2;
/// Base chance, per gift-craft while doubly spent (flame-fever AND iron-ache),
/// that the gift ruptures forever (the rauta-huuta). Leaned by fortune (#428).
const BASE_GIFT_RUPTURE_PROB: f64 = 0.10;

/// A plain lowercase name for the ground, for status lines.
fn terrain_label(t: Terrain) -> &'static str {
    match t {
        Terrain::Forest => "forest",
        Terrain::Swamp => "mire",
        Terrain::Grass => "grassland",
        Terrain::Farmland => "farmland",
        Terrain::Coast => "shore",
        Terrain::Tundra => "tundra",
        Terrain::Mountain => "high rock",
        Terrain::Sand => "sand",
        Terrain::DeepDesert => "deep desert",
        Terrain::Cave => "dark",
        Terrain::Settlement => "streets",
        Terrain::House => "house",
        Terrain::Wall => "wall",
        Terrain::Floor => "floor",
        Terrain::Door => "doorway",
        Terrain::Hearth => "hearth",
        Terrain::Road => "road",
        Terrain::Water => "water",
    }
}

/// The chance a craft is spoiled, leaned by the crafter's fortune. The blessed
/// botch less, the cursed more; never certain either way.
pub(crate) fn craft_botch_chance(fortune: crate::model::Fortune) -> f64 {
    fortune.tilt_bad(BASE_CRAFT_BOTCH_PROB)
}

/// The botch chance for a given crafter: the craftless steadier than the
/// gifted reaching outside their sense (#430).
pub(crate) fn craft_botch_chance_for(fortune: crate::model::Fortune, craftless: bool) -> f64 {
    let base = craft_botch_chance(fortune);
    if craftless {
        base * CRAFTLESS_RELIABILITY
    } else {
        base
    }
}

impl App {
    /// How rich the ground is in medicinal plants — the supply half of
    /// herbalism (#456). Real boreal physic comes from the deep wood and the
    /// mire (bark, fungi, root, bog-herb); the open country gives some; the
    /// cold heights, the sand, and the bare stone give little or nothing.
    /// Distinct from `gather_from` (which yields a terrain's primary material).
    fn medicine_richness(terrain: Terrain) -> u32 {
        match terrain {
            Terrain::Forest | Terrain::Swamp => 2,
            Terrain::Grass | Terrain::Farmland | Terrain::Coast => 1,
            Terrain::Tundra | Terrain::Mountain | Terrain::Sand | Terrain::DeepDesert => 0,
            _ => 0,
        }
    }

    /// Forage for medicine: range the ground for healing herbs, biome- and
    /// season-true. The deep wood and the mire give freely, the open country
    /// less, the cold and the sand almost nothing; Frost thins it everywhere,
    /// a storm thins it more. Luck leans the haul, and now and then turns up a
    /// stand of true physic — a potent find. The herbalist's supply (#456).
    pub fn forage_herbs(&mut self) {
        if self.clock.time_of_day().is_dark() {
            self.status_msg = Some("Too dark to forage — the herbs hide in the dark.".into());
            return;
        }
        let terrain = self
            .player_pos
            .and_then(|pos| {
                self.sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(pos.region_idx))
                    .and_then(|r| r.terrain.get(pos.px, pos.py))
            })
            .unwrap_or(Terrain::Grass);
        let richness = Self::medicine_richness(terrain);
        if richness == 0 {
            self.status_msg = Some(format!(
                "Little physic grows in {} — barren ground for an herbalist.",
                terrain_label(terrain)
            ));
            self.advance_clock(1);
            self.vitals.energy = (self.vitals.energy - 0.04 * self.vow_work_energy_mult()).max(0.0);
            return;
        }
        let season = self.clock.season();
        let weather = self
            .player_pos
            .map(|pos| self.region_weather(pos.region_idx))
            .unwrap_or(Weather::Clear);
        let mult = season.gather_multiplier() * weather.gather_modifier();
        let mut count = (richness as f64 * mult).round() as i32;
        // Luck leans the forage: a fortunate hand turns up more, the cursed less.
        let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
        let find = crate::rng::unit_from_hash(crate::rng::mix_u64(
            self.seed ^ crate::rng::mix_u64(tick ^ 0x4EDB_05EE),
        ));
        let mut note = "";
        let mut potent = false;
        if find < self.fortune.tilt_good(0.10) {
            count += 1;
        } else if find > 1.0 - self.fortune.tilt_bad(0.10) {
            count -= 1;
        }
        // A stand of true physic — rare, fortune-leaned, richer where the land
        // is rich.
        let potent_roll = crate::rng::unit_from_hash(crate::rng::mix_u64(
            self.seed ^ crate::rng::mix_u64(tick ^ 0x9111_F12E),
        ));
        if richness >= 2 && potent_roll < self.fortune.tilt_good(0.08) {
            count += 2;
            potent = true;
            note = " You find a stand of true physic — a rare, potent harvest.";
        }
        let count = count.max(0) as u32;
        // Foraging the deep wood and mire honours the forest-keeper.
        if matches!(terrain, Terrain::Forest | Terrain::Swamp) {
            self.god_affinity.adjust(GodName::Keuru, 0.02);
        }
        self.advance_clock(1);
        self.vitals.energy = (self.vitals.energy - 0.06 * self.vow_work_energy_mult()).max(0.0);
        self.vitals.hunger = (self.vitals.hunger - 0.03).max(0.0);
        if count == 0 {
            self.status_msg = Some(format!(
                "You comb the {} but the season gives no physic.",
                terrain_label(terrain)
            ));
            return;
        }
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.add(ItemType::Herb, count);
        }
        self.play_sound(crate::audio::SoundEvent::UiClick);
        self.status_msg = Some(if potent {
            format!("You forage {count} herbs.{note}")
        } else {
            format!(
                "You forage {count} herb{}.",
                if count == 1 { "" } else { "s" }
            )
        });
    }

    pub fn gather(&mut self) {
        if self.clock.time_of_day().is_dark() {
            self.status_msg = Some("Too dark to gather".into());
            return;
        }
        let (terrain_item, terrain) = self
            .player_pos
            .and_then(|pos| {
                self.sim.as_ref().map(|sim| {
                    let region = sim.world.regions.get(pos.region_idx);
                    let t = region.and_then(|r| r.terrain.get(pos.px, pos.py));
                    let item = t.and_then(ItemType::gather_from);
                    (item, t)
                })
            })
            .unwrap_or((None, None));
        if let (Some(item), Some(terrain)) = (terrain_item, terrain) {
            match terrain {
                Terrain::Forest => {
                    self.god_affinity.adjust(GodName::Keuru, 0.03);
                    self.god_affinity.adjust(GodName::Oltzed, -0.01);
                }
                Terrain::Grass | Terrain::Farmland => {
                    self.god_affinity.adjust(GodName::Oltzed, 0.03);
                    self.god_affinity.adjust(GodName::Keuru, -0.01);
                }
                _ => {}
            }
            let season = self.clock.season();
            // Weather affects the harvest too, not just travel — a storm or
            // whiteout thins what the land gives. gather_modifier was previously
            // applied only to NPC farm growth, never to player gathering.
            let weather = self
                .player_pos
                .map(|pos| self.region_weather(pos.region_idx))
                .unwrap_or(Weather::Clear);
            let mult = season.gather_multiplier() * weather.gather_modifier();
            let pp = self.inter_people_bias.player_people;
            let people_bonus = Terrain::people_gather_bonus(pp, terrain);
            let base = 1 + people_bonus;
            let tool_bonus = if let Some(ref ps) = self.player_start {
                // A crafted Tool beats improvised iron/wood/stone.
                if ps.inventory.has(ItemType::Tool) && !ps.inventory.is_broken(ItemType::Tool) {
                    2
                } else {
                    let best_tool = [ItemType::Iron, ItemType::Wood, ItemType::Stone]
                        .into_iter()
                        .filter(|t| ps.inventory.has(*t) && !ps.inventory.is_broken(*t))
                        .max_by_key(|t| t.base_price());
                    if best_tool.is_some() {
                        1
                    } else {
                        0
                    }
                }
            } else {
                0
            };
            if tool_bonus == 2 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.use_tool(ItemType::Tool);
                }
            }
            // A gathering animal (e.g. a hound) makes the player more productive.
            let companion_gather = self
                .player_start
                .as_ref()
                .map(|ps| {
                    ps.companions
                        .iter()
                        .map(|c| c.animal.gathering_bonus())
                        .fold(0.0, f64::max)
                })
                .unwrap_or(0.0);
            let count =
                ((base + tool_bonus) as f64 * mult * (1.0 + companion_gather)).floor() as u32;
            let mut boon_msg = None;
            let patron = terrain.patron_god();
            let count = if let Some(god) = patron {
                if self.god_affinity.get(god) > 0.5 && count > 0 {
                    boon_msg = Some("The land yields generously under your hands.");
                    count + 1
                } else {
                    count
                }
            } else {
                count
            };
            // Luck in the everyday: the land gives the fortunate a little extra
            // now and then, and the cursed a lean haul — leaned by the hidden
            // star, symmetric at neutral (a find as likely as a shortfall).
            let mut count = count;
            if count > 0 {
                let tick = self.sim.as_ref().map(|s| s.world.tick).unwrap_or(0);
                let find = crate::rng::unit_from_hash(crate::rng::mix_u64(
                    self.seed ^ crate::rng::mix_u64(tick ^ 0x60A7_F1ED),
                ));
                let mishap = crate::rng::unit_from_hash(crate::rng::mix_u64(
                    self.seed ^ crate::rng::mix_u64(tick ^ 0x9111_8A2D),
                ));
                if find < self.fortune.tilt_good(0.10) {
                    count += 1;
                    boon_msg = Some("A lucky find — the land gave more than its wont.");
                } else if mishap < self.fortune.tilt_bad(0.10) {
                    count -= 1;
                    if count == 0 {
                        boon_msg = Some("A poor haul — the day's luck was thin.");
                    }
                }
            }
            if count == 0 {
                self.status_msg = Some(if weather.gather_modifier() < 0.7 {
                    format!("The {} keeps the land's gifts hidden", weather.name())
                } else {
                    format!("Too scarce in {} to gather {}", season, item.name())
                });
                return;
            }
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, count);
                let decay_items = [
                    ItemType::Wood,
                    ItemType::Stone,
                    ItemType::Iron,
                    ItemType::Cloth,
                ];
                for di in decay_items {
                    if ps.inventory.has(di) {
                        ps.inventory.decay(di, 0.05);
                    }
                }
            }
            self.advance_clock_hour();
            self.fire_hint(hints::HINT_FIRST_GATHER);
            self.play_sound(crate::audio::SoundEvent::Gather);
            self.check_quests_on_gather();
            let msg = format!("Gathered {} {} (1h, {})", count, item.name(), season);
            self.status_msg = Some(if let Some(b) = boon_msg {
                format!("{}. {}", msg, b)
            } else {
                msg
            });
        } else {
            self.status_msg = Some("Nothing to gather here".into());
        }
    }

    pub fn enter_inventory(&mut self) {
        self.screen = Screen::Inventory;
    }

    pub fn exit_inventory(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    pub fn enter_craft(&mut self) {
        self.screen = Screen::Craft { scroll: 0 };
    }

    pub fn exit_craft(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    pub fn craft_recipe(&mut self, recipe_idx: usize) {
        let player_people = self.inter_people_bias.player_people;
        let bias_bonus = self.current_settlement_people().map_or(0u32, |npc_people| {
            if self.inter_people_bias.effective_bias(npc_people) > 0.10 {
                1
            } else {
                0
            }
        });
        let recipes: Vec<_> = craft_recipes()
            .into_iter()
            .filter(|r| r.people.is_none() || r.people == Some(player_people))
            .collect();
        // Meal-type crafts at your own hearth yield one more: a kota (or any
        // homestead with a hearth-pit) cooks better than a traveler's pot.
        let at_own_hearth = self.own_hearth_here();
        if let Some(recipe) = recipes.get(recipe_idx) {
            let hearth_bonus = if at_own_hearth && recipe.output == ItemType::Food {
                1
            } else {
                0
            };
            let fortune = self.fortune;
            let seed = self.seed;
            let (day, hour) = (self.clock.day, self.clock.hour);
            // A gifted crafter masters the work their sense answers: it cannot
            // botch and yields a little more — but the gift costs the body, paid
            // after the craft (#427).
            let gift_aids = self
                .gift
                .sense()
                .map(|s| s.aids_craft(recipe))
                .unwrap_or(false);
            // The craftless are not lesser (#430): the undivided, un-taxed hand
            // is steadier at ordinary work — it botches less, and it never pays
            // the gift's bodily price. Worth the gifted house cannot count.
            let craftless = !self.gift.has();
            // (msg, did_craft, gift_used) — did_craft gates the clock cost,
            // gift_used the bodily strain. Computed inside the inventory borrow,
            // applied after it ends to avoid re-borrowing self while `inv` is live.
            let outcome: Option<(String, bool, bool)> = if let Some(ref mut ps) = self.player_start
            {
                let inv = &mut ps.inventory;
                let can_craft = recipe
                    .inputs
                    .iter()
                    .all(|(item, count)| inv.get(*item) >= *count);
                if can_craft {
                    // The work can be spoiled: a botch wastes the materials and
                    // yields nothing. Fortune leans the odds — the blessed
                    // botch less, the cursed more — but caution never makes a
                    // sure thing of it. Deterministic from seed + day/hour.
                    let mut rng =
                        crate::rng::SeedRng::new(seed ^ crate::rng::fnv1a_hash(&recipe.name))
                            .fork_for(&format!("craft-botch-{day}-{hour}"));
                    let botch_p = craft_botch_chance_for(fortune, craftless);
                    // The gift does not botch the work it was born to.
                    let botched = !gift_aids && rng.gen_f64() < botch_p;
                    for (item, count) in &recipe.inputs {
                        inv.remove(*item, *count);
                    }
                    inv.decay(ItemType::Iron, 0.03);
                    inv.decay(ItemType::Wood, 0.04);
                    if botched {
                        Some((
                            format!(
                                "The {} is spoiled in the making — the materials are wasted. (2h)",
                                recipe.name
                            ),
                            true,
                            false,
                        ))
                    } else {
                        let gift_bonus = if gift_aids { 1 } else { 0 };
                        let output_count =
                            recipe.output_count + bias_bonus + hearth_bonus + gift_bonus;
                        let flavor = if gift_aids {
                            " The work answers your gift — clean, and more of it."
                        } else if hearth_bonus > 0 {
                            " A real fire beats a traveler's pot."
                        } else if bias_bonus > 0 {
                            " Skilled hands guide yours."
                        } else {
                            ""
                        };
                        inv.add(recipe.output, output_count);
                        Some((
                            format!("Crafted {} (x{}) (2h){}", recipe.name, output_count, flavor),
                            true,
                            gift_aids,
                        ))
                    }
                } else {
                    Some(("Not enough materials".into(), false, false))
                }
            } else {
                None
            };
            if let Some((msg, did_craft, gift_used)) = outcome {
                if did_craft {
                    self.advance_clock(2);
                }
                let mut msg = msg;
                if gift_used {
                    if let Some(note) = self.use_gift() {
                        msg.push_str(&note);
                    }
                }
                self.status_msg = Some(msg);
            }
        }
    }

    /// Sit down and treat your own sickness with what you carry (#451). The
    /// counter the post-Fall mortality (#448) left thin: a brewed herb-physic
    /// eases ANY fever (the field answer a salve never gave), a salve answers
    /// the wound-illnesses strongest, a bandage dresses what it can. Each eases
    /// the case and shortens its course — fewer days sick is fewer death-rolls.
    /// A lucky brew (and the root-eye's healing gift, #452) does more. Costs an
    /// hour and one remedy.
    pub fn tend_illness(&mut self) {
        use crate::model::{Disease, ItemType};
        let has_illness = self
            .player_start
            .as_ref()
            .is_some_and(|ps| !ps.person.illnesses.is_empty());
        if !has_illness {
            self.status_msg = Some("You are not sick — there is nothing to tend.".into());
            return;
        }
        // The root-eye reads the herb true: the healer's gift (#452).
        let root_eye = self.gift.sense() == Some(crate::model::CraftSense::RootEye);
        // A lucky hand brews a better physic — fortune leans the easing.
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let lucky = {
            let h = crate::rng::mix_u64(self.seed ^ crate::rng::mix_u64(tick ^ 0x7EA1_05EE));
            crate::rng::unit_from_hash(h) < self.fortune.tilt_good(0.30)
        };
        let is_wound =
            |d: Disease| matches!(d, Disease::Infection | Disease::Venom | Disease::Sprain);

        let used: Option<&'static str> = if let Some(ref mut ps) = self.player_start {
            let has_wound = ps.person.illnesses.iter().any(|d| is_wound(d.disease));
            if has_wound && ps.inventory.remove(ItemType::Salve, 1) {
                for d in ps
                    .person
                    .illnesses
                    .iter_mut()
                    .filter(|d| is_wound(d.disease))
                {
                    d.tend_strong();
                    if root_eye || lucky {
                        d.tend_strong();
                    }
                }
                Some("You work a salve deep into the wound — it answers, and the heat goes out of it.")
            } else if ps.inventory.remove(ItemType::Herb, 1) {
                // A herb-draught: the only field counter to a fever.
                for d in ps.person.illnesses.iter_mut() {
                    d.tend();
                    if root_eye || lucky {
                        d.tend();
                    }
                }
                Some("You brew what physic the herbs allow and drink it down. The fever loosens its grip a little.")
            } else if ps.inventory.remove(ItemType::Bandage, 1) {
                for d in ps.person.illnesses.iter_mut() {
                    d.tend();
                    if root_eye {
                        d.tend();
                    }
                }
                Some("You dress what you can with a clean bandage.")
            } else {
                None
            }
        } else {
            None
        };

        let Some(base) = used else {
            self.status_msg =
                Some("You have nothing to treat it with — no herb, no salve, no bandage.".into());
            return;
        };
        let mut msg = base.to_string();

        // The root-eye's gift can break a mild sickness outright — and it costs
        // the body like any other working of the gift (#427/#428).
        if root_eye {
            let cured = if let Some(ref mut ps) = self.player_start {
                let before = ps.person.illnesses.len();
                // Only the milder, non-acute fevers yield to a single tending.
                ps.person.illnesses.retain(|d| {
                    !(lucky
                        && matches!(
                            d.disease,
                            Disease::Fever | Disease::WinterCough | Disease::MarshFever
                        ))
                });
                before != ps.person.illnesses.len()
            } else {
                false
            };
            if cured {
                msg.push_str(" Under your hands the sickness simply lets go — the root-eye reads what the body needs.");
            }
            if let Some(note) = self.use_gift() {
                msg.push_str(&note);
            }
        }

        if let Some(sim) = self.sim.as_mut() {
            sim.log(
                tick,
                crate::sim::journal::Voice::Scar,
                "I tended my own sickness as best I knew. The body keeps its own counsel.".into(),
            );
        }
        self.advance_clock(1);
        self.status_msg = Some(msg);
    }

    /// Use the gift for any act (craft, trade, calm — #439): surface it the
    /// first time (#431), then pay the body (#427/#428). Returns the combined
    /// note, if the body answered or the gift first showed.
    pub(crate) fn use_gift(&mut self) -> Option<String> {
        let sense = self.gift.sense()?;
        let mut note = String::new();
        if !self.gift_revealed {
            self.gift_revealed = true;
            note.push(' ');
            note.push_str(sense.revelation());
            let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
            if let Some(sim) = self.sim.as_mut() {
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Scar,
                    sense.revelation().to_string(),
                );
            }
        }
        if let Some(cost) = self.pay_gift_strain() {
            note.push_str(&cost);
        }
        if note.is_empty() {
            None
        } else {
            Some(note)
        }
    }

    /// Working the gift adds to the day's strain. Reaching for it while already
    /// spent — the flame-fever and the iron-ache both on you — risks the
    /// rauta-huuta: the boundary breaks and the sense is gone forever (#428).
    /// Short of that, a day's overuse brings the flame-fever (#427), the cursed
    /// sooner than the blessed. Returns a note when the body answers.
    fn pay_gift_strain(&mut self) -> Option<String> {
        self.gift_strain += GIFT_STRAIN_PER_CRAFT;
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let bad = self.fortune.bad_multiplier();

        // The boundary: doubly spent, the gift can rupture.
        let (has_flame, has_ache) = self
            .player_start
            .as_ref()
            .map(|ps| {
                let f = ps
                    .person
                    .illnesses
                    .iter()
                    .any(|d| d.disease == crate::model::Disease::FlameFever);
                let a = ps
                    .person
                    .illnesses
                    .iter()
                    .any(|d| d.disease == crate::model::Disease::IronAche);
                (f, a)
            })
            .unwrap_or((false, false));
        if has_flame && has_ache && self.gift.has() {
            let roll = crate::rng::unit_from_hash(crate::rng::mix_u64(
                self.seed ^ crate::rng::mix_u64(tick ^ 0x9217_3AC0),
            ));
            let rupture_p = (BASE_GIFT_RUPTURE_PROB * bad).clamp(0.0, 0.5);
            if roll < rupture_p {
                self.gift = crate::model::Gift::NONE;
                self.gift_strain = 0.0;
                self.gift_overworked_days = 0;
                if let Some(sim) = self.sim.as_mut() {
                    sim.log(
                        tick,
                        crate::sim::journal::Voice::Scar,
                        "The iron-scream, once — and then a silence where the song had always \
                         been. I reached too far. It will not come again."
                            .into(),
                    );
                }
                return Some(
                    " The iron-scream — then silence. The gift is spent, and will not return."
                        .into(),
                );
            }
        }

        // Short of rupture: a day's overuse brings the flame-fever.
        let effective = self.gift_strain * bad;
        if effective < GIFT_FLAME_THRESHOLD {
            return None;
        }
        let ps = self.player_start.as_mut()?;
        let already = ps
            .person
            .illnesses
            .iter()
            .any(|d| d.disease == crate::model::Disease::FlameFever);
        if already || ps.person.illnesses.len() >= MAX_PLAYER_ILLNESSES {
            return None;
        }
        ps.person.illnesses.push(crate::model::ActiveDisease::new(
            crate::model::Disease::FlameFever,
            tick,
        ));
        Some(" The gift turns on you — flame-fever rises.".into())
    }

    pub fn player_inventory(&self) -> Inventory {
        self.player_start
            .as_ref()
            .map(|ps| ps.inventory.clone())
            .unwrap_or_default()
    }

    /// Put goods into the house. The stash stays with the building — and the
    /// building stays with the line: heirs inherit the house and what's in it.
    pub fn stash_item(&mut self, item: ItemType, count: u32) {
        let held = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.get(item))
            .unwrap_or(0);
        let n = count.min(held);
        if n == 0 {
            self.status_msg = Some(format!("No {} to stash.", item.name()));
            return;
        }
        let Some(store) = self.own_store_here() else {
            self.status_msg = Some("No house of yours here to keep things in.".into());
            return;
        };
        store.stash.add(item, n);
        let label = store.kind.label().to_string();
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(item, n);
        }
        self.status_msg = Some(format!("Stashed {} {} in the {}.", n, item.name(), label));
    }

    /// Take goods back out of the house.
    pub fn take_item(&mut self, item: ItemType, count: u32) {
        let Some(store) = self.own_store_here() else {
            self.status_msg = Some("No house of yours here.".into());
            return;
        };
        let kept = store.stash.get(item);
        let n = count.min(kept);
        if n == 0 {
            self.status_msg = Some(format!("No {} in the stash.", item.name()));
            return;
        }
        store.stash.remove(item, n);
        let label = store.kind.label().to_string();
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.add(item, n);
        }
        self.status_msg = Some(format!("Took {} {} from the {}.", n, item.name(), label));
    }
}

#[cfg(test)]
mod tests {
    use super::craft_botch_chance;
    use crate::model::Fortune;

    #[test]
    fn cursed_botch_more_than_blessed() {
        let cursed = craft_botch_chance(Fortune::from_value(-1.0));
        let plain = craft_botch_chance(Fortune::from_value(0.0));
        let blessed = craft_botch_chance(Fortune::from_value(1.0));
        assert!(
            cursed > plain && plain > blessed,
            "botch should rise with ill fortune: cursed={cursed} plain={plain} blessed={blessed}"
        );
    }

    #[test]
    fn the_craftless_hand_is_steadier() {
        use super::craft_botch_chance_for;
        // At the same luck, the undivided craftless hand botches less than a
        // gifted one reaching outside its sense (#430).
        for v in [-1.0, 0.0, 1.0] {
            let f = Fortune::from_value(v);
            let craftless = craft_botch_chance_for(f, true);
            let gifted = craft_botch_chance_for(f, false);
            assert!(craftless < gifted, "craftless should botch less at {v}");
            assert!(craftless > 0.0, "still a chance, never impossible");
        }
    }

    #[test]
    fn botch_chance_stays_a_chance() {
        // Never a certainty, never impossible — luck leans, it does not lock.
        for v in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let p = craft_botch_chance(Fortune::from_value(v));
            assert!(
                p > 0.0 && p < 1.0,
                "botch chance {p} out of (0,1) at fortune {v}"
            );
        }
    }
}
