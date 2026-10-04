# NetMerc — advanced I/O and first boot checkpoint

Status: 2026-10-03, partial. The real I/O firmware runs on the advanced board,
and automated cabinet-input probes reach a rendered 3D scene. The bounded
factory-SRAM/service persistence and DPRAM handshake audit is complete below.
This is not complete gameplay, physical HMD tracking or peripheral validation.
Current tracking scope: the protocol-level serial peer and desktop stick/sensor
input below are implemented; i386SX/magnetic sensor emulation is not.

## Scope and reference

- Reuse `model1io2`, the consolidated Z80, 315-5338A, CTC/SIO/PIO, ADC,
  EEPROM and DPRAM already used by Wing War.
- Load `epr-18021.6` through `roms_db.dat`, with the generator updated too.
  Firmware: 65536 bytes, SHA-1 `bf5b9aad99c0f8f5e262e0855796f39119d11a97`.
  The previous database excluded this firmware and selected board 1 for NetMerc.
- Keep ADC0 = Stick X, ADC2 = Stick Y; disconnected channels float to `FF`.
  Existing frontend input routing supplies these pins; no additional binding
  catalogue or host-input dependency is introduced.
- Reference: MAME `src/mame/sega/model1.cpp`, `model1io2.cpp`, and
  `src/devices/video/hd44780.cpp`. The runtime comparison used the existing
  local MAME `0.289 (mame0289-1171-g6c504efbee8)` executable. These are
  implementation references, not proof of complete physical hardware fidelity.

## Diagnostic LCD

### Desktop Presentation — 2026-10-03

Settings exposes **Diagnostic Display**: Off (default), Overlay,
Dedicated Window. The board still owns all LCD state; the frontend reads
`IoBoard::diagnostic_lines()` after emulated advancement. Fullscreen replaces
the dedicated window with the overlay without rewriting the saved mode;
leaving fullscreen recreates the auxiliary window. Closing that window selects
Off and saves the preference; no game loaded/no LCD closes it automatically.
The main-window overlay stays visible with F1 or fullscreen and does not capture
input. The auxiliary window never routes keys/mouse into the emulated cabinet.

The four corner choices have a margin scaled to the GUI font size. Background
opacity is 0–100%; text remains opaque and aligned in fixed character cells.
HD44780 custom/non-ASCII character codes are shown as a placeholder, not a
claim of character-ROM/CGRAM fidelity. The blit/UI rendering path is reused;
the auxiliary ImGui context is activated only while drawing and suspended
afterward, preserving the main context's input/navigation and DPI state.

Persisted desktop-only keys:

```ini
diagnostic_display = off
diagnostic_position = top_right
diagnostic_opacity = 80
```

Mode keys: `off`, `window`, `overlay`. Position keys: `top_left`, `top_right`,
`bottom_right`, `bottom_left`. Invalid values warn and retain defaults.
No SDL object, window, host path or UI preference enters the core/snapshot;
save/load naturally refreshes the panel from restored LCD state on the next
frontend update.

Verification: offline workspace tests, mode/position/opacity configuration
round-trips, four-corner draw data, fullscreen preference policy, auxiliary
context switching/input preservation, and release build (514 workspace tests
passed). Isolated NetMerc window-mode and fullscreen-overlay launches ran
without renderer errors. Computer Use cannot identify
the unbundled Rust executable here, so no screenshot or interactive visual
acceptance is claimed; user checks remain for readability, window/fullscreen
transitions and live save/load refresh. Personal NVRAM/settings were untouched.

### Hardware State (Earlier Checkpoint)

The previous isolated firmware probe stopped at instruction `03A9` on a CN6
diagnostic-LCD write. The board now owns an in-memory HD44780 write-side model:
DDRAM/CGRAM, address counter, function/entry/display controls, display shift,
and incomplete four-bit transfers. Port-E data and the port-F CN6/E/RW/RS
qualification follow the reference board. EEPROM DI/CLK/CS updates retain their
existing clock-before-data/select ordering on every port-F write.

This board provides no LCD readback path. Busy timing, font ROM rasterization,
blink animation and a desktop LCD panel are outside this checkpoint. Visible
character codes can be read through `diagnostic_lines()` without GUI resources.
The original MAME source's port-write qualification is retained; this does not
claim electrically timed enable-edge accuracy.

Command behavior was also checked against the [Hitachi HD44780U datasheet,
pp. 26–27](https://www.sparkfun.com/datasheets/LCD/HD44780.pdf). Display-shift
direction follows the documented visible movement. Return home preserves entry
direction; clear selects increment without changing S; CGRAM transfers do not
shift the display. These are deliberate differences from the inspected MAME
implementation, justified by the manufacturer's command description rather
than by matching software output alone. The actual NetMerc firmware text agrees
with MAME in the sampled startup sequence.

LCD state is part of board snapshots, including a pending high nibble. Board
identity is validated before restore so Wing War/NetMerc/R360 states cannot be
cross-loaded through the shared advanced-device implementation. Complete Model 1
save-state format is now **2**; format-1 files are rejected explicitly, without
mutating the machine. Existing NVRAM files and the Model 2 state format are
unaffected. New states restore the LCD and DPRAM directly, without replaying
hardware writes or writing persistent NVRAM.

## Unintended desktop rumble at startup — 2026-10-03

The startup vibration was not evidence of a working NetMerc motor adapter.
The desktop routed every non-VR Model 1 game's `drive_cmd` through the old
Daytona-derived intensity fallback. On advanced I/O, this latch is port E,
which is also CN6's diagnostic LCD data bus. NetMerc's firmware writes display
commands/characters there even with no cabinet buttons pressed. Sampled cold
boot bytes include `61`, `2A` and `6D`; the fallback turned them into gains
`1/7`, `2/7` and `5/7`, respectively. Out-of-range bytes did not stop the
previous effect. This explains unrelated and potentially sustained pad buzz.

The source wiring independently confirms the mismatch: MAME's
`model1io2_device::io_pe_w` stores LCD data and invokes the optional drive
callback, but the NetMerc machine does not connect that callback. Its
`netmerc_outputs_w` is connected to port D instead; documented bit 2 controls
the trigger/thumb motor, alongside unrelated backlight/holder/coin outputs.
This is a wiring/protocol diagnosis, not a claim that MAME fully emulates that
physical motor or proves its polarity and gamepad intensity.

The desktop now explicitly sends zero for NetMerc in the legacy rumble path,
both during normal emulation and after loading a machine state. VR/VFormula,
Model 2 and the other existing legacy paths are unchanged. The change is
backend-independent (SDL3/gilrs), does not mute sound, and leaves the board's
output latches, LCD, inputs and serialized device state intact. The true port-D
motor is connected by the separate adapter described below; no effects are invented from LCD
traffic, button presses, audio or game motion. Other unaudited legacy paths
remain outside this bounded NetMerc correction.

### Binary Motor Adapter — 2026-10-03

The core exposes `model1board::IoBoard::netmerc_motor()` as an optional boolean
derived directly from advanced-board port D bit 2 (`outputs.lamps`). No host
controller, gain, new mutable state or snapshot-version change enters the core.
The desktop handles this output before the legacy drive decoder and uses the
existing SDL3/gilrs rumble path: both pad motors at the existing Model 1 ceiling
of 0.6 while asserted, zero while cleared. This motor/gain choice is a frontend
approximation, not a measured cabinet force. No extra pulses, envelopes or
effects are inferred from weapon inputs, audio, LCD traffic or game motion.
Pause explicitly stops this output; close/backend change and state loading
reuse existing stop/reset paths. The restored port latch takes effect on the
next emulated frame, not during a paused load.

Evidence for active-high polarity is not just MAME's port annotation:
the original service routine at V60 `FDCFA7..FDCFF4`, with MOTOR selected,
sets bit 2 and writes it to `C0001E` when Thumb or Trigger is pressed, otherwise
writes zero. In a fresh isolated firmware/gameplay probe, boot and released
inputs remained off. Holding Thumb for 300 frames after the documented start
sequence yielded 39 asserted frame samples and six output transitions; the
following 120 released frames were off. Trigger alone did not assert it in this
particular gameplay sequence. This is evidence of firmware-driven output,
not proof of every weapon/scene or the physical motor response. The controller
test probe did not successfully demonstrate a MOTOR-row pulse and is not
counted as runtime service-test acceptance.

A bounded wiring test checks D-bit decoding independently of LCD port E,
board-family gating and snapshot/continuation equality. Captured probe evidence:
`/private/tmp/tgpulse-netmerc-motor-check.log`. Physical gamepad feel remains
unverified; the frame-sampling audit below closes the bounded pulse-loss check.

#### Motor Pulse Sampling Audit — 2026-10-03

The opt-in `model1_motor` trace observes actual port-D bit-2 transitions at the
advanced Z80 bus write boundary, including output-direction changes. It records
the existing elapsed board clock, not host time. Timestamps have instruction-
boundary precision; they do not claim sub-instruction bus-cycle accuracy. LCD
port E and unrelated output bits produce no motor transitions. Trace collection
adds no emulated state, event queue or save-format change.

Reproduce the bounded real-ROM capture with:

```sh
RUST_LOG=model1_motor=trace cargo run --offline --release -p tgpulse \
  --example netmerc-motor -- roms/netmerc.zip > /tmp/netmerc-motor.log 2>&1
```

The example uses a fresh in-memory cabinet and the loader's factory NVRAM path,
without loading/saving personal NVRAM or opening audio/controller devices. It
runs 3,368 frames through boot, coin, Holder, game start, held Trigger, held
Thumb, both buttons and 60 alternating one-frame Thumb presses/releases.
`BEGIN`/`END` rows identify every frame and its sampled latch/elapsed clock;
trace rows capture transitions between those samples. This allows each asserted
interval to be compared with the frame samples used by the desktop adapter.

| Input Phase At Assertion | Pulse Duration (ms) | Asserted Frame Samples |
| --- | ---: | ---: |
| Thumb held | 226.7 | 13 |
| Thumb held | 207.7 | 12 |
| Thumb held | 208.7 | 12 |
| Both held | 5,218.1 | 300 |
| One-frame Thumb press | 210.7 | 12 |

There were ten transitions/five complete asserted intervals, no missed pulses
and no disagreement between edge-derived latch state and any final frame sample.
Boot/released inputs stayed off. The one-frame input generated a roughly 211 ms
motor interval in the firmware itself; the desktop did not synthesize that
duration. The capture ended with the motor off. No pulse shorter than a frame
occurred in this scenario, so there is no demonstrated loss justifying a rumble
buffer or artificial extension. Existing frontend sampling is retained.

This closes the audit for the replayed sequences, not every scene, operator
configuration or physical controller response. Diagnostic timing does not prove
host motor latency or force. Private evidence is
`/private/tmp/tgpulse-netmerc-motor-edges-clean.log` and
`/private/tmp/tgpulse-netmerc-motor-analysis.txt`; the example is the reusable
source runner. Other Model 1/2 rumble adapters are unchanged.

Verification: 519 offline workspace tests passed with no failures after allowing
the existing loopback TCP tests to bind local sockets outside the sandbox.
Formatting/diff checks and the offline development release build passed. This
is automated core/loader/build evidence; physical pad response remains a user
acceptance check.

Verification: a 120-frame headless cold boot reproduces the sampled LCD bytes;
an exhaustive unit test excludes all 256 possible NetMerc latch bytes from
the legacy decoder. A separate test
preserves the existing legacy command gains/ignore behavior for other sets.
Final offline workspace suite: 507 passed, no failures (local TCP tests run
with host permission).
Formatting, diff checks and the development release build pass. The rebuilt
headless desktop completes the 1,868-frame cabinet startup sequence without
an I/O fault. The desktop Library additionally provides an ImGui model-filter
button; tests cover all five board families, unknown sets under All, hidden
selection invalidation and Refresh retention. Manual gamepad and GUI acceptance
of the rebuilt desktop remain user tests.

## HMD power-on pose

NetMerc's constructor supplies the same initial HMD position/orientation as
`netmerc_state::machine_reset`: six little-endian signed words
`0, 0, 0, 12868, 25736, 12868` at DPRAM `80..8B`. This selects a stationary
view facing forward. The old all-zero pose produced an incorrectly oriented
camera once the game reached 3D. The seed is cold-boot state only; restoring a
saved pose does not replace it with defaults.

This does **not** implement the Polhemus i386SX board, serial tracking protocol,
live HMD movement or any successful-ready/acknowledge shortcut.

## Verification and evidence boundaries

The headless desktop debugger and pure-core probes use the existing user-owned
ROMs, fresh temporary working directories and shipped factory SRAM. No user
NVRAM, ROM ZIP, installed release, MAME checkout or toolkit file is modified.

- Before the forward-pose seed, 1800 NetMerc frames run without a latched I/O
  fault. The initial WARNING screen persists with no cabinet activity; merely
  sampling the V60 around `FC095B/FC0960/FC0968` does not establish a CPU deadlock.
- At 300 and 900 frames, MAME and TGPulse show the same initial WARNING screen
  and the same diagnostic text: `(pramater display)` and respectively
  `0240 0000 0000 0005` / `0240 0000 0000 000F`. The spelling is the firmware's.
  An early 60-frame LCD sample also agrees. These comparisons justify closing
  the unsupported-LCD fault, not all I/O timing or tracking behavior.
- One sampled DPRAM flag differs: byte `20` is `01` in MAME and `00` in TGPulse
  at frame 900. The follow-up below identifies the changing request/acknowledge
  cycle; the single frame-boundary sample was not evidence of a stuck board.
- Cabinet sequence: after 900 frames, Coin low for 4 frames, then release and
  300 frames; MVD Holder low for 60 frames, release and 300 frames; Trigger low
  for 4 frames, release and 300 frames. A 3D scene appears in this probe.
  This is automated input/render evidence, not a human gameplay acceptance test.
  After the forward-pose initialization, TGPulse and a fresh MAME instance
  reach the same tunnel/weapon scene at frame 1868 with the same cabinet
  sequence. The camera now faces forward. This is scene-level agreement,
  not pixel equality, successful aim/movement or a tracking-emulation claim.
- Holding Test after startup reaches `BAD DATA CHECKSUM / RAM NOW CLEAR /
  PUSH TEST TO EXIT`. This also occurs in a fresh reference run with the shipped
  SRAM. Initialization and restart persistence were subsequently checked below;
  no factory-ROM patch or synthetic checksum bypass was necessary.
- Real-firmware NetMerc and Wing War snapshots at frame 300 restore into fresh
  systems and continue for 120 frames with identical encoded machine state and
  audio samples. Synthetic tests separately cover mid-nibble LCD continuation,
  CN6 write qualification, shared EEPROM pins, board-variant rejection and
  preservation of a nondefault HMD pose during restore.
- Wing War reaches the attract scene at frame 900 with no device fault.
- Final gates: `cargo test --offline --workspace` passes 498 tests (local TCP
  tests run with the required host permission); `cargo fmt --all -- --check`,
  `git diff --check`, and `cargo build --offline --release -p tgpulse` pass.
  The regenerated NetMerc ROM record matches the checked-in database. Existing
  dependency warnings remain outside this change. Runtime checks and save/load
  probes were repeated on the final forward-pose implementation.

For manual gameplay tests with default bindings, insert Coin, press/release
`NetMerc: MVD Holder` (Start or D-pad Down / L), then `NetMerc: Button 1` (South / J / E /
Space, the Trigger). The automated sequence establishes one successful startup
path; whether MVD Holder is necessary, and how the physical holder switch should
behave through later games, remain operator/gameplay checks. User-rebound
controls take precedence over these defaults.

The debugger now accepts `regs io` on advanced Model 1 boards, reporting board
kind, Z80 PC/elapsed clocks, fault and both diagnostic text lines. Example from
an isolated working directory:

```sh
/Users/andrea/dev/TGPulse/target/release/tgpulse --debug \
  /Users/andrea/dev/TGPulse/roms/netmerc.zip \
  -c 'input in1 255; input analog0 127; input analog1 255; input analog2 127; input analog3 255; run 900; state; regs io'
```

## SRAM/service and mailbox audit — 2026-10-03

### Factory data and persistent restart

The same cabinet sequence was repeated with independent temporary NVRAM on both
sides: run 300 frames, hold Test for 300, release for 60, press Test for 4 and
release for 296, then press Test for 4 and release for 296. At frames 600, 960
and 1260, both sides respectively show the checksum-clear notice, OUTPUT TEST,
and the main TEST MODE menu. All three screenshots match pixel-for-pixel at
496 x 384 (190464 pixels each). The final MAME runs use `-noreadconfig`,
`-nocheat`, `-noplugins`, isolated cfg/NVRAM/snapshot/diff paths and native-size
snapshots; no external configuration or cheat is required for this result.

TGPulse's `nvram save` writes only the temporary `nvram/netmerc.nv`. A fresh
machine decodes and reloads its complete 65536-byte SRAM and 128-byte EEPROM
exactly. Entering Test after a cold restart shows the normal menu, rather than
repeating the checksum-clear notice. A native-resolution comparison of the
initialized TEST MODE menu matches MAME pixel-for-pixel. The sampled tile RAM,
character RAM, palette and colour-translation RAM also agree byte-for-byte.
The cross-run SRAM comparison differs only at offsets `04..05`, a changing
runtime field; this audit does not assign that field a hardware meaning.

Conclusion of the unpatched audit: the shipped factory SRAM requires this
operator initialization in both implementations. It is not a TGPulse persistence
or renderer defect. The subsequent ROM-default-only initialization policy below
avoids this first-use notice without changing ZIPs, personal SRAM loading or
the game's own checksum/clear logic. This validates the tested restart path,
not every operator setting, bookkeeping value or service submenu.

### BAD DATA CHECKSUM: exact cause and original ROM algorithm

The main V60 program (`epr-18120.ic5`) performs this check when entering
service mode, not as a ROM audit. At `FDBD0C` it calls `FDD847`; a nonzero
result calls the game's clear routine `FDD4F3` and selects service page `0E`
(`BAD DATA CHECKSUM / RAM NOW CLEAR / PUSH TEST TO EXIT`). A zero result
continues to the ordinary TEST MODE menu. This identifies the cause without
treating agreement with MAME alone as a correctness oracle.

`FDD801` selects the active bookkeeping bank from SRAM byte `400000`: value
`0F` selects `401000..401FFF`; any other value selects `402000..402FFF`.
`FDD847` sums 1024 little-endian 32-bit words, modulo 2^32, and subtracts the
32-bit stored checksum at `403000`. The check passes exactly when the result
is zero. The game maintains two alternating bookkeeping banks; the checksum
does not cover the entire 64 KiB SRAM, EEPROM, ROMs or HMD tracking state.

The supplied `netmerc_nvram.bin` contains `FF` everywhere except offsets
`0038` and `0040`, which are `00`; neither exception belongs to the checked
banks. Its selector is `FF`, all 1024 words of the selected bank are
`FFFFFFFF`, their sum is `FFFFFC00`, and the stored checksum is `FFFFFFFF`.
The result is therefore **`FFFFFC01`**, not zero. The checksum is invalid in
the supplied starting image itself, before emulation or cabinet input.
The author describes the image as a handcrafted gun-calibration seed in
[MAME PR #15642](https://github.com/mamedev/mame/pull/15642), not a fully
initialized bookkeeping capture. Official MAME 0.289 and master declare it
as a required `ROM_LOAD` in the default `nvram` region. This is not evidence
of a physical ROM/board fault.

`FDD4F3` clears both 4 KiB banks (`401000..402FFF`), writes selector `0F`,
sets the stored checksum to zero and resets additional cabinet/bookkeeping
fields. This is original game behavior, not an emulator initialization bypass.
The repaired banks then sum to zero. Normal SRAM persistence retains this
result across the tested cold restart.

An isolated real-ROM TGPulse probe confirms the return value at V60 PC
`FDD862`, immediately after the checksum subtraction:

| Input SRAM | Calculated and actual V60 R0 | Service page | Check after game's clear |
| --- | --- | --- | --- |
| Supplied factory image | `FFFFFC01` | `0E` (checksum-clear notice) | Valid |
| Previously initialized SRAM | `00000000` | `00` (normal menu) | Valid |
| Initialized SRAM with one byte at offset `1200` flipped from 0 to 1 | `00000001` | `0E` (checksum-clear notice) | Valid |

All three inputs retain their expected checksum through 300 startup frames
before Test is pressed. The corruption case uses only an in-memory copy;
no personal SRAM or ROM is changed. A read-only check of the existing local
`nvram/netmerc.nv` also finds the same `FFFFFC01` result as the factory image;
this is a diagnostic snapshot, not a guarantee about later user edits or
another launcher's configured NVRAM path.

Evidence is retained outside Git under
`/private/tmp/netmerc-checksum.q6csXe`: ROM disassembly `checksum-code.txt`,
`service-code.txt`, isolated probe `probe.rs` and its result
`tgpulse-checksum.log`. No production code or persistent user data changes are
required for this diagnosis. Let the game's own Test-mode initialization run,
then exit normally so the desktop saves SRAM. A checksum notice that repeats
after a verified initialized save/load would be a separate concrete fault.

### Supplied gun calibration: exact service-menu values — 2026-10-03

The seed concerns **TEST MODE -> CONTROLLER UNIT TEST**, not HMD tracking.
The following mapping is confirmed by the main ROM's service draw/copy
routines (`FDCD43..FDCE7C`) and a real-ROM desktop-debugger run entering the
page through Test/Service input, without writing NVRAM. Values shown by this
menu are **hexadecimal ADC endpoint values**, not percentages or angles.

| Service-menu item (exact label) | SRAM byte offset | V60 address | Supplied value | Menu display |
| --- | --- | --- | --- | --- |
| `CONTROLLER LEFT MIN` | `003C` | `40003C` | 255 | `FF` |
| `CONTROLLER RIGHT MAX` | `0038` | `400038` | 0 | `00` |
| `CONTROLLER UP MAX` | `0040` | `400040` | 0 | `00` |
| `CONTROLLER DOWN MIN` | `0044` | `400044` | 255 | `FF` |

Only bytes `0038` and `0040` differ from the original all-FF image. Thus the
file supplies a full 8-bit endpoint pair for each gun/controller axis, with
the left/down endpoint at FF and right/up endpoint at 00. Keep the game's
labels and polarity as observed; do not swap values to make the words MIN/MAX
look like an ascending numeric interval. The gameplay conversion at
`FBE75C..FBE875` reads these four **low bytes**, orders each pair for its
calculation and uses their span and midpoint to scale the raw ADC positions.
This explains why equal FF endpoints would not provide a usable range.

The storage slots are four bytes wide: the supplied bytes are respectively
`FF FF FF FF`, `00 FF FF FF`, `00 FF FF FF`, `FF FF FF FF` in menu order.
Do not misreport the 32-bit words (`FFFFFFFF` or `FFFFFF00`) as the displayed
calibration values. The menu copies full slots into working RAM, but renders
only their low-byte hexadecimal value. In this run all four slots remain
unchanged on entering the menu; no automatic replacement with the ROM's
other hardcoded fallback endpoints occurred.

Selecting an endpoint allows the original service code to capture the current
axis reading into its working value. On EXIT, `FDCF32..FDCF56` writes the four
working slots back to SRAM. This documents the ROM path, not a new test of
physical controller calibration accuracy or all possible endpoint values.

Other labels on this page are **not persistent calibration supplied by this
image**:

- `POSITION: L/R= ... U/D= ...`: current ADC readings. Both show `7F` in the
  isolated run because the test supplies centered ADC0/ADC2 values of 127.
- `MOTOR: THUMB` and `TRIGGER`: current button/output test indications, not
  additional stored axis endpoints. Both show OFF with no button pressed.
- `MVD TEST`: a separate tracking/pose diagnostic. No additional HMD
  calibration data is established by this two-byte seed; the forward pose
  discussed above is initialized separately in volatile DPRAM.

The earlier bookkeeping patch modifies only byte `0000` and `1000..3003`,
so these four calibration slots are unchanged both in the original ZIP image
and in the regenerated personal NVRAM checked in this audit. The original
factory image and current personal file were read only. Menu script, register
dump and native 496 x 384 screenshot are retained outside Git in
`/private/tmp/netmerc-calibration.6jrIIN` (`menu.dbg`, `menu.log`,
`controller.ppm`, `controller.png`); the ROM disassembly is in
`/private/tmp/netmerc-checksum.q6csXe/main-code.txt`. No emulator code, ROM,
personal NVRAM, release or other project is changed by this documentation
checkpoint.

### ROM-default-only initialization — 2026-10-03

At the user's request, the ROM loader now initializes the bookkeeping portion
of the known `netmerc_nvram.bin` **in memory**, alongside the existing legacy
`315-5711.bin` correction. Recognition requires NetMerc board identity, exactly
65536 bytes and SHA-1 `411134c1e6307f2e32c3b4b372597b45b14a9834`.
Unknown, modified or already initialized images remain untouched.

Only offsets `1000..3003` (both bookkeeping banks and stored checksum) are
zeroed, and selector byte `0000` becomes `0F`. Every other byte, including
the gun-calibration fields, remains identical to the supplied default.
This deliberately implements only the bookkeeping part of `FDD4F3`, not its
additional operator-field resets. It is a first-use convenience policy, not
an emulation fidelity fix or a general checksum-repair facility.

Persistent SRAM loading and complete machine snapshot restore remain exact;
neither applies this patch. A saved personal/default NVRAM takes precedence
over the ROM-set seed, even if its checksum is invalid. The original game
still detects and clears a real bookkeeping corruption through service mode.
No new frontend, filesystem or wall-clock dependency is introduced into the
hardware core, and the ROM ZIP is never rewritten by this mechanism.

Verification: the workspace suite passes 502 tests (local TCP tests need host
socket permission), formatting/diff checks and the offline development release
build pass. Synthetic coverage checks full-image recognition, mutation bounds,
idempotency, other board families, saved SRAM/EEPROM precedence and exact
snapshot restore. The real-ROM probe observes checksum return `R0=0` and
normal TEST MODE for the patched seed; an intentionally changed bookkeeping
byte still returns `R0=1` and enters the game's checksum-clear page.

The user authorized removal of the existing active `nvram/netmerc.nv`. It was
moved to a temporary recovery copy outside Git, then recreated by starting
the final desktop debugger binary and saving through its existing NVRAM API.
The new 65536-byte SRAM/128-byte EEPROM container has a valid checksum;
first-boot and fresh-process restart TEST MODE renders are byte-identical.
The ZIP member's original SHA-1 is unchanged. A further automated cabinet
sequence reaches the 3D scene without a device fault; this remains a smoke
test, not manual gameplay or HMD-tracking acceptance.

Runtime evidence and the removed-file recovery copy are under
`/private/tmp/netmerc-default-install.cOoXdc`; no ROM, personal NVRAM or
diagnostic artifact is included in Git. No commit, push or release deployment
is performed by this checkpoint.

### DPRAM 20: changing mailbox, not a constant ready flag

Read-only MAME memory taps identify two writers: the V60 submits `01`, and the
Z80 firmware writes `00` through the 315-5338A host transfer at PC `150C`
(the MAME callback observes PC `150F` after decoding that instruction). An
isolated board restored from an actual frame-300 snapshot consumes an injected
request and clears it through that instruction. The injected request is test
stimulus only; it is never applied by normal emulation.

Sampling TGPulse in 64-V60-clock slices across frames 300..304, including each
real vblank trigger, observes a complete `00 -> 01 -> 00` exchange every frame.
MAME taps around the same frames also observe both writes. Its frame callback
samples `01`, whereas TGPulse's completed-frame sample sees `00`; the earlier
comparison sampled different phases of an active handshake. This does not
establish identical sub-frame latency, but it rules out the proposed permanent
missing-ready condition. Do not force the byte to MAME's sampled value or alter
the scheduler merely to match that instant. Hardware-cycle fidelity would need
additional evidence of an incorrect observable result.

The existing NVRAM-container regression now covers NetMerc alongside the original
and Wing War boards. It checks all EEPROM words and SRAM endpoints, and that a
cold NVRAM reload does not import the previous session's volatile mailbox/HMD
pose. Real-firmware probes remain separate from synthetic unit tests; no ROM or
runtime NVRAM is checked into Git. This checkpoint adds regression coverage and
evidence, not a production hardware workaround.

## HD44780 Dot Rendering — 2026-10-03

The diagnostic presentation has independent location and rendering settings:
`diagnostic_display = off/overlay/window` (default Off) and
`diagnostic_rendering = hd44780/text` (default HD44780). Fullscreen still uses
the overlay for the Window preference, without altering saved settings.

The desktop reads optional `hd44780_a00.bin` from `netmerc.zip` first, then
the adjacent `hd44780.zip` MAME device BIOS. It checks the 4096-byte size and
SHA-1 `65cf075a988cdcbb316b9afdd0529b374a1a65ec`; missing/invalid resources
fall back to Text without blocking boot. The requested local BIOS was copied
beside `roms/netmerc.zip`; neither ZIP contents nor font bytes are committed.

Reference: MAME `src/devices/video/hd44780.cpp` (`render`, `screen_update`)
and `src/mame/sega/model1io2.cpp` (`lcd_pixel_update`, `lcd_palette`). Model 1
uses original HD44780 A00, not HD44780U A00/A02. MAME marks this font BAD_DUMP:
it is reconstructed from the 1985 datasheet, not a verified silicon ROM dump.

The existing LCD state produces palette-index pixels at 121x19: 20x2 cells,
5x8 dots, one-dot border and gaps. Codes 0..15 select the eight CGRAM glyphs
(including aliases 8..15). Display shift/off and cursor/blink follow the
reference renderer. Blink phase uses already serialized board elapsed clocks
with the reference's typical, unmeasured 270 kHz LCD oscillator and 102400-cycle
phase period. No host timer, font resource or new mutable state enters snapshots.
Frontends provide CGROM in memory and may consume the same read-only raster;
the desktop uses square primitives at integer scale, shared by both displays.

Verification: 517 workspace tests pass, including CGROM/CGRAM aliases,
one-/two-line geometry, shift, cursor/blink, display-off and restored output.
Settings round-trip and both GUI render paths are covered. An isolated resource
probe verified embedded-font precedence, adjacent BIOS lookup, missing-font
Text fallback and recovery from a corrupt game archive using the valid BIOS;
the actual local A00 font produces the expected five-bit rows and panel border.
Fixtures are under `/private/tmp/tgpulse-lcd-font-check.x3Zw6j`; the probe uses
only resource copies, not user ROM modifications, settings or NVRAM writes.
The device BIOS is excluded from the game library. No live visual/gameplay
acceptance or complete hardware-font equivalence is inferred from these checks.

## HMD tracking contract audit — 2026-10-03

This audit checkpoint changed documentation, not the production device or bindings.
It identifies the actual game/firmware interface before choosing a tracking
implementation. The six-word power-on seed was the behavior at this checkpoint.
User gameplay acceptance, live head tracking and the Polhemus board are still
unimplemented/unverified; successful synthetic serial input is not a physical
tracker or gameplay certification.

### What the main program consumes

The normal main loop at `FFE303` calls `FC250B` once per iteration, after the
cabinet-input routine. It reads twelve DPRAM bytes as six signed little-endian
16-bit values. DPRAM bytes occupy the low byte of successive V60 16-bit lanes:
offset `n` is read at `C00000 + 2*n`, not at consecutive V60 byte addresses.
The separate service routine at `FDCA3C` consumes the same values for MVD TEST
page 1. Its own labels identify them as follows:

| DPRAM bytes | V60 low-byte addresses | MVD label | Normal-game destination/conversion |
| --- | --- | --- | --- |
| `80..81` | `C00100`, `C00102` | XPOS | Raw word at `513472`; world component at `516112` = `raw * g` |
| `82..83` | `C00104`, `C00106` | YPOS | Raw word at `51346E`; world component at `51610A` = `raw * g` |
| `84..85` | `C00108`, `C0010A` | ZPOS | Raw word at `513470`; world component at `51610E` = `-raw * g` |
| `86..87` | `C0010C`, `C0010E` | XANG | `513474` = `raw * 32767 / 25736` |
| `88..89` | `C00110`, `C00112` | YANG | `513476` = `-(raw * 32767 / 25736)` |
| `8A..8B` | `C00114`, `C00116` | ZANG | `513478` = `-(raw * 32767 / 25736)` |

`g` is the ROM's single-precision constant at `FA3728`, printed as
`0.050758384`; MVD uses the same constant at `FDDF0C`. Orientation uses signed
integer multiplication/division and stores the final low 16 bits. Service mode
additionally divides the angular conversion by `182` and applies its display
wrap logic; do not confuse its displayed angles with raw ADC/serial words.
The divisor `25736` agrees with the existing 180-degree seed, independently of
MAME's comment. Physical axis mounting, position units and the alternate
routine at `FC2656` remain to be established; that routine changes orientation
signs and was not the sampled normal-loop call.

Two isolated real-game probes delivered in-memory DPRAM poses and let the V60
continue normally; no PC, calculation result, ROM or saved NVRAM was patched:

| Delivered pose | Game orientation words | Game position components (`51610A`, `51610E`, `516112`) |
| --- | --- | --- |
| `[4, -8, 120, 12868, 25736, -12868]` | `[16383, -32767, 16383]` | `[-0.40606707, -6.0910063, 0.20303354]` |
| `[16, 24, -40, -12868, 0, 12868]` | `[-16383, 0, -16383]` | `[1.2182012, 2.0303354, 0.81213415]` |

These results assert the byte order, signedness, permutations and conversions
against the executed firmware, not just a disassembler prediction. They do not
establish the comfort/range of future desktop head controls.

### Firmware selection: board jumpers, not just DIP switches

At I/O ROM `0213`, the Z80 reads board buttons/jumpers at **`8040`**, complements
the byte and masks `30`. This is not the DSW1 register at `8080`. The shared
board currently supplies `board_switches = 3F`, leaving both jumpers inactive;
the firmware consequently leaves tracking enable `F0B2` at zero. The main-board
wrapper always supplies these defaults, so changing a frontend DSW alone would
not activate tracking.

| Board pins 5:4 | Full probe board-switch byte | DSW1 bit 7 | Firmware mode `F0B0` | Startup bytes on SIO A |
| --- | --- | --- | --- | --- |
| `11` | `3F` | Either | Disabled (`F0B2 = 0`) | None |
| `10` | `2F` | 1 | 0 | `63 53 75 66 6D 43` (`c S u f m C`) |
| `10` | `2F` | 0 | 2 | `63 53 75 66 4B 43` (`c S u f K C`) |
| `01` | `1F` | 1 | 1 | `02 5A 45 03` |
| `01` | `1F` | 0 | 3 | `02 5A 45 03` |

Both active-low jumpers enabled (`00`) select neither tracking branch at boot;
the board's separate debug-entry procedure uses that combination together with
board button 0 held at reset. It is not an additional tracker format.
The inspected MAME board names bit 4 JP4/ROM_EMU and bit 5
JP3/MODE; these generic names are not proof of NetMerc's actual cabinet setting.
All probe bytes preserve inactive board buttons 0..3. DSW1 is `FF` or `7F` in
the active-mode rows, without changing the baud-rate bits.

`F0B2 = 1` means the startup sequence is in progress, `2` means its command
table has finished. This is internal firmware state, not an external tracker
ready acknowledgement. The sequence advances via the real timer ISR and its
countdown (`15000` for modes 0/2, `5000` for modes 1/3; `250` between commands).
The table's `40` sentinel terminates the sequence and is **not** transmitted.
The serial probe captured all four command sequences from actual TX pin edges
over 180,000,000 emulated board clocks, rather than reading only the ROM table.

### Serial framing and receive formats

Tracking arrives on **SIO channel A / CN7**, data/control ports `18/19`.
Channel B/CN8 is separate. The firmware programs async 8N1 with an x16 SIO
clock. CTC channel 2 and DSW1 bits 0..1 select 38400/19200/9600/4800 baud;
DSW1 `FF` gives 38400, or 256 board clocks per serial bit at 9.8304 MHz.
The existing SIO can service this without a new serial or Z80 implementation.

The RX interrupt handler at `04DF` calls `108D`, then selects one of two parsers:

- **Modes 0/2 (`10A3`)**: a record buffered at `F400` with 7-bit data bytes and
  overflow-bit maps. Pose low-seven-bit bytes are read from record offsets
  `3,4,5,6,8,9,10,11,12,13,14,16`; their high bits come from offsets `7,15,19`.
  Those positions fit a 20-byte encoded record. Header/trailer bytes are not
  validated by this pose parser; accepting a synthetic header does not prove
  it is the correct physical tracker response.
- **Modes 1/3 (`1177`)**: twelve pose bytes, two 7-bit bytes per word.
  Each reconstructed word is `((lo & 7F) | ((hi & 7F) << 7)) << 2`, retaining
  the low 16 bits. Thus the signed 14-bit payload becomes a signed 16-bit value
  with two low zero bits. Do not assign this branch a manufacturer/device name
  solely because other trackers have a similar encoding.

In both branches, bit 7 marks the first byte of a new record. The handler
decodes the **previous** buffer before storing that new first byte. A complete
pose therefore needs the next record's marker before it becomes visible.
There is no complete-record length or header validation in these inspected
handlers; the first marker also decodes the initially cleared buffer. A future
endpoint must stream correctly framed records, not send one pose and stop.

The Polhemus-authored [ISOTRAK II manual (2001 rev. A), pp. 29–31 and command
index](https://manualzilla.com/doc/6857117/3space%C2%AE-isotrak-ii%C2%AE-user-s-manual)
(archival mirror) documents continuous-binary overflow-bit packing and the
`c/u/f/m/K/C` command family. This supports an ISOTRAK-family interpretation of
modes 0/2, not identification of the installed hardware revision or modes 1/3.
Modern FASTRAK floating-point formats are not interchangeable with this parser.

### Firmware publication and actual receive probe

Decoded pose words live at `F480..F48B`. On the ordinary input request,
`12F3` returns immediately when tracking is disabled. Otherwise it copies all
twelve bytes, with interrupts disabled, to `F4C0..F4CB`, reenables interrupts,
then writes that local snapshot to DPRAM `80..8B` through the existing 315-5338A
host transfer routine `1504`. The receive ISR is allowed to update the next
pose during these DPRAM transfers; this is not a newly invented atomic V60 bus.
No additional external-ready flag is checked by this publication routine.
Enabling the jumpers without delivering serial data would publish zeroed pose
RAM and replace the current forward-view seed; do not enable them unconditionally.

An isolated probe runs the unchanged real `epr-18021.6` firmware on the existing
advanced board and supplies timed RXD bits, not decoded words or register
patches. It tests modes 0 and 1 with the distinct signed pose
`[4, -8, 120, 12868, 25736, -12868]`. In each case:

1. The first complete record leaves decoded pose RAM zero.
2. The next record's marker makes `F480..F48B` equal the expected twelve bytes.
3. A simulated host supplies the ordinary `SEGA` signature and commands `2`
   then `1` through DPRAM, without changing firmware/CPU RAM or PC.
4. The board publishes the same pose at `80..8B` and clears mailbox `20` itself.

The probe also verifies all four mode selections and their startup TX sequences.
These are targeted real-firmware experiments, not additional speculative
workspace unit tests. Inputs, EEPROM and emitted poses are temporary/in memory;
user ROM ZIPs and persistent NVRAM are untouched.

### Implementation boundary and decision

The inspected MAME NetMerc configuration provides an i386SX at 16 MHz with RAM
and the interleaved `u1/u2` ROMs, but no Polhemus I/O map or serial link to CN7.
Copying that configuration alone would not deliver HMD motion. TGPulse does
not load/run those board ROMs yet. Physical sensor processing and its board
peripherals need additional evidence if full low-level emulation is chosen.

The existing advanced-board `set_serial_inputs()` and timestamped output
events provide a reusable integration boundary. The motherboard wrapper
currently discards those events and supplies no serial peer. Any future
protocol-level endpoint should exchange serial pins in emulated time, deliver
explicit pose inputs independently of GUI/gamepads, and serialize command,
record, in-flight bit and timing state. Keep persistent NVRAM separate.
It must be labelled a documented protocol-level model, not emulation of the
i386SX or its magnetic sensor processing. Do not bypass the real parser by
writing DPRAM every frame or silently repurpose the cabinet's aiming stick.

Confidence: high for the executed serial/pose contract and selection gates;
medium for the ISOTRAK-family identification; unresolved for physical mounting,
the other tracker family, exact device revision and hardware sample timing.

## Clocked serial measurement peer — 2026-10-03

`model1io2/tracking.rs` adds a **protocol-level endpoint**, not an i386SX CPU
or magnetic sensor simulation. It reuses the existing async 8N1 x16 `i8251::Uart`
for the external wire and the existing CTC/SIO for the I/O board. Only NetMerc
fits the peer on CN7 and selects JP4 on / JP3 off (firmware modes 0/2). Wing War,
R360 and CN8 retain their previous serial wiring. There is no decoded-pose
injection into DPRAM: the firmware's real RX ISR and transfer routine publish it.

The frontend-independent `HmdPose` contains six signed raw words: position
XYZ and orientation XYZ. `model1board::IoBoard::set_hmd_pose()` changes the next
measurement; it neither mutates an in-flight record nor repurposes aiming-stick
inputs. `tracking_status()` reports input pose, streaming, generated-record
count, last command, unsupported commands and UART errors. The default pose
matches the established forward-view seed. The firmware may publish zero pose
RAM during its startup command delay, before it enables the serial stream.

Implemented contract:

- `c`, `u`, `f`, `C` stop, select metric/binary mode and start continuous output.
  Output remains gated until binary and metric selections have been received.
- Default station-1 binary records contain the ASCII header `01 `, six LE
  signed words and CR/LF, with the documented overflow-bit packing and first-byte
  sync marker. The record is 20 encoded bytes, including the three MSB maps.
- DSW1 baud selection supplies matching emulated clocks at 38400, 19200, 9600
  or 4800. The external UART and board advance in bounded clock slices; no
  wall-clock timer, host callback, thread or SDL object belongs to the endpoint.
- Measurements use a 60 Hz cadence, supported by the ISOTRAK II manual, **not
  verified for this exact 1992 hardware revision**. A busy slow link discards
  intervening samples instead of growing an output queue. This is a bounded
  compatibility model, not a cycle-accurate magnetic tracker.
- `m`/`K` select physical quiet filtering in the documented device. This peer
  receives already-provided measurements and does not add artificial filtering.
- `S` identity/BIT output is **not implemented**: the installed device revision,
  firmware version and diagnostic contents are unknown. It is counted as an
  unsupported request, with no fabricated identity or healthy self-test reply.
  The actual I/O firmware proceeds to `C` without waiting for that response.
  Other commands and the unidentified modes 1/3 device are not implemented.

All mutable endpoint state, including input/latched packet, UART shift/holding
registers, pending bits, clock phase, command mode and sampling countdown, is
serialized with the board. A soft board CPU reset does not power-cycle the
external tracker. Whole-machine format is now **3**; older states are rejected
explicitly, not decoded using a changed layout. NVRAM remains independent and
untouched by pose changes or snapshot restoration.

### Verification of the implemented path

Two focused workspace tests check the signed binary packet against the firmware's
known byte positions and a clocked `cSufmC` exchange with identical continuation
after a mid-record serialized restore. The second also checks that `c` stops
generation; it is not a synthetic physical-sensor test.
The final `cargo test --offline --workspace` run passes 509 tests; the development
release build, format check and diff whitespace check pass. Existing fixture-only
ignored tests are outside this checkpoint. TCP tests require localhost sockets
outside the filesystem sandbox. No new dependencies were installed.

Isolated real-ROM experiments (ROMs/NVRAM never added to Git):

- Unchanged `epr-18021.6` receives and publishes
  `[4, -8, 120, 12868, 25736, -12868]` through the attached endpoint at all four
  baud selections, and with DSW1 bit 7 low (the `K` startup variant). UART errors
  stay zero, DPRAM mailbox `20` clears normally, and board save/restore preserves
  CPU, wire events and pose publication over another 700001 clocks.
- The complete real NetMerc machine reaches frame 1200 with streaming enabled.
  Changing the input pose then running 30 frames yields the same six DPRAM
  words and the V60 orientation `[16383, -32767, 16383]`, without patching PC,
  game RAM, ROM or decoded pose memory.
- A complete-machine save with the peer active restores and continues for 120
  frames with byte-identical encoded state and audio samples. This is automated
  device/loader evidence, not human gameplay or physical tracking acceptance.
- The existing scripted coin/holder/trigger sequence still reaches the rendered
  3D tunnel at frame 1868, now with the default pose delivered by 982 serial
  records and no UART errors. A headless screenshot confirms the forward-facing
  scene; it does not establish manual gameplay or graphics fidelity. Wing War's
  unchanged path also passes the real-ROM 300-frame save + 120-frame identical
  state/audio continuation check.

### Scriptable diagnostic input

`hmd` displays status. `hmd x y z ax ay az` accepts exactly six signed decimal
16-bit raw words. It changes only the endpoint input, not firmware RAM or DPRAM.
For example, in a headless diagnostic session:

```sh
tgpulse.dev netmerc --debug -c "run 1200; hmd 4 -8 120 12868 25736 -12868; run 30; hmd"
```

This is not a gameplay binding. The GUI debugger also owns an independent
machine, so its commands do not change the currently displayed gameplay session.
Until a desktop pose provider is implemented, ordinary gameplay retains the
fixed default orientation. Malformed or out-of-range poses are rejected without
changing the previous input.

### MVD cabinet controls and operator naming — 2026-10-03

Use **MVD** (Mega Visor Display) in operator-facing GUI labels, matching the
game's `MVD TEST` and the existing `NetMerc: MVD Holder` binding. A future motion
option should be named `MVD tracking`, not HMD/MHD. HMD remains a valid generic
hardware term in internal APIs/documentation; this does not rename `HmdPose`
or the diagnostic `hmd` command. The desktop MVD input selector below uses
fixed/stick or calibrated relative gyro poses. Holder Auto/Manual is
implemented separately below.

The user-requested stick Y inversion is in the **desktop cabinet ADC mapping**
for NetMerc only: SkyY positive/up produces channel-2 `00`, negative/down `FF`,
and rest remains `7F`. X, channel order, other flight games and bindings are
unchanged. Both gamepad backends, keys and rebound native signal sources use
the same mapping. It does not invert MVD orientation words or alter EEPROM
calibration. The existing independent analog-channel test checks this polarity.
This is an explicit user preference change, not a claim that MAME's declared
reverse flag or a physical arcade stick is wrong.

NetMerc's main LS signals (`Analog Joystick X/Y`) bypass the application's
15% sampling dead zone so small movements reach the ADC. This exception is
game/signal-specific: MVD/RS and other games retain their existing behavior.
SDL3 supplies normalized axes directly; Gilrs' device-level default filters
remain unchanged, so this does not remove backend/driver-side dead zones.

#### Holder state notification

The desktop frontend observes the supported game's acknowledged Holder word
at V60 `52766A` and displays `MVD Holder: latched / cleared` for three seconds
on entry and whenever that state changes. Releasing the button does not imply
`cleared`: the notification follows the game latch, including its reset when
starting a game and changes after loading a save state. It uses the existing
non-interactive notification overlay, also visible with the menu hidden.
Leaving NetMerc removes its notification. The core only exposes a read-only
state query; notification timing belongs to the frontend, not emulation.

Verification extends the existing save/restore and hidden-menu notification
tests. Actual on-screen gameplay acceptance remains a user test.

#### Holder level versus toggle

The real game reads the active-low Holder at `C00012`, bit 2 (physical IN1:04).
At `FF8294..FF82AC`, when credits/game state permit the check, a low level sets
the internal word `52766A` to 1. Releasing the input does not clear that word.
The game itself resets it during lifecycle transitions, including `FDFE3A`
when starting a game. `FF830D..FF831A` also reads the live Holder level for the
holder-light output; do not mistake a lamp condition for a gameplay interlock.

An isolated real-ROM experiment uses the existing temporary NVRAM previously
initialized by the game's own service clear, not personal files. After boot and
one coin, the actual credit count is 1. Three copies of that machine state are
run with identical timing:

| Holder stimulus before Trigger | Latch after stimulus/release | Latch after game starts | Result |
| --- | --- | --- | --- |
| Never pressed | 0 | 0 | Trigger consumes the credit and starts the game |
| Low for 60 frames, then released for 300 | 1, remains 1 | 0 | Trigger consumes the credit and starts the game |
| Low for 360 frames, held through Trigger | 1 | 1 | Trigger consumes the credit and starts the game |

A later 60-frame press in each running game sets the latch; releasing it for
120 frames leaves the latch at 1 and the gameplay dispatcher unchanged. The
startup dispatcher is `00FCEE58` in all three cases. These checks exercise normal
execution without patching registers, game RAM or NVRAM. The initial probe with
the partially repaired factory image had `FFFF` credit bytes and is **not** used
as paid-play evidence; the initialized service fixture avoids that confounder.

Conclusion: **no frontend toggle is required for the tested startup path**.
Keep the manual signal momentary. The previous 60-frame script pulse was a
diagnostic stimulus, not an installed emulator automation, and is not necessary
to start the sampled game. The explicit Auto convenience below does not justify
a permanent asserted level or automatic shooting/starting. Physical holder
wiring, lamp behavior and later end/repeat-game cases
remain separate operator checks; this result does not identify a dock sensor
versus a release button with certainty.

#### MVD Holder Auto / Manual — 2026-10-03

Settings exposes **NetMerc MVD Holder: Auto / Manual**, independently of MVD
pose selection. Default **Auto** is persisted as `mvd_holder = auto`;
`mvd_holder = manual` disables synthesized Holder input without reloading.
The regular West / L binding remains available in both modes. Switching to
Manual does not clear a latch already acknowledged by the game: the physical
switch's release does not clear it either. Existing three-second notifications
continue to report the actual program latch.

Auto reads the supported program's credit word (`400018`), session word
(`527668`) and Holder latch (`52766A`). With valid nonzero credits or an active
session, it supplies active-low IN1:04 until acknowledged, capped at 60 emulated
frames per attempt. All-ones uninitialized credits are not treated as payment.
An acknowledged latch or no eligible credits/session rearms the pulse; a game
start that clears the latch can therefore generate a fresh bounded pulse.
Manual cancels the sequencer and supplies only the existing physical binding.
No Trigger bit, ROM, game RAM, NVRAM or permanent toggle is synthesized.

This is **frontend convenience policy**, not inferred arcade-board behavior or
an asserted gameplay requirement. The read-only core context is derived from
already serialized SRAM/work RAM. The host sequencer uses emulated frame calls,
not wall time, and is recreated on a new session, mode switch or successful
state load; a restored latch itself is never rewritten. Future frontends can
reuse the bounded policy without SDL/GUI or choose their own physical input.

Verification: the isolated `holder-auto` probe reuses the actual sequencer source
and the service-initialized NVRAM fixture. Auto sets then releases Holder after
one actual credit while leaving the game stopped. A manual Trigger consumes the
credit and starts the session; Auto reacknowledges its cleared latch, whereas
Manual leaves it clear. A subsequent manual Holder press/release still latches.
The probe checks that only bit 04 may be synthesized and retains diagnostics at
`/private/tmp/tgpulse-netmerc.cUd0Qm/holder-auto.log`. The established ready-to-start
timing (360 frames after the post-credit baseline) is preserved; an earlier
Trigger was not accepted and is not reported as a start result. Full long-game,
service-menu and physical-device acceptance remain user checks.

### Desktop MVD input — 2026-10-03

Settings exposes **NetMerc MVD input**: `Auto`, `Off (fixed camera)`,
`Right Stick` and `Sensors`, persisted as `mvd_input = auto/off/right_stick/sensors`
(default `auto`).
Changes take effect on the next emulated frame, without reloading the game.
`Off` supplies the existing forward pose. Auto/Sensors prefer calibrated sensors
of the assigned P1 SDL3 controller, then the assigned stick, then fixed pose.
Unavailable, waiting, rejected or stale sensor input uses the existing fallback.
Explicit Right Stick uses the assignable signals even when rebound to keys.

The Cabinet P1 list now contains `NetMerc: MVD Look X / Y`, default RS-X/RS-Y.
They do not modify the cabinet stick's ADC channels or its LS bindings. P2 has
no corresponding MVD channel. Positive X looks right; positive native Y looks
up. Absolute deflection selects up to +/-30 degrees horizontal and +/-20
degrees vertical, with a rescaled 15% dead zone and target recenter on release.
The default limits are selectable in the GUI from 0 to 90 degrees in steps of
10, persisted as `mvd_horizontal_degrees` / `mvd_vertical_degrees`. Each value
is the maximum per side; zero locks that axis. Changes apply on the next frame.
The GUI shows `Off` instead of `0 degrees` for each axis; the configuration
keeps numeric `0`. These ranges apply only to simulated stick orientation,
including Auto when it selects the stick, not sensor measurements.
These are initial desktop usability limits, not measured cabinet specifications.
XYZ stays at the existing forward values; stick simulation keeps roll fixed.

Real-ROM visual probes established that increasing XANG pans right, increasing
ZANG looks down, while changing YANG tilts the image. Horizontal input therefore
adds to XANG and positive/up input subtracts from ZANG. Rendered tunnel scenes
are retained in `/private/tmp/tgpulse-netmerc.cUd0Qm/look-*.png`; the `look`
probe verifies actual serial/firmware delivery to DPRAM and return to default.
It uses the factory seed only to inspect camera geometry, **not as paid-play
evidence**. The service-initialized fixture remained on a 2D title scene in
this visual probe and is not used to infer camera axis directions.

Host sampling and mode policy live in desktop `input/mvd.rs`. Before each
NetMerc frame the frontend supplies a pose through `set_hmd_pose`, not DPRAM
writes or wall-clock time. Existing states already serialize the endpoint;
live frontend input supplies the next frame just like other controls. Tests
cover mode fallback, limits, dead zone, release, rebinding, independent cabinet
ADC input and settings round trips. Physical controller/gameplay comfort remains
unverified.

#### Relative Gyro Orientation — 2026-10-03

The desktop enables gyro/accel only for the selected P1 controller in NetMerc
Auto/Sensors, using the existing SDL3 sensor callback. Switching to Off/Right
Stick, closing/replacing the session, changing backend or P1 assignment stops
the previous sensor consumer and discards its estimator. No other game/P2
receives MVD samples. Calibration success or rejection (with its reason) is
reported once using the existing on-screen notification, including fullscreen
and paused sessions. Unsupported-device/enable errors also produce a notification;
Settings contains only adjustments and action buttons, not dynamic sensor status.
A newly selected controller starts a fresh five-second stationary
window (two seconds settling, three seconds bias estimation), with the same
100-sample/0.1 rad/s mean/0.02 rad/s deviation gates as the diagnostic.
Rejected calibration keeps the stick fallback until **Calibrate MVD** is used.

`OrientationTracker` consumes explicit monotonically increasing gyro sample
timestamps and integrates bias-corrected body angular velocity into a normalized
quaternion. It imports no SDL, device handle, wall clock or core state. A
sample gap over 250 ms restarts calibration rather than integrating a jump;
the desktop also discards a pose after 500 ms without valid gyro arrivals.
Duplicate, out-of-order and non-finite samples are ignored. No integration
is driven by emulated FPS, renderer FPS or SDL's advertised 1000 Hz rate.
The core still receives only the existing `HmdPose` once per emulated frame.

**Recenter MVD** resets the relative rotation without discarding valid bias;
**Calibrate MVD** discards both and starts the stationary sequence again.
Successful machine state loads restore the board's pose/serial state; subsequent
host input replaces that pose as with ordinary controls. Host calibration,
controller identity and quaternions are not machine snapshot data.

Mapping uses relative Y-X-Z Euler pitch/yaw/roll. Positive SDL yaw maps towards
decreasing XANG (left), positive pitch towards decreasing ZANG (up), roll to
YANG; signs combine SDL's documented right-handed axes with the earlier ROM
camera probes. Angles are guarded at ±90 degrees per axis, independent of the
stick-range settings, with a full-turn raw-angle wrap rather than i16 overflow.
The user confirmed the physical axis orientations in gameplay on 2026-10-03.
The captured live log contains three accepted calibrations and no rejection;
the earlier reported rejection cannot be diagnosed from that log.
Gyro-only tracking has residual drift. Optional gravity feedback is described
below; acceleration is never integrated into XYZ. Controller-to-head mounting
and Euler behavior near extreme pitch are not a six-degree-of-freedom tracker
solution. Absolute heading/position remain outside the current implementation.

##### Optional Gravity Stabilization — 2026-10-03

**MVD Gravity Stabilization** is persisted as
`mvd_gravity_stabilization = on/off`, default **On** by user request on
2026-10-04 (initially Off), and applies only to the
Auto/Sensors path. Changing it takes effect immediately without recalibration;
Right Stick and fixed-camera input are unchanged.

The shipped frontend preference and Restore Defaults select On. Explicit Off
in an existing profile remains Off; no personal configuration is rewritten.
The estimator still receives the choice explicitly from the frontend, keeping
the reusable motion helper independent of desktop defaults.

The existing quaternion integrator receives proportional gravity-error feedback:
normalized measured acceleration crossed with predicted body-frame gravity,
added to the bias-corrected angular rate. This follows the IMU-only
[Mahony complementary-filter reference](https://github.com/xioTechnologies/Open-Source-AHRS-With-x-IMU/blob/master/x-IMU%20IMU%20and%20AHRS%20Algorithms/x-IMU%20IMU%20and%20AHRS%20Algorithms/AHRS/MahonyAHRS.cs),
with proportional gain 1/s and no integral term or new dependency. Existing
stationary gyro calibration continues to own the bias. Unlike a fixed world-up
assumption, the gravity reference is averaged during the same stationary window,
in the controller's calibrated neutral frame. Recenter transforms that reference
to the new neutral frame, preserving deliberately tilted centers. If acceleration
first becomes available later, it establishes a reference without a pose jump.

Engineering gates, not emulated hardware constants: accept acceleration magnitude
within 15% of 9.80665 m/s² and samples no older than 100 ms; reject duplicate,
out-of-order and non-finite acceleration. Missing/stale/rejected acceleration
uses gyro-only integration. Only gyro arrivals update pose freshness. A detected
gyro gap/recalibration clears sensor history but preserves the selected option.
The gain and gates are bounded initial policy, not physical-controller acceptance.
Acceleration near 1 g can still contain translation; magnitude rejection cannot
perfectly distinguish it from tilt. Heading rotation about gravity is unobservable
without another reference, so yaw drift remains. No magnetometer, absolute XYZ,
device handle, wall clock or machine-state dependency is added to the helper.

Synthetic checks cover residual tilt drift versus uncorrected heading, genuine
motion with a tilted neutral, recenter continuation, absent/stale/accelerating
samples, live disable equivalence and configuration persistence. Physical feel
and translation sensitivity remain user tests, not automated gameplay proof.

Cabinet P1 exposes assignable virtual commands **NetMerc: MVD Calibrate**
(North) and **NetMerc: MVD Recenter** (West), with no default keyboard bindings.
They use the same frontend actions and notifications as the Settings buttons,
once per press, including while paused; neither sends a cabinet input to the
hardware core. Simultaneous presses prioritize calibration. Holder defaults
to Start OR D-pad Down, retaining L on the keyboard. Existing customized
bindings are preserved; the local runtime Holder binding was updated separately.

Tests cover stationary acceptance, each single-axis rotation, quaternion
continuation/recenter, duplicate timestamps and long-gap rejection; existing
MVD tests also check sensor priority, Off/Right Stick independence, neutral
pose, angle wrapping and unchanged cabinet ADC routing. Settings round-trips
cover the new mode. Physical gameplay acceptance remains with the user.

Verification: 515 offline workspace tests passed and the development release
build succeeded. The isolated NetMerc SDL3 launch in
`/private/tmp/tgpulse-mvd-sensors.BBpHEf/smoke.log` enabled the assigned P1's
gyro/accel and accepted bias `[0.00047825, -0.00979708, 0.00596562]` rad/s.
This is actual desktop sensor-path/startup evidence, not controller-motion or
camera-polarity acceptance. The run used copied temporary input settings and
NVRAM, with audio/rumble off; no personal runtime files were modified.

### SDL3 motion diagnostic — 2026-10-03

The desktop SDL3 backend now exposes explicitly enabled gyro/accelerometer
samples via an optional callback. NetMerc Auto/Sensors now also uses the same
callback for relative orientation. Rust SDL3's `hidapi` feature is an
empty API gate on the already installed SDL3 library: no extra dependency or
installation was needed. Sensor handles remain frontend-owned and disconnect
cleanup uses the existing controller lifecycle.

`input/motion.rs` defines frontend-neutral samples (gyro rad/s, acceleration
m/s² including gravity, explicit monotonic host event timestamp in ns) and a
stationary gyro-bias estimator. Neither imports SDL or emulation state. Timestamp
duplicates/out-of-order samples and non-finite values are ignored. At least
100 distinct samples, low mean rotation and low variance are required; the
user must still hold the controller stationary, since statistics alone cannot
prove that a slow constant rotation is gyro bias. This diagnostic does not
produce game poses, persist bias or add machine snapshot fields.

Use the bounded diagnostic: keep the controller still until `MOVE`, then rotate
pitch/yaw/roll separately, returning to neutral between rotations. Note the
first physical direction of each rotation so gyro signs can be interpreted.

```sh
cd /Users/andrea/dev/TGPulse
cargo run --offline --release -p tgpulse --example gamepad-motion -- 20 DualSense
# Or, after building the example:
/Users/andrea/dev/TGPulse/target/release/examples/gamepad-motion 20 DualSense
```

The diagnostic now skips the first two seconds for sensor/controller settling
and estimates bias over seconds 2–5. It prints the distinct-sample count,
per-axis mean and sample standard deviation (rad/s), accepted/rejected status
and the first failed condition/axis with measured value and limit. Thresholds
are unchanged: at least 100 samples, absolute mean at most 0.1 rad/s and
standard deviation at most 0.02 rad/s per axis. The helper's bias result and
diagnostic reason share one policy, not separate acceptance rules. This only
improves diagnosis; a rejected estimate is never used to generate a pose.
The diagnostic does not retry or weaken thresholds; gameplay integration is
described in the separate relative-orientation checkpoint above.

It matches exactly one controller by name, runs for 5..60 seconds, reports
advertised rates separately from received events, and exits on missing sensors,
disconnect or missing samples. It does not write input/settings/NVRAM files or
start a ROM. On this macOS host sandboxed enumeration returned no matching
device; the approved host-device run succeeded. Diagnostics are retained in
`/private/tmp/tgpulse-dualsense-motion.log`.

Earlier diagnostic (before the settling interval) Bluetooth DualSense evidence:
388 gyro and 388 accelerometer events in
six seconds (about 65 Hz observed, versus 1000 Hz advertised by SDL). The
193-sample initial gyro window estimated bias `[0.00018765, -0.00994937,
0.00627315]` rad/s. Acceleration was approximately `[0.23, 9.59, 1.08]` m/s²,
consistent with a nearly level stationary device. This establishes sensor
availability and a stable-window estimate, **not** empirical pitch/yaw/roll
signs, physical mounting, sensor-to-MVD mapping or gameplay acceptance.
SDL's documented device convention identifies gyro X=pitch, Y=yaw, Z=roll:
[SDL sensor units and axes](https://wiki.libsdl.org/SDL3/SDL_SensorType).
The next checkpoint verifies physical axis signs and then supplies calibrated
orientation through the existing pose API, without SDL entering the core.

### Possible Tracking Refinements

SDL3's [PS5 driver](https://github.com/libsdl-org/SDL/blob/release-3.4.12/src/joystick/hidapi/SDL_hidapi_ps5.c)
registers gyro and accelerometer sensors for supported DualSense devices.
[SDL sensor data](https://wiki.libsdl.org/SDL3/SDL_SensorType) provides angular
velocity and acceleration, not absolute position. The desktop now provides
relative gyro orientation; future gravity fusion could stabilize pitch/roll,
keeping XYZ fixed. Absolute XYZ obtained by integrating acceleration would
drift and is not a reliable six-degree-of-freedom tracker substitute.
Yaw drift, controller-to-head mounting, axis signs and neutral offsets require
hardware/gameplay validation. This integration adds no dependencies and does
not change gamepad bindings or the selected desktop input backend.
The pose API is reusable by a separate frontend without importing SDL into
hardware emulation.

### Fullscreen Surface Synchronization — 2026-10-03

User testing with the diagnostic display active exposed a macOS transition
where ImGui had already switched to 2940x1846 physical pixels while the acquired
surface remained 1984x1536. The resulting scissor exceeded the attachment and
wgpu correctly rejected it; the later winit poisoned-lock panic was secondary.
Both main and diagnostic redraw paths now synchronize surface size with the
current window before preparing GUI draw data, without reconfiguring unchanged
sizes. The UI pass also receives the acquired texture's actual dimensions and
discards mismatched draw data for that frame rather than drawing an incorrectly
projected interface. Scissors are bounded by the real target, not inferred IO
geometry. A regression check covers both directions of the reported size
transition. Interactive fullscreen acceptance remains a separate user test;
no core/device state or installed release is changed.

## Audio ROM Audit — 2026-10-03

The user reports static in NetMerc while Virtua Racing audio works. A read-only
archive audit found that `mpr-18134.ic32` contains only `0xff` throughout its
2,097,152 bytes. Its SHA-1 is
`ba59caac5f5a80fc52c507d8a47f322a380aa9a1`, matching MAME's `BAD_DUMP`
declaration in `src/mame/sega/model1.cpp`. MAME's ROM verifier reports
`NEEDS REDUMP` and accepts the set as best available; that is not evidence of
complete or valid sample data. The other sample ROMs contain nonuniform data.

The loader's sound-program word swap/reload and sample-region placement match
MAME. The shared sound-board maps, sample banking and clocks also match the
reference at this inspection level. In an isolated 2,588-frame NetMerc run,
commands reached the sound CPU, but active PCM1 instruments decoded their
headers from the blank ROM and all had start address `0x3fffff`. PCM2 and FM
were silent during the separately muted capture phases. This does not establish
that those sources are unused in every sequence.

A separate 48-second MAME capture, supplied with copied initial NVRAM and the
same coin/holder/trigger frame sequence, also produced weak, irregular audio.
Mixed-output RMS/peak were approximately 21/3199 signed 16-bit units in MAME
and 26/3278 during TGPulse's initial mixed phase. These aggregate measurements
are not a sample-aligned waveform comparison or listening acceptance. They
support a ROM-data limitation rather than a desktop audio-device failure, but
do not exclude additional NetMerc emulation defects.

No production audio code, gains, ROMs or personal NVRAM were changed. Captures
and the temporary core probe are under
`/private/tmp/tgpulse-netmerc-audio.1bjTAG`. A faithful repair needs valid sample
data or independently documented reconstruction; inventing sample headers or
substituting another game's ROM is not justified by this evidence. The blank
ROM alone does not prove that reconstruction from other surviving data is
impossible.

### TeknoParrot Audio Restoration Follow-Up — 2026-10-03

The user supplied [a NetMerc video](https://www.youtube.com/watch?v=bA0e1GM3s8g).
Direct video access through the research tool failed, so no personal listening
or transcript inspection is claimed. The team's public
[Model 1 / VR announcement](https://www.patreon.com/TeknoParrotTeam/posts/teknoparrot-does-168035857)
explicitly announces restored NetMerc sound and music. The public
[UI updater source](https://github.com/teknogods/TeknoParrotUI/blob/master/TeknoParrotUi/MainWindow.xaml.cs)
marks `TeknoModel1` as `opensource = false`.

The inspected public material does not specify changed ROM hashes, a redump,
sample-header reconstruction, external replacement audio or a chip/banking
correction. None of these explanations is established. The restoration claim
is a reason to investigate beyond MAME equivalence, not evidence that the
local blank ROM is good or that a particular workaround is faithful. Next
useful evidence would be the team's technical explanation or a legally owned
working setup's ROM hash/size manifest and resource layout, without distributing
ROMs. Keep this audio follow-up open; do not patch the loader speculatively.

#### User-Supplied TeknoParrot Set Comparison — 2026-10-03

Read-only comparison of the supplied `Sega NetMerc (1993)[Sega Model 1][TP].zip`
against the development `roms/netmerc.zip` found three nested archives:
`netmerc.zip`, `model1io2.zip`, and `hd44780.zip`. Every member of the supplied
game archive is byte-identical to its local counterpart, including all four
audio ROMs and the factory NVRAM. In particular, `mpr-18134.ic32` is still
2 MiB of `FF` with SHA-1 `ba59caac5f5a80fc52c507d8a47f322a380aa9a1`.
The three Model 1 I/O firmwares and LCD font also match. Other files in the
supplied `model1io2.zip` are additional device resources, not changes to these
matched NetMerc resources. No external audio files or emulator executable are
included in this package. All ZIP members were read and compared without
extracting or replacing installed resources.

This rules out different ROM bytes **in this supplied package**, not in every
TeknoParrot distribution or the cited video's exact setup. If that setup uses
this same package, its audio recovery must depend on emulator behavior or
resources provided outside it; the specific mechanism remains unverified.

### Cross-Game Audio Recovery Feasibility — 2026-10-03

This checkpoint compares local VR, Virtua Formula, VF, Wing War and SWA audio
resources, the current MultiPCM implementation and MAME's
`src/devices/sound/multipcm.cpp` / `src/mame/shared/segam1audio.cpp` at local
revision `bd7e0b81584`. MAME supplies the chip-format reference, not proof that
its current NetMerc ROM placement or playback is correct.

#### What Survives And What Is Missing

- The sound program `epr-18121.ic7` contains an executable 68000 sequencer and
  instrument/key-split data. The note-on path at decoded address `0xA4C` indexes
  the pointer table at `0x39F6`, chooses a six-byte record by note, and reads the
  sample number from bytes 4–5. Pitch adjustments come from bytes 2–3. The
  percussion path at `0x952` instead indexes the table at `0x3E30` and reads
  sample number, pitch and pan/routing information. These are not copies of the
  chip's sample-address/loop/envelope table.
- The melodic pointer table has 37 distinct targets. For example, program 0
  selects sample IDs 70–74 across successive pitch ranges. This is a concrete
  constraint for future identification, not evidence that program 0 is any
  particular named instrument or that the complete soundtrack is validated.
- At `0xCD6` the driver writes the sample index to chip register 1; key-on follows
  at `0xD12`. The MultiPCM itself loads a 12-byte descriptor at `sample_id * 12`:
  waveform start/format, loop start, length, envelope and LFO parameters. Thus
  intact sequencing code cannot compensate for absent descriptors by itself.
- `mpr-18134.ic32` is entirely FF, including the PCM1 descriptor table. Other
  games' first sample ROMs hold both descriptors and waveform data, so it is not
  justified to assume that only metadata was lost from NetMerc's missing 2 MiB.
  Its original waveform contents cannot be determined from the blank bytes.
- `mpr-18135.ic33` and `mpr-18136.ic4` retain nonuniform, waveform-like data.
  The former begins with `0x2010` FF bytes; the latter does not begin with a
  coherent descriptor table under the currently implemented format. A scan for
  runs of at least 24 plausible 12-byte descriptors found no candidate in either
  file. Matches in sound-program sequence data were false positives on inspection.
  This heuristic does not exclude another encoding or a shorter/fragmented table;
  neither the missing-table extent nor the current PCM2 placement is thereby
  proven correct.

#### Donor Comparison

Virtua Formula uses the same audio ROM bytes as VR and adds no independent donor.
In the two surviving NetMerc sample files, a first search of 176 nonconstant
128-byte signatures, sampled every 16 KiB, found no exact match at any byte offset
in the other games' sample ROMs. A reverse search then selected signatures from
plausible donor sample headers, considering each bank for banked addresses:

| Donor | Distinct 128-Byte Signatures Searched | Exact Matches In NetMerc |
| --- | ---: | ---: |
| VR | 525 | 0 |
| VF | 2,856 | 0 |
| Wing War | 3,131 | 0 |
| SWA | 1,515 | 0 |

These are signature counts, not counts of independently verified instruments.
The comparison rejects an easy byte-identical donor on the examined data, not
all shared source recordings: resampling, changed encoding, short waveforms or
other transformations would evade it. It cannot compare a donor against the
actual contents of the missing ROM. Copying another game's descriptor table
would attach that game's addresses and envelopes to different/unidentified data.

#### Recovery Decision

Partial reconstruction remains plausible: identify one preserved waveform,
associate it with a driver sample ID, establish its storage/banking, and then
infer or recover its boundaries, loop and envelope. Validate that single voice
in isolation before generalizing. Parameters inferred by ear or substituted
from a donor must be labelled approximate, not original recovered bytes.
The surviving key-split/pitch map helps constrain this work but does not uniquely
determine those missing parameters. Unrelated donor sounds would be replacement
audio, not a verified restoration.

TeknoParrot's restoration claim does not establish which of these steps it uses
or whether it embeds additional data. This checkpoint found no evidence-backed
drop-in patch, so no donor patch or experimental playback was installed. Normal
ROMs, NVRAM, executable and audio gains are unchanged; no fresh gameplay/listening
acceptance is claimed. Private audit inputs, header summaries and disassembly
are under `/private/tmp/netmerc-audio-recovery.67WG1G`; analysis scripts are
`/private/tmp/netmerc-audio-audit.py` and `/private/tmp/netmerc-donor-probe.py`.
No ROM bytes or generated ROM-derived tables are added to Git.

## Next bounded checkpoint

### TeknoParrot Installed-Reference Audit — 2026-10-03

Read-only inspection of the user's live RETROSTATION installation identifies
`TeknoModel1.exe`, SHA-256
`CF7EED930ACF9ABD756928CD8E43084D21BFB8AB65410990501989864EBF28BC`.
Its installed NOTICE explicitly describes project-owned source as private;
no TeknoModel1 implementation was copied or incorporated into TGPulse.

The installed `UserProfiles/netmerc.xml` exposes **NetMerc Audio Donor**, with
`vf`, `vr`, `swa`, `wingwar` and `off`. Its hint describes donor-assisted recovery
using user-provided merged donor ROMs, with a procedural fallback for `off`.
The actual running command includes `--netmerc-donor vf` and an explicit ROM
directory. This establishes a donor/procedural interface, not authentic recovery
of NetMerc's missing data or the algorithm associating donor samples with voices.

At initial inspection that directory contained only `netmerc.zip`,
`model1io2.zip` and `hd44780.zip`; VF was not present there. The selected donor
therefore did not establish availability/use. Another search path, embedded
resources or fallback was not verified. With explicit user authorization,
`vf.zip` was subsequently copied from the local ROM collection without replacing
any existing file. Source/staged/destination SHA-256:
`86CB63B30093F0F9E3ACB1C993711720EB83FA362676DF974941E2C199D64C24`.
The existing NetMerc process was not restarted, so the copy does not prove it
loaded the newly available donor. No profile or NVRAM was edited.

The existing Windows remote test transport was reused with a non-Libretro
capture bundle. Its successful second capture corrected DPI handling and brought
only the game window to the foreground; the first capture was occluded and is
not visual game evidence. The screenshot shows the title/start scene, not audio
playback. No usable audio recording tool was found on PATH; no software was
installed and no listening comparison is claimed. A bounded `--help` diagnostic
produced no output and did not exit within eight seconds; only that owned process
was stopped. The original game PID remained unchanged.

Private capture evidence: `/private/tmp/teknoparrot-audit.IIKG4u/results-b/`;
isolated Windows test roots: `tekno-netmerc-audit-20261003a` and
`tekno-netmerc-audit-20261003b`. The reusable installed skill is
`windows-teknoparrot-audit`, outside this repository. Next audio evidence would
compare the same scene with a verified available donor and procedural fallback,
using a qualified capture method and isolated save data. Implementing an optional
donor mode in TGPulse requires a separate decision; it must not be labelled
original-ROM restoration without evidence.

### TeknoParrot Controlled Audio Trials — 2026-10-03

The subsequent checkpoint uses the same installed TeknoModel1 binary, with
authorized restarts and private ROM/save directories. No resident TeknoParrot
process was running at its start; every process launched by the trials is owned
and closed by the collector. The normal profile and original save are not edited.

No capture package was installed. A small Windows/.NET helper records the native
WASAPI render-loopback stream in the interactive desktop session. The existing
Windows test runner supplies transport and process isolation. A two-second idle
baseline contains no audio frames; the gameplay recordings contain 29.99 seconds
of 48kHz stereo float32 audio. This is output-device mix capture, not per-process
capture. One initial discontinuity is recorded per trial; it is not evidence of
continuous dropouts. Initial title-only pilot captures are silent and excluded
from the in-game comparison.

The repeatable input sequence uses TPUI's public cabinet-input interface, not a
copied emulator implementation: centered NetMerc axes, Fire for 250ms at one
second and every four seconds thereafter. The engine first reaches its title
scene; recording lasts 30 seconds and end screenshots confirm gameplay/shooting.
The same original save is copied afresh for each run. Recorded seed SHA-256:
`0401EC35AED5A169F0C54CAD5A050122BB4833FDD0E39CF40BB33900EC6C7EFC`.
The first successful `off` run predates automated seed-hash recording; subsequent
trials include it. This is matched-input, wall-clock playback, not frame-locked
or deterministic waveform playback.

The experiment controls both `--netmerc-donor` and ZIP availability in the supplied
ROM directory. The normal installation's newly copied VF ZIP is not removed;
other internal resource search paths have not been traced. Mere ZIP availability
therefore does not prove which bank the engine loaded.

Measured results (RMS and peak over both channels, no normalization):

| Trial | Donor Option | VF ZIP In Trial Directory | RMS dBFS | Peak dBFS | Gameplay Audio |
| --- | --- | --- | ---: | ---: | --- |
| `fire` | `off` | No | -30.054 | -13.640 | Present |
| `vf-missing` | `vf` | No | -31.427 | -14.888 | Present |
| `vf-present` | `vf` | Yes | -31.418 | -14.618 | Present |
| `off-repeat` | `off` | No | -31.300 | -14.624 | Present |

Every trial produces approximately 28.5 seconds of non-silent 100ms blocks,
following the initial title/start transition. Average spectral power in the
0–250Hz / 250–1000Hz / 1–4kHz / above-4kHz bands is respectively:

- First `off`: 34.860 / 60.375 / 4.220 / 0.545 percent.
- `vf-missing`: 36.158 / 58.800 / 4.483 / 0.559 percent.
- `vf-present`: 36.733 / 58.502 / 4.211 / 0.554 percent.
- Repeated `off`: 36.292 / 59.227 / 3.924 / 0.557 percent.

VF absent/present differ by only 0.009dB in overall RMS; the two `off` runs
differ by 1.246dB. Aligned mono waveform correlation over the first 25 seconds
is also low for repeated `off` (~0.317), comparable to the cross-option results
(absolute correlations ~0.28–0.41). These metrics establish neither identical
audio nor donor-specific changes: timing/phase and scene advancement are not
frame-locked, and only one same-setting repeat was collected. No subjective
listening or original-cabinet fidelity acceptance is claimed.

Conclusions and confidence:

- **High:** the engine produces in-game audio with `off`, and with `vf` selected
  while the supplied ROM directory has no VF ZIP. This independently confirms
  the user's observation that having sound does not require that ZIP there.
- **Medium:** the measured behavior is consistent with the installed profile's
  explicit procedural-fallback description. It does not identify the synthesis
  algorithm, and does not prove that `vf`-missing and `off` share a code path.
- **Unresolved:** whether this binary loads/uses the supplied VF samples for
  this scene. Bank-load diagnostics or a bounded file-access trace would be the
  next discriminating check, before more donor-selection listening trials.
- **Not demonstrated:** authentic reconstruction of the all-FF descriptor ROM,
  or the sample-to-command mapping needed to restore original NetMerc sound in
  TGPulse. Donor/procedural replacement remains a separate implementation choice.

All four trial scripts returned success with visible gameplay. Final host
inspection found no remaining TeknoModel1/TPUI test process; the original save
and executable hashes still match the recorded values. Normal profile settings
were never rewritten. Private Windows roots use the IDs
`tekno-audio-fire-20261003`, `tekno-audio-vfmissing-20261003`,
`tekno-audio-vfpresent-20261003` and `tekno-audio-offrepeat-20261003`.

Private evidence and the numerical WAV analyzer are retained under
`/private/tmp/tekno-audio-ab.PoVSrN/`. The reusable `windows-teknoparrot-audit` skill
now includes the verified native recording helper and the isolated trial recipe.
ROMs, save copies and recordings remain outside Git. No TGPulse emulation/audio
code, installed binary, gains or normal configuration changed in this checkpoint.

Primary interface references:

- [TPUI engine arguments](https://github.com/teknogods/TeknoParrotUI/blob/master/TeknoParrotUi/Views/GameRunningCode/ProcessManagement/TeknoViperVegasLauncher.cs)
- [TPUI Model 1 cabinet input](https://github.com/teknogods/TeknoParrotUI/blob/master/TeknoParrotUi.Common/Pipes/TeknoModel1Pipe.cs)
- [TPUI shared input page](https://github.com/teknogods/TeknoParrotUI/blob/master/TeknoParrotUi.Common/Jvs/JvsHelper.cs)
- [Microsoft render-loopback capture](https://learn.microsoft.com/en-us/windows/win32/coreaudio/loopback-recording)

### TeknoModel1 Binary / File-Access Checkpoint — 2026-10-03

The user authorized focused static/runtime investigation, followed by explicit
authorization to download and run Microsoft Process Monitor in an isolated test
directory (including its administrative capture driver). No debugger installation
or TGPulse implementation change is part of this checkpoint.

The private copy of the installed `TeknoModel1.exe` matches the previously
recorded SHA-256. It is a native x86-64 PE32+ GUI executable, not a managed .NET
assembly. Original-named `.text`, `.rdata`, `.data` and `.pdata` sections have
zero raw data size on disk; additional `.teknoGo` sections contain the payload
and entry point. The PE debug directory is empty. A bounded printable-string
search finds no NetMerc/donor/MultiPCM identifiers. These observations are
consistent with packing/protection; they do not identify an audio algorithm,
prove encryption of every section, or establish that runtime analysis is
impossible. No unpacking, protection modification or authentication bypass was
performed. Public TPUI code constructs the same `--netmerc-donor` argument used
in the preceding trials; it does not contain the engine's recovery algorithm.

Process Monitor was obtained from Microsoft's official distribution. Windows
Authenticode reports a valid Microsoft Corporation signature; the host runs
Windows 11 Pro and the SSH account can execute the explicitly authorized
administrative test. The ordinary interactive runner remains unchanged: a
private copy changes only this trial's scheduled-task run level to Highest.
The initial limited-privilege UI inspection did not yield usable controls; an
incomplete private runner copy also failed before launch. Neither is audio or
file-access evidence. Collector setup uses `/NoConnect` before any trace and
validates process/file-system filtering before capture.

Collector qualification: Win32 message automation initially failed to add the
filter. UI Automation required enumeration/warm-up before the controls exposed
their ComboBox/Value/Invoke patterns. The final setup verifies actual enabled
`Process Name is TeknoModel1.exe Include` and `Event Class is File System Include`
rows. Each acquisition enables Process Monitor's Drop Filtered Events setting
and restores its previous value afterward. CSV export is checked for foreign
process rows; the raw PML is retained privately. Reuse the successful private
`probe/run.ps1` and `trace/run.ps1` rather than the failed earlier setup trials.

Two matched-input recordings keep `vf.zip` present and change only the donor
option. The file trace starts before engine launch and covers initialization,
gameplay and private-save shutdown:

| Trial | File Events | NetMerc ZIP ReadFile | I/O 2 ZIP ReadFile | VF ZIP ReadFile | Other Relevant Result |
| --- | ---: | ---: | ---: | ---: | --- |
| `tekno-files-off-20261003` | 13,344 | 11,776 | 218 | 0 | No access to VF ZIP despite its availability |
| `tekno-files-vf-20261003` | 16,764 | 11,776 | 218 | 3,482 | One failed lookup of adjacent `model1io.zip` |
| `tekno-files-vfbios-20261003` | 20,334 | 11,776 | 218 | 6,964 | Four successful ReadFile operations on `model1io.zip` |
| `tekno-files-offbios-20261003` | 13,312 | 11,776 | 218 | 0 | Neither VF nor I/O 1 BIOS accessed despite both being present |

All counted ReadFile events above succeeded. Each exported trace contains one
TeknoModel1 PID and only file-system operations. Normal controlled monitor close
returns code 1 in these runs; CSV export returns 0 and the trial script returns
0. Evidence rests on the parsed complete trace through save shutdown and game
screenshots, not an assumed monitor exit-code convention. Audio remains present
in both traces (RMS -31.441 and -31.361dBFS respectively); this still does not
prove sample substitution or reconstruction.

**New high-confidence result:** the donor option is acted on at the file-loading
level. With `vf`, the engine opens VF for Generic Read and issues thousands of
successful reads; with `off`, no VF access is observed. This resolves the earlier
uncertainty about whether the available ZIP was even read, not whether the loaded
bank is accepted/used. Missing `model1io.zip` is a potential loader confound, not
a diagnosed failure: the supplied merged VF ZIP itself contains its three I/O
firmware variants.

The additional paired trials resolve the loader confound at the observable
behavior level. A private copy of the user's `model1io.zip` was staged only in
the test directory (SHA-256
`4D595B927D9C20DE4064638327375D380CED62C42D3D91B658524662AFD42185`).
With `vf` plus this BIOS, successful VF reads double and the captured audio
changes markedly: RMS -19.045dBFS, peak -3.725dBFS, spectral-band percentages
47.059 / 35.300 / 14.229 / 3.411. Switching only to `off` while retaining both
VF and BIOS restores the previous profile: RMS -31.300dBFS, peak -14.624dBFS,
bands 36.292 / 59.227 / 3.924 / 0.557. Both runs use the same seed, sequence,
endpoint and format, and screenshots confirm the same gameplay scene.

**Strong behavioral conclusion:** this binary needs the adjacent I/O 1 BIOS for
the tested VF-assisted path to progress beyond its initial VF read and change
the generated audio. Availability of a merged VF archive alone was insufficient.
The approximately 12.25dB RMS difference and spectral change are far larger than
the preceding repeat variability; no perceptual/original-hardware fidelity claim
follows. The earlier near-equal VF-present/absent recordings did not establish
an active donor bank. They remain valid observations of their incomplete resource
setup, not evidence that Audio Donor is ineffective.

The internal mapping remains unresolved: no descriptor table or synthesis
algorithm has been extracted. Reading VF and observing donor-dependent sound
does not distinguish replacement tables from higher-level command interception.
The next bounded implementation-analysis checkpoint is to locate the runtime
sample/descriptor or command-dispatch boundary under this now-qualified resource
setup, not more blind donor-selection comparisons. No proprietary code or table
has been incorporated into TGPulse.

Process Monitor 4.11 and isolated evidence are retained for reuse. Final host
inspection after all four captures confirms no remaining test processes/PROCMON
driver, Drop Filtered Events restored to 0, and the original save hash unchanged. The
ordinary Windows installation still has the earlier authorized VF copy; I/O 1
BIOS was added only to isolated trials, not to the ordinary ROM directory.

Private investigation files: `/private/tmp/tekno-binary-audit.Rt9zOY/`.
Do not publish the engine copy, memory contents, ROMs or raw private traces.

### TeknoModel1 Runtime Audio Boundary — 2026-10-03

This checkpoint supersedes the preceding uncertainty about the internal audio
boundary for the recorded binary hash; it does not recover authentic missing
NetMerc descriptors. A private instance used copied ROMs and save data. Standard
Windows process-query/read access captured only its original-named code/data
sections after ordinary startup: 289 pages, no failed reads. No process writes,
injection, suspension, protection changes, authentication bypass, debugger
installation or whole-process dump were used. The resident TeknoParrot UI was
left running. Private binary/disassembly evidence stays outside Git.

Focused instruction analysis identifies three paths:

| Path | Observed Boundary | Interpretation |
| --- | --- | --- |
| Original MultiPCM | Two existing instances receive sound-CPU register writes | Normal hardware-emulation path remains present |
| Donor | Two additional instances receive the same register writes and bank selections; the mixer selects their combined output when both exist | Sample-bank substitution, not evidence of restored NetMerc descriptors |
| Recovery Without Donor | A separate two-chip/28-voice-per-chip engine observes register writes, counts key-ons and generates output from oscillator/envelope state | Procedural replacement sound, not playback proof for the missing ROM |

The inspected donor setup validates each supplied bank, creates two instances
with the ordinary PCM implementation and retains their ROM spans. Its validator
reads candidate sample descriptors; the constructor/reset initializes mutable
chip state. Neither inspected routine rewrites ROM descriptors. Guest writes at
the two MultiPCM windows and bank controls are mirrored into the additional
instances without a separate game-command translation at that boundary.

Recovery voice setup branches on chip/sample identifiers; the renderer advances
oscillator phases and uses calculated waveforms, noise and envelope state. This
establishes a procedural path, not its perceptual accuracy or the original sound
associated with every sample ID. No proprietary waveform recipes, descriptor
tables or implementation code have been copied into this project.

The binary also exposes native `--diagnostics <path>`, including
`netmerc_audio_recovery`, `netmerc_audio_donor` and `netmerc_recovery_keyons`.
Instruction analysis shows that the donor field tests the presence of **both**
additional PCM instances. The recovery flag is separate: it can remain enabled
when donor audio is selected. Key-on counters observe the register stream even
with an active donor, so nonzero counters alone do not prove that procedural
audio reaches the mixer.

Native-diagnostic validation used two approximately 40-second runs, both with
VF and I/O 1 BIOS available, the same private save seed, and the engine's own
`--auto-start`. No shared input-page injection was used. Screenshots show the
same gameplay scene; these are bounded automated runs, not a full gameplay or
original-hardware listening validation.

| Native Diagnostic | VF Donor | Donor Off |
| --- | ---: | ---: |
| Frames | 2,343 | 2,344 |
| `netmerc_audio_recovery` | 1 | 1 |
| `netmerc_audio_donor` | 1 | 0 |
| `netmerc_recovery_keyons` | 411+32 | 411+32 |
| `audio_nonzero` | 694,279 | 658,669 |
| `audio_peak` | 14,354 | 7,030 |
| `audio_clipped` / `audio_dropped` | 0 / 0 | 0 / 0 |

Both native reports have empty audio/fatal-error fields. The one-frame duration
difference prevents treating totals as sample-aligned comparisons; identical
key-on counts and the explicit donor flag are the relevant evidence here.
Together with the observed mixer selection, these results establish active
donor and no-donor recovery paths in this installed build.

Runs: `tekno-diag-vf-20261003` and `tekno-diag-off-20261003`; local results are
`diag-vf/` and `diag-off/` under the private evidence root below. Both save seeds
and the source save after the paired runs have SHA-256
`1EA1B718967CEC8C32D1B28CB3DC960A502E5D614927BAA5267CB829A689DEEE`.
This is the seed for these runs, not the older audit's save snapshot. The
resident UI retained PID 16700; no TeknoModel1 test process remained afterward.

Reproducible private evidence: `/private/tmp/tekno-memory-audit.SwlT0s/`;
module acquisition run `tekno-memory-vf-20261003`. Analysis locations below are
PE RVAs (not ASLR-dependent runtime addresses): donor setup `0x5d2f0`, guest
write dispatch around `0x5d7bd`, mixer `0x5db70`, recovery register handler
`0x602b0`, recovery renderer `0x60990`, bank validator `0x89400`, PCM reset
`0x894f0`, native audio diagnostic fields around `0x1a055`.

Scope limit: this is a targeted path analysis, not a complete audit of all
loader transformations or all versions of TeknoModel1. It supports an optional
donor/substitute-audio design if separately requested; it does **not** justify
claiming that the all-FF NetMerc descriptor ROM has been reconstructed. TGPulse
audio behavior is unchanged by this investigation.

### TGPulse Donor Checkpoint — 2026-10-03

The first implementation checkpoint adds donor sample substitution, not the
procedural fallback. `netmerc_audio_donor` persists in the desktop settings file:
`vf` (default), `vr`, `swa`, `wingwar`, `off`. The Settings → Audio selector applies
on load/reset. Other games, including Model 2, do not access donor resources.

The optional filesystem helper loads the selected set's two PCM regions from
an adjacent `<donor>.zip`, reusing the ROM database's layout/interleave/copy
logic. It requires all declared sample chips at their declared lengths, but
does not load the donor CPU, graphics, I/O BIOS or DSB. In particular,
`model1io.zip` is **not** needed by TGPulse's donor loader. The sample regions
are validated and applied together; failures leave the original resources
unchanged and produce a desktop notification plus detailed log. Off preserves
original audio, which remains defective for the available NetMerc dump. Do not
describe this interim behavior as TeknoModel1's procedural fallback.

`loader::audio_donor::SampleBanks::apply` also accepts frontend-provided memory
resources, without paths or implicit I/O. Only the two sample-bank images change:
NetMerc's 68000 program, V60 program and command/timing paths remain untouched.
This deliberately reuses the two existing MultiPCM chips rather than adding two
shadow chips: their host-visible register reads are constant zero, their sample
state has no CPU interrupt output, and only the chosen PCM output is needed in
this load-time implementation. Existing board gains/mutes and YM3438 mixing
remain unchanged; TeknoModel1-specific gains/filtering are not copied. Neither
bit-identical TeknoModel1 output nor authentic cabinet sound is claimed.

The existing Model 1 resource hash includes both banks. The normal chip snapshot
therefore suffices, with no new mutable device state or format bump. States with
different loaded banks are rejected atomically; matching-bank states restore
active voices and deterministic continuation. Runtime bank switching is not
implemented, avoiding partially reinitialized voices and stale state identities.

Verification:

- `cargo test --offline --workspace`: 525 tests passed. The sandbox denied two
  existing loopback socket tests; the full rerun with host socket access passed.
  New focused tests cover selected-region loading without unrelated BIOS/DSB,
  missing/truncated/oversized sample rejection, settings persistence, unchanged
  sound program, resource identity and active-audio save/load continuation.
- `cargo build --offline --release -p tgpulse`: development binary built.
- Private headless real-ROM runs reuse the existing NetMerc cabinet sequence,
  with in-memory NVRAM and no host audio/controller device. Each reaches 2,168
  frames, then verifies 60 frames of byte-identical audio and full state after
  restore. These are smoke/determinism checks, not listening acceptance.

| Donor | Nonzero Stereo Samples / 232,821 | Peak Absolute Sample | Key-Ons After Continuation |
| --- | ---: | ---: | --- |
| VF | 232,821 | 16,993 | 358 + 4 |
| VR | 232,012 | 20,844 | 358 + 4 |
| SWA | 232,810 | 19,617 | 358 + 4 |
| Wing War | 232,820 | 16,665 | 358 + 4 |
| Off (Original) | 13,847 | 1,910 | 358 + 4 |

Private WAV/log evidence: `/private/tmp/tgpulse-donor-smoke.DbCdDs/`.
The existing temporary headless harness was extended with `audio-donor.rs` under
`/private/tmp/tgpulse-netmerc.cUd0Qm/`; ROMs and recordings stay outside Git.
No installation, toolkit release, commit or push is part of this checkpoint.
The user subsequently accepted donor playback. The procedural checkpoint below
supersedes the interim Off/missing-donor policy described above.

### TGPulse Procedural Checkpoint — 2026-10-03

**Scope:** best-effort substitute audio, not the missing ROM's recovered contents
or a claim of authentic NetMerc timbres. A complete, verified redump remains the
definitive solution. The original sound program still determines notes, rhythm,
key-ons, pitch changes, pan and level. No music recordings or donor assets are
embedded in this fallback.

Reference: the same installed TeknoModel1 binary and authorized private runtime
section snapshot documented above. Focused analysis covered voice setup,
register dispatch, pitch/pan/level calculations, waveform generation and the
recovery mixer. The TGPulse module is independently written Rust expressing
the observed functional rules; no proprietary source, instruction bytes,
waveform/ROM tables or binaries are distributed. Private analysis evidence
remains under `/private/tmp/tekno-memory-audit.SwlT0s/`.

#### Observed Rules And Implementation

| Boundary | Functional Rule |
| --- | --- |
| Register Interface | Two groups of 28 voices; data, slot and register selectors reuse the physical MultiPCM selector mapping. Registers 0–5 drive pan, nine-bit sample identifier, pitch, key-on/off and attenuation. Registers 6–7 are retained but LFO synthesis is not implemented in this substitute path, matching the inspected fallback. |
| Classification | First chip: IDs below 73 are percussion, higher IDs are melodic. Second chip: effects. Melodic IDs 83–85, 105–106, 124 and ≥135 select distinct waveform/envelope families; percussion uses ID modulo four and effects use ID parity. These are substitute timbres, not proven original instrument names. |
| Pitch | Signed four-bit octave, ten-bit fraction, octave-minus-one scaling. Melodic reference 523.2511306011972 Hz; percussion/effect frequency derives from sample ID and pitch, with a 0.5× floor and bounded phase increment. Kick/tom-like percussion uses separate envelope/ID-dependent frequency rules. |
| Tone Generation | Sine/triangle, band-limited saw/square, detuned second oscillator and phase-modulated harmonics, with family-specific combinations and one-pole smoothing. |
| Percussion/Effects | Deterministic per-voice xorshift noise, sine components, low-/high-pass combinations and decaying envelopes. Key-on seeds noise from sample ID and chip index; no host randomness. |
| Envelope | Melodic attack followed by key-off release; percussion/effect exponential decay, accelerated on key-off. Release cutoff 0.0001. |
| Level/Pan | 0.375 dB total-level steps, 3 dB pan steps, center/side/mute positions. Phase and filter state advance even when the frontend mutes an output. |
| Clock | Native 10 MHz / 224 sample grid, advanced by the existing sound-board scheduler; no wall-clock timer or second host audio stream. |

The substitute output lives in `sound/netmerc.rs`; the existing CPU/bus and
MultiPCM chips continue receiving writes and advancing normally. Recovery
replaces their **output**, not their guest-facing status or program execution.
Both outputs retain their own existing gain/mute controls and the existing FM
route. This is selected at machine construction, never halfway through a song.

Deliberate differences and confidence limits:

- The inspected slow-attack melodic family (ID 124) starts at 0.00008, below
  the unconditional 0.0001 silence threshold. Instruction flow suggests the
  reference would terminate it on its first sample. TGPulse applies this cutoff
  only to decaying/releasing voices: an attack below the threshold is valid and
  must be allowed to grow. This is a device-logic-based correction, not a claim
  to reproduce a TP defect. A focused test retains the voice through attack.
- TP scales the combined recovery pool by 0.105 and applies recovery DC removal.
  TGPulse keeps those coefficients but applies DC removal independently to the
  two source routes, before its existing per-source clip/gain/mute boundary.
  Output scaling compensates for the existing 50% reference routes; no stored
  user gain changes. TP's combined-pool clipping and TGPulse's source clipping
  can differ at saturation. FM mixing and the accepted donor path remain as
  previously implemented. Bit-identical TP output is **not** claimed.
- Noise/filter/oscillator rules have direct instruction evidence. Associating
  their timbres with original cabinet sounds lacks a valid dump/hardware
  recording; perceptual accuracy remains best effort.

#### Resource Policy, Frontend And State

The loader enables recovery only for NetMerc with the known all-FF PCM1
descriptor table. A complete donor atomically replaces both banks and disables
recovery; Off does not access donor files, and a missing/invalid donor leaves
the original resources plus recovery flag unchanged. A future valid original
descriptor table does not trigger this fallback. The core receives an explicit
load-time resource policy, without knowing desktop paths or scanning ZIPs.

Settings/config keep the same five donor choices. Off now means no donor, not
no sound. The desktop reports procedural selection or a missing-donor warning;
the existing native-rate audio queue, gains, mutes and master volume are reused.
No personal configuration or NVRAM is rewritten by this integration.

Model 1 state format **4** and audio-board state **2** include recovery voices,
register selectors, phases, envelopes, noise seeds and DC/filter histories.
The machine identity includes recovery mode as well as loaded banks; restore
validates before mutation, does not replay register writes or access disk, and
preserves frontend gains/mutes. Older state files are rejected explicitly;
persistent NVRAM is unchanged. Frontends can provide resources and drive native
sample/frame advancement without the desktop event loop.

#### Verification

- Focused tests cover observed register/classification/pitch/pan/release rules,
  the slow attack, both-chip mid-voice serialization including noise/filter
  history, physical bus delivery, native-clock slice equivalence and output-only
  muting. Existing donor tests now verify that substitution disables recovery.
- The existing full-machine identity test includes the recovery policy. The
  existing late-rejection test used literal audio-state version 2 as invalid;
  it now uses `u32::MAX`, so the new valid version does not invalidate the test.
- `cargo test --offline --workspace`: **530 tests passed** after integration.
  The two socket tests require host loopback permission as before. The first
  host rerun exposed the hard-coded invalid version described above; adapting
  that test produced a fully passing final run.
- `cargo build --offline --release -p tgpulse`: development binary built at
  `target/release/tgpulse`; the installed `tgpulse.dev` launcher points there.

Private real-ROM verification reuses the existing 2,168-frame cabinet/input
sequence with in-memory NVRAM, then saves/restores and compares 60 frames of
audio and complete state byte-for-byte. No physical controller, host playback
or manual listening acceptance is implied by these checks.

| Mode | Nonzero Stereo Samples / 232,821 | Peak Absolute Sample | Clipped Samples | Continuation |
| --- | ---: | ---: | ---: | --- |
| Off → Procedural | 232,796 | 6,965 | 0 | 60 Frames Identical |
| VF Absent → Procedural | 232,796 | 6,965 | 0 | 60 Frames Identical |
| VF Present → Donor | 232,821 | 16,993 | 0 | 60 Frames Identical |

Off/missing-donor WAVs are byte-identical. The new VF WAV is byte-identical to
the accepted donor checkpoint WAV, not merely similar in peak/sample count.
Evidence: `/private/tmp/tgpulse-procedural-smoke.8QDYpU/`, existing private
`audio-donor.rs` harness; build/test logs use `tgpulse-procedural-*` in
`/private/tmp/`. ROMs, recordings and private reference code remain outside Git.
The remaining acceptance gate is user listening with Audio Donor = Off after
game reload/reset. No commit, push, toolkit deployment or release replacement
is included in this checkpoint.

### Alternative Audio Gains — 2026-10-04

`NetMerc Alternative Audio Gains` is an optional output preset, saved as
`netmerc_alternative_gains = on|off` (shipped default Off). It applies immediately
to the running NetMerc session, without resetting CPUs, samples or voices.
Disabling it restores the stored manual gains; other games always use those
manual gains. Master volume and output mutes remain independent.

| Actual Loaded Audio Path | MultiPCM 1 | MultiPCM 2 | FM |
| --- | ---: | ---: | ---: |
| Donor | 38% | 38% | 30% |
| Procedural / Original | 50% | 50% | 30% |

The session retains whether resource loading actually succeeded, rather than
using the currently selected donor preference (which may apply only on the next
load). An unavailable donor therefore selects the procedural/original levels.
Recovery retains its internal `0.105` normalization, existing DC filters and
50% reference-route compensation. These are alternative mixer coefficients,
not a claim of authentic cabinet gains or bit-identical reference playback:
chip normalization, filtering and saturation can still differ.

While enabled, the existing channel sliders show the effective values and
reference ticks read-only; mute checkboxes and master volume stay usable.
Frontend settings are not added to machine snapshots or ROM identity. A state
restore reapplies the currently selected output preferences, as before.
No change to state format, NVRAM, game timing or emulated audio-chip state.

Verification: `cargo test --offline --workspace` passes **531 tests**, including
actual-path preset selection, other-game isolation, manual-gain preservation
and settings persistence. Existing UI layout/reset checks pass; desktop release
build passes. The two existing networking checks require local socket access
outside the restricted sandbox. Real-ROM donor/procedural continuation is
checked with the existing private `audio-donor.rs` runner: both VF donor and
Off/procedural paths complete 2,168 frames and 60-frame byte-identical
save/restore continuation. Controlled-sequence peaks are 12,915 and 6,965,
respectively, with no clipped samples; this is not an all-game clipping claim.
Private evidence: `/private/tmp/tgpulse-alternative-audio.dMtwEm/`.
User UI/listening acceptance remains separate. No commit, push or release deployment.

### Stage 2 / City Freeze Investigation — 2026-10-04

The user's pre-transition format-4 snapshot reproduces a gameplay stall after
991 emulated frames (absolute frame 15,241). This is not a stopped scheduler:
the V60 repeats the path-normalization loop at `FBABB6..FBABFE` with a NaN in
R3, so its comparison against 1.0 never terminates the loop. The first invalid
calculation follows the final path-table word at `FAF94A` (`47EE68B8`); the next
word at `FAF94E` is the integer header `00000007`, not floating-point 7.0.
TGP division overflows to `7F800000`, then multiplication by zero produces
`7FC00000`. The same operands produce Inf/NaN through the compared MAME TGP
command path; that comparison does not establish correct hardware behavior or
reproduce a whole MAME gameplay session.

The tunnel may be a path/level reinitialization boundary. That remains a
specific hypothesis: the investigation has not proved which variables reset
there, nor that the supplied snapshot already contains an earlier divergence.
The next useful comparison must observe the actual tunnel-to-City transition,
including the path pointer, count/index and accumulated fractional progress.

Private diagnostics, not production fixes:

- The inspected reference's City option forces the mode-0 float-to-integer
  conversion at TGP firmware PC `02E1`. Applying that override to the supplied
  snapshot did not remove the stall. This negative result does not determine
  whether the option prevents an earlier divergence when starting a new game.
- The inspected reference also decodes TGP exponent 255 as finite and flushes
  exponent-zero values; overflow encoding saturates at `7FFFFFFF`. An
  independently written, private finite-format ALU experiment still reaches
  the bad table boundary at frame 15,241. Its saturated response becomes NaN
  when consumed as IEEE single by the existing V60. The initial probe stopped
  at that first NaN; it did not establish a persistent freeze for this variant.
  The extended continuation below supersedes that initial negative inference.
  No finite-format change, ROM patch, NaN replacement or unconditional loop
  escape has been integrated into the production build.
- The Fujitsu MB86232 datasheet, page 2, gives zero for exponent zero and a
  finite maximum for exponent 255. The family documentation is a useful
  hardware lead, not proof of every MB86233 operation or V60 interoperability.
  Source: [Fujitsu MB86232 Datasheet](https://ftpmirror.your.org/pub/misc/bitsavers/pdf/fujitsu/_dataSheets/MB86232_DSP_Sep89.pdf).
- Two isolated native reference boots used the same persistent cabinet seed,
  idle/centered inputs and 18,000-frame limit, with City Workaround Off/On.
  Both exited normally, producing identical final pixels and display-list
  traces, but the final scene is still the desert. **These trials did not
  establish a successful City transition**, and their persistent `.tm1save`
  files are not converted copies of the user's full-machine snapshot.

Private evidence: `/private/tmp/tgpulse-stage2-before.EMyrqS/` (baseline,
conversion and finite-format experiments) and
`/private/tmp/tekno-city-transition.XvZD5y/results-b/` (native paired trials).
The optional Windows screen capture failed; the engine's internal final
frame capture succeeded. The Windows audio endpoint was unavailable, so no
new listening claim is made. Both owned test instances were cleaned up;
the original reference NVRAM and user snapshot hashes are unchanged.
The production development executable is unchanged by this investigation.

#### Interactive Stage 2 Diagnostic — 2026-10-04

Private source/build/runtime root:
`/private/tmp/tgpulse-netmerc-stage2.pkPWUs`.
The copied workspace includes the current pending integrations, but experimental
arithmetic and observation are confined to this private copy. No production
core, launcher or executable was replaced.

Launch the first user gameplay test **from boot**, with both experiments enabled:

```sh
/private/tmp/tgpulse-netmerc-stage2.pkPWUs/test-netmerc.sh
```

Default `combined` selects City conversion mode 0 at actual firmware PC
`02E1` (`ppc`) and the independent finite-format TGP arithmetic experiment.
The V60, ROMs, clock/scheduler and save bytes are not patched. Alternatives:

```sh
/private/tmp/tgpulse-netmerc-stage2.pkPWUs/test-netmerc.sh baseline
/private/tmp/tgpulse-netmerc-stage2.pkPWUs/test-netmerc.sh city
/private/tmp/tgpulse-netmerc-stage2.pkPWUs/test-netmerc.sh finite
# Optional reproduction from the copied user snapshot:
/private/tmp/tgpulse-netmerc-stage2.pkPWUs/test-netmerc.sh combined --from-save
```

Each invocation creates a fresh `runs/<variant>.XXXXXX` directory and prints
its absolute path. Settings, input bindings, assets, NetMerc NVRAM and slot 0
are copied from a fixed seed; ROM ZIPs are read from the development ROM folder.
Only private runtime copies are written. For the first from-boot trial, do not
load slot 0. Close the game/emulator normally afterward and provide the printed
run directory. The script is foreground-only and does not stop other instances.

Evidence files:

- `run.log`: desktop/loader messages.
- `progress.log`: variant, frame, CPU registers, FIFO depths, conversion-site
  count and LCD state every 60 emulated frames.
- `first-fault.log` / `first-fault.state`: first sampled NaN in the known
  V60 normalization loop, with a bounded 4,096-event V60/TGP/FIFO history.
  **A first NaN alone is not classified as a freeze.**
- `persistent-loop.state`: emitted only after ten consecutive frame endpoints
  remain in the known normalization loop with a NaN; accompanying progress
  marker distinguishes this from a transient NaN.

The hardware-side observer is in-memory and bounded; the desktop owns log/state
file output. Diagnostic flags are excluded from snapshots, explicitly reapplied
after loading, and never repair a saved state. The ring stops at the first NaN;
this is targeted evidence, not a complete instruction trace or detector for
every possible gameplay failure.

Extended snapshot continuation:

| Variant | Known NaN at Frame 15,241 | Subsequent Observation |
| --- | --- | --- |
| Baseline | Yes | Persistent loop through ten consecutive frame endpoints |
| City Only | Yes | Persistent loop through ten consecutive frame endpoints |
| Finite Only | Yes | Returns to normal execution; display lists evolve through Frame 16,050 |
| Combined | Yes | Returns to normal execution; display lists evolve through Frame 16,050; final software frame shows the city street |

All four variants have identical serialized machine hashes at the 60-frame
pre-divergence checkpoint. Enabling diagnostic selection itself leaves snapshot
bytes unchanged. Both finite variants were observed for 809 frames after the
first NaN, rather than stopping there. This supports further gameplay testing;
it does **not** establish faithful path selection, complete Stage 2 gameplay or
correct hardware floating-point behavior. Baseline arithmetic uses the original
f32 implementation, preserving NaN behavior instead of widening its operations.

Verification: release frontend build, offline workspace checks (534 passed),
CLI/help/invalid-argument checks and native interactive launcher boot through
Frame 360 with `combined`, followed by cleanup of the owned process. The normal
development executable, user settings/input file, NVRAM and supplied snapshot
retain their original SHA-256 hashes. The temporary build/seed survives until
removed or temporary storage is cleaned; it is not a release or installation.
Private build logs, source and continuation captures are retained under the
diagnostic root; no ROM, NVRAM, state or reference binary belongs in Git.

#### New User Snapshot And Dump Latency Comparison — 2026-10-04

The user tested the private Combined build from boot: City entry briefly paused
but then continued. The new user slot is under
`/private/tmp/tgpulse-netmerc-stage2.pkPWUs/runs/combined.nDsu8w/states/netmerc.0.state`
(SHA-256 `b86cd3933aebd3a671d7a39facba7931fddba9000b92a86b6e1e625670b081a5`).
The previous development slot is unchanged. The interactive log has its first
NaN at frame 11,511, with normal execution already resumed at that frame's end.
Subsequent logs reach frame 12,360; no persistent-loop marker was emitted.

The existing private `stage2_check` example was extended with optional snapshot
and dump-style arguments, not replaced with another runner. All comparisons
load the same byte-identical private snapshot copy, starting at frame 10,378,
and hold its saved input values constant; no host controller or NVRAM file is
opened for writing. They do not replay the user's subsequent controller motion.

| Arithmetic Variant | First NaN | Result With Identical Saved Inputs |
| --- | --- | --- |
| City Only | Frame 11,511, 1,133 frames after load | Persistent V60 NaN loop; stopped at ten consecutive endpoints, frame 11,520 |
| Finite Only | Same frame | Normal execution continues through frame 12,178 |
| Combined | Same frame | Normal execution continues through frame 12,178 |

Finite Only and Combined have identical final work-RAM hashes and identical
496×384 software-rendered final images (zero differing pixels). Their complete
snapshot hashes differ: City changes conversion results and intermediate/render
state, so this is **not** full-machine equivalence or proof that City is useless
in every scene. There is no evidence in these snapshots that City is needed to
avoid the persistent freeze. This checkpoint supports the finite codec as the
effective candidate, still subject to its documented hardware/fidelity limits;
it does not justify an unconditional path-index, NaN or V60 ROM patch.

Host latency was measured separately with `Instant` in the private headless
runner, never in device emulated time:

| Combined Dump Strategy | Critical CPU Frame | Trace Formatting/Writing | Total Dump Work | Final Snapshot Hash |
| --- | --- | --- | --- | --- |
| Existing unbuffered file writes | 3.231 ms | 259.207 ms | 298.655 ms | `81ef35cd3bb8f84d` |
| 64 KiB buffered file writes | 3.285 ms | 1.018 ms | 38.188 ms | Same |
| No dump file I/O | 3.326 ms | Not performed | Not performed | Same |

The existing writer formats each of 4,096 debug events directly into a File,
causing many small synchronous writes. In the Combined trial, trace sync takes
5.045 ms, snapshot serialization 20.985 ms and snapshot file writing 13.417 ms,
in addition to the 259.207 ms trace-writing cost. Ordinary Combined frames have
a 3.124 ms median and 3.291 ms 95th percentile. Thus the reproducible ~0.30 s
host pause is principally diagnostic output, not slow execution of the
critical emulated frame. This strongly explains the observed brief pause, but
its exact duration in the user's GUI session was not recorded.

The buffered and unbuffered runs produce byte-identical first-fault logs and
snapshot SHA-256 values. Buffered/no-I/O trials also have the same final full
snapshot hash as Combined with the existing writer, demonstrating that those
host I/O strategies do not alter the observed emulated continuation.
These are individual sequential measurements, not a performance guarantee:
filesystem caches, other host activity and the unmeasured GUI/render cost can
affect elapsed time. No production or interactive-build writer was changed by
this comparison.

Canonical replay, using the retained private build and snapshot:

```sh
# flags: 1 City Only, 2 Finite Only, 3 Combined
# dump style: plain, buffered, noio; output directory must already exist
/private/tmp/tgpulse-netmerc-stage2.pkPWUs/target/release/examples/stage2_check \
  3 /private/tmp/netmerc-stage2-compare.ia121E/combined \
  /private/tmp/netmerc-stage2-compare.ia121E/before-city.state plain
```

Private evidence/source: `/private/tmp/netmerc-stage2-compare.ia121E/`, including
the pre-extension runner copy, read-only-source snapshot copy, build log,
per-variant logs, diagnostic dumps and final rendered images. The runner's
release build passed; the interactive binary, normal development executable,
original snapshot and settings were not replaced. No extra user gameplay trial
is required to discriminate these three variants at this checkpoint. Production
integration and further fidelity/scene coverage remain separately authorized
work; no commit or push was performed.

### Production Arithmetic Integration — 2026-10-04

The preceding diagnostic comparisons are historical isolated experiments. The
authorized production integration now uses one MB86233 implementation with a
board-selected arithmetic policy, not a second CPU or a diagnostic writer.
NetMerc selects the independently implemented finite codec; other Model 1 and
Model 2 boards retain the original f32 IEEE operations. No instruction timing,
FIFO, ROM, path index or V60 instruction patch is included.

`NetMerc City Workaround` is independently selectable under Settings → Machine,
default On after the user's visual acceptance below (initially Off).
Configuration: `netmerc_city_workaround = on|off`; CLI:
`--netmerc-city-workaround on|off`. It forces conversion mode 0 only at the
executing NetMerc TGP instruction `02E1` (previous PC, not advanced PC).
Changing the option affects the next matching conversion, does not reset the
machine and does not undo prior results. Reload a pre-transition state for a
controlled comparison. Reset reuses the current preference. Settings persistence
and CLI precedence follow the existing frontend implementation.

The arithmetic policy and City choice are not firmware state: serialization
excludes them, and Model 1 restore reconnects the destination board policy and
frontend preference. Existing format-4 state layout and resource identities
remain unchanged. A frontend can select City through `Config` and
`Model1System::set_netmerc_city_workaround` without a GUI, filesystem or host
clock dependency. The old diagnostic log/dump writer is not shipped, avoiding
its synchronous pause at the critical frame.

#### Reference Scope And Correctness Boundary

A read-only follow-up audit of the same installed TeknoModel1 build found the
finite decoder/encoder in the shared TGP ALU, called by its normal instruction
executor without a NetMerc game-name gate. City conversion activation is
separately NetMerc-specific. Restricting the finite policy to NetMerc here is
therefore a conservative TGPulse rollout choice, **not** the reference's scope.
This static evidence does not establish execution coverage for every reference
game or every emulator version. No proprietary implementation or binary is
included in this project.

The known NetMerc calculation (`442719AA / 00000007`, followed by multiplication
by zero) explains the persistent failure under IEEE Inf/NaN and finite saturation
avoids it. The MB86232-family documentation and the inspected reference do not
fully agree on exponent-255 interpretation. Consequently the chosen codec is
explicitly a tested compatibility path, not proven complete hardware accuracy
or an automatic general improvement for other games. Any broader rollout needs
separate evidence and authorization. Finite Only also continued through both
supplied City snapshots, while full intermediate machine state differs with
City enabled. Successful continuation does not establish correct geometry.

The subsequent user gameplay comparison found City Workaround necessary for
correct road-plane rendering in the City level. Its shipped default is now On,
while Off remains available for comparisons. Existing explicit configuration
choices are preserved: changing the default does not overwrite an Off stored
in a user's profile. The current local profile already selects On.

#### Integration Verification

- `cargo test --offline --workspace`: 539 tests passed; localhost networking
  tests required execution outside the restricted socket sandbox.
- `cargo build --offline --release -p tgpulse`: development executable built.
  Existing dependency warnings remain; no dependency installation was needed.
- Before/after headless real-ROM comparison: VR, VF, SWA, VFormula, Wing War,
  Wing War 360; Daytona (Model 2 original), VF2 (2A), Virtua Striker (2B) and
  The House Of The Dead (2C). Each build ran 1,800 frames per game from the
  same in-memory boot defaults, with idle controls and no host NVRAM writes.
  At frames 60/300/900/1,800, serialized machine hashes and cumulative PCM
  hashes match between builds for all ten games. The 2B/2C samples also cover
  unrelated coprocessor paths. This demonstrates bounded no-change
  equivalence, not a new manual gameplay or full hardware-fidelity certificate.
- Both supplied NetMerc pre-City states run another 1,800 frames with City
  Off and On, without ten consecutive endpoints in the persistent V60 NaN
  loop. The newer save ends at frame 12,178 with Finite Only full-state FNV
  `055be58910c2b0fc` and Combined `81ef35cd3bb8f84d`, exactly matching the
  isolated diagnostic comparison. Both end with work-RAM FNV
  `2b357b7291f73040`. The older save also continues, ending at frame 16,050.
- Numeric checks cover the observed overflow/zero product, codec boundaries,
  rounding and unchanged default IEEE operations. Settings on/off round-trip,
  CLI override precedence and existing all-variant machine restore/continuation
  checks cover persistence and retention of the destination City preference.

The private, headless before/after probe and build/results logs are retained at
`/private/tmp/tgpulse-city-regression.WvZPke/`; it reuses the retained pre-integration
source/cache under `/private/tmp/tgpulse-netmerc-stage2.pkPWUs/`. Canonical calls:
`baseline/target` is not assumed: baseline executable is
`/private/tmp/tgpulse-netmerc-stage2.pkPWUs/target/release/city-regression-probe`,
and the current executable is `target/release/city-regression-probe`; arguments
are `m1|m2 GAME`. The current private `netmerc` probe takes `off|on STATE_PATH`.
These probes are not installed launchers. User ROMs, settings, states and NVRAM
are read-only inputs, and no private reference code or ROM data belongs in Git.

The normal development launcher is `tgpulse.dev netmerc`, executing
`target/release/tgpulse`. No toolkit/current release is replaced. Fresh
production GUI/gameplay validation remains distinct from the prior accepted
interactive diagnostic trial. No commit or push was performed.

### Reference NetMerc Motor Audit — 2026-10-04

The same inspected TeknoModel1 build exports `MotorTriggerThumb` in its NetMerc
output descriptors with mask `04`, separately from holder/backlight/coin
outputs. An exported cabinet output is not itself proof of gamepad rumble.
The inspected direct SDL force-feedback bridge installs its drive-write hook
for `vr` and `vformula`; no NetMerc motor-to-SDL translation was found in that
path. The additional SDL rumble call in the application loop sends zero levels
when stopping effects, not a separate NetMerc effect generator. This is a
bounded audit of this engine build, not a claim about external output plugins,
other versions or physical actuator response.

There is consequently no verified richer NetMerc rumble algorithm to import.
TGPulse retains the actual port-D bit-2 boolean adapter described above;
both pad motors at 0.6 is a frontend choice, not a recovered TP coefficient.
No synthetic engine/weapon effects or new rumble code are introduced here.

### Gameplay And Remaining Validation

On 2026-10-04 the user explicitly closes four milestones: MVD tracking,
diagnostic LCD presentation, NetMerc pad rumble and substitute audio. This
acceptance supersedes the pending user checks recorded in earlier historical
checkpoints; it does not claim hardware-cycle accuracy or authentic recovery
of the missing audio ROM. Gravity stabilization and City Workaround default On;
the existing Off choices remain available. The pad adapter uses the actual
port-D bit-2 motor output and never interprets LCD traffic as motor commands.

Extended later-level gameplay and repeat-game lifecycle remain the overall
NetMerc validation scope. The bounded SRAM/service and DPRAM `20` checks are
closed; additional operator/timing work needs a concrete fault. No new CPU is
required by the proven I/O receive path. Full low-level Polhemus
peripheral emulation is optional future work only, not a next checkpoint or
completion requirement; no current practical need has been identified. Reopening
it requires a concrete benefit and explicit user approval.

Integrating a future **verified corrected audio dump** is retained as possible,
availability-dependent work: update catalogue identity/resource loading, use
original playback rather than best-effort recovery where appropriate, and verify
audio plus save-state resource compatibility. This is not an open gate for the
accepted substitute-audio milestone; a redump remains the definitive route to
authentic missing-ROM content. Coprocessor
differences must be localized to an actual incorrect calculation or command sequence; the MAME
`MACHINE_NOT_WORKING` flag alone does not identify a missing TGP function.
