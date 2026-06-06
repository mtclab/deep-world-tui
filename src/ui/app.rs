use crate::charts::Charts;
use crate::gen::player::generate_player_start;
use crate::model::{
    craft_recipes, GameClock, Inventory, ItemType, Need, PlayerPos, PlayerStart, PlayerVitals,
    Settlement, Terrain,
};
use crate::rng::SeedRng;
use crate::save::{self, SaveData};
use crate::sim::SimState;

use super::event::AppEvent;

pub enum Screen {
    CharacterCreation,
    World,
    Map {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Overmap {
        region_idx: usize,
    },
    Inventory,
    Craft {
        scroll: u16,
    },
    WorldAlerts {
        scroll: u16,
    },
    Location {
        region_idx: usize,
        settlement_idx: usize,
        scroll: u16,
    },
    Npc {
        region_idx: usize,
        settlement_idx: usize,
        person_idx: usize,
        scroll: u16,
    },
    Journal {
        scroll: u16,
    },
    Talk {
        region_idx: usize,
        settlement_idx: usize,
        person_idx: usize,
        scroll: u16,
    },
}

pub struct App {
    pub sim: Option<SimState>,
    pub player_start: Option<PlayerStart>,
    pub running: bool,
    pub tick_interval: u64,
    pub screen: Screen,
    pub status_msg: Option<String>,
    pub player_pos: Option<PlayerPos>,
    pub clock: GameClock,
    pub vitals: PlayerVitals,
    seed: u64,
    charts: Charts,
    player_rng: Option<SeedRng>,
}

impl App {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let player_rng = SeedRng::new(seed);
        App {
            sim: None,
            player_start: None,
            running: true,
            tick_interval: 100,
            screen: Screen::CharacterCreation,
            status_msg: None,
            player_pos: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            seed,
            charts,
            player_rng: Some(player_rng),
        }
    }

    pub fn generate_player(&mut self) {
        if let Some(ref mut rng) = self.player_rng {
            let ps = generate_player_start(rng, &self.charts);
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
            self.player_start = Some(ps);
            let sim = SimState::new(self.seed, self.charts.clone());
            self.sim = Some(sim);
            self.screen = Screen::World;
        }
    }

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
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_settlement(&mut self) {
        self.screen = Screen::World;
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
            _ => None,
        }
    }

    pub fn enter_npc(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_npc(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn enter_journal(&mut self) {
        self.screen = Screen::Journal { scroll: 0 };
    }

    pub fn exit_journal(&mut self) {
        self.screen = Screen::World;
    }

    pub fn save_game(&mut self) {
        if let Some(ref sim) = self.sim {
            let data = SaveData {
                sim: sim.clone(),
                player_start: self.player_start.clone(),
            };
            match save::save_game(&data, "save.ron") {
                Ok(()) => self.status_msg = Some("Saved to save.ron".into()),
                Err(e) => self.status_msg = Some(format!("Save failed: {}", e)),
            }
        }
    }

    pub fn load_game(&mut self) {
        match save::load_game("save.ron") {
            Ok(data) => {
                self.sim = Some(data.sim);
                self.player_start = data.player_start;
                self.screen = Screen::World;
                self.status_msg = Some("Loaded from save.ron".into());
            }
            Err(e) => self.status_msg = Some(format!("Load failed: {}", e)),
        }
    }

    pub fn enter_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Talk {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn give_food(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Food, 1) {
                self.status_msg = Some("No food to give".into());
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Food, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    sim.relationships.update_relationship(
                        pid,
                        &person_id,
                        "gave food",
                        sim.world.tick,
                        0.05,
                        0.03,
                    );
                    sim.reputation.adjust_local(pid, sid, 0.02);
                }
                self.status_msg = Some(format!("Gave food to {}", person.name));
            }
        }
    }

    pub fn give_coin(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Coin, 1) {
                self.status_msg = Some("No coin to give".into());
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Money, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    sim.relationships.update_relationship(
                        pid,
                        &person_id,
                        "gave coin",
                        sim.world.tick,
                        0.03,
                        0.02,
                    );
                    sim.reputation.adjust_local(pid, sid, 0.01);
                }
                self.status_msg = Some(format!("Gave coin to {}", person.name));
            }
        }
    }

    pub fn enter_alerts(&mut self) {
        self.screen = Screen::WorldAlerts { scroll: 0 };
    }

    pub fn exit_alerts(&mut self) {
        self.screen = Screen::World;
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
        self.screen = Screen::Map { region_idx, px, py };
    }

    fn find_settlement_pos(&self, region_idx: usize) -> (usize, usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if let Some(pos) = region
                    .terrain
                    .tiles
                    .iter()
                    .position(|&t| t == Terrain::Settlement)
                {
                    return (pos % region.terrain.width, pos / region.terrain.width);
                }
            }
        }
        (20, 10)
    }

    pub fn exit_map(&mut self) {
        self.screen = Screen::World;
    }

    pub fn enter_overmap(&mut self) {
        let region_idx = match &self.screen {
            Screen::Map { region_idx, .. } => *region_idx,
            _ => 0,
        };
        self.screen = Screen::Overmap { region_idx };
    }

    pub fn exit_overmap(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn gather(&mut self) {
        if self.clock.time_of_day().is_dark() {
            self.status_msg = Some("Too dark to gather".into());
            return;
        }
        let terrain_item = self.player_pos.and_then(|pos| {
            self.sim.as_ref().and_then(|sim| {
                sim.world
                    .regions
                    .get(pos.region_idx)
                    .and_then(|r| r.terrain.get(pos.px, pos.py))
                    .and_then(ItemType::gather_from)
            })
        });
        if let Some(item) = terrain_item {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, 1);
            }
            self.advance_clock_hour();
            self.status_msg = Some(format!("Gathered 1 {} (1h)", item.name()));
        } else {
            self.status_msg = Some("Nothing to gather here".into());
        }
    }

    pub fn enter_inventory(&mut self) {
        self.screen = Screen::Inventory;
    }

    pub fn exit_inventory(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn enter_craft(&mut self) {
        self.screen = Screen::Craft { scroll: 0 };
    }

    pub fn exit_craft(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn craft_recipe(&mut self, recipe_idx: usize) {
        let recipes = craft_recipes();
        if let Some(recipe) = recipes.get(recipe_idx) {
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
                    inv.add(recipe.output, recipe.output_count);
                    self.advance_clock(2);
                    self.status_msg = Some(format!(
                        "Crafted {} (x{}) (2h)",
                        recipe.name, recipe.output_count
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

    pub fn advance_clock(&mut self, hours: u32) {
        self.clock.advance(hours);
        if let Some(ref mut ps) = self.player_start {
            self.vitals.tick(hours, &mut ps.inventory);
        }
        if let Some(ref mut sim) = self.sim {
            for _ in 0..hours {
                sim.step();
            }
        }
    }

    pub fn advance_clock_hour(&mut self) {
        self.advance_clock(1);
    }

    pub fn rest(&mut self) {
        self.advance_clock(8);
        self.vitals.rest();
        self.status_msg = Some("Rested (8h)".into());
    }

    pub fn clock_str(&self) -> String {
        let tod = self.clock.time_of_day();
        format!(
            "D{} {:02}:00 {}",
            self.clock.day,
            self.clock.hour,
            tod.glyph()
        )
    }
}

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
    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let result = self.compute_move(dx, dy);
        match result {
            Some(MoveResult::EdgeTransition { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.region_idx = region_idx;
                    p.px = px;
                    p.py = py;
                }
                self.advance_clock_hour();
                self.screen = Screen::Map { region_idx, px, py };
            }
            Some(MoveResult::Step { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.px = px;
                    p.py = py;
                }
                self.advance_clock_hour();
                self.screen = Screen::Map { region_idx, px, py };
            }
            Some(MoveResult::Blocked { msg }) => {
                self.status_msg = Some(msg);
            }
            None => {}
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
                if t.passable() {
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
        if let Some(ref pos) = self.player_pos {
            if let Some(ref sim) = self.sim {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if region.terrain.get(pos.px, pos.py) == Some(Terrain::Settlement) {
                        let mut idx = 0;
                        for (si, _sett) in region.settlements.iter().enumerate() {
                            let spacing = 40 / region.settlements.len().max(1);
                            let sx = (spacing / 2 + si * spacing).min(39);
                            if sx == pos.px {
                                idx = si;
                                break;
                            }
                        }
                        return Some((pos.region_idx, idx));
                    }
                }
            }
        }
        None
    }

    pub fn critical_need_people(&self) -> Vec<(String, String, String, Need, f64)> {
        let mut out = Vec::new();
        if let Some(ref sim) = self.sim {
            let needs = [
                Need::Food,
                Need::Money,
                Need::Care,
                Need::Presence,
                Need::Safety,
            ];
            for region in &sim.world.regions {
                for settlement in &region.settlements {
                    for person in &settlement.people {
                        for need in &needs {
                            let val = person.needs.get(*need);
                            if val < 0.2 {
                                out.push((
                                    person.name.clone(),
                                    person.settlement.clone(),
                                    person.profession.clone(),
                                    *need,
                                    val,
                                ));
                            }
                        }
                    }
                }
            }
        }
        out.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => match self.screen {
                Screen::CharacterCreation => match key.code {
                    crossterm::event::KeyCode::Char('r') => {
                        self.reroll_player();
                    }
                    crossterm::event::KeyCode::Enter => {
                        if self.player_start.is_none() {
                            self.generate_player();
                        }
                        self.accept_player();
                    }
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    _ => {}
                },
                Screen::World => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char('a') => {
                        if let Some(ref mut sim) = self.sim {
                            for _ in 0..10 {
                                sim.step();
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        let list = self.settlement_list();
                        if let Some((ri, si, _)) = list.get(idx) {
                            self.enter_settlement(*ri, *si);
                        }
                    }
                    crossterm::event::KeyCode::Char('j') => {
                        self.enter_journal();
                    }
                    crossterm::event::KeyCode::Char('s') => {
                        self.save_game();
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.load_game();
                    }
                    crossterm::event::KeyCode::Char('!') => {
                        self.enter_alerts();
                    }
                    crossterm::event::KeyCode::Char('m') => {
                        self.enter_map(0);
                    }
                    _ => {}
                },
                Screen::Map {
                    region_idx,
                    px: _,
                    py: _,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_map();
                    }
                    crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Left => {
                        self.move_player(-1, 0);
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.move_player(1, 0);
                    }
                    crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                        self.move_player(0, -1);
                    }
                    crossterm::event::KeyCode::Char('j') => {
                        self.move_player(0, 1);
                    }
                    crossterm::event::KeyCode::Right => {
                        self.move_player(1, 0);
                    }
                    crossterm::event::KeyCode::Down => {
                        self.move_player(0, 1);
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some((ri, si)) = self.player_on_settlement() {
                            self.enter_settlement(ri, si);
                        }
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char(c @ '1'..='9') => {
                        let idx = (c as usize) - ('1' as usize);
                        if idx != region_idx {
                            self.enter_map(
                                idx.min(
                                    self.sim
                                        .as_ref()
                                        .map(|s| s.world.regions.len())
                                        .unwrap_or(1)
                                        - 1,
                                ),
                            );
                        }
                    }
                    crossterm::event::KeyCode::Char('M') => {
                        self.enter_overmap();
                    }
                    crossterm::event::KeyCode::Char('i') => {
                        self.enter_inventory();
                    }
                    crossterm::event::KeyCode::Char('g') => {
                        self.gather();
                    }
                    crossterm::event::KeyCode::Char('r') => {
                        self.rest();
                    }
                    crossterm::event::KeyCode::Char('c') => {
                        self.enter_craft();
                    }
                    _ => {}
                },
                Screen::Overmap { region_idx } => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('M') => {
                        self.exit_overmap();
                    }
                    crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Left => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(west) = region.neighbors.west {
                                    self.screen = Screen::Overmap { region_idx: west };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('l') | crossterm::event::KeyCode::Right => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(east) = region.neighbors.east {
                                    self.screen = Screen::Overmap { region_idx: east };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(north) = region.neighbors.north {
                                    self.screen = Screen::Overmap { region_idx: north };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(south) = region.neighbors.south {
                                    self.screen = Screen::Overmap { region_idx: south };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.screen = Screen::Map {
                            region_idx,
                            px: 20,
                            py: 10,
                        };
                    }
                    _ => {}
                },
                Screen::Inventory => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('i') => {
                        self.exit_inventory();
                    }
                    _ => {}
                },
                Screen::Craft { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_craft();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Char(c @ '1'..='9') => {
                        let idx = (c as usize) - ('1' as usize);
                        self.craft_recipe(idx);
                    }
                    _ => {}
                },
                Screen::Location {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_settlement();
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.enter_npc(region_idx, settlement_idx, 0);
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        self.enter_npc(region_idx, settlement_idx, idx);
                    }
                    _ => {}
                },
                Screen::Npc {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_npc(region_idx, settlement_idx);
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char('t') => {
                        self.enter_talk(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::WorldAlerts { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_alerts();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Journal { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_journal();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Talk {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_talk(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Char('f') => {
                        self.give_food(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Char('c') => {
                        self.give_coin(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
            },
            AppEvent::Tick => {}
        }
    }
}
