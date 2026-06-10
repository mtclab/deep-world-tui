use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_game_over_screen(f: &mut Frame, app: &App) {
    use crate::sim::milestones::{faction_key, legacy_summary};

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

    let is_elder = app
        .milestones
        .has(crate::sim::milestones::MilestoneKind::ElderAchieved);
    let header_text = if is_elder {
        " A life fulfilled"
    } else {
        " You have perished"
    };
    let header = Paragraph::new(Line::from(vec![Span::styled(
        header_text,
        Style::default()
            .fg(if is_elder {
                theme.warm_brown()
            } else {
                theme.archive_red()
            })
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
        if is_elder {
            lines.push(Line::from(Span::styled(
                "  A long journey reaches its end. The world remembers.",
                Style::default().fg(theme.warm_brown()),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "  But this time, you did not wake.",
                Style::default().fg(theme.need_color(0.0)),
            )));
        }
        if let Some(god) = collapse.rescued_by {
            lines.push(Line::from(Span::styled(
                format!("  Even {} could not reach you this time.", god.label()),
                Style::default().fg(theme.warm_brown()),
            )));
        }
    }
    if let Some(cause) = app.death_cause {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", cause.flavor()),
            Style::default()
                .fg(theme.dark_brown())
                .add_modifier(Modifier::ITALIC),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  The world continues without you.",
        Style::default().fg(theme.dark_brown()),
    )));
    lines.push(Line::from(""));
    let settlements = app.milestones.settlements_visited;
    let quests = app.milestones.quests_completed;
    lines.push(Line::from(Span::styled(
        format!(
            "  Days survived: {}  |  Encounters: {}  |  Collapses: {}",
            app.clock.day, app.encounters_had, app.collapses_had
        ),
        Style::default().fg(theme.dark_brown()),
    )));
    if settlements > 0 || quests > 0 {
        lines.push(Line::from(Span::styled(
            format!(
                "  Settlements visited: {}  |  Quests completed: {}",
                settlements, quests,
            ),
            Style::default().fg(theme.dark_brown()),
        )));
    }
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

    let structures_built = app.sim.as_ref().map_or(0, |s| {
        s.structures.iter().filter(|st| !st.is_npc_built).count()
    });
    let has_companion = app
        .player_start
        .as_ref()
        .is_some_and(|ps| !ps.companions.is_empty());
    let sim_ref = app.sim.as_ref();
    let player_id = app.player_start.as_ref().map(|ps| ps.person.id.clone());
    let legacy_lines = legacy_summary(
        &app.milestones,
        structures_built,
        has_companion,
        |people: crate::model::PeopleKind| {
            let fk = faction_key(people);
            let pid = match &player_id {
                Some(id) => id.as_str(),
                None => return 0.5,
            };
            let sim = match &sim_ref {
                Some(s) => s,
                None => return 0.5,
            };
            let total: f64 = sim
                .reputation
                .entries
                .values()
                .filter(|e| e.person_id == pid)
                .map(|e| e.reputation.by_faction.get(fk).copied().unwrap_or(0.5))
                .sum::<f64>();
            let count = sim
                .reputation
                .entries
                .values()
                .filter(|e| e.person_id == pid)
                .count();
            if count > 0 {
                total / count as f64
            } else {
                0.5
            }
        },
    );
    if !legacy_lines.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Legacy:",
            Style::default()
                .fg(theme.ink())
                .add_modifier(Modifier::BOLD),
        )));
        for ll in &legacy_lines {
            lines.push(Line::from(Span::styled(
                format!("  {}", ll),
                Style::default().fg(theme.dark_brown()),
            )));
        }
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
