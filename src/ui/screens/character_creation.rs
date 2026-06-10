use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::App;
use crate::ui::theme::Theme;

use super::common::{stance_color, stance_label};

pub(crate) fn draw_character_creation(f: &mut Frame, app: &App) {
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
        let stance = stance_label(bias);
        let stance_color = stance_color(bias, &theme);
        lines.push(Line::from(vec![
            Span::styled("   Toward you    ", Style::default().fg(theme.ink())),
            Span::styled(stance, Style::default().fg(stance_color)),
        ]));
        lines.push(Line::from(Span::styled(
            format!("   People        {}", p.people),
            Style::default().fg(theme.ink()),
        )));
        let npc_pk = crate::model::PeopleKind::from_name(&p.people);
        lines.push(Line::from(Span::styled(
            format!("     {}", crate::sim::hints::people_flavor(npc_pk)),
            Style::default().fg(theme.dark_brown()),
        )));
        lines.push(Line::from(Span::styled(
            format!("   Profession    {}", p.profession),
            Style::default().fg(theme.ink()),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "     {}",
                crate::sim::hints::profession_flavor(&p.profession)
            ),
            Style::default().fg(theme.dark_brown()),
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
