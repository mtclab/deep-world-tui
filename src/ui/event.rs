use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn poll(timeout: Duration) -> Option<AppEvent> {
    if event::poll(timeout).ok()? {
        match event::read().ok()? {
            CrosstermEvent::Key(key) => Some(AppEvent::Key(key)),
            CrosstermEvent::Resize(_, _) => None,
            _ => None,
        }
    } else {
        Some(AppEvent::Tick)
    }
}
