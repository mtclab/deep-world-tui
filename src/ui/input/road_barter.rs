use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::{App, Screen};
use crate::model::PeopleKind;

/// Road-barter input (#649 slice 2b): a number lays down that good in trade; Esc
/// (or q) walks on. The panel only lists trades the player can make, so a number
/// in range always resolves to a real barter.
pub fn handle_road_barter_input(app: &mut App, key: KeyEvent, people: PeopleKind) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
            let region_idx = app.player_pos.map(|p| p.region_idx).unwrap_or(0);
            app.screen = Screen::World { region_idx };
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let n = c.to_digit(10).unwrap_or(0) as usize;
            if n == 0 {
                return;
            }
            let offers = app.road_barter_offers(people);
            if let Some((offered, _, _)) = offers.get(n - 1) {
                app.road_barter(people, *offered);
                // A trade leaves the trader; walk on after, the deal made.
                let region_idx = app.player_pos.map(|p| p.region_idx).unwrap_or(0);
                app.screen = Screen::World { region_idx };
            }
        }
        _ => {}
    }
}
