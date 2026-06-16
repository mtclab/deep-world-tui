use super::common::{build_npc_map, terrain_color, terrain_color_at, MapViewport};
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
use std::collections::HashMap;

pub(crate) fn draw_map_screen(f: &mut Frame, app: &App, region_idx: usize) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
    };
    let px = app.player_pos.map(|p| p.px).unwrap_or(20);
    let py = app.player_pos.map(|p| p.py).unwrap_or(10);
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(4),
    ])
    .split(f.area());

    let region_name = app
        .sim
        .as_ref()
        .and_then(|sim| sim.world.regions.get(region_idx).map(|r| r.name.clone()))
        .unwrap_or_else(|| "Unknown".into());

    let weather_span = if let (Some(ref sim), Some(pos)) = (&app.sim, app.player_pos) {
        if let Some(terrain) = sim
            .world
            .regions
            .get(pos.region_idx)
            .and_then(|r| r.terrain.get(pos.px, pos.py))
        {
            let _ = terrain;
            let w = sim
                .world
                .regions
                .get(pos.region_idx)
                .map(|r| r.weather)
                .unwrap_or(crate::model::Weather::Clear);
            Some(Span::styled(
                format!(" {} ", w.glyph()),
                Style::default().fg(theme.warm_brown()),
            ))
        } else {
            None
        }
    } else {
        None
    };

    let mut header_spans = vec![
        Span::styled(
            " Map — ",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &region_name,
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if let Some(ws) = weather_span {
        header_spans.push(ws);
    }
    header_spans.push(Span::styled(
        format!(
            "{}  {} {}",
            app.clock_str(),
            app.vitals.hunger_label(),
            app.vitals.energy_label()
        ),
        Style::default().fg(theme.dark_ink()),
    ));

    let header =
        Paragraph::new(Line::from(header_spans)).block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let map_area = chunks[1];
    let view_w = map_area.width as usize;
    let view_h = map_area.height as usize;

    let (map_w, map_h, tiles) = if let Some(ref sim) = app.sim {
        if let Some(region) = sim.world.regions.get(region_idx) {
            (
                region.terrain.width,
                region.terrain.height,
                &region.terrain.tiles,
            )
        } else {
            (0, 0, &Vec::new())
        }
    } else {
        (0, 0, &Vec::new())
    };

    if map_w == 0 || map_h == 0 {
        let empty =
            Paragraph::new("No terrain data").style(Style::default().fg(theme.dark_brown()));
        f.render_widget(empty, chunks[1]);
        return;
    }

    let half_w = view_w / 2;
    let half_h = view_h / 2;
    let cam_x = px.saturating_sub(half_w);
    let cam_y = py.saturating_sub(half_h);

    let dark = app.clock.time_of_day().is_dark();

    let vp = MapViewport {
        px,
        py,
        view_w,
        view_h,
        cam_x,
        cam_y,
    };
    let npc_map = build_npc_map(app, region_idx, &vp);

    let structure_map: HashMap<(usize, usize), char> = app
        .sim
        .as_ref()
        .and_then(|sim| sim.world.regions.get(region_idx))
        .map(|r| {
            r.structures
                .iter()
                .map(|s| ((s.x as usize, s.y as usize), s.kind.glyph()))
                .collect()
        })
        .unwrap_or_default();

    let build_map: HashMap<(usize, usize), f64> = app
        .sim
        .as_ref()
        .map(|sim| {
            sim.build_sites
                .iter()
                .filter(|s| s.region_idx == region_idx)
                .map(|s| ((s.x as usize, s.y as usize), s.progress_fraction(s.kind)))
                .collect()
        })
        .unwrap_or_default();

    // A sign over each service building's door — the tavern, the temple, the
    // forge told apart from the street, not by knocking on every door (#458).
    let service_door_map: HashMap<(usize, usize), crate::model::SettlementService> = app
        .sim
        .as_ref()
        .and_then(|sim| sim.world.regions.get(region_idx))
        .map(|r| {
            let mut m = HashMap::new();
            for s in &r.settlements {
                let buildings = crate::gen::town::town_buildings(s);
                for (b, svc) in buildings.iter().zip(s.services.iter()) {
                    m.insert(b.door, *svc);
                }
            }
            m
        })
        .unwrap_or_default();

    // Furnishings inside each building — a table, a chest, a bed-pallet — so a
    // building reads as a lived-in room, not an empty box (#458 interiors).
    let furnishing_map: HashMap<(usize, usize), char> = app
        .sim
        .as_ref()
        .and_then(|sim| sim.world.regions.get(region_idx))
        .map(|r| {
            let mut m = HashMap::new();
            for s in &r.settlements {
                let seed = crate::gen::building::town_seed(s.map_x, s.map_y);
                for b in crate::gen::town::town_buildings(s) {
                    for (fx, fy, g) in crate::gen::building::building_furnishings(&b, seed) {
                        m.insert((fx, fy), g);
                    }
                }
            }
            m
        })
        .unwrap_or_default();

    let memorial_set: std::collections::HashSet<(usize, usize)> = app
        .sim
        .as_ref()
        .map(|sim| {
            sim.memorials
                .iter()
                .filter(|m| m.region_idx == region_idx)
                .map(|m| (m.x as usize, m.y as usize))
                .collect()
        })
        .unwrap_or_default();

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
                    } else if let Some(npc) = npc_map.get(&(mx, my)) {
                        spans.push(Span::styled(
                            npc.glyph.to_string(),
                            Style::default().fg(npc.color),
                        ));
                    } else if let Some(&glyph) = structure_map.get(&(mx, my)) {
                        spans.push(Span::styled(
                            glyph.to_string(),
                            Style::default().fg(theme.archive_red()),
                        ));
                    } else if let Some(&pct) = build_map.get(&(mx, my)) {
                        let pct_char = if pct < 0.33 {
                            '░'
                        } else if pct < 0.66 {
                            '▒'
                        } else {
                            '▓'
                        };
                        spans.push(Span::styled(
                            pct_char.to_string(),
                            Style::default().fg(theme.archive_red()),
                        ));
                    } else if let Some(terrain) = tiles.get(my * map_w + mx) {
                        let is_explored = app
                            .explored
                            .get(region_idx)
                            .map(|e| e.is_explored(mx, my))
                            .unwrap_or(true);
                        let is_memorial = is_explored && memorial_set.contains(&(mx, my));
                        if !is_explored {
                            spans.push(Span::styled(
                                "·".to_string(),
                                Style::default().fg(theme.dark_brown()),
                            ));
                        } else if is_memorial {
                            spans.push(Span::styled(
                                crate::model::memorial::Memorial::glyph().to_string(),
                                Style::default().fg(theme.archive_red()),
                            ));
                        } else if let Some(svc) = (*terrain == Terrain::Door)
                            .then(|| service_door_map.get(&(mx, my)))
                            .flatten()
                        {
                            spans.push(Span::styled(
                                svc.map_sign().to_string(),
                                Style::default()
                                    .fg(theme.archive_red())
                                    .add_modifier(Modifier::BOLD),
                            ));
                        } else if let Some(&furn) = (*terrain == Terrain::Floor)
                            .then(|| furnishing_map.get(&(mx, my)))
                            .flatten()
                        {
                            // A lived-in room: furniture in muted ink on the floor.
                            spans.push(Span::styled(
                                furn.to_string(),
                                Style::default().fg(theme.dark_brown()),
                            ));
                        } else {
                            let c = terrain.glyph();
                            spans.push(Span::styled(
                                c.to_string(),
                                Style::default().fg(terrain_color_at(*terrain, dark)),
                            ));
                        }
                    } else {
                        spans.push(Span::styled(" ", Style::default().fg(theme.dark_brown())));
                    }
                }
            }
        }
        lines.push(Line::from(spans));
    }

    let map_widget = Paragraph::new(lines).style(Style::default().bg(theme.paper()));
    f.render_widget(map_widget, map_area);

    let legend_lines = vec![
        Line::from(vec![
            Span::styled(",", Style::default().fg(terrain_color(Terrain::Grass))),
            Span::styled("Grass ", Style::default().fg(theme.dark_brown())),
            Span::styled("▓", Style::default().fg(terrain_color(Terrain::Forest))),
            Span::styled("Forest ", Style::default().fg(theme.dark_brown())),
            Span::styled("≈", Style::default().fg(terrain_color(Terrain::Water))),
            Span::styled("Water ", Style::default().fg(theme.dark_brown())),
        ]),
        Line::from(vec![
            Span::styled("▲", Style::default().fg(terrain_color(Terrain::Mountain))),
            Span::styled("Mtn ", Style::default().fg(theme.dark_brown())),
            Span::styled("·", Style::default().fg(terrain_color(Terrain::Road))),
            Span::styled("Road ", Style::default().fg(theme.dark_brown())),
            Span::styled("⌂", Style::default().fg(terrain_color(Terrain::House))),
            Span::styled("House ", Style::default().fg(theme.dark_brown())),
        ]),
        Line::from(vec![
            Span::styled("▒", Style::default().fg(terrain_color(Terrain::Farmland))),
            Span::styled("Farm ", Style::default().fg(theme.dark_brown())),
            Span::styled("~", Style::default().fg(terrain_color(Terrain::Swamp))),
            Span::styled("Swmp ", Style::default().fg(theme.dark_brown())),
            Span::styled("·", Style::default().fg(theme.dark_brown())),
            Span::styled("Fog  ", Style::default().fg(theme.dark_brown())),
            Span::styled(
                "@",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("You", Style::default().fg(theme.dark_brown())),
        ]),
        Line::from(vec![
            Span::styled("+", Style::default().fg(terrain_color(Terrain::Door))),
            Span::styled("Door ", Style::default().fg(theme.dark_brown())),
            Span::styled(
                "T",
                Style::default()
                    .fg(theme.archive_red())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("avern,", Style::default().fg(theme.dark_brown())),
            Span::styled(
                "C",
                Style::default()
                    .fg(theme.archive_red())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("hapel…", Style::default().fg(theme.dark_brown())),
        ]),
    ];
    let legend = Paragraph::new(legend_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.paper())),
    );
    let legend_rect = ratatui::layout::Rect {
        x: map_area.x + map_area.width.saturating_sub(22),
        y: map_area.y,
        width: 22,
        height: 6,
    };
    f.render_widget(legend, legend_rect);

    let terrain_name = if let Some(t) = tiles.get(py * map_w + px) {
        format!("{:?}", t)
    } else {
        "??".into()
    };

    let coord = format!(" ({},{}) {}", px, py, terrain_name);

    let status_line = if let Some(ref msg) = app.status_msg {
        Line::from(Span::styled(
            format!(" ⚠ {}", msg),
            Style::default().fg(theme.archive_red()),
        ))
    } else {
        Line::from("")
    };

    let help_line = Line::from(vec![
        Span::styled(
            " [hjkl/↑↓←→]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" move ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[w]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" wait ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[g]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" gather ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[r]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" rest ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[i]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" inv ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[c]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" craft ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" enter ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[?]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" help", Style::default().fg(theme.dark_brown())),
    ]);
    let help_line2 = Line::from(vec![
        Span::styled(
            " [s]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" save ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[m]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" journal ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[M]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" overmap ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[b]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" build ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" title", Style::default().fg(theme.dark_brown())),
        Span::styled(coord, Style::default().fg(theme.dark_brown())),
    ]);
    let help = Paragraph::new(vec![status_line, help_line, help_line2])
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
