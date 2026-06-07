use ratatui::prelude::Stylize;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::model::{craft_recipes, ItemType, Need, Season, Terrain};
use crate::sim::relationships::BondCategory;
use crate::ui::app::{App, Screen};
use crate::ui::theme::Theme;
use crate::voice::Situation;

fn need_bar(val: f64, width: usize) -> String {
    let filled = (val * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

const STATUS_HEIGHT: u16 = 3;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(theme.paper())),
        area,
    );
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
        Screen::Collapse => {
            draw_collapse_screen(f, app);
        }
        Screen::GameOver => {
            draw_game_over_screen(f, app);
        }
        Screen::Help => {
            draw_help_screen(f, app);
        }
        Screen::Settings => {
            draw_settings_screen(f, app);
        }
    }
    draw_status_bar(f, app);
}

fn draw_status_bar(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let area = f.area();
    let status_top = area.height.saturating_sub(STATUS_HEIGHT);
    let status_area = Rect {
        x: 0,
        y: status_top,
        width: area.width,
        height: STATUS_HEIGHT.min(area.height),
    };
    let season = Season::from_day(app.clock.day);
    let season_name = match season {
        Season::Spring => "Spring",
        Season::Summer => "Summer",
        Season::Autumn => "Autumn",
        Season::Winter => "Winter",
    };
    let day = app.clock.day;

    if let Some(ref ps) = app.player_start {
        let people_kind = crate::model::PeopleKind::from_name(&ps.person.people);
        let people = people_kind.label();
        let profession = ps.person.profession.as_str();
        let location = app
            .player_pos
            .and_then(|pos| {
                let region = app.sim.as_ref()?.world.regions.get(pos.region_idx)?;
                let settlement = region.settlements.first()?;
                Some(settlement.name.as_str())
            })
            .unwrap_or("unknown");
        let food = ps.person.needs.get(Need::Food);
        let energy = app.vitals.energy;
        let hunger = app.vitals.hunger;
        let safety = ps.person.needs.get(Need::Safety);
        let money = ps.person.needs.get(Need::Money);
        let line1 = Line::from(vec![
            Span::styled(
                format!(" {} ", ps.person.name),
                Style::default().fg(theme.ink()).bold(),
            ),
            Span::styled(format!("{} ", people), Style::default().fg(theme.ink())),
            Span::styled(
                format!("{} ", profession),
                Style::default().fg(theme.dark_ink()),
            ),
            Span::styled(
                format!("| {} ", location),
                Style::default().fg(theme.warm_brown()),
            ),
            Span::styled(
                format!("| {} d{}", season_name, day),
                Style::default().fg(theme.dark_ink()),
            ),
        ]);
        let line2 = Line::from(vec![
            Span::styled(" F:", Style::default().fg(theme.need_color(food))),
            Span::styled(
                format!("{:.0}% ", food * 100.0),
                Style::default().fg(theme.need_color(food)),
            ),
            Span::styled("E:", Style::default().fg(theme.need_color(energy))),
            Span::styled(
                format!("{:.0}% ", energy * 100.0),
                Style::default().fg(theme.need_color(energy)),
            ),
            Span::styled("H:", Style::default().fg(theme.need_color(hunger))),
            Span::styled(
                format!("{:.0}% ", hunger * 100.0),
                Style::default().fg(theme.need_color(hunger)),
            ),
            Span::styled("S:", Style::default().fg(theme.need_color(safety))),
            Span::styled(
                format!("{:.0}% ", safety * 100.0),
                Style::default().fg(theme.need_color(safety)),
            ),
            Span::styled("M:", Style::default().fg(theme.need_color(money))),
            Span::styled(
                format!("{:.0}%", money * 100.0),
                Style::default().fg(theme.need_color(money)),
            ),
        ]);
        let line3 = Line::from(Span::styled(
            " Tab:switch  Esc:back  ?:help  i:inv  j:journal  m:map  g:gather  r:rest",
            Style::default().fg(theme.dark_ink()).dim(),
        ));
        let status = Paragraph::new(vec![line1, line2, line3])
            .block(Block::default().style(Style::default().bg(theme.paper())));
        f.render_widget(status, status_area);
    } else {
        let line = Line::from(Span::styled(
            format!(" {} d{} | Press Enter to begin", season_name, day),
            Style::default().fg(theme.dark_ink()),
        ));
        let status =
            Paragraph::new(line).block(Block::default().style(Style::default().bg(theme.paper())));
        f.render_widget(status, status_area);
    }
}

fn draw_character_creation(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — Who Are You?", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(ref ps) = app.player_start {
        let p = &ps.person;
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " The fates have shaped you thus:",
            Style::default()
                .fg(theme.warm_brown())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("   Personality    {}", p.personality.join(", ")),
            Style::default().fg(theme.ink()),
        )));
        let npc_pk = crate::model::PeopleKind::from_name(&p.people);
        let bias = app.inter_people_bias.player_people.bias_toward(npc_pk);
        let stance = if bias > 0.05 {
            "ally"
        } else if bias > -0.05 {
            "neutral"
        } else if bias > -0.15 {
            "wary"
        } else {
            "hostile"
        };
        let stance_color = if bias > 0.05 {
            theme.need_color(1.0)
        } else if bias < -0.05 {
            theme.need_color(0.0)
        } else {
            theme.dark_brown()
        };
        lines.push(Line::from(vec![
            Span::styled("   Toward you    ", Style::default().fg(theme.ink())),
            Span::styled(stance, Style::default().fg(stance_color)),
        ]));
        lines.push(Line::from(Span::styled(
            format!("   People        {}", p.people),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Profession    {}", p.profession),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Craft         {}", p.craft_affinity),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Social Class  {}", p.social_class),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Age           {}", p.age_band),
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
        lines.push(Line::from(Span::styled(
            format!("   Rerolls: {}", ps.reroll_count),
            Style::default().fg(theme.dark_brown()),
        )));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " You stand at the threshold of the Kingdom of Ahjorath.",
            Style::default().fg(theme.warm_brown()),
        )));
        lines.push(Line::from(Span::styled(
            " The Archive watches. The Sepát wait.",
            Style::default().fg(theme.warm_brown()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Press Enter to see who you might become.",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Enter]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" accept  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[R]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" reroll  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Q]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_world_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — Archive of Ahjorath",
            Style::default().fg(theme.warm_brown()),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(ref sim) = app.sim {
        let world = &sim.world;
        lines.push(Line::from(Span::styled(
            format!(" Tick {}", world.tick),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        let settlements = app.settlement_list();
        for (ri, region) in world.regions.iter().enumerate() {
            lines.push(Line::from(Span::styled(
                format!(" {} [{}]", region.name, region.region_type),
                Style::default()
                    .fg(theme.region_color(&region.region_type))
                    .add_modifier(Modifier::BOLD),
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
                        Style::default().fg(theme.dark_brown()),
                    )));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  [{}]", key_label),
                            Style::default()
                                .fg(theme.archive_red())
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(
                                " {} ({}, pop {})",
                                settlement.name, size_label, settlement.population
                            ),
                            Style::default().fg(theme.dark_brown()),
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
            Style::default().fg(theme.warm_brown()),
        )));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [1-9]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " enter settlement  ",
            Style::default().fg(theme.dark_brown()),
        ),
        Span::styled(
            "[Space]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" step  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[A]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" x10  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[J]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" journal  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[S]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" save  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[L]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" load  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Q]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(theme.dark_brown())),
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
    let theme = Theme {
        monochrome: app.monochrome,
    };
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
            .fg(theme.archive_red())
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
                    .fg(theme.archive_red())
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
                    Style::default().fg(theme.warm_brown()),
                )));
            }
        }
        if let Some(dominant) = s.people.first() {
            let npc_people = crate::model::PeopleKind::from_name(&dominant.people);
            let player_people = app.inter_people_bias.player_people;
            let atmosphere = if player_people == npc_people {
                "Your people's settlement. Familiar faces, familiar ways."
            } else {
                let bias = player_people.bias_toward(npc_people);
                if bias > 0.05 {
                    "Allies dwell here. You sense goodwill in the air."
                } else if bias > -0.05 {
                    "Strangers, but not unfriendly. The market watches you evenly."
                } else if bias > -0.15 {
                    "Tension in the glances. You are watched more than welcomed."
                } else {
                    "Hostile gazes follow you. You are not wanted here."
                }
            };
            let atm_color = {
                let bias = player_people.bias_toward(npc_people);
                if bias > 0.05 {
                    theme.need_color(1.0)
                } else if bias < -0.05 {
                    theme.need_color(0.0)
                } else {
                    theme.dark_brown()
                }
            };
            lines.push(Line::from(Span::styled(
                format!(" {}", atmosphere),
                Style::default().fg(atm_color),
            )));
        }
        lines.push(Line::from(Span::styled(
            format!(" Population: {}", s.population),
            Style::default().fg(theme.warm_brown()),
        )));
        lines.push(Line::from(Span::styled(
            format!(" Size: {}", s.size),
            Style::default().fg(theme.warm_brown()),
        )));
        if !s.services.is_empty() {
            let svc_str: String = s
                .services
                .iter()
                .map(|svc| format!("{} {}", svc.glyph(), svc.label()))
                .collect::<Vec<_>>()
                .join("  ");
            lines.push(Line::from(Span::styled(
                format!(" Services: {}", svc_str),
                Style::default().fg(theme.warm_brown()),
            )));
        }
        if !s.description.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" {}", s.description),
                Style::default().fg(theme.dark_brown()),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " People",
            Style::default()
                .fg(theme.archive_red())
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
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        " {} — {} ({})",
                        person.name, person.profession, person.people
                    ),
                    Style::default().fg(theme.ink()),
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
                    Style::default().fg(theme.need_color(val)),
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" person  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[m]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" market  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[s]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" service  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc/Q]",
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

fn draw_npc_screen(
    f: &mut Frame,
    app: &App,
    region_idx: usize,
    settlement_idx: usize,
    person_idx: usize,
    scroll: u16,
) {
    let theme = Theme {
        monochrome: app.monochrome,
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
                        theme.need_color(rel.strength),
                    )
                } else {
                    ("   Bond     stranger".into(), theme.dark_brown())
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

fn draw_journal_screen(f: &mut Frame, app: &App, scroll: u16) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — Journal", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(ref sim) = app.sim {
        if sim.journal.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " The Archive holds no records yet.",
                Style::default().fg(theme.warm_brown()),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Time passes. Events will be recorded here.",
                Style::default().fg(theme.dark_brown()),
            )));
        } else {
            for entry in sim.journal.iter().rev() {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" [{}] ", entry.tick),
                        Style::default()
                            .fg(theme.archive_red())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(entry.text.clone(), Style::default().fg(theme.ink())),
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
        Span::styled(" scroll", Style::default().fg(theme.dark_brown())),
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
    let theme = Theme {
        monochrome: app.monochrome,
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

    let header = if let Some(p) = &person {
        format!(" Deep World — Talking to {}", p.name)
    } else {
        " Deep World — Talk".to_string()
    };

    let title = Paragraph::new(Line::from(Span::styled(
        header,
        Style::default()
            .fg(theme.archive_red())
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
        let npc_people = crate::model::PeopleKind::from_name(&p.people);
        let player_people = app.inter_people_bias.player_people;
        if player_people != npc_people {
            let greeting = player_people.greeting_to(npc_people);
            lines.push(Line::from(Span::styled(
                format!("  {}", greeting),
                Style::default().fg(theme.dark_brown()),
            )));
        }
        lines.push(Line::from(""));

        for (sit, label) in &situations {
            let vline = crate::voice::voice_line_situation_biased(
                p,
                *sit,
                app.inter_people_bias.player_people,
            );
            lines.push(Line::from(Span::styled(
                format!(" [{}]", label),
                Style::default()
                    .fg(theme.warm_brown())
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("   {}", vline),
                Style::default().fg(theme.ink()),
            )));
            lines.push(Line::from(""));
        }

        let low_food = p.needs.get(Need::Food) < 0.5;
        let low_money = p.needs.get(Need::Money) < 0.5;

        lines.push(Line::from(Span::styled(
            " Actions",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        if low_food {
            lines.push(Line::from(Span::styled(
                "   Food is low. Share a meal?",
                Style::default().fg(theme.need_color(0.0)),
            )));
        }
        if low_money {
            lines.push(Line::from(Span::styled(
                "   Coin is thin. Offer payment?",
                Style::default().fg(theme.need_color(0.0)),
            )));
        }
        if low_money {
            lines.push(Line::from(Span::styled(
                "   Coin is thin. Offer payment?",
                Style::default().fg(theme.need_color(0.0)),
            )));
        }

        if let Some(ref msg) = app.status_msg {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" {}", msg),
                Style::default().fg(theme.warm_brown()),
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" give food  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[C]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" give coin  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc/Q]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_alerts_screen(f: &mut Frame, app: &App, scroll: u16) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — Need Alerts", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let critical = app.critical_need_people();
    let mut lines: Vec<Line> = Vec::new();

    if critical.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " No critical needs. The Archive rests.",
            Style::default().fg(theme.warm_brown()),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!(" {} people in dire need", critical.len()),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        for (name, settlement, profession, need, val) in &critical {
            let bar = need_bar(*val, 8);
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", name), Style::default().fg(theme.ink())),
                Span::styled(
                    format!("({}) ", settlement),
                    Style::default().fg(theme.dark_brown()),
                ),
                Span::styled(
                    format!("{}, ", profession),
                    Style::default().fg(theme.dark_brown()),
                ),
                Span::styled(
                    format!("{:?} ", need),
                    Style::default()
                        .fg(theme.need_color(*val))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(bar, Style::default().fg(theme.need_color(*val))),
                Span::styled(
                    format!(" {:.0}%", val * 100.0),
                    Style::default().fg(theme.need_color(*val)),
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
        Span::styled(" scroll", Style::default().fg(theme.dark_brown())),
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
        Terrain::Coast => Color::Rgb(0x6a, 0x9a, 0xba),
        Terrain::Cave => Color::Rgb(0x4a, 0x4a, 0x5a),
        Terrain::Tundra => Color::Rgb(0xaa, 0xc0, 0xcc),
        Terrain::DeepDesert => Color::Rgb(0xd2, 0xba, 0x8a),
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
    let theme = Theme {
        monochrome: app.monochrome,
    };
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &region_name,
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  {}  {} {}",
                app.clock_str(),
                app.vitals.hunger_label(),
                app.vitals.energy_label()
            ),
            Style::default().fg(theme.dark_ink()),
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
            Span::styled("░", Style::default().fg(terrain_color(Terrain::Grass))),
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
            Span::styled("█", Style::default().fg(terrain_color(Terrain::Settlement))),
            Span::styled("Town ", Style::default().fg(theme.dark_brown())),
        ]),
        Line::from(vec![
            Span::styled("▒", Style::default().fg(terrain_color(Terrain::Farmland))),
            Span::styled("Farm ", Style::default().fg(theme.dark_brown())),
            Span::styled("~", Style::default().fg(terrain_color(Terrain::Swamp))),
            Span::styled("Swmp ", Style::default().fg(theme.dark_brown())),
            Span::styled(
                "@",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("You", Style::default().fg(theme.dark_brown())),
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
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" move  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[g]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" gather  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[r]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" rest  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[i]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" inv  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[c]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" craft  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" enter  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back", Style::default().fg(theme.dark_brown())),
        Span::styled(coord, Style::default().fg(theme.dark_brown())),
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
    let theme = Theme {
        monochrome: app.monochrome,
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

fn draw_inventory_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
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
            ItemType::Coin => Color::Rgb(0xc2, 0x9a, 0x2a),
            ItemType::Herb => theme.need_color(1.0),
            ItemType::Wood => theme.warm_brown(),
            ItemType::Stone => Color::Rgb(0x8a, 0x7a, 0x6a),
            ItemType::Cloth => Color::Rgb(0xc2, 0x9a, 0x6b),
            ItemType::Iron => Color::Rgb(0x5a, 0x5a, 0x6a),
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
                        let stance = if bias > 0.05 {
                            "welcomed"
                        } else if bias > -0.05 {
                            "tolerated"
                        } else if bias > -0.15 {
                            "distrusted"
                        } else {
                            "unwelcome"
                        };
                        let stance_color = if bias > 0.05 {
                            theme.need_color(1.0)
                        } else if bias < -0.05 {
                            theme.need_color(0.0)
                        } else {
                            theme.dark_brown()
                        };
                        lines.push(Line::from(vec![
                            Span::styled("  Here:   ", Style::default().fg(theme.dark_brown())),
                            Span::styled(
                                format!("{} settlement", npc_people.label()),
                                Style::default().fg(theme.ink()),
                            ),
                            Span::styled(
                                format!(" — {}", stance),
                                Style::default().fg(stance_color),
                            ),
                        ]));
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
        let label = if *val > 0.6 {
            "favored"
        } else if *val > 0.3 {
            "pleased"
        } else if *val > 0.0 {
            "noticed"
        } else if *val == 0.0 {
            "unknown"
        } else if *val > -0.3 {
            "wary"
        } else if *val > -0.6 {
            "displeased"
        } else {
            "angered"
        };
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

fn draw_craft_screen(f: &mut Frame, app: &App, scroll: u16) {
    let theme = Theme {
        monochrome: app.monochrome,
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

fn draw_market_screen(f: &mut Frame, app: &App, scroll: u16) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " Market",
        Style::default()
            .fg(theme.archive_red())
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
        Style::default().fg(theme.warm_brown()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Buy",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )));
    for (i, &item) in items.iter().enumerate() {
        let price = item.base_price();
        let can = coins >= price;
        let color = if can {
            theme.need_color(1.0)
        } else {
            theme.dark_brown()
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" [{}] ", buy_keys[i]),
                Style::default()
                    .fg(theme.archive_red())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<8}", item.name()), Style::default().fg(color)),
            Span::styled(
                format!(" {} coins", price),
                Style::default().fg(theme.dark_brown()),
            ),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Sell",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )));
    for (i, &item) in items.iter().enumerate() {
        let price = item.base_price();
        let have = inv.get(item);
        let color = if have > 0 {
            theme.need_color(1.0)
        } else {
            theme.dark_brown()
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" [{}] ", sell_keys[i]),
                Style::default()
                    .fg(theme.archive_red())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<8}", item.name()), Style::default().fg(color)),
            Span::styled(
                format!(" (have {}) -> {} coins", have, price),
                Style::default().fg(theme.dark_brown()),
            ),
        ]));
    }

    let para = Paragraph::new(lines)
        .style(Style::default().bg(theme.paper()))
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [1-6]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" buy  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[a-f]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" sell  ", Style::default().fg(theme.dark_brown())),
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

fn draw_encounter_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " Encounter!",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    if let Some(enc) = app.encounter {
        lines.push(Line::from(Span::styled(
            enc.kind.description(),
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
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
            Style::default().fg(theme.warm_brown()),
        )));
        if let Some(npc_people) = app.current_settlement_people() {
            let bias = app.inter_people_bias.player_people.bias_toward(npc_people)
                + app.clock.season().bias_modifier();
            let stance = if bias > 0.05 {
                "ally"
            } else if bias > -0.05 {
                "neutral"
            } else if bias > -0.15 {
                "wary"
            } else {
                "hostile"
            };
            let stance_color = if bias > 0.05 {
                theme.need_color(1.0)
            } else if bias < -0.05 {
                theme.need_color(0.0)
            } else {
                theme.dark_brown()
            };
            lines.push(Line::from(vec![
                Span::styled("  Local stance: ", Style::default().fg(theme.warm_brown())),
                Span::styled(stance, Style::default().fg(stance_color)),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " What do you do?",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
        for action in enc.kind.available_actions() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" [{}] ", action.key()),
                    Style::default()
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(action.label(), Style::default().fg(theme.ink())),
            ]));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  Hunger: {}  Energy: {}",
            app.vitals.hunger_label(),
            app.vitals.energy_label()
        ),
        Style::default().fg(theme.dark_brown()),
    )));

    let para = Paragraph::new(lines).style(Style::default().bg(theme.paper()));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [key]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" act  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" flee", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_collapse_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " You collapsed!",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    if let Some(collapse) = app.collapse {
        lines.push(Line::from(Span::styled(
            collapse.outcome.description(),
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", collapse.outcome.glyph()),
            Style::default().fg(theme.archive_red()),
        )));
        if let Some(god) = collapse.rescued_by {
            lines.push(Line::from(Span::styled(
                format!("  {} watches over you.", god.label()),
                Style::default().fg(theme.warm_brown()),
            )));
        }
        if collapse.outcome.is_hostile() {
            lines.push(Line::from(Span::styled(
                "  You are wounded and shaken.",
                Style::default().fg(theme.need_color(0.0)),
            )));
        }
        if collapse.outcome.is_beast_aided() {
            lines.push(Line::from(Span::styled(
                "  The forest creatures remember your kindness.",
                Style::default().fg(theme.need_color(1.0)),
            )));
        }
        if collapse.outcome.is_divine() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  The fire's warmth lingers. Something impossible happened here.",
                Style::default().fg(theme.warm_brown()),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {} hours passed", collapse.outcome.hours_passed()),
            Style::default().fg(theme.dark_brown()),
        )));
        if collapse.outcome.coin_loss() > 0 {
            lines.push(Line::from(Span::styled(
                format!("  Lost {} coins", collapse.outcome.coin_loss()),
                Style::default().fg(theme.need_color(0.0)),
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
        Style::default().fg(theme.dark_brown()),
    )));
    if app.god_affinity.oltzed != 0.0
        || app.god_affinity.keuru != 0.0
        || app.god_affinity.sampsa != 0.0
        || app.god_affinity.masa != 0.0
        || app.god_affinity.kukri != 0.0
    {
        lines.push(Line::from(Span::styled(
            format!(
                "  Gods: Oltzed {:.0}%  Keuru {:.0}%  Sampsa {:.0}%  Masa {:.0}%  Kukri {:.0}%",
                app.god_affinity.oltzed * 100.0,
                app.god_affinity.keuru * 100.0,
                app.god_affinity.sampsa * 100.0,
                app.god_affinity.masa * 100.0,
                app.god_affinity.kukri * 100.0,
            ),
            Style::default().fg(theme.dark_brown()),
        )));
    }

    let para = Paragraph::new(lines).style(Style::default().bg(theme.paper()));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Enter/Esc]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" continue", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_game_over_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let header = Paragraph::new(Line::from(vec![Span::styled(
        " You have perished",
        Style::default()
            .fg(theme.archive_red())
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    if let Some(collapse) = app.collapse {
        lines.push(Line::from(Span::styled(
            collapse.outcome.description(),
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  But this time, you did not wake.",
            Style::default().fg(theme.need_color(0.0)),
        )));
        if let Some(god) = collapse.rescued_by {
            lines.push(Line::from(Span::styled(
                format!("  Even {} could not reach you this time.", god.label()),
                Style::default().fg(theme.warm_brown()),
            )));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  The world continues without you.",
        Style::default().fg(theme.dark_brown()),
    )));
    if app.god_affinity.oltzed != 0.0
        || app.god_affinity.keuru != 0.0
        || app.god_affinity.sampsa != 0.0
        || app.god_affinity.masa != 0.0
        || app.god_affinity.kukri != 0.0
    {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(
                "  Final standing: Oltzed {:.0}%  Keuru {:.0}%  Sampsa {:.0}%  Masa {:.0}%  Kukri {:.0}%",
                app.god_affinity.oltzed * 100.0,
                app.god_affinity.keuru * 100.0,
                app.god_affinity.sampsa * 100.0,
                app.god_affinity.masa * 100.0,
                app.god_affinity.kukri * 100.0,
            ),
            Style::default().fg(theme.dark_brown()),
        )));
    }

    let para = Paragraph::new(lines).style(Style::default().bg(theme.paper()));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [r]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" restart  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[Esc/Q]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn draw_help_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let area = f.area();
    let text = vec![
        Line::from(Span::styled(
            "=== DEEP WORLD — KEY BINDINGS ===",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Movement",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   h/←/j/k/l/↑↓→  Move on map"),
        Line::from("   1-9              Switch region"),
        Line::from("   M                Region overview (overmap)"),
        Line::from(""),
        Line::from(Span::styled(
            " Actions",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   g                Gather resources"),
        Line::from("   r                Rest (8h)"),
        Line::from("   Enter            Enter settlement"),
        Line::from("   Esc/Q            Exit settlement / go back"),
        Line::from("   Space            Advance 1 hour"),
        Line::from(""),
        Line::from(Span::styled(
            " In Settlement",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   t                Talk to NPC (voice lines)"),
        Line::from("   i                Inventory"),
        Line::from("   c                Craft"),
        Line::from("   m                Market (buy/sell)"),
        Line::from("   j                Journal"),
        Line::from("   svcs             Use service (tavern/temple/etc.)"),
        Line::from(""),
        Line::from(Span::styled(
            " Encounter",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   flee/bribe/talk/trade/calm/intimidate/push/shelter"),
        Line::from(""),
        Line::from(Span::styled(
            " Other",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   Ctrl+S           Save game"),
        Line::from("   Ctrl+L           Load game"),
        Line::from("   ?                This help screen"),
        Line::from("   ,                Settings"),
        Line::from("   Q/Esc            Quit"),
    ];
    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(theme.archive_red())),
    );
    f.render_widget(paragraph, area);
}

fn draw_settings_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
    };
    let area = f.area();
    let llm_status = if app.llm_enabled {
        "ON  (persona prompts from LLM)"
    } else {
        "OFF (using voice.rs templates)"
    };
    let mono_status = if app.monochrome {
        "ON  (ink-only palette for accessibility)"
    } else {
        "OFF (full color palette)"
    };
    let text = vec![
        Line::from(Span::styled(
            "=== SETTINGS ===",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " LLM Narrator",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("   Status: {}", llm_status)),
        Line::from(format!("   Endpoint: {}", app.llm_endpoint)),
        Line::from(format!("   Model: {}", app.llm_model)),
        Line::from("   [l] Toggle LLM narrator on/off"),
        Line::from("   [e] Edit endpoint  [o] Edit model"),
        Line::from(""),
        Line::from(Span::styled(
            " Monochrome Mode",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("   Status: {}", mono_status)),
        Line::from("   [m] Toggle monochrome mode"),
        Line::from(""),
        Line::from(" [Esc/Q/,]  Back to game"),
    ];
    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Settings ")
            .border_style(Style::default().fg(theme.archive_red())),
    );
    f.render_widget(paragraph, area);
}
