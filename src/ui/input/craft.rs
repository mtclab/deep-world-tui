use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_craft_input(app: &mut App, key: KeyEvent, scroll: u16) -> u16 {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.exit_craft();
            0
        }
        KeyCode::Down => scroll.saturating_add(1),
        KeyCode::Up => scroll.saturating_sub(1),
        KeyCode::Char(c @ '1'..='9') => {
            let idx = (c as usize) - ('1' as usize);
            app.craft_recipe(idx);
            scroll
        }
        _ => scroll,
    }
}
