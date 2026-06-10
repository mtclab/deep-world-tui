use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_location_input(
    app: &mut App,
    key: KeyEvent,
    scroll: u16,
    region_idx: usize,
    settlement_idx: usize,
) -> u16 {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.exit_settlement();
            return 0;
        }
        KeyCode::Char(' ') => {
            if let Some(ref mut sim) = app.sim {
                sim.step();
            }
        }
        KeyCode::Down => {
            return scroll.saturating_add(1);
        }
        KeyCode::Up => {
            return scroll.saturating_sub(1);
        }
        KeyCode::Enter => {
            app.enter_npc(region_idx, settlement_idx, 0);
        }
        KeyCode::Char(c) if ('1'..='9').contains(&c) => {
            let idx = (c as usize) - ('1' as usize);
            app.enter_npc(region_idx, settlement_idx, idx);
        }
        KeyCode::Char('m') => {
            app.enter_market(region_idx, settlement_idx);
        }
        KeyCode::Char('s') => {
            if let Some(ref sim) = app.sim {
                if let Some(region) = sim.world.regions.get(region_idx) {
                    if let Some(settlement) = region.settlements.get(settlement_idx) {
                        if let Some(&service) = settlement.services.first() {
                            app.use_service(service);
                        } else {
                            app.status_msg = Some("No services here".into());
                        }
                    }
                }
            }
        }
        _ => {}
    }
    scroll
}
