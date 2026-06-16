use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

/// Input for the city-visit panel (#456): scroll the city, or take the road
/// home. Read-only — the trade settled on the road.
pub fn handle_city_input(app: &mut App, key: KeyEvent, scroll: u16) -> u16 {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Left => {
            app.exit_city_visit();
            0
        }
        KeyCode::Down => scroll.saturating_add(1),
        KeyCode::Up => scroll.saturating_sub(1),
        _ => scroll,
    }
}
