/// Main TUI application state. Stub for issue #6.

pub struct App {
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self { running: true }
    }

    pub fn tick(&mut self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }
}