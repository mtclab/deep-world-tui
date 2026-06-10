use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::{App, Screen};

pub fn handle_title_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('n') => {
            app.screen = Screen::CharacterCreation;
        }
        KeyCode::Char('l') => {
            app.save_entries = crate::save::saves_dir_list();
            app.screen = Screen::SaveBrowser {
                scroll: 0,
                delete_confirm: None,
            };
        }
        KeyCode::Char('?') => {
            app.previous_screen = Some(Screen::TitleScreen);
            app.screen = Screen::Help;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
        }
        _ => {}
    }
}
