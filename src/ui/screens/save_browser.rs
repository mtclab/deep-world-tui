use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::App;
use crate::ui::theme::Theme;

use super::common::focus_cursor;

pub(crate) fn draw_save_browser_screen(
    f: &mut Frame,
    app: &App,
    scroll: u16,
    delete_confirm: Option<usize>,
) {
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
        Span::styled(" — Save Archives", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if app.save_entries.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " The Archive holds no records of past journeys.",
            Style::default().fg(theme.warm_brown()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Begin a new story, and it shall be remembered.",
            Style::default().fg(theme.dark_brown()),
        )));
    } else {
        lines.push(Line::from(""));
        for (i, entry) in app.save_entries.iter().enumerate() {
            let cursor = focus_cursor((i as u16) == scroll);
            if let (Some(name), Some(people)) = (&entry.character_name, &entry.people) {
                let pk = crate::model::PeopleKind::from_name(people);
                let desc = format!(
                    "A {} wanderer, {} days into the deep",
                    pk.label(),
                    entry.day
                );
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}[{}] ", cursor, i + 1),
                        Style::default()
                            .fg(theme.archive_red())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} ", name),
                        Style::default()
                            .fg(theme.ink())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(desc, Style::default().fg(theme.dark_brown())),
                ]));
            } else {
                let desc = format!("An unknown traveler, {} days in", entry.day);
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}[{}] ", cursor, i + 1),
                        Style::default()
                            .fg(theme.archive_red())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(entry.filename.clone(), Style::default().fg(theme.ink())),
                    Span::styled(
                        format!(" — {}", desc),
                        Style::default().fg(theme.dark_brown()),
                    ),
                ]));
            }
            if delete_confirm == Some(i) {
                lines.push(Line::from(Span::styled(
                    "     Erase this record? Press D again to confirm.",
                    Style::default().fg(theme.need_color(0.0)),
                )));
            }
        }
    }

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Enter]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" load  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[D]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" delete  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            " [↑↓]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" select  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
