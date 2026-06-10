use super::common::terrain_color;
use crate::model::Terrain;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn region_type_glyph(region_type: &str) -> char {
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

pub(crate) fn draw_overmap_screen(f: &mut Frame, app: &App, current_region: usize) {
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

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " World Map — ",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("choose region", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let (regions, cols) = if let Some(ref sim) = app.sim {
        (sim.world.regions.len(), sim.world.region_cols)
    } else {
        (0, 1)
    };

    if regions == 0 {
        let empty = Paragraph::new("No regions").style(Style::default().fg(theme.dark_brown()));
        f.render_widget(empty, chunks[1]);
        return;
    }

    let rows_count = regions.div_ceil(cols);
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    for row in 0..rows_count {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled("  ", Style::default().fg(theme.dark_brown())));
        for col in 0..cols {
            let idx = row * cols + col;
            if idx < regions {
                if let Some(ref sim) = app.sim {
                    if let Some(region) = sim.world.regions.get(idx) {
                        let glyph = region_type_glyph(&region.region_type);
                        let danger =
                            region.danger_level_biased(app.inter_people_bias.player_people);
                        let danger_glyph = danger.glyph();
                        let danger_color = match danger {
                            crate::model::DangerLevel::Safe => theme.need_color(1.0),
                            crate::model::DangerLevel::Risky => theme.warm_brown(),
                            crate::model::DangerLevel::Dangerous => theme.need_color(0.0),
                        };
                        let width_label = format!(" {:3}{}", idx + 1, danger_glyph);
                        if idx == current_region {
                            spans.push(Span::styled(
                                format!("[{}]", glyph),
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ));
                            spans.push(Span::styled(width_label, Style::default().fg(theme.ink())));
                            spans.push(Span::styled(
                                format!("{}", danger_glyph),
                                Style::default().fg(danger_color),
                            ));
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
                            spans.push(Span::styled(
                                width_label,
                                Style::default().fg(theme.dark_brown()),
                            ));
                            spans.push(Span::styled(
                                format!("{}", danger_glyph),
                                Style::default().fg(danger_color),
                            ));
                        }
                    }
                }
            } else {
                spans.push(Span::styled(
                    "     ",
                    Style::default().fg(theme.dark_brown()),
                ));
            }
        }
        let mut name_spans: Vec<Span> = Vec::new();
        name_spans.push(Span::styled("  ", Style::default().fg(theme.dark_brown())));
        for col in 0..cols {
            let idx = row * cols + col;
            if idx < regions {
                if let Some(ref sim) = app.sim {
                    if let Some(region) = sim.world.regions.get(idx) {
                        let dominant = region
                            .settlements
                            .first()
                            .and_then(|s| s.people.first())
                            .map(|p| crate::model::PeopleKind::from_name(&p.people));
                        let label = if region.name.len() > 12 {
                            format!("{:.9}..", region.name)
                        } else {
                            format!("{:<12}", region.name)
                        };
                        if idx == current_region {
                            name_spans.push(Span::styled(
                                label,
                                Style::default()
                                    .fg(theme.ink())
                                    .add_modifier(Modifier::BOLD),
                            ));
                        } else {
                            let name_color = dominant.map_or_else(
                                || theme.region_color(&region.region_type),
                                |pk| theme.people_color(pk),
                            );
                            name_spans.push(Span::styled(label, Style::default().fg(name_color)));
                        }
                    }
                }
            } else {
                name_spans.push(Span::styled(
                    "            ",
                    Style::default().fg(theme.dark_brown()),
                ));
            }
        }
        lines.push(Line::from(spans));
        lines.push(Line::from(name_spans));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(vec![
        Span::styled("  Danger: ", Style::default().fg(theme.dark_brown())),
        Span::styled("·", Style::default().fg(theme.need_color(1.0))),
        Span::styled(" safe  ", Style::default().fg(theme.dark_brown())),
        Span::styled("⚠", Style::default().fg(theme.warm_brown())),
        Span::styled(" risky  ", Style::default().fg(theme.dark_brown())),
        Span::styled("☠", Style::default().fg(theme.need_color(0.0))),
        Span::styled(" dangerous", Style::default().fg(theme.dark_brown())),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  People: ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "Metsik ",
            Style::default().fg(theme.people_color(crate::model::PeopleKind::Metsik)),
        ),
        Span::styled(
            "Arkit ",
            Style::default().fg(theme.people_color(crate::model::PeopleKind::Arkit)),
        ),
        Span::styled(
            "Väylä ",
            Style::default().fg(theme.people_color(crate::model::PeopleKind::Vayla)),
        ),
        Span::styled(
            "Laakso ",
            Style::default().fg(theme.people_color(crate::model::PeopleKind::Laakso)),
        ),
        Span::styled(
            "Sepät ",
            Style::default().fg(theme.people_color(crate::model::PeopleKind::Sepat)),
        ),
        Span::styled(
            "Ahjo",
            Style::default().fg(theme.people_color(crate::model::PeopleKind::Ahjo)),
        ),
    ]));

    let map_widget = Paragraph::new(lines).style(Style::default().bg(theme.paper()));
    f.render_widget(map_widget, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [hjkl/↑↓←→]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" navigate  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" enter map  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc/M]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
