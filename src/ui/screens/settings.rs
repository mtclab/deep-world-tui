use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_settings_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
    };
    let area = f.area();
    let llm_status = if app.llm_enabled {
        "ON  (persona prompts from LLM)"
    } else {
        "OFF (using voice.rs templates)"
    };
    let mono_status = if app.monochrome {
        "ON  (ink-only palette for accessibility)"
    } else {
        "OFF (full color palette)"
    };
    let text = vec![
        Line::from(Span::styled(
            "=== Settings ===",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " LLM Narrator",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("   Status: {}", llm_status)),
        Line::from(format!("   Endpoint: {}", app.llm_endpoint)),
        Line::from(format!("   Model: {}", app.llm_model)),
        Line::from("   [l] Toggle LLM narrator on/off"),
        Line::from("   [e] Edit endpoint  [o] Edit model"),
        Line::from(""),
        Line::from(Span::styled(
            " Monochrome".to_string(),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("   Status: {}", mono_status)),
        Line::from("   [m] Toggle monochrome mode"),
        Line::from(""),
        Line::from(Span::styled(
            " High Contrast".to_string(),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "   Status: {}",
            if app.high_contrast { "on" } else { "off" }
        )),
        Line::from("   [h] Toggle high contrast mode"),
        Line::from(""),
        Line::from(Span::styled(
            " Animations".to_string(),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "   Reduced motion: {}",
            if app.reduced_motion { "on" } else { "off" }
        )),
        Line::from("   [p] Toggle reduced motion (disables animations)"),
        Line::from(""),
        Line::from(Span::styled(
            " Language".to_string(),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   Current: en"),
        Line::from(""),
        Line::from(Span::styled(
            " Audio",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "   Status: {}",
            if app.audio_enabled {
                "ON  (sound effects enabled)"
            } else {
                "OFF (silent)"
            }
        )),
        Line::from(format!("   Volume: {:.0}%", app.audio_volume * 100.0)),
        Line::from("   [a] Toggle audio   [+/-] Volume"),
        Line::from(""),
        Line::from(" [Esc/Q/,]  Back to game"),
    ];
    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Settings ")
            .border_style(Style::default().fg(theme.archive_red())),
    );
    f.render_widget(paragraph, area);
}
