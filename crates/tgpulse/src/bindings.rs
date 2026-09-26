//! User-facing signal bindings and emulator hotkeys.
//! The old Control identifiers are internal cabinet requests, not a second catalogue.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::input::signals::{self, expression::Binding, Signal};
use gilrs::{Axis, Button};
use winit::keyboard::KeyCode;

/// A cabinet control, named by what it does rather than where it is.
///
/// Not every machine has every one of these: a scheme reads the ones its
/// cabinet had, so binding `GearUp` does nothing in a fighting game.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Control {
    // Direction, as an 8-way stick or the digital edges of an analog control.
    Up,
    Down,
    Left,
    Right,
    // Attack/action buttons, in the order the I/O board reports them.
    Button1,
    Button2,
    Button3,
    Button4,
    // Cabinet furniture.
    Coin1,
    Coin2,
    Start1,
    Test,
    Service,
    // Daytona's four coloured view buttons.
    ViewRed,
    ViewBlue,
    ViewYellow,
    ViewGreen,
    // Racing.
    Throttle,
    Brake,
    SteerLeft,
    SteerRight,
    GearUp,
    GearDown,
    // Lightgun.
    Fire,
    Reload,
    // Bikes, jetskis, skis and skateboards.
    LeanLeft,
    LeanRight,
    ViewChange,
}

/// Something the emulator itself does, rather than the machine.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Hotkey {
    ToggleMenu,
    Fullscreen,
    SaveState,
    LoadState,
    NextSlot,
    PreviousSlot,
    Reset,
    Pause,
    FastForward,
}

impl Hotkey {
    pub const ALL: &'static [Hotkey] = &[
        Hotkey::ToggleMenu,
        Hotkey::Fullscreen,
        Hotkey::SaveState,
        Hotkey::LoadState,
        Hotkey::NextSlot,
        Hotkey::PreviousSlot,
        Hotkey::Reset,
        Hotkey::Pause,
        Hotkey::FastForward,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Hotkey::ToggleMenu => "Show/hide menu",
            Hotkey::Fullscreen => "Fullscreen",
            Hotkey::SaveState => "Save state",
            Hotkey::LoadState => "Load state",
            Hotkey::NextSlot => "Next state slot",
            Hotkey::PreviousSlot => "Previous state slot",
            Hotkey::Reset => "Reset machine",
            Hotkey::Pause => "Pause",
            Hotkey::FastForward => "Fast forward (hold)",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Hotkey::ToggleMenu => "toggle_menu",
            Hotkey::Fullscreen => "fullscreen",
            Hotkey::SaveState => "save_state",
            Hotkey::LoadState => "load_state",
            Hotkey::NextSlot => "next_slot",
            Hotkey::PreviousSlot => "previous_slot",
            Hotkey::Reset => "reset",
            Hotkey::Pause => "pause",
            Hotkey::FastForward => "fast_forward",
        }
    }

    fn from_key(s: &str) -> Option<Hotkey> {
        Hotkey::ALL.iter().copied().find(|h| h.key() == s)
    }
}

/// One physical thing that can drive a control.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Key(KeyCode),
    Pad(Button),
    /// A stick or trigger past the deadzone in one direction.
    PadAxis(Axis, Sign),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sign {
    Positive,
    Negative,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Key(k) => write!(f, "{}", key_name(*k)),
            Source::Pad(b) => write!(f, "Pad {}", pad_button_name(*b)),
            Source::PadAxis(a, s) => write!(
                f,
                "Pad {}{}",
                pad_axis_name(*a),
                match s {
                    Sign::Positive => "+",
                    Sign::Negative => "-",
                }
            ),
        }
    }
}

/// Everything the player has bound.
#[derive(Clone, Debug)]
pub struct Bindings {
    pub controls: BTreeMap<Signal, Binding>,
    pub hotkeys: BTreeMap<Hotkey, KeyCode>,
}

impl Default for Bindings {
    /// Match SM2-Emu's RetroPad positions where a cabinet control has the
    /// same meaning. Keyboard bindings retain their original layout.
    fn default() -> Self {
        let controls = signals::defaults();

        let hotkeys = BTreeMap::from([
            (Hotkey::ToggleMenu, KeyCode::F1),
            (Hotkey::Fullscreen, KeyCode::F11),
            (Hotkey::SaveState, KeyCode::F5),
            (Hotkey::LoadState, KeyCode::F7),
            (Hotkey::NextSlot, KeyCode::F6),
            (Hotkey::PreviousSlot, KeyCode::F4),
            (Hotkey::Reset, KeyCode::F3),
            (Hotkey::Pause, KeyCode::F9),
            (Hotkey::FastForward, KeyCode::Tab),
        ]);

        Self { controls, hotkeys }
    }
}

impl Bindings {
    pub fn binding(&self, signal: Signal) -> &Binding {
        &self.controls[&signal]
    }
    pub fn set_expression(&mut self, signal: Signal, text: &str) -> Result<(), String> {
        self.controls
            .insert(signal, Binding::parse(text, signal.signed())?);
        Ok(())
    }

    pub fn hotkey(&self, hotkey: Hotkey) -> Option<KeyCode> {
        self.hotkeys.get(&hotkey).copied()
    }

    /// The hotkey a key press triggers, if any.
    pub fn hotkey_for(&self, key: KeyCode) -> Option<Hotkey> {
        self.hotkeys
            .iter()
            .find(|(_, bound)| **bound == key)
            .map(|(hotkey, _)| *hotkey)
    }

    pub fn bind_hotkey(&mut self, hotkey: Hotkey, key: KeyCode) {
        // A key drives one hotkey; taking it from another is the intent.
        self.hotkeys.retain(|_, bound| *bound != key);
        self.hotkeys.insert(hotkey, key);
    }

    /// Reads the bindings, writing the defaults out first if there is no file
    /// yet -- so the format is discoverable and hand-editable without having to
    /// change something in the interface to make one appear.
    pub fn load_or_create(path: &Path) -> Self {
        if !path.exists() {
            let defaults = Self::default();
            if let Err(e) = defaults.save(path) {
                log::warn!(target: "input", "cannot write {}: {e}", path.display());
            }
            return defaults;
        }
        let bindings = Self::load(path);
        if let Ok(text) = std::fs::read_to_string(path) {
            if !text.lines().any(|l| l.trim() == "format = signals-v1") {
                let backup = path.with_extension("conf.pre-signals");
                // Never overwrite an earlier backup.
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&backup)
                {
                    Ok(mut file) => {
                        use std::io::Write;
                        if file.write_all(text.as_bytes()).is_ok() {
                            if let Err(e) = bindings.save(path) {
                                log::warn!("Cannot save migrated bindings: {e}");
                            }
                        }
                    }
                    Err(e) => log::warn!(
                        "Cannot back up {}; migration remains in memory: {e}",
                        path.display()
                    ),
                }
            }
        }
        bindings
    }

    pub fn path() -> PathBuf {
        PathBuf::from("config").join("input.conf")
    }

    /// Reads the file, falling back to the defaults for anything it does not
    /// mention -- so a file written by an older build still works, and so a
    /// control the player has not touched keeps its shipped binding.
    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let mut bindings = Self::default();
        let modern = text.lines().any(|l| l.trim() == "format = signals-v1");
        if !modern {
            bindings.migrate_keyboard(&text);
        }
        for (number, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((name, value)) = line.split_once('=') else {
                log::warn!(target: "input", "{}:{}: not a binding", path.display(), number + 1);
                continue;
            };
            let (name, value) = (name.trim(), value.trim());
            if name == "format" {
                continue;
            }
            if let Some(signal) = Signal::from_key(name).filter(|_| modern) {
                if let Err(e) = bindings.set_expression(signal, value) {
                    log::warn!("Invalid binding {name}: {e}; keeping default");
                }
            } else if let Some(hotkey) = Hotkey::from_key(name) {
                if let Some(key) = parse_key(value) {
                    bindings.hotkeys.insert(hotkey, key);
                } else if value.is_empty() {
                    bindings.hotkeys.remove(&hotkey);
                }
            } else if modern {
                log::warn!(target: "input", "{}:{}: unknown control '{name}'", path.display(), number + 1);
            }
        }
        log::info!(target: "input", "bindings from {}", path.display());
        bindings
    }

    fn migrate_keyboard(&mut self, text: &str) {
        // Import keyboard assignments once, keeping the new controller defaults.
        let old: BTreeMap<&str, Vec<&str>> = text
            .lines()
            .filter_map(|l| l.split('#').next()?.split_once('='))
            .map(|(k, v)| {
                (
                    k.trim(),
                    v.split(',')
                        .map(str::trim)
                        .filter(|s| parse_key(s).is_some())
                        .collect(),
                )
            })
            .collect();
        let mut merged: BTreeMap<Signal, Vec<&str>> = BTreeMap::new();
        for (old_name, signal) in [
            ("coin1", Signal::Coin),
            ("coin2", Signal::Coin2),
            ("start1", Signal::Start),
            ("start2", Signal::Start2),
            ("test", Signal::Test),
            ("service", Signal::Service),
            ("up", Signal::Up),
            ("down", Signal::Down),
            ("left", Signal::Left),
            ("right", Signal::Right),
            ("button1", Signal::Action1),
            ("button2", Signal::Action2),
            ("button3", Signal::Action3),
            ("button4", Signal::Action4),
            ("view_red", Signal::View1),
            ("view_blue", Signal::View2),
            ("view_yellow", Signal::View3),
            ("view_green", Signal::View4),
            ("gear_up", Signal::Action1),
            ("gear_down", Signal::Action2),
            ("fire", Signal::Action1),
            ("reload", Signal::Action2),
        ] {
            if let Some(keys) = old.get(old_name) {
                merged
                    .entry(signal)
                    .or_default()
                    .extend(keys.iter().copied());
            }
        }
        for (signal, mut keys) in merged {
            keys.sort_unstable();
            keys.dedup();
            let pad: Vec<_> = signal
                .default_text()
                .split(',')
                .map(str::trim)
                .filter(|s| s.starts_with("pad:"))
                .collect();
            self.set_expression(
                signal,
                &keys.into_iter().chain(pad).collect::<Vec<_>>().join(", "),
            )
            .expect("valid migrated keys");
        }
        for (negative, positive, signals) in [(
            "left",
            "right",
            &[
                Signal::Steering,
                Signal::Handle,
                Signal::SkyX,
                Signal::Curving,
                Signal::Swing,
            ][..],
        )] {
            if let (Some(neg), Some(pos)) = (old.get(negative), old.get(positive)) {
                if !neg.is_empty() && !pos.is_empty() {
                    let pairs: Vec<_> = (0..neg.len().max(pos.len()))
                        .map(|i| format!("keys:{}/{}", neg[i % neg.len()], pos[i % pos.len()]))
                        .collect();
                    for &signal in signals {
                        let pad: Vec<_> = signal
                            .default_text()
                            .split(',')
                            .map(str::trim)
                            .filter(|s| s.starts_with("pad:"))
                            .collect();
                        self.set_expression(
                            signal,
                            &pairs
                                .iter()
                                .map(String::as_str)
                                .chain(pad)
                                .collect::<Vec<_>>()
                                .join(", "),
                        )
                        .unwrap();
                    }
                }
            }
        }
        for (direction, pedal, signal) in [
            ("up", "throttle", Signal::Accelerator),
            ("down", "brake", Signal::Brake),
        ] {
            if old.contains_key(direction) || old.contains_key(pedal) {
                let mut keys = old.get(direction).cloned().unwrap_or_default();
                keys.extend(old.get(pedal).cloned().unwrap_or_default());
                keys.sort_unstable();
                keys.dedup();
                let pad: Vec<_> = signal
                    .default_text()
                    .split(',')
                    .map(str::trim)
                    .filter(|s| s.starts_with("pad:"))
                    .collect();
                self.set_expression(
                    signal,
                    &keys.into_iter().chain(pad).collect::<Vec<_>>().join(", "),
                )
                .unwrap();
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let mut out = String::from("# TGPulse: one signal list for all games.\n# Comma = alternatives; & = simultaneous chord.\n# pad:Axis = signed axis, pad:Axis~ = inverted, +/- = half axis.\n# keys:Negative/Positive = keyboard axis. Empty = unbound.\nformat = signals-v1\n\n");
        for signal in Signal::ALL {
            out += &format!(
                "# {}\n{} = {}\n",
                signal.label(),
                signal.key(),
                self.binding(*signal).text
            );
        }
        out += "\n# Emulator hotkeys\n";
        for hotkey in Hotkey::ALL {
            let key = self.hotkey(*hotkey).map(key_token).unwrap_or_default();
            out += &format!("{} = {}\n", hotkey.key(), key);
        }
        std::fs::write(path, out).map_err(|e| e.to_string())
    }
}

pub(crate) fn parse_source(token: &str) -> Option<Source> {
    if let Some(rest) = token.strip_prefix("pad:") {
        if let Some(name) = rest.strip_suffix('+') {
            return pad_axis_from_token(name).map(|a| Source::PadAxis(a, Sign::Positive));
        }
        if let Some(name) = rest.strip_suffix('-') {
            return pad_axis_from_token(name).map(|a| Source::PadAxis(a, Sign::Negative));
        }
        return pad_button_from_token(rest).map(Source::Pad);
    }
    parse_key(token).map(Source::Key)
}

/// Keys are written as winit names it it, which are stable and unambiguous.
fn key_token(key: KeyCode) -> String {
    format!("{key:?}")
}

pub(crate) fn parse_key(token: &str) -> Option<KeyCode> {
    KEYS.iter()
        .copied()
        .find(|k| format!("{k:?}").eq_ignore_ascii_case(token))
}

/// A friendlier spelling for the interface; the file keeps the exact name.
fn key_name(key: KeyCode) -> String {
    let raw = format!("{key:?}");
    for prefix in ["Key", "Digit", "Numpad", "Arrow"] {
        if let Some(rest) = raw.strip_prefix(prefix) {
            return match prefix {
                "Numpad" => format!("Num {rest}"),
                "Arrow" => rest.to_string(),
                _ => rest.to_string(),
            };
        }
    }
    raw
}

fn pad_button_from_token(token: &str) -> Option<Button> {
    PAD_BUTTONS
        .iter()
        .copied()
        .find(|b| format!("{b:?}").eq_ignore_ascii_case(token))
}

/// Pad buttons named as they are printed on the two common layouts, since
/// "South" means nothing to anyone holding the controller.
fn pad_button_name(button: Button) -> &'static str {
    match button {
        Button::South => "A / Cross",
        Button::East => "B / Circle",
        Button::West => "X / Square",
        Button::North => "Y / Triangle",
        Button::LeftTrigger => "L1 / LB",
        Button::RightTrigger => "R1 / RB",
        Button::LeftTrigger2 => "L2 / LT",
        Button::RightTrigger2 => "R2 / RT",
        Button::Select => "Select",
        Button::Start => "Start",
        Button::LeftThumb => "L3",
        Button::RightThumb => "R3",
        Button::DPadUp => "D-pad up",
        Button::DPadDown => "D-pad down",
        Button::DPadLeft => "D-pad left",
        Button::DPadRight => "D-pad right",
        _ => "button",
    }
}

fn pad_axis_from_token(token: &str) -> Option<Axis> {
    PAD_AXES
        .iter()
        .copied()
        .find(|a| format!("{a:?}").eq_ignore_ascii_case(token))
}

fn pad_axis_name(axis: Axis) -> &'static str {
    match axis {
        Axis::LeftStickX => "left stick X",
        Axis::LeftStickY => "left stick Y",
        Axis::RightStickX => "right stick X",
        Axis::RightStickY => "right stick Y",
        Axis::LeftZ => "left trigger",
        Axis::RightZ => "right trigger",
        _ => "axis",
    }
}

/// The keys offered for binding. Anything a cabinet button might reasonably
/// live on; the exotic ones are left out so the picker stays readable.
pub const KEYS: &[KeyCode] = &[
    KeyCode::KeyA,
    KeyCode::KeyB,
    KeyCode::KeyC,
    KeyCode::KeyD,
    KeyCode::KeyE,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyI,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::KeyM,
    KeyCode::KeyN,
    KeyCode::KeyO,
    KeyCode::KeyP,
    KeyCode::KeyQ,
    KeyCode::KeyR,
    KeyCode::KeyS,
    KeyCode::KeyT,
    KeyCode::KeyU,
    KeyCode::KeyV,
    KeyCode::KeyW,
    KeyCode::KeyX,
    KeyCode::KeyY,
    KeyCode::KeyZ,
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::Space,
    KeyCode::Enter,
    KeyCode::NumpadEnter,
    KeyCode::Tab,
    KeyCode::Backspace,
    KeyCode::ShiftLeft,
    KeyCode::ShiftRight,
    KeyCode::ControlLeft,
    KeyCode::ControlRight,
    KeyCode::AltLeft,
    KeyCode::AltRight,
    KeyCode::Comma,
    KeyCode::Period,
    KeyCode::Slash,
    KeyCode::Semicolon,
    KeyCode::Quote,
    KeyCode::BracketLeft,
    KeyCode::BracketRight,
    KeyCode::Minus,
    KeyCode::Equal,
    KeyCode::Backquote,
    KeyCode::F1,
    KeyCode::F2,
    KeyCode::F3,
    KeyCode::F4,
    KeyCode::F5,
    KeyCode::F6,
    KeyCode::F7,
    KeyCode::F8,
    KeyCode::F9,
    KeyCode::F10,
    KeyCode::F11,
    KeyCode::F12,
];

pub const PAD_BUTTONS: &[Button] = &[
    Button::South,
    Button::East,
    Button::West,
    Button::North,
    Button::LeftTrigger,
    Button::RightTrigger,
    Button::LeftTrigger2,
    Button::RightTrigger2,
    Button::Select,
    Button::Start,
    Button::LeftThumb,
    Button::RightThumb,
    Button::DPadUp,
    Button::DPadDown,
    Button::DPadLeft,
    Button::DPadRight,
];

pub const PAD_AXES: &[Axis] = &[
    Axis::LeftStickX,
    Axis::LeftStickY,
    Axis::RightStickX,
    Axis::RightStickY,
    Axis::LeftZ,
    Axis::RightZ,
];

impl FromStr for Source {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        parse_source(s).ok_or_else(|| format!("unknown input source '{s}'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_signals_round_trip() {
        let path =
            std::env::temp_dir().join(format!("tgpulse-signals-{}.conf", std::process::id()));
        let mut written = Bindings::default();
        written
            .set_expression(
                Signal::Gear1,
                "KeyJ & KeyK, pad:RightStickX- & pad:RightStickY+",
            )
            .unwrap();
        written.set_expression(Signal::Coin2, "").unwrap();
        written.bind_hotkey(Hotkey::Reset, KeyCode::F5);
        written.save(&path).unwrap();
        let read = Bindings::load(&path);
        for signal in Signal::ALL {
            assert_eq!(read.binding(*signal), written.binding(*signal));
        }
        assert_eq!(read.hotkeys, written.hotkeys);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn invalid_binding_is_not_applied() {
        let mut b = Bindings::default();
        assert!(b.set_expression(Signal::Gear1, "KeyJ & nonsense").is_err());
        assert_eq!(b.binding(Signal::Gear1).text, Signal::Gear1.default_text());
    }
    #[test]
    fn migration_keeps_custom_keyboard_and_hotkey_uniqueness() {
        let mut b = Bindings::default();
        b.migrate_keyboard("coin1 = F12, pad:North");
        assert_eq!(b.binding(Signal::Coin).text, "F12, pad:Select");
        b.bind_hotkey(Hotkey::Reset, KeyCode::F5);
        assert_eq!(b.hotkey(Hotkey::SaveState), None);
    }

    #[test]
    fn migration_backs_up_original_once() {
        let path =
            std::env::temp_dir().join(format!("tgpulse-migrate-{}.conf", std::process::id()));
        let backup = path.with_extension("conf.pre-signals");
        let original = "coin1 = F12\nleft = KeyA\nright = KeyD\nfullscreen = F10\n";
        std::fs::write(&path, original).unwrap();
        let migrated = Bindings::load_or_create(&path);
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), original);
        assert!(migrated
            .binding(Signal::Steering)
            .text
            .contains("keys:KeyA/KeyD"));
        assert_eq!(migrated.hotkey(Hotkey::Fullscreen), Some(KeyCode::F10));
        assert_eq!(Bindings::load_or_create(&path).controls, migrated.controls);
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), original);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(backup).unwrap();
    }
}
