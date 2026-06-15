# Steam Controller / Gamepad Support — Plan (#484)

Goal: play the whole game with a controller, targeting **Valve's new (2025)
Steam Controller** first, generic gamepads second. The game is a terminal
ratatui roguelike driven entirely by `KeyCode` events, so controller support is
fundamentally a *second input source that produces the same key events* — not a
rewrite of input handling.

## Why this is tractable for a TUI

Every screen already routes a `crossterm` `KeyEvent` through a handler
(`handle_world_input`, the menu handlers, etc.). If a controller can be made to
emit the same `KeyCode`s, the entire game is already controller-playable. So the
work is a thin, well-tested translation layer plus a backend that reads the
device — and **Steam Input already does most of the device half for us**.

## The new Steam Controller (2025)

It presents over Steam Input as a standard gamepad — face buttons, d-pad, two
sticks (magnetic TMR), bumpers, triggers — plus its signatures: two **back grip
buttons**, two **trackpads**, and a gyro. A grid-walking TUI does not need the
gyro or analog precision; we use the d-pad/left-stick for the four-way walk, the
face buttons for the common verbs, and the grips/pads for the verbs you'd
otherwise leave the sticks to reach. Because it speaks Steam Input, the *same*
plan covers every controller Steam exposes (Xbox, DualSense, Deck) for free.

## Architecture (three layers)

1. **Backend (device → logical button).** A reader turns physical inputs into
   `input::gamepad::GamepadButton` (device-agnostic). Two backends:
   - `gilrs` (cross-platform) behind a `gamepad` cargo feature, for running
     outside Steam.
   - Steam Input, when shipped on Steam — see the config section.
2. **Mapping (logical button → game intent → keystroke).** This is
   `src/ui/input/gamepad.rs`, **already implemented and unit-tested**:
   `GamepadButton::default_action() -> Option<GamepadAction>` and
   `GamepadAction::to_key() -> KeyCode`, composed as `key_for(button)`. Pure, no
   device dependency, so it is tested in CI without hardware.
3. **Dispatch (keystroke → existing handlers).** The main loop feeds the
   resulting `KeyEvent` into the same screen handlers the keyboard uses. One
   source of truth for what every input does; no per-screen controller code.

```
device → GamepadButton → GamepadAction → KeyCode → existing handlers
        (backend)        (gamepad.rs, tested)       (unchanged)
```

## Default binding (implemented in `gamepad.rs`)

| Input | Action | Key |
|-------|--------|-----|
| D-pad / left stick | Walk N/S/W/E | `k`/`j`/`h`/`l` |
| A (south) | Confirm / interact / market | `Enter` |
| B (east) | Back / cancel | `Esc` |
| X (west) | Gather | `g` |
| Y (north) | Rest | `R` |
| Left bumper | Forage | `f` |
| Right bumper | Pray | `p` |
| Left trigger | Inventory | `i` |
| Right trigger | Act (Enter) | `Enter` |
| **Grip left** | Journey to a city | `J` |
| **Grip right** | Wait an hour | `w` |
| Start | Map | `M` |
| Select | Help | `?` |
| Trackpad clicks | *(free for the player's own Steam Input binds)* | — |

## Implementation phases

- **Phase 0 — pure mapping (DONE, this PR).** `input::gamepad` + tests. No
  dependency, no device, CI-safe. The whole binding is decided and locked here.
- **Phase 1 — gilrs backend behind `--features gamepad`.** Add `gilrs`
  (optional dep). In the event loop, poll gilrs each frame; on a button-down,
  resolve `key_for(button)` and inject a synthetic `KeyEvent` into the existing
  dispatch. Left-stick beyond a deadzone maps to the d-pad actions (with a
  repeat delay so a held stick steps, not sprints). Feature-gated so headless
  and CI builds never pull the dependency.
- **Phase 2 — Steam Input ship config.** Add a Steam Input Game Actions File
  (`.vdf`) defining the action set (Move, Confirm, Cancel, Gather, Rest,
  Forage, Pray, Journey, Wait, Inventory, Map, Help) and a default
  Steam-Controller binding matching the table above; map the grips and a pad to
  the secondary verbs. With Steam Input emulating a gamepad, the gilrs backend
  receives it transparently — or we read the Steam Input action set directly via
  the Steamworks SDK if shipped. Players can remap freely at the Steam layer.
- **Phase 3 — discoverability.** A small on-screen hint line / help page for
  controller players; on-screen glyphs for the bound buttons.

## Testing strategy

- The mapping layer is pure and fully unit-tested (Phase 0): every bound button
  resolves to a key the game already handles; the walk, the verbs, and the
  Steam-Controller grips are pinned by tests.
- The gilrs backend and Steam Input config need a device/Steam and so are
  validated by hand, not in CI; the feature flag keeps them out of the default
  build and the test gate.

## Open questions

- Stick-walk feel: step-per-press vs. timed repeat while held (Phase 1 tuning).
- Whether to surface a trackpad as a radial verb menu (a natural fit for the new
  controller, but a real UI; deferred past Phase 3).
