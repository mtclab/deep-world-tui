use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;
use crate::model::EncounterAction;

pub fn handle_encounter_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.resolve_encounter(EncounterAction::Flee);
        }
        KeyCode::Enter => {
            app.resolve_encounter(EncounterAction::Flee);
        }
        KeyCode::Char(c) => {
            if let Some(enc) = app.encounter {
                for action in enc.available_actions() {
                    if action.key() == c {
                        app.resolve_encounter(action);
                        break;
                    }
                }
            }
        }
        _ => {}
    }
}
