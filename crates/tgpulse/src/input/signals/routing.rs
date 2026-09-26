//! Translation boundary: public signals -> existing cabinet requests / I/O.
//! No physical button names belong in the game tables below.
use super::{expression::Atom, Signal as S};
use crate::bindings::{Control as C, Source};
use crate::input::{AnalogRole as A, Axes, ControlScheme as Scheme, InputState};
use tgpulse_core::config::Inputs;

impl InputState {
    pub fn set_game(&mut self, game: &str) {
        self.game = game.to_owned();
    }

    pub(in crate::input) fn signal(&self, signal: S) -> f32 {
        self.sample_signal(signal, false, false)
    }
    fn sample_signal(&self, signal: S, keyboard_only: bool, pad_only: bool) -> f32 {
        self.bindings.binding(signal).value(|atom| match atom {
            Atom::Source(Source::Key(k)) if !pad_only => f32::from(u8::from(self.held(*k))),
            Atom::Source(Source::Key(_)) => 0.0,
            Atom::Source(s) if !keyboard_only => self.source_amount(*s),
            Atom::Axis(axis, sign) if !keyboard_only => {
                let v = self.axis_value(*axis) * sign;
                if v.abs() > 0.15 {
                    v
                } else {
                    0.0
                }
            }
            Atom::Keys(negative, positive) if !pad_only => {
                f32::from(u8::from(self.held(*positive)))
                    - f32::from(u8::from(self.held(*negative)))
            }
            _ => 0.0,
        })
    }

    fn steering_signal(&self) -> S {
        match self.scheme {
            Scheme::Jetski => S::Handle,
            Scheme::Skate => S::Curving,
            Scheme::Ski if self.game.starts_with("skisuprg") => S::Swing,
            Scheme::Ski => S::WaterSlide,
            _ => S::Steering,
        }
    }

    pub(in crate::input) fn routed_amount(&self, control: C) -> f32 {
        let driving = matches!(self.scheme, Scheme::Racing | Scheme::Bike);
        let flight = self.scheme == Scheme::Flight;
        // The racing frontend has separate ramped-key and direct-axis paths.
        let axis_signal = self.steering_signal();
        match control {
            C::SteerLeft | C::SteerRight => {
                let sign = if control == C::SteerLeft { -1.0 } else { 1.0 };
                return (sign * self.sample_signal(axis_signal, false, true)).max(0.0);
            }
            C::Left | C::Right if driving => {
                let sign = if control == C::Left { -1.0 } else { 1.0 };
                return (sign * self.sample_signal(axis_signal, true, false)).max(0.0);
            }
            C::Up | C::Down if driving => {
                return self.sample_signal(
                    if control == C::Up {
                        S::Accelerator
                    } else {
                        S::Brake
                    },
                    true,
                    false,
                )
            }
            C::Throttle | C::Brake if driving => {
                return self.sample_signal(
                    if control == C::Throttle {
                        S::Accelerator
                    } else {
                        S::Brake
                    },
                    false,
                    true,
                )
            }
            C::Left | C::Right | C::Up | C::Down if flight => {
                let horizontal = matches!(control, C::Left | C::Right);
                let signal = if horizontal { S::SkyX } else { S::SkyY };
                let sign = if matches!(control, C::Left | C::Down) {
                    -1.0
                } else {
                    1.0
                };
                return (self.signal(signal) * sign).max(0.0);
            }
            C::LeanLeft | C::LeanRight => {
                return (self.signal(axis_signal) * if control == C::LeanLeft { -1.0 } else { 1.0 })
                    .max(0.0)
            }
            _ => {}
        }
        let signal = match control {
            C::Up => S::Up,
            C::Down => S::Down,
            C::Left => S::Left,
            C::Right => S::Right,
            C::Coin1 => S::Coin,
            C::Coin2 => S::Coin2,
            C::Start1 => S::Start,
            C::Test => S::Test,
            C::Service => S::Service,
            C::Button1 if flight => S::Action1,
            C::Button2 if flight => S::Action2,
            C::Button1 if self.game == "vf" || self.game.starts_with("doa") => S::Action3,
            C::Button2 if self.game == "vf" || self.game.starts_with("doa") => S::Action1,
            C::Button3 if self.game == "vf" || self.game.starts_with("doa") => S::Action2,
            C::Button2 if self.game.starts_with("vstriker") => S::Action3,
            C::Button3 if self.game.starts_with("vstriker") => S::Action2,
            C::Button1 if self.game.starts_with("dynabb") => S::Action2,
            C::Button2 if self.game.starts_with("dynabb") => S::Action1,
            C::Button1 => S::Action1,
            C::Button2 => S::Action2,
            C::Button3 => S::Action3,
            C::Button4 => S::Action4,
            C::ViewRed => S::View1,
            C::ViewBlue => S::View2,
            C::ViewYellow => S::View3,
            C::ViewGreen => S::View4,
            C::Throttle => S::Accelerator,
            C::Brake => S::Brake,
            C::GearUp => S::Action1,
            C::GearDown => S::Action2,
            C::Fire => S::Action1,
            C::Reload => S::Action2,
            C::ViewChange => S::View4,
            C::SteerLeft | C::SteerRight | C::LeanLeft | C::LeanRight => unreachable!(),
        };
        self.signal(signal)
    }

    /// Override independent axes absent from the old scheme, retaining its
    /// calibration and the DB's channel ordering for everything else.
    pub(in crate::input) fn routed_axis(&self, role: A, axes: &Axes) -> u8 {
        let axis = |s| (128.0 + self.signal(s) * 127.0).round().clamp(0.0, 255.0) as u8;
        match role {
            A::Roll => axis(S::Roll),
            A::Pitch => axis(S::Pitch),
            A::Slide => axis(if self.scheme == Scheme::Skate {
                S::SkaterSlide
            } else {
                S::WaterSlide
            }),
            A::Curving => axis(S::Curving),
            A::Swing => axis(S::Swing),
            A::Incline => axis(S::Inclining),
            A::Bat1 => (255.0 * self.signal(S::BatSwing)).round() as u8,
            _ => axes.by_role(role),
        }
    }

    fn h_gate(&self) -> bool {
        self.game.is_empty() || self.game.starts_with("daytona") || self.game.starts_with("srally")
    }

    pub(in crate::input) fn apply_direct_gear(&mut self) {
        if !self.h_gate() {
            return;
        }
        let mut active = [S::Neutral, S::Gear1, S::Gear2, S::Gear3, S::Gear4]
            .into_iter()
            .enumerate()
            .filter(|(_, s)| self.signal(*s) > 0.5)
            .map(|(i, _)| i);
        // A released stick or conflicting choices leave the current gear alone.
        let gear = active.next();
        if active.next().is_none() {
            if let Some(gear) = gear {
                self.gear = gear;
            }
        }
    }

    /// Digital exceptions cannot be inferred from the DB's analog role list.
    pub(in crate::input) fn route_ports(&mut self, out: &mut Inputs) {
        if self.game == "desert" {
            let shift = self.signal(S::Action3) > 0.5;
            if shift && !self.special_shift_held {
                self.special_shift = !self.special_shift;
            }
            self.special_shift_held = shift;
        }
        let pressed = |s| {
            let touch = match s {
                S::Start => self.touch_amount(C::Start1),
                S::Action1 => self.touch_amount(C::Fire),
                S::Action2 => self.touch_amount(C::Reload),
                _ => 0.0,
            };
            self.signal(s).max(touch) > 0.5
        };
        let write = |port: &mut u8, mask: u8, on: bool| {
            *port |= mask;
            if on {
                *port &= !mask;
            }
        };
        // These cabinets wire Start to IN0:40, often shared with a menu/VR
        // function. Do not create a second configurable Start signal.
        let start40 = self.scheme == Scheme::Bike
            || self.game.starts_with("srally")
            || self.game.starts_with("stcc")
            || self.game.starts_with("overrev")
            || self.game == "sgt24h";
        if start40 {
            out.in0 |= 0xf0;
            write(&mut out.in0, 0x40, pressed(S::Start));
        }
        if self.game.starts_with("von") {
            out.in1 = 0xff;
            out.in2 = 0xff;
            for (port, x, y, shot, dash) in [
                (
                    &mut out.in1,
                    S::TwinLeftX,
                    S::TwinLeftY,
                    S::Action1,
                    S::Action3,
                ),
                (
                    &mut out.in2,
                    S::TwinRightX,
                    S::TwinRightY,
                    S::Action2,
                    S::Action4,
                ),
            ] {
                for (bit, on) in [
                    (0x80, self.signal(x) < -0.5),
                    (0x40, self.signal(x) > 0.5),
                    (0x20, self.signal(y) > 0.5),
                    (0x10, self.signal(y) < -0.5),
                    (1, pressed(shot)),
                    (2, pressed(dash)),
                ] {
                    write(port, bit, on);
                }
            }
        } else if self.game == "skytargt" {
            out.in1 = 0xff;
            write(&mut out.in1, 0x10, pressed(S::Action1));
            write(&mut out.in1, 0x20, pressed(S::Action2));
            write(&mut out.in0, 0x20, pressed(S::View4));
        } else if self.game == "desert" {
            out.in1 = 0xff;
            write(&mut out.in0, 0x80, pressed(S::View4));
            write(&mut out.in1, 0x10, pressed(S::Action1));
            write(&mut out.in1, 0x20, pressed(S::Action2));
            write(&mut out.in1, 1, self.special_shift);
            out.brake = (128.0 + self.signal(S::Elevation) * 127.0).round() as u8;
            out.analog[2] = out.brake;
        } else if matches!(self.game.as_str(), "vr" | "vformula") {
            // Model 1 VR uses momentary, active-low shifts, NOT Daytona's
            // active-high H-gate code. Leave VR4 (bit 0) untouched.
            out.in1 |= 0x70;
            let up = self.on(C::GearUp);
            let down = self.on(C::GearDown);
            write(&mut out.in1, 0x20, up && !down);
            write(&mut out.in1, 0x10, down && !up);
        } else if self.scheme == Scheme::Bike || (self.scheme == Scheme::Racing && !self.h_gate()) {
            out.in1 |= 0x70;
            write(&mut out.in1, 1, false);
            write(&mut out.in1, 0x10, pressed(S::Action1));
            write(&mut out.in1, 0x20, pressed(S::Action2));
            if self.game.starts_with("overrev") {
                write(&mut out.in1, 1, pressed(S::View4));
                write(&mut out.in1, 2, pressed(S::View1));
            } else if self.game == "sgt24h" {
                write(&mut out.in1, 1, pressed(S::View1));
            }
        }
        if self.game.starts_with("srally") {
            // Rally handbrake is active high, unlike cabinet buttons.
            out.in2 = if pressed(S::Action4) { 0xff } else { 0x00 };
            write(&mut out.in0, 0x20, pressed(S::View1));
            write(&mut out.in1, 1, false);
        }
        if self.scheme == Scheme::Joystick || self.scheme == Scheme::Gun {
            write(&mut out.in0, 0x20, pressed(S::Start2));
        }
        if self.game == "segawski" {
            out.in0 |= 0x10;
            out.in1 = 0xff;
            write(&mut out.in0, 0x40, pressed(S::Start));
            write(&mut out.in1, 2, pressed(S::View4));
            write(&mut out.in1, 1, pressed(S::Action3));
            write(&mut out.in1, 4, pressed(S::Action1));
            write(&mut out.in1, 8, pressed(S::Action2));
        } else if self.game == "skisuprg" {
            out.in1 = 0xff;
            write(&mut out.in0, 0x20, pressed(S::View4));
            write(&mut out.in1, 1, pressed(S::View1));
            write(&mut out.in0, 0x80, pressed(S::Action4));
            write(&mut out.in0, 0x10, pressed(S::Start));
            write(&mut out.in0, 0x40, pressed(S::Action3));
            out.in2 = if pressed(S::Action1) { 0xf0 } else { 0 }
                | if pressed(S::Action2) { 0x0f } else { 0 };
        } else if self.game.starts_with("topskatr") {
            out.in1 = 0xff;
            write(&mut out.in0, 0x40, pressed(S::Start));
            write(&mut out.in0, 0x80, pressed(S::View2));
            write(&mut out.in0, 0x10, pressed(S::View3));
            write(&mut out.in0, 0x20, pressed(S::Action1));
            write(&mut out.in1, 1, pressed(S::Action2));
        }
        if self.game == "bel" {
            out.gun_offscreen = false;
            write(&mut out.in1, 1, pressed(S::Action1) || self.mouse_fire);
            write(&mut out.in1, 0x10, pressed(S::Action2) || self.mouse_reload);
        }
    }
}
