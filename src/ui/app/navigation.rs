use crate::model::{FestivalKind, PlayerPos, Settlement, Terrain};

use super::*;

enum MoveResult {
    EdgeTransition {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Step {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Blocked {
        msg: String,
    },
}

impl App {
    pub fn settlement_list(&self) -> Vec<(usize, usize, String)> {
        let mut out = Vec::new();
        if let Some(ref sim) = self.sim {
            for (ri, region) in sim.world.regions.iter().enumerate() {
                for (si, sett) in region.settlements.iter().enumerate() {
                    out.push((ri, si, format!("{} — {}", sett.name, region.name)));
                }
            }
        }
        out
    }

    pub fn enter_settlement(&mut self, region_idx: usize, settlement_idx: usize) {
        self.milestones.settlements_visited += 1;

        // Roll leadership events for the settlement
        if let Some(ref mut sim) = self.sim {
            if let Some(pos) = self.player_pos {
                if let Some(region) = sim.world.regions.get_mut(pos.region_idx) {
                    if let Some(settlement) = region.settlements.get_mut(settlement_idx) {
                        let seed = self
                            .seed
                            .wrapping_add(self.clock.day as u64)
                            .wrapping_add(settlement_idx as u64);
                        if let Some(event) = settlement.politics.roll_leadership_event(seed) {
                            self.status_msg = Some(event.flavor().to_string());
                        }
                    }
                }
            }
        }

        if let Some(npc_people) = self.current_settlement_people() {
            let bias = self.inter_people_bias.effective_bias(npc_people)
                + self.clock.season().bias_modifier();
            if bias < -0.20 {
                self.status_msg = Some(format!(
                    "Guards block your path. 'No {} allowed beyond this point.' You turn back.",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
            if bias < -0.10 {
                self.status_msg = Some(
                    "A guard eyes you suspiciously but lets you pass. 'Keep your head down.'"
                        .into(),
                );
            }
        }

        // Create obligations for NPCs with dependents
        if let Some(ref mut sim) = self.sim {
            if let Some(pos) = self.player_pos {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if let Some(settlement) = region.settlements.get(settlement_idx) {
                        for person in &settlement.people {
                            if person.has_spouse || person.children_count > 0 {
                                let obl = crate::sim::needs_dependent::Obligation {
                                    caregiver_id: person.id.clone(),
                                    dependent_id: person.id.clone(),
                                    need: crate::model::Need::Care,
                                    strength: if person.children_count > 0 {
                                        0.15
                                    } else {
                                        0.10
                                    },
                                };
                                sim.obligations.push(obl);
                            }
                        }
                    }
                }
            }
        }

        let season = self.clock.season();
        let festival_now = self
            .sim
            .as_ref()
            .and_then(|sim| sim.world.regions.get(region_idx))
            .and_then(|r| r.settlements.get(settlement_idx))
            .is_some_and(|s| s.in_festival(self.clock.day));
        if festival_now {
            {
                let people = self
                    .current_settlement_people()
                    .unwrap_or(self.inter_people_bias.player_people);
                let festival = FestivalKind::for_people(people);
                self.god_affinity.adjust(festival.patron_god(), 0.03);
                // Festivals are when fences mend: showing up softens old grudges.
                self.inter_people_bias.mod_toward(people, 0.03);
                let bias = self.current_settlement_people().map_or(0.0, |p| {
                    self.inter_people_bias.effective_bias(p) + season.bias_modifier()
                });
                if bias > -0.10 {
                    self.vitals.hunger = (self.vitals.hunger + 0.2).min(1.0);
                    self.vitals.energy = (self.vitals.energy + 0.1).min(1.0);
                }
                self.status_msg = Some(format!(
                    "A {} is underway! {}",
                    festival.label(),
                    festival.flavor()
                ));
            }
        }
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_settlement(&mut self) {
        self.screen = self.world_screen();
    }

    pub fn current_settlement(&self) -> Option<&Settlement> {
        match &self.screen {
            Screen::Location {
                region_idx,
                settlement_idx,
                ..
            }
            | Screen::Npc {
                region_idx,
                settlement_idx,
                ..
            }
            | Screen::Talk {
                region_idx,
                settlement_idx,
                ..
            } => self.sim.as_ref().and_then(|sim| {
                sim.world
                    .regions
                    .get(*region_idx)
                    .and_then(|r| r.settlements.get(*settlement_idx))
            }),
            // Off the menu screens, the town is wherever you stand: streets
            // and roofs resolve to their settlement (walk-in layer, #372).
            _ => {
                let (ri, si) = self.player_on_settlement()?;
                self.sim
                    .as_ref()
                    .and_then(|sim| sim.world.regions.get(ri))
                    .and_then(|r| r.settlements.get(si))
            }
        }
    }

    pub fn reveal_around(&mut self, region_idx: usize, px: usize, py: usize) {
        let mut radius = crate::model::ExploredMap::reveal_radius_for_elder(self.elder);
        // A scouting animal (falcon, crow) rides ahead and widens what the
        // player sees. scouting_bonus existed on Animal but was never applied.
        let scout = self
            .player_start
            .as_ref()
            .map(|ps| {
                ps.companions
                    .iter()
                    .map(|c| c.animal.scouting_bonus())
                    .fold(0.0, f64::max)
            })
            .unwrap_or(0.0);
        radius += (scout * 5.0).round() as usize;
        if region_idx < self.explored.len() {
            self.explored[region_idx].reveal(px, py, radius);
        }
    }

    pub fn enter_map(&mut self, region_idx: usize) {
        let (px, py) = if let Some(ref pos) = self.player_pos {
            if pos.region_idx == region_idx {
                (pos.px, pos.py)
            } else {
                self.find_settlement_pos(region_idx)
            }
        } else {
            self.find_settlement_pos(region_idx)
        };
        self.player_pos = Some(PlayerPos { region_idx, px, py });
        self.reveal_around(region_idx, px, py);
        self.screen = Screen::World { region_idx };
    }

    fn find_settlement_pos(&self, region_idx: usize) -> (usize, usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                let w = region.terrain.width.max(1);
                // Prefer a settlement tile.
                if let Some(pos) = region
                    .terrain
                    .tiles
                    .iter()
                    .position(|&t| t == Terrain::Settlement)
                {
                    return (pos % w, pos / w);
                }
                // Otherwise any passable tile — never strand the player in water
                // or on a mountain (e.g. a settlement-less upland region).
                if let Some(pos) = region.terrain.tiles.iter().position(|&t| t.passable()) {
                    return (pos % w, pos / w);
                }
            }
        }
        (20, 10)
    }

    pub fn exit_map(&mut self) {
        self.screen = self.world_screen();
    }

    pub(crate) fn world_screen(&self) -> Screen {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        Screen::World { region_idx }
    }

    pub fn enter_overmap(&mut self) {
        let region_idx = match &self.screen {
            Screen::World { region_idx } => *region_idx,
            _ => 0,
        };
        self.screen = Screen::Overmap { region_idx };
    }

    pub fn exit_overmap(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let weather = self.sim.as_ref().and_then(|sim| {
            let pos = self.player_pos?;
            let region = sim.world.regions.get(pos.region_idx)?;
            Some(region.weather)
        });
        if let Some(w) = weather {
            if let Some(ref mut rng) = self.player_rng {
                if crate::sim::weather::forced_shelter(w, rng.gen_f64()) {
                    let flavor = crate::sim::weather::weather_travel_flavor(w, true);
                    if let Some(ref mut sim) = self.sim {
                        sim.log(
                            sim.world.tick,
                            crate::sim::journal::Voice::Travel,
                            flavor.to_string(),
                        );
                    }
                    self.status_msg = Some(flavor.to_string());
                    return;
                }
            }
        }
        // The town gate lives on the map now: stepping from open ground onto
        // a settlement's footprint passes its guards (#372 PR 5). Hostile
        // peoples are turned back at the edge; a first step in counts the
        // visit and rolls the town's politics, as entering always did.
        if let Some(pos) = self.player_pos {
            let (nx, ny) = (pos.px as i32 + dx, pos.py as i32 + dy);
            if nx >= 0 && ny >= 0 {
                let (ux, uy) = (nx as usize, ny as usize);
                let entering = self
                    .sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(pos.region_idx))
                    .and_then(|r| {
                        let ti = r.settlements.iter().position(|s| s.contains_tile(ux, uy))?;
                        let already_inside = r
                            .settlements
                            .get(ti)
                            .map(|s| s.contains_tile(pos.px, pos.py))
                            .unwrap_or(false);
                        if already_inside {
                            None
                        } else {
                            Some(ti)
                        }
                    });
                if let Some(si) = entering {
                    let npc_people = self
                        .sim
                        .as_ref()
                        .and_then(|s| s.world.regions.get(pos.region_idx))
                        .and_then(|r| r.settlements.get(si))
                        .and_then(|s| s.people.first())
                        .map(|p| crate::model::PeopleKind::from_name(&p.people));
                    if let Some(np) = npc_people {
                        let bias = self.inter_people_bias.effective_bias(np)
                            + self.clock.season().bias_modifier();
                        if bias < -0.20 {
                            self.status_msg = Some(format!(
                                "Guards block your path. 'No {} allowed beyond this                                  point.' You turn back.",
                                self.inter_people_bias.player_people.label()
                            ));
                            return;
                        }
                    }
                    self.milestones.settlements_visited += 1;
                    let event = self
                        .sim
                        .as_mut()
                        .and_then(|s| s.world.regions.get_mut(pos.region_idx))
                        .and_then(|r| r.settlements.get_mut(si))
                        .and_then(|s| {
                            let seed = self
                                .seed
                                .wrapping_add(self.clock.day as u64)
                                .wrapping_add(si as u64);
                            s.politics.roll_leadership_event(seed)
                        });
                    if let Some(ev) = event {
                        self.status_msg = Some(ev.flavor().to_string());
                    }
                }
            }
        }
        // A person is not a tile either: stepping into someone in the street
        // greets them (#372 PR 4) — what you see is who you can meet.
        if let Some(pos) = self.player_pos {
            let (nx, ny) = (pos.px as i32 + dx, pos.py as i32 + dy);
            if nx >= 0 && ny >= 0 {
                let (ux, uy) = (nx as usize, ny as usize);
                let hit = self
                    .sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(pos.region_idx))
                    .and_then(|r| {
                        let si = r.settlements.iter().position(|s| s.contains_tile(ux, uy))?;
                        let s = &r.settlements[si];
                        crate::gen::town::npc_street_positions(s, self.clock.day, self.clock.hour)
                            .into_iter()
                            .find(|&(_, x, y)| x == ux && y == uy)
                            .map(|(pi, _, _)| (si, pi))
                    });
                if let Some((si, pi)) = hit {
                    self.enter_talk(pos.region_idx, si, pi);
                    return;
                }
            }
        }
        // A door is not a wall: stepping into a house enters it (#372 PR 3) —
        // the tavern serves, the temple blesses, a home answers the knock.
        if let Some(pos) = self.player_pos {
            let (nx, ny) = (pos.px as i32 + dx, pos.py as i32 + dy);
            if nx >= 0 && ny >= 0 {
                let target = self
                    .sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(pos.region_idx))
                    .and_then(|r| r.terrain.get(nx as usize, ny as usize));
                if target == Some(Terrain::House) {
                    self.enter_door(pos.region_idx, nx as usize, ny as usize);
                    return;
                }
            }
        }
        let weather_mult = weather
            .map(crate::sim::weather::travel_hours_multiplier)
            .unwrap_or(1.0);
        let companion_travel = self.companion_travel_mult();
        let result = self.compute_move(dx, dy);
        match result {
            Some(MoveResult::EdgeTransition { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.region_idx = region_idx;
                    p.px = px;
                    p.py = py;
                }
                let terrain = self
                    .sim
                    .as_ref()
                    .and_then(|sim| sim.world.regions.get(region_idx))
                    .and_then(|r| r.terrain.get(px, py))
                    .unwrap_or(Terrain::Grass);
                let bias_mod = self.current_settlement_people().map_or(0, |npc_people| {
                    let bias = self.inter_people_bias.effective_bias(npc_people)
                        + self.clock.season().bias_modifier();
                    if bias < -0.15 {
                        1
                    } else if bias > 0.10 {
                        -1
                    } else {
                        0
                    }
                });
                let tile_hours = if self.path_structure_at(region_idx, px, py).is_some() {
                    1 // a laid trail or sound planks carry you clean across
                } else {
                    terrain.travel_hours()
                };
                // Finer tiles walk in half-hours: two open tiles to the
                // hour. The fraction is owed to the clock and paid when it
                // accumulates to whole hours.
                let cost = (tile_hours as f64 * 0.5 * weather_mult * companion_travel
                    + bias_mod as f64 * 0.5)
                    .max(0.25);
                self.travel_debt += cost;
                let whole = self.travel_debt.floor() as u32;
                if whole > 0 {
                    self.travel_debt -= whole as f64;
                    self.advance_clock(whole);
                }
                self.log_travel(terrain);
                self.reveal_around(region_idx, px, py);
                self.check_encounter(terrain);
                self.check_memorial();
                self.check_discovery(region_idx, px, py);
                self.check_quests_on_travel(region_idx);
                if self.encounter.is_none() {
                    self.screen = Screen::World { region_idx };
                }
            }
            Some(MoveResult::Step { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.px = px;
                    p.py = py;
                }
                let terrain = self
                    .sim
                    .as_ref()
                    .and_then(|sim| sim.world.regions.get(region_idx))
                    .and_then(|r| r.terrain.get(px, py))
                    .unwrap_or(Terrain::Grass);
                let bias_mod = self.current_settlement_people().map_or(0, |npc_people| {
                    let bias = self.inter_people_bias.effective_bias(npc_people)
                        + self.clock.season().bias_modifier();
                    if bias < -0.15 {
                        1
                    } else if bias > 0.10 {
                        -1
                    } else {
                        0
                    }
                });
                let tile_hours = if self.path_structure_at(region_idx, px, py).is_some() {
                    1 // a laid trail or sound planks carry you clean across
                } else {
                    terrain.travel_hours()
                };
                // Finer tiles walk in half-hours: two open tiles to the
                // hour. The fraction is owed to the clock and paid when it
                // accumulates to whole hours.
                let cost = (tile_hours as f64 * 0.5 * weather_mult * companion_travel
                    + bias_mod as f64 * 0.5)
                    .max(0.25);
                self.travel_debt += cost;
                let whole = self.travel_debt.floor() as u32;
                if whole > 0 {
                    self.travel_debt -= whole as f64;
                    self.advance_clock(whole);
                }
                self.log_travel(terrain);
                self.reveal_around(region_idx, px, py);
                // A waymarker within two tiles keeps the ground around it
                // known — the cairn shows the path onward.
                let marks: Vec<(usize, usize)> = self
                    .sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(region_idx))
                    .map(|r| {
                        r.structures
                            .iter()
                            .filter(|st| {
                                st.kind == crate::sim::structures::BuildKind::Waymarker
                                    && st.x.abs_diff(px as u32) <= 2
                                    && st.y.abs_diff(py as u32) <= 2
                            })
                            .map(|st| (st.x as usize, st.y as usize))
                            .collect()
                    })
                    .unwrap_or_default();
                for (mx, my) in marks {
                    self.reveal_around(region_idx, mx, my);
                }
                // A signal fire is seen from anywhere in the region — the
                // ground around it stays known while it stands.
                let fires: Vec<(usize, usize)> = self
                    .sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(region_idx))
                    .map(|r| {
                        r.structures
                            .iter()
                            .filter(|st| st.kind == crate::sim::structures::BuildKind::Beacon)
                            .map(|st| (st.x as usize, st.y as usize))
                            .collect()
                    })
                    .unwrap_or_default();
                for (fx, fy) in fires {
                    if region_idx < self.explored.len() {
                        self.explored[region_idx].reveal(fx, fy, 4);
                    }
                }
                self.check_encounter(terrain);
                self.check_memorial();
                self.check_discovery(region_idx, px, py);
                if self.encounter.is_none() {
                    self.screen = Screen::World { region_idx };
                }
            }
            Some(MoveResult::Blocked { msg }) => {
                self.status_msg = Some(msg);
            }
            None => {}
        }
    }

    /// Step through a house door (#372 PR 3): a service building serves, a
    /// plain home answers the knock. The walker stays on the doorstep — the
    /// roof tile remains solid ground you don't occupy.
    fn enter_door(&mut self, region_idx: usize, x: usize, y: usize) {
        let owner = self
            .sim
            .as_ref()
            .and_then(|s| s.world.regions.get(region_idx))
            .and_then(|r| {
                r.settlements
                    .iter()
                    .position(|s| s.contains_tile(x, y))
                    .map(|si| (si, r.settlements[si].clone()))
            });
        let Some((_si, settlement)) = owner else {
            self.status_msg = Some("The door is barred and no one answers.".into());
            return;
        };
        match crate::gen::town::service_at(&settlement, x, y) {
            Some(svc) => self.use_service(svc),
            None => {
                // An ordinary home: a knock, a face, a moment by the fire.
                let host = if settlement.people.is_empty() {
                    None
                } else {
                    let idx = (x.wrapping_mul(31) ^ y.wrapping_mul(17)) % settlement.people.len();
                    settlement.people.get(idx).map(|p| p.name.clone())
                };
                self.advance_clock(1);
                self.status_msg = Some(match host {
                    Some(name) => format!(
                        "You knock. {} waves you in to warm up by the hearth (1h).",
                        name
                    ),
                    None => "You knock. No one answers; the house stands empty.".into(),
                });
            }
        }
    }

    fn compute_move(&self, dx: i32, dy: i32) -> Option<MoveResult> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let map_w = region.terrain.width;
        let map_h = region.terrain.height;
        let nx = pos.px as i32 + dx;
        let ny = pos.py as i32 + dy;

        if nx < 0 {
            region
                .neighbors
                .west
                .map(|west| MoveResult::EdgeTransition {
                    region_idx: west,
                    px: map_w - 1,
                    py: pos.py,
                })
        } else if nx >= map_w as i32 {
            region
                .neighbors
                .east
                .map(|east| MoveResult::EdgeTransition {
                    region_idx: east,
                    px: 0,
                    py: pos.py,
                })
        } else if ny < 0 {
            region
                .neighbors
                .north
                .map(|north| MoveResult::EdgeTransition {
                    region_idx: north,
                    px: pos.px,
                    py: map_h - 1,
                })
        } else if ny >= map_h as i32 {
            region
                .neighbors
                .south
                .map(|south| MoveResult::EdgeTransition {
                    region_idx: south,
                    px: pos.px,
                    py: 0,
                })
        } else {
            let ux = nx as usize;
            let uy = ny as usize;
            let terrain = region.terrain.get(ux, uy);
            if let Some(t) = terrain {
                // A standing footbridge makes its water crossable — while it
                // stands. Decay takes the planks, and the water comes back.
                let bridged = t == Terrain::Water
                    && self.path_structure_at(pos.region_idx, ux, uy)
                        == Some(crate::sim::structures::BuildKind::Footbridge);
                if t.passable() || bridged {
                    Some(MoveResult::Step {
                        region_idx: pos.region_idx,
                        px: ux,
                        py: uy,
                    })
                } else {
                    Some(MoveResult::Blocked {
                        msg: format!("Blocked: {:?}", t),
                    })
                }
            } else {
                None
            }
        }
    }

    pub fn player_on_settlement(&self) -> Option<(usize, usize)> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        // Streets and houses both count as being in town.
        if !matches!(
            region.terrain.get(pos.px, pos.py),
            Some(Terrain::Settlement) | Some(Terrain::House)
        ) {
            return None;
        }
        // The settlement whose painted footprint this tile belongs to —
        // settlements are squares of ground now, not points, so a town's
        // every street resolves to the same town.
        if let Some(si) = region
            .settlements
            .iter()
            .position(|s| s.contains_tile(pos.px, pos.py))
        {
            return Some((pos.region_idx, si));
        }
        if region.settlements.is_empty() {
            return None;
        }
        // No footprint claims the tile (a pre-anchor save, or stray paint):
        // fall back to the nearest anchor.
        let nearest = region
            .settlements
            .iter()
            .enumerate()
            .min_by_key(|(_, s)| {
                let dx = (s.map_x as i64 - pos.px as i64).abs();
                let dy = (s.map_y as i64 - pos.py as i64).abs();
                dx + dy
            })
            .map(|(i, _)| i)
            .unwrap_or(0);
        Some((pos.region_idx, nearest))
    }
}
