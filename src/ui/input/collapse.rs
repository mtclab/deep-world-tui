use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_collapse_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
            app.dismiss_collapse();
        }
        _ => {}
    }
}
