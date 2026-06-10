use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::{App, Screen};

pub fn handle_character_creation_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') => {
            app.reroll_player();
        }
        KeyCode::Enter => {
            if app.player_start.is_none() {
                app.generate_player();
            }
            app.accept_player();
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::TitleScreen;
        }
        _ => {}
    }
}
