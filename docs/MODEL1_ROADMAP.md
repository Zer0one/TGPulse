# Model 1 roadmap

Baseline: 2026-09-27. This is a source-backed work plan, not a claim that every
game is playable. Keep these changes isolated from Model 2 and from the separate
macos-emulation-toolkit and MAME projects.

## Effort and consumption planning

### Consolidated verification — 2026-09-27

- The complete pending integration passes `cargo test --offline --workspace`:
  276 passed. The offline development
  release build also passes; the existing `block` future-compatibility warning
  remains. These checks are not a fresh manual gameplay validation.
- Wing War gameplay and throttle direction were confirmed by the user;
  general Model 1 timing audit and Z80-copy consolidation remain final,
  post-implementation activities. NetMerc/R360 are not promoted by this result.
- The frontend's Return to game menu / quit also operates in the library,
  retaining the CLI/fullscreen exception and held-chord state across game close.
  Manual UI/gamepad validation of the new library-exit path remains pending.
- No ROM, NVRAM or personal configuration is included in publication; toolkit
  and installed `current` releases are unchanged.
- DSB/MPEG is integrated for SWA/SWAJ; the user reports basic SWA playback
  working. Shared flight throttle, SWA view routing, optional sRGB correction,
  Model 1 2D palette intensity and persistent source gains are implemented.
  Master volume now also has a 100% reference and double-click reset.
- Next bounded fidelity checkpoint: investigate the reported MultiPCM1
  distortion in SWA. Equivalent-frame colour comparison, extended audio/input
  acceptance and Player 2 remain open; Z80 unification and the timing audit
  remain final post-implementation work.

Apply the preflight/checkpoint/recap agreement in [AGENTS.md](../AGENTS.md).
The table is an initial engineering estimate, not a measured cost or completion
promise. Recommend the actual available model by name at the start of each
activity; use a capable coding/reasoning model for hardware work and a lighter
one only for well-specified mechanical changes. Obtain approval before switching.

| Activity | Initial reasoning effort | Expected consumption | Checkpoint |
| --- | --- | --- | --- |
| Documentation, ROM audit, established build/test commands | low | Low | Stop at a verified report; diagnose unexpected failures separately |
| Bounded loader/NVRAM fix with a known reference | medium | Low–medium | Regression test before expanding scope |
| Timer, CPU or rendering discrepancy investigation | high | Medium–high | Reproducer and source comparison before implementation |
| Model 1 I/O board 2 | high; xhigh if protocol behavior remains ambiguous | High | Separate register map, boot handshake, controls and persistence |
| YM3438 synthesis | high | High | Choose a verified implementation/integration approach before full audio testing |
| Z80/MPEG DSB board | high; xhigh for unresolved timing/protocol analysis | High–very high | Separate serial protocol, CPU/decoder and mixing |
| NetMerc integration and gameplay diagnosis | high | High, low confidence | Verified ROMs and I/O first; reassess after boot evidence |
| Save states | high | Medium–high | State inventory, round-trip, then gameplay continuity |
| Cabinet link / drive / motion fidelity | high–xhigh | Very high, low confidence | Specify one protocol/device and observable acceptance criteria |

Confidence is medium for bounded work and low for missing hardware. New measured
runs can refine estimates, but account-wide percentage deltas are not per-task
costs. Do not backfill a consumption figure for Phase 1: no comparable start/end
usage baseline was collected for that implementation.

## Cross-cutting direction — future Model 1 Libretro core

During every integration, consider a future Libretro frontend initially limited
to Model 1: execution/lifecycle, input, audio/video, resources, portability and
save states, not serialization alone. Apply the concrete guidance in
[AGENTS.md](../AGENTS.md) where relevant, keeping changes minimal and avoiding
an unsolicited adapter or broad rewrite. This is not a claim of Libretro support.

## Phase 1 — bounded compatibility fixes

- [x] Prefer the corrected `315-5711.bin` (MAME 0.289). Recognise the complete
  legacy 8 KiB dump by SHA-1 and repair its two bad bits **in memory at load time**.
  Do not modify unknown programs or rewrite ZIPs during normal loading.
- [x] Preserve the last observed timer count when software stops either timer,
  matching MAME #15715. Cover stop/read/restart and periodic expiry.
- [x] Pass the ROM set's factory `nvram` region to Model 1 initialization.
  A complete saved user NVRAM image overrides it. Preserve the existing zero
  default for games without a factory image; do not reset users' saves.
- [x] Honour declared ROM load lengths: NetMerc reloads only 128 KiB of its
  512 KiB sound program. Previously the loader ignored that length and failed
  with a region overrun. Cover the real database record with a synthetic chip.
- [x] Supply the missing `netmerc_nvram.bin` from the author's verified MAME PR attachment.
  Wiring the region does not supply its contents or make NetMerc playable.

### ROM baseline and fallback

MAME change: [#15649](https://github.com/mamedev/mame/pull/15649), included in
[0.289](https://www.mamedev.org/?p=565).

| `315-5711.bin` | CRC32 | SHA-1 |
| --- | --- | --- |
| Legacy | `c5ddb8fc` | `9e21d3a07ffa315e0139483b664e3fa283ef4e06` |
| Corrected | `6a21f304` | `d5c61ea6e4744f10170ea556068c248bd43bb111` |

Clear bit 1 in bytes `0x22c` and `0x19f8` (little-endian instructions at PCs
`0x8b` and `0x67e`). The result matches MAME's corrected SHA-1. This is a repaired
dump, still marked BAD_DUMP by MAME, not a newly verified physical-chip dump.
Affected sets: `swa`, `swaj`, `wingwar`, `wingwaru`, `wingwarj`, `wingwar360`, `netmerc`.
Updating these ZIP members does not certify unrelated ROMs or missing devices.

NetMerc factory image: 65536 bytes, CRC32 `09866826`,
SHA-1 `411134c1e6307f2e32c3b4b372597b45b14a9834`.
Source: author-provided `netmerc_nvram.zip` in
[MAME PR #15642](https://github.com/mamedev/mame/pull/15642),
[attachment](https://github.com/user-attachments/files/29659125/netmerc_nvram.zip).
This is the initialization/calibration image supplied with that change, not a
replacement for a user's persistent gameplay NVRAM.

## Phase 2 — missing hardware (separate implementation tasks)

1. **Model 1 I/O board 2 — Wing War first:** implement the shared board with
   `wingwar`, `wingwaru` and `wingwarj` as the initial acceptance targets.
   Validate boot handshake, digital and analog controls, EEPROM persistence
   and regional variants against the reference. R360 cabinet-specific behavior
   is a separate follow-up; NetMerc is not an acceptance gate. The current
   frontend signal mapping is not a substitute for this board.
   [Register/wiring contract and checkpoints](MODEL1_IOBOARD2.md): shared
   315-5338A/ADC extraction, advanced bus map, CTC, internal watchdog and IRQ
   arbitration, CPU-to-bus wiring, PIO and a bounded asynchronous SIO subset
   are tested. Base Wing War now selects the advanced board and passes startup,
   a MAME DPRAM trace replay, physical input delivery and EEPROM reload checks.
   World/Japan attract-mode 3D frames were inspected. The user subsequently
   reported successful Wing War play/control tests, including the corrected
   throttle polarity. This does not certify every regional set, audio,
   extended play or link support. NetMerc and R360-specific work are deferred.
   Timing audit and Z80 consolidation are now final post-implementation tasks,
   not gates before YM3438 work, unless a concrete defect makes one necessary.

   **Separate timing follow-up:** audit the one-cycle DPRAM read wait for the
   original I/O-board games. It is now implemented on the advanced-board path
   to prevent Wing War's premature I/O timeout; original-board timing was kept
   unchanged and its existing five-game regression baseline still matches.
   This is part of the [systematic Model 1 timing audit](#model-1-timing-audit-against-mame)
   below, not a completed system-wide alignment.
2. **YM3438 synthesis:** Rust FM/DAC synthesis is implemented and connected to
   the production MultiPCM board. Per-title listening and level acceptance remain
   pending; do not equate synthetic reference tests with verified game audio.
   **Current checkpoint:** [audio integration contract](MODEL1_AUDIO.md).
   Output-only mute controls for MultiPCM/SCSP, FM and the integrated DSB are
   available. The user confirmed mute operation and FM sounds in VR gameplay.
   The YMFM subset inventory and isolated
   C++ reference smoke/state-continuation probe are complete; see the contract
   for exclusions and evidence limits. Recommendation remains a specialized
   Rust port, with a native wrapper only as an explicitly agreed fallback.
   The isolated Rust register/timer/Busy/state boundary now passes nine tests,
   including a pinned YMFM trace with 16,384 seeded operations and serialized
   mid-timer continuation. Clock/state
   APIs are frontend-independent; no native production dependency was added.
   Isolated operators/envelopes, eight algorithms, feedback, LFO/SSG-EG,
   special frequencies/CSM and DAC now match the pinned YMFM audio oracle for
   1,668,200 stereo frames across bounded synthetic scenarios. Mid-note serialized
   continuation also passes; 14 YM3438 tests in total. Seven new bridge/board
   tests cover fractional scheduling, conversion-state continuation, real 68000
   bus programming of a tone, mix gains/clipping and output-only FM mute.
   The timer stub is removed. Native FM is converted with a causal box filter,
   not MAME's resampler; bus timing remains instruction-granular. A 600-frame
   smoke for VR/VF/Wing War has no audio-CPU exceptions, but isolated FM is silent
   in those initial sequences; actual game listening acceptance is still open.
   New chip/converter state is serializable now; adapting the existing sound
   CPU/PCM/UART/scheduler belongs to the later complete-machine-state phase.
   **Follow-up audit:** a 1..254 sound-command sweep finds timer-only activity
   after initialization in VF/Wing War/Daytona, but **VR generates real FM**:
   eight commands reproduce it independently from a fresh sound board. The user
   subsequently confirmed FM sounds during actual VR gameplay. Exact command-to-
   event names remain unassigned, but FM is not merely unused/diagnostic code.
   The experimental timers-only option has been removed; synthesis always runs
   and only output muting remains. See the
   [audit method and limits](MODEL1_AUDIO.md#driver-use-audit--2026-09-27).
   Next: extend per-game listening acceptance; DSB integration is recorded below.
3. **Star Wars DSB:** implement the Z80/MPEG board and its filtered serial command
   path. Test music independently from the existing Model 1 sound board.
   [Source audit and integration contract](MODEL1_DSB.md) complete: use the local
   `tgpulse-z80`; receive the 68000 sound firmware's output, not raw V60 commands.
   Isolated bus/i8251/CPU boundary implemented with twelve synthetic tests,
   including mid-transfer state continuation. Isolated Layer II decoder passes synthetic and
   local SWA-data comparisons against MAME, plus history restore/truncation tests;
   joint stereo is explicitly unsupported (reference anomaly documented).
   Real firmware now runs with an opt-in clocked 68000-output serial link:
   15 controlled commands transmitted/consumed, with playback/loop/pan register
   programming and seven additional synthetic integration tests. Buffered 32 kHz playback/loop handling and
   mid-buffer restore now work in the isolated board: six SWA firmware commands
   produced PCM for 60-second probes; two looping runs reproduced their next
   32,000 stereo frames exactly after restore. Nine new synthetic tests cover
   playback, clock slicing, segment transitions and snapshot/error boundaries.
   Loader resources, error propagation, causal rate conversion/mixing and DSB mute
   are now integrated: normal SWA/SWAJ constructors enable the board. Settings
   persist `mute_dsb`; missing/wrong-size DSB chips fail explicitly. The filtered
   68000 -> DSB -> mixed signal path passes for both sets. Basic SWA playback
   is user-confirmed. Next: extended SWA/SWAJ listening/gameplay acceptance,
   including transitions and balance; no full-machine
   save-state or MAME-resampler-equivalence claim.
   Verification: 260 workspace tests and 36 release DSB tests passed; normal SWA/SWAJ
   boot probes completed 1,800 release frames each. Six musical commands produced
   nonzero isolated DSB mix for both sets; five Model 1 titles pass 120-frame CLI smoke.
   Separately investigate the existing
   V60 string-operation multiplication overflow encountered in the debug probe
   (`ops.rs`, register-28 update); do not mask it with global overflow settings.
4. **NetMerc initialization — deferred, separate milestone:** after Wing War,
   with verified ROMs and I/O, validate factory NVRAM,
   startup and gameplay. MAME itself still marks NetMerc not working, so it is
   not a complete gameplay oracle.
   Review the additional Polhemus/i386SX tracking subsystem separately from
   the I/O board; the reference machine configuration includes it. Do not require
   NetMerc's boot, diagnostic LCD or tracking work to complete the Wing War
   milestone. Shared hardware fixes remain reusable, not NetMerc-specific hacks.

Priority confirmed on 2026-09-27 against the
[official MAME driver](https://github.com/mamedev/mame/blob/master/src/mame/sega/model1.cpp):
NetMerc is declared `MACHINE_NOT_WORKING`; Wing War World/US/Japan and R360
have flags `0`. This is MAME's declared status, not a claim of perfect hardware
fidelity or a new gameplay test. NetMerc's Polhemus/i386SX subsystem is additional
to the shared I/O board and cannot be treated as a proven complete reference.

## Phase 3 — fidelity and remaining features

- [x] **Per-source output gains and mute:** GUI rows expose an absolute gain
  slider, independent Mute checkbox and source name, with a fixed reference
  tick (PCM 50%, FM 30%, DSB/SCSP 100%). Persist gains in settings and reapply
  on load/reset/state restore. This is frontend-controlled output mixing,
  not chip state or a replacement for the clipping investigation below.
  Channel sliders are limited to 0–100%, with full-height dark-blue markers
  drawn below the 50%-opaque native grab (idle and active);
  double-click resets just that gain, including while the second click is
  held. Master retains its 0–800% range, with the same marker style at 100%
  and double-click reset to 100%.
  Verification: 276 workspace tests passed, including absolute gain/default
  rounding, mute/gain independence, PCM/DSB/SCSP continuation, settings
  persistence/validation, headless ImGui row layout and double-click/held-click
  reset isolation for channels and master, draw-order/opacity and no style leakage
  into other widgets. Development release
  rebuilt; real-game listening and manual slider interaction remain pending.

- [ ] **MultiPCM1 distortion during SWA acceleration/deceleration:** user
  reports apparent clipping attributed to this source (2026-09-27). Capture
  reproducible pre/post-mix peaks and clipped samples, compare the MultiPCM
  voice accumulation, sample conversion and output with MAME, and distinguish
  source distortion from final mixer saturation. Existing nominal board gains
  match MAME; do not lower them speculatively. Basic SWA playback is now
  user-confirmed, not a certification of audio fidelity or all sequences.
- [ ] **Model 1 colour fidelity versus MAME: gameplay comparison pending.**
  Two bounded corrections are implemented (2026-09-27):
  - Optional **sRGB correction**, persisted as `srgb = on/off` (default off),
    updates immediately. The frontend samples display RGB through an sRGB
    texture only when the output surface also encodes sRGB. The non-sRGB
    fallback stays byte-preserving; core/compute pixels and GUI are unchanged.
    Applies to both Model 1 and Model 2; no host conversion is baked into the
    reusable core. Apple M4 Metal offscreen readback verifies all 256 grey
    levels with correction off/on/off and both surface
    formats (tolerance one byte). Without correction, 128 becomes about 188
    on an sRGB surface. The GPU test reports when no adapter is available;
    it was explicitly rerun outside the sandbox and exercised Metal here.
  - Model 1 tile pens now always halve RGB8 channels when palette bit 15 is
    clear, matching `model1_paletteram_w`. A board-specific trait hook keeps
    Model 2 unchanged; this affects selected 2D pens, not every polygon.
    Reference: [MAME Model 1 video source](https://github.com/mamedev/mame/blob/master/src/mame/sega/model1_v.cpp).
  Compare equivalent game frames (including SWA's dim grey backdrop) next;
  synthetic colour checks do not certify complete in-game visual fidelity.
  Verification: 267 workspace tests passed; the GPU case was also explicitly
  exercised on Metal. Exhaustive RGB555/intensity tests cover Model 1 and
  unchanged Model 2 palette expansion; settings round-trips include sRGB.
  Offline release build passed. No game screenshots or manual GUI test yet.

SWA input follow-up (2026-09-27): `swa`/`swaj` hardware button 3 now routes to
`View / Select 1` (default D-pad Down / Z), matching Sega Rally's view binding,
not to `Action 3`. SWA and Wing War now share `Throttle Up` / `Throttle Down`,
with both game families listed below the GUI entries, independent of the
driving Accelerator/Brake bindings. Up is R2 OR right stick up (W/Up keys);
Down is L2 OR right stick down (S/Down keys). Both use lower ADC for more power
and rest at 128, retaining SWA's 28..228 and Wing War's 1..255 ADC ranges.
Verification: 263 workspace tests passed, including SWA/SWAJ default-view,
shared throttle polarity/partial-travel/pedal-isolation and binding migration
tests plus the all-set digital crosstalk audit.
Offline development release build passed. The new view binding still needs
manual gameplay confirmation; colour and clipping checks above remain open.

- Compare rendering and timing per title: clipping, moire, palette translation,
  HUD ordering, gamma and monitor modes. In particular, the Model 1 tile source
  still returns `false` for `colorxlat_written()` and identity monitor gamma;
  do not change this merely by analogy without tracing actual game writes.
- Validate VR/Virtua Formula, VF, SWA and the Wing War variants in-game; the
  README's tested-title list is not a complete compatibility matrix.
- [ ] **Player 2 controls — Model 1 first, SWA as initial checkpoint.**
  Coin 2 / Start 2 alone do not establish two-player support: the frontend
  currently selects one gamepad and SWA's second stick stays centred.
  Reuse the shared signal catalogue with independent per-player bindings
  exposed in the GUI and persisted in configuration; avoid a divergent P2
  catalogue. Assign controllers explicitly to players, including disconnect /
  reconnect handling, without changing existing P1 assignments.
  Route each player's digital and analog signals to the actual per-game I/O,
  validating SWA/SWAJ's second stick, buttons and start/coin wiring against
  the reference. Keep frontend-owned player input independent of host device
  APIs for a future Libretro port. Test simultaneous inputs, no P1/P2
  crosstalk, persistence and controller reconnection, then confirm real
  two-player gameplay. Inventory other applicable games separately; Model 2
  follow-up must use the tested SM2 Libretro reference. This is local
  same-cabinet multiplayer, not cabinet-link emulation. Not implemented yet.
- Add Model 1 machine save states (separate from persistent NVRAM), preserving
  an in-memory API suitable for a future Model 1-only Libretro frontend.
  New integrations should inventory state and add serialization/continuation
  coverage where applicable now, without waiting for the full adapter. The
  current shared I/O chips are covered; the complete Model 1 machine is not.
- Implement cabinet link and review drive/motion-board fidelity separately from
  the existing controller rumble approximation.
- Consider checksum-aware ROM diagnostics beyond the narrowly guarded TGP
  fallback. The current general loader matches names, not expected hashes.

## Phase 4 — final post-implementation consolidation

User priority: keep the following work until after the feature implementations.
Bring forward only a bounded fix required by a demonstrated blocking defect;
do not start a general audit or CPU migration just because Wing War now works.

### Consolidate the Z80 implementations

The local `tgpulse-z80` adaptation and registry `z80` dependency are a temporary
isolation boundary, not the intended permanent architecture. Status: planned,
not implemented; separate review/checkpoint from new-board implementation.

- Inventory existing Z80 consumers and preserve their I/O contracts. Optional
  interrupt/clock hooks must retain compatible default behavior.
- Compare reset, IRQ/NMI, EI/HALT, RETI, cycle accounting and representative
  execution before/after migration, separately from manual audio/gameplay.
- Move consumers to one implementation only after compatibility checks pass;
  then remove the redundant dependency and reconcile the lockfile.
- User decision: converge existing consumers onto the local `tgpulse-z80`
  adaptation, also selected for the new DSB. Keep its origin and delta auditable;
  do not migrate the old consumers before the final consolidation checks.
- CPU hook/state tests and Wing War firmware validation remain prerequisites;
  NetMerc support is not. Do not migrate solely to eliminate duplication.

### Model 1 timing audit against MAME

**Status: planned, not started; deferred to final consolidation by the user.**
This is not the next activity after Wing War testing. Do not change timing
merely because this audit is on the roadmap.

- Inventory access waits across the Model 1 memory/device map: DPRAM, other
  mapped devices and coprocessor accesses. Distinguish explicit extra cycles
  from accesses for which the reference models no additional delay.
- Compare V60/TGP FIFO full/empty behavior, stall/HALT and resumption conditions,
  including interrupt and instruction-boundary interactions.
- Compare device clock ratios, fractional cycle debt, timers, interrupt delivery
  and serial scheduling between the main CPU, I/O board, TGP and sound devices.
- Start with the known DPRAM discrepancy: Wing War's advanced-board path now
  charges MAME's one-cycle read wait; the original-board games still retain
  their previous timing. Validate any extension separately across VR/Virtua
  Formula, VF and SWA rather than applying it without regression evidence.
- Record reference revisions and approximation boundaries. MAME's V60 uses an
  eight-cycle average per instruction, so agreement with MAME is not proof of
  cycle-exact hardware timing. Do not invent delays where the reference is
  incomplete, or replace real handshakes with forced ready values.

Deliver an evidence-backed discrepancy list before fixes. Apply confirmed
corrections in small, independently reviewable changes, with bounded traces,
slice-size/continuation tests where applicable and per-title regression checks.
Preserve frontend-independent emulated time and snapshot-relevant scheduling
state for a future Model 1 Libretro core. Keep automated timing/boot evidence
separate from manual controls, audio and gameplay validation.

## Verification

No ROM data is stored in Git. Automated regression tests run offline:

```sh
cargo test --offline --workspace
TGPULSE_MODEL1_TEST_ZIP="$PWD/roms/swa.zip" cargo test --offline -p tgpulse-core \
  legacy_and_corrected_tgp_load_identically -- --ignored
```

The opt-in test works with either dump revision, verifies both SHA-1 values,
idempotence and the actual loader path. Run it on each affected ZIP. Normal tests
also reject an unknown program even if its two instruction words match the old
dump, and cover factory NVRAM precedence and timer register behavior.

Manual acceptance still needed: VF hair/physics and gameplay after timer changes,
SWA graphics/collision behavior with the repaired program, audio and extended play.
Passing unit tests or `--list` alone does not establish those results.

The Model 2 Manx TT sound-board selection issue is outside this Model 1 change.
Publication and toolkit/current release replacement require separate requests.

### Local verification — 2026-09-27

- 95 workspace tests passed; the one ROM-dependent test is excluded by default
  and was explicitly run successfully on all seven affected sets, both original
  and updated ZIPs (14 runs).
- Updated only `315-5711.bin` in the seven local `roms/` archives. All other member
  hashes were verified unchanged. Original ZIPs are retained under
  `roms/.backup-315-5711-20260927/`; the separate MAME collection is unchanged.
- A size scan across all 100 local sets found NetMerc's sound reload to be the
  only load where a present ZIP member exceeds the database's declared length.
- Offline release build passed. `tgpulse.dev --list` lists 100 sets; this is a
  launcher check, not a completeness/CRC audit.
- VR, VF and SWA each executed 120 debugger frames without a crash, with an
  isolated temporary working directory and no user NVRAM writes. This short
  smoke test does not validate a complete boot, visuals, sound or gameplay.
- These checks precede publication; publication revisions are recorded in Git
  history. No toolkit release update was performed.

### NetMerc factory NVRAM recovery — 2026-09-27

- The inspected local Model 1 and MAME archives lacked the file. Downloaded the
  author's separate attachment above and verified length, CRC32 and SHA-1 against
  the local MAME driver before installation.
- Added only `netmerc_nvram.bin` to local `roms/netmerc.zip`; every pre-existing
  member's SHA-1 was unchanged, including the corrected TGP program. Backup:
  `roms/.backup-netmerc-nvram-20260927/netmerc.zip`.
- In an isolated debugger run, read all 65536 bytes at `0x400000` before execution:
  their SHA-1 matched the factory image exactly. Then executed 120 frames without
  a crash. `tgpulse.dev --list` now reports NetMerc with no missing database files.
- No existing user saves, MAME archives, toolkit files or executable changed.
  No new claim of playable NetMerc: I/O board 2 and further integration are still
  outstanding. This recovery required no emulator code changes.

### I/O board 2 shared-peripheral checkpoint — 2026-09-27

- [Implementation contract](MODEL1_IOBOARD2.md) records the MAME revision,
  memory/port maps, firmware, cabinet wiring and remaining CPU/IRQ work.
- Extracted the 315-5338A and MSM6253 into crate-private reusable modules used
  by the existing board. Corrected power-on output latches to `FF` per MAME;
  no advanced-board firmware or new game support is enabled yet.
- Added 14 ROM-free tests. `cargo test --offline --workspace`: 109 passed,
  one ROM-dependent test ignored (not rerun for this peripheral-only change).
- `cargo build --offline --release -p tgpulse` passed. The existing dependency
  warning about future Rust compatibility of `block` 0.1.6 remains unrelated.
- Compared the previous and new release builds on `vr`, `vformula`, `vf`, `swa`
  and `swaj`, each in an isolated temporary working directory. At frame 120,
  complete debugger output matched byte-for-byte: `state`, 64 KiB at `400000`
  and the 4 KiB V60 window at `C00000` containing the 2 KiB dual-port RAM.
  This is a bounded regression check, not complete CPU-state equality or proof
  of gameplay/audio/rendering correctness.
- The development executable is `target/release/tgpulse`. No user ROMs, saves,
  bindings, MAME checkout or toolkit-managed release was modified. Publication
  was subsequently authorized; publication revisions are recorded in Git.

### Libretro-oriented integration policy — 2026-09-27

- Recorded the general Model 1-first architecture criterion in `AGENTS.md` and
  the board integration contract: frontend-independent execution, lifecycle,
  inputs, audio/video, resources and portability, including (not limited to)
  serialization. No Libretro adapter or complete machine save-state claim.
- Added serialization for the two extracted I/O chips using existing serde /
  bincode dependencies. Two additional tests restore a configured host transfer
  and a partially consumed ADC sample, then verify identical continuation.
- Re-ran `cargo test --offline --workspace`: 111 passed, one opt-in ROM-dependent
  test ignored. No runtime behavior or persistent user-file format was changed
  by these serialization derives; the snapshot APIs are still crate-private.

### I/O board 2 bus / CTC / interrupt checkpoint — 2026-09-27

- Added an isolated advanced-board bus using the existing 315-5338A, MSM6253
  and EEPROM components. ROM/RAM mapping, physical pin wiring, ADC mirrors,
  EEPROM protocol and host dual-port RAM access are covered without game ROMs.
- Implemented four-channel CTC timing/counters and nested IRQ service, the
  TMPZ84C015 priority register/port mirrors and internal watchdog. Clocks are
  emulated integers; CTC output edges retain their order and clock offsets.
- Added peripheral-bus snapshots without firmware/host resources, plus tests
  for continued execution across snapshots and different time-slice lengths.
  The bus remains independent of the desktop frontend and does not own a CPU.
- Inspected Z80 1.0.2 and recorded the remaining CPU integration requirements:
  explicit acknowledge/RETI hooks, canonical RETI IFF behavior and complete
  CPU state access. The installed dependency and all existing game execution
  paths are unchanged. SIO/PIO and other known missing devices fail explicitly
  at the new bus boundary rather than returning an invented ready status.
- Added 28 ROM-free tests. `cargo test --offline --workspace`: 139 passed,
  one opt-in ROM-dependent test ignored. Offline release build passed; no new
  dependencies were installed. The pre-existing `block` warning remains.
- At 120 frames, the same debugger state, 64 KiB main NVRAM and 2 KiB shared
  RAM snapshots still match the earlier baseline for `vr`, `vformula`, `vf`,
  `swa` and `swaj`. This verifies that the unchanged board-1 game path remains
  stable in this bounded check, not that the new board firmware has booted.
- No advanced-board firmware has been run and no new game support is enabled.
  The next checkpoint is the CPU adaptation and remaining SIO/PIO work before
  Wing War firmware handshake testing. Changes remain local pending publication
  authorization; MAME, toolkit releases and user ROMs/saves were not modified.

### I/O board 2 isolated Z80 checkpoint — 2026-09-27

- Added local `tgpulse-z80`, based on the already installed MIT-licensed Z80
  1.0.2 source. Original authors, source hash, minimal code delta and limitations
  are documented in [the component README](../crates/z80/README.md).
- Added live interrupt acknowledgement, canonical RETI notification/IFF
  restoration, integer emulated-clock callbacks and typed CPU snapshot/restore.
  Corrected EI delaying NMI in this local adaptation only. The original registry
  dependency and the running first-generation I/O board remain unchanged.
- Added 18 synthetic integration tests, all passing in development and release
  profiles. State tests verify registers and identical continued execution at
  16 cut points through a block transfer, IM2 service, port I/O and HALT.
- `cargo test --offline --workspace`: 157 passed, one opt-in ROM-dependent test
  ignored. `cargo build --offline --release -p tgpulse` passed. The existing
  `block` 0.1.6 future-compatibility warning remains unrelated.
- At 120 debugger frames, `vr`, `vformula`, `vf`, `swa` and `swaj` still match
  the preceding baseline byte-for-byte for reported CPU/FIFO state, 64 KiB
  main NVRAM and the 4 KiB window containing shared RAM. Runs used an isolated
  temporary working directory and did not write user saves. This is a bounded
  old-path regression check, not a gameplay or new-firmware boot test.
- Checkpoint reached before SIO/PIO: CPU-to-bus wiring, combined board/scheduler
  snapshots and firmware execution are still pending. IM0 timing, undocumented
  RETI aliases and asserted-NMI behavior are not certified by this adaptation;
  it remains instruction-stepped, not a T-state-accurate bus implementation.
- Development executable: `target/release/tgpulse`. No toolkit installation,
  ROM/save/configuration change, dependency download, commit or push performed.
  Z80 unification remains the separate follow-up above, after board validation.

### I/O board 2 CPU/SIO/PIO checkpoint — 2026-09-27

- Connected the adapted CPU to the actual advanced-board bus. CTC/SIO/PIO
  acknowledge and RETI use live priority/service state; unsupported operations
  latch a diagnostic fault and prevent later instructions from running.
- Added PIO register/bit-control operation and a bounded asynchronous SIO model
  clocked from CTC2/3, with FIFO/error handling and timestamped serial pin
  events. Unsupported modes fail explicitly. This is not full SIO fidelity;
  supported formats and remaining timing/protocol limits are documented in
  [the integration contract](MODEL1_IOBOARD2.md#fourth-checkpoint--cpubus-wiring-and-bounded-siopio).
- Added combined CPU/peripheral/scheduler snapshots. Tests check continuation
  inside interrupt service and serial transfers, integer cycle debt, no replayed
  output events, and identical results for whole versus one-clock time slices.
- 21 new ROM-free tests; `cargo test --offline --workspace`: 178 passed,
  one opt-in ROM-dependent test ignored. Release-profile core tests also passed
  (77 passed, the same opt-in test ignored). Offline release build passed. The
  existing `block` 0.1.6 warning is unrelated.
- At 120 frames the same debugger CPU/FIFO and memory snapshots match the
  previous baseline for `vr`, `vformula`, `vf`, `swa` and `swaj`. This checks
  unchanged first-generation game paths, not advanced-board gameplay.
- First isolated, hash-verified firmware probes: Wing War executes 10,000,002
  clocks but remains polling `F080=01` for `02` at `0830`; NetMerc stops on its
  diagnostic LCD write at instruction `03A9` (memory `8005`, 121479 clocks).
  Neither established a complete handshake; main-board/serial peers were absent.
  No firmware wait was patched or missing-device error ignored.
- **Next (priority clarified):** diagnose Wing War's wait against the reference.
  Add the diagnostic LCD only if that path proves necessary for Wing War;
  defer NetMerc-specific blockers to its separate milestone. Keep game selection
  disabled until the corresponding title's initialization/handshake checks pass.
- No ROM/save/configuration changes, toolkit deployment, new software download,
  commit or push. Development executable remains `target/release/tgpulse`.

### Wing War reference-trace checkpoint — 2026-09-27

- Diagnosed `F080=01` without changing emulation code or patching the firmware.
  The earlier ~1.017-second probe ended during a normal EEPROM read sequence.
  TGPulse reaches `F080=02` at 21,305,943 board clocks (~2.167352600 s);
  the installed MAME 0.289 binary reaches the same transition at ~2.167353312 s.
  Both complete the same 64-word blank-EEPROM buffer and clear the same transfer
  state. This is not a general cycle-accuracy or full-machine equivalence claim.
- The isolated TGPulse board runs for 20 seconds without a bus fault; normal
  Wing War initialization did not need the diagnostic LCD. With no main-board
  requests supplied, DPRAM remains zero and the firmware stays at state `02`.
  The fresh MAME three-second trace continues into subsequent states with its
  main CPU present. No successful host exchange is synthesized in TGPulse.
- See [the comparison and limits](MODEL1_IOBOARD2.md#wing-war-eeprom-initialization-comparison--2026-09-27)
  for reference binary/source versions, trace points and isolated test setup.
  Offline core rebuild passed; workspace tests remain 178 passed, one opt-in
  ROM-dependent test ignored. No production code or release binary changed.
- **Next:** validate the main-board/DPRAM request-response contract against
  Wing War, then wire the advanced board and its clock into the existing system
  boundary. World/US/Japan boot, controls and EEPROM persistence remain pending;
  R360 and NetMerc remain separate. No commit, push or deployment performed.

### Wing War motherboard integration checkpoint — 2026-09-27

- Replayed 619 real MAME host writes against the isolated board: sampled
  firmware states, final 2 KiB DPRAM and 128-byte EEPROM match the reference.
- Added the narrow `model1board` boundary and enabled firmware/clock selection
  for World/US/Japan only. Inputs and NVRAM use existing interfaces; no new GUI
  controls. Advanced snapshots include the fractional V60/board clock ratio.
- Fixed the integrated I/O timeout by modeling MAME's one-cycle DPRAM read wait
  on this new path. Original-board timing remains isolated pending its audit.
  Also fixed V60 scaled negative-index overflow in debug builds without changing
  release arithmetic. Unsupported board accesses stop with a reported error.
- All three base sets pass boot and in-memory NVRAM reload; digital/analog
  changes reach the expected DPRAM bytes. World/Japan attract-mode 3D frames
  were inspected. Manual controls, service-menu option edits, audio and extended
  gameplay remain unvalidated. R360, NetMerc and link play remain separate.
- 187 tests passed, one ROM opt-in ignored; offline release build passed.
  The five existing Model 1 debugger/memory baselines remain byte-identical.
  See [the detailed evidence and limits](MODEL1_IOBOARD2.md#wing-war-dpram-and-motherboard-integration--2026-09-27).
- Development binary: `target/release/tgpulse`. No user saves/settings/ROM ZIPs,
  MAME or toolkit installations changed; no commit/push performed.
