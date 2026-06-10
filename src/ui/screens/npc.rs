use super::common::need_bar;
use crate::model::Need;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use crate::voice::Situation;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub(crate) fn draw_npc_screen(
    f: &mut Frame,
    app: &App,
    region_idx: usize,
    settlement_idx: usize,
    person_idx: usize,
    scroll: u16,
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
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(p) = &person {
        lines.push(Line::from(Span::styled(
            format!(" {} — {} of {}", p.name, p.profession, p.people),
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled(
            " Identity",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("   People        {}", p.people),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Sex           {}", p.sex),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Age           {}", p.age_band),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Social Class  {}", p.social_class),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Craft         {}", p.craft_affinity),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Personality    {}", p.personality.join(", ")),
            Style::default().fg(theme.ink()),
        )));
        if p.has_spouse {
            lines.push(Line::from(Span::styled(
                "   Household     married",
                Style::default().fg(theme.ink()),
            )));
        }
        if p.children_count > 0 {
            lines.push(Line::from(Span::styled(
                format!("   Children      {}", p.children_count),
                Style::default().fg(theme.ink()),
            )));
        }
        lines.push(Line::from(""));

        if let (Some(ref sim), Some(ref ps)) = (&app.sim, &app.player_start) {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if let Some(settlement) = region.settlements.get(settlement_idx) {
                    let rep = sim.reputation.get(&ps.person.id, &settlement.id);
                    let hash = crate::sim::signals::hash_str(&p.id);
                    let gesture = crate::sim::signals::body_language(rep, hash);
                    let engagement = app.npc_will_engage(&p.people, &p.id);
                    let narration = crate::sim::signals::engagement_narration(engagement, &p.name);
                    lines.push(Line::from(Span::styled(
                        format!(" {narration}"),
                        Style::default()
                            .fg(theme.archive_red())
                            .add_modifier(Modifier::ITALIC),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!(" They {gesture}."),
                        Style::default()
                            .fg(theme.dark_ink())
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            }
        }
        lines.push(Line::from(""));

        if let Some(ref sim) = app.sim {
            if let Some(ref ps) = app.player_start {
                let bond = sim.relationships.get(&ps.person.id, &p.id);
                let (bond_str, bond_color) = if let Some(rel) = bond {
                    let desc = crate::sim::relationships::bond_descriptor(rel.strength, &p.id);
                    (format!("   {desc}"), theme.need_color(rel.strength))
                } else {
                    ("   They keep their distance.".into(), theme.dark_brown())
                };
                lines.push(Line::from(Span::styled(
                    " Relationship",
                    Style::default()
                        .fg(theme.archive_red())
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
                .fg(theme.archive_red())
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
                Style::default().fg(theme.need_color(val)),
            )));
        }
        lines.push(Line::from(""));

        if let Some(ref sim) = app.sim {
            let rels = sim.relationships.relationships_for(&p.id);
            if !rels.is_empty() {
                lines.push(Line::from(Span::styled(
                    " Relationships",
                    Style::default()
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                )));
                for rel in rels {
                    let other = if rel.from_id == p.id {
                        &rel.to_id
                    } else {
                        &rel.from_id
                    };
                    let dir = if rel.from_id == p.id { "→" } else { "←" };
                    let descriptor =
                        crate::sim::relationships::bond_descriptor(rel.strength, other);
                    let regard = crate::sim::relationships::bond_descriptor(rel.trust, other);
                    lines.push(Line::from(Span::styled(
                        format!("   {} {} — {}. {}", dir, other, descriptor, regard),
                        Style::default().fg(theme.dark_brown()),
                    )));
                }
                lines.push(Line::from(""));
            }

            let rep = sim.reputation.get(&p.id, &p.settlement);
            if rep != 0.0 {
                lines.push(Line::from(Span::styled(
                    " Reputation",
                    Style::default()
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("   {} local reputation", rep),
                    Style::default().fg(theme.ink()),
                )));
            }

            let vline = crate::voice::voice_line_situation(p, Situation::Greeting);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Voice",
                Style::default()
                    .fg(theme.archive_red())
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                vline,
                Style::default().fg(theme.dark_brown()),
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
        Span::styled(" scroll  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Space]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" step", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
