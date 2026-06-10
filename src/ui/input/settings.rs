use crossterm::event::{KeyCode, KeyEvent};

use super::super::app::App;

pub fn handle_settings_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char(',') => {
            if let Some(prev) = app.previous_screen.take() {
                app.screen = prev;
            }
        }
        KeyCode::Char('l') => {
            app.llm_enabled = !app.llm_enabled;
            app.status_msg = Some(if app.llm_enabled {
                "LLM narrator enabled".into()
            } else {
                "LLM narrator disabled (using voice.rs templates)".into()
            });
            app.save_settings();
        }
        KeyCode::Char('m') => {
            app.monochrome = !app.monochrome;
            app.status_msg = Some(if app.monochrome {
                "Monochrome mode on".into()
            } else {
                "Full color mode on".into()
            });
            app.save_settings();
        }
        KeyCode::Char('h') => {
            app.high_contrast = !app.high_contrast;
            app.status_msg = Some(if app.high_contrast {
                "High contrast mode on".into()
            } else {
                "Standard contrast mode on".into()
            });
            app.save_settings();
        }
        KeyCode::Char('p') => {
            app.reduced_motion = !app.reduced_motion;
            app.status_msg = Some(if app.reduced_motion {
                "Reduced motion on".into()
            } else {
                "Animations enabled".into()
            });
            app.save_settings();
        }
        KeyCode::Char('a') => {
            app.audio_enabled = !app.audio_enabled;
            app.status_msg = Some(if app.audio_enabled {
                "Audio enabled".into()
            } else {
                "Audio disabled".into()
            });
            app.save_settings();
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.audio_volume = (app.audio_volume + 0.1).clamp(0.0, 1.0);
            app.status_msg = Some(format!("Volume: {:.0}%", app.audio_volume * 100.0));
            app.save_settings();
        }
        KeyCode::Char('-') => {
            app.audio_volume = (app.audio_volume - 0.1).clamp(0.0, 1.0);
            app.status_msg = Some(format!("Volume: {:.0}%", app.audio_volume * 100.0));
            app.save_settings();
        }
        KeyCode::Char('e') => {
            let endpoints = [
                "http://localhost:11434/v1",
                "http://localhost:8080/v1",
                "https://api.openai.com/v1",
            ];
            let idx = endpoints
                .iter()
                .position(|e| *e == app.llm_endpoint)
                .unwrap_or(0);
            app.llm_endpoint = endpoints[(idx + 1) % endpoints.len()].to_string();
            app.status_msg = Some(format!("Endpoint: {}", app.llm_endpoint));
            app.save_settings();
        }
        KeyCode::Char('o') => {
            let models = ["llama3", "mistral", "gemma2", "phi3", "qwen2"];
            let idx = models.iter().position(|m| *m == app.llm_model).unwrap_or(0);
            app.llm_model = models[(idx + 1) % models.len()].to_string();
            app.status_msg = Some(format!("Model: {}", app.llm_model));
            app.save_settings();
        }
        _ => {}
    }
}
