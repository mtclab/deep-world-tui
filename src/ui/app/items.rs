use crate::model::{craft_recipes, GodName, Inventory, ItemType, Terrain, Weather};
use crate::sim::hints;

use super::*;

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
            if let Some(ref mut ps) = self.player_start {
                let inv = &mut ps.inventory;
                let can_craft = recipe
                    .inputs
                    .iter()
                    .all(|(item, count)| inv.get(*item) >= *count);
                if can_craft {
                    for (item, count) in &recipe.inputs {
                        inv.remove(*item, *count);
                    }
                    let output_count = recipe.output_count + bias_bonus + hearth_bonus;
                    let flavor = if hearth_bonus > 0 {
                        " A real fire beats a traveler's pot."
                    } else if bias_bonus > 0 {
                        " Skilled hands guide yours."
                    } else {
                        ""
                    };
                    inv.add(recipe.output, output_count);
                    inv.decay(ItemType::Iron, 0.03);
                    inv.decay(ItemType::Wood, 0.04);
                    self.advance_clock(2);
                    self.status_msg = Some(format!(
                        "Crafted {} (x{}) (2h){}",
                        recipe.name, output_count, flavor
                    ));
                } else {
                    self.status_msg = Some("Not enough materials".into());
                }
            }
        }
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
