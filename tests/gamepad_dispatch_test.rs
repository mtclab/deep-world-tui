// Gamepad dispatch (#484): a controller button drives the game through the
// same handler the keyboard uses — no per-screen controller code. These tests
// prove the wiring end to end without any device (the pure mapping decides the
// key, `handle_gamepad_button` runs it).
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{PlayerPos, Terrain};
use deep_world_tui::ui::app::App;
use deep_world_tui::ui::input::gamepad::GamepadButton;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 10;
    a
}

/// An open wild tile (Grass) with an open Grass tile to its south, well clear
/// of any settlement — so a southward step is unobstructed.
fn open_with_south(a: &App) -> Option<(usize, usize)> {
    let r = &a.sim.as_ref().unwrap().world.regions[0];
    for y in 0..r.terrain.height - 1 {
        for x in 0..r.terrain.width {
            if r.terrain.get(x, y) == Some(Terrain::Grass)
                && r.terrain.get(x, y + 1) == Some(Terrain::Grass)
                && !r
                    .settlements
                    .iter()
                    .any(|s| s.contains_tile(x, y) || s.contains_tile(x, y + 1))
            {
                return Some((x, y));
            }
        }
    }
    None
}

#[test]
fn the_dpad_walks_the_player() {
    let mut a = app();
    let (x, y) = open_with_south(&a).expect("open ground");
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: x,
        py: y,
    });
    a.handle_gamepad_button(GamepadButton::DpadDown);
    let p = a.player_pos.unwrap();
    assert_eq!((p.px, p.py), (x, y + 1), "the d-pad steps the player south");
}

#[test]
fn a_face_button_acts() {
    let mut a = app();
    a.status_msg = None;
    // X (west face) gathers — it always reports something back.
    a.handle_gamepad_button(GamepadButton::FaceWest);
    assert!(
        a.status_msg.is_some(),
        "a gather button does something the game answers"
    );
}

#[test]
fn an_unbound_button_does_nothing() {
    let mut a = app();
    a.status_msg = None;
    let before = a.player_pos;
    // The trackpad clicks are left free — no default action.
    a.handle_gamepad_button(GamepadButton::LeftPadClick);
    assert!(a.status_msg.is_none(), "an unbound button is a no-op");
    assert_eq!(a.player_pos, before, "and moves nothing");
}
