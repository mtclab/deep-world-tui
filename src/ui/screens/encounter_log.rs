use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub(crate) fn draw_encounter_log_screen(f: &mut Frame, app: &App, scroll: u16) {
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
            " Deep World",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — Encounter Log", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if app.encounter_log.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " No encounters yet.",
            Style::default().fg(theme.warm_brown()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Roads, forests, storms — events will be recorded here.",
            Style::default().fg(theme.dark_brown()),
        )));
    } else {
        for entry in app.encounter_log.iter().rev() {
            let day_str = format!("day {:>3}  {:02}:00", entry.day, entry.hour);
            let kind_color = if entry.hostile {
                theme.archive_red()
            } else {
                theme.warm_brown()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {}  ", day_str),
                    Style::default().fg(theme.archive_red()),
                ),
                Span::styled(
                    format!("{:<9} ", format!("{:?}", entry.kind)),
                    Style::default().fg(kind_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:>10}  ", format!("{:?}", entry.terrain)),
                    Style::default().fg(theme.dark_brown()),
                ),
                Span::styled(
                    format!("via {}", entry.action.label()),
                    Style::default().fg(theme.ink()),
                ),
            ]));
        }
    }

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc/Q/H/←]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[↑↓]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
