use super::common::{stance_color, stance_label};
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_encounter_screen(f: &mut Frame, app: &App) {
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
            crate::model::EncounterKind::GodShrine => "✦ God Shrine",
            crate::model::EncounterKind::AncientRuin => "🏛 Ancient Ruin",
            crate::model::EncounterKind::HermitCamp => "🔥 Hermit Camp",
            crate::model::EncounterKind::TravelingBard => "🎵 Traveling Bard",
            crate::model::EncounterKind::SpringBloom => "❀ Spring Bloom",
            crate::model::EncounterKind::HarvestMarket => "🏪 Harvest Market",
            crate::model::EncounterKind::WinterSurvivor => "❄ Winter Survivor",
            crate::model::EncounterKind::MerchantCaravan => "🐪 Merchant Caravan",
        };
        lines.push(Line::from(Span::styled(
            format!("  Kind: {}", kind_str),
            Style::default().fg(theme.warm_brown()),
        )));
        if let Some(npc_people) = app.current_settlement_people() {
            let bias = app.inter_people_bias.player_people.bias_toward(npc_people)
                + app.clock.season().bias_modifier();
            let stance = stance_label(bias);
            let stance_color = stance_color(bias, &theme);
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
