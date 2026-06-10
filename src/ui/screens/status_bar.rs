use super::common::{pulse_style, STATUS_HEIGHT};
use crate::model::{Need, Season};
use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::prelude::Stylize;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub(crate) fn draw_status_bar(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
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
        Season::Thaw => "Thaw",
        Season::Green => "Green",
        Season::Frost => "Frost",
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
            Span::styled(
                " F:",
                pulse_style(
                    theme.need_color(food),
                    app.tick_count,
                    food < 0.3,
                    app.reduced_motion,
                ),
            ),
            Span::styled(
                format!("{:.0}% ", food * 100.0),
                pulse_style(
                    theme.need_color(food),
                    app.tick_count,
                    food < 0.3,
                    app.reduced_motion,
                ),
            ),
            Span::styled(
                "E:",
                pulse_style(
                    theme.need_color(energy),
                    app.tick_count,
                    energy < 0.3,
                    app.reduced_motion,
                ),
            ),
            Span::styled(
                format!("{:.0}% ", energy * 100.0),
                pulse_style(
                    theme.need_color(energy),
                    app.tick_count,
                    energy < 0.3,
                    app.reduced_motion,
                ),
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
            " Tab:switch  Esc:back  ?:help  i:inv  j:journal  m:map  g:gather  r:rest  b:build  c:craft",
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
