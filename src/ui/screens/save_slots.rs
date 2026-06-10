use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::save::{slot_filename, SAVE_SLOT_COUNT};
use crate::ui::app::App;
use crate::ui::theme::Theme;

use super::common::focus_cursor;

pub(crate) fn draw_save_slots_screen(f: &mut Frame, app: &App, scroll: u16) {
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
        Span::styled(" — Save to slot", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = vec![Line::from("")];
    for slot in 1..=SAVE_SLOT_COUNT {
        let cursor = focus_cursor((slot as u16 - 1) == scroll);
        let fname = slot_filename(slot);
        let entry = app.save_entries.iter().find(|e| e.filename == fname);
        let desc = match entry {
            Some(e) => match (&e.character_name, &e.people) {
                (Some(name), Some(people)) => format!("{name} the {people} — day {}", e.day),
                _ => format!("occupied — day {}", e.day),
            },
            None => "(empty)".to_string(),
        };
        let style = if entry.is_some() {
            Style::default().fg(theme.ink())
        } else {
            Style::default().fg(theme.dark_brown())
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {cursor} Slot {slot}: "),
                Style::default().fg(theme.warm_brown()),
            ),
            Span::styled(desc, style),
        ]));
    }
    let body = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
    f.render_widget(body, chunks[1]);

    let footer = Paragraph::new(Line::from(Span::styled(
        " ↑/↓ or j/k select · Enter or 1-9 save · Esc cancel (overwrites existing)",
        Style::default().fg(theme.dark_brown()),
    )))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
