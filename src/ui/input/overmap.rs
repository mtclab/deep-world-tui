use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::{App, Screen};

pub fn handle_overmap_input(app: &mut App, key: KeyEvent, region_idx: usize) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('M') => {
            app.exit_overmap();
        }
        KeyCode::Char('h') | KeyCode::Left => {
            if let Some(ref sim) = app.sim {
                if let Some(region) = sim.world.regions.get(region_idx) {
                    if let Some(west) = region.neighbors.west {
                        app.screen = Screen::Overmap { region_idx: west };
                    }
                }
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if let Some(ref sim) = app.sim {
                if let Some(region) = sim.world.regions.get(region_idx) {
                    if let Some(east) = region.neighbors.east {
                        app.screen = Screen::Overmap { region_idx: east };
                    }
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(ref sim) = app.sim {
                if let Some(region) = sim.world.regions.get(region_idx) {
                    if let Some(north) = region.neighbors.north {
                        app.screen = Screen::Overmap { region_idx: north };
                    }
                }
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(ref sim) = app.sim {
                if let Some(region) = sim.world.regions.get(region_idx) {
                    if let Some(south) = region.neighbors.south {
                        app.screen = Screen::Overmap { region_idx: south };
                    }
                }
            }
        }
        KeyCode::Enter => {
            app.screen = Screen::World { region_idx };
        }
        _ => {}
    }
}
