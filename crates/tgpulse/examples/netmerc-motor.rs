//! Headless real-ROM motor audit. Reads the supplied ZIP, creates an in-memory
//! cabinet, and never opens a controller/audio device or saves NVRAM.
//! Run with RUST_LOG=model1_motor=trace to compare write edges with frame samples.
use tgpulse_core::{config::Config, loader, model1::Model1System};

struct Probe {
    machine: Model1System,
    frame: u32,
}

impl Probe {
    fn phase(&mut self, name: &str, frames: u32, in0: u8, in1: u8) -> Result<(), String> {
        println!("PHASE {name}");
        self.machine.inputs.in0 = in0;
        self.machine.inputs.in1 = in1;
        for _ in 0..frames {
            println!("BEGIN frame={}", self.frame);
            self.machine.run_frame().map_err(|e| e.to_string())?;
            self.machine.sound.samples.clear();
            let (_, clock) = self.machine.ioboard.advanced_cpu_state().unwrap();
            let on = self.machine.ioboard.netmerc_motor().unwrap();
            println!("END frame={} clock={clock} on={}", self.frame, u8::from(on));
            self.frame += 1;
        }
        Ok(())
    }
}

fn main() -> Result<(), String> {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("Usage: netmerc-motor <netmerc.zip>")?;
    if args.next().is_some() {
        return Err("Usage: netmerc-motor <netmerc.zip>".into());
    }
    let roms = loader::load_model1_zip(&path).map_err(|e| e.to_string())?;
    if roms.ioboard_kind != tgpulse_core::model1board::Kind::NetMerc {
        return Err("This audit requires a NetMerc ROM set".into());
    }
    let machine = Model1System::with_config(&roms, Config::default()).map_err(|e| e.to_string())?;
    let mut probe = Probe { machine, frame: 0 };
    probe.machine.inputs.analog[0] = 127;
    probe.machine.inputs.analog[2] = 127;
    probe.phase("boot", 900, 255, 255)?;
    probe.phase("coin", 4, 254, 255)?;
    probe.phase("credit", 300, 255, 255)?;
    probe.phase("holder", 60, 255, 251)?;
    probe.phase("holder-release", 300, 255, 255)?;
    probe.phase("start-trigger", 4, 255, 254)?;
    probe.phase("game-start", 300, 255, 255)?;
    probe.phase("trigger-held", 300, 255, 254)?;
    probe.phase("trigger-release", 120, 255, 255)?;
    probe.phase("thumb-held", 300, 255, 253)?;
    probe.phase("thumb-release", 120, 255, 255)?;
    probe.phase("both-held", 300, 255, 252)?;
    probe.phase("both-release", 120, 255, 255)?;
    for _ in 0..60 {
        probe.phase("thumb-one-frame", 1, 255, 253)?;
        probe.phase("thumb-one-frame-release", 1, 255, 255)?;
    }
    probe.phase("settle", 120, 255, 255)?;
    Ok(())
}
