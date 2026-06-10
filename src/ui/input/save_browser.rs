use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::{App, Screen};

pub fn handle_save_browser_input(app: &mut App, key: KeyEvent) {
    let (scroll_val, confirm_val) = match app.screen {
        Screen::SaveBrowser {
            scroll,
            delete_confirm,
        } => (scroll, delete_confirm),
        _ => return,
    };
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            let max_scroll = app.save_entries.len().saturating_sub(1) as u16;
            app.screen = Screen::SaveBrowser {
                scroll: scroll_val.min(max_scroll).saturating_add(1).min(max_scroll),
                delete_confirm: confirm_val,
            };
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.screen = Screen::SaveBrowser {
                scroll: scroll_val.saturating_sub(1),
                delete_confirm: confirm_val,
            };
        }
        KeyCode::Enter => {
            if let Some(entry) = app.save_entries.get(scroll_val as usize) {
                match crate::save::load_game_file(&entry.filename) {
                    // Shared restore path — the browser's own field list had
                    // drifted from load_game (it dropped `explored`, and like
                    // slot-load it never re-anchored the seed).
                    Ok(data) => app.apply_save_data(data),
                    Err(e) => {
                        app.status_msg = Some(format!("Load failed: {}", e));
                    }
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(idx) = confirm_val {
                if idx == scroll_val as usize {
                    if let Some(entry) = app.save_entries.get(idx) {
                        let _ = crate::save::delete_save(&entry.filename);
                    }
                    app.save_entries = crate::save::saves_dir_list();
                    let new_scroll = if scroll_val as usize >= app.save_entries.len() {
                        app.save_entries.len().saturating_sub(1) as u16
                    } else {
                        scroll_val
                    };
                    app.screen = Screen::SaveBrowser {
                        scroll: new_scroll,
                        delete_confirm: None,
                    };
                } else {
                    app.screen = Screen::SaveBrowser {
                        scroll: scroll_val,
                        delete_confirm: Some(scroll_val as usize),
                    };
                }
            } else if !app.save_entries.is_empty() {
                app.screen = Screen::SaveBrowser {
                    scroll: scroll_val,
                    delete_confirm: Some(scroll_val as usize),
                };
            }
        }
        KeyCode::Esc => {
            app.screen = Screen::TitleScreen;
        }
        _ => {}
    }
}
