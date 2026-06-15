//! Gamepad / Steam Controller input mapping (#484, plan in
//! `docs/STEAM_CONTROLLER.md`).
//!
//! This module is the **pure, testable core** of controller support: it knows
//! nothing of any device or driver. A backend (e.g. `gilrs`, behind the
//! `gamepad` feature, or Steam Input) reads physical buttons and turns them
//! into [`GamepadButton`]s; this module maps those to a [`GamepadAction`] (the
//! game's intent), and an action down to the same `KeyCode` the keyboard
//! handlers already understand — so the controller drives the existing input
//! paths and nothing else has to change.
//!
//! Targeting Valve's new (2025) Steam Controller: it presents as a standard
//! gamepad through Steam Input (face buttons, d-pad, sticks, bumpers, triggers)
//! plus its signatures — back **grip** buttons and **trackpads** — which we
//! give the secondary verbs. A grid-walking TUI needs no gyro.

use crossterm::event::KeyCode;

/// A logical controller button, device-agnostic. The backend maps a physical
/// input (an Xbox `A`, a Steam Controller pad-click, a DualSense cross) onto
/// one of these; the game never sees the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadButton {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    /// Bottom face button (Xbox `A`, PlayStation cross).
    FaceSouth,
    /// Right face button (Xbox `B`, PlayStation circle).
    FaceEast,
    /// Left face button (Xbox `X`, PlayStation square).
    FaceWest,
    /// Top face button (Xbox `Y`, PlayStation triangle).
    FaceNorth,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Start,
    Select,
    /// A back grip button — a Steam Controller signature.
    GripLeft,
    /// A back grip button — a Steam Controller signature.
    GripRight,
    /// A trackpad pressed as a click (the new controller's pads).
    LeftPadClick,
    RightPadClick,
}

/// The game's intent behind a press — what the player means, not which key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadAction {
    MoveNorth,
    MoveSouth,
    MoveWest,
    MoveEast,
    /// Confirm / interact / open the thing in front of you (Enter).
    Confirm,
    /// Back out / cancel / leave (Esc).
    Cancel,
    Gather,
    Rest,
    Forage,
    Pray,
    Journey,
    Wait,
    Inventory,
    Map,
    Help,
}

impl GamepadButton {
    /// The default action this button means. `None` for buttons left unbound
    /// (a user's Steam Input profile can still remap them at the device layer).
    pub fn default_action(self) -> Option<GamepadAction> {
        use GamepadAction as A;
        use GamepadButton as B;
        Some(match self {
            B::DpadUp => A::MoveNorth,
            B::DpadDown => A::MoveSouth,
            B::DpadLeft => A::MoveWest,
            B::DpadRight => A::MoveEast,
            B::FaceSouth => A::Confirm,
            B::FaceEast => A::Cancel,
            B::FaceWest => A::Gather,
            B::FaceNorth => A::Rest,
            B::LeftBumper => A::Forage,
            B::RightBumper => A::Pray,
            B::LeftTrigger => A::Inventory,
            B::RightTrigger => A::Confirm, // the trigger is "act" — same as Enter
            B::GripLeft => A::Journey,
            B::GripRight => A::Wait,
            B::Start => A::Map,
            B::Select => A::Help,
            // Trackpad clicks are free for the user's own Steam Input binds.
            B::LeftPadClick | B::RightPadClick => return None,
        })
    }
}

impl GamepadAction {
    /// The `KeyCode` this action presses, so a controller drives the very same
    /// keyboard handlers the game already has (`handle_world_input` and the
    /// rest) — one source of truth for what every input does.
    pub fn to_key(self) -> KeyCode {
        use GamepadAction as A;
        match self {
            A::MoveNorth => KeyCode::Char('k'),
            A::MoveSouth => KeyCode::Char('j'),
            A::MoveWest => KeyCode::Char('h'),
            A::MoveEast => KeyCode::Char('l'),
            A::Confirm => KeyCode::Enter,
            A::Cancel => KeyCode::Esc,
            A::Gather => KeyCode::Char('g'),
            A::Rest => KeyCode::Char('R'),
            A::Forage => KeyCode::Char('f'),
            A::Pray => KeyCode::Char('p'),
            A::Journey => KeyCode::Char('J'),
            A::Wait => KeyCode::Char('w'),
            A::Inventory => KeyCode::Char('i'),
            A::Map => KeyCode::Char('M'),
            A::Help => KeyCode::Char('?'),
        }
    }
}

/// The whole chain a backend uses: a physical button → its default action →
/// the keystroke the game already understands. `None` if the button is unbound.
pub fn key_for(button: GamepadButton) -> Option<KeyCode> {
    button.default_action().map(GamepadAction::to_key)
}

/// The dead-zone past which a stick counts as pushed in a direction.
pub const STICK_DEADZONE: f32 = 0.5;

/// Which way the left stick points, as a d-pad direction — the four-way walk a
/// grid map needs (#484). Inside the dead-zone it points nowhere; outside, the
/// dominant axis wins. Convention: stick **up is +y** (gilrs' default), so a
/// pushed-up stick walks north. Pure, so it is tested without any device.
pub fn stick_direction(x: f32, y: f32) -> Option<GamepadButton> {
    if x.abs() < STICK_DEADZONE && y.abs() < STICK_DEADZONE {
        return None;
    }
    Some(if x.abs() >= y.abs() {
        if x > 0.0 {
            GamepadButton::DpadRight
        } else {
            GamepadButton::DpadLeft
        }
    } else if y > 0.0 {
        GamepadButton::DpadUp
    } else {
        GamepadButton::DpadDown
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dpad_walks_the_four_ways() {
        assert_eq!(key_for(GamepadButton::DpadUp), Some(KeyCode::Char('k')));
        assert_eq!(key_for(GamepadButton::DpadDown), Some(KeyCode::Char('j')));
        assert_eq!(key_for(GamepadButton::DpadLeft), Some(KeyCode::Char('h')));
        assert_eq!(key_for(GamepadButton::DpadRight), Some(KeyCode::Char('l')));
    }

    #[test]
    fn the_stick_points_a_cardinal_way() {
        // Inside the dead-zone: nowhere.
        assert_eq!(stick_direction(0.0, 0.0), None);
        assert_eq!(stick_direction(0.3, -0.2), None);
        // Pushed each way (up is +y), dominant axis wins.
        assert_eq!(stick_direction(0.0, 0.9), Some(GamepadButton::DpadUp));
        assert_eq!(stick_direction(0.0, -0.9), Some(GamepadButton::DpadDown));
        assert_eq!(stick_direction(0.9, 0.0), Some(GamepadButton::DpadRight));
        assert_eq!(stick_direction(-0.9, 0.0), Some(GamepadButton::DpadLeft));
        // A diagonal resolves to the stronger axis.
        assert_eq!(stick_direction(0.9, 0.6), Some(GamepadButton::DpadRight));
        assert_eq!(stick_direction(0.6, 0.9), Some(GamepadButton::DpadUp));
    }

    #[test]
    fn the_face_buttons_are_the_common_verbs() {
        assert_eq!(
            GamepadButton::FaceSouth.default_action(),
            Some(GamepadAction::Confirm)
        );
        assert_eq!(
            GamepadButton::FaceEast.default_action(),
            Some(GamepadAction::Cancel)
        );
        assert_eq!(
            GamepadButton::FaceWest.default_action(),
            Some(GamepadAction::Gather)
        );
        assert_eq!(
            GamepadButton::FaceNorth.default_action(),
            Some(GamepadAction::Rest)
        );
    }

    #[test]
    fn the_steam_controller_grips_carry_the_journey_and_the_wait() {
        // The back grips — a Steam Controller signature — get the road and the
        // pause, verbs you reach for without leaving the sticks.
        assert_eq!(key_for(GamepadButton::GripLeft), Some(KeyCode::Char('J')));
        assert_eq!(key_for(GamepadButton::GripRight), Some(KeyCode::Char('w')));
    }

    #[test]
    fn trackpad_clicks_are_left_free_for_the_player() {
        assert_eq!(GamepadButton::LeftPadClick.default_action(), None);
        assert_eq!(key_for(GamepadButton::RightPadClick), None);
    }

    #[test]
    fn every_bound_action_reaches_a_key() {
        // Sweep the buttons: a bound one must resolve to a keystroke the game
        // already handles (no action is left dangling).
        let buttons = [
            GamepadButton::DpadUp,
            GamepadButton::DpadDown,
            GamepadButton::DpadLeft,
            GamepadButton::DpadRight,
            GamepadButton::FaceSouth,
            GamepadButton::FaceEast,
            GamepadButton::FaceWest,
            GamepadButton::FaceNorth,
            GamepadButton::LeftBumper,
            GamepadButton::RightBumper,
            GamepadButton::LeftTrigger,
            GamepadButton::RightTrigger,
            GamepadButton::Start,
            GamepadButton::Select,
            GamepadButton::GripLeft,
            GamepadButton::GripRight,
        ];
        for b in buttons {
            assert!(
                b.default_action().is_some(),
                "{b:?} should have a default action"
            );
            assert!(key_for(b).is_some(), "{b:?} should resolve to a key");
        }
    }
}
