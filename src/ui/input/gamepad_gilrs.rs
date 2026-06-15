//! The live gamepad backend (#484), behind the `gamepad` feature. Reads a
//! physical controller via `gilrs` and turns its presses into device-agnostic
//! [`GamepadButton`]s; the pure mapping in `super::gamepad` takes it from there.
//!
//! Feature-gated so the default build (and CI) never pulls `gilrs` — which on
//! Linux needs the system `libudev` to build. With Steam Input, the new Steam
//! Controller presents as a standard gamepad, so this same reader serves it and
//! the grips/trackpads arrive as whatever Steam Input binds them to.

use super::gamepad::GamepadButton;
use gilrs::{Button, EventType, Gilrs};

/// A connected-controller reader. `None` from [`Pad::new`] means no gamepad
/// subsystem (then the game simply runs on the keyboard as before).
pub struct Pad {
    gilrs: Gilrs,
}

impl Pad {
    pub fn new() -> Option<Self> {
        Gilrs::new().ok().map(|gilrs| Pad { gilrs })
    }

    /// Drain the controller's pending events and return the logical buttons
    /// pressed (button-down edges) since the last poll. Call once per frame.
    pub fn poll_pressed(&mut self) -> Vec<GamepadButton> {
        let mut out = Vec::new();
        while let Some(ev) = self.gilrs.next_event() {
            if let EventType::ButtonPressed(button, _) = ev.event {
                if let Some(b) = map_button(button) {
                    out.push(b);
                }
            }
        }
        out
    }
}

/// Map a physical `gilrs` button to our logical [`GamepadButton`]. Note gilrs'
/// names: `LeftTrigger`/`RightTrigger` are the *bumpers* (LB/RB), and
/// `LeftTrigger2`/`RightTrigger2` are the *triggers* (LT/RT).
pub fn map_button(b: Button) -> Option<GamepadButton> {
    use GamepadButton as G;
    Some(match b {
        Button::DPadUp => G::DpadUp,
        Button::DPadDown => G::DpadDown,
        Button::DPadLeft => G::DpadLeft,
        Button::DPadRight => G::DpadRight,
        Button::South => G::FaceSouth,
        Button::East => G::FaceEast,
        Button::West => G::FaceWest,
        Button::North => G::FaceNorth,
        Button::LeftTrigger => G::LeftBumper,
        Button::RightTrigger => G::RightBumper,
        Button::LeftTrigger2 => G::LeftTrigger,
        Button::RightTrigger2 => G::RightTrigger,
        Button::Start => G::Start,
        Button::Select => G::Select,
        // The Steam Controller's grips/trackpads arrive through Steam Input as
        // whatever it binds them to; gilrs has no native name for them, so the
        // user's Steam Input profile owns those.
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gilrs_buttons_map_to_logical_ones() {
        assert_eq!(map_button(Button::DPadDown), Some(GamepadButton::DpadDown));
        assert_eq!(map_button(Button::South), Some(GamepadButton::FaceSouth));
        // gilrs LeftTrigger is the bumper; LeftTrigger2 is the trigger.
        assert_eq!(
            map_button(Button::LeftTrigger),
            Some(GamepadButton::LeftBumper)
        );
        assert_eq!(
            map_button(Button::LeftTrigger2),
            Some(GamepadButton::LeftTrigger)
        );
        assert_eq!(map_button(Button::Unknown), None);
    }
}
