use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::model::Need;
use crate::sim::relationships::BondCategory;
use crate::ui::app::{App, Screen};

const ARCHIVE_RED: Color = Color::Rgb(0x7a, 0x2e, 0x1d);
const INK: Color = Color::Rgb(0x3a, 0x2a, 0x1a);
const WARM_BROWN: Color = Color::Rgb(0x8b, 0x73, 0x55);
const DARK_BROWN: Color = Color::Rgb(0x5a, 0x4a, 0x3a);
const PAPER: Color = Color::Rgb(0xef, 0xe9, 0xdd);
const NEED_LOW: Color = Color::Rgb(0x6b, 0x8e, 0x4a);
const NEED_MID: Color = Color::Rgb(0xc2, 0x9a, 0x6b);
const NEED_HIGH: Color = Color::Rgb(0x7a, 0x2e, 0x1d);

fn need_color(val: f64) -> Color {
    if val >= 0.7 {
        NEED_LOW
    } else if val >= 0.3 {
        NEED_MID
    } else {
        NEED_HIGH
    }
}

fn need_bar(val: f64, width: usize) -> String {
    let filled = (val * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(Block::default().style(Style::default().bg(PAPER)), f.area());
    match app.screen {
        Screen::CharacterCreation => draw_character_creation(f, app),
        Screen::World => draw_world_screen(f, app),
        Screen::Location {
            region_idx,
            settlement_idx,
            scroll,
        } => {
            draw_location_screen(f, app, region_idx, settlement_idx, scroll);
        }
        Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll,
        } => {
            draw_npc_screen(f, app, region_idx, settlement_idx, person_idx, scroll);
        }
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
        lines.push(Line::from(Span::styled(
            format!("   Profession    {}", p.profession),
            Style::default().fg(INK),
        )));
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

        let settlements = app.settlement_list();
        for (ri, region) in world.regions.iter().enumerate() {
            lines.push(Line::from(Span::styled(
                format!(" {} [{}]", region.name, region.region_type),
                Style::default().fg(WARM_BROWN).add_modifier(Modifier::BOLD),
            )));
            for (si, settlement) in region.settlements.iter().enumerate() {
                let idx = settlements
                    .iter()
                    .position(|(r, s, _)| *r == ri && *s == si);
                let key_label = idx.map(|i| format!("{}", i + 1)).unwrap_or_default();
                let size_label = match settlement.population {
                    p if p >= 1000 => "city",
                    p if p >= 400 => "town",
                    p if p >= 100 => "village",
                    _ => "hamlet",
                };
                if key_label.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!(
                            "   {} ({}, pop {})",
                            settlement.name, size_label, settlement.population
                        ),
                        Style::default().fg(DARK_BROWN),
                    )));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  [{}]", key_label),
                            Style::default()
                                .fg(ARCHIVE_RED)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(
                                " {} ({}, pop {})",
                                settlement.name, size_label, settlement.population
                            ),
                            Style::default().fg(DARK_BROWN),
                        ),
                    ]));
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
            " [1-9]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" enter settlement  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Space]",
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

fn draw_location_screen(
    f: &mut Frame,
    app: &App,
    region_idx: usize,
    settlement_idx: usize,
    scroll: u16,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let settlement = app.sim.as_ref().and_then(|sim| {
        sim.world
            .regions
            .get(region_idx)
            .and_then(|r| r.settlements.get(settlement_idx))
    });

    let header_text = if let Some(s) = &settlement {
        format!(" Deep World — {} ({})", s.name, s.region)
    } else {
        " Deep World — Location".to_string()
    };

    let title = Paragraph::new(Line::from(Span::styled(
        header_text,
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(s) = &settlement {
        if let Some(ref sim) = app.sim {
            lines.push(Line::from(Span::styled(
                format!(" Tick {}", sim.world.tick),
                Style::default()
                    .fg(ARCHIVE_RED)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        lines.push(Line::from(Span::styled(
            format!(" Population: {}", s.population),
            Style::default().fg(WARM_BROWN),
        )));
        lines.push(Line::from(Span::styled(
            format!(" Size: {}", s.size),
            Style::default().fg(WARM_BROWN),
        )));
        if !s.description.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" {}", s.description),
                Style::default().fg(DARK_BROWN),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " People",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (pi, person) in s.people.iter().enumerate() {
            let key = if pi < 9 {
                format!("{}", pi + 1)
            } else {
                " ".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  [{}]", key),
                    Style::default()
                        .fg(ARCHIVE_RED)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        " {} — {} ({})",
                        person.name, person.profession, person.people
                    ),
                    Style::default().fg(INK),
                ),
            ]));
            let needs = &person.needs;
            let need_pairs: [(Need, &str); 5] = [
                (Need::Food, "Food"),
                (Need::Money, "Money"),
                (Need::Care, "Care"),
                (Need::Presence, "Pres"),
                (Need::Safety, "Safe"),
            ];
            for (need, label) in &need_pairs {
                let val = needs.get(*need);
                let bar = need_bar(val, 10);
                lines.push(Line::from(Span::styled(
                    format!("   {:4} {} {:.0}%", label, bar, val * 100.0),
                    Style::default().fg(need_color(val)),
                )));
            }
            lines.push(Line::from(""));
        }
    }

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [1-9]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" person  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Esc/Q]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[↑↓]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Space]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" step", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_npc_screen(
    f: &mut Frame,
    app: &App,
    region_idx: usize,
    settlement_idx: usize,
    person_idx: usize,
    scroll: u16,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let person = app.sim.as_ref().and_then(|sim| {
        sim.world
            .regions
            .get(region_idx)
            .and_then(|r| r.settlements.get(settlement_idx))
            .and_then(|s| s.people.get(person_idx))
    });

    let header_text = if let Some(p) = &person {
        format!(" Deep World — {}", p.name)
    } else {
        " Deep World — Person".to_string()
    };

    let title = Paragraph::new(Line::from(Span::styled(
        header_text,
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(p) = &person {
        lines.push(Line::from(Span::styled(
            format!(" {} — {} of {}", p.name, p.profession, p.people),
            Style::default().fg(INK).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled(
            " Identity",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("   People        {}", p.people),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Sex           {}", p.sex),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Age           {}", p.age_band),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Social Class  {}", p.social_class),
            Style::default().fg(INK),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Craft         {}", p.craft_affinity),
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
            " Needs",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
        let needs = &p.needs;
        let need_pairs: [(Need, &str); 5] = [
            (Need::Food, "Food"),
            (Need::Money, "Money"),
            (Need::Care, "Care"),
            (Need::Presence, "Pres"),
            (Need::Safety, "Safe"),
        ];
        for (need, label) in &need_pairs {
            let val = needs.get(*need);
            let bar = need_bar(val, 12);
            lines.push(Line::from(Span::styled(
                format!("   {:4} {} {:.0}%", label, bar, val * 100.0),
                Style::default().fg(need_color(val)),
            )));
        }
        lines.push(Line::from(""));

        if let Some(ref sim) = app.sim {
            let rels = sim.relationships.relationships_for(&p.id);
            if !rels.is_empty() {
                lines.push(Line::from(Span::styled(
                    " Relationships",
                    Style::default()
                        .fg(ARCHIVE_RED)
                        .add_modifier(Modifier::BOLD),
                )));
                for rel in rels {
                    let other = if rel.from_id == p.id {
                        &rel.to_id
                    } else {
                        &rel.from_id
                    };
                    let dir = if rel.from_id == p.id { "→" } else { "←" };
                    let bond = BondCategory::from_strength(rel.strength);
                    lines.push(Line::from(Span::styled(
                        format!(
                            "   {} {} {:?} str={:.0}% trust={:.0}%",
                            dir,
                            other,
                            bond,
                            rel.strength * 100.0,
                            rel.trust * 100.0
                        ),
                        Style::default().fg(DARK_BROWN),
                    )));
                }
                lines.push(Line::from(""));
            }

            let rep = sim.reputation.get(&p.id, &p.settlement);
            if rep != 0.0 {
                lines.push(Line::from(Span::styled(
                    " Reputation",
                    Style::default()
                        .fg(ARCHIVE_RED)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("   {} local reputation", rep),
                    Style::default().fg(INK),
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
            " [Esc/Q]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[↑↓]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Space]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" step", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
