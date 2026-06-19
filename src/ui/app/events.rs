use crate::sim::hints;

use crate::ui::event::AppEvent;

use super::*;

impl App {
    pub(super) fn fire_hint(&mut self, hint_id: &str) {
        if !self.hint_tracker.should_show(hint_id) {
            return;
        }
        if let Some(text) = hints::hint_text(hint_id) {
            self.hint_tracker.mark_shown(hint_id);
            if let Some(ref mut sim) = self.sim {
                use crate::sim::journal::Voice;
                let voice = match hint_id {
                    hints::HINT_FIRST_ENCOUNTER | hints::HINT_FIRST_COLLAPSE => Voice::Encounter,
                    _ => Voice::Travel,
                };
                sim.log(sim.world.tick, voice, text.to_string());
            }
        }
    }

    /// Best-effort sound playback; no-op if audio disabled.
    pub fn play_sound(&self, event: crate::audio::SoundEvent) {
        let cfg = crate::audio::AudioConfig {
            enabled: self.audio_enabled,
            volume: self.audio_volume,
        };
        crate::audio::play(event, cfg);
    }

    pub fn pre_draw(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        if self.flash_frames > 0 {
            self.flash_frames -= 1;
        }
    }

    pub fn trigger_flash(&mut self) {
        if !self.reduced_motion {
            self.flash_frames = 3;
        }
    }

    /// Drive the game from a controller button (#484): the button becomes the
    /// keystroke it maps to and runs the very same `handle_event` path the
    /// keyboard uses, so a gamepad reaches every screen with no per-screen
    /// controller code. A backend (gilrs, Steam Input) calls this on a press;
    /// an unbound button does nothing.
    pub fn handle_gamepad_button(&mut self, button: crate::ui::input::gamepad::GamepadButton) {
        if let Some(code) = crate::ui::input::gamepad::key_for(button) {
            let key = crossterm::event::KeyEvent::new(code, crossterm::event::KeyModifiers::NONE);
            self.handle_event(AppEvent::Key(key));
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        if let AppEvent::Key(key) = &event {
            if key.code == crossterm::event::KeyCode::Esc {
                self.play_sound(crate::audio::SoundEvent::UiCancel);
            }
        }
        match event {
            AppEvent::Key(key) => match self.screen {
                Screen::TitleScreen => {
                    crate::ui::input::title::handle_title_input(self, key);
                }
                Screen::SaveBrowser { .. } => {
                    crate::ui::input::save_browser::handle_save_browser_input(self, key);
                }
                Screen::SaveSlots { scroll } => {
                    crate::ui::input::save_slots::handle_save_slots_input(self, key, scroll);
                }
                Screen::RestPrompt { hours } => {
                    crate::ui::input::rest_prompt::handle_rest_prompt_input(self, key, hours);
                }
                Screen::CharacterCreation => {
                    crate::ui::input::character_creation::handle_character_creation_input(
                        self, key,
                    );
                }
                Screen::World { region_idx } => {
                    crate::ui::input::world::handle_world_input(self, key, region_idx);
                }
                Screen::Overmap { region_idx } => {
                    crate::ui::input::overmap::handle_overmap_input(self, key, region_idx);
                }
                Screen::Inventory => {
                    crate::ui::input::inventory::handle_inventory_input(self, key);
                }
                Screen::Craft { scroll } => {
                    let new_scroll = crate::ui::input::craft::handle_craft_input(self, key, scroll);
                    if let Screen::Craft { .. } = self.screen {
                        self.screen = Screen::Craft { scroll: new_scroll };
                    }
                }
                Screen::Location {
                    scroll,
                    region_idx,
                    settlement_idx,
                } => {
                    let new_scroll = crate::ui::input::location::handle_location_input(
                        self,
                        key,
                        scroll,
                        region_idx,
                        settlement_idx,
                    );
                    if let Screen::Location {
                        region_idx,
                        settlement_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Location {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                        };
                    }
                }
                Screen::Npc {
                    scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => {
                    let new_scroll = crate::ui::input::npc::handle_npc_input(
                        self,
                        key,
                        scroll,
                        region_idx,
                        settlement_idx,
                        person_idx,
                    );
                    if let Screen::Npc {
                        region_idx,
                        settlement_idx,
                        person_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Npc {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                            person_idx,
                        };
                    }
                }
                Screen::Journal { scroll } => {
                    let new_scroll =
                        crate::ui::input::journal::handle_journal_input(self, key, scroll);
                    if let Screen::Journal { .. } = self.screen {
                        self.screen = Screen::Journal { scroll: new_scroll };
                    }
                }
                Screen::Faith { scroll } => {
                    let new_scroll = crate::ui::input::faith::handle_faith_input(self, key, scroll);
                    if let Screen::Faith { .. } = self.screen {
                        self.screen = Screen::Faith { scroll: new_scroll };
                    }
                }
                Screen::CityVisit { idx, scroll } => {
                    let new_scroll = crate::ui::input::city::handle_city_input(self, key, scroll);
                    if let Screen::CityVisit { .. } = self.screen {
                        self.screen = Screen::CityVisit {
                            idx,
                            scroll: new_scroll,
                        };
                    }
                }
                Screen::Talk {
                    scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => {
                    let new_scroll = crate::ui::input::talk::handle_talk_input(
                        self,
                        key,
                        scroll,
                        region_idx,
                        settlement_idx,
                        person_idx,
                    );
                    if let Screen::Talk {
                        region_idx,
                        settlement_idx,
                        person_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Talk {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                            person_idx,
                        };
                    }
                }
                Screen::Market {
                    scroll,
                    region_idx: _,
                    settlement_idx: _,
                } => {
                    let new_scroll =
                        crate::ui::input::market::handle_market_input(self, key, scroll);
                    if let Screen::Market {
                        region_idx,
                        settlement_idx,
                        ..
                    } = self.screen
                    {
                        self.screen = Screen::Market {
                            scroll: new_scroll,
                            region_idx,
                            settlement_idx,
                        };
                    }
                }
                Screen::RoadBarter { people } => {
                    crate::ui::input::road_barter::handle_road_barter_input(self, key, people);
                }
                Screen::Collapse => {
                    crate::ui::input::collapse::handle_collapse_input(self, key);
                }
                Screen::GameOver => {
                    crate::ui::input::game_over::handle_game_over_input(self, key);
                }
                Screen::Help => {
                    crate::ui::input::help::handle_help_input(self, key);
                }
                Screen::Settings => {
                    crate::ui::input::settings::handle_settings_input(self, key);
                }
            },
            AppEvent::Tick => {}
        }
    }
}
