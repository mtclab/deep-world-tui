use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::model::{craft_recipes, ItemType, Need, Terrain};
use crate::sim::relationships::BondCategory;
use crate::ui::app::{App, Screen};
use crate::voice::Situation;

const ARCHIVE_RED: Color = Color::Rgb(0x7a, 0x2e, 0x1d);
const INK: Color = Color::Rgb(0x3a, 0x2a, 0x1a);
const DARK_INK: Color = Color::Rgb(0x6a, 0x5a, 0x4a);
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
        Screen::WorldAlerts { scroll } => draw_alerts_screen(f, app, scroll),
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
        Screen::Talk {
            region_idx,
            settlement_idx,
            person_idx,
            scroll,
        } => {
            draw_talk_screen(f, app, region_idx, settlement_idx, person_idx, scroll);
        }
        Screen::Journal { scroll } => {
            draw_journal_screen(f, app, scroll);
        }
        Screen::Map { region_idx, px, py } => {
            draw_map_screen(f, app, region_idx, px, py);
        }
        Screen::Overmap { region_idx } => {
            draw_overmap_screen(f, app, region_idx);
        }
        Screen::Inventory => {
            draw_inventory_screen(f, app);
        }
        Screen::Craft { scroll } => {
            draw_craft_screen(f, app, scroll);
        }
        Screen::Market { scroll, .. } => {
            draw_market_screen(f, app, scroll);
        }
        Screen::Encounter => {
            draw_encounter_screen(f, app);
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

    if let Some(ref msg) = app.status_msg {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(" {}", msg),
            Style::default().fg(WARM_BROWN),
        )));
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
            "[J]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" journal  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[S]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" save  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[L]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" load  ", Style::default().fg(DARK_BROWN)),
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
            if let Some(ref ps) = app.player_start {
                let rep = sim.reputation.get(&ps.person.id, &s.id);
                let rep_label = if rep >= 0.8 {
                    "trusted"
                } else if rep >= 0.6 {
                    "liked"
                } else if rep >= 0.4 {
                    "neutral"
                } else if rep >= 0.2 {
                    "suspect"
                } else {
                    "shunned"
                };
                lines.push(Line::from(Span::styled(
                    format!(" Your reputation: {:.0}% ({})", rep * 100.0, rep_label),
                    Style::default().fg(WARM_BROWN),
                )));
            }
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
            "[m]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" market  ", Style::default().fg(DARK_BROWN)),
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

        if let Some(ref sim) = app.sim {
            if let Some(ref ps) = app.player_start {
                let bond = sim.relationships.get(&ps.person.id, &p.id);
                let (bond_str, bond_color) = if let Some(rel) = bond {
                    let cat = crate::sim::relationships::BondCategory::from_strength(rel.strength);
                    let label = match cat {
                        crate::sim::relationships::BondCategory::Bonded => "bonded",
                        crate::sim::relationships::BondCategory::Kin => "kin",
                        crate::sim::relationships::BondCategory::Friend => "friend",
                        crate::sim::relationships::BondCategory::Acquaintance => "acquaintance",
                        crate::sim::relationships::BondCategory::Stranger => "stranger",
                    };
                    (
                        format!("   Bond     {:.0}% {}", rel.strength * 100.0, label),
                        need_color(rel.strength),
                    )
                } else {
                    ("   Bond     stranger".into(), DARK_BROWN)
                };
                lines.push(Line::from(Span::styled(
                    " Relationship",
                    Style::default()
                        .fg(ARCHIVE_RED)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    bond_str,
                    Style::default().fg(bond_color),
                )));
            }
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

            let vline = crate::voice::voice_line_situation(p, Situation::Greeting);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Voice",
                Style::default()
                    .fg(ARCHIVE_RED)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                vline,
                Style::default().fg(DARK_BROWN),
            )));
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

fn draw_journal_screen(f: &mut Frame, app: &App, scroll: u16) {
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
        Span::styled(" — Journal", Style::default().fg(WARM_BROWN)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(ref sim) = app.sim {
        if sim.journal.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " The Archive holds no records yet.",
                Style::default().fg(WARM_BROWN),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Time passes. Events will be recorded here.",
                Style::default().fg(DARK_BROWN),
            )));
        } else {
            for entry in sim.journal.iter().rev() {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" [{}] ", entry.tick),
                        Style::default()
                            .fg(ARCHIVE_RED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(entry.text.clone(), Style::default().fg(INK)),
                ]));
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
        Span::styled(" scroll", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_talk_screen(
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

    let header = if let Some(p) = &person {
        format!(" Deep World — Talking to {}", p.name)
    } else {
        " Deep World — Talk".to_string()
    };

    let title = Paragraph::new(Line::from(Span::styled(
        header,
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(p) = &person {
        let situations = [
            (Situation::Greeting, "Greeting"),
            (Situation::Trade, "Trade"),
            (Situation::NeedDire, "In Need"),
            (Situation::NeedFine, "Well"),
            (Situation::Farewell, "Farewell"),
            (Situation::Gossip, "Gossip"),
        ];

        lines.push(Line::from(Span::styled(
            format!(" {} speaks:", p.name),
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (sit, label) in &situations {
            let vline = crate::voice::voice_line_situation(p, *sit);
            lines.push(Line::from(Span::styled(
                format!(" [{}]", label),
                Style::default().fg(WARM_BROWN).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("   {}", vline),
                Style::default().fg(INK),
            )));
            lines.push(Line::from(""));
        }

        let low_food = p.needs.get(Need::Food) < 0.5;
        let low_money = p.needs.get(Need::Money) < 0.5;

        lines.push(Line::from(Span::styled(
            " Actions",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        if low_food {
            lines.push(Line::from(Span::styled(
                "   Food is low. Share a meal?",
                Style::default().fg(NEED_HIGH),
            )));
        }
        if low_money {
            lines.push(Line::from(Span::styled(
                "   Coin is thin. Offer payment?",
                Style::default().fg(NEED_HIGH),
            )));
        }
        if low_money {
            lines.push(Line::from(Span::styled(
                "   Coin is thin. Offer payment?",
                Style::default().fg(NEED_HIGH),
            )));
        }

        if let Some(ref msg) = app.status_msg {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" {}", msg),
                Style::default().fg(WARM_BROWN),
            )));
        }
    }

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [F]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" give food  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[C]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" give coin  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Esc/Q]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_alerts_screen(f: &mut Frame, app: &App, scroll: u16) {
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
        Span::styled(" — Need Alerts", Style::default().fg(WARM_BROWN)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let critical = app.critical_need_people();
    let mut lines: Vec<Line> = Vec::new();

    if critical.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " No critical needs. The Archive rests.",
            Style::default().fg(WARM_BROWN),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!(" {} people in dire need", critical.len()),
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        for (name, settlement, profession, need, val) in &critical {
            let bar = need_bar(*val, 8);
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", name), Style::default().fg(INK)),
                Span::styled(
                    format!("({}) ", settlement),
                    Style::default().fg(DARK_BROWN),
                ),
                Span::styled(format!("{}, ", profession), Style::default().fg(DARK_BROWN)),
                Span::styled(
                    format!("{:?} ", need),
                    Style::default()
                        .fg(need_color(*val))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(bar, Style::default().fg(need_color(*val))),
                Span::styled(
                    format!(" {:.0}%", val * 100.0),
                    Style::default().fg(need_color(*val)),
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
        Span::styled(" scroll", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn terrain_color(terrain: Terrain) -> Color {
    match terrain {
        Terrain::Grass => Color::Rgb(0x6b, 0x8e, 0x4a),
        Terrain::Forest => Color::Rgb(0x3a, 0x5a, 0x2a),
        Terrain::Water => Color::Rgb(0x4a, 0x7a, 0x9e),
        Terrain::Mountain => Color::Rgb(0x8a, 0x7a, 0x6a),
        Terrain::Road => Color::Rgb(0x9a, 0x8a, 0x6a),
        Terrain::Settlement => Color::Rgb(0x7a, 0x2e, 0x1d),
        Terrain::Farmland => Color::Rgb(0x8a, 0x9a, 0x4a),
        Terrain::Sand => Color::Rgb(0xc2, 0x9a, 0x6b),
        Terrain::Swamp => Color::Rgb(0x5a, 0x6a, 0x3a),
    }
}

fn dim_color(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
        other => other,
    }
}

fn terrain_color_at(terrain: Terrain, dark: bool) -> Color {
    let c = terrain_color(terrain);
    if dark {
        dim_color(c)
    } else {
        c
    }
}

fn draw_map_screen(f: &mut Frame, app: &App, region_idx: usize, px: usize, py: usize) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let region_name = app
        .sim
        .as_ref()
        .and_then(|sim| sim.world.regions.get(region_idx).map(|r| r.name.clone()))
        .unwrap_or_else(|| "Unknown".into());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Map — ",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &region_name,
            Style::default().fg(INK).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  {}  {} {}",
                app.clock_str(),
                app.vitals.hunger_label(),
                app.vitals.energy_label()
            ),
            Style::default().fg(DARK_INK),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let map_area = chunks[1];
    let view_w = map_area.width as usize;
    let view_h = map_area.height as usize;

    let (map_w, map_h, tiles) = if let Some(ref sim) = app.sim {
        if let Some(region) = sim.world.regions.get(region_idx) {
            (
                region.terrain.width,
                region.terrain.height,
                region.terrain.tiles.clone(),
            )
        } else {
            (0, 0, Vec::new())
        }
    } else {
        (0, 0, Vec::new())
    };

    if map_w == 0 || map_h == 0 {
        let empty = Paragraph::new("No terrain data").style(Style::default().fg(DARK_BROWN));
        f.render_widget(empty, chunks[1]);
        return;
    }

    let half_w = view_w / 2;
    let half_h = view_h / 2;
    let cam_x = px.saturating_sub(half_w);
    let cam_y = py.saturating_sub(half_h);

    let dark = app.clock.time_of_day().is_dark();

    let mut lines: Vec<Line> = Vec::new();
    for vy in 0..view_h {
        let my = cam_y + vy;
        let mut spans: Vec<Span> = Vec::new();
        if my < map_h {
            for vx in 0..view_w {
                let mx = cam_x + vx;
                if mx < map_w {
                    if mx == px && my == py {
                        spans.push(Span::styled(
                            "@",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else if let Some(terrain) = tiles.get(my * map_w + mx) {
                        let c = terrain.glyph();
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().fg(terrain_color_at(*terrain, dark)),
                        ));
                    } else {
                        spans.push(Span::styled(" ", Style::default().fg(DARK_BROWN)));
                    }
                }
            }
        }
        lines.push(Line::from(spans));
    }

    let map_widget = Paragraph::new(lines).style(Style::default().bg(PAPER));
    f.render_widget(map_widget, map_area);

    let legend_lines = vec![
        Line::from(vec![
            Span::styled("░", Style::default().fg(terrain_color(Terrain::Grass))),
            Span::styled("Grass ", Style::default().fg(DARK_BROWN)),
            Span::styled("▓", Style::default().fg(terrain_color(Terrain::Forest))),
            Span::styled("Forest ", Style::default().fg(DARK_BROWN)),
            Span::styled("≈", Style::default().fg(terrain_color(Terrain::Water))),
            Span::styled("Water ", Style::default().fg(DARK_BROWN)),
        ]),
        Line::from(vec![
            Span::styled("▲", Style::default().fg(terrain_color(Terrain::Mountain))),
            Span::styled("Mtn ", Style::default().fg(DARK_BROWN)),
            Span::styled("·", Style::default().fg(terrain_color(Terrain::Road))),
            Span::styled("Road ", Style::default().fg(DARK_BROWN)),
            Span::styled("█", Style::default().fg(terrain_color(Terrain::Settlement))),
            Span::styled("Town ", Style::default().fg(DARK_BROWN)),
        ]),
        Line::from(vec![
            Span::styled("▒", Style::default().fg(terrain_color(Terrain::Farmland))),
            Span::styled("Farm ", Style::default().fg(DARK_BROWN)),
            Span::styled("~", Style::default().fg(terrain_color(Terrain::Swamp))),
            Span::styled("Swmp ", Style::default().fg(DARK_BROWN)),
            Span::styled(
                "@",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("You", Style::default().fg(DARK_BROWN)),
        ]),
    ];
    let legend = Paragraph::new(legend_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(PAPER)),
    );
    let legend_rect = ratatui::layout::Rect {
        x: map_area.x + map_area.width.saturating_sub(22),
        y: map_area.y,
        width: 22,
        height: 5,
    };
    f.render_widget(legend, legend_rect);

    let terrain_name = if let Some(t) = tiles.get(py * map_w + px) {
        format!("{:?}", t)
    } else {
        "??".into()
    };

    let coord = format!(" ({},{}) {}", px, py, terrain_name);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [hjkl/↑↓←→]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" move  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[g]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" gather  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[r]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" rest  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[i]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" inv  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[c]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" craft  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" enter  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(DARK_BROWN)),
        Span::styled(coord, Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn region_type_glyph(region_type: &str) -> char {
    match region_type {
        "river_valley" => '~',
        "coast" => '≈',
        "forest" => '♣',
        "upland" => '▲',
        "steppe" => '░',
        "delta" => '¤',
        _ => '?',
    }
}

fn draw_overmap_screen(f: &mut Frame, app: &App, current_region: usize) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " World Map — ",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("choose region", Style::default().fg(WARM_BROWN)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let (regions, cols) = if let Some(ref sim) = app.sim {
        (sim.world.regions.len(), sim.world.region_cols)
    } else {
        (0, 1)
    };

    if regions == 0 {
        let empty = Paragraph::new("No regions").style(Style::default().fg(DARK_BROWN));
        f.render_widget(empty, chunks[1]);
        return;
    }

    let rows_count = regions.div_ceil(cols);
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    for row in 0..rows_count {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled("  ", Style::default().fg(DARK_BROWN)));
        for col in 0..cols {
            let idx = row * cols + col;
            if idx < regions {
                if let Some(ref sim) = app.sim {
                    if let Some(region) = sim.world.regions.get(idx) {
                        let glyph = region_type_glyph(&region.region_type);
                        let width_label = format!(" {:3} ", idx + 1);
                        if idx == current_region {
                            spans.push(Span::styled(
                                format!("[{}]", glyph),
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ));
                            spans.push(Span::styled(width_label, Style::default().fg(INK)));
                        } else {
                            spans.push(Span::styled(
                                format!(" {} ", glyph),
                                Style::default().fg(terrain_color(
                                    match region.region_type.as_str() {
                                        "river_valley" => Terrain::Water,
                                        "coast" => Terrain::Sand,
                                        "forest" => Terrain::Forest,
                                        "upland" => Terrain::Mountain,
                                        "steppe" => Terrain::Grass,
                                        "delta" => Terrain::Swamp,
                                        _ => Terrain::Grass,
                                    },
                                )),
                            ));
                            spans.push(Span::styled(width_label, Style::default().fg(DARK_BROWN)));
                        }
                    }
                }
            } else {
                spans.push(Span::styled("     ", Style::default().fg(DARK_BROWN)));
            }
        }
        let mut name_spans: Vec<Span> = Vec::new();
        name_spans.push(Span::styled("  ", Style::default().fg(DARK_BROWN)));
        for col in 0..cols {
            let idx = row * cols + col;
            if idx < regions {
                if let Some(ref sim) = app.sim {
                    if let Some(region) = sim.world.regions.get(idx) {
                        let label = if region.name.len() > 12 {
                            format!("{:.9}..", region.name)
                        } else {
                            format!("{:<12}", region.name)
                        };
                        if idx == current_region {
                            name_spans.push(Span::styled(
                                label,
                                Style::default().fg(INK).add_modifier(Modifier::BOLD),
                            ));
                        } else {
                            name_spans.push(Span::styled(label, Style::default().fg(DARK_BROWN)));
                        }
                    }
                }
            } else {
                name_spans.push(Span::styled(
                    "            ",
                    Style::default().fg(DARK_BROWN),
                ));
            }
        }
        lines.push(Line::from(spans));
        lines.push(Line::from(name_spans));
        lines.push(Line::from(""));
    }

    let map_widget = Paragraph::new(lines).style(Style::default().bg(PAPER));
    f.render_widget(map_widget, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [hjkl/↑↓←→]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" navigate  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" enter map  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Esc/M]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_inventory_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " Inventory",
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let inv = app.player_inventory();
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    let all_items = [
        ItemType::Food,
        ItemType::Coin,
        ItemType::Herb,
        ItemType::Wood,
        ItemType::Stone,
        ItemType::Cloth,
        ItemType::Iron,
    ];
    for item in &all_items {
        let count = inv.get(*item);
        let bar = if count > 0 {
            format!("{:<8}", "█".repeat(count.min(8) as usize))
        } else {
            "        ".into()
        };
        let color = match item {
            ItemType::Food => NEED_LOW,
            ItemType::Coin => Color::Rgb(0xc2, 0x9a, 0x2a),
            ItemType::Herb => NEED_LOW,
            ItemType::Wood => WARM_BROWN,
            ItemType::Stone => Color::Rgb(0x8a, 0x7a, 0x6a),
            ItemType::Cloth => Color::Rgb(0xc2, 0x9a, 0x6b),
            ItemType::Iron => Color::Rgb(0x5a, 0x5a, 0x6a),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:6}", item.name()), Style::default().fg(INK)),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(format!(" x{}", count), Style::default().fg(DARK_BROWN)),
        ]));
    }

    let para = Paragraph::new(lines).style(Style::default().bg(PAPER));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc/i]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_craft_screen(f: &mut Frame, app: &App, scroll: u16) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " Craft",
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let inv = app.player_inventory();
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    for (i, recipe) in craft_recipes().iter().enumerate() {
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
        let can_color = if has_all { NEED_LOW } else { DARK_BROWN };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" [{}] ", key),
                Style::default()
                    .fg(ARCHIVE_RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<10}", recipe.name),
                Style::default().fg(can_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("({})", inputs), Style::default().fg(DARK_BROWN)),
            Span::styled(
                format!(" -> {}x{}", recipe.output_count, recipe.output.name()),
                Style::default().fg(can_color),
            ),
        ]));
    }

    let para = Paragraph::new(lines)
        .style(Style::default().bg(PAPER))
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [1-9]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" craft  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_market_screen(f: &mut Frame, app: &App, scroll: u16) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " Market",
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let inv = app.player_inventory();
    let coins = inv.get(crate::model::ItemType::Coin);
    let items = crate::model::ItemType::tradeable_items();
    let buy_keys = ['1', '2', '3', '4', '5', '6'];
    let sell_keys = ['a', 'b', 'c', 'd', 'e', 'f'];
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(" You have {} coins", coins),
        Style::default().fg(WARM_BROWN),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Buy",
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )));
    for (i, &item) in items.iter().enumerate() {
        let price = item.base_price();
        let can = coins >= price;
        let color = if can { NEED_LOW } else { DARK_BROWN };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" [{}] ", buy_keys[i]),
                Style::default()
                    .fg(ARCHIVE_RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<8}", item.name()), Style::default().fg(color)),
            Span::styled(format!(" {} coins", price), Style::default().fg(DARK_BROWN)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Sell",
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )));
    for (i, &item) in items.iter().enumerate() {
        let price = item.base_price();
        let have = inv.get(item);
        let color = if have > 0 { NEED_LOW } else { DARK_BROWN };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" [{}] ", sell_keys[i]),
                Style::default()
                    .fg(ARCHIVE_RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<8}", item.name()), Style::default().fg(color)),
            Span::styled(
                format!(" (have {}) -> {} coins", have, price),
                Style::default().fg(DARK_BROWN),
            ),
        ]));
    }

    let para = Paragraph::new(lines)
        .style(Style::default().bg(PAPER))
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [1-6]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" buy  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[a-f]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" sell  ", Style::default().fg(DARK_BROWN)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_encounter_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " Encounter!",
        Style::default()
            .fg(ARCHIVE_RED)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    if let Some(enc) = app.encounter {
        lines.push(Line::from(Span::styled(
            enc.kind.description(),
            Style::default().fg(INK).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        let kind_str = match enc.kind {
            crate::model::EncounterKind::Wildlife => "Wildlife",
            crate::model::EncounterKind::Bandit => "Bandit",
            crate::model::EncounterKind::Traveler => "Traveler",
            crate::model::EncounterKind::Storm => "Storm",
        };
        lines.push(Line::from(Span::styled(
            format!("  Kind: {}", kind_str),
            Style::default().fg(WARM_BROWN),
        )));
        if enc.kind.is_hostile() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  You lost some energy fleeing!",
                Style::default().fg(NEED_HIGH),
            )));
        } else if matches!(enc.kind, crate::model::EncounterKind::Traveler) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  They share news of the road ahead.",
                Style::default().fg(NEED_LOW),
            )));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  Hunger: {}  Energy: {}",
            app.vitals.hunger_label(),
            app.vitals.energy_label()
        ),
        Style::default().fg(DARK_BROWN),
    )));

    let para = Paragraph::new(lines).style(Style::default().bg(PAPER));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Enter/Esc]",
            Style::default()
                .fg(ARCHIVE_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" continue", Style::default().fg(DARK_BROWN)),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
