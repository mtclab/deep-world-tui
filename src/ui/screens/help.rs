use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw_help_screen(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
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
        Line::from("   f                Forage for medicinal herbs (forest/mire richest)"),
        Line::from("   r                Rest (8h)    R   Quick full night's rest"),
        Line::from("   p                Pray (devotion to the god you keep)"),
        Line::from("   O                Keep the festival (when a holy day is underway)"),
        Line::from("   o                Offer at a shrine/temple (gives Food, deepens devotion)"),
        Line::from(
            "   P                Pilgrimage from a holy site (needs Food; the long road of faith)",
        ),
        Line::from("   F                Faith ledger (your standing with each of the Five)"),
        Line::from("   V                Swear/renounce a god-vow (at a temple, when Blessed)"),
        Line::from("   J                Journey to a great city (from a town; needs Food)"),
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
        Line::from("   (walk into NPC)  Talk (then t for voice lines)"),
        Line::from("   t                Tend your sickness (herb/salve/bandage)"),
        Line::from("   i                Inventory"),
        Line::from("   c                Craft"),
        Line::from("   m                Market (buy/sell)"),
        Line::from("   j                Journal"),
        Line::from("   H                Encounter log"),
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
        Line::from("   (controller)     Built with --features gamepad: d-pad/stick walk,"),
        Line::from("                    A act, B back, X gather, Y rest, bumpers forage/pray"),
        Line::from(""),
        Line::from(Span::styled(
            " Accessibility",
            Style::default()
                .fg(theme.archive_red())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   All screens fully keyboard-navigable (no mouse needed)"),
        Line::from("   Stance symbols: ++ ally, ~ neutral, - wary, -- hostile"),
        Line::from("   ▸ marks selected item in scrollable lists"),
        Line::from("   [h] High contrast mode (white on black, settings screen)"),
        Line::from("   Color never the sole signal; symbols/markers always present"),
    ];
    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(theme.archive_red())),
    );
    f.render_widget(paragraph, area);
}
