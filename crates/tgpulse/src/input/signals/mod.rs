//! Single user-facing catalogue. Cabinet controls remain private routing requests.
pub mod expression;
mod routing;
use expression::Binding;
use std::collections::BTreeMap;
macro_rules! signals {
    ($( $id:ident, $label:literal, $key:literal, $axis:literal, $default:literal; )*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Signal { $( $id, )* }
        impl Signal {
            pub const ALL: &'static [Self] = &[$( Self::$id, )*];
            pub fn label(self) -> &'static str { match self { $( Self::$id => $label, )* } }
            pub fn key(self) -> &'static str { match self { $( Self::$id => $key, )* } }
            pub fn signed(self) -> bool { match self { $( Self::$id => $axis, )* } }
            pub fn default_text(self) -> &'static str { match self { $( Self::$id => $default, )* } }
            pub fn from_key(key: &str) -> Option<Self> { Self::ALL.iter().copied().find(|s| s.key()==key) }
        }
    }
}
signals! {
    Coin, "Coin", "coin", false, "Digit5, pad:Select";
    Coin2, "Coin 2", "coin2", false, "Digit6";
    Start, "Start", "start", false, "Enter, NumpadEnter, pad:Start";
    Start2, "Start 2", "start2", false, "Digit2";
    Test, "Test", "test", false, "F2, pad:LeftThumb";
    Service, "Service", "service", false, "F8, pad:RightThumb";
    Up, "Joystick Up", "up", false, "ArrowUp, KeyW, pad:DPadUp";
    Down, "Joystick Down", "down", false, "ArrowDown, KeyS, pad:DPadDown";
    Left, "Joystick Left", "left", false, "ArrowLeft, KeyA, pad:DPadLeft";
    Right, "Joystick Right", "right", false, "ArrowRight, KeyD, pad:DPadRight";
    Action1, "Action 1", "action1", false, "KeyJ, KeyE, Space, pad:East, pad:RightTrigger";
    Action2, "Action 2", "action2", false, "KeyK, KeyQ, KeyR, pad:South, pad:LeftTrigger";
    Action3, "Action 3", "action3", false, "KeyL, pad:West";
    Action4, "Extra Action", "action4", false, "KeyI, pad:North";
    View1, "View / Select 1", "view1", false, "KeyZ, pad:DPadDown";
    View2, "View / Select 2", "view2", false, "KeyX, pad:DPadLeft";
    View3, "View / Select 3", "view3", false, "KeyC, pad:DPadRight";
    View4, "View / Select 4", "view4", false, "KeyV, pad:DPadUp";
    Steering, "Steering / Bank", "steering", true, "keys:ArrowLeft/ArrowRight, keys:KeyA/KeyD, pad:LeftStickX";
    Accelerator, "Accelerator", "accelerator", false, "KeyW, ArrowUp, pad:RightZ+";
    Brake, "Brake", "brake", false, "KeyS, ArrowDown, pad:LeftZ+";
    Gear1, "H-Gate: Gear 1", "gear1", false, "Digit1, pad:RightStickX- & pad:RightStickY+";
    Gear2, "H-Gate: Gear 2", "gear2", false, "Digit2, pad:RightStickX- & pad:RightStickY-";
    Gear3, "H-Gate: Gear 3", "gear3", false, "Digit3, pad:RightStickX+ & pad:RightStickY+";
    Gear4, "H-Gate: Gear 4", "gear4", false, "Digit4, pad:RightStickX+ & pad:RightStickY-";
    Neutral, "H-Gate: Neutral", "neutral", false, "Digit0, pad:West";
    SkyX, "Analog Joystick X", "analog_x", true, "keys:ArrowLeft/ArrowRight, keys:KeyA/KeyD, pad:LeftStickX";
    SkyY, "Analog Joystick Y", "analog_y", true, "keys:KeyG/KeyT, pad:LeftStickY";
    Handle, "Wave Runner: Handle", "handle", true, "keys:ArrowLeft/ArrowRight, keys:KeyA/KeyD, pad:LeftStickX";
    GunYaw, "Gun Yaw", "gun_yaw", true, "keys:ArrowLeft/ArrowRight, keys:KeyA/KeyD, pad:LeftStickX";
    GunPitch, "Gun Pitch", "gun_pitch", true, "keys:ArrowDown/ArrowUp, keys:KeyS/KeyW, pad:LeftStickY";
    Elevation, "Desert Tank: Elevation", "elevation", true, "keys:KeyG/KeyT, pad:LeftStickY";
    TwinLeftX, "Virtual On: Left Joystick X", "twin_left_x", true, "keys:KeyA/KeyD, pad:LeftStickX";
    TwinLeftY, "Virtual On: Left Joystick Y", "twin_left_y", true, "keys:KeyS/KeyW, pad:LeftStickY";
    TwinRightX, "Virtual On: Right Joystick X", "twin_right_x", true, "keys:ArrowLeft/ArrowRight, pad:RightStickX";
    TwinRightY, "Virtual On: Right Joystick Y", "twin_right_y", true, "keys:ArrowDown/ArrowUp, pad:RightStickY";
    Pitch, "Wave Runner: Pitch", "pitch", true, "keys:KeyG/KeyT, pad:LeftStickY";
    Roll, "Wave Runner: Roll", "roll", true, "keys:KeyU/KeyO, pad:RightStickX";
    WaterSlide, "Water Ski: Slide", "water_ski_slide", true, "keys:ArrowLeft/ArrowRight, keys:KeyA/KeyD, pad:LeftStickX";
    SkaterSlide, "Top Skater: Slide", "top_skater_slide", true, "keys:KeyU/KeyO, pad:RightStickX";
    Curving, "Top Skater: Curving", "curving", true, "keys:ArrowLeft/ArrowRight, keys:KeyA/KeyD, pad:LeftStickX";
    Swing, "Ski Super G: Swing", "swing", true, "keys:ArrowLeft/ArrowRight, keys:KeyA/KeyD, pad:LeftStickX";
    Inclining, "Ski Super G: Inclining", "inclining", true, "keys:KeyU/KeyO, pad:RightStickX";
    BatSwing, "Bat Swing", "bat_swing", false, "KeyI, pad:RightStickY-";
}
pub fn defaults() -> BTreeMap<Signal, Binding> {
    Signal::ALL
        .iter()
        .map(|&s| {
            (
                s,
                Binding::parse(s.default_text(), s.signed()).expect("valid default"),
            )
        })
        .collect()
}

impl Signal {
    /// Game families, including their revisions. Kept with the public labels.
    pub fn usage(self) -> Option<&'static str> {
        match self {
            Self::SkyX | Self::SkyY => Some("(Sky Target, Star Wars Arcade, Wing War, NetMerc)"),
            Self::Action4 => Some(
                "(Sega Rally: Handbrake; Virtual On: Right Dash / Turbo; Ski Super G: Select 2)",
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn driving_signals_are_consecutive() {
        let start = Signal::ALL
            .iter()
            .position(|s| *s == Signal::Steering)
            .unwrap();
        assert_eq!(
            &Signal::ALL[start..start + 8],
            &[
                Signal::Steering,
                Signal::Accelerator,
                Signal::Brake,
                Signal::Gear1,
                Signal::Gear2,
                Signal::Gear3,
                Signal::Gear4,
                Signal::Neutral
            ]
        );
    }
}
