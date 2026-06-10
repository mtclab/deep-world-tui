use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::App;
use crate::ui::theme::Theme;

pub(crate) fn draw_title_screen(f: &mut Frame, app: &App) {
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
            " THE DEEP WORLD",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(theme.dark_brown()),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let flavor_lines = [
        " The Archive watches from beyond the mountain. The forge-smoke",
        " rises. The forest remembers. Your steps have not yet been written.",
        "",
        " A threshold of fates stands before you, unwalked.",
    ];
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    for line in &flavor_lines {
        lines.push(Line::from(Span::styled(
            *line,
            Style::default().fg(theme.warm_brown()),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [N]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" New Game  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[L]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Load Game  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[?]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Controls  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Q]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quit", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
