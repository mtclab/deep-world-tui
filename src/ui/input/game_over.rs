use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_game_over_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') => {
            app.restart_game();
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
        }
        _ => {}
    }
}
