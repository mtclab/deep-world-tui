use crate::charts::Charts;
use crate::gen::player::generate_player_start;
use crate::model::PlayerStart;
use crate::rng::SeedRng;
use crate::sim::SimState;

use super::event::AppEvent;

pub enum Screen {
    CharacterCreation,
    World,
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
                    _ => {}
                },
            },
            AppEvent::Tick => {}
        }
    }
}
