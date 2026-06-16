use crate::model::GodName;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

const GODS: [GodName; 5] = [
    GodName::Oltzed,
    GodName::Keuru,
    GodName::Sampsa,
    GodName::Masa,
    GodName::Kukri,
];

/// A ten-cell bar for a god's standing: filled for devotion, faint for a
/// grudge. Pure text so it reads the same in monochrome.
fn standing_bar(affinity: f64) -> String {
    let cells = 10;
    let filled = ((affinity.abs() * cells as f64).round() as usize).min(cells);
    let glyph = if affinity < 0.0 { '·' } else { '█' };
    let mut s = String::new();
    for i in 0..cells {
        s.push(if i < filled { glyph } else { '░' });
    }
    s
}

/// The faith ledger (#457): a read-only panel of where the player stands with
/// each of the Five — affinity, devotion rank, and the grace each rank grants.
pub(crate) fn draw_faith_screen(f: &mut Frame, app: &App, scroll: u16) {
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
        Span::styled(" — Faith", Style::default().fg(theme.warm_brown())),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();

    // The god you keep: the one you've served most, else your people's patron.
    let kept = app
        .god_affinity
        .strongest_ally()
        .or_else(|| crate::sim::god::patron_of(app.inter_people_bias.player_people.label()));
    match kept {
        Some(god) => {
            let rank = crate::sim::god::devotion_rank(app.god_affinity.get(god));
            let rank_str = rank.map(|r| format!(" — {r}")).unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled(
                    " The god you keep: ",
                    Style::default().fg(theme.dark_brown()),
                ),
                Span::styled(
                    format!("{} {}{}", god.glyph(), god.label(), rank_str),
                    Style::default()
                        .fg(theme.ink())
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        None => {
            lines.push(Line::from(Span::styled(
                " You keep no god yet — the practice is a long road.",
                Style::default().fg(theme.dark_brown()),
            )));
        }
    }
    lines.push(Line::from(""));

    let patron = crate::sim::god::patron_of(app.inter_people_bias.player_people.label());
    for god in GODS {
        let aff = app.god_affinity.get(god);
        let rank = crate::sim::god::devotion_rank(aff);
        let is_patron = patron == Some(god);
        let name_color = if rank.is_some() {
            theme.ink()
        } else {
            theme.warm_brown()
        };
        let mut name = format!("{} {:<7}", god.glyph(), god.label());
        if is_patron {
            name.push_str(" (your people's)");
        }
        lines.push(Line::from(vec![
            Span::styled(format!(" {name:<26}"), Style::default().fg(name_color)),
            Span::styled(
                standing_bar(aff),
                Style::default().fg(if aff < 0.0 {
                    theme.dark_brown()
                } else {
                    theme.archive_red()
                }),
            ),
            Span::styled(
                format!("  {}", rank.unwrap_or("—")),
                Style::default()
                    .fg(theme.warm_brown())
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("   {}", god.domains()),
            Style::default().fg(theme.dark_brown()),
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        " The ranks of devotion:",
        Style::default()
            .fg(theme.warm_brown())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "   a Keeper, then Devoted, then Blessed. At Devoted and beyond, a god's",
        Style::default().fg(theme.dark_brown()),
    )));
    lines.push(Line::from(Span::styled(
        "   particular grace answers the festival kept and the pilgrim road walked.",
        Style::default().fg(theme.dark_brown()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Deepen it: pray (p), lay an offering (o), keep a festival (O),",
        Style::default().fg(theme.dark_brown()),
    )));
    lines.push(Line::from(Span::styled(
        "   walk a pilgrimage (P) from a shrine or temple.",
        Style::default().fg(theme.dark_brown()),
    )));

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc/Q/F/←]",
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

#[cfg(test)]
mod tests {
    use super::standing_bar;

    #[test]
    fn an_empty_standing_shows_no_fill() {
        assert_eq!(standing_bar(0.0), "░░░░░░░░░░");
    }

    #[test]
    fn full_devotion_fills_the_bar_solid() {
        assert_eq!(standing_bar(1.0), "██████████");
    }

    #[test]
    fn a_grudge_marks_the_bar_faintly_not_solid() {
        let bar = standing_bar(-0.5);
        assert!(bar.contains('·'), "a grudge reads faint, not solid");
        assert!(!bar.contains('█'), "a grudge is never solid devotion");
    }

    #[test]
    fn standing_never_overflows_the_bar() {
        // Affinity is clamped to [-1, 1], but the bar must hold even past it.
        assert_eq!(standing_bar(2.0).chars().count(), 10);
        assert_eq!(standing_bar(-2.0).chars().count(), 10);
    }
}
