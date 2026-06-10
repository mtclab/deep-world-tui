use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_inventory_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('i') => {
            app.exit_inventory();
        }
        _ => {}
    }
}
