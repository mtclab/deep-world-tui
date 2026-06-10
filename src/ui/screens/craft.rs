use crate::model::craft_recipes;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_craft_screen(f: &mut Frame, app: &App, scroll: u16) {
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
        " Craft",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let inv = app.player_inventory();
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    let player_people = app.inter_people_bias.player_people;
    for (i, recipe) in craft_recipes()
        .iter()
        .enumerate()
        .filter(|(_, r)| r.people.is_none() || r.people == Some(player_people))
    {
        let key = format!("{}", i + 1);
        let has_all = recipe
            .inputs
            .iter()
            .all(|&(item, count)| inv.get(item) >= count);
        let inputs: String = recipe
            .inputs
            .iter()
            .map(|&(item, count)| format!("{}x{} ", count, item.name()))
            .collect::<Vec<_>>()
            .join("+ ");
        let can_color = if has_all {
            theme.need_color(1.0)
        } else {
            theme.dark_brown()
        };
        let people_tag = if let Some(pk) = recipe.people {
            format!(" [{}]", pk.label())
        } else {
            String::new()
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" [{}] ", key),
                Style::default()
                    .fg(theme.archive_red())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<10}{}", recipe.name, people_tag),
                Style::default().fg(can_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({})", inputs),
                Style::default().fg(theme.dark_brown()),
            ),
            Span::styled(
                format!(" -> {}x{}", recipe.output_count, recipe.output.name()),
                Style::default().fg(can_color),
            ),
        ]));
    }

    let para = Paragraph::new(lines)
        .style(Style::default().bg(theme.paper()))
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [1-9]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" craft  ", Style::default().fg(theme.dark_brown())),
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
