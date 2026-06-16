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
                    out.push((ri, si, format!("{} — {}", sett.display_name(), region.name)));
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
        // Walking into an enclave of the Five is not like entering a human town
        // (#454): the place announces itself in its own character. (A festival
        // greeting, if one is underway, takes precedence.)
        if !festival_now {
            if let Some(people) = self.current_settlement_people() {
                if let Some(welcome) = people.enclave_welcome() {
                    self.status_msg = Some(welcome.to_string());
                }
                // First time among this people of the Five: a lasting lore
                // reveal, told once and kept in the journal, and the aid they
                // ask of a stranger — a fetch repaid in the good only they make
                // (#454).
                if people.is_of_the_five() && !self.enclaves_seen.contains(&people) {
                    self.enclaves_seen.push(people);
                    let day = self.clock.day;
                    let quest = crate::sim::quest_gen::enclave_quest(people, day);
                    if let Some(ref mut sim) = self.sim {
                        if let Some(lore) = people.enclave_lore() {
                            let tick = sim.world.tick;
                            sim.log(tick, crate::sim::journal::Voice::Dream, lore.to_string());
                        }
                        if let Some(q) = quest {
                            let dup = sim.quests.iter().any(|e| e.description == q.description);
                            if !dup {
                                let tick = sim.world.tick;
                                sim.log(
                                    tick,
                                    crate::sim::journal::Voice::Encounter,
                                    format!(
                                        "The {} ask a thing of me. {}",
                                        people.label(),
                                        q.description
                                    ),
                                );
                                sim.quests.push(q);
                            }
                        }
                    }
                }
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

    /// Roads watched and partly closed while the province is at war: travel
    /// costs a little more here (#415). 1.0 in peace. Deterministic per season.
    fn road_watch_mult(&self) -> f64 {
        let Some(sim) = self.sim.as_ref() else {
            return 1.0;
        };
        let day = self.clock.day;
        let season_ord = (day / 30) % 4;
        let year = day / 120;
        if sim.world.polity.in_tension(self.seed, season_ord, year) {
            1.15
        } else {
            1.0
        }
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
        // A door is a way in, not a wall (#458): you step through it onto the
        // building's floor. Crossing the threshold from outside, the tavern
        // serves, the temple blesses, a home answers the knock — once, as you
        // enter, not again while you move about the rooms or step back out.
        if let Some(pos) = self.player_pos {
            let (nx, ny) = (pos.px as i32 + dx, pos.py as i32 + dy);
            if nx >= 0 && ny >= 0 {
                let region_idx = pos.region_idx;
                let (cur, target) = self
                    .sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(region_idx))
                    .map(|r| {
                        (
                            r.terrain.get(pos.px, pos.py),
                            r.terrain.get(nx as usize, ny as usize),
                        )
                    })
                    .unwrap_or((None, None));
                if target == Some(Terrain::Door) {
                    // Already inside (on floor or another doorway) means you
                    // are leaving or crossing through — just a step. Coming
                    // from the street, this is an entrance.
                    let entering = !matches!(
                        cur,
                        Some(Terrain::Floor) | Some(Terrain::Door) | Some(Terrain::Hearth)
                    );
                    if let Some(ref mut p) = self.player_pos {
                        p.px = nx as usize;
                        p.py = ny as usize;
                    }
                    self.reveal_around(region_idx, nx as usize, ny as usize);
                    if entering {
                        self.enter_door(region_idx, nx as usize, ny as usize);
                    }
                    self.screen = Screen::World { region_idx };
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
                let cost = (tile_hours as f64
                    * 0.5
                    * weather_mult
                    * companion_travel
                    * self.road_watch_mult()
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
                let cost = (tile_hours as f64
                    * 0.5
                    * weather_mult
                    * companion_travel
                    * self.road_watch_mult()
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

    /// What greets you as you cross a threshold (#458): a service building
    /// serves (the tavern rests you, the temple blesses), a plain home answers
    /// the knock. Called as you step in through the door; you end up inside, on
    /// the floor, free to walk the rooms.
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
            // A holding out in the country, beyond any town's footprint (#458).
            // Often the folk are out in the fields — but knock at the right hour
            // and a holder is in, and the country keeps the old hospitality.
            self.knock_at_a_holding(x, y);
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

    /// Knock at a rural holding's door (#458): the country keeps the old
    /// hospitality. Deterministic per (holding, day) — often the folk are out
    /// in the fields, but find a holder in and they share bread and water for
    /// the road and the word of the valley. No menu, no NPC roster: a holding
    /// is map life, met at its door.
    fn knock_at_a_holding(&mut self, x: usize, y: usize) {
        let day = self.clock.day as u64;
        let roll = crate::rng::unit_from_hash(crate::rng::mix_u64(
            self.seed
                ^ (x as u64).wrapping_shl(20)
                ^ (y as u64).wrapping_shl(40)
                ^ day.wrapping_mul(0x9E37_79B9),
        ));
        if roll < 0.45 {
            // Someone is home. A holder, by their work.
            let holders = [
                "An old farmer",
                "A broad-armed holder",
                "A farmwife with flour on her hands",
                "A boy minding the steading",
                "A herdsman come in from the field",
            ];
            let who = holders
                [(crate::rng::mix_u64(self.seed ^ x as u64 ^ y as u64) as usize) % holders.len()];
            self.advance_clock(1);
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(crate::model::ItemType::Food, 1);
                ps.inventory.add(crate::model::ItemType::Water, 1);
            }
            // The hearth-keeper marks a welcome given and taken.
            self.god_affinity
                .adjust(crate::model::GodName::Oltzed, 0.01);
            self.status_msg = Some(format!(
                "{who} waves you in. Bread and water for the road, and the news of the valley. (1h)"
            ));
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                sim.log(
                    tick,
                    crate::sim::journal::Voice::Travel,
                    "A holding took me in. Bread, water, and a while by a stranger's fire.".into(),
                );
            }
        } else {
            self.status_msg =
                Some("A farm-holding. You knock, but the folk are out in the fields.".into());
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
        // Streets and the real buildings (walls, floors, doors) all count as
        // being in town.
        if !matches!(
            region.terrain.get(pos.px, pos.py),
            Some(
                Terrain::Settlement
                    | Terrain::House
                    | Terrain::Wall
                    | Terrain::Floor
                    | Terrain::Door
                    | Terrain::Hearth
            )
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

    /// Sit a while in prayer (#457): the gods are withdrawn since the Fall, so
    /// this is devotion, not a summons — a quiet hour that deepens your bond
    /// with the god you most keep (or your people's patron), and leaves you a
    /// little steadied. The god never answers in the world; the practice
    /// changes only the one who keeps it, and the comfort is deniable.
    pub fn pray(&mut self) {
        // Who you keep: the god you've served most, else your people's patron,
        // else Kukri — the lonely god of the long road, fit for the godless.
        let god = self
            .god_affinity
            .strongest_ally()
            .or_else(|| crate::sim::god::patron_of(self.inter_people_bias.player_people.label()))
            .unwrap_or(crate::model::GodName::Kukri);
        // Devotion deepens, but the practice plateaus — the more you already
        // keep a god, the less a single hour adds. Faith is a long road.
        let have = self.god_affinity.get(god);
        let delta = 0.03 * (1.0 - have).max(0.0);
        self.god_affinity.adjust(god, delta);
        self.advance_clock(1);
        // The comfort of the practice — a small steadying, nothing more.
        self.vitals.energy = (self.vitals.energy + 0.03).min(1.0);
        let pid = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let line = crate::sim::god::prayer_flavor(god, &pid);
        self.status_msg = Some(format!("You sit a while in prayer. {line} (1h)"));
    }

    /// Set out on a pilgrimage from a holy site (#457): the longest road of
    /// devotion. Where prayer gives an hour and an offering a measure of food,
    /// a pilgrimage gives **days** — provisioned, walked, the road itself the
    /// prayer — and so deepens the bond more than any single act, answers the
    /// devout with the god's grace, and marks the life. Plateaus, like all
    /// devotion: no road makes a god of the walker.
    pub fn pilgrimage(&mut self) {
        use crate::model::{GodName, ItemType, SettlementService};
        let at_holy_site = self.current_settlement().is_some_and(|s| {
            s.services.contains(&SettlementService::Shrine)
                || s.services.contains(&SettlementService::Temple)
        });
        if !at_holy_site {
            self.status_msg =
                Some("A pilgrimage sets out from a holy site — a shrine or a temple.".into());
            return;
        }
        const PROVISIONS: u32 = 3;
        let provisioned = self
            .player_start
            .as_mut()
            .is_some_and(|ps| ps.inventory.remove(ItemType::Food, PROVISIONS));
        if !provisioned {
            self.status_msg = Some(format!(
                "The pilgrim road is long — provision it with {PROVISIONS} Food first."
            ));
            return;
        }
        let god = self
            .god_affinity
            .strongest_ally()
            .or_else(|| crate::sim::god::patron_of(self.inter_people_bias.player_people.label()))
            .unwrap_or(GodName::Kukri);
        // The road deepens devotion more than any single act.
        let have = self.god_affinity.get(god);
        let delta = 0.12 * (1.0 - have).max(0.0);
        self.god_affinity.adjust(god, delta);
        self.advance_clock(36); // ~a day and a half on the road
        let day = self.clock.day;

        // The devout pilgrim is answered with the god's grace at the road's end
        // (a lower bar than the festival — the walking earned it).
        let devout = self.god_affinity.get(god);
        let blessing = if devout >= 0.50 {
            if god == GodName::Masa {
                if let Some(ps) = self.player_start.as_mut() {
                    if !ps.person.illnesses.is_empty() {
                        ps.person.illnesses.remove(0);
                    }
                }
            }
            self.vitals.energy = 1.0;
            self.vitals.hunger = 1.0;
            self.vitals.thirst = 1.0;
            Some(crate::sim::god::grace_flavor(god))
        } else {
            None
        };

        let marked = self
            .milestones
            .record(crate::sim::milestones::MilestoneKind::MadePilgrimage, day);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Faith,
                "I walked the pilgrim road to the holy site and back. The god I keep is nearer for it.".to_string(),
            );
            if marked {
                let line = crate::sim::milestones::MilestoneKind::MadePilgrimage.journal_text();
                sim.log(tick, crate::sim::journal::Voice::Faith, line);
            }
        }
        self.status_msg = Some(match blessing {
            Some(b) => format!("You walk the pilgrim road and return. {b} (~1.5 days)"),
            None => "You walk the pilgrim road and return, the practice deepened. (~1.5 days)"
                .to_string(),
        });
    }

    /// Lay an offering at a shrine or temple (#457): devotion that costs
    /// something real. Where prayer gives only an hour, an offering gives up
    /// food from your own stores — and so deepens the bond with the god you
    /// keep further than prayer can, though it still plateaus: no gift ever
    /// makes a god of the giver.
    pub fn make_offering(&mut self) {
        use crate::model::{ItemType, SettlementService};
        let has_place = self.current_settlement().is_some_and(|s| {
            s.services.contains(&SettlementService::Shrine)
                || s.services.contains(&SettlementService::Temple)
        });
        if !has_place {
            self.status_msg =
                Some("There is no shrine or temple here to receive an offering.".into());
            return;
        }
        let gave = self
            .player_start
            .as_mut()
            .is_some_and(|ps| ps.inventory.remove(ItemType::Food, 1));
        if !gave {
            self.status_msg = Some("You have nothing to lay down — an offering needs Food.".into());
            return;
        }
        let god = self
            .god_affinity
            .strongest_ally()
            .or_else(|| crate::sim::god::patron_of(self.inter_people_bias.player_people.label()))
            .unwrap_or(crate::model::GodName::Kukri);
        let have = self.god_affinity.get(god);
        let delta = 0.06 * (1.0 - have).max(0.0);
        self.god_affinity.adjust(god, delta);
        self.advance_clock(1);
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Faith,
                "I laid an offering down at the shrine, and kept nothing back of it.".to_string(),
            );
        }
        self.status_msg =
            Some("You lay an offering at the shrine — bread given, not bartered. The keeping deepens. (1h)".into());
    }

    /// Keep the festival in earnest (#457): when a settlement's holy day is
    /// underway, take deliberate part — not the passing nod of arrival, but the
    /// long table, the drum-circle, the candle and the named dead. It deepens
    /// devotion to the festival's god, mends standing with its people faster
    /// than trade can, and steadies you, at the cost of the hours it takes.
    pub fn observe_festival(&mut self) {
        let day = self.clock.day;
        let underway = self
            .current_settlement()
            .is_some_and(|s| s.in_festival(day));
        if !underway {
            self.status_msg = Some("There is no festival to keep here today.".into());
            return;
        }
        let people = self
            .current_settlement_people()
            .unwrap_or(self.inter_people_bias.player_people);
        let festival = FestivalKind::for_people(people);
        let god = festival.patron_god();
        // Keeping the day in earnest deepens devotion more than merely passing
        // through, but it still plateaus — faith is a long road.
        let have = self.god_affinity.get(god);
        let delta = 0.06 * (1.0 - have).max(0.0);
        self.god_affinity.adjust(god, delta);
        // Showing up for their holy day mends fences faster than any trade.
        self.inter_people_bias.mod_toward(people, 0.05);
        // Communal food, fire, and welcome — a real steadying.
        self.vitals.energy = (self.vitals.energy + 0.15).min(1.0);
        self.vitals.hunger = (self.vitals.hunger + 0.25).min(1.0);
        self.advance_clock(3);
        let grace_line = festival.observance_grace();

        // Deep devotion is answered: at "Devoted" standing or beyond, the
        // patron's particular grace touches you — a concrete blessing, not a
        // stat nudge (#457). Masa's is mercy in healing; the rest steady the
        // body whole. Gated on the festival's rarity, so it cannot be farmed.
        let devout = self.god_affinity.get(god);
        let blessing = if crate::sim::god::devotion_rank(devout).is_some_and(|_| devout >= 0.60) {
            if god == crate::model::GodName::Masa {
                if let Some(ps) = self.player_start.as_mut() {
                    if !ps.person.illnesses.is_empty() {
                        ps.person.illnesses.remove(0);
                    }
                }
            }
            self.vitals.energy = 1.0;
            self.vitals.hunger = 1.0;
            self.vitals.thirst = 1.0;
            Some(crate::sim::god::grace_flavor(god))
        } else {
            None
        };

        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            let mut entry = format!("I kept the {} with them. {grace_line}", festival.label());
            if let Some(b) = blessing {
                entry.push(' ');
                entry.push_str(b);
            }
            sim.log(tick, crate::sim::journal::Voice::Faith, entry);
        }
        self.status_msg = Some(match blessing {
            Some(b) => format!("You keep the {}. {grace_line} {b} (3h)", festival.label()),
            None => format!("You keep the {}. {grace_line} (3h)", festival.label()),
        });
    }

    /// Set out on the long roads to one of the named cities of the continent
    /// (#456): the playable map is a province slice — the great cities never
    /// stand on it — but from a town on the roads you can make the days-long
    /// journey there and back, returning with the city's own goods and word of
    /// the wider world. You must provision the road (Food); the trip eats it.
    pub fn journey_to_city(&mut self) {
        if self.current_settlement().is_none() {
            self.status_msg =
                Some("The great cities lie down the long roads — set out from a town.".into());
            return;
        }
        const PROVISIONS: u32 = 6;
        let have_food = self
            .player_start
            .as_ref()
            .map(|ps| ps.inventory.get(crate::model::ItemType::Food))
            .unwrap_or(0);
        if have_food < PROVISIONS {
            self.status_msg = Some(format!(
                "The road to the great cities is long — provision it with {PROVISIONS} Food first."
            ));
            return;
        }
        // Which city, settled by the seed and the day — a different road each time.
        let day = self.clock.day as u64;
        let idx = (crate::rng::mix_u64(self.seed ^ day.wrapping_mul(0x9E37_79B9)) as usize)
            % crate::sim::CANON_CITIES.len();
        let (city, blurb) = crate::sim::CANON_CITIES[idx];
        // You set out rested and provisioned; the road takes its days, and the
        // provisions are eaten on the way (the clock auto-feeds from the pack).
        self.vitals.energy = self.vitals.energy.max(0.9);
        self.advance_clock(72); // ~three days there and back
                                // The city's own goods come home with you, the luck of the road on top.
        let lucky = crate::rng::unit_from_hash(crate::rng::mix_u64(self.seed ^ day ^ 0x10AD_5EED))
            < self.fortune.tilt_good(0.30);
        let bonus = if lucky { 1 } else { 0 };
        // Long-haul trade (#456): the bulk commodities you hauled out fetch a
        // city premium — the province's cheap surplus is the city's dear
        // import. This is the arbitrage loop: carry goods to the road's end,
        // come home with coin. Gear, food, and water are kept (not sold).
        let premium = if lucky { 1.8 } else { 1.6 };
        let mut sold_units = 0u32;
        let mut earned = 0u32;
        if let Some(ref mut ps) = self.player_start {
            // First sell the haul you carried out (before the city's own goods
            // come home, so those aren't immediately resold).
            for item in HAULABLE {
                let n = ps.inventory.get(item);
                if n > 0 && ps.inventory.remove(item, n) {
                    sold_units += n;
                    earned += ((item.base_price() as f64 * premium).round() as u32) * n;
                }
            }
            if earned > 0 {
                ps.inventory.add(crate::model::ItemType::Coin, earned);
            }
            // Then the city's own specialty goods travel home with you.
            for (item, qty) in city_goods(idx) {
                ps.inventory.add(item, qty + bonus);
            }
        }
        self.god_affinity.adjust(crate::model::GodName::Masa, 0.03);
        // The roads are not safe after the Fall (#449): now and then the long
        // way takes its toll — the lawless, a washed-out ford, a stretch of
        // hungry country. Fortune tilts it; the more silver you carry home, the
        // more there is to lose. (Rolled after the trade, so it bites the purse.)
        let robbed = crate::rng::unit_from_hash(crate::rng::mix_u64(self.seed ^ day ^ 0x80AD_DEAD))
            < self.fortune.tilt_bad(0.18);
        let mut toll = 0u32;
        if robbed {
            if let Some(ref mut ps) = self.player_start {
                let coin = ps.inventory.get(crate::model::ItemType::Coin);
                toll = coin / 4; // the road's tax — a quarter of your silver
                if toll > 0 {
                    ps.inventory.remove(crate::model::ItemType::Coin, toll);
                }
            }
            self.vitals.energy = (self.vitals.energy - 0.2).max(0.0);
        }
        // You come home with word of the wider world, not just goods.
        let mut news_rng =
            crate::rng::SeedRng::new(self.seed.wrapping_add(day).wrapping_add(0x4E5_03AD))
                .fork_for("journey-news");
        let news = crate::sim::journal::rumor_text(&mut news_rng);
        let trade = if earned > 0 {
            format!(" Your haul of {sold_units} sold at {city} prices for {earned} coin.")
        } else {
            String::new()
        };
        let road = if robbed && toll > 0 {
            format!(" The road took its toll — the lawless lightened your purse by {toll} coin.")
        } else if robbed {
            " The road was hard — you came home wearied and watchful.".to_string()
        } else {
            String::new()
        };
        self.status_msg = Some(format!(
            "You walked the long roads to {city} and back — {blurb}.{trade}{road} Word travels: {news} (~3 days)"
        ));
        if let Some(ref mut sim) = self.sim {
            let tick = sim.world.tick;
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                format!("I walked the long roads to {city}. {blurb}."),
            );
            // The city itself, at canon scale (#456): a place far larger than
            // any province town — the crowd, the quarters, the weight of it.
            sim.log(
                tick,
                crate::sim::journal::Voice::Travel,
                city_arrival(idx).to_string(),
            );
            sim.log(tick, crate::sim::journal::Voice::Rumor, news);
        }
        // The first such journey marks the life: the province is not the world
        // (#456). Recorded once, it stands in the legacy at the end.
        if self.milestones.record(
            crate::sim::milestones::MilestoneKind::WalkedToGreatCity,
            day as u32,
        ) {
            if let Some(ref mut sim) = self.sim {
                let tick = sim.world.tick;
                let line = crate::sim::milestones::MilestoneKind::WalkedToGreatCity.journal_text();
                sim.log(tick, crate::sim::journal::Voice::Travel, line);
            }
        }
    }
}

/// The felt arrival at a great city of the continent (#456): canon scale and
/// character, so the journey lands somewhere far larger than the province —
/// the crowd, the districts, the weight of a place that holds fifteen thousand
/// and more (population_scale_and_settlement_hierarchy.md). One per CANON_CITY,
/// by index.
fn city_arrival(idx: usize) -> &'static str {
    match idx {
        0 => "Sampa Crossing sprawls where the Basin roads meet — fifteen thousand souls and more, grain-barges thick on the water, a market quarter that does not empty from dawn to dark. After a province of a few hundred, the crowd alone is a kind of vertigo.",
        1 => "Vessenath rises grey above its lake, a city of twenty thousand under a haze of forge-smoke — steel-halls and fur-markets, the cold water crowded with fishing craft. You have never seen so many roofs stand in one place.",
        2 => "Halkess holds the grain-price of the south in its fists — a walled city of merchants and granaries whose scales are the law for a hundred leagues. Coin moves through its counting-houses in rivers, and everyone walks a little faster.",
        3 => "Velkarath broods over the harbor of the old capital — half its grandeur fallen, half still standing, salvage-crews picking the bones of the world before the Fall. It is a city haunted by how much larger it once was.",
        _ => "Keuramark stands at the treeline of the north, a frontier city of log halls and amber-traders — the last great market before the cold country, loud with timber-crews and the long-winter trade.",
    }
}

/// Bulk commodities worth hauling the long road to a city market (#456):
/// raw and worked trade goods, never the traveller's own gear, food, or water.
const HAULABLE: [crate::model::ItemType; 9] = {
    use crate::model::ItemType as I;
    [
        I::Wood,
        I::Stone,
        I::Cloth,
        I::Iron,
        I::Glass,
        I::Hide,
        I::Leather,
        I::Herb,
        I::Cordage,
    ]
};

/// What each named city sends home with a traveller — its own canon specialty
/// (#456), matched to the goods of the game.
fn city_goods(idx: usize) -> Vec<(crate::model::ItemType, u32)> {
    use crate::model::ItemType as I;
    match idx {
        0 => vec![(I::Food, 4)], // Sampa Crossing — Basin grain
        1 => vec![(I::Hide, 2), (I::Iron, 1), (I::Food, 2)], // Vessenath — furs, steel, fish
        2 => vec![(I::Food, 3)], // Halkess — grain
        3 => vec![(I::Tool, 1), (I::Glass, 1)], // Velkarath — salvage, harbor-goods
        _ => vec![(I::Wood, 3)], // Keuramark — timber, amber
    }
}
