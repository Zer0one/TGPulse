# Model 1 I/O board 2 — implementation checkpoint

Status: 2026-09-27. **Shared peripherals prepared; advanced board not yet enabled.**
This is a bounded first step of the [Model 1 roadmap](MODEL1_ROADMAP.md), not a
claim that Wing War or NetMerc now runs its real I/O firmware.

## Reference and author style

Inspected local MAME revision `bd7e0b815842ec461e8ad2538d127f3332f5c96c`;
the following reference files had no working-tree changes:

- `src/mame/sega/model1io2.cpp` and `.h`: board map, firmware and pin wiring
  (BSD-3-Clause, Dirk Best).
- `src/mame/sega/model1.cpp`: Wing War, R360 and NetMerc machine connections.
- `src/mame/sega/315_5338a.cpp`: shared I/O controller (BSD-3-Clause, Dirk Best).
- `src/devices/machine/msm6253.cpp`: ADC shift/latch behavior (BSD-3-Clause, AJR).
- `src/devices/cpu/z80/tmpz84c015.cpp`: integrated CPU peripherals and interrupts
  (BSD-3-Clause, hap).

Follow TGPulse's existing `model1io`, `sound` and `sound2a` separation: execute
original firmware on the CPU, put address decoding and cabinet wiring in the
board, keep reusable chips separate. Do not synthesize successful handshakes or
create another frontend input list. The chip reference notices and terms are
retained in the source headers and [license notice](../LICENSES/MAME-BSD-3-Clause.txt).

## Implemented in this checkpoint

- Extracted `sega3155338.rs` and `msm6253.rs` from the first-generation board.
  They are crate-private and already used by `model1io.rs`, not unused stubs.
- The 315-5338A exposes register reads and write effects (parallel output mask
  or one host byte). The board applies the effects immediately and supplies
  input reads, host memory decoding and physical output connections.
- Host addresses remain 16-bit inside the chip. The existing board, not the
  chip, wraps them to its 2 KiB MB8421 dual-port RAM.
- Corrected the seven power-on output latches from `00` to `FF`, matching MAME.
  Construction does not issue output callbacks; register writes/direction
  changes do. Command/status behavior and existing cabinet wiring are retained.
- Added 14 ROM-free tests: controller registers, direction transitions, serial
  transfers, all 256 ADC values, channel selection, first-generation memory and
  digital wiring, immediate outputs, and EEPROM write/read through board pins.

The controller still completes transfers immediately and returns status `08`
(finished, active-low ACK), like the inspected MAME implementation. No serial
transport timing/slave mode was added. ADC conversion timing remains unmodeled.

## Future Libretro integration — Model 1 first

The integration target remains the standalone emulator; a Model 1-only Libretro
frontend is a future consumer, not part of this checkpoint. Preserve the current
frontend-independent chip boundaries and an in-memory serialization path. This
is a general criterion: execution scheduling, input delivery, audio/video output,
load/reset/unload lifecycle, resource ownership and dependency portability must
also remain frontend-independent where an integration touches them. Do not add
a Libretro adapter or a speculative framework to implement this policy.

The two shared chips now serialize their complete mutable state. The 315-5338A
snapshot includes all seven output latches, direction, command, serial data and
host address; the ADC includes the partially shifted sample. Callbacks passed
to register reads are not stored in either device. Write effects are consumed
synchronously by the board and must not be replayed during restore. Tests cover
restoring a configured serial transfer and a partially read ADC conversion.

This is **device-level coverage only**, not complete Model 1 save states. Future
board snapshots must also include CPU state, RAM/DPRAM, EEPROM protocol state,
bank/output latches, cycle debt and pending timer/IRQ/serial work. Keep firmware,
host devices and diagnostic display resources outside the serialized state.
Review snapshot access as well as IRQ hooks when choosing how to integrate the
TMPZ84C015; neither requirement can be satisfied by saving its public PC alone.

The existing `savestate.rs` targets `Model2System`; do not treat it as an existing
Model 1 implementation. Eventually use an explicit, validated machine snapshot
format and test equal continuation after restore. Keep persistent NVRAM separate
from that snapshot and let the frontend own persistence; no implicit file writes
or dependency on a particular host clock/directory should be added to devices.

## Advanced board contract — next implementation

### CPU and memory map

TMPZ84C015 at **9,830,400 Hz** (`19.6608 MHz / 2`), not the existing 4 MHz Z80.
Its Z80-compatible execution core alone is insufficient: it also integrates
CTC, SIO, PIO and a watchdog.

| CPU address | Device / behavior |
| --- | --- |
| `0000–7FFF` | Bottom 32 KiB of the 64 KiB I/O firmware |
| `8000–800F` | Shared 315-5338A registers |
| `8040` | Four active-low board buttons, JP4/JP3, EEPROM DO at bit 6 |
| `8080` | DSW1 |
| `8100–810F` | Virtua Cop FPGA interface; do not infer flight controls from it |
| `8200–8207` | MSM6253; channel is address bits 0–1, bit 2 mirrors |
| `E000–EFFF` | RAM labelled backup RAM in MAME |
| `F000–FFFF` | RAM |

At `8040`, idle buttons/jumpers/unused bit give `BF | (EEPROM_DO << 6)`.
MAME maps the backup area as ordinary RAM, not an NVRAM device here. Review
firmware use before changing persistent file formats or claiming battery-backed
behavior. It is distinct from the main board's 64 KiB NVRAM and the 93C45 EEPROM.

Internal I/O ports decode the low byte (high byte mirrored):

| Port | Peripheral |
| --- | --- |
| `10–13` | Four CTC channels |
| `18–1B` | SIO, `ba_cd` register order |
| `1C–1F` | PIO, alternate register order; DSW2/DSW3 on ports A/B |
| `F0` | Watchdog mode register, reset `FB` |
| `F1` | Watchdog control |
| `F4` | CTC/SIO/PIO interrupt priority |

CTC channels 2/3 clock SIO A/B. SIO A connects CN7; SIO B connects the CN8
debug terminal. There is also an external MB3773 watchdog. Do not return fixed
"ready" values for these peripherals just to advance the firmware.

The existing `z80` 1.0.2 dependency has memory/port callbacks, cycle stepping and
vectored IRQ entry, but its I/O trait exposes no interrupt acknowledge/RETI
callback. Define and test that boundary for daisy-chain interrupt service before
claiming TMPZ84C015 fidelity. Do not install another CPU dependency implicitly.

### 315-5338A cabinet wiring

Port letters correspond to register indices A=0 through G=6.

| Port | First-generation board (unchanged) | Advanced board (to implement) |
| --- | --- | --- |
| A | EEPROM CLK7/CS6/DI5; bit 0 selects digital/analog bank | Digital IN0 |
| B | IN0 or DSW1 | Digital IN1 |
| C | IN1 or DSW2 | Digital IN2 |
| D | IN2 or DSW3 | Lamps / coin outputs |
| E | Drive read/write | Drive read/write and diagnostic LCD data latch |
| F | Lamps / coin outputs | EEPROM DI6/CLK5/CS4; diagnostic LCD control bits 0–3 |
| G | EEPROM DO7 / board buttons | Watchdog7; analog bank6; active-low comm-error LED5 |

Advanced-board bank selection changes **only the four analog channels**, not
the three digital ports. Diagnostic LCD support is a separate acceptance item;
do not conflate that panel with the main GUI. Preserve and test EEPROM edge
ordering against firmware instead of copying board 1's pin masks.

### Firmware and game connections

| Games | Firmware | ADC / board-specific connections in MAME |
| --- | --- | --- |
| Wing War / Japan / US | `epr-16891.6`, 64 KiB, CRC `a33f84d1` | 0=Stick X, 1=Stick Y, 2=Throttle |
| Wing War R360 | Same | 0/1=Stick X/Y, 2=constant zero; IN2 and drive output use R360 handlers |
| NetMerc | `epr-18021.6`, 64 KiB, CRC `5551837e` | 0=Stick X, **2=Stick Y**; other unconnected analog callbacks default `FF` |

These channel numbers describe the board pins, not new assignable GUI signals.
Keep any game-specific translation at the existing input/board boundary.
Virtua Cop uses another firmware and an FPGA upload/lightgun path; integrating
that Model 2 variant is outside this Model 1 milestone.

NetMerc also has a Polhemus tracking subsystem represented by an i386SX in
MAME's machine configuration. Its presence is a further integration concern,
not proof that MAME has complete tracking emulation. Wing War's M1COMM and R360
motion/drive behavior likewise remain separate from the base I/O board.

## Integration and acceptance sequence

1. **Done:** reusable chip implementations and board-1 regression coverage.
2. Implement the advanced memory/port map, TMPZ84C015 peripheral behavior and
   interrupt boundary; test register access, mirroring, timers and IRQ service
   independently of game ROMs.
3. Run EPR-16891/EPR-18021 firmware in an isolated board test; compare handshake,
   DPRAM and EEPROM transactions with the reference. Do not patch firmware.
4. Only then select the correct board per set in the core and enable `iocpu`
   regions currently excluded by `IOBOARD_NONE` in `tools/gen_roms_db.py`.
   Keep the existing `IoBoard` operations as the small integration boundary;
   select each board's clock rather than using the fixed board-1 constant.
5. Verify digital/analog input channels, settings persistence and independent
   Wing War/R360/NetMerc behavior through actual game tests. Do not alter
   existing bindings or user saves as a side effect of board integration.

No advanced-board firmware, ROM database selection, GUI, user configuration,
ROM archive or installed toolkit release was changed in this checkpoint.
