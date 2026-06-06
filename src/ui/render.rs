use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::{App, Screen};

const ARCHIVE_RED: Color = Color::Rgb(0x7a, 0x2e, 0x1d);
const INK: Color = Color::Rgb(0x3a, 0x2a, 0x1a);
const WARM_BROWN: Color = Color::Rgb(0x8b, 0x73, 0x55);
const DARK_BROWN: Color = Color::Rgb(0x5a, 0x4a, 0x3a);
const PAPER: Color = Color::Rgb(0xef, 0xe9, 0xdd);

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(Block::default().style(Style::default().bg(PAPER)), f.area());
    match app.screen {
        Screen::CharacterCreation => draw_character_creation(f, app),
        Screen::World => draw_world_screen(f, app),
    }
}

fn draw_character_creation(f: &mut Frame, app: &App) {
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
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — Who Are You?", Style::default().fg(WARM_BROWN)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref ps) = app.player_start {
        let p = &ps.person;
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " The fates have shaped you thus:",
            Style::default().fg(WARM_BROWN).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("   Name          {}", p.name),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   People        {}", p.people),
            Style::default().fg(INK),
        )));
        lines.push(Line::styled(
            format!("   Social Class  {}", p.social_class),
            Style::default().fg(INK),
        ));
        lines.push(Line::from(Span::styled(
            format!("   Craft         {}", p.craft_affinity),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Social Class  {}", p.social_class),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Age           {}", p.age_band),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Personality    {}", p.personality.join(", ")),
            Style::default().fg(INK),
        )));
        if p.has_spouse {
            lines.push(Line::from(Span::styled(
                "   Household     married",
                Style::default().fg(INK),
            )));
        }
        if p.children_count > 0 {
            lines.push(Line::from(Span::styled(
                format!("   Children      {}", p.children_count),
                Style::default().fg(INK),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("   Rerolls: {}", ps.reroll_count),
            Style::default().fg(DARK_BROWN),
        )));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " You stand at the threshold of the Kingdom of Ahjorath.",
            Style::default().fg(WARM_BROWN),
        )));
        lines.push(Line::from(Span::styled(
            " The Archive watches. The Sepát wait.",
            Style::default().fg(WARM_BROWN),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Press Enter to see who you might become.",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Enter]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" accept  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[R]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" reroll  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Q]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_world_screen(f: &mut Frame, app: &App) {
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
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — Archive of Ahjorath", Style::default().fg(WARM_BROWN)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref sim) = app.sim {
        let world = &sim.world;
        lines.push(Line::from(Span::styled(
            format!(" Tick {}", world.tick),
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (ri, region) in world.regions.iter().enumerate() {
            lines.push(Line::from(Span::styled(
                format!(" {} [{}]", region.name, region.region_type),
                Style::default().fg(WARM_BROWN).add_modifier(Modifier::BOLD),
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
                    Style::default().fg(DARK_BROWN),
                )));
                let people_shown = settlement.people.len().min(5);
                for person in settlement.people.iter().take(people_shown) {
                    lines.push(Line::from(Span::styled(
                        format!(
                            "     {} — {} ({})",
                            person.name, person.profession, person.people
                        ),
                        Style::default().fg(INK),
                    )));
                }
                if settlement.people.len() > 5 {
                    lines.push(Line::from(Span::styled(
                        format!("     … +{} more", settlement.people.len() - 5),
                        Style::default().fg(INK),
                    )));
                }
            }
            if ri < world.regions.len() - 1 {
                lines.push(Line::from(""));
            }
        }
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Space]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" step  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[A]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" x10  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Q/Esc]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
