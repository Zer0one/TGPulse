# <img src="assets/logo.png" alt="TGPulse Emulator" width="420">

This fork is maintained as [TGPulse-Next](https://github.com/Zer0one/TGPulse-Next),
based on [deepblueworks/TGPulse](https://github.com/deepblueworks/TGPulse).
The executable name remains `tgpulse`; the local development launcher remains `tgpulse.dev`.

Emulator for Sega's Model 1 and Model 2 arcade boards, written in Rust.
Desktop releases provide macOS ARM64 (Metal), Linux x86_64 and Windows x86_64
packages. Android remains a separate source/build path, not a published desktop
release artifact. Download from [GitHub Releases](https://github.com/Zer0one/TGPulse-Next/releases).

## Latest Integrations — 0.1.0.1

- **NetMerc:** advanced I/O, persistent factory SRAM/calibration, reusable serial
  tracking endpoint and complete standalone save-state integration.
- **City:** finite TGP compatibility arithmetic prevents the persistent freeze;
  City Workaround defaults On for correct road-plane geometry. Other games
  retain their existing arithmetic policy.
- **MVD:** fixed camera, right stick or SDL3 sensors, calibration/recenter,
  optional gravity stabilization (default On) and Auto/Manual Holder behavior.
- **Diagnostic LCD:** Off, corner overlay or dedicated window, HD44780/Text
  rendering and overlay position/opacity settings; fullscreen uses overlay.
- **Audio and rumble:** accepted donor/procedural substitute audio, optional
  alternative gains and rumble from NetMerc's actual motor output. A verified
  corrected audio dump remains possible future work, not an included ROM repair.
- **Desktop:** Board filter, configurable rumble intensity and corrected
  diagnostic-window/fullscreen render-target sizing.

Tracking, LCD, NetMerc rumble and substitute audio are closed by user acceptance.
City entry and road geometry have been tested; extended later-level gameplay
coverage remains open. NetMerc is not claimed to be completely accurate or
fully gameplay-validated. See the [current roadmap](docs/MODEL1_ROADMAP.md).

Every processor is emulated at instruction level: the NEC V60 and MB86233 "TGP"
on the Model 1; the Intel i960, the TGP, the ADSP-21062 SHARC and the MB86235
"TGPx4" across the Model 2 revisions. No dump of the geometry engine exists, so
its display-list stream is decoded behaviourally, as MAME's is. Pixel coverage
is a hardware model in a compute shader rather than a modern rasterizer, so
dither patterns and other artefacts are reproduced.

## Running

```sh
cargo build --release
mkdir -p roms && cp ~/wherever/vf2.zip roms/
./target/release/tgpulse
```

**ROMs.** One zip per set in `roms/`, or `--roms <dir>`. Use the standard
zipped chip dumps; do not unzip them or rename their contents. Archive names
are ignored, since sets are identified by matching contents against the ROM
database. None are included here; you need the rights to the data.

**Launching.** With no arguments, the library window lists the sets found in
`roms/`, identifies them, and reports anything missing. Double-click to run;
Refresh after adding archives.
The desktop Library's **Board** button filters by Model 1, Model 2 (original),
Model 2A, Model 2B or Model 2C; **All** restores the complete list, including
unrecognized archives. The filter lasts for the current session, survives
Refresh and returning from a game, and does not affect CLI launches or `--list`.

```sh
./target/release/tgpulse vf2        # by set name
./target/release/tgpulse --list     # list roms/, no window
./target/release/tgpulse --help     # all options
```

**Diagnostic panels.** Open Statistics and/or the GUI Debugger at startup:

```sh
./target/release/tgpulse wingwar --show-stats --show-debugger
```

Both flags also work without a romset, starting in the library. They apply only
to this launch and do not save panel preferences. These are the existing
View menu panels, not `--debug`, which runs the scriptable debugger without a
window. GUI panel flags cannot be combined with `--debug` or `--list`.
The existing visibility rules still apply: Statistics remains visible in
fullscreen, while the Debugger requires the interface to be visible. Add
`--fullscreen off` to inspect the Debugger if games normally start fullscreen.

**Diagnostic LCD.** Settings → **Diagnostic Display** selects Off (default),
Overlay or Dedicated Window for boards exposing the 2 × 20 character LCD.
Fullscreen always uses the overlay when enabled, restoring the dedicated-window
preference on exit. Overlay Position selects one of four corners with a scaled
margin; Overlay Background Opacity controls the background only (0–100%).
Characters stay opaque. These settings persist as `diagnostic_display =
off/window/overlay`, `diagnostic_position = top_left/top_right/bottom_right/bottom_left`
and `diagnostic_opacity = 80`. Closing the auxiliary window selects Off;
closing the game removes its display. The panel works with the menu hidden.
HD44780 dot rendering (default) and Text are selectable independently; see
Diagnostic Rendering below. This does not claim cycle-exact LCD presentation.

**Controls.** Coin `5` (Select), start `Enter` (Start), digital movement on
arrows/WASD or the d-pad. Shared Button 1/Kick is South OR L1, Button 2/Punch
is East OR R1, and Button 3/Guard/Jump/Hold/Barrier is West. Eight cabinets
have independent game-prefixed action signals; Power Sled: Cancel Error uses North. Gun Primary/Secondary
Fire and Sky Target: Machine Gun/Missile have separate assignable signals.
Dedicated Gear Down (E/L1) and Gear Up (Q/R1) control
sequential shifting independently of Action 1/2. Analog driving uses left
stick X and R2/L2. Direct gears 1–4 use right-stick diagonals and latch when
released; West selects neutral. Gun aim uses the mouse or left stick.
Service is R3 and Test L3. The full, unfiltered signal list is editable in
Settings → Input and stored in `config/input.conf`.
See [input bindings and game routing](docs/INPUTS.md) for keyboard assignments,
combination syntax, migration and deliberate differences from SM2-Emu.

**Rumble Intensity.** Settings → Machine → **Rumble Intensity** adjusts supported
P1 pad effects from 0–100%, independently of the rumble enable checkbox.
The default 100% preserves the existing decoder levels; 0% silences both motors.
Changes apply immediately to SDL3 and gilrs output and persist as
`rumble_intensity = 100` in the active settings profile. CLI override:
`--rumble on --rumble-intensity 50`. The touch menu provides the same preference
in 10% steps. This scales host output only, not emulated motor commands or
machine snapshots, and does not add rumble support to unsupported devices.

**Widescreen.** Settings → Widescreen offers Off, On and Auto. Auto follows
the saved monitor/cabinet setting for VR, Indy 500 and Sega Touring Car;
unknown games use 4:3. CLI: `--widescreen auto`. See
[native aspect detection and supported sets](docs/WIDESCREEN.md).

**Colours.** Settings → **sRGB correction** enables correct display-RGB sampling
for the game framebuffer (Model 1 and Model 2), without changing GUI colours.
It takes effect immediately and persists as `srgb = on` in
`config/settings.conf`; default `off` retains the previous presentation.

**Diagnostic Rendering** independently selects `HD44780` (default) or `Text`,
persisted as `diagnostic_rendering = hd44780/text`. HD44780 draws the board's
5x8 dot characters, including programmable CGRAM glyphs, shift and cursor/blink,
in both the overlay and dedicated window. The optional `hd44780_a00.bin` is
loaded first from the game archive, then from MAME's `hd44780.zip` beside it;
missing/invalid resources fall back to Text without preventing game startup.
The font is an external ROM resource and is not included in this repository
or machine save states.

NetMerc's **MVD input** setting supports `Auto`, `Off (fixed camera)`,
`Right Stick` and `Sensors` (`mvd_input = auto/off/right_stick/sensors`, default
`auto`). Auto/Sensors prefer calibrated P1 gyro input on SDL3, falling back to
the right stick while unavailable/calibrating/rejected, then fixed if no P1 pad.
Keep the P1 controller still for five seconds after activation. Calibration
success or rejection (with its reason) appears once as an on-screen notification,
also in fullscreen; Settings contains only adjustments and action buttons.
**Recenter MVD** zeros relative orientation without recalibration,
and **Calibrate MVD** repeats the stationary estimate. Optional **MVD Gravity
Stabilization** (`mvd_gravity_stabilization = on/off`, default `on`) uses the
accelerometer to limit tilt drift in Auto/Sensors. It can be toggled live and
preserves recentering; heading/yaw can still drift and XYZ stays fixed. Cabinet P1
exposes assignable `NetMerc: MVD Look X / Y`, default right stick, with limited
head rotation and recentering on release. Horizontal/vertical limits default
to 30/20 degrees and are selectable from 0 to 90 in steps of 10, persisted as
`mvd_horizontal_degrees` / `mvd_vertical_degrees`. This is independent of the cabinet
stick. These ranges apply only to stick simulation, not sensor orientation.
Changes apply without restarting; the tracking milestone is closed by user
acceptance. Relative yaw can still drift and XYZ stays fixed by design.
Assignable virtual controls `NetMerc: MVD Calibrate` (North) and
`NetMerc: MVD Recenter` (West) invoke the same actions once per press,
including while paused, with no default keyboard binding or emulated I/O pin.

**NetMerc MVD Holder** independently selects `Auto` (default) or `Manual`,
persisted as `mvd_holder = auto/manual`. Auto supplies a bounded Holder pulse
when credit/session state permits, releasing when the game acknowledges it;
Trigger remains manual. Manual uses only the cabinet binding (Start OR D-pad Down / L).
This does not clear a latch already stored by the game or change MVD orientation.

The bounded macOS sensor diagnostic can read DualSense gyro/accelerometer and
estimate stationary gyro bias without launching a ROM or enabling MVD tracking:
`cargo run --offline --release -p tgpulse --example gamepad-motion -- 20 DualSense`.
Keep the controller still until `MOVE` (two seconds settling, then three seconds
calibrating); then rotate each axis separately. The log reports mean, standard
deviation and the precise calibration rejection reason, if any. See
[NetMerc motion checkpoint](docs/MODEL1_NETMERC.md).
For a headless audit of NetMerc's actual motor output versus frame sampling, run
`RUST_LOG=model1_motor=trace cargo run --offline --release -p tgpulse --example netmerc-motor -- roms/netmerc.zip`.
This bounded replay uses an in-memory cabinet and opens no controller/audio
devices or persistent NVRAM. See the
[motor sampling audit](docs/MODEL1_NETMERC.md#motor-pulse-sampling-audit--2026-10-03).
Non-sRGB output surfaces already preserve RGB bytes and need no conversion.
Separately, Model 1's 2D palette intensity bit is always emulated: bit 15 clear
halves RGB after expansion, matching MAME. This hardware correction is not
controlled by the sRGB option and does not alter Model 2 palette behaviour.

**Audio sources.** While a game is loaded, Settings → Audio shows a gain slider,
Mute checkbox and name for each output: MultiPCM 1/2 and FM (YM3438), or SCSP;
SWA/SWAJ also show DSB (MPEG). Sliders set absolute gains (50% = 0.5), with a
full-height dark-blue reference marker: MultiPCM 50%, FM 30%, DSB/SCSP 100%.
The channel slider's grab is 50% opaque and drawn above the reference marker.
Channel range 0–100%; double-click a channel slider to restore its default.
Master volume remains separate (0–800%), with the same marker style and a
100% reference restored by double-click. Gains and mutes persist in `config/settings.conf`.
Mute keeps the selected gain; chip/CPU/timer emulation continues even at zero.
YM3438 synthesis always runs: FM sounds in Virtua Racing have also been
confirmed in-game by the user. Muting FM silences only its output.
See [audio integration and limits](docs/MODEL1_AUDIO.md).

**NetMerc Audio Donor.** Settings → Audio selects Virtua Fighter (default),
Virtua Racing, Star Wars Arcade, Wing War or Off; the saved key is
`netmerc_audio_donor = vf|vr|swa|wingwar|off`. Put the selected donor ZIP beside
`netmerc.zip`; only its MultiPCM sample ROMs are needed, not its BIOS, program,
graphics or DSB data. Selection applies on game load/reset, not mid-song.

This substitutes instruments/sounds while NetMerc's own sound program keeps
driving the chips; it is not an authentic missing-ROM repair. Off or an
unavailable/incomplete donor uses best-effort procedural synthesis for the
known blank NetMerc descriptor dump. A future valid original descriptor dump
keeps original audio with Off. Redumping the missing ROM remains the definitive
solution. An unavailable donor produces an OSD warning and detailed log without
preventing game launch.
Existing MultiPCM gain/mute controls apply. Save states require the same loaded
sample banks and recovery mode; change back to the corresponding donor before
restoring a state. See [fallback rules and limits](docs/MODEL1_NETMERC.md#tgpulse-procedural-checkpoint--2026-10-03).

**NetMerc Alternative Audio Gains.** Optional output preset, applied immediately
and saved as `netmerc_alternative_gains = on|off` (shipped default Off).
With an actually loaded donor, MultiPCM 1/2/FM use 38%/38%/30%; original or
procedural audio uses 50%/50%/30%, preserving procedural normalization.
The channel sliders show the active preset read-only; master volume and mutes
remain editable. Turning the option off restores the stored manual gains.
Other games are unaffected.

**NetMerc City Workaround.** Settings → Machine exposes an optional conversion
override, saved as `netmerc_city_workaround = on|off` (default On); CLI:
`--netmerc-city-workaround on|off`. It applies to subsequent calculations without
resetting the game. Load a pre-transition state when comparing the two settings;
toggling does not undo already calculated results. User gameplay checks found
the override necessary for correct City road geometry. NetMerc always uses the
finite-format TGP compatibility path tested at City entry; other games retain
their existing IEEE arithmetic. This is a bounded compatibility integration,
not a claim of complete hardware arithmetic accuracy. See
[arithmetic integration](docs/MODEL1_NETMERC.md#production-arithmetic-integration--2026-10-04).

**Model 1 networking (experimental).** `cabinet = twin` fits the network board
and enables TCP for supported games. Settings exposes `AddressIn`, `PortIn`,
`AddressOut` and `PortOut`, following Supermodel Standalone's naming with MAME's
local bind address. Apply saves the configuration; reload/reset the game to use
it. TCP uses MAME's M1COMM ring frames, with no automatic loopback. Transport,
board and VR/Wing War boot/link tests pass; synchronized gameplay is not yet
validated. Wing War R360 boots, but its link test currently fails after the game
reinitializes the supplied EEPROM configuration. See
[setup, verification boundaries and remaining work](docs/MODEL1_NETWORK.md).

**Player 2 (Model 1 and Model 2).** Settings → Input has Cabinet P1/P2 tabs with
independent bindings and controller selection. P2 supports local two-player
joystick games, Model 2 guns, Dynamite Baseball's bat, Power Sled's second seat
and SWA/SWAJ's Gunner (stick and two fire buttons, no Start/view/throttle).
Signals without a P2 counterpart remain grey. P2 uses P1 gamepad conventions, with no default gameplay keys;
Coin/Start and shared Test/Service retain keyboard defaults. See
[player assignment and migration](docs/INPUTS.md#player-2--model-1-and-model-2).

| Key | |
| --- | --- |
| `F1` | show or hide the interface over a running game |
| `F3` | reset the machine |
| `F5` / `F7` | save and load the current state slot |
| `F4` / `F6` | previous and next slot |
| `F9` | pause |
| `F11` | fullscreen |
| `Tab` | fast forward while held |

**Model 1 save states.** Use Machine → Save/Load state or `F5`/`F7`, with
slots 0–9 selected by `F4`/`F6`. States use `states/<set>.<slot>.state`
in the emulator's runtime directory, like Model 2. A versioned, ROM-identified
snapshot restores the standalone machine, including in-memory NVRAM/EEPROM,
without immediately writing persistent NVRAM. Current audio/video preferences
and pause mode remain unchanged. Success/errors appear briefly even in fullscreen.
Model 1 save/load requires `cabinet = single`; a fitted COMM board is refused
even before a peer connects. This does not snapshot a network session.
The desktop integration has automated coverage and the user confirms basic
save/load working in VR and SWA; other games and broader scenarios remain to validate.
F4/F6 display the selected slot briefly, including in fullscreen.
The current Model 1 state format is 4 (including NetMerc's in-flight serial
tracking peer and procedural audio state); older state files are rejected.
Persistent NVRAM is unaffected.
See the [checkpoint evidence](docs/MODEL1_ROADMAP.md#desktop-saveload-checkpoint--2026-09-27).

**Files written.** Battery-backed RAM (high scores, rankings, test menu
settings) to `nvram/<set>.nv` on close, reloaded on the next run. Save states to
`states/`, options and bindings to `config/`. Nothing is written elsewhere.

## Rendering

Native output is 496x384 with no antialiasing. The rasterizer is reproduced
exactly, including dither patterns, stipple transparency and painter-sort
artefacts.

- `--ssaa 1..4` (default 2) supersamples the 3D layer and presents it at twice
  board resolution; tile layers (HUD, text, sky) scale by integer factors.
  `--ssaa 1` is the board's output pixel for pixel.
- `--widescreen on` renders 16:9 by widening the frustum and viewport around
  the centre rather than stretching. Not hardware behaviour: games composed for
  4:3 may show edge-pinned HUD elements and scenery ending where the original
  camera stopped. 2D layers stretch to fill unless `--widescreen-stretch-2d
  off`.
- `--smooth-shadows off` reproduces the board's lack of an alpha channel:
  checkerboard-stipple shadows and thresholded per-texel coverage on
  translucent textures. The default blends them.

## Status

The ROM database covers 100 sets, but an entry only means the memory image can
be built, not that the game runs. The established upstream sample set boots and
plays, with rendered frames checked against MAME. The rest may do anything from black-screen
to subtly wrong.

| Board | Tested |
| --- | --- |
| Model 1 | Virtua Racing, Virtua Fighter, Star Wars Arcade; this fork also has user-confirmed Wing War gameplay. NetMerc City entry/geometry and individual peripherals are accepted, but extended gameplay remains under validation. |
| Model 2 | Daytona USA (and Special Edition), Virtua Cop |
| Model 2A | Sega Rally Championship, Virtua Fighter 2 |
| Model 2B | Virtua Striker, Sonic Championship |
| Model 2C | The House of the Dead, Wave Runner |

**Model 2 networking limit.** M2COMM is modelled only as far as one cabinet needs: the
ring closes on itself so the network check passes. Twin Daytona and Virtua
Striker's versus play run as a single machine.

## Roadmap

For this fork's source-audited Model 1 gaps, ROM baseline and bounded fixes, see
[Model 1 roadmap](docs/MODEL1_ROADMAP.md).

- **More games.** New sets tend to expose real bugs: Virtua Fighter 2's hair was an i960 burst-read bug, Wave Runner's failure to boot a missing EEPROM.
- **Performance improvements.**  Could be achieved by moving the coprocessors to their own threads and a JIT/dynarec. Currently it can be slow on low powered devices.
- **Multiplayer.** Link two instances over a socket, as MAME's `m2comm` does.
- **Model 1 save states.** Closed after user tests across the catalogue; COMM-fitted saves remain unsupported.
- **Encrypted sets.** The 315-5881 implementation is in the tree but unused, so Dynamite Cop, Zero Gunner and the rest do not run.
- **NetMerc gameplay.** Tracking, LCD, rumble and substitute audio are closed.
  Extended later-level and repeat-game coverage remain. Low-level Polhemus
  emulation and integration of a future verified corrected audio dump are
  possible follow-ups, not prerequisites for the accepted features.
  See the [NetMerc checkpoint](docs/MODEL1_NETMERC.md).

## Layout

```
crates/i960        Intel i960KB              -- Model 2 main CPU
crates/v60         NEC V60                   -- Model 1 main CPU
crates/mb86233     Fujitsu MB86233 "TGP"     -- Model 1 and 2/2A geometry
crates/mb86235     Fujitsu MB86235 "TGPx4"   -- Model 2C geometry
crates/sharc       Analog Devices ADSP-21062 -- Model 2B geometry
crates/sega-crypt  Sega 315-5881 decryption
crates/tgpulse-core  the machine: memory maps, geometry, sound, save states
crates/tgpulse       the front end: window, renderer, interface, input, audio
tools/               the ROM database generator and a MAME comparison harness
```

`tgpulse-core` has no window, GPU or controller: a front end drives it a frame
at a time and reads back the framebuffer, so the debugger and the comparison
harness can run it headlessly.

## Accuracy

MAME is the reference. Memory maps follow its ordering so the two can be read
side by side, and divergences are found by diffing instruction traces, work RAM
and rendered frames (`tools/mamediff.sh`).

The debugger is script-driven and the machine deterministic, so investigations
replay exactly:

```sh
./target/release/tgpulse vf2 --debug -c "run 1400; geo; vertices"
```

Output is one `kind key=value` line at a time, and every command reports what
it did.

## Building

A Cargo workspace with no vendored dependencies. A stable toolchain from
[rustup](https://rustup.rs) is enough; the binary lands in
`target/release/tgpulse`.

**Linux.**

```sh
sudo apt install build-essential libudev-dev libasound2-dev   # Debian, Ubuntu
sudo pacman -S base-devel systemd-libs alsa-lib               # Arch
sudo dnf install gcc systemd-devel alsa-lib-devel             # Fedora

cargo build --release
```

`libudev` is for gamepad detection, ALSA for audio. Rendering uses Vulkan where
the driver offers it and GL otherwise.

**Windows.** Native builds need the MSVC toolchain and Visual Studio's C++
build tools, then `cargo build --release`. Cross-compiling from Linux needs
only the target and mingw, the linker already being named in
`.cargo/config.toml`:

```sh
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64          # or: pacman -S mingw-w64-gcc
cargo build --release --target x86_64-pc-windows-gnu
```

The result is `target/x86_64-pc-windows-gnu/release/tgpulse.exe`, which wants
the same `roms/` directory beside it.

**Tests.** `cargo test --release`. The processor tests assemble their own
programs and need no ROM data.

Android is packaged with `cargo-apk` and covered in
[docs/BUILDING.md](docs/BUILDING.md); it has not been run on a device.

## Thanks

**The MAME project** — public documentation of these boards, down to chip
identification, bus wiring and undocumented registers, is what makes an
independent implementation practical, and is the reference used here.

**ElSemi** — Nebula Model 2 established much of the current understanding of
the geometry pipeline and rasterizer, and widescreen mode follows it.

Neither is affiliated with this project. Any inaccuracy is this program's own.

## License

MIT, in [LICENSE](LICENSE). ROM images are copyrighted by their publishers and
none are included here.

The YM3438 register/timer and synthesis adaptation retains Aaron Giles' YMFM
[BSD-3-Clause notice](LICENSES/YMFM-BSD-3-Clause.txt). Its current scope and
reference checks are documented in [Model 1 audio](docs/MODEL1_AUDIO.md).

## Logging

Off by default, addressable per subsystem:

```sh
RUST_LOG=info ./target/release/tgpulse vf2
RUST_LOG=warn,geo=trace ./target/release/tgpulse vf2
```

Targets include `geo`, `fifo`, `io`, `sound`, `copro`, `nvram`, `backup`,
`comm`, `library` and `video`.
