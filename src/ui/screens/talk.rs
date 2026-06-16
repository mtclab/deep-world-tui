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

pub(crate) fn draw_talk_screen(
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

        // Show "met before" if we have memory of this NPC
        if app.has_met_npc(&p.id) {
            let memory = app.npc_memory(&p.id);
            let count = memory.map_or(0, |m| m.count());
            let trust = app.npc_trust_bonus(&p.id);
            let trust_label = if trust > 0.1 {
                "warmly"
            } else if trust < -0.1 {
                "warily"
            } else {
                "neutrally"
            };
            lines.push(Line::from(Span::styled(
                format!(
                    "  You've met {} time{}. They remember you {}.",
                    count,
                    if count == 1 { "" } else { "s" },
                    trust_label
                ),
                Style::default().fg(theme.warm_brown()),
            )));
            lines.push(Line::from(""));
        }

        for (sit, label) in &situations {
            let vline = crate::voice::voice_line_maybe_llm(
                p,
                *sit,
                app.inter_people_bias.player_people,
                app.llm_enabled,
                &app.llm_endpoint,
                &app.llm_model,
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
        lines.push(Line::from(Span::styled(
            "   Ask them for news of the world (N).",
            Style::default().fg(theme.warm_brown()),
        )));

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
            "[H]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" heal  ", Style::default().fg(theme.dark_brown())),
        Span::styled(
            "[N]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ask news  ", Style::default().fg(theme.dark_brown())),
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
