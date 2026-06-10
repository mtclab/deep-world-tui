use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::{App, Screen};

pub fn handle_rest_prompt_input(app: &mut App, key: KeyEvent, hours: u32) {
    let set = |app: &mut App, h: u32| {
        app.screen = Screen::RestPrompt {
            hours: h.clamp(1, App::MAX_REST_HOURS),
        };
    };
    match key.code {
        KeyCode::Char('k') | KeyCode::Up | KeyCode::Char('+') | KeyCode::Right => {
            set(app, hours + 1);
        }
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('-') | KeyCode::Left => {
            set(app, hours.saturating_sub(1).max(1));
        }
        KeyCode::Char(c @ '1'..='9') => {
            set(app, c as u32 - '0' as u32);
        }
        KeyCode::Enter | KeyCode::Char('r') => {
            app.rest_hours(hours);
            app.return_to_world();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.return_to_world();
        }
        _ => {}
    }
}
