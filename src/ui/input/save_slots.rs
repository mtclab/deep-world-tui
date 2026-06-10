use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::{App, Screen};
use crate::save::SAVE_SLOT_COUNT;

pub fn handle_save_slots_input(app: &mut App, key: KeyEvent, scroll: u16) {
    let max = (SAVE_SLOT_COUNT as u16).saturating_sub(1);
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.screen = Screen::SaveSlots {
                scroll: scroll.saturating_add(1).min(max),
            };
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.screen = Screen::SaveSlots {
                scroll: scroll.saturating_sub(1),
            };
        }
        // Direct slot select by digit.
        KeyCode::Char(c @ '1'..='9') => {
            let slot = c as usize - '0' as usize;
            if slot <= SAVE_SLOT_COUNT {
                app.save_to_slot(slot);
                app.return_to_world();
            }
        }
        KeyCode::Enter => {
            app.save_to_slot(scroll as usize + 1);
            app.return_to_world();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.return_to_world();
        }
        _ => {}
    }
}
