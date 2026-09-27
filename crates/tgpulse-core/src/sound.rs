//! Sega Model 1 sound board (`segam1audio`), as fitted to Daytona.
//!
//! A 68000 at 10MHz with its own program ROM, two MultiPCM samplers, a YM3438
//! for FM, and an i8251 UART that is the only wire back to the i960. The main
//! board sends it one byte at a time and it does the rest on its own.
//!
//! This is emulated rather than reimplemented because the program is the game's
//! own: `epr-16720`/`epr-16721` are Daytona's sound driver, so an HLE would
//! mean rewriting that driver by hand for every Model 2 title. Same reasoning as
//! the TGP.
//!
//! Layout:
//!
//! ```text
//!   000000-03ffff program ROM
//!   080000-09ffff mirror of the upper ROM socket (sndcpu + 0x20000)
//!   c20000-c20003 i8251 UART        (odd bytes only)
//!   c40000-c40007 MultiPCM 1        (odd bytes only)
//!   c50000-c50001 MultiPCM 1 bank
//!   c60000-c60007 MultiPCM 2        (odd bytes only)
//!   c70000-c70001 MultiPCM 2 bank
//!   d00000-d00007 YM3438            (odd bytes only)
//!   f00000-f0ffff work RAM
//! ```

use crate::config::AudioMutes;
use crate::multipcm::MultiPcm;
use m68000::cpu_details::Mc68000;
use m68000::exception::{Exception, Vector};
use m68000::memory_access::MemoryAccess;
use m68000::M68000;
mod fm;
use fm::FmPath;
pub use fm::FmPathState;

/// 20MHz crystal divided by two.
pub const SND_CPU_HZ: u32 = 10_000_000;

/// Implemented audio outputs, not hardware enable/disable switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSource {
    MultiPcm1,
    MultiPcm2,
    Ym3438,
    Scsp,
}

pub const MULTIPCM_SOURCES: &[AudioSource] = &[
    AudioSource::MultiPcm1,
    AudioSource::MultiPcm2,
    AudioSource::Ym3438,
];
pub const SCSP_SOURCES: &[AudioSource] = &[AudioSource::Scsp];

#[cfg(test)]
mod mute_tests {
    use super::*;

    fn sounding_board(chip: usize) -> SoundSystem {
        let mut program = vec![0; 16];
        program[..4].copy_from_slice(&0x00f0fff0u32.to_be_bytes());
        program[4..8].copy_from_slice(&8u32.to_be_bytes());
        program[8..12].copy_from_slice(&[0x4e, 0x71, 0x60, 0xfc]); // NOP; BRA
        let mut samples = vec![0; 0x140];
        // Instrument 0: signed 8-bit waveform at 0x100, loop 0..64,
        // instant attack, sustained envelope, no LFO.
        samples[..12].copy_from_slice(&[0, 1, 0, 0, 0, 0xff, 0xc0, 0, 0xf0, 0, 0xff, 0]);
        for i in 0..64 {
            samples[0x100 + i] = (i as i8 * 2 - 64) as u8;
        }
        let mut sound = SoundSystem::new(program, samples.clone(), samples);
        for (reg, value) in [(0, 0), (1, 0), (2, 0), (3, 0x10), (5, 1), (4, 0x80)] {
            sound.board.pcm[chip].write(2, reg);
            sound.board.pcm[chip].write(0, value);
        }
        for (address, value) in [(0x24, 255), (0x25, 3), (0x27, 5)] {
            sound.board.ym.write(0, address);
            sound.board.ym.write(1, value);
        }
        sound
    }

    #[test]
    fn each_multipcm_mute_changes_only_output_and_preserves_continuation() {
        for chip in 0..2 {
            let mut reference = sounding_board(chip);
            let mut muted = sounding_board(chip);
            muted.set_mutes(AudioMutes {
                multipcm1: chip == 0,
                multipcm2: chip == 1,
                ..AudioMutes::default()
            });
            reference.run(70_013, SND_CPU_HZ);
            muted.run(70_013, SND_CPU_HZ);
            assert!(!reference.samples.is_empty());
            assert!(reference.samples.iter().any(|&s| s != (0, 0)));
            assert_eq!(reference.samples.len(), muted.samples.len());
            assert!(muted.samples.iter().all(|&s| s == (0, 0)));
            assert_eq!(reference.cpu.regs.pc, muted.cpu.regs.pc);
            assert_eq!(
                reference.board.pcm[chip].active_samples(),
                muted.board.pcm[chip].active_samples()
            );
            assert_eq!(reference.board.ym.read(0), muted.board.ym.read(0));
            assert_ne!(muted.board.ym.read(0) & 1, 0, "YM timer still runs");
            reference.samples.clear();
            muted.set_mutes(AudioMutes::default());
            assert!(muted.samples.is_empty());
            reference.run(50_003, SND_CPU_HZ);
            muted.run(50_003, SND_CPU_HZ);
            assert_eq!(
                reference.samples, muted.samples,
                "unmute must not restart/freeze the chip"
            );
        }
    }

    #[test]
    fn muting_the_other_multipcm_does_not_change_gain() {
        for chip in 0..2 {
            let mut reference = sounding_board(chip);
            let mut muted = sounding_board(chip);
            muted.set_mutes(AudioMutes {
                multipcm1: chip != 0,
                multipcm2: chip != 1,
                ..AudioMutes::default()
            });
            reference.run(70_013, SND_CPU_HZ);
            muted.run(70_013, SND_CPU_HZ);
            assert_eq!(reference.samples, muted.samples);
        }
    }

    fn fm_dac(sound: &mut SoundSystem) {
        // Use the actual odd-byte bus map, not a second control path.
        for (address, data) in [(0x2b, 0x80), (0x2a, 255)] {
            sound.board.write8(0xd00001, address);
            sound.board.write8(0xd00003, data);
        }
    }

    #[test]
    fn fm_mute_is_output_only_and_unmute_continues_identically() {
        let mut reference = sounding_board(0);
        let mut muted = sounding_board(0);
        for board in [&mut reference, &mut muted] {
            fm_dac(board);
            board.set_mutes(AudioMutes {
                multipcm1: true,
                multipcm2: true,
                ..AudioMutes::default()
            });
        }
        muted.set_mutes(AudioMutes {
            multipcm1: true,
            multipcm2: true,
            ym3438: true,
            ..AudioMutes::default()
        });
        reference.run(70_013, SND_CPU_HZ);
        muted.run(70_013, SND_CPU_HZ);
        assert!(reference.samples.iter().any(|s| *s != (0, 0)));
        assert!(muted.samples.iter().all(|s| *s == (0, 0)));
        assert_eq!(reference.snapshot_fm_path(), muted.snapshot_fm_path());
        reference.samples.clear();
        muted.set_mutes(AudioMutes {
            multipcm1: true,
            multipcm2: true,
            ..AudioMutes::default()
        });
        reference.run(50_003, SND_CPU_HZ);
        muted.run(50_003, SND_CPU_HZ);
        assert_eq!(reference.samples, muted.samples);
    }

    #[test]
    fn mixed_stream_and_cpu_are_independent_of_main_slice_size() {
        for hz in [16_000_000, 25_000_000] {
            let mut whole = sounding_board(0);
            let mut sliced = sounding_board(0);
            fm_dac(&mut whole);
            fm_dac(&mut sliced);
            whole.run(100_003, hz);
            for _ in 0..100_003 {
                sliced.run(1, hz);
            }
            assert_eq!(whole.cpu.regs.pc, sliced.cpu.regs.pc);
            assert_eq!(whole.remainder, sliced.remainder);
            assert_eq!(whole.main_fraction, sliced.main_fraction);
            assert_eq!(whole.samples, sliced.samples);
            assert_eq!(whole.snapshot_fm_path(), sliced.snapshot_fm_path());
            assert!(!whole.samples.is_empty());
        }
    }

    #[test]
    fn bus_ports_follow_ymfm_and_do_not_wire_fm_irq_to_uart() {
        let mut sound = sounding_board(0);
        sound.board.write8(0xd00000, 0x2a); // unused high byte
        assert_eq!(sound.board.ym_writes, 0);
        fm_dac(&mut sound);
        assert_eq!(sound.board.ym_writes, 4);
        assert_eq!(sound.board.read8(0xd00000), 0);
        assert_ne!(sound.board.read8(0xd00001) & 0x80, 0);
        assert_eq!(sound.board.read8(0xd00005), 0);
        sound.render_cycles(500);
        assert_eq!(sound.board.read8(0xd00001) & 0x80, 0);
        assert_ne!(sound.board.read8(0xd00001) & 1, 0);
        assert!(!sound.irq_pending);
    }

    #[test]
    fn mixer_keeps_reference_gains_and_clips_final_sum() {
        assert_eq!(
            mix((1000, -1000), (2000, -2000), [1000, -1000], [false; 3]),
            (1800, -1800)
        );
        assert_eq!(
            mix(
                (1000, -1000),
                (2000, -2000),
                [1000, -1000],
                [false, false, true]
            ),
            (1500, -1500)
        );
        assert_eq!(
            mix(
                (1000, -1000),
                (2000, -2000),
                [1000, -1000],
                [true, false, false]
            ),
            (1300, -1300)
        );
        assert_eq!(
            mix(
                (100000, -100000),
                (100000, -100000),
                [32768, -32896],
                [false; 3]
            ),
            (32767, -32768)
        );
    }

    #[test]
    fn actual_68000_writes_produce_fm_through_the_board_mixer() {
        let mut rom = vec![0; 8];
        rom[..4].copy_from_slice(&0x00f0fff0u32.to_be_bytes());
        rom[4..8].copy_from_slice(&8u32.to_be_bytes());
        let mut registers = vec![(0xb0, 7), (0xb4, 0xc0)];
        for slot in [0, 4, 8, 12] {
            registers.extend([
                (0x30 + slot, 1),
                (0x40 + slot, 32),
                (0x50 + slot, 31),
                (0x80 + slot, 15),
            ]);
        }
        registers.extend([(0xa4, 0x22), (0xa0, 0x69), (0x28, 0xf0)]);
        let writes = registers.len() * 2;
        for (reg, data) in registers {
            for (address, value) in [(0x00d00001u32, reg), (0x00d00003, data)] {
                rom.extend(0x13fcu16.to_be_bytes()); // MOVE.B #imm,abs.l
                rom.extend((value as u16).to_be_bytes());
                rom.extend(address.to_be_bytes());
            }
        }
        rom.extend(0x60feu16.to_be_bytes()); // BRA.S self
        let mut sound = SoundSystem::new(rom, vec![], vec![]);
        sound.run(100_000, SND_CPU_HZ);
        assert_eq!(sound.board.ym_writes, writes as u64);
        assert_eq!(sound.exception_counts.iter().sum::<u64>(), 0);
        assert!(sound.samples.iter().any(|&(l, r)| l > 0 && r > 0));
        assert!(sound.samples.iter().any(|&(l, r)| l < 0 && r < 0));
    }
}

/// The board's RAM: the reference maps 0xf00000-0xf0ffff, noting the real PCB carries
/// two 8Kx8 SRAMs.
const SND_RAM_SIZE: usize = 0x10000;

/// i8251 status bits the driver polls. TxRDY and TxEMPTY are always true here:
/// we consume a byte the instant it is written, so the transmitter is never
/// busy. RxRDY is raised only when the main board has actually sent something.
const UART_TX_RDY: u8 = 0x01;
const UART_RX_RDY: u8 = 0x02;
const UART_TX_EMPTY: u8 = 0x04;

/// The board's memory map and devices, everything except the 68000 itself.
pub struct SoundBoard {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    /// The two samplers. Between them they carry Daytona's entire mix bar the
    /// YM3438's FM (see `ym_writes`).
    pub pcm: [MultiPcm; 2],

    // --- i8251, main board -> sound board ---
    /// Bytes received from the i960 and not yet read by the driver. A queue,
    /// not a single slot: back-to-back command bytes (VF's sound handshake
    /// sends three in a row) must not overwrite each other.
    pub rx: std::collections::VecDeque<u8>,
    /// How many bytes the main board has sent us.
    pub rx_count: u64,
    /// How many of those the driver actually collected from the data register.
    pub rx_read_count: u64,
    /// Byte the driver sent back, waiting for the i960 to collect it.
    pub tx: Option<u8>,

    /// YM3438 bus traffic for diagnostics (including address writes).
    pub ym_writes: u64,
    ym: FmPath,
}

impl SoundBoard {
    pub fn new(rom: Vec<u8>, pcm1: Vec<u8>, pcm2: Vec<u8>) -> Self {
        Self {
            rom,
            ram: vec![0; SND_RAM_SIZE],
            pcm: [
                MultiPcm::new(pcm1, SND_CPU_HZ as f32),
                MultiPcm::new(pcm2, SND_CPU_HZ as f32),
            ],
            rx: std::collections::VecDeque::new(),
            rx_count: 0,
            rx_read_count: 0,
            tx: None,
            ym_writes: 0,
            ym: FmPath::new(),
        }
    }

    /// The i960 has put a byte on the wire.
    pub fn uart_send(&mut self, data: u8) {
        self.rx.push_back(data);
        if self.rx.len() > 8 {
            self.rx.pop_front();
        }
        self.rx_count += 1;
        log::trace!(target: "sound", "main -> driver: {:02X}", data);
    }

    /// i8251 mode/command register. The driver's framing does not matter to us
    /// -- we move whole bytes -- but a command with the reset bit set clears
    /// anything in flight, which does.
    pub fn uart_control(&mut self, val: u8) {
        log::trace!(target: "sound", "main uart ctl: {:02X}", val);
        // Command register bit 6 = internal reset.
        if val & 0x40 != 0 {
            self.rx.clear();
            self.tx = None;
        }
    }

    /// True while the driver has a byte waiting to be read back.
    pub fn uart_rx_ready(&self) -> bool {
        self.tx.is_some()
    }

    /// True while the main board has a byte waiting for the driver.
    pub fn uart_rx_full(&self) -> bool {
        !self.rx.is_empty()
    }

    /// True while the UART can accept another byte for the board.
    ///
    /// Always, here: `uart_send` hands the byte over on the spot, so the
    /// transmitter is never busy. This matters more than it looks -- the main
    /// board's sound interrupt is asserted on TxRDY *or* RxRDY, so this line is
    /// what keeps the game's sound task running at all.
    pub fn uart_tx_ready(&self) -> bool {
        true
    }

    fn uart_status(&self) -> u8 {
        let mut s = UART_TX_RDY | UART_TX_EMPTY;
        if !self.rx.is_empty() {
            s |= UART_RX_RDY;
        }
        s
    }

    /// Reads one byte of the board's address space. The 68000's byte lanes are
    /// handled by the caller; this takes a flat address.
    fn read8(&mut self, addr: u32) -> u8 {
        let a = addr & 0xffffff;
        match a {
            0x000000..=0x03ffff => self.rom.get(a as usize).copied().unwrap_or(0xff),
            // Mirror of the upper ROM socket.
            0x080000..=0x09ffff => self
                .rom
                .get((a - 0x080000 + 0x20000) as usize)
                .copied()
                .unwrap_or(0xff),
            0xf00000..=0xf0ffff => self.ram[(a - 0xf00000) as usize],

            // i8251: even register = data, odd = status (odd bytes of the word).
            0xc20001 => {
                if !self.rx.is_empty() {
                    self.rx_read_count += 1;
                }
                let v = self.rx.pop_front().unwrap_or(0);
                log::trace!(target: "sound", "driver reads cmd: {:02X}", v);
                v
            }
            0xc20003 => self.uart_status(),

            0xc40001..=0xc40007 if a & 1 == 1 => self.pcm[0].read(),
            0xc60001..=0xc60007 if a & 1 == 1 => self.pcm[1].read(),
            // YM3438: address/status A, data A, address/status B, data B,
            // all on the low byte lane of successive 16-bit words.
            0xd00001..=0xd00007 if a & 1 == 1 => self.ym.read(((a - 0xd00001) >> 1) as u8),
            _ => 0,
        }
    }

    fn write8(&mut self, addr: u32, val: u8) {
        let a = addr & 0xffffff;
        match a {
            0xf00000..=0xf0ffff => self.ram[(a - 0xf00000) as usize] = val,

            0xc20001 => {
                log::trace!(target: "sound", "driver -> main: {:02X}", val);
                self.tx = Some(val);
            }
            // Mode/command register: the driver configures the UART here. We
            // have no framing to configure, so there is nothing to keep.
            0xc20003 => {}

            0xc50000..=0xc50001 => self.pcm[0].set_bank(val as u32),
            0xc70000..=0xc70001 => self.pcm[1].set_bank(val as u32),
            0xc40012..=0xc40013 => {} // the reference: nopw

            // The chips sit on the odd byte lanes (the reference: umask16(0x00ff)), so
            // 68000 address 0xc40001 is chip offset 0, 0xc40003 offset 1,...
            0xc40001..=0xc40007 if a & 1 == 1 => self.pcm[0].write((a - 0xc40001) >> 1, val),
            0xc60001..=0xc60007 if a & 1 == 1 => self.pcm[1].write((a - 0xc60001) >> 1, val),
            0xd00001..=0xd00007 if a & 1 == 1 => {
                self.ym_writes += 1;
                self.ym.write(((a - 0xd00001) >> 1) as u8, val);
            }
            0x000000..=0x09ffff => {} // ROM
            _ => {}
        }
    }
}

impl MemoryAccess for SoundBoard {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        Some(self.read8(addr))
    }
    fn get_word(&mut self, addr: u32) -> Option<u16> {
        // The 68000 is big endian.
        Some(((self.read8(addr) as u16) << 8) | self.read8(addr.wrapping_add(1)) as u16)
    }
    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        self.write8(addr, value);
        Some(())
    }
    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        self.write8(addr, (value >> 8) as u8);
        self.write8(addr.wrapping_add(1), value as u8);
        Some(())
    }
    fn reset_instruction(&mut self) {}
}

/// The board with its CPU attached.
pub struct SoundSystem {
    pub cpu: M68000<Mc68000>,
    pub board: SoundBoard,
    /// Cycle budget carried between slices, since the 68000 runs at its own
    /// clock rather than the i960's.
    remainder: i64,
    /// The UART's rxrdy line is wired to the 68000's IRQ 2.
    irq_pending: bool,
    /// Main-CPU to sound-CPU fractional conversion; callers keep main Hz fixed.
    main_fraction: u64,
    muted: [bool; 3],
    /// Rendered stereo output at the chip rate, drained by the front end.
    /// Headless tools never drain it, so it is capped rather than unbounded.
    pub samples: std::collections::VecDeque<(i16, i16)>,
    /// Exceptions raised by executed opcodes, indexed by vector. IRQ2 is
    /// injected separately and therefore does not appear here.
    pub exception_counts: [u64; 256],
}

/// Upper bound on buffered audio (~2s) so headless runs don't accumulate it.
const MAX_BUFFERED_SAMPLES: usize = 90_000;

fn mix(pcm1: (i32, i32), pcm2: (i32, i32), fm: [i32; 2], muted: [bool; 3]) -> (i16, i16) {
    let side = |a: i32, b: i32, fm: i32| {
        let a = if muted[0] { 0 } else { a.clamp(-32768, 32767) };
        let b = if muted[1] { 0 } else { b.clamp(-32768, 32767) };
        let fm = if muted[2] { 0 } else { fm };
        // MAME board gains: MultiPCM 0.5 each, FM 0.30. Clip only the final mix
        // after the existing per-MultiPCM clamp; never normalize for muted chips.
        (((a + b) * 5 + fm * 3) / 10).clamp(-32768, 32767) as i16
    };
    (side(pcm1.0, pcm2.0, fm[0]), side(pcm1.1, pcm2.1, fm[1]))
}

impl SoundSystem {
    pub fn new(rom: Vec<u8>, pcm1: Vec<u8>, pcm2: Vec<u8>) -> Self {
        let mut board = SoundBoard::new(rom, pcm1, pcm2);
        // M68000::new() resets, which reads the vectors through the bus.
        let cpu = {
            let mut c: M68000<Mc68000> = M68000::new();
            // Force the reset to happen against our map straight away, so the
            // vectors are reported now rather than on the first instruction.
            c.interpreter(&mut board);
            c
        };
        log::debug!(
            target: "sound",
            "68000 reset: ssp={:08X} pc={:08X}",
            cpu.regs.ssp.0,
            cpu.regs.pc.0
        );
        Self {
            cpu,
            board,
            remainder: 0,
            irq_pending: false,
            main_fraction: 0,
            muted: [false; 3],
            samples: std::collections::VecDeque::new(),
            exception_counts: [0; 256],
        }
    }

    /// Output sample rate: the MultiPCMs' native clock (10MHz / 224).
    pub fn sample_rate(&self) -> f32 {
        self.board.pcm[0].sample_rate()
    }

    pub fn set_mutes(&mut self, mutes: AudioMutes) {
        let muted = [mutes.multipcm1, mutes.multipcm2, mutes.ym3438];
        if self.muted != muted {
            self.muted = muted;
            self.samples.clear();
        }
    }

    /// Hands the driver a byte from the i960 and raises the UART's interrupt,.
    pub fn send(&mut self, data: u8) {
        self.board.uart_send(data);
        self.irq_pending = true;
    }

    /// Only the FM chip/converter, not a complete board or machine snapshot.
    /// A future full restore must also restore PCM, 68000, UART and run budgets.
    pub fn snapshot_fm_path(&self) -> FmPathState {
        self.board.ym.snapshot()
    }

    pub fn restore_fm_path(&mut self, state: &FmPathState) -> Result<(), &'static str> {
        self.board.ym.restore(state)?;
        self.samples.clear(); // host output is not emulated state
        Ok(())
    }

    /// Advance all sound devices on the 68000 instruction grid. The CPU core
    /// exposes instruction totals, not timed bus micro-operations: reads/writes
    /// occur at that instruction's start, then we render its elapsed interval.
    /// This preserves write order without pretending to be bus-cycle accurate.
    fn render_cycles(&mut self, cycles: usize) {
        let SoundBoard { ym, pcm, .. } = &mut self.board;
        let muted = self.muted;
        let samples = &mut self.samples;
        ym.advance(cycles, |fm| {
            // All chips run even if muted or if the bounded output queue is full.
            let pcm1 = pcm[0].generate();
            let pcm2 = pcm[1].generate();
            let mixed = mix(pcm1, pcm2, fm, muted);
            if samples.len() < MAX_BUFFERED_SAMPLES {
                samples.push_back(mixed);
            }
        });
    }

    /// Runs the board for `i960_cycles` of main-board time.
    pub fn run(&mut self, i960_cycles: i32, i960_hz: u32) {
        // Convert the main board's budget into this board's clock.
        let scaled = i960_cycles as i64 * SND_CPU_HZ as i64 + self.main_fraction as i64;
        self.remainder += scaled.div_euclid(i960_hz as i64);
        self.main_fraction = scaled.rem_euclid(i960_hz as i64) as u64;
        // The 8251's rxrdy line drives the driver's level-2 interrupt. Assert
        // it when a byte arrives; then re-assert it (edge per byte) each time
        // the driver actually consumes one while more remain, so a burst -- VF
        // sends three command bytes back to back -- drains one interrupt at a
        // time. Asserting unconditionally while the FIFO is non-empty instead
        // leaves a stale level-2 pending that fires again after the byte is
        // read, and the driver then services a phantom 0 and stops making sound.
        if self.irq_pending {
            self.irq_pending = false;
            self.cpu
                .exception(Exception::from(Vector::Level2Interrupt as u8));
        }
        while self.remainder > 0 {
            let reads_before = self.board.rx_read_count;
            let (used, exception) = self.cpu.interpreter_exception(&mut self.board);
            if self.board.rx_read_count > reads_before && !self.board.rx.is_empty() {
                self.cpu
                    .exception(Exception::from(Vector::Level2Interrupt as u8));
            }
            if let Some(vector) = exception {
                self.exception_counts[vector as usize] += 1;
                self.cpu.exception(Exception::from(vector));
            }
            // A stopped CPU reports no cycles; do not spin on it.
            if used == 0 {
                let rest = self.remainder as usize;
                self.remainder = 0;
                self.render_cycles(rest);
            } else {
                self.remainder -= used as i64;
                self.render_cycles(used);
            }
        }
    }
}

/// Either of the two sound boards a Model 2 game can ship with: the
/// segam1audio MultiPCM board (original Model 2, e.g. Daytona) or the
/// SCSP board (Model 2A/2B/2C, e.g. Sega Rally). The main board talks to both
/// through the same i8251 UART, so this exposes the union of what the memory
/// map and front end use.
pub enum Sound {
    // Boxed because the two boards differ by several hundred kilobytes of
    // sample RAM and voice state, and every `Sound` would otherwise be as big
    // as the larger of them.
    MultiPcm(Box<SoundSystem>),
    Scsp(Box<crate::sound2a::SoundSystem2A>),
}

impl Sound {
    pub fn sources(&self) -> &'static [AudioSource] {
        match self {
            Self::MultiPcm(_) => MULTIPCM_SOURCES,
            Self::Scsp(_) => SCSP_SOURCES,
        }
    }

    pub fn set_mutes(&mut self, mutes: AudioMutes) {
        match self {
            Self::MultiPcm(s) => s.set_mutes(mutes),
            Self::Scsp(s) => s.set_muted(mutes.scsp),
        }
    }

    /// The i960 has put a byte on the wire.
    pub fn send(&mut self, data: u8) {
        match self {
            Sound::MultiPcm(s) => s.send(data),
            Sound::Scsp(s) => s.send(data),
        }
    }

    /// i8251 mode/command register write from the i960 side.
    pub fn control(&mut self, val: u8) {
        match self {
            Sound::MultiPcm(s) => s.board.uart_control(val),
            Sound::Scsp(s) => s.board.uart_control(val),
        }
    }

    /// Takes the byte the driver sent back, if any (a read of the UART's data
    /// register consumes it).
    pub fn take_reply(&mut self) -> u8 {
        match self {
            Sound::MultiPcm(s) => s.board.tx.take().unwrap_or(0),
            Sound::Scsp(s) => s.board.tx.take().unwrap_or(0),
        }
    }

    /// True while the driver has a byte waiting to be read back.
    pub fn reply_ready(&self) -> bool {
        match self {
            Sound::MultiPcm(s) => s.board.uart_rx_ready(),
            Sound::Scsp(s) => s.board.uart_rx_ready(),
        }
    }

    /// True while the UART can accept another byte for the board.
    pub fn tx_ready(&self) -> bool {
        match self {
            Sound::MultiPcm(s) => s.board.uart_tx_ready(),
            Sound::Scsp(s) => s.board.uart_tx_ready(),
        }
    }

    /// Runs the board for `i960_cycles` of main-board time.
    pub fn run(&mut self, i960_cycles: i32, i960_hz: u32) {
        match self {
            Sound::MultiPcm(s) => s.run(i960_cycles, i960_hz),
            Sound::Scsp(s) => s.run(i960_cycles, i960_hz),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        match self {
            Sound::MultiPcm(s) => s.sample_rate(),
            Sound::Scsp(s) => s.sample_rate(),
        }
    }

    /// Drains the rendered stereo output.
    pub fn drain_samples(&mut self) -> std::collections::vec_deque::Drain<'_, (i16, i16)> {
        match self {
            Sound::MultiPcm(s) => s.samples.drain(..),
            Sound::Scsp(s) => s.samples.drain(..),
        }
    }

    /// The segam1audio board, for tools that report MultiPCM-specific stats.
    pub fn as_multi_pcm(&self) -> Option<&SoundSystem> {
        match self {
            Sound::MultiPcm(s) => Some(s),
            Sound::Scsp(_) => None,
        }
    }

    /// The SCSP board, for tools that report 2A-specific stats.
    pub fn as_scsp(&self) -> Option<&crate::sound2a::SoundSystem2A> {
        match self {
            Sound::MultiPcm(_) => None,
            Sound::Scsp(s) => Some(s),
        }
    }
}
