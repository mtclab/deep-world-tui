use crate::model::{GodName, ItemType, Terrain};
use crate::rng::SeedRng;
use crate::sim::hints;

use super::*;

impl App {
    /// Maintain a weathering structure under the player (resets its decay clock).
    /// Returns true if a maintainable structure was here (handled), false if not.
    fn maintain_structure_here(&mut self) -> bool {
        let pos = match self.player_pos {
            Some(p) => p,
            None => return false,
        };
        let (px, py) = (pos.px as u32, pos.py as u32);
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let here = |st: &crate::sim::structures::Structure| {
            st.x == px && st.y == py && !st.is_npc_built && st.kind.decay_years().is_some()
        };
        let found = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(pos.region_idx))
            .is_some_and(|r| r.structures.iter().any(here));
        if !found {
            return false;
        }
        if !self
            .player_start
            .as_ref()
            .is_some_and(|ps| ps.inventory.has(ItemType::Wood))
        {
            self.status_msg = Some("Need 1 Wood to maintain this structure.".into());
            return true;
        }
        let mut label = "structure";
        if let Some(ref mut sim) = self.sim {
            if let Some(region) = sim.world.regions.get_mut(pos.region_idx) {
                // Match the `here` detect predicate exactly — only player-built,
                // decaying structures are maintainable. Filtering on position
                // alone would refresh (and mislabel/charge for) a co-located
                // NPC-built or non-decaying structure.
                for st in region.structures.iter_mut().filter(|st| {
                    st.x == px && st.y == py && !st.is_npc_built && st.kind.decay_years().is_some()
                }) {
                    st.last_maintenance_tick = tick;
                    label = st.kind.label();
                }
            }
            for st in sim.structures.iter_mut().filter(|st| {
                st.region_idx == pos.region_idx
                    && st.x == px
                    && st.y == py
                    && !st.is_npc_built
                    && st.kind.decay_years().is_some()
            }) {
                st.last_maintenance_tick = tick;
            }
        }
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.remove(ItemType::Wood, 1);
        }
        self.advance_clock(2);
        self.status_msg = Some(format!("Maintained {label} (1 Wood, 2h)"));
        true
    }

    /// Plot allowance from the homestead tier: a cabin works one field, a
    /// longhouse two, a home three. No homestead, no farm.
    fn plot_allowance_near(&self, region_idx: usize, px: u32, py: u32) -> usize {
        use crate::sim::structures::BuildKind;
        let Some(region) = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(region_idx))
        else {
            return 0;
        };
        region
            .structures
            .iter()
            .filter(|st| !st.is_npc_built && st.x.abs_diff(px) <= 2 && st.y.abs_diff(py) <= 2)
            .map(|st| match st.kind {
                BuildKind::Cabin => 1,
                BuildKind::Longhouse => 2,
                BuildKind::Home => 3,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }

    /// Plant a field on this tile. Costs a measure of Food as seed — nothing
    /// from nothing — and farming against the forest edge costs standing with
    /// the forest's people and its god (the Kaelva tension, in miniature).
    pub fn plant(&mut self) {
        use crate::model::economy::{CropType, PlayerFarm};
        let Some(pos) = self.player_pos else {
            self.status_msg = Some("No position".into());
            return;
        };
        let (region_idx, px, py) = (pos.region_idx, pos.px as u32, pos.py as u32);
        let terrain = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(region_idx))
            .and_then(|r| r.terrain.get(pos.px, pos.py))
            .unwrap_or(Terrain::Grass);
        if !matches!(terrain, Terrain::Farmland | Terrain::Grass) {
            self.status_msg = Some("This ground will not take a crop.".into());
            return;
        }
        if self.clock.season() == crate::model::Season::Frost {
            self.status_msg = Some("Nothing planted in the frost survives the week.".into());
            return;
        }
        if self
            .player_farms
            .iter()
            .any(|f| f.region_idx == region_idx && f.x == px && f.y == py)
        {
            self.status_msg = Some("A field already works this ground.".into());
            return;
        }
        let allowance = self.plot_allowance_near(region_idx, px, py);
        let worked = self
            .player_farms
            .iter()
            .filter(|f| f.region_idx == region_idx)
            .count();
        if allowance == 0 {
            self.status_msg =
                Some("A field needs a homestead — raise a cabin within two tiles first.".into());
            return;
        }
        if worked >= allowance {
            self.status_msg = Some(format!(
                "The homestead can work {} field{} — no hands for more.",
                allowance,
                if allowance == 1 { "" } else { "s" }
            ));
            return;
        }
        // Seed: nothing from nothing.
        let paid = self
            .player_start
            .as_mut()
            .map(|ps| ps.inventory.remove(ItemType::Food, 1))
            .unwrap_or(false);
        if !paid {
            self.status_msg = Some("No seed to plant (needs 1 Food).".into());
            return;
        }
        // Forest-edge tension: clearing ground against the wood is not free.
        let forest_adjacent = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(region_idx))
            .map(|r| {
                let (ix, iy) = (pos.px as i32, pos.py as i32);
                (-1..=1).any(|dy: i32| {
                    (-1..=1).any(|dx: i32| {
                        let (nx, ny) = (ix + dx, iy + dy);
                        nx >= 0
                            && ny >= 0
                            && r.terrain.get(nx as usize, ny as usize) == Some(Terrain::Forest)
                    })
                })
            })
            .unwrap_or(false);
        if forest_adjacent {
            self.god_affinity.adjust(GodName::Keuru, -0.05);
            self.inter_people_bias
                .mod_toward(crate::model::PeopleKind::Metsik, -0.03);
            if let Some(ref mut sim) = self.sim {
                let t = sim.world.tick;
                sim.log(
                    t,
                    crate::sim::journal::Voice::Scar,
                    "I put the plough to ground at the forest's edge. The wood watched me do it."
                        .into(),
                );
            }
        }
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let crop = CropType::all()
            .into_iter()
            .max_by(|a, b| {
                a.regional_suitability(terrain)
                    .partial_cmp(&b.regional_suitability(terrain))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(CropType::Grain);
        let farm_seed = self.seed.wrapping_add(tick) ^ ((px as u64) << 16) ^ (py as u64);
        self.player_farms.push(PlayerFarm {
            farm: crate::model::economy::Farm::new(farm_seed, crop, tick, terrain),
            region_idx,
            x: px,
            y: py,
        });
        self.advance_clock(2);
        self.status_msg = Some(format!(
            "Planted {} (1 Food as seed, 2h){}",
            crop.name(),
            if forest_adjacent {
                " — the forest marks the clearing"
            } else {
                ""
            }
        ));
    }

    /// Harvest a ready field on this tile.
    pub fn harvest(&mut self) {
        let Some(pos) = self.player_pos else {
            self.status_msg = Some("No position".into());
            return;
        };
        let (region_idx, px, py) = (pos.region_idx, pos.px as u32, pos.py as u32);
        let Some(idx) = self
            .player_farms
            .iter()
            .position(|f| f.region_idx == region_idx && f.x == px && f.y == py)
        else {
            self.status_msg = Some("No field of yours here.".into());
            return;
        };
        if !self.player_farms[idx].farm.is_ready() {
            self.status_msg = Some(format!(
                "The {} stands {} — not ready.",
                self.player_farms[idx].farm.crop.name(),
                self.player_farms[idx].farm.stage.name()
            ));
            return;
        }
        // A harvest is sacks, not meals: the base yield is a settlement-scale
        // unit; for one pair of hands it converts at 5 meals the sack. The
        // math has to clear subsistence (~4 meals/day) or no farmer in the
        // world could feed themself — the first test draft proved they
        // couldn't.
        let yield_n = self.player_farms[idx].farm.harvest_yield() * 5;
        let crop = self.player_farms[idx].farm.crop;
        self.player_farms.remove(idx);
        if let Some(ref mut ps) = self.player_start {
            ps.inventory.add(ItemType::Food, yield_n);
        }
        self.advance_clock(3);
        self.status_msg = Some(format!(
            "Harvested {} — {} Food (3h). The field wants seed again.",
            crop.name(),
            yield_n
        ));
    }

    /// Player fields grow with the days; frost kills what stands.
    pub(super) fn tick_player_farms(&mut self) {
        let frost = self.clock.season() == crate::model::Season::Frost;
        let Some(ref sim) = self.sim else { return };
        let tick = sim.world.tick;
        let weathers: Vec<_> = sim.world.regions.iter().map(|r| r.weather).collect();
        let mut killed = false;
        self.player_farms.retain_mut(|f| {
            if frost {
                killed = true;
                return false;
            }
            let w = weathers
                .get(f.region_idx)
                .copied()
                .unwrap_or(crate::model::Weather::Clear);
            f.farm.update_growth(tick, w);
            true
        });
        if killed {
            self.status_msg = Some("The frost took the standing crops.".into());
        }
    }

    /// If you feed a place, people come (#345). A homestead on wild ground
    /// with a working field, full stores, fit land, and a name the nearest
    /// town doesn't spit at draws settlers — rumor first, then wagons. The
    /// settlers are real: drawn out of neighboring settlements and counted
    /// off their rolls. At ~12 souls the world recognizes the place: a hamlet
    /// is born, named by the region's naming tradition — the founding goes in
    /// the record; the player gets fame, not naming rights.
    pub fn tick_founding(&mut self) {
        use crate::sim::structures::BuildKind;
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let pocket_food = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.get(ItemType::Food))
            .unwrap_or(0);
        // 1. Find a qualifying homestead.
        let site = {
            let Some(ref sim) = self.sim else { return };
            let mut found: Option<(usize, usize, usize)> = None;
            'outer: for (ri, region) in sim.world.regions.iter().enumerate() {
                for st in &region.structures {
                    if st.is_npc_built
                        || !matches!(
                            st.kind,
                            BuildKind::Cabin | BuildKind::Longhouse | BuildKind::Home
                        )
                    {
                        continue;
                    }
                    let (sx, sy) = (st.x as usize, st.y as usize);
                    // A working field within two tiles — a homestead, not a hut.
                    let farmed = self.player_farms.iter().any(|f| {
                        f.region_idx == ri && f.x.abs_diff(st.x) <= 2 && f.y.abs_diff(st.y) <= 2
                    });
                    if !farmed {
                        continue;
                    }
                    // Wild ground: no settlement within twelve tiles — the
                    // same ground the old six covered on the coarser grid.
                    let near_town = (sy.saturating_sub(12)..(sy + 13).min(region.terrain.height))
                        .any(|ty| {
                            (sx.saturating_sub(12)..(sx + 13).min(region.terrain.width))
                                .any(|tx| region.terrain.get(tx, ty) == Some(Terrain::Settlement))
                        });
                    if near_town {
                        continue;
                    }
                    // Land fit to feed more mouths than yours.
                    if region.game_richness <= 0.5 {
                        continue;
                    }
                    // Full stores: stash and pockets together past a winter's margin.
                    if st.stash.get(ItemType::Food) + pocket_food <= 30 {
                        continue;
                    }
                    // Not ill-regarded where word of you comes from.
                    let standing = region
                        .settlements
                        .first()
                        .map(|s| sim.reputation.get(&player_id, &s.id))
                        .unwrap_or(0.5);
                    if standing < 0.45 {
                        continue;
                    }
                    found = Some((ri, sx, sy));
                    break 'outer;
                }
            }
            found
        };
        let Some((ri, x, y)) = site else { return };
        // 2. The rumor precedes the wagons.
        if !self.homestead_rumored {
            self.homestead_rumored = true;
            if let Some(ref mut sim) = self.sim {
                let t = sim.world.tick;
                sim.log(
                    t,
                    crate::sim::journal::Voice::Rumor,
                    "Families on the road, they say — asking after the homestead that feeds \
                     travelers."
                        .into(),
                );
            }
            self.status_msg = Some("Word of the homestead is on the roads.".into());
            return;
        }
        let day = self.clock.day;
        let mut rng = SeedRng::new(self.seed).fork_for(&format!("founding-{day}"));
        // 3. A wave arrives — drawn out of real settlements, counted off their rolls.
        if self.homestead_settlers.len() < 12 {
            let n = 2 + rng.gen_range(3);
            if let Some(ref mut sim) = self.sim {
                let drawn = crate::sim::founding::draw_settlers(sim, ri, n, &mut rng);
                if drawn.is_empty() {
                    return;
                }
                let arrived = drawn.len();
                self.homestead_settlers.extend(drawn);
                let total = self.homestead_settlers.len();
                let t = sim.world.tick;
                sim.log(
                    t,
                    crate::sim::journal::Voice::Encounter,
                    format!(
                        "{} more souls have camped by my fields — {} now, waiting for the \
                         place to become somewhere.",
                        arrived, total
                    ),
                );
                self.status_msg = Some(format!("Settlers by the homestead: {total} souls."));
            }
        }
        // 4. At ~12 souls the world recognizes the place.
        if self.homestead_settlers.len() >= 12 {
            let settlers = std::mem::take(&mut self.homestead_settlers);
            if let Some(ref mut sim) = self.sim {
                if let Some((id, name)) =
                    crate::sim::founding::spawn_settlement(sim, ri, x, y, settlers, &mut rng)
                {
                    // Founder status: the place holds the player in the
                    // highest regard it can — standing, not naming rights.
                    if !player_id.is_empty() {
                        sim.reputation.adjust_settlement(&player_id, &id, 0.5);
                    }
                    let t = sim.world.tick;
                    sim.log(
                        t,
                        crate::sim::journal::Voice::Scar,
                        format!(
                            "They are calling it {}. It grew up around my fields. The record \
                             will keep my name; the place keeps its own.",
                            name
                        ),
                    );
                    self.homestead_rumored = false;
                    self.status_msg = Some(format!("A hamlet is born: {name}."));
                }
            }
        }
    }

    /// Whether the player owns a completed Cabin/Longhouse/Home anywhere —
    /// an oath needs a roof to live under.
    pub(super) fn owns_a_home(&self) -> bool {
        use crate::sim::structures::BuildKind;
        self.sim
            .as_ref()
            .map(|s| {
                s.world.regions.iter().any(|r| {
                    r.structures.iter().any(|st| {
                        !st.is_npc_built
                            && matches!(
                                st.kind,
                                BuildKind::Cabin | BuildKind::Longhouse | BuildKind::Home
                            )
                    })
                })
            })
            .unwrap_or(false)
    }

    /// Births of the house (#363), on the ten-day calendar: a living
    /// marriage and a fed larder now and then bring a child. The pace is
    /// ordinary and the house has limits — no dynasty factories.
    pub fn tick_household(&mut self) {
        if self.spouse_id.is_none() || self.household_children.len() >= 4 {
            return;
        }
        let fed = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.get(ItemType::Food) >= 10)
            .unwrap_or(false);
        if !fed {
            return;
        }
        let day = self.clock.day;
        let mut rng = SeedRng::new(self.seed).fork_for(&format!("household-{day}"));
        if rng.gen_range(4) != 0 {
            return;
        }
        let people = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.people.clone())
            .unwrap_or_default();
        let name = self
            .sim
            .as_ref()
            .and_then(|s| crate::gen::name::generate_name(&mut rng, &people, "", &s.charts).ok())
            .unwrap_or_else(|| "the little one".into());
        self.household_children.push(crate::model::HouseholdChild {
            name: name.clone(),
            born_day: day,
        });
        if let Some(ref mut sim) = self.sim {
            let t = sim.world.tick;
            sim.log(
                t,
                crate::sim::journal::Voice::Scar,
                format!(
                    "Born to us: {}. The house is louder and better for it.",
                    name
                ),
            );
        }
        self.status_msg = Some(format!("A child is born to the house: {name}."));
    }

    /// Longhouse waystation (#347): an own completed Longhouse within two
    /// tiles of a road shelters travelers — the dying waystation network,
    /// one node quietly restarted. Each ten-day check it earns a small
    /// trickle of standing with the region's settlement, and now and then
    /// word of it travels.
    pub fn tick_waystations(&mut self) {
        use crate::sim::structures::BuildKind;
        let player_id = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        if player_id.is_empty() {
            return;
        }
        let day = self.clock.day;
        let Some(ref mut sim) = self.sim else { return };
        let seed = sim.world.seed;
        let mut earned: Vec<(String, String)> = Vec::new(); // (settlement_id, region name)
        for region in sim.world.regions.iter() {
            for st in &region.structures {
                if st.is_npc_built || st.kind != BuildKind::Longhouse {
                    continue;
                }
                let (sx, sy) = (st.x as usize, st.y as usize);
                let near_road =
                    (sy.saturating_sub(2)..(sy + 3).min(region.terrain.height)).any(|ty| {
                        (sx.saturating_sub(2)..(sx + 3).min(region.terrain.width))
                            .any(|tx| region.terrain.get(tx, ty) == Some(Terrain::Road))
                    });
                if !near_road {
                    continue;
                }
                if let Some(s) = region.settlements.first() {
                    earned.push((s.id.clone(), region.name.clone()));
                }
            }
        }
        for (sid, region_name) in earned {
            sim.reputation.adjust_settlement(&player_id, &sid, 0.02);
            let mut rng = SeedRng::new(seed).fork_for(&format!("waystation-{day}-{sid}"));
            if rng.gen_range(3) == 0 {
                let t = sim.world.tick;
                sim.log(
                    t,
                    crate::sim::journal::Voice::Rumor,
                    format!(
                        "The long hall on the {} road took in the storm-bound again, \
                         they say. Travelers speak well of it.",
                        region_name
                    ),
                );
            }
        }
    }

    /// The player's own storing structure (Cabin+) on this tile, if any.
    pub(super) fn own_store_here(&mut self) -> Option<&mut crate::sim::structures::Structure> {
        use crate::sim::structures::BuildKind;
        let pos = self.player_pos?;
        let region = self
            .sim
            .as_mut()
            .and_then(|s| s.world.regions.get_mut(pos.region_idx))?;
        region.structures.iter_mut().find(|st| {
            !st.is_npc_built
                && st.x == pos.px as u32
                && st.y == pos.py as u32
                && matches!(
                    st.kind,
                    BuildKind::Cabin | BuildKind::Longhouse | BuildKind::Home
                )
        })
    }

    /// Standing at one's own hearth: a completed Kota or better on this tile.
    pub(super) fn own_hearth_here(&self) -> bool {
        use crate::sim::structures::BuildKind;
        let Some(pos) = self.player_pos else {
            return false;
        };
        self.sim
            .as_ref()
            .and_then(|s| s.world.regions.get(pos.region_idx))
            .map(|r| {
                r.structures.iter().any(|st| {
                    !st.is_npc_built
                        && st.x == pos.px as u32
                        && st.y == pos.py as u32
                        && matches!(
                            st.kind,
                            BuildKind::Kota
                                | BuildKind::Cabin
                                | BuildKind::Longhouse
                                | BuildKind::Home
                        )
                })
            })
            .unwrap_or(false)
    }

    /// The god of one's own shrine within a tile of here, if any.
    pub(super) fn own_shrine_god_near(&self) -> Option<GodName> {
        use crate::sim::structures::BuildKind;
        let pos = self.player_pos?;
        let region = self.sim.as_ref()?.world.regions.get(pos.region_idx)?;
        region
            .structures
            .iter()
            .find(|st| {
                !st.is_npc_built
                    && st.kind == BuildKind::Shrine
                    && st.x.abs_diff(pos.px as u32) <= 1
                    && st.y.abs_diff(pos.py as u32) <= 1
            })
            .and_then(|st| st.name.as_deref())
            .and_then(GodName::from_label)
    }

    /// An own structure of the given kind within `range` tiles of the player.
    pub(super) fn own_structure_near(
        &self,
        kind: crate::sim::structures::BuildKind,
        range: u32,
    ) -> bool {
        let Some(pos) = self.player_pos else {
            return false;
        };
        self.sim
            .as_ref()
            .and_then(|s| s.world.regions.get(pos.region_idx))
            .map(|r| {
                r.structures.iter().any(|st| {
                    !st.is_npc_built
                        && st.kind == kind
                        && st.x.abs_diff(pos.px as u32) <= range
                        && st.y.abs_diff(pos.py as u32) <= range
                })
            })
            .unwrap_or(false)
    }

    /// A laid path structure (trail or footbridge) standing on this tile.
    pub(super) fn path_structure_at(
        &self,
        region_idx: usize,
        x: usize,
        y: usize,
    ) -> Option<crate::sim::structures::BuildKind> {
        use crate::sim::structures::BuildKind;
        let region = self.sim.as_ref()?.world.regions.get(region_idx)?;
        region
            .structures
            .iter()
            .find(|st| {
                st.x == x as u32
                    && st.y == y as u32
                    && matches!(st.kind, BuildKind::Trail | BuildKind::Footbridge)
            })
            .map(|st| st.kind)
    }

    pub fn start_build(&mut self) {
        self.start_build_kind(None);
    }

    /// Work a labor-gated build site on this tile: a day's labor (8h)
    /// advances it. Frost doubles the groundwork.
    pub fn work_site(&mut self) {
        let Some(pos) = self.player_pos else {
            self.status_msg = Some("No position".into());
            return;
        };
        let frost = self.clock.season() == crate::model::Season::Frost;
        let Some(ref mut sim) = self.sim else { return };
        let Some(site) = sim.build_sites.iter_mut().find(|s| {
            s.region_idx == pos.region_idx
                && ((s.x == pos.px as u32 && s.y == pos.py as u32)
                    // A footbridge is worked from the bank beside it.
                    || (s.kind == crate::sim::structures::BuildKind::Footbridge
                        && s.x.abs_diff(pos.px as u32) <= 1
                        && s.y.abs_diff(pos.py as u32) <= 1))
        }) else {
            self.status_msg = Some("No build site here.".into());
            return;
        };
        let gained = if frost { 4 } else { 8 };
        site.hours_done += gained;
        let kind = site.kind;
        let done = site.hours_done;
        let needed = kind.build_hours();
        self.advance_clock(8);
        self.vitals.energy = (self.vitals.energy - 0.15).max(0.0);
        self.status_msg = Some(if done >= needed {
            format!(
                "{} nearly stands — the next hour will finish it.",
                kind.label()
            )
        } else {
            format!(
                "Worked the {} site ({}h of {}h{})",
                kind.label(),
                done.min(needed),
                needed,
                if frost { ", slow in the frost" } else { "" }
            )
        });
        self.fire_hint(hints::HINT_FIRST_STRUCTURE);
    }

    pub fn start_build_kind(&mut self, wanted: Option<crate::sim::structures::BuildKind>) {
        let pos = match self.player_pos {
            Some(p) => p,
            None => {
                self.status_msg = Some("No position".into());
                return;
            }
        };
        // Standing on your own weathering structure? Maintain it instead of
        // trying to build a new one on the same tile.
        if self.maintain_structure_here() {
            return;
        }
        let region_idx = pos.region_idx;
        let px = pos.px as u32;
        let py = pos.py as u32;
        let inv = match self.player_start.as_ref() {
            Some(ps) => ps.inventory.clone(),
            None => {
                self.status_msg = Some("No inventory".into());
                return;
            }
        };
        let terrain = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(region_idx))
            .and_then(|r| r.terrain.get(pos.px, pos.py))
            .unwrap_or(Terrain::Grass);
        // A footbridge is raised from the bank, onto the water beside you.
        let mut build_x = px;
        let mut build_y = py;
        let mut build_terrain = terrain;
        if wanted == Some(crate::sim::structures::BuildKind::Footbridge) {
            let water = self.sim.as_ref().and_then(|s| {
                let r = s.world.regions.get(region_idx)?;
                [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)]
                    .into_iter()
                    .find_map(|(dx, dy)| {
                        let nx = pos.px as i32 + dx;
                        let ny = pos.py as i32 + dy;
                        if nx < 0 || ny < 0 {
                            return None;
                        }
                        (r.terrain.get(nx as usize, ny as usize) == Some(Terrain::Water))
                            .then_some((nx as u32, ny as u32))
                    })
            });
            match water {
                Some((wx, wy)) => {
                    build_x = wx;
                    build_y = wy;
                    build_terrain = Terrain::Water;
                }
                None => {
                    self.status_msg =
                        Some("A footbridge wants water to cross — stand on the bank.".into());
                    return;
                }
            }
        }
        // Building on a settlement's ground needs the settlement's consent:
        // the council grants ground to those it does not distrust.
        if terrain == Terrain::Settlement {
            let bias = self
                .sim
                .as_ref()
                .and_then(|s| s.world.regions.get(region_idx))
                .and_then(|r| r.settlements.first())
                .and_then(|st| st.people.first())
                .map(|p| {
                    self.inter_people_bias
                        .effective_bias(crate::model::PeopleKind::from_name(&p.people))
                })
                .unwrap_or(0.0);
            let standing = self.reputation_in_current_settlement();
            if bias < -0.15 || standing < 0.45 {
                self.status_msg =
                    Some("The council will not grant you ground. Earn their regard first.".into());
                return;
            }
            if let Some(ref mut sim2) = self.sim {
                let t = sim2.world.tick;
                sim2.log(
                    t,
                    crate::sim::journal::Voice::Travel,
                    "The council walked the plot with me and drove the corner-stakes. I build among neighbors now.".into(),
                );
            }
        }
        let sim = match self.sim.as_mut() {
            Some(s) => s,
            None => return,
        };
        let affordable = |k: &crate::sim::structures::BuildKind| {
            !k.cost().is_empty()
                && k.cost()
                    .iter()
                    .all(|(item, count)| inv.get(*item) >= *count)
        };
        let kind = match wanted {
            Some(k) => {
                if !affordable(&k) {
                    self.status_msg = Some(format!(
                        "Not enough materials for a {} ({})",
                        k.label(),
                        k.cost()
                            .iter()
                            .map(|(i, c)| format!("{} {}", c, i.name()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                    return;
                }
                k
            }
            None => {
                // No stated intent: raise the best thing this land and these
                // materials allow (the old behavior, now land-aware).
                let mut best = None;
                for k in crate::sim::structures::BuildKind::all() {
                    if affordable(k) && k.stands_on(terrain) {
                        best = Some(*k);
                    }
                }
                match best {
                    Some(k) => k,
                    None => {
                        self.status_msg =
                            Some("Nothing to build here — need materials and fitting land".into());
                        return;
                    }
                }
            }
        };
        if !kind.stands_on(build_terrain) {
            self.status_msg = Some(format!(
                "A {} cannot stand on {}.",
                kind.label(),
                format!("{:?}", build_terrain).to_lowercase()
            ));
            return;
        }
        if kind.needs_tool() && (!inv.has(ItemType::Tool) || inv.is_broken(ItemType::Tool)) {
            self.status_msg = Some(format!(
                "Raising a {} needs a proper Tool in hand.",
                kind.label()
            ));
            return;
        }
        if let Some(ref mut ps) = self.player_start {
            for (item, count) in kind.cost() {
                ps.inventory.remove(item, count);
            }
            if kind.needs_tool() {
                ps.inventory.use_tool(ItemType::Tool);
            }
        }
        if kind.is_short_build() {
            let structure = crate::sim::structures::Structure {
                kind,
                region_idx,
                x: px,
                y: py,
                built_tick: sim.world.tick,
                last_maintenance_tick: sim.world.tick,
                name: None,
                is_npc_built: false,
                stash: Default::default(),
            };
            if let Some(region) = sim.world.regions.get_mut(region_idx) {
                region.structures.push(structure.clone());
            }
            sim.structures.push(structure);
            self.fire_hint(hints::HINT_FIRST_STRUCTURE);
            self.status_msg = Some(format!("Built {}!", kind.label()));
        } else {
            // A shrine is raised TO someone: the god the player carries
            // closest. Devotional practice, chosen at the first stone.
            let dedication = if kind == crate::sim::structures::BuildKind::Shrine {
                let ga = &self.god_affinity;
                let mut best = (GodName::Kukri, f64::MIN);
                for g in [
                    GodName::Oltzed,
                    GodName::Keuru,
                    GodName::Sampsa,
                    GodName::Masa,
                    GodName::Kukri,
                ] {
                    if ga.get(g) > best.1 {
                        best = (g, ga.get(g));
                    }
                }
                Some(best.0.label().to_string())
            } else {
                None
            };
            let site = crate::sim::structures::BuildSite {
                kind,
                region_idx,
                x: build_x,
                y: build_y,
                hours_done: 0,
                started_tick: sim.world.tick,
                dedication,
            };
            sim.build_sites.push(site);
            self.status_msg = Some(format!(
                "Started building {} ({}h)",
                kind.label(),
                kind.build_hours()
            ));
        }
    }

    pub(super) fn structure_at_player(&self) -> Option<crate::sim::structures::Structure> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        region
            .structures
            .iter()
            .find(|s| s.at_position(pos.region_idx, pos.px as u32, pos.py as u32))
            .cloned()
    }
}
