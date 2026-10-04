//! Desktop-only MVD input policy. The board receives the same raw pose API
//! regardless of host device; no SDL, GUI or clock enters hardware emulation.
use super::{InputState, Player, Signal};
use tgpulse_core::model1io2::HmdPose;
pub mod holder;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Auto,
    Off,
    RightStick,
    Sensors,
}

impl Mode {
    pub const ALL: [Self; 4] = [Self::Auto, Self::Off, Self::RightStick, Self::Sensors];
    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Off => "off",
            Self::RightStick => "right_stick",
            Self::Sensors => "sensors",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Off => "Off (fixed camera)",
            Self::RightStick => "Right Stick",
            Self::Sensors => "Sensors",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.key() == value)
    }
}

impl InputState {
    /// Frontend commands, not emulated cabinet pins. One event per press.
    pub fn mvd_commands_requested(&mut self) -> [bool; 2] {
        let active = [Signal::NetmercMvdCalibrate, Signal::NetmercMvdRecenter]
            .map(|signal| self.game == "netmerc" && self.signal(signal) > 0.5);
        let pressed = std::array::from_fn(|i| active[i] && !self.mvd_commands_held[i]);
        self.mvd_commands_held = active;
        pressed
    }

    pub fn mvd_pose(&self, mode: Mode, range: [u32; 2]) -> HmdPose {
        if matches!(mode, Mode::Auto | Mode::Sensors) {
            if let Some(angles) = self.motion_tracker.angles() {
                return sensor_pose(angles);
            }
        }
        pose(
            mode,
            self.has_pad(Player::One),
            self.signal(Signal::NetmercMvdX),
            self.signal(Signal::NetmercMvdY),
            range,
        )
    }
}

impl InputState {
    pub fn set_mvd_mode(&mut self, mode: Mode) {
        if self.motion_mode != mode {
            self.reset_motion_device();
            self.motion_mode = mode;
        }
        self.refresh_motion_device();
    }
    pub fn recenter_mvd(&mut self) -> bool {
        if self.motion_tracker.angles().is_none() {
            return false;
        }
        self.motion_tracker.recenter();
        true
    }
    pub fn set_mvd_gravity_stabilization(&mut self, enabled: bool) {
        self.motion_tracker.set_gravity_stabilization(enabled);
    }
    pub fn recalibrate_mvd(&mut self) {
        self.motion_tracker.reset();
        self.motion_last = None;
        self.motion_notice = None;
    }
    /// Consume each calibration outcome/unavailable-device notification once.
    pub fn take_mvd_notice(&mut self) -> Option<(String, bool)> {
        self.motion_notice.take()
    }
    pub(super) fn reset_motion_device(&mut self) {
        #[cfg(target_os = "macos")]
        if let (Some(sdl), Some(id)) = (&self.sdl, self.motion_pad) {
            sdl.disable_motion(id);
        }
        self.motion_pad = None;
        self.recalibrate_mvd();
    }
    pub(super) fn refresh_motion_device(&mut self) {
        #[cfg(target_os = "macos")]
        let wanted = (self.game == "netmerc"
            && matches!(self.motion_mode, Mode::Auto | Mode::Sensors)
            && self.sdl.is_some())
        .then(|| self.assignments.id(Player::One))
        .flatten();
        #[cfg(not(target_os = "macos"))]
        let wanted = None;
        if wanted != self.motion_pad {
            self.reset_motion_device();
            self.motion_pad = wanted;
            #[cfg(target_os = "macos")]
            if let (Some(sdl), Some(id)) = (&self.sdl, wanted) {
                match sdl.enable_motion(id) {
                    Ok(rates) if rates[0].is_some() => {
                        log::info!(target: "input", "MVD sensors enabled for P1 device {id}; advertised gyro/accel Hz {rates:?}");
                    }
                    Ok(_) => {
                        sdl.disable_motion(id);
                        self.motion_notice = Some((
                            "MVD calibration unavailable: P1 controller has no gyro".into(),
                            true,
                        ));
                    }
                    Err(error) => {
                        self.motion_notice =
                            Some((format!("MVD calibration unavailable: {error}"), true))
                    }
                }
            }
        }
        if self
            .motion_last
            .is_some_and(|last| last.elapsed() > std::time::Duration::from_millis(500))
        {
            self.recalibrate_mvd();
        }
    }
}

fn sensor_pose([pitch, yaw, roll]: [f64; 3]) -> HmdPose {
    let mut pose = HmdPose::default();
    // SDL positive yaw turns left, pitch looks up. ROM-probed MVD offsets
    // use +XANG right and +ZANG down. Physical axes accepted by the user.
    let raw =
        |radians: f64| (radians.to_degrees().clamp(-90.0, 90.0) * 25736.0 / 180.0).round() as i32;
    let wrap = |angle: i32| {
        let angle = if angle > 25736 {
            angle - 51472
        } else if angle < -25736 {
            angle + 51472
        } else {
            angle
        };
        angle as i16
    };
    pose.orientation[0] = wrap(i32::from(pose.orientation[0]) - raw(yaw));
    pose.orientation[1] = wrap(i32::from(pose.orientation[1]) + raw(roll));
    pose.orientation[2] = wrap(i32::from(pose.orientation[2]) - raw(pitch));
    pose
}

fn pose(mode: Mode, pad_available: bool, x: f32, y: f32, range: [u32; 2]) -> HmdPose {
    let mut pose = HmdPose::default();
    if mode == Mode::Off || (matches!(mode, Mode::Auto | Mode::Sensors) && !pad_available) {
        return pose;
    }
    // 25736 raw units = 180 degrees. Absolute deflection, not an integrated
    // velocity: releasing the stick restores neutral without drift.
    // Real-ROM scene probes: +XANG looks right, +ZANG looks down; YANG is
    // image roll. Native stick Y is positive up. Physical mounting remains
    // separate from this desktop camera policy.
    let raw = |value: f32, degrees: f32| {
        let value = value.clamp(-1.0, 1.0);
        let magnitude =
            ((value.abs() - super::STICK_DEADZONE) / (1.0 - super::STICK_DEADZONE)).max(0.0);
        (value.signum() * magnitude * degrees * 25736.0 / 180.0).round() as i16
    };
    pose.orientation[0] += raw(x, range[0].min(90) as f32);
    pose.orientation[2] -= raw(y, range[1].min(90) as f32);
    pose
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn virtual_commands_are_assignable_edges_not_cabinet_inputs() {
        use gilrs::Button;
        let mut input = InputState::with_gilrs(None);
        input.set_game("netmerc");
        input.set_scheme(super::super::ControlScheme::Flight);
        for (button, expected) in [
            (Button::North, [true, false]),
            (Button::West, [false, true]),
        ] {
            input.set_pad_button(button, true);
            assert_eq!(input.mvd_commands_requested(), expected);
            assert_eq!(input.mvd_commands_requested(), [false; 2]);
            let mut cabinet = tgpulse_core::config::Inputs::default();
            input.poll(&mut cabinet);
            assert_eq!(cabinet.in1 & 7, 7); // Neither command presses Trigger/Thumb/Holder.
            input.set_pad_button(button, false);
            assert_eq!(input.mvd_commands_requested(), [false; 2]);
        }
        for button in [Button::Start, Button::DPadDown] {
            input.set_pad_button(button, true);
            let mut cabinet = tgpulse_core::config::Inputs::default();
            input.poll(&mut cabinet);
            assert_eq!(cabinet.in1 & 4, 0);
            input.set_pad_button(button, false);
        }
        input
            .bindings
            .set_expression(Signal::NetmercMvdCalibrate, "KeyI")
            .unwrap();
        input.on_key(winit::keyboard::KeyCode::KeyI, true);
        assert_eq!(input.mvd_commands_requested(), [true, false]);
        input.set_game("vr");
        assert_eq!(input.mvd_commands_requested(), [false; 2]);
    }

    #[test]
    fn modes_and_absolute_pose_are_bounded_and_recenter() {
        let neutral = HmdPose::default();
        let range = [30, 20];
        assert_eq!(pose(Mode::Off, true, 1.0, 1.0, range), neutral);
        assert_eq!(pose(Mode::Auto, false, 1.0, 1.0, range), neutral);
        assert_eq!(pose(Mode::Sensors, false, 1.0, 1.0, range), neutral);
        assert_eq!(sensor_pose([0.0; 3]), neutral);
        let sensor = sensor_pose([0.2, -0.3, 0.1]);
        assert_eq!(sensor.position, neutral.position);
        assert!(sensor.orientation[0] > neutral.orientation[0]);
        assert!(sensor.orientation[2] < neutral.orientation[2]);
        assert!(sensor.orientation[1] < 0); // 180° + roll wraps, not i16 overflow.
        assert_eq!(pose(Mode::Auto, true, 0.1, -0.1, range), neutral);
        let extreme = pose(Mode::Auto, true, 1.0, -1.0, range);
        assert_eq!(extreme.position, neutral.position);
        assert_eq!(extreme.orientation, [17157, 25736, 15728]);
        assert_eq!(pose(Mode::RightStick, false, 1.0, -1.0, range), extreme);
        assert_eq!(pose(Mode::Auto, true, 0.0, 0.0, range), neutral);
        assert_eq!(pose(Mode::RightStick, true, 1.0, 1.0, [0, 0]), neutral);
        assert_eq!(
            pose(Mode::RightStick, true, 1.0, 1.0, [90, 90]).orientation,
            [25736, 25736, 0]
        );
    }

    #[test]
    fn assignable_mvd_axes_do_not_change_the_cabinet_stick() {
        use winit::keyboard::KeyCode;
        let mut input = InputState::with_gilrs(None);
        input.set_game("netmerc");
        input.set_scheme(super::super::ControlScheme::Flight);
        use super::super::AnalogRole as A;
        input.set_analog_roles([
            A::StickX,
            A::None,
            A::StickY,
            A::None,
            A::None,
            A::None,
            A::None,
            A::None,
        ]);
        input
            .bindings
            .set_expression(Signal::NetmercMvdX, "keys:F11/F12")
            .unwrap();
        input
            .bindings
            .set_expression(Signal::NetmercMvdY, "keys:F9/F10")
            .unwrap();
        input.on_key(KeyCode::F12, true);
        input.on_key(KeyCode::F10, true);
        let mut cabinet = tgpulse_core::config::Inputs::default();
        input.poll(&mut cabinet);
        assert_eq!((cabinet.analog[0], cabinet.analog[2]), (127, 127));
        assert_eq!(
            input.mvd_pose(Mode::RightStick, [30, 20]).orientation,
            [17157, 25736, 10008]
        );
        input.on_key(KeyCode::F12, false);
        input.on_key(KeyCode::F10, false);
        assert_eq!(
            input.mvd_pose(Mode::RightStick, [30, 20]),
            HmdPose::default()
        );
        // NetMerc's main stick passes small deflections; MVD keeps its dead zone.
        input
            .bindings
            .set_expression(Signal::NetmercMvdX, "pad:LeftStickX")
            .unwrap();
        input.set_pad_stick(0.01, 0.01);
        assert_eq!(
            input.mvd_pose(Mode::RightStick, [30, 20]),
            HmdPose::default()
        );
        input.poll(&mut cabinet);
        assert!(cabinet.analog[0] > 127);
        assert!(cabinet.analog[2] < 127);
        // Other games retain the shared joystick dead zone.
        input.set_game("wingwar");
        assert_eq!(input.signal(Signal::SkyX), 0.0);
        assert_eq!(input.signal(Signal::SkyY), 0.0);
        // A calibrated sensor pose takes precedence only in Auto/Sensors.
        input.set_game("netmerc");
        for n in 0..=500 {
            input
                .motion_tracker
                .observe(super::super::motion::MotionSample {
                    kind: super::super::motion::SensorKind::Gyro,
                    data: [0.0; 3],
                    timestamp_ns: n * 10_000_000,
                });
        }
        for n in 501..=600 {
            input
                .motion_tracker
                .observe(super::super::motion::MotionSample {
                    kind: super::super::motion::SensorKind::Gyro,
                    data: [0.0, -0.5, 0.0],
                    timestamp_ns: n * 10_000_000,
                });
        }
        let tracked = input.mvd_pose(Mode::Auto, [0, 0]);
        assert!(tracked.orientation[0] > HmdPose::default().orientation[0]);
        assert_eq!(tracked, input.mvd_pose(Mode::Sensors, [90, 90]));
        assert_eq!(input.mvd_pose(Mode::Off, [30, 20]), HmdPose::default());
        assert_eq!(input.mvd_pose(Mode::RightStick, [0, 0]), HmdPose::default());
        assert!(input.recenter_mvd());
        assert_eq!(input.mvd_pose(Mode::Auto, [30, 20]), HmdPose::default());
        input.recalibrate_mvd();
        assert!(!input.recenter_mvd());
    }
}
