use crate::model::{craft_recipes, GodName, Inventory, ItemType, Terrain, Weather};
use crate::sim::hints;

use super::*;

/// Base chance a craft is botched (materials wasted, no output) before fortune
/// leans it. Low enough that crafting is reliable, high enough that the cursed
/// feel it — luck rides this roll like every other.
const BASE_CRAFT_BOTCH_PROB: f64 = 0.10;

/// Strain added by one gift-aided craft. About three a day reaches the
/// flame-fever threshold (#427).
const GIFT_STRAIN_PER_CRAFT: f64 = 0.34;
/// The day's gift-strain (fortune-leaned) at which the flame-fever takes.
const GIFT_FLAME_THRESHOLD: f64 = 1.0;
/// Most illnesses the player carries at once (mirrors the encounter cap).
const MAX_PLAYER_ILLNESSES: usize = 2;

/// The chance a craft is spoiled, leaned by the crafter's fortune. The blessed
/// botch less, the cursed more; never certain either way.
pub(crate) fn craft_botch_chance(fortune: crate::model::Fortune) -> f64 {
    fortune.tilt_bad(BASE_CRAFT_BOTCH_PROB)
}

impl App {
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
                    let botch_p = craft_botch_chance(fortune);
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
                    if let Some(cost) = self.pay_gift_strain() {
                        msg.push_str(&cost);
                    }
                }
                self.status_msg = Some(msg);
            }
        }
    }

    /// Working the gift adds to the day's strain; past a day's measure the
    /// body protests with the flame-fever (lieska-kuume), the cursed sooner
    /// than the blessed. Returns a note when the fever takes (#427).
    fn pay_gift_strain(&mut self) -> Option<String> {
        self.gift_strain += GIFT_STRAIN_PER_CRAFT;
        // The effective load is leaned by fortune: an ill-starred body breaks
        // sooner. bad_multiplier > 1 for the cursed, < 1 for the blessed.
        let effective = self.gift_strain * self.fortune.bad_multiplier();
        if effective < GIFT_FLAME_THRESHOLD {
            return None;
        }
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
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
