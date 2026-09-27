//! Translation boundary: public signals -> existing cabinet requests / I/O.
//! No physical button names belong in the game tables below.
use super::{expression::Atom, Signal as S};
use crate::bindings::{Control as C, Source};
use crate::input::players::Player;
use crate::input::{AnalogRole as A, Axes, ControlScheme as Scheme, InputState};
use tgpulse_core::config::Inputs;

impl InputState {
    pub fn set_game(&mut self, game: &str) {
        if self.game != game {
            self.special_shift = false;
            self.special_shift_held = false;
            self.cursor_p2_active = false;
            self.game = game.to_owned();
            let rest_fraction = |role| {
                let (min, max, rest, reverse) = self.positional_gun_range(role);
                let fraction = (rest - min) as f32 / (max - min) as f32;
                if reverse {
                    1.0 - fraction
                } else {
                    fraction
                }
            };
            self.cursor_p2 = (rest_fraction(A::Gun2X), rest_fraction(A::Gun2Y));
        }
    }

    pub(in crate::input) fn signal(&self, signal: S) -> f32 {
        let value = self.sample_signal(signal, false, false);
        // Single-view cabinets may use either gamepad view position. Reuse the
        // assignable View4 pad binding (default D-pad Up), not a hidden button
        // check. Do not alias keyboard View4 or multi-view cabinets.
        if signal == S::View1
            && (self.game.starts_with("srally")
                || matches!(self.game.as_str(), "sgt24h" | "swa" | "swaj"))
        {
            value.max(self.sample_signal(S::View4, false, true))
        } else {
            value
        }
    }
    fn sample_signal(&self, signal: S, keyboard_only: bool, pad_only: bool) -> f32 {
        self.sample_player_signal(Player::One, signal, keyboard_only, pad_only)
    }
    pub(in crate::input) fn signal_p2(&self, signal: S) -> f32 {
        if !signal.supports_p2() {
            return 0.0;
        }
        self.sample_player_signal(Player::Two, signal, false, false)
    }
    fn sample_player_signal(
        &self,
        player: Player,
        signal: S,
        keyboard_only: bool,
        pad_only: bool,
    ) -> f32 {
        self.bindings
            .player_binding(player, signal)
            .value(|atom| match atom {
                Atom::Source(Source::Key(k)) if !pad_only => f32::from(u8::from(self.held(*k))),
                Atom::Source(Source::Key(_)) => 0.0,
                Atom::Source(s) if !keyboard_only => self.source_amount_for(player, *s),
                Atom::Axis(axis, sign) if !keyboard_only => {
                    let v = self.axis_value_for(player, *axis) * sign;
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

    fn positional_gun_range(&self, role: A) -> (u8, u8, u8, bool) {
        match (self.game.as_str(), role) {
            ("gunblade" | "bel", A::Gun1X) => (0x69, 0xff, 0xb1, false),
            ("gunblade" | "bel", A::Gun2X) => (0x00, 0x96, 0x50, false),
            ("gunblade" | "bel", _) => (0x11, 0xae, 0x5f, false),
            ("rchase2", A::Gun1X) => (0x3a, 0xca, 0x82, true),
            ("rchase2", A::Gun2X) => (0x34, 0xc7, 0x7d, true),
            ("rchase2", _) => (0x1c, 0xcb, 0x73, true),
            _ => (0x00, 0xff, 0x80, false),
        }
    }

    pub(in crate::input) fn routed_amount(&self, control: C) -> f32 {
        let driving = matches!(self.scheme, Scheme::Racing | Scheme::Bike);
        let flight = self.scheme == Scheme::Flight;
        // The racing frontend has separate ramped-key and direct-axis paths.
        let axis_signal = self.steering_signal();
        match control {
            C::Coin2 => return self.signal_p2(S::Coin),
            C::Test => return self.signal(S::Test).max(self.signal_p2(S::Test)),
            C::Service => return self.signal(S::Service).max(self.signal_p2(S::Service)),
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
        self.signal(self.control_signal(control))
    }

    /// Shared semantic translation; the player only selects the binding/device.
    fn control_signal(&self, control: C) -> S {
        let flight = self.scheme == Scheme::Flight;
        match control {
            C::Up => S::Up,
            C::Down => S::Down,
            C::Left => S::Left,
            C::Right => S::Right,
            C::Coin1 => S::Coin,
            C::Coin2 => unreachable!(),
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
            // SWA's third hardware button changes view, rather than firing.
            C::Button3 if matches!(self.game.as_str(), "swa" | "swaj") => S::View1,
            C::Button1
                if self.game.starts_with("dynabb")
                    || matches!(self.game.as_str(), "hpyagu98" | "rascot2" | "airwlkrs") =>
            {
                S::Action2
            }
            C::Button2
                if self.game.starts_with("dynabb")
                    || matches!(self.game.as_str(), "hpyagu98" | "rascot2" | "airwlkrs") =>
            {
                S::Action1
            }
            C::Button1 => S::Action1,
            C::Button2 => S::Action2,
            C::Button3 => S::Action3,
            C::Button4 => S::Action4,
            C::ViewRed => S::View1,
            C::ViewBlue => S::View2,
            C::ViewYellow => S::View3,
            C::ViewGreen => S::View4,
            C::Throttle
                if matches!(self.game.as_str(), "swa" | "swaj")
                    || self.game.starts_with("wingwar") =>
            {
                S::ThrottleUp
            }
            C::Brake
                if matches!(self.game.as_str(), "swa" | "swaj")
                    || self.game.starts_with("wingwar") =>
            {
                S::ThrottleDown
            }
            C::Throttle => S::Accelerator,
            C::Brake => S::Brake,
            C::GearUp => S::Action2,
            C::GearDown => S::Action1,
            C::Fire => S::Action1,
            C::Reload => S::Action2,
            C::ViewChange => S::View4,
            C::SteerLeft | C::SteerRight | C::LeanLeft | C::LeanRight => unreachable!(),
        }
    }

    /// Override independent axes absent from the old scheme, retaining its
    /// calibration and the DB's channel ordering for everything else.
    pub(in crate::input) fn routed_axis(&self, role: A, axes: &Axes) -> u8 {
        let axis = |s| centered(self.signal(s), 128, 0, 255);
        // The legacy racing sampler ramps within Daytona/VR's 20..e0 range.
        // Translate that travel, not the bindings, for full-range cabinets.
        let full_range_car = matches!(self.scheme, Scheme::Racing | Scheme::Bike)
            && !self.game.is_empty()
            && !matches!(self.game.as_str(), "vr" | "vformula");
        if full_range_car {
            let pedal = |v: u8| {
                ((v.saturating_sub(0x20) as f32 / 192.0) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            match role {
                A::Steer => {
                    return centered(
                        (axes.steer as f32 - 128.0) / 96.0
                            * if self.scheme == Scheme::Bike {
                                -1.0
                            } else {
                                1.0
                            },
                        128,
                        0,
                        255,
                    )
                }
                A::Brake if self.game == "desert" => {
                    return centered(-self.signal(S::Elevation), 128, 0, 255)
                }
                A::Accel | A::Brake | A::Throttle => {
                    let v = pedal(axes.by_role(role));
                    return if self.game.starts_with("overrev") || self.game == "sgt24h" {
                        255 - v
                    } else {
                        v
                    };
                }
                _ => {}
            }
        }
        if self.scheme == Scheme::Flight && !self.game.is_empty() {
            let swa = self.game.starts_with("swa");
            let (mid, lo, hi) = if swa { (127, 27, 227) } else { (128, 0, 255) };
            match role {
                A::StickX => {
                    return centered(
                        self.signal(S::SkyX)
                            * if self.game == "netmerc" || self.game == "wingwar360" {
                                1.0
                            } else {
                                -1.0
                            },
                        if self.game == "netmerc" { 127 } else { mid },
                        lo,
                        hi,
                    )
                }
                // gilrs Y is positive UP; MAME's non-reversed ADC Y is positive DOWN.
                A::StickY => {
                    return centered(
                        self.signal(S::SkyY)
                            * if self.game == "netmerc" || self.game == "wingwar360" {
                                1.0
                            } else {
                                -1.0
                            },
                        if self.game == "netmerc" { 127 } else { mid },
                        lo,
                        hi,
                    )
                }
                A::Stick2X | A::Stick2Y if swa => {
                    return centered(
                        -self.signal_p2(if role == A::Stick2X { S::SkyX } else { S::SkyY }),
                        127,
                        27,
                        227,
                    );
                }
                A::Throttle if swa || self.game.starts_with("wingwar") => {
                    // Two assignable half-axes drive one cabinet ADC. Centre
                    // at release is a gamepad adaptation, not MAME's idle value.
                    // User-selected polarity: Up raises the ADC, Down lowers it.
                    // Keep signal names and right-stick assignments unchanged.
                    return centered(
                        self.signal(S::ThrottleUp) - self.signal(S::ThrottleDown),
                        128,
                        if swa { 28 } else { 1 },
                        if swa { 228 } else { 255 },
                    );
                }
                _ => {}
            }
        }
        match role {
            // Positional gun cabinets have calibrated ADC travel, not the
            // serial lightgun coordinates. rchase2a really differs from rchase2.
            A::Gun1X | A::Gun1Y | A::Gun2X | A::Gun2Y if !self.serial_gun() => {
                let (min, max, _rest, reverse) = self.positional_gun_range(role);
                // Do not quantize to an intermediate 0..255 axis before
                // applying the calibrated range (SM2 scales the cursor itself).
                let raw = match role {
                    A::Gun1X => self.cursor.0,
                    A::Gun1Y => self.cursor.1,
                    A::Gun2X => self.cursor_p2.0,
                    _ => self.cursor_p2.1,
                };
                let fraction = if reverse { 1.0 - raw } else { raw };
                (min as f32 + fraction * (max - min) as f32).round() as u8
            }
            A::Roll => axis(S::Roll),
            A::Pitch => axis(S::Pitch),
            A::Slide if self.scheme == Scheme::Skate => axis(S::SkaterSlide),
            A::Slide => centered(-self.signal(S::WaterSlide), 128, 0, 255),
            A::Curving => centered(-self.signal(S::Curving), 128, 0, 255),
            A::Swing => centered(-self.signal(S::Swing), 128, 0, 255),
            A::Incline => axis(S::Inclining),
            A::Bat1 => (255.0 * self.signal(S::BatSwing)).round() as u8,
            A::Bat2 => (255.0 * self.signal_p2(S::BatSwing)).round() as u8,
            A::P2R => (255.0 * self.signal_p2(S::Accelerator)).round() as u8,
            A::P2L => (255.0 * self.signal_p2(S::Brake)).round() as u8,
            _ => axes.by_role(role),
        }
    }

    pub(in crate::input) fn serial_gun(&self) -> bool {
        self.game.is_empty() || self.game.starts_with("vcop") || self.game.starts_with("hotd")
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
            || self.game.starts_with("indy500")
            || self.game.starts_with("stcc")
            || self.game.starts_with("overrev")
            || self.game == "skytargt"
            || self.game == "sgt24h";
        // No stale IN2 value may leak from a previous cabinet/frame.
        if matches!(self.scheme, Scheme::Racing | Scheme::Bike) {
            out.in2 = 0xff;
        }
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
        } else if self.game.starts_with("wingwar") {
            out.in1 = 0xff;
            write(&mut out.in1, 0x10, pressed(S::Action1));
            write(&mut out.in1, 0x20, pressed(S::Action2));
            write(&mut out.in1, 0x40, pressed(S::Action3));
            if self.game != "wingwar360" {
                for (bit, signal) in [(0x20, S::View1), (0x40, S::View2), (0x80, S::View3)] {
                    write(&mut out.in0, bit, pressed(signal));
                }
                write(&mut out.in1, 1, pressed(S::View4));
                out.in0 |= 2; // no Coin 2 on the standard cabinet
            }
        } else if self.game == "netmerc" {
            out.in0 |= 0xf2; // no Start switch or Coin 2
            out.in1 = 0xff;
            for (bit, signal) in [(1, S::Action1), (2, S::Action2), (4, S::Action3)] {
                write(&mut out.in1, bit, pressed(signal));
            }
        } else if self.game.starts_with("swa") {
            // Gunner is the pilot's stick + two fire buttons only. MAME names
            // IN0:20 Start2, but do not expose it as a gunner control (user's
            // cabinet convention); Start, view and throttle belong to P1.
            write(&mut out.in0, 0x20, false);
            write(&mut out.in1, 0x04, self.signal_p2(S::Action1) > 0.5);
            write(&mut out.in1, 0x08, self.signal_p2(S::Action2) > 0.5);
        } else if matches!(self.game.as_str(), "vr" | "vformula") {
            // Model 1 VR uses momentary, active-low shifts, NOT Daytona's
            // active-high H-gate code. Leave VR4 (bit 0) untouched.
            out.in1 |= 0x70;
            let up = self.on(C::GearUp);
            let down = self.on(C::GearDown);
            write(&mut out.in1, 0x20, up && !down);
            write(&mut out.in1, 0x10, down && !up);
        } else if self.scheme == Scheme::Bike || (self.scheme == Scheme::Racing && !self.h_gate()) {
            out.in1 = 0xff;
            // Motor Raid has independent Punch/Kick, not a sequential shifter.
            let independent = self.game.starts_with("motoraid");
            let up = pressed(if independent { S::Action1 } else { S::Action2 });
            let down = pressed(if independent { S::Action2 } else { S::Action1 });
            write(&mut out.in1, 0x10, up && (independent || !down));
            write(&mut out.in1, 0x20, down && (independent || !up));
            if self.game.starts_with("overrev")
                || self.game.starts_with("indy500")
                || self.game.starts_with("stcc")
            {
                write(&mut out.in1, 1, pressed(S::View4));
                write(&mut out.in1, 2, pressed(S::View1));
            } else if self.game == "sgt24h" {
                write(&mut out.in1, 1, pressed(S::View1));
            }
        }
        if self.game.starts_with("srally") {
            // Rally handbrake is active high, unlike cabinet buttons.
            out.in2 = (255.0 * self.signal(S::Action4).clamp(0.0, 1.0)).round() as u8;
            write(&mut out.in0, 0x20, pressed(S::View1));
            write(&mut out.in1, 1, false);
        }
        if (self.scheme == Scheme::Joystick
            && !self.game.starts_with("von")
            && self.game != "rascot2")
            || matches!(self.scheme, Scheme::Gun | Scheme::Sled)
        {
            write(&mut out.in0, 0x20, self.signal_p2(S::Start) > 0.5);
        }
        if self.scheme == Scheme::Joystick
            && !self.game.starts_with("von")
            && self.game != "rascot2"
        {
            // Local second player, not a linked cabinet. Virtual On's IN2 is
            // P1's right stick; Royal Ascot II has only one gameplay panel.
            out.in2 = 0xff;
            for (mask, control) in [
                (1, C::Button1),
                (2, C::Button2),
                (4, C::Button3),
                (0x10, C::Down),
                (0x20, C::Up),
                (0x40, C::Right),
                (0x80, C::Left),
            ] {
                write(
                    &mut out.in2,
                    mask,
                    self.signal_p2(self.control_signal(control)) > 0.5,
                );
            }
        }
        // These boards have only two action buttons. In particular Bat Swing
        // must not also produce an unused third button in Dynamite Baseball.
        if self.game.starts_with("dynabb")
            || self.game.starts_with("pltkids")
            || self.game.starts_with("zerogun")
        {
            out.in1 |= 0x0c;
            out.in2 |= 0x0c;
        }
        if self.game == "segawski" {
            out.in0 |= 0x10;
            out.in1 = 0xff;
            write(&mut out.in0, 0x40, pressed(S::Start));
            write(&mut out.in1, 2, pressed(S::View4));
            write(&mut out.in1, 1, pressed(S::Action3));
            write(&mut out.in1, 4, pressed(S::Action2)); // Pitch Left
            write(&mut out.in1, 8, pressed(S::Action1)); // Pitch Right
        } else if self.game == "skisuprg" {
            out.in1 = 0xff;
            write(&mut out.in0, 0x20, pressed(S::View4));
            write(&mut out.in1, 1, pressed(S::View1));
            write(&mut out.in0, 0x80, pressed(S::Action4));
            write(&mut out.in0, 0x10, pressed(S::Start));
            write(&mut out.in0, 0x40, pressed(S::Action3));
            out.in2 = if pressed(S::Action2) { 0xf0 } else { 0 } // left foot
                | if pressed(S::Action1) { 0x0f } else { 0 }; // right foot
        } else if self.game.starts_with("topskatr") {
            out.in1 = 0xff;
            write(&mut out.in0, 0x40, pressed(S::Start));
            write(&mut out.in0, 0x80, pressed(S::View2));
            write(&mut out.in0, 0x10, pressed(S::View3));
            write(&mut out.in0, 0x20, pressed(S::Action1));
            write(&mut out.in1, 1, pressed(S::Action2));
        }
        if self.game == "bel" {
            // BEL reverses the electrical Test/Service lines, not their bindings.
            write(&mut out.in0, 4, self.on(C::Service));
            write(&mut out.in0, 8, self.on(C::Test));
            out.gun_offscreen = false;
            write(&mut out.in1, 1, pressed(S::Action1) || self.mouse_fire);
            write(&mut out.in1, 0x10, pressed(S::Action2) || self.mouse_reload);
            write(&mut out.in1, 0x20, self.signal_p2(S::Action2) > 0.5);
        }
        if self.game == "segawski" || self.game == "waverunr" {
            out.in0 |= 2; // single coin input, not Coin 2
        }
        if self.scheme == Scheme::Sled {
            // Do not let Action 3 operate the second seat's Entry input.
            out.in1 |= 0xfc;
            write(&mut out.in1, 4, self.signal_p2(S::Action1) > 0.5);
            write(&mut out.in1, 8, self.signal_p2(S::Action2) > 0.5);
            write(&mut out.in0, 0x80, pressed(S::Action4));
        }
    }
}

fn centered(value: f32, rest: u8, min: u8, max: u8) -> u8 {
    let v = value.clamp(-1.0, 1.0);
    let span = if v < 0.0 { rest - min } else { max - rest };
    (rest as f32 + v * span as f32).round() as u8
}
