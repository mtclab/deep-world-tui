use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_help_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
            if let Some(prev) = app.previous_screen.take() {
                app.screen = prev;
            }
        }
        _ => {}
    }
}
