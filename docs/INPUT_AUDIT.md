# Cabinet input audit — 2026-09-26

## Authority and scope

The user-designated, tested **SM2-Emu Libretro** reference takes precedence
for every Model 2 title it contains, including disagreements with MAME:

- Repository: `Zer0one/sm2-emu-libretro`, revision
  `f4d9051e2a0dbc54dec8ce78feb9a619f2eea812`.
- `Docs/revisione_profili_model2.xlsx`, sheet `Profili`, `A1:AB44`:
  authoritative action/profile/axis inventory. SHA-256:
  `c8c83e39b9257dd211d0527f8ea35526f9111f1ff1bbc610cc33dbf32622f2c2`.
- `src/libretro/input.cpp`: actual profile selection and action translation;
  `data/games.xml`, `src/rom/game.h`, `src/rom/game_db.cpp`: channels, ranges,
  polarity, default wheel-button masks and clone inheritance.
- `src/hw/model2.cpp` and `src/hw/model2_machine_base.h`: electrical port
  interpretation and Air Walkers' matrix. The workbook's Model 3 comparison
  rows are not additional TGPulse games.

MAME `src/mame/sega/model1.cpp` / `model2.cpp` at
`bd7e0b815842ec461e8ad2538d127f3332f5c96c` supplements the reference for
Model 1 and Power Sled, not as an override to tested Model 2 behaviour.
Both reference repositories and the workbook were read-only.

The audit covers all **100 ROM sets** currently in TGPulse's database, including
clones. It verifies frontend signal-to-port translation and the supported
player's axes, not game compatibility, graphics, audio, netplay or full
multiplayer implementation.

## Corrections

| Cabinet / family | Corrected routing |
| --- | --- |
| Indy 500, all revisions | Start at IN0:40; View 1/2 at IN1:01/02; no D-pad Left → Start alias |
| Sega Touring Car, all revisions | Both view inputs restored, using SM2's default wheel masks on IN1 |
| Indy / Touring / Over Rev / Super GT / Manx | Momentary shifts have no H-gate residue; conflicting shifts are released |
| Motor Raid | Punch and Kick remain independent and may be pressed together |
| Sky Target | Start at IN0:40, not IN0:10; proper full-range stick axes and channel order |
| Behind Enemy Lines | Service IN0:04, Test IN0:08, without swapping the user's physical bindings; Missile is not off-screen reload |
| Wing War | Machine Gun / Missile / Smoke at IN1:10/20/40; all four views restored; 360 variant has no view switches and has different stick polarity |
| NetMerc | MVD Holder at IN1:04, no invented Start switch; Y reaches channel 2, not the throttle mirror |
| Star Wars Arcade | Start 2 restored; Y direction, throttle centre/range and independent idle gunner axes |
| Two-button joystick cabinets | Third/unused action line stays released; Dynamite Baseball retains its own Button 1/2 order and Bat Swing |
| Hanguk Pro Yagu / Royal Ascot / Air Walkers | Anonymous Button 1/2 follow SM2's South/East ordering rather than the fighting template |
| All Model 2 cars / bikes | Full 00..ff ADC travel, including Daytona; reversed Bank; reversed Over Rev / Super GT pedals |
| Sega Rally | Handbrake retains intermediate values when rebound to an analog source |
| Ski Super G | Tested SM2 order: ADC 0 Inclining, ADC 1 reversed Swing; not the opposite MAME-generated order |
| Water Ski / Top Skater | Reversed Slide / Curving per SM2; Top Skater Slide remains independent and direct; Water Ski Pitch Left/Right use Action 2/1 (now R1/L1) |
| Wave Runner | Reversed throttle half-range 80→00, independent Roll/Pitch; only the real coin line is used |
| Gun cabinets | Stick aim moves and holds a cursor; reload only on serial guns; correct per-title ADC / serial ranges; rchase2a remains distinct from rchase2 |
| Air Walkers | Port F bit 7 selects the other player pair; P1 controls and Start are not mirrored onto P3. Unsupported P3/P4 remain released |
| Power Sled | Action 3 no longer presses seat 2 Entry; Extra Action reaches Cancel Error |
| Original I/O boards | steer/accel/brake mirror the routed ADC channels instead of bypassing their game-specific calibration |

Per-game unused inputs are released. IN2 is reinitialized on every racing
sample so another cabinet's handbrake/port values cannot leak through.

## Deliberate differences from SM2 physical defaults

The agreed **single global signal catalogue and binding file** remain in place.
This audit is not permission to replace them with per-game RetroPad bindings.

- Action 1 is South OR L1 (Punch / Shot / Shift Down and other collapsed
  primary functions), Action 2 East OR R1 (including Shift Up), Action 3 West,
  Extra Action North. Sequential shifts therefore retain L1 down/R1 up;
  Motor Raid's Punch/Kick are not shifts. Fighting, soccer and baseball retain their semantic
  port-order exceptions. Shoulder/face aliases remain OR, not chords.
- Service remains R3/F8 and Test L3/F2, as explicitly requested previously.
  BEL swaps the destination bits, not these bindings.
- Views retain the one global Down/Left/Right/Up list. Two-view driving
  cabinets consume View / Select 4 (Up) and 1 (Down), following SM2's layout.
- Start also serves Manx TT/Motor Raid Start/VR, Water Ski Select Down and
  Ski Super G Select 3. There are no redundant configurable Start aliases.
- Ski Super G uses Action 2/R1 for left foot and Action 1/L1 for right foot,
  Action 3 for Select 1 and Extra Action for Select 2; it does not copy SM2's
  separate face-button and shoulder assignments.
- Keyboard ramping, deadzones, H-gate latching and initial gear policy are
  retained. This is not a port of all SM2 controller tuning options.

## Implementation boundary

Most changes are confined to `input/signals/routing.rs`, `input.rs` and tests.
Ski Super G's channel override is guarded by the old metadata order; a future
corrected database does not get swapped back. No ROM database regeneration or
second public input list is required.

Air Walkers needs a small hardware mux correction: its immutable wiring flag
comes from the identified ROM in the loader, and the already-saved port-F
latch selects the pair. No new live-input or save-state serialized fields
were added. The initial latch selects P1/P2, matching the reference.

No NVRAM or input configuration was rewritten. The development build includes
the pre-existing pending fullscreen and Return to game menu changes. The
toolkit, updater-managed build and `current` release were not modified.

## Verification and honest limits

`input/signals/audit.rs` contains independent expectations for all 100 sets:

- Every set must have exactly one explicit audit case; a new/removed set makes
  the coverage test fail instead of silently inheriting a generic layout.
- Every public signal is individually rebound, pressed and released; complete
  IN0/IN1/IN2 bytes are checked, including unused inputs and latch exceptions.
- Real default button identifiers are checked for Indy/Touring/Over Rev,
  including Start, both views, Service/Test and conflicting shifts.
- All driving revisions are checked for ADC idle/endpoints, polarity and
  original-board channel mirrors. Additional tests cover flight/body axes,
  gun calibration/cursor persistence, throttle ranges and partial handbrake.
- A core test checks that selecting Air Walkers' second pair preserves
  coin/service lines but never repeats P1/P2 Start or controls.

Run `cargo test --workspace --offline` and `cargo build --release --offline`.
Tests use a device-free input constructor: they neither poll the user's
controller nor start force-feedback worker threads.

The local verification passed **89 tests** (55 in the frontend) and the release
build. `tgpulse.dev --list` also completed through the installed launcher and
found 100 ROM sets. This smoke check does not launch a game.

These are source-backed automated routing checks, **not physical-controller
or in-game proof for all 100 titles**. Validate using `tgpulse.dev`, starting
with Indy 500's input test and gameplay, then revised analog cabinets.
Existing NVRAM calibrated against the former ADC range may need the game's
normal input-calibration procedure; never erase it automatically.

Existing capability limits remain explicit: TGPulse exposes one gameplay
controller, not SM2's full P1–P4 device configuration. Start 2 alone does not
provide P2 gameplay controls. Air Walkers P3/P4 and Power Sled's second seat
are not implemented. Power Sled's extra IN3 network-check switch has no
frontend/core input lane and was not invented as another action or public
configuration list. No full multiplayer or device-emulation expansion is
claimed by this mapping audit.
