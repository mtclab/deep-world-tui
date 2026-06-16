use ratatui::{
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

use crate::ui::app::{App, Screen};
use crate::ui::theme::Theme;

use crate::ui::screens;
use screens::character_creation::draw_character_creation;
use screens::city::draw_city_screen;
use screens::collapse::draw_collapse_screen;
use screens::craft::draw_craft_screen;
use screens::encounter_log::draw_encounter_log_screen;
use screens::encounter_screen::draw_encounter_screen;
use screens::game_over::draw_game_over_screen;
use screens::help::draw_help_screen;
use screens::inventory::draw_inventory_screen;
use screens::journal::draw_journal_screen;
use screens::location::draw_location_screen;
use screens::map::draw_map_screen;
use screens::market::draw_market_screen;
use screens::npc::draw_npc_screen;
use screens::overmap::draw_overmap_screen;
use screens::save_browser::draw_save_browser_screen;
use screens::settings::draw_settings_screen;
use screens::status_bar::draw_status_bar;
use screens::talk::draw_talk_screen;
use screens::title::draw_title_screen;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = Theme {
        monochrome: app.monochrome,
        high_contrast: app.high_contrast,
    };
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(theme.paper())),
        area,
    );
    match app.screen {
        Screen::TitleScreen => draw_title_screen(f, app),
        Screen::SaveBrowser {
            scroll,
            delete_confirm,
        } => {
            draw_save_browser_screen(f, app, scroll, delete_confirm);
        }
        Screen::SaveSlots { scroll } => {
            screens::save_slots::draw_save_slots_screen(f, app, scroll);
        }
        Screen::RestPrompt { hours } => {
            screens::rest_prompt::draw_rest_prompt_screen(f, app, hours);
        }
        Screen::CharacterCreation => draw_character_creation(f, app),
        Screen::World { region_idx } => {
            draw_map_screen(f, app, region_idx);
        }
        Screen::Location {
            region_idx,
            settlement_idx,
            scroll,
        } => {
            draw_location_screen(f, app, region_idx, settlement_idx, scroll);
        }
        Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll,
        } => {
            draw_npc_screen(f, app, region_idx, settlement_idx, person_idx, scroll);
        }
        Screen::Talk {
            region_idx,
            settlement_idx,
            person_idx,
            scroll,
        } => {
            draw_talk_screen(f, app, region_idx, settlement_idx, person_idx, scroll);
        }
        Screen::Journal { scroll } => {
            draw_journal_screen(f, app, scroll);
        }
        Screen::EncounterLog { scroll } => {
            draw_encounter_log_screen(f, app, scroll);
        }
        Screen::CityVisit { idx, scroll } => {
            draw_city_screen(f, app, idx, scroll);
        }
        Screen::Overmap { region_idx } => {
            draw_overmap_screen(f, app, region_idx);
        }
        Screen::Inventory => {
            draw_inventory_screen(f, app);
        }
        Screen::Craft { scroll } => {
            draw_craft_screen(f, app, scroll);
        }
        Screen::Market { scroll, .. } => {
            draw_market_screen(f, app, scroll);
        }
        Screen::Encounter => {
            draw_encounter_screen(f, app);
        }
        Screen::Collapse => {
            draw_collapse_screen(f, app);
        }
        Screen::GameOver => {
            draw_game_over_screen(f, app);
        }
        Screen::Help => {
            draw_help_screen(f, app);
        }
        Screen::Settings => {
            draw_settings_screen(f, app);
        }
    }
    draw_status_bar(f, app);
    if app.flash_frames > 0 && !app.reduced_motion {
        let flash_color = if app.flash_frames % 2 == 1 {
            theme.archive_red()
        } else {
            theme.ink()
        };
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(flash_color)),
            f.area(),
        );
    }
}

pub use screens::minimap::render_minimap;

#[cfg(test)]
mod accessibility_tests {
    use crate::ui::screens::common::*;
    use crate::ui::theme::Theme;
    use ratatui::style::{Color, Modifier};

    #[test]
    fn stance_label_contains_symbol() {
        assert!(stance_label(0.1).starts_with("++"));
        assert!(stance_label(0.0).starts_with("~"));
        assert!(stance_label(-0.1).starts_with("-"));
        assert!(stance_label(-0.2).starts_with("--"));
    }

    #[test]
    fn reputation_label_contains_symbol() {
        assert!(reputation_label(0.7).starts_with("++"));
        assert!(reputation_label(0.0).starts_with("?"));
        assert!(reputation_label(-0.5).starts_with("--"));
        assert!(reputation_label(-0.8).starts_with("---"));
    }

    #[test]
    fn focus_cursor_visible_on_selected() {
        assert_eq!(focus_cursor(true), "▸");
        assert_eq!(focus_cursor(false), " ");
    }

    #[test]
    fn stance_color_varies_with_bias() {
        let theme = Theme {
            monochrome: false,
            high_contrast: false,
        };
        let ally_color = stance_color(0.1, &theme);
        let hostile_color = stance_color(-0.2, &theme);
        let neutral_color = stance_color(0.0, &theme);
        assert_ne!(ally_color, hostile_color);
        assert_eq!(neutral_color, theme.dark_brown());
    }

    #[test]
    fn pulse_style_bold_when_low_and_animating() {
        let color = Color::Red;
        let style = pulse_style(color, 0, true, false);
        assert!(style.add_modifier.contains(Modifier::BOLD));
        let style_off = pulse_style(color, 2, true, false);
        assert!(!style_off.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn pulse_style_no_bold_when_reduced_motion() {
        let color = Color::Red;
        let style = pulse_style(color, 0, true, true);
        assert!(!style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn pulse_style_no_bold_when_not_low() {
        let color = Color::Red;
        let style = pulse_style(color, 0, false, false);
        assert!(!style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn people_kind_glyph_deterministic() {
        use crate::model::PeopleKind;
        let g1 = PeopleKind::Metsik.glyph();
        let g2 = PeopleKind::Metsik.glyph();
        assert_eq!(g1, g2);
        assert_ne!(PeopleKind::Metsik.glyph(), PeopleKind::Arkit.glyph());
        assert_ne!(PeopleKind::Metsik.glyph(), PeopleKind::Vayla.glyph());
    }
}
