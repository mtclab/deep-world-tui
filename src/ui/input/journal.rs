use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_journal_input(app: &mut App, key: KeyEvent, scroll: u16) -> u16 {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.exit_journal();
            0
        }
        KeyCode::Down => scroll.saturating_add(1),
        KeyCode::Up => scroll.saturating_sub(1),
        _ => scroll,
    }
}
