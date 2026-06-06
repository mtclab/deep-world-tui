use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    draw_header(f, chunks[0], app);
    draw_world(f, chunks[1], app);
    draw_footer(f, chunks[2]);
}

fn draw_header(f: &mut Frame, area: Rect, _app: &App) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Deep World",
            Style::default()
                .fg(Color::Rgb(0x7a, 0x2e, 0x1d))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — Archive of Ahjorath",
            Style::default().fg(Color::Rgb(0x8b, 0x73, 0x55)),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, area);
}

fn draw_world(f: &mut Frame, area: Rect, app: &App) {
    let world = &app.sim.world;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        format!(" Tick {}", world.tick),
        Style::default()
            .fg(Color::Rgb(0x7a, 0x2e, 0x1d))
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (ri, region) in world.regions.iter().enumerate() {
        lines.push(Line::from(Span::styled(
            format!(" {} [{}]", region.name, region.region_type),
            Style::default()
                .fg(Color::Rgb(0xc2, 0x9a, 0x6b))
                .add_modifier(Modifier::BOLD),
        )));

        for settlement in &region.settlements {
            let size_label = match settlement.population {
                p if p >= 1000 => "city",
                p if p >= 400 => "town",
                p if p >= 100 => "village",
                _ => "hamlet",
            };
            lines.push(Line::from(Span::styled(
                format!(
                    "   {} ({}, pop {})",
                    settlement.name, size_label, settlement.population
                ),
                Style::default().fg(Color::Rgb(0x8b, 0x73, 0x55)),
            )));

            let people_shown = settlement.people.len().min(5);
            for person in settlement.people.iter().take(people_shown) {
                lines.push(Line::from(Span::styled(
                    format!(
                        "     {} — {} ({})",
                        person.name, person.profession, person.people
                    ),
                    Style::default().fg(Color::Rgb(0x5a, 0x4a, 0x3a)),
                )));
            }
            if settlement.people.len() > 5 {
                lines.push(Line::from(Span::styled(
                    format!("     … +{} more", settlement.people.len() - 5),
                    Style::default().fg(Color::Rgb(0x5a, 0x4a, 0x3a)),
                )));
            }
        }

        if ri < world.regions.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::Rgb(0xef, 0xe9, 0xdd))),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Space]",
            Style::default()
                .fg(Color::Rgb(0x7a, 0x2e, 0x1d))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" step  ", Style::default().fg(Color::Rgb(0x5a, 0x4a, 0x3a))),
        Span::styled(
            "[A]",
            Style::default()
                .fg(Color::Rgb(0x7a, 0x2e, 0x1d))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" x10  ", Style::default().fg(Color::Rgb(0x5a, 0x4a, 0x3a))),
        Span::styled(
            "[Q/Esc]",
            Style::default()
                .fg(Color::Rgb(0x7a, 0x2e, 0x1d))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(Color::Rgb(0x5a, 0x4a, 0x3a))),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, area);
}
