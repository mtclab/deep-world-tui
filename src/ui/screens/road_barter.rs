use crate::model::PeopleKind;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Barter with a trader met on the road (#649 slice 2b): a small panel of what
/// this people will take from what the player carries, and what they give for
/// it, in kind. No coin, no haggle — pick a good to lay down. Not the old
/// encounter popup: a trade laid out where you stopped on the road.
pub(crate) fn draw_road_barter_screen(f: &mut Frame, app: &App, people: PeopleKind) {
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
            format!(" A {} trader", people.label()),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — barter on the road (no coin)",
            Style::default().fg(theme.warm_brown()),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let offers = app.road_barter_offers(people);
    let mut lines: Vec<Line> = Vec::new();
    if offers.is_empty() {
        lines.push(Line::from(Span::styled(
            "They look over what you carry and shake their heads — nothing here they want today.",
            Style::default().fg(theme.dark_ink()),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "They lay out their goods. Press a number to lay down yours in trade:",
            Style::default().fg(theme.dark_ink()),
        )));
        lines.push(Line::from(""));
        for (i, (offered, cost, gives)) in offers.iter().enumerate() {
            let gives_str: Vec<String> = gives
                .iter()
                .map(|(it, q)| format!("{} {}", q, it.name()))
                .collect();
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}. ", i + 1),
                    Style::default()
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("give {} {}", cost, offered.name()),
                    Style::default().fg(theme.ink()),
                ),
                Span::styled("  →  ", Style::default().fg(theme.dark_ink())),
                Span::styled(
                    format!("get {}", gives_str.join(", ")),
                    Style::default().fg(theme.warm_brown()),
                ),
            ]));
        }
    }
    let body = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(body, chunks[1]);

    let footer = Paragraph::new(Line::from(Span::styled(
        " 1-9 trade · Esc walk on ",
        Style::default().fg(theme.dark_ink()),
    )))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
