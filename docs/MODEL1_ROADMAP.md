# Model 1 roadmap

Baseline: 2026-09-27. This is a source-backed work plan, not a claim that every
game is playable. Keep these changes isolated from Model 2 and from the separate
macos-emulation-toolkit and MAME projects.

## Effort and consumption planning

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

1. **Model 1 I/O board 2:** implement the actual board/firmware interface used by
   Wing War and NetMerc. Validate boot handshake, digital and analog controls,
   EEPROM persistence and per-game variants against the reference. The current
   frontend signal mapping is not a substitute for this board.
   [Register/wiring contract and checkpoints](MODEL1_IOBOARD2.md): shared
   315-5338A/ADC extraction and board-1 regression tests are complete; the
   TMPZ84C015, advanced memory map and firmware integration are still pending.
2. **YM3438 synthesis:** currently only timers/status are implemented. Add FM
   generation and correct routing beside the existing MultiPCM output; verify
   timing, levels and sound tests per title.
3. **Star Wars DSB:** implement the Z80/MPEG board and its filtered serial command
   path. Test music independently from the existing Model 1 sound board.
4. **NetMerc initialization:** with verified ROMs and I/O, validate factory NVRAM,
   startup and gameplay. MAME itself still marks NetMerc not working, so it is
   not a complete gameplay oracle.
   Review the additional Polhemus/i386SX tracking subsystem separately from
   the I/O board; the reference machine configuration includes it.

## Phase 3 — fidelity and remaining features

- Compare rendering and timing per title: clipping, moire, palette translation,
  HUD ordering, gamma and monitor modes. In particular, the Model 1 tile source
  still returns `false` for `colorxlat_written()` and identity monitor gamma;
  do not change this merely by analogy without tracing actual game writes.
- Validate VR/Virtua Formula, VF, SWA and the Wing War variants in-game; the
  README's tested-title list is not a complete compatibility matrix.
- Add Model 1 machine save states (separate from persistent NVRAM), preserving
  an in-memory API suitable for a future Model 1-only Libretro frontend.
  New integrations should inventory state and add serialization/continuation
  coverage where applicable now, without waiting for the full adapter. The
  current shared I/O chips are covered; the complete Model 1 machine is not.
- Implement cabinet link and review drive/motion-board fidelity separately from
  the existing controller rumble approximation.
- Consider checksum-aware ROM diagnostics beyond the narrowly guarded TGP
  fallback. The current general loader matches names, not expected hashes.

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
