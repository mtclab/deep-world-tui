use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_market_input(app: &mut App, key: KeyEvent, scroll: u16) -> u16 {
    // The shelf can be long; the screen shows a window of MARKET_WINDOW rows
    // starting at `scroll`, and the 1-6 / a-f / A-F keys act on that window.
    let items = app.market_items();
    let win = App::MARKET_WINDOW;
    let max_scroll = items.len().saturating_sub(win) as u16;
    let off = scroll.min(max_scroll) as usize;
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.exit_market();
            return 0;
        }
        KeyCode::Down => {
            return scroll.saturating_add(1).min(max_scroll);
        }
        KeyCode::Up => {
            return scroll.saturating_sub(1);
        }
        KeyCode::Char(c) if ('1'..='6').contains(&c) => {
            let idx = off + (c as usize) - ('1' as usize);
            if let Some(&item) = items.get(idx) {
                app.buy_item(item);
            }
        }
        KeyCode::Char(c) if ('a'..='f').contains(&c) => {
            let idx = off + (c as usize) - ('a' as usize);
            if let Some(&item) = items.get(idx) {
                app.sell_item(item);
            }
        }
        // Shift+letter: the other way of acquiring things.
        KeyCode::Char(c) if ('A'..='F').contains(&c) => {
            let idx = off + (c as usize) - ('A' as usize);
            if let Some(&item) = items.get(idx) {
                app.steal_item(item);
            }
        }
        _ => {}
    }
    scroll.min(max_scroll)
}
