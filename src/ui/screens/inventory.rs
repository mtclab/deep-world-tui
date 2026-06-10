use super::common::reputation_label;
use crate::model::ItemType;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_inventory_screen(f: &mut Frame, app: &App) {
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
        " Inventory",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let inv = app.player_inventory();
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    let all_items = [
        ItemType::Food,
        ItemType::Water,
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
            ItemType::Food => theme.need_color(1.0),
            ItemType::Water => Color::Rgb(0x4a, 0x8a, 0xc2),
            ItemType::Coin => Color::Rgb(0xc2, 0x9a, 0x2a),
            ItemType::Herb => theme.need_color(1.0),
            ItemType::Wood => theme.warm_brown(),
            ItemType::Stone => Color::Rgb(0x8a, 0x7a, 0x6a),
            ItemType::Cloth => Color::Rgb(0xc2, 0x9a, 0x6b),
            ItemType::Iron => Color::Rgb(0x5a, 0x5a, 0x6a),
            ItemType::Branches => theme.warm_brown(),
            ItemType::Cordage => Color::Rgb(0x8a, 0x6a, 0x4a),
            ItemType::Tinder => Color::Rgb(0xaa, 0x8a, 0x5a),
            ItemType::Nails => Color::Rgb(0x5a, 0x5a, 0x6a),
            ItemType::Thatch => Color::Rgb(0xaa, 0x9a, 0x4a),
            ItemType::Glass => Color::Rgb(0x8a, 0xc2, 0xca),
        };
        let dur = inv.durability(*item);
        let quality = crate::model::QualityTier::from_durability(dur);
        let dur_bar = if count > 0 && dur < 1.0 {
            let dur_color = if dur > 0.5 {
                Color::Green
            } else if dur > 0.25 {
                Color::Yellow
            } else {
                Color::Red
            };
            let filled = (dur * 10.0).round() as usize;
            let quality_str = format!(" {}", quality.label());
            let dur_str = format!(
                " [{}{}]{quality_str}",
                "█".repeat(filled),
                "░".repeat(10 - filled)
            );
            Span::styled(dur_str, Style::default().fg(dur_color))
        } else if count > 0 && quality != crate::model::QualityTier::Sturdy {
            let quality_str = format!(" ({})", quality.label());
            Span::styled(quality_str, Style::default().fg(Color::Cyan))
        } else {
            Span::raw("")
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:6}", item.name()),
                Style::default().fg(theme.ink()),
            ),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(
                format!(" x{}", count),
                Style::default().fg(theme.dark_brown()),
            ),
            dur_bar,
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Identity",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )));
    let people_label = app.inter_people_bias.player_people.label();
    lines.push(Line::from(vec![
        Span::styled("  People: ", Style::default().fg(theme.dark_brown())),
        Span::styled(people_label, Style::default().fg(theme.ink())),
    ]));
    let title = app
        .god_affinity
        .people_title(app.inter_people_bias.player_people);
    if !title.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Known as: ", Style::default().fg(theme.dark_brown())),
            Span::styled(
                title,
                Style::default()
                    .fg(theme.warm_brown())
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    }
    if let Some(ref sim) = app.sim {
        if let Some(pos) = app.player_pos {
            if let Some(region) = sim.world.regions.get(pos.region_idx) {
                if let Some(settlement) = region.settlements.first() {
                    if let Some(dominant) = settlement.people.first() {
                        let npc_people = crate::model::PeopleKind::from_name(&dominant.people);
                        let bias = app.inter_people_bias.player_people.bias_toward(npc_people);
                        let rep_in_settle = app
                            .player_start
                            .as_ref()
                            .map(|ps| sim.reputation.get(&ps.person.id, &settlement.id))
                            .unwrap_or(0.5);
                        let stance_color = if bias > 0.05 {
                            theme.need_color(1.0)
                        } else if bias < -0.05 {
                            theme.need_color(0.0)
                        } else {
                            theme.dark_brown()
                        };
                        let here_line = if app.inter_people_bias.player_people == npc_people {
                            format!("  Here:   {} settlement — home", npc_people.label())
                        } else {
                            let adverb =
                                crate::sim::signals::settlement_welcome_adverb(rep_in_settle);
                            if adverb.is_empty() {
                                format!(
                                    "  Here:   {} settlement — strangers move {adverb}",
                                    npc_people.label()
                                )
                            } else {
                                format!(
                                    "  Here:   {} settlement — they greet you {adverb}",
                                    npc_people.label()
                                )
                            }
                        };
                        lines.push(Line::from(Span::styled(
                            here_line,
                            Style::default().fg(stance_color),
                        )));
                    }
                }
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Vitals",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Hunger: ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            app.vitals.hunger_label(),
            Style::default().fg(theme.need_color(app.vitals.hunger)),
        ),
        Span::styled("  Energy: ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            app.vitals.energy_label(),
            Style::default().fg(theme.need_color(app.vitals.energy)),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Gods",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )));
    let gods = [
        (crate::model::GodName::Oltzed, app.god_affinity.oltzed),
        (crate::model::GodName::Keuru, app.god_affinity.keuru),
        (crate::model::GodName::Sampsa, app.god_affinity.sampsa),
        (crate::model::GodName::Masa, app.god_affinity.masa),
        (crate::model::GodName::Kukri, app.god_affinity.kukri),
    ];
    for (god, val) in &gods {
        let label = reputation_label(*val);
        let color = if *val > 0.0 {
            theme.need_color(1.0)
        } else if *val < 0.0 {
            theme.need_color(0.0)
        } else {
            theme.dark_brown()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", god.glyph()), Style::default().fg(color)),
            Span::styled(
                format!("{:<8}", god.label()),
                Style::default().fg(theme.ink()),
            ),
            Span::styled(format!(" {}", label), Style::default().fg(color)),
        ]));
    }

    let para = Paragraph::new(lines).style(Style::default().bg(theme.paper()));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc/i]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
