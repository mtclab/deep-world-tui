use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::app::App;
use crate::ui::theme::Theme;

pub(crate) fn draw_rest_prompt_screen(f: &mut Frame, app: &App, hours: u32) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
    };
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Rest",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — how long?", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let label = match hours {
        1..=2 => "a short nap",
        3..=5 => "a brief rest",
        6..=8 => "a long rest",
        _ => "a full night",
    };
    let bar: String =
        "█".repeat(hours as usize) + &"░".repeat((App::MAX_REST_HOURS - hours) as usize);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("{hours}h  "),
                Style::default()
                    .fg(theme.ink())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(bar, Style::default().fg(theme.warm_brown())),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("   {label} — rest quality depends on where you are."),
            Style::default().fg(theme.dark_brown()),
        )),
    ];
    f.render_widget(Paragraph::new(lines), chunks[1]);

    let footer = Paragraph::new(Line::from(Span::styled(
        " ↑/↓ or +/- adjust · 1-9 set · Enter rest · Esc cancel",
        Style::default().fg(theme.dark_brown()),
    )))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
