use crate::model::ItemType;
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// The great city reached by a journey (#456): a read-only panel of its scale,
/// quarters, deeper services, and the long-haul market — rendered so the
/// continent's scale is felt, not just heard. The trade itself settled on the
/// road (the status line); this is the city you stood in.
pub(crate) fn draw_city_screen(f: &mut Frame, app: &App, idx: usize, scroll: u16) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
    };
    let (name, blurb) = crate::sim::CANON_CITIES
        .get(idx)
        .copied()
        .unwrap_or(("a great city", "the trade of the continent"));

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(f.area());

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {name}"),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  —  {} souls",
                fmt_thousands(crate::sim::city_population(idx))
            ),
            Style::default().fg(theme.warm_brown()),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let head = |s: &str, theme: &Theme| {
        Line::from(Span::styled(
            format!(" {s}"),
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ))
    };
    let body = |s: String, theme: &Theme| {
        Line::from(Span::styled(
            format!("   {s}"),
            Style::default().fg(theme.ink()),
        ))
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    for chunk in wrap_words(crate::sim::city_arrival(idx), 76) {
        lines.push(Line::from(Span::styled(
            format!(" {chunk}"),
            Style::default().fg(theme.warm_brown()),
        )));
    }
    lines.push(Line::from(""));

    lines.push(head("Quarters", &theme));
    for d in crate::sim::city_districts(idx) {
        lines.push(body(format!("· {d}"), &theme));
    }
    lines.push(Line::from(""));

    lines.push(head("Of note", &theme));
    for s in crate::sim::city_services(idx) {
        lines.push(body(format!("· {s}"), &theme));
    }
    lines.push(Line::from(""));

    lines.push(head("The long-haul market", &theme));
    lines.push(body(
        format!("Known for {blurb}. The city pays city prices for what the province sends:"),
        &theme,
    ));
    for item in ItemType::tradeable_items() {
        if matches!(item, ItemType::Food | ItemType::Water | ItemType::Coin) {
            continue;
        }
        // The premium the journey trades at (1.6x base) — why the haul pays.
        let city_price = ((item.base_price() as f64) * 1.6).round() as u32;
        lines.push(Line::from(vec![
            Span::styled(
                format!("   {:<10}", item.name()),
                Style::default().fg(theme.ink()),
            ),
            Span::styled(
                format!("{city_price:>3} coin"),
                Style::default().fg(theme.warm_brown()),
            ),
        ]));
    }

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc/Enter]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " take the road home  ",
            Style::default().fg(theme.dark_brown()),
        ),
        Span::styled(
            "[↑↓]",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll", Style::default().fg(theme.dark_brown())),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

fn fmt_thousands(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}
