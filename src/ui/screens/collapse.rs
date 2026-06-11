use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_collapse_screen(f: &mut Frame, app: &App) {
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

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " You collapsed!",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    if let Some(collapse) = app.collapse {
        lines.push(Line::from(Span::styled(
            collapse.outcome.description(),
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", collapse.outcome.glyph()),
            Style::default().fg(theme.archive_red()),
        )));
        if let Some(god) = collapse.rescued_by {
            lines.push(Line::from(Span::styled(
                format!(
                    "  Something of {} lingered in that place. Or so it felt.",
                    god.label()
                ),
                Style::default().fg(theme.warm_brown()),
            )));
        }
        if collapse.outcome.is_hostile() {
            lines.push(Line::from(Span::styled(
                "  You are wounded and shaken.",
                Style::default().fg(theme.need_color(0.0)),
            )));
        }
        if collapse.outcome.is_beast_aided() {
            lines.push(Line::from(Span::styled(
                "  The forest creatures remember your kindness.",
                Style::default().fg(theme.need_color(1.0)),
            )));
        }
        if collapse.outcome.is_divine() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  The fire's warmth lingers. Something impossible happened here.",
                Style::default().fg(theme.warm_brown()),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {} hours passed", collapse.outcome.hours_passed()),
            Style::default().fg(theme.dark_brown()),
        )));
        if collapse.outcome.coin_loss() > 0 {
            lines.push(Line::from(Span::styled(
                format!("  Lost {} coins", collapse.outcome.coin_loss()),
                Style::default().fg(theme.need_color(0.0)),
            )));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  Hunger: {}  Energy: {}  |  Day {}  Encounters: {}",
            app.vitals.hunger_label(),
            app.vitals.energy_label(),
            app.clock.day,
            app.encounters_had
        ),
        Style::default().fg(theme.dark_brown()),
    )));
    if app.god_affinity.oltzed != 0.0
        || app.god_affinity.keuru != 0.0
        || app.god_affinity.sampsa != 0.0
        || app.god_affinity.masa != 0.0
        || app.god_affinity.kukri != 0.0
    {
        lines.push(Line::from(Span::styled(
            format!(
                "  Gods: Oltzed {:.0}%  Keuru {:.0}%  Sampsa {:.0}%  Masa {:.0}%  Kukri {:.0}%",
                app.god_affinity.oltzed * 100.0,
                app.god_affinity.keuru * 100.0,
                app.god_affinity.sampsa * 100.0,
                app.god_affinity.masa * 100.0,
                app.god_affinity.kukri * 100.0,
            ),
            Style::default().fg(theme.dark_brown()),
        )));
    }

    let para = Paragraph::new(lines).style(Style::default().bg(theme.paper()));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Enter/Esc]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" continue", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
