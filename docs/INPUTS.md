# Input translation layer

The frontend has one public signal catalogue, one binding file
(`config/input.conf`), and one unfiltered Cabinet list in Settings → Input.
Bindings produce logical signal values; the input translator routes them to
the existing cabinet controls and I/O ports according to the ROM set and its
`Scheme` / `AnalogRole` metadata. The emulation core is unchanged.

Desktop pad buttons are tracked per device from gilrs' logical press/release
events. Native-code fallback polling is deliberately avoided: on macOS an
SDL-mapped Xbox R3 can share the fallback code for D-pad Right, otherwise
activating Service and View / Select 3 together. Disconnecting clears the
device's button state; this does not change the binding catalogue.

Trigger bindings keep the `LeftZ` / `RightZ` names. When a pad mapping has no
corresponding axis, the frontend reads the analog value of `LeftTrigger2` /
`RightTrigger2` instead. Actual mapped axes take priority; the existing pedal
deadzone and calibration apply equally to both representations.

## Editing bindings

Click a Cabinet binding, edit its expression, then Apply. Cancel discards the
editor contents. An invalid expression is reported without replacing the
working binding. Emulator hotkeys retain keyboard capture.

## Shared defaults and game meanings

One signal has one binding, regardless of the game. SM2-Emu positions are
retained where compatible with this rule. There are no game-specific hidden
physical bindings and no redundant Shot/Shift/Foot Sensor entries.

| Signal | Pad | Keyboard | Examples of routed functions |
| --- | --- | --- | --- |
| Action 1 | East OR R1 | J, E, Space | Punch, long pass, shot, shift up, left shot/pitch/foot |
| Action 2 | South OR L1 | K, Q, R | Kick, short pass, secondary/reload, shift down, right shot/pitch/foot |
| Action 3 | West | L | Guard/hold, shoot in soccer, Desert shift, left dash, Water Ski set |
| Extra Action | North | I | Rally handbrake, right dash, Ski Super G Select 2 |
| View / Select 1–4 | Down, Left, Right, Up | Z, X, C, V | VR buttons, view changes, menu selections and zoom |
| H-Gate gears 1–4 | Right-stick diagonals | 1–4 | Direct gear selection |
| H-Gate neutral | West | 0 | Neutral |

Face buttons and shoulders are OR alternatives, not a chord: either one
activates the same signal. R2/L2 remain analog pedals. The aliases preserve
SM2-Emu's shot/shift shoulder positions as well as fighting face positions.
Virtual On uses the four action signals for its four independent shot/dash
functions, with both sticks kept independent.

Axis defaults use left X for steering/bank/handle/curving/swing; left X/Y for
gun aim and analog flight; right X for roll/inclining; and right Y
(downward half) for Bat Swing. Accelerator and Brake use R2/L2.
Slide is split into Water Ski: Slide (left X, arrows/A/D) and Top Skater:
Slide (right X, U/O), matching their respective SM2-Emu defaults.
The GUI shows game-family names in parentheses below Analog Joystick X/Y
and Extra Action. For Extra Action, each game also includes its function:
Sega Rally: Handbrake; Virtual On: Right Dash / Turbo; Ski Super G: Select 2.
This is explanatory text, not a game filter.
Driving signals are ordered Steering / Bank, Accelerator, Brake, then all
H-Gate gears and neutral. Cars and bikes use the same `steering` binding.

Keyboard steering retains arrows and A/D; pedals retain W/S and Up/Down.
Independent roll/Top Skater slide/inclining use U/O. Flight Y, elevation and Wave Runner
pitch use G/T, avoiding simultaneous pedal or action activation. Flight X is
arrows/A/D. Virtual On uses WASD for the left stick and arrows for the right.
The generic Analog Joystick X/Y labels cover both Sky Target and Model 1
flight games, so they are no longer prefixed with one game's name.

Examples:

```ini
format = signals-v1
steering = keys:ArrowLeft/ArrowRight, pad:LeftStickX
gear1 = Digit1, pad:RightStickX- & pad:RightStickY+
gear2 = Digit2, pad:RightStickX- & pad:RightStickY-
gear3 = Digit3, pad:RightStickX+ & pad:RightStickY+
gear4 = Digit4, pad:RightStickX+ & pad:RightStickY-
neutral = Digit0, pad:West
```

- Commas separate alternatives. The strongest absolute value wins.
- `&` requires every source simultaneously above 0.5.
- `pad:Axis` reads a signed axis; `pad:Axis~` reverses it.
- `pad:Axis+` and `pad:Axis-` read one half of an axis.
- `keys:Negative/Positive` creates a signed keyboard axis.
- An empty value unbinds a signal; a missing entry retains its default.
- Full signed axes and keyboard pairs cannot be used in a digital chord.

gilrs uses positive stick Y for **up**. This is opposite to Libretro's Y
convention; the H-gate expressions account for that difference. Direct gear
selection latches when released. Conflicting direct selections leave the
current gear unchanged. Sequential shifting remains edge-triggered.

Virtua Racing / Virtua Formula do not use the Daytona H-gate encoding:
Action 1/2 drive their native active-low Shift Up/Down switches. Both are
released at rest and on conflicting requests. Direct H-gate signals are
unused for those two games. VR4 remains independent of the shift switches.

Old-format files are migrated at startup after a backup is written to
`input.conf.pre-signals`. Existing keyboard directions, common actions and
hotkeys are adapted; controller assignments use the new defaults. The backup
is never overwritten. It is not a second active binding file.
Merged actions retain the union of their old keyboard keys. The old
Space-for-view-change alias is intentionally not imported: Space now belongs
to Action 1, so importing it would also fire in Sky Target. Use V for View 4.
Previously shared flight-Y and body-axis keys use the independent defaults
above to avoid co-activating pedals or actions.

## Upstream reapplication boundary

- `input/signals/mod.rs`: sole public catalogue and defaults.
- `input/signals/expression.rs`: expression parser/evaluator.
- `input/signals/routing.rs`: game-aware translation, independent axes,
  digital exceptions and direct gear selection.
- `bindings.rs`: persistence, migration and hotkeys.
- `input.rs`: existing polling/calibration, with narrow calls to the translator.
- `app.rs`: passes the identified ROM set to input.
- `gui/mod.rs`: displays the full catalogue and edits its bindings.

The retained `Control` enum identifies internal cabinet requests and touch
overlay inputs. It has no separate user-facing catalogue or configuration.
Digital cabinet wiring is not fully represented in the ROM database, so
those exceptions live in the translator rather than changing the core.

Automated tests check expressions, persistence/migration, analog travel,
game-dependent routing, H-gate latching and D-pad/pedal separation. They do
not substitute for physical-controller and in-game testing.
