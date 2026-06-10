use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;
use crate::model::ItemType;

pub fn handle_market_input(app: &mut App, key: KeyEvent, scroll: u16) -> u16 {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.exit_market();
            return 0;
        }
        KeyCode::Down => {
            return scroll.saturating_add(1);
        }
        KeyCode::Up => {
            return scroll.saturating_sub(1);
        }
        KeyCode::Char(c) if ('1'..='6').contains(&c) => {
            let idx = (c as usize) - ('1' as usize);
            let items = ItemType::tradeable_items();
            if let Some(&item) = items.get(idx) {
                app.buy_item(item);
            }
        }
        KeyCode::Char(c) if ('a'..='f').contains(&c) => {
            let idx = (c as usize) - ('a' as usize);
            let items = ItemType::tradeable_items();
            if let Some(&item) = items.get(idx) {
                app.sell_item(item);
            }
        }
        // Shift+letter: the other way of acquiring things.
        KeyCode::Char(c) if ('A'..='F').contains(&c) => {
            let idx = (c as usize) - ('A' as usize);
            let items = ItemType::tradeable_items();
            if let Some(&item) = items.get(idx) {
                app.steal_item(item);
            }
        }
        _ => {}
    }
    scroll
}
