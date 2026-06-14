use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_talk_input(
    app: &mut App,
    key: KeyEvent,
    scroll: u16,
    region_idx: usize,
    settlement_idx: usize,
    person_idx: usize,
) -> u16 {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.exit_talk(region_idx, settlement_idx, person_idx);
            return 0;
        }
        KeyCode::Char('f') => {
            app.give_food(region_idx, settlement_idx, person_idx);
        }
        KeyCode::Char('c') => {
            app.give_coin(region_idx, settlement_idx, person_idx);
        }
        KeyCode::Char('h') => {
            app.heal_npc(region_idx, settlement_idx, person_idx);
        }
        KeyCode::Down => {
            return scroll.saturating_add(1);
        }
        KeyCode::Up => {
            return scroll.saturating_sub(1);
        }
        _ => {}
    }
    scroll
}
