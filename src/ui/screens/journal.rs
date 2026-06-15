use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub(crate) fn draw_journal_screen(f: &mut Frame, app: &App, scroll: u16) {
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
        Span::styled(" — Journal", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(ref sim) = app.sim {
        if sim.journal.is_empty() && sim.quests.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " The Archive holds no records yet.",
                Style::default().fg(theme.warm_brown()),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Time passes. Events will be recorded here.",
                Style::default().fg(theme.dark_brown()),
            )));
        } else {
            for entry in sim.journal.entries.iter().rev() {
                let color = match entry.voice {
                    crate::sim::journal::Voice::Encounter => theme.ink(),
                    crate::sim::journal::Voice::Travel => theme.warm_brown(),
                    crate::sim::journal::Voice::Rest => theme.dark_ink(),
                    crate::sim::journal::Voice::Dream => theme.archive_red(),
                    crate::sim::journal::Voice::Scar => theme.archive_red(),
                    crate::sim::journal::Voice::Rumor => theme.ink(),
                    crate::sim::journal::Voice::Faith => theme.warm_brown(),
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" [{}] ", entry.tick),
                        Style::default()
                            .fg(theme.archive_red())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(entry.text.clone(), Style::default().fg(color)),
                ]));
            }
            if !sim.quests.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " What weighs on me:",
                    Style::default()
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                )));
                for quest in &sim.quests {
                    lines.push(Line::from(Span::styled(
                        format!(" {}", quest.description),
                        Style::default().fg(theme.warm_brown()),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("   {}", quest.progress_hint()),
                        Style::default().fg(theme.dark_brown()),
                    )));
                }
            }
        }
    }

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc/Q]",
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
