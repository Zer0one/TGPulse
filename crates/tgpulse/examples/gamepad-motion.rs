//! Bounded sensor diagnostic: no ROM, settings writes, tracking or integration.
#[cfg(target_os = "macos")]
#[path = "../src/input/motion.rs"]
mod motion;
#[cfg(target_os = "macos")]
#[allow(dead_code)]
#[path = "../src/input/sdl_pads.rs"]
mod sdl_pads;

#[cfg(target_os = "macos")]
fn main() -> Result<(), String> {
    use motion::{GyroCalibration, SensorKind};
    use std::time::{Duration, Instant};
    let args: Vec<_> = std::env::args().skip(1).collect();
    let seconds = args
        .first()
        .map_or(Ok(10), |s| s.parse::<u64>())
        .map_err(|_| "Usage: gamepad-motion [5..60 seconds] [name fragment]".to_string())?;
    if !(5..=60).contains(&seconds) {
        return Err("Duration must be 5..60 seconds".into());
    }
    let name = args
        .get(1)
        .map_or("dualsense", String::as_str)
        .to_lowercase();
    let mut pads = sdl_pads::SdlPads::new()?;
    println!(
        "SDL3 devices: {:?}",
        pads.devices()
            .iter()
            .map(|(_, _, name)| name)
            .collect::<Vec<_>>()
    );
    let devices: Vec<_> = pads
        .devices()
        .into_iter()
        .filter(|(_, _, label)| label.to_lowercase().contains(&name))
        .collect();
    if devices.len() != 1 {
        return Err(format!(
            "Expected one matching controller; found {devices:?}"
        ));
    }
    let (id, _, label) = &devices[0];
    let id = *id;
    let rates = pads.enable_motion(id)?;
    println!("Device: {label}; gyro/accel advertised Hz: {rates:?}");
    if rates.iter().all(Option::is_none) {
        return Err("SDL3 reports no motion sensors".into());
    }
    println!("Keep controller STILL until MOVE: first 2 seconds settle, then 3 seconds calibrate. No MVD tracking is enabled.");
    println!("Calibration limits: at least {} distinct samples; |mean| <= {} rad/s, standard deviation <= {} rad/s per axis (X, Y, Z).",
        motion::MIN_CALIBRATION_SAMPLES, motion::MAX_GYRO_MEAN, motion::MAX_GYRO_DEVIATION);
    let start = Instant::now();
    let mut calibration = GyroCalibration::default();
    let mut counts = [0u64; 2];
    let mut latest = [None; 2];
    let mut min = [[f32::INFINITY; 3]; 2];
    let mut max = [[f32::NEG_INFINITY; 3]; 2];
    let mut reported = 0;
    let mut calibration_reported = false;
    while start.elapsed() < Duration::from_secs(seconds) {
        pads.poll_motion(|device, sample| {
            if device != id {
                return;
            }
            let index = usize::from(sample.kind == SensorKind::Accel);
            counts[index] += 1;
            latest[index] = Some(sample.data);
            for axis in 0..3 {
                min[index][axis] = min[index][axis].min(sample.data[axis]);
                max[index][axis] = max[index][axis].max(sample.data[axis]);
            }
            let elapsed = start.elapsed();
            if elapsed >= Duration::from_secs(2) && elapsed < Duration::from_secs(5) {
                calibration.observe(sample);
            }
        });
        if !pads.has(id) {
            return Err("Controller disconnected during capture".into());
        }
        let second = start.elapsed().as_secs();
        if second >= 5 && !calibration_reported {
            report_calibration(&calibration);
            println!("MOVE: rotate pitch (X), yaw (Y), roll (Z) separately, returning to neutral between rotations. Record which direction you move first for each axis.");
            calibration_reported = true;
        }
        if second > reported {
            reported = second;
            println!(
                "t={second}s phase={} gyro(rad/s)={:?} accel(m/s2)={:?} events={counts:?}",
                if second < 2 {
                    "settle"
                } else if second < 5 {
                    "calibrate"
                } else {
                    "move"
                },
                latest[0],
                latest[1]
            );
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if !calibration_reported {
        report_calibration(&calibration);
    }
    println!("Observed gyro/accel min={min:?}, max={max:?}, events={counts:?}");
    if counts
        .iter()
        .zip(rates)
        .any(|(n, rate)| rate.is_some() && *n == 0)
    {
        return Err("Advertised sensor supplied no events".into());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn report_calibration(calibration: &motion::GyroCalibration) {
    println!("Stationary gyro samples={} mean(rad/s)={:?} standard_deviation(rad/s)={:?} bias(rad/s)={:?}",
        calibration.samples(), calibration.mean(), calibration.standard_deviation(), calibration.bias());
    match calibration.result() {
        Ok(_) => println!("Calibration ACCEPTED"),
        Err(reason) => println!("Calibration REJECTED: {reason}"),
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("SDL3 desktop gamepad backend currently targets macOS only.");
    std::process::exit(1);
}
