use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_faith_input(app: &mut App, key: KeyEvent, scroll: u16) -> u16 {
    match key.code {
        KeyCode::Char('q')
        | KeyCode::Esc
        | KeyCode::Char('F')
        | KeyCode::Char('f')
        | KeyCode::Left => {
            app.exit_faith();
            0
        }
        KeyCode::Down | KeyCode::Char('j') => scroll.saturating_add(1),
        KeyCode::Up | KeyCode::Char('k') => scroll.saturating_sub(1),
        _ => scroll,
    }
}
