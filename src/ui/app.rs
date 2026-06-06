use crate::charts::Charts;
use crate::gen::player::generate_player_start;
use crate::model::{PlayerStart, Settlement};
use crate::rng::SeedRng;
use crate::sim::SimState;

use super::event::AppEvent;

pub enum Screen {
    CharacterCreation,
    World,
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
}

pub struct App {
    pub sim: Option<SimState>,
    pub player_start: Option<PlayerStart>,
    pub running: bool,
    pub tick_interval: u64,
    pub screen: Screen,
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
                    ..
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_npc(region_idx, settlement_idx);
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
            },
            AppEvent::Tick => {}
        }
    }
}
