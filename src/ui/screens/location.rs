use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::common::need_bar;
use crate::model::Need;
use crate::ui::app::App;
use crate::ui::theme::Theme;

pub(crate) fn draw_location_screen(
    f: &mut Frame,
    app: &App,
    region_idx: usize,
    settlement_idx: usize,
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
                let hash = crate::sim::signals::hash_str(&ps.person.id);
                let gesture = crate::sim::signals::body_language(rep, hash);
                lines.push(Line::from(Span::styled(
                    format!(" At the gate, a face in the crowd {gesture}."),
                    Style::default().fg(theme.dark_ink()),
                )));
                let welcome = crate::sim::signals::settlement_welcome_note(rep);
                if !welcome.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!(" {welcome}"),
                        Style::default()
                            .fg(theme.archive_red())
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            }
        }
        if let Some(dominant) = s.people.first() {
            let npc_people = crate::model::PeopleKind::from_name(&dominant.people);
            let player_people = app.inter_people_bias.player_people;
            let rep_in_settle = app
                .sim
                .as_ref()
                .and_then(|sim| {
                    app.player_start
                        .as_ref()
                        .map(|ps| sim.reputation.get(&ps.person.id, &s.id))
                })
                .unwrap_or(0.5);
            let adverb = crate::sim::signals::settlement_welcome_adverb(rep_in_settle);
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
            let atm_with_adverb = if adverb.is_empty() || player_people == npc_people {
                atmosphere.to_string()
            } else {
                format!("{atmosphere} They greet you {adverb}.")
            };
            lines.push(Line::from(Span::styled(
                format!(" {atm_with_adverb}"),
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
            let current_activity = person.schedule.activity_at_hour(app.clock.hour);
            let activity_text = format!(" — {} ({})", person.profession, current_activity.name());
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  [{}]", key),
                    Style::default()
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}{}", person.name, activity_text),
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

        if let Some(ref ps) = app.player_start {
            if !ps.companions.is_empty() {
                lines.push(Line::from(Span::styled(
                    " Companions",
                    Style::default()
                        .fg(theme.archive_red())
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                for c in &ps.companions {
                    let state = if !c.is_alive() {
                        "gone"
                    } else if c.is_starving() {
                        "starving"
                    } else if c.is_exhausted() {
                        "exhausted"
                    } else if c.food_need > 40.0 || c.rest_need > 40.0 {
                        "weary"
                    } else {
                        "hardy"
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {} ", c.animal.name()),
                            Style::default()
                                .fg(theme.warm_brown())
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("\"{}\" — ", c.name),
                            Style::default().fg(theme.ink()),
                        ),
                        Span::styled(
                            state.to_string(),
                            Style::default().fg(theme.need_color(if state == "hardy" {
                                1.0
                            } else if state == "weary" {
                                0.5
                            } else {
                                0.2
                            })),
                        ),
                    ]));
                }
                lines.push(Line::from(""));
            }
        }

        if let Some(s) = &settlement {
            if s.allows_companions() {
                if let Some(ref ps) = app.player_start {
                    if ps.companions.len() < 3 {
                        lines.push(Line::from(Span::styled(
                            " The yards hold animals for adoption.",
                            Style::default().fg(theme.dark_brown()),
                        )));
                        lines.push(Line::from(""));
                    }
                }
            }
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
            "[a]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" adopt  ", Style::default().fg(theme.dark_brown())),
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
