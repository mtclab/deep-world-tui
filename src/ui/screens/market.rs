use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_market_screen(f: &mut Frame, app: &App, scroll: u16) {
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
    let enclave = app.current_settlement().and_then(|s| s.enclave_people());
    let mut lines: Vec<Line> = Vec::new();
    if let Some(pk) = enclave {
        // A trade floor of the Five: no coin, fixed in-kind measures. Lay down
        // a good (the sell keys) and take theirs.
        lines.push(Line::from(Span::styled(
            format!(" The {} take no coin — they trade in kind.", pk.label()),
            Style::default().fg(theme.warm_brown()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Lay down a good, take theirs",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )));
        for (i, &item) in items.iter().enumerate() {
            let have = inv.get(item);
            match crate::model::enclave_barter(pk, item) {
                Some((cost, gives)) => {
                    let got = gives
                        .iter()
                        .map(|(it, q)| format!("{} {}", q, it.name()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let color = if have >= cost {
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
                        Span::styled(
                            format!("{} {:<7}", cost, item.name()),
                            Style::default().fg(color),
                        ),
                        Span::styled(
                            format!(" (have {have}) -> {got}"),
                            Style::default().fg(theme.dark_brown()),
                        ),
                    ]));
                }
                None => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!(" [{}] ", sell_keys[i]),
                            Style::default().fg(theme.dark_brown()),
                        ),
                        Span::styled(
                            format!("{:<9}", item.name()),
                            Style::default().fg(theme.dark_brown()),
                        ),
                        Span::styled(
                            " — they want none of it".to_string(),
                            Style::default().fg(theme.dark_brown()),
                        ),
                    ]));
                }
            }
        }
    } else {
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
            let price = app.quote_buy_price(item);
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
            let price = app.quote_sell_price(item);
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
