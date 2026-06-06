use crate::charts::Charts;
use crate::sim::SimState;

use super::event::AppEvent;

pub struct App {
    pub sim: SimState,
    pub running: bool,
    pub tick_interval: u64,
}

impl App {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let sim = SimState::new(seed, charts);
        App {
            sim,
            running: true,
            tick_interval: 100,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => match key.code {
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                    self.running = false;
                }
                crossterm::event::KeyCode::Char(' ') => {
                    self.sim.step();
                }
                crossterm::event::KeyCode::Char('a') => {
                    for _ in 0..10 {
                        self.sim.step();
                    }
                }
                _ => {}
            },
            AppEvent::Tick => {}
        }
    }
}
