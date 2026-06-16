use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_world_input(app: &mut App, key: KeyEvent, region_idx: usize) {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            app.move_player(-1, 0);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.move_player(1, 0);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_player(0, -1);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_player(0, 1);
        }
        KeyCode::Char('w') | KeyCode::Char('.') => {
            app.advance_clock(1);
        }
        KeyCode::Enter => {
            // The town is walked, not entered (#372): doors serve, people
            // talk when bumped — Enter opens the market stall for trade.
            if let Some((ri, si)) = app.player_on_settlement() {
                app.enter_market(ri, si);
            }
        }
        KeyCode::Char(' ') => {
            if let Some(ref mut sim) = app.sim {
                sim.step();
            }
        }
        KeyCode::Char('a') => {
            if let Some((ri, si)) = app.player_on_settlement() {
                app.adopt_companion(ri, si);
            }
        }
        KeyCode::Char('g') => {
            app.gather();
        }
        KeyCode::Char('f') => {
            app.forage_herbs();
        }
        KeyCode::Char('J') => {
            app.journey_to_city();
        }
        KeyCode::Char('p') => {
            app.pray();
        }
        KeyCode::Char('O') => {
            app.observe_festival();
        }
        KeyCode::Char('o') => {
            app.make_offering();
        }
        KeyCode::Char('P') => {
            app.pilgrimage();
        }
        KeyCode::Char('G') => {
            app.give_gift();
        }
        KeyCode::Char('r') => {
            app.open_rest_prompt();
        }
        KeyCode::Char('R') => {
            // Quick full night's rest, skipping the picker.
            app.rest();
        }
        KeyCode::Char('i') => {
            app.enter_inventory();
        }
        KeyCode::Char('c') => {
            app.enter_craft();
        }
        KeyCode::Char('t') => {
            app.tend_illness();
        }
        KeyCode::Char('b') => {
            app.start_build();
        }
        KeyCode::Char('s') => {
            app.open_save_slots();
        }
        KeyCode::Char('M') => {
            app.enter_overmap();
        }
        KeyCode::Char('m') => {
            app.enter_journal();
        }
        KeyCode::Char('H') => {
            app.open_encounter_log();
        }
        KeyCode::Char('F') => {
            app.open_faith();
        }
        KeyCode::Char(c @ '1'..='9') => {
            let idx = (c as usize) - ('1' as usize);
            if idx != region_idx {
                app.enter_map(
                    idx.min(app.sim.as_ref().map(|s| s.world.regions.len()).unwrap_or(1) - 1),
                );
            }
        }
        KeyCode::Char('?') => {
            app.previous_screen = Some(app.screen.clone());
            app.screen = super::super::app::Screen::Help;
        }
        KeyCode::Char(',') => {
            app.previous_screen = Some(app.screen.clone());
            app.screen = super::super::app::Screen::Settings;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = super::super::app::Screen::TitleScreen;
        }
        _ => {}
    }
}
