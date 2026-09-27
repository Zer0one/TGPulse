//! Independent expectations from the SM2 profile workbook and MAME port maps.
//! See docs/INPUT_AUDIT.md for sources, intentional layout differences and limits.
use super::Signal as S;
use crate::bindings::Bindings;
use crate::input::{AnalogRole as A, ControlScheme as Scheme, InputState};
use tgpulse_core::config::Inputs;
use winit::keyboard::KeyCode;

struct Cabinet {
    sets: &'static str,
    scheme: Scheme,
    idle: [u8; 3],
    start: u8,
    start2: bool,
    coin2: bool,
    buttons: &'static [(S, usize, u8)],
}

fn cabinets() -> Vec<Cabinet> {
    use Scheme::*;
    use S::*;
    let mut cases = Vec::new();
    let mut add = |sets, scheme, idle, start, start2, coin2, buttons| {
        cases.push(Cabinet {
            sets,
            scheme,
            idle,
            start,
            start2,
            coin2,
            buttons,
        });
    };
    add(
        "daytona daytona93 daytonagtx daytonam daytonas daytonase daytonat daytonata",
        Racing,
        [255, 143, 255],
        0x10,
        false,
        true,
        &[
            (View1, 0, 0x20),
            (View2, 0, 0x40),
            (View3, 0, 0x80),
            (View4, 1, 1),
            (Action2, 1, 0x20),
            (Gear1, 1, 0x20),
            (Gear2, 1, 0x10),
            (Gear3, 1, 0x60),
            (Gear4, 1, 0x50),
        ],
    );
    add(
        "vr vformula",
        Racing,
        [255; 3],
        0x10,
        false,
        true,
        &[
            (View1, 0, 0x20),
            (View2, 0, 0x40),
            (View3, 0, 0x80),
            (View4, 1, 1),
            (Action2, 1, 0x20),
            (Action1, 1, 0x10),
        ],
    );
    add(
        "srallyc srallycb srallycc srallycdx srallycdxa",
        Racing,
        [255, 143, 0],
        0x40,
        false,
        true,
        &[
            (View1, 0, 0x20),
            (Action4, 2, 255),
            (Action2, 1, 0x20),
            (Gear1, 1, 0x20),
            (Gear2, 1, 0x10),
            (Gear3, 1, 0x60),
            (Gear4, 1, 0x50),
        ],
    );
    add(
        "indy500 indy500d indy500to stcc stcca stccb stcco overrev overrevb overrevba",
        Racing,
        [255; 3],
        0x40,
        false,
        true,
        &[
            (View4, 1, 1),
            (View1, 1, 2),
            (Action2, 1, 0x10),
            (Action1, 1, 0x20),
        ],
    );
    add(
        "sgt24h",
        Racing,
        [255; 3],
        0x40,
        false,
        true,
        &[(View1, 1, 1), (Action2, 1, 0x10), (Action1, 1, 0x20)],
    );
    add(
        "manxtt manxttc manxttdx",
        Bike,
        [255; 3],
        0x40,
        false,
        true,
        &[(Action2, 1, 0x10), (Action1, 1, 0x20)],
    );
    add(
        "motoraid motoraiddx",
        Bike,
        [255; 3],
        0x40,
        false,
        true,
        &[(Action1, 1, 0x10), (Action2, 1, 0x20)],
    );
    add(
        "desert",
        Racing,
        [255; 3],
        0x10,
        false,
        true,
        &[
            (View1, 0, 0x20),
            (View2, 0, 0x40),
            (View4, 0, 0x80),
            (Action1, 1, 0x10),
            (Action2, 1, 0x20),
            (Action3, 1, 1),
        ],
    );
    add(
        "vf doa doaa doaab doaae doab",
        Joystick,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 2), (Action2, 1, 4), (Action3, 1, 1)],
    );
    add("vf2 vf2a vf2b vf2o fvipers fvipersa fvipersb lastbrnx lastbrnxj lastbrnxu dynamcop dynamcopb dynamcopc dyndeka2 dyndeka2b schamp sfight",
        Joystick, [255;3], 0x10, true, true,
        &[(Action1,1,1),(Action2,1,2),(Action3,1,4)]);
    add(
        "hpyagu98 airwlkrs",
        Joystick,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 2), (Action2, 1, 1), (Action3, 1, 4)],
    );
    add(
        "rascot2",
        Joystick,
        [255; 3],
        0x10,
        false,
        true,
        &[(Action1, 1, 2), (Action2, 1, 1), (Action3, 1, 4)],
    );
    add(
        "vstriker vstrikero",
        Joystick,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 1), (Action2, 1, 4), (Action3, 1, 2)],
    );
    add(
        "dynabb dynabb97",
        Joystick,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 2), (Action2, 1, 1)],
    );
    add(
        "zerogun zeroguna zerogunaj zerogunj pltkids pltkidsa",
        Joystick,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 1), (Action2, 1, 2)],
    );
    add(
        "von vonj vonr vonu",
        Joystick,
        [255; 3],
        0x10,
        false,
        true,
        &[
            (Action1, 1, 1),
            (Action2, 2, 1),
            (Action3, 1, 2),
            (Action4, 2, 2),
            (TwinLeftX, 1, 0x40),
            (TwinLeftY, 1, 0x20),
            (TwinRightX, 2, 0x40),
            (TwinRightY, 2, 0x20),
        ],
    );
    add(
        "vcop vcopa vcop2 hotd hotdo hotdp",
        Gun,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 1), (Action2, 1, 1)],
    );
    add(
        "gunblade rchase2 rchase2a",
        Gun,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 1)],
    );
    add(
        "bel",
        Gun,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 1), (Action2, 1, 0x10)],
    );
    add(
        "skytargt",
        Flight,
        [255; 3],
        0x40,
        false,
        true,
        &[(Action1, 1, 0x10), (Action2, 1, 0x20), (View4, 0, 0x20)],
    );
    add(
        "swa swaj",
        Flight,
        [255; 3],
        0x10,
        false, // Gunner has no Start: user cabinet convention, unlike MAME's generic Start2 bit.
        true,
        &[(Action1, 1, 1), (Action2, 1, 2), (View1, 1, 0x10)],
    );
    add(
        "wingwar wingwarj wingwaru",
        Flight,
        [255; 3],
        0x10,
        false,
        false,
        &[
            (Action1, 1, 0x10),
            (Action2, 1, 0x20),
            (Action3, 1, 0x40),
            (View1, 0, 0x20),
            (View2, 0, 0x40),
            (View3, 0, 0x80),
            (View4, 1, 1),
        ],
    );
    add(
        "wingwar360",
        Flight,
        [255; 3],
        0x10,
        false,
        true,
        &[(Action1, 1, 0x10), (Action2, 1, 0x20), (Action3, 1, 0x40)],
    );
    add(
        "netmerc",
        Flight,
        [255; 3],
        0,
        false,
        false,
        &[(Action1, 1, 1), (Action2, 1, 2), (Action3, 1, 4)],
    );
    add(
        "segawski",
        Ski,
        [255; 3],
        0x40,
        false,
        false,
        &[
            (View4, 1, 2),
            (Action1, 1, 8),
            (Action2, 1, 4),
            (Action3, 1, 1),
        ],
    );
    add(
        "skisuprg",
        Ski,
        [255, 255, 0],
        0x10,
        false,
        true,
        &[
            (View4, 0, 0x20),
            (View1, 1, 1),
            (Action1, 2, 0x0f),
            (Action2, 2, 0xf0),
            (Action3, 0, 0x40),
            (Action4, 0, 0x80),
        ],
    );
    add(
        "topskatr topskatrj topskatru topskatruo",
        Skate,
        [255; 3],
        0x40,
        false,
        true,
        &[
            (View2, 0, 0x80),
            (View3, 0, 0x10),
            (Action1, 0, 0x20),
            (Action2, 1, 1),
        ],
    );
    add(
        "waverunr",
        Jetski,
        [255, 255, 0xf7],
        0x40,
        false,
        false,
        &[(View4, 1, 1)],
    );
    add(
        "powsled powsledm powsledr",
        Sled,
        [255; 3],
        0x10,
        true,
        true,
        &[(Action1, 1, 1), (Action2, 1, 2), (Action4, 0, 0x80)],
    );
    cases
}

fn state(game: &str, scheme: Scheme) -> InputState {
    let mut input = InputState::with_gilrs(None);
    input.set_scheme(scheme);
    input.set_game(game);
    input
}

fn ports(out: &Inputs) -> [u8; 3] {
    [out.in0, out.in1, out.in2]
}

#[test]
fn every_set_routes_p2_signals_without_changing_p1_or_single_seat_controls() {
    use crate::input::players::Player;
    for cabinet in cabinets() {
        for game in cabinet.sets.split_whitespace() {
            for &signal in S::ALL.iter().filter(|s| s.supports_p2()) {
                let mut input = state(game, cabinet.scheme);
                input
                    .bindings
                    .set_player_expression(
                        Player::Two,
                        signal,
                        if signal.signed() {
                            "keys:F11/F12"
                        } else {
                            "F12"
                        },
                    )
                    .unwrap();
                input.on_key(KeyCode::F12, true);
                let mut out = Inputs::default();
                input.poll(&mut out);
                let mut expected = cabinet.idle;
                let common = match signal {
                    S::Coin if cabinet.coin2 => 2,
                    S::Start if cabinet.start2 => 0x20,
                    S::Test => {
                        if game == "bel" {
                            8
                        } else {
                            4
                        }
                    }
                    S::Service => {
                        if game == "bel" {
                            4
                        } else {
                            8
                        }
                    }
                    _ => 0,
                };
                expected[0] ^= common;
                if cabinet.scheme == Scheme::Joystick
                    && !game.starts_with("von")
                    && game != "rascot2"
                {
                    expected[2] ^= match signal {
                        S::Up => 0x20,
                        S::Down => 0x10,
                        S::Left => 0x80,
                        S::Right => 0x40,
                        _ => cabinet
                            .buttons
                            .iter()
                            .find(|&&(s, p, _)| s == signal && p == 1)
                            .map_or(0, |&(_, _, mask)| mask),
                    };
                } else if cabinet.scheme == Scheme::Gun {
                    let reload = matches!(
                        game,
                        "vcop" | "vcopa" | "vcop2" | "hotd" | "hotdo" | "hotdp"
                    ) && signal == S::Action2;
                    if signal == S::Action1 || reload {
                        let (port, mask) = if game == "hotd" { (2, 1) } else { (1, 2) };
                        expected[port] ^= mask;
                    }
                    if game == "bel" && signal == S::Action2 {
                        expected[1] ^= 0x20;
                    }
                    assert_eq!(out.gun2_offscreen, reload, "{game}: {signal:?}");
                    assert!(!out.gun_offscreen, "P2 reload must not reload P1");
                } else if game.starts_with("swa") || cabinet.scheme == Scheme::Sled {
                    expected[1] ^= match signal {
                        S::Action1 => 4,
                        S::Action2 => 8,
                        _ => 0,
                    };
                }
                assert_eq!(ports(&out), expected, "{game}: P2 {signal:?}");
                input.on_key(KeyCode::F12, false);
                input.poll(&mut out);
                assert_eq!(ports(&out), cabinet.idle, "{game}: P2 release {signal:?}");
            }
        }
    }
}

#[test]
fn p2_bats_and_sled_pedals_have_independent_full_range_adc_channels() {
    use crate::input::players::Player;
    for (games, signals) in [
        ("dynabb dynabb97", &[(S::BatSwing, A::Bat1, A::Bat2)][..]),
        (
            "powsled powsledm powsledr",
            &[(S::Accelerator, A::P1R, A::P2R), (S::Brake, A::P1L, A::P2L)][..],
        ),
    ] {
        for game in games.split_whitespace() {
            for &(signal, p1_role, p2_role) in signals {
                let mut input = state(
                    game,
                    if game.starts_with("dynabb") {
                        Scheme::Joystick
                    } else {
                        Scheme::Sled
                    },
                );
                let roles = db_roles(game);
                input.set_analog_roles(roles);
                // Feed the same normalized axis path as a real controller;
                // the default RightStickY/Z source strings are tested separately.
                input
                    .bindings
                    .set_player_expression(Player::Two, signal, "pad:LeftStickX+")
                    .unwrap();
                let ch1 = roles.iter().position(|&r| r == p1_role).unwrap();
                let ch2 = roles.iter().position(|&r| r == p2_role).unwrap();
                let mut out = Inputs::default();
                for (value, expected) in [(0.0, 0), (0.5, 128), (1.0, 255), (0.0, 0)] {
                    input.external_p2.left_x = value;
                    input.poll(&mut out);
                    assert_eq!(out.analog[ch2], expected, "{game}: {signal:?}");
                    assert_eq!(out.analog[ch1], 0, "{game}: P1 unaffected");
                    assert_eq!(ports(&out), [255; 3]);
                }
            }
        }
    }
}

#[test]
fn p2_gun_calibrations_cursor_hold_and_mouse_isolation() {
    for (game, xmin, xmax, ymin, ymax) in [
        ("vcop", 0x80, 0x273, 0x27, 0x1a9),
        ("vcopa", 0x80, 0x273, 0x27, 0x1a9),
        ("vcop2", 0x86, 0x273, 0x24, 0x1a9),
        ("hotd", 0xa3, 0x254, 0x57, 0x17c),
        ("hotdo", 0xa3, 0x254, 0x57, 0x17c),
        ("hotdp", 0xa3, 0x254, 0x57, 0x17c),
    ] {
        let mut input = state(game, Scheme::Gun);
        let mut out = Inputs::default();
        input.on_cursor(0.25, 0.75);
        input.poll(&mut out);
        let p1 = (out.gun_x, out.gun_y);
        for (x, y, expected) in [(-1.0, 1.0, (xmin, ymin)), (1.0, -1.0, (xmax, ymax))] {
            input.external_p2.left_x = x;
            input.external_p2.left_y = y;
            for _ in 0..60 {
                input.poll(&mut out);
            }
            assert_eq!((out.gun2_x, out.gun2_y), expected, "{game}");
            assert_eq!((out.gun_x, out.gun_y), p1);
        }
        input.external_p2.left_x = 0.0;
        input.external_p2.left_y = 0.0;
        input.poll(&mut out);
        assert_eq!(
            (out.gun2_x, out.gun2_y),
            (xmax, ymax),
            "cursor holds on release"
        );
        input.external_p2.buttons.insert(gilrs::Button::East);
        input.poll(&mut out);
        assert_eq!((out.gun2_x, out.gun2_y), (xmin, ymin));
        input.external_p2.buttons.clear();
        input.poll(&mut out);
        assert_eq!(
            (out.gun2_x, out.gun2_y),
            (xmax, ymax),
            "reload doesn't lose aim"
        );
    }
    for (game, xrole, yrole, min, max) in [
        ("gunblade", A::Gun2X, A::Gun2Y, [0x00, 0x11], [0x96, 0xae]),
        ("bel", A::Gun2X, A::Gun2Y, [0x00, 0x11], [0x96, 0xae]),
        ("rchase2", A::Gun2X, A::Gun2Y, [0xc7, 0xcb], [0x34, 0x1c]),
        ("rchase2a", A::Gun2X, A::Gun2Y, [0x00, 0x00], [0xff, 0xff]),
    ] {
        let mut input = state(game, Scheme::Gun);
        let roles = db_roles(game);
        input.set_analog_roles(roles);
        let xch = roles.iter().position(|&r| r == xrole).unwrap();
        let ych = roles.iter().position(|&r| r == yrole).unwrap();
        let mut out = Inputs::default();
        for (x, y, expected) in [(-1.0, 1.0, min), (1.0, -1.0, max)] {
            input.external_p2.left_x = x;
            input.external_p2.left_y = y;
            for _ in 0..60 {
                input.poll(&mut out);
            }
            assert_eq!([out.analog[xch], out.analog[ych]], expected, "{game}");
        }
    }
}

fn db_roles(game: &str) -> [A; 8] {
    let db = include_str!("../../../../tgpulse-core/src/roms_db.dat");
    let header = db
        .lines()
        .find(|line| line.starts_with(&format!("G {game} ")))
        .unwrap();
    let mut roles = [A::None; 8];
    for (i, name) in header.split_whitespace().skip(4).enumerate() {
        roles[i] = match name {
            "steer" => A::Steer,
            "accel" => A::Accel,
            "brake" => A::Brake,
            "throttle" => A::Throttle,
            "stickx" => A::StickX,
            "sticky" => A::StickY,
            "stick2x" => A::Stick2X,
            "stick2y" => A::Stick2Y,
            "gun1x" => A::Gun1X,
            "gun1y" => A::Gun1Y,
            "gun2x" => A::Gun2X,
            "gun2y" => A::Gun2Y,
            "roll" => A::Roll,
            "pitch" => A::Pitch,
            "slide" => A::Slide,
            "curving" => A::Curving,
            "swing" => A::Swing,
            "incline" => A::Incline,
            "bat1" => A::Bat1,
            "bat2" => A::Bat2,
            "p1r" => A::P1R,
            "p1l" => A::P1L,
            "p2r" => A::P2R,
            "p2l" => A::P2L,
            "none" => A::None,
            _ => panic!("unaudited role {name}"),
        };
    }
    roles
}

#[test]
fn every_catalogued_set_has_an_explicit_audit_case() {
    use std::collections::BTreeSet;
    let db = include_str!("../../../../tgpulse-core/src/roms_db.dat");
    let actual: BTreeSet<_> = db
        .lines()
        .filter_map(|line| line.strip_prefix("G "))
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    let cases = cabinets();
    let expected: BTreeSet<_> = cases
        .iter()
        .flat_map(|c| c.sets.split_whitespace())
        .collect();
    assert_eq!(actual, expected, "new/removed sets require an input audit");
    assert_eq!(
        cases
            .iter()
            .map(|c| c.sets.split_whitespace().count())
            .sum::<usize>(),
        expected.len()
    );
}

#[test]
fn all_sets_route_each_signal_without_digital_crosstalk() {
    for cabinet in cabinets() {
        for game in cabinet.sets.split_whitespace() {
            for &signal in S::ALL {
                let mut input = state(game, cabinet.scheme);
                let mut bindings = Bindings::default();
                for &s in S::ALL {
                    bindings.set_expression(s, "").unwrap();
                }
                bindings
                    .set_expression(
                        signal,
                        if signal.signed() {
                            "keys:F11/F12"
                        } else {
                            "F12"
                        },
                    )
                    .unwrap();
                input.set_bindings(bindings);
                let mut out = Inputs::default();
                input.poll(&mut out);
                assert_eq!(ports(&out), cabinet.idle, "{game} idle");
                input.on_key(KeyCode::F12, true);
                input.poll(&mut out);
                let mut expected = cabinet.idle;
                let common = match signal {
                    S::Coin => 1,
                    S::Test => {
                        if game == "bel" {
                            8
                        } else {
                            4
                        }
                    }
                    S::Service => {
                        if game == "bel" {
                            4
                        } else {
                            8
                        }
                    }
                    S::Start => cabinet.start,
                    _ => 0,
                };
                expected[0] ^= common;
                if cabinet.scheme == Scheme::Joystick && !game.starts_with("von") {
                    expected[1] ^= match signal {
                        S::Up => 0x20,
                        S::Down => 0x10,
                        S::Left => 0x80,
                        S::Right => 0x40,
                        _ => 0,
                    };
                }
                for &(s, port, mask) in cabinet.buttons {
                    if signal == s {
                        expected[port] ^= mask;
                    }
                }
                assert_eq!(ports(&out), expected, "{game}: {signal:?}");
                input.on_key(KeyCode::F12, false);
                input.poll(&mut out);
                let latching = (game.starts_with("daytona") || game.starts_with("srally"))
                    && matches!(
                        signal,
                        S::Action2 | S::Gear1 | S::Gear2 | S::Gear3 | S::Gear4
                    )
                    || game == "desert" && signal == S::Action3;
                assert_eq!(
                    ports(&out),
                    if latching { expected } else { cabinet.idle },
                    "{game}: release {signal:?}"
                );
            }
        }
    }
}

#[test]
fn default_indy_stcc_overrev_buttons_are_isolated() {
    use gilrs::Button as B;
    for game in [
        "indy500",
        "indy500d",
        "indy500to",
        "stcc",
        "stcca",
        "stccb",
        "stcco",
        "overrev",
        "overrevb",
        "overrevba",
    ] {
        let mut input = state(game, Scheme::Racing);
        let mut out = Inputs::default();
        for (button, expected) in [
            (B::Start, [0xbf, 255, 255]),
            (B::DPadUp, [255, 0xfe, 255]),
            (B::DPadDown, [255, 0xfd, 255]),
            (B::DPadLeft, [255; 3]),
            (B::DPadRight, [255; 3]),
            (B::RightTrigger, [255, 0xef, 255]),
            (B::LeftTrigger, [255, 0xdf, 255]),
            (B::RightThumb, [0xf7, 255, 255]),
            (B::LeftThumb, [0xfb, 255, 255]),
        ] {
            input.set_pad_button(button, true);
            input.poll(&mut out);
            assert_eq!(ports(&out), expected, "{game} {button:?}");
            input.set_pad_button(button, false);
            input.poll(&mut out);
            assert_eq!(ports(&out), [255; 3]);
        }
        input.set_pad_button(B::RightTrigger, true);
        input.set_pad_button(B::LeftTrigger, true);
        input.poll(&mut out);
        assert_eq!(ports(&out), [255; 3], "{game} conflicting shifts");
    }
}

#[test]
fn player_two_furniture_preserves_all_existing_cabinet_routes() {
    use crate::input::players::Player;
    for cabinet in cabinets() {
        for game in cabinet.sets.split_whitespace() {
            for signal in [S::Coin, S::Start, S::Test, S::Service] {
                let mut input = state(game, cabinet.scheme);
                let mut bindings = Bindings::default();
                bindings
                    .set_player_expression(Player::Two, signal, "F12")
                    .unwrap();
                input.set_bindings(bindings);
                input.on_key(KeyCode::F12, true);
                let mut out = Inputs::default();
                input.poll(&mut out);
                let mask = match signal {
                    S::Coin if cabinet.coin2 => 2,
                    S::Start if cabinet.start2 => 0x20,
                    S::Test => {
                        if game == "bel" {
                            8
                        } else {
                            4
                        }
                    }
                    S::Service => {
                        if game == "bel" {
                            4
                        } else {
                            8
                        }
                    }
                    _ => 0,
                };
                let mut expected = cabinet.idle;
                expected[0] ^= mask;
                assert_eq!(ports(&out), expected, "{game}: P2 {signal:?}");
                input.on_key(KeyCode::F12, false);
                input.poll(&mut out);
                assert_eq!(ports(&out), cabinet.idle);
            }
        }
    }
}

#[test]
fn motor_raid_attacks_remain_independent() {
    let mut input = state("motoraid", Scheme::Bike);
    input.set_pad_button(gilrs::Button::East, true);
    input.set_pad_button(gilrs::Button::South, true);
    let mut out = Inputs::default();
    input.poll(&mut out);
    assert_eq!(out.in1, 0xcf);
}

#[test]
fn handbrake_retains_partial_travel_when_rebound_to_an_axis() {
    let mut input = state("srallyc", Scheme::Racing);
    input
        .bindings
        .set_expression(S::Action4, "pad:LeftStickX+")
        .unwrap();
    input.set_pad_stick(0.5, 0.0);
    let mut out = Inputs::default();
    input.poll(&mut out);
    assert_eq!(out.in2, 128);
}

#[test]
fn gun_reload_is_only_for_serial_cabinets_and_calibration_is_per_set() {
    for (game, roles, corner, other) in [
        (
            "gunblade",
            [A::Gun1X, A::Gun2X, A::Gun1Y, A::Gun2Y],
            [0x69, 0x11],
            [0x50, 0x5f],
        ),
        (
            "bel",
            [A::Gun1X, A::Gun2X, A::Gun1Y, A::Gun2Y],
            [0x69, 0x11],
            [0x50, 0x5f],
        ),
        (
            "rchase2",
            [A::Gun2X, A::Gun1X, A::Gun2Y, A::Gun1Y],
            [0xca, 0xcb],
            [0x7d, 0x73],
        ),
        (
            "rchase2a",
            [A::Gun2X, A::Gun1X, A::Gun2Y, A::Gun1Y],
            [0, 0],
            [0x80, 0x80],
        ),
    ] {
        let mut input = state(game, Scheme::Gun);
        let mut channels = [A::None; 8];
        channels[..4].copy_from_slice(&roles);
        input.set_analog_roles(channels);
        input.on_cursor(0.0, 0.0);
        input.mouse_reload = true;
        let mut out = Inputs::default();
        input.poll(&mut out);
        assert!(!out.gun_offscreen, "{game}");
        assert_ne!(out.in1 & 1, 0, "secondary must not fire {game}");
        for (role, value) in [
            (A::Gun1X, corner[0]),
            (A::Gun1Y, corner[1]),
            (A::Gun2X, other[0]),
            (A::Gun2Y, other[1]),
        ] {
            assert_eq!(
                out.analog[roles.iter().position(|r| *r == role).unwrap()],
                value,
                "{game} {role:?}"
            );
        }
    }
    for (game, expected) in [
        ("vcop", (131, 36)),
        ("vcop2", (137, 36)),
        ("hotd", (173, 87)),
        ("hotdo", (173, 87)),
        ("hotdp", (173, 87)),
    ] {
        let mut input = state(game, Scheme::Gun);
        input.on_cursor(0.0, 0.0);
        input.mouse_reload = true;
        let mut out = Inputs::default();
        input.poll(&mut out);
        assert!(out.gun_offscreen);
        assert_eq!(out.in1 & 1, 0);
        assert_eq!((out.gun_x, out.gun_y), expected);
    }
}

#[test]
fn car_and_bike_adc_ranges_polarity_and_mirrors_cover_every_revision() {
    for cabinet in cabinets()
        .into_iter()
        .filter(|c| matches!(c.scheme, Scheme::Racing | Scheme::Bike))
    {
        for game in cabinet.sets.split_whitespace() {
            let legacy = matches!(game, "vr" | "vformula");
            for (signal, role) in [
                (S::Steering, A::Steer),
                (
                    S::Accelerator,
                    if cabinet.scheme == Scheme::Bike {
                        A::Throttle
                    } else {
                        A::Accel
                    },
                ),
                (S::Brake, A::Brake),
            ] {
                if game == "desert" && signal == S::Brake {
                    continue;
                }
                let mut input = state(game, cabinet.scheme);
                let roles = db_roles(game);
                input.set_analog_roles(roles);
                let channel = roles.iter().position(|r| *r == role).unwrap();
                let steer = signal == S::Steering;
                input
                    .bindings
                    .set_expression(signal, if steer { "keys:F11/F12" } else { "F12" })
                    .unwrap();
                let reverse = if steer {
                    cabinet.scheme == Scheme::Bike
                } else {
                    game.starts_with("overrev") || game == "sgt24h"
                };
                let (min, max) = if legacy { (32, 224) } else { (0, 255) };
                for (key, expected) in [
                    (
                        None,
                        if steer {
                            128
                        } else if reverse {
                            max
                        } else {
                            min
                        },
                    ),
                    (Some(KeyCode::F12), if reverse { min } else { max }),
                    (
                        Some(KeyCode::F11),
                        if steer && reverse {
                            max
                        } else if !steer && reverse {
                            max
                        } else {
                            min
                        },
                    ),
                ] {
                    input.keys.clear();
                    if let Some(k) = key {
                        input.on_key(k, true);
                    }
                    let mut out = Inputs::default();
                    for _ in 0..40 {
                        input.poll(&mut out);
                    }
                    assert_eq!(out.analog[channel], expected, "{game} {signal:?} {key:?}");
                    assert_eq!(
                        [out.steer, out.accel, out.brake],
                        out.analog[..3],
                        "original I/O mirrors {game}"
                    );
                }
            }
        }
    }
}

#[test]
fn independent_analog_signals_reach_the_documented_channel_only() {
    // Expected values: negative, centre, positive. Positive Y means UP.
    let specs: &[(&str, Scheme, S, usize, [u8; 3])] = &[
        ("skytargt", Scheme::Flight, S::SkyX, 2, [255, 128, 0]),
        ("skytargt", Scheme::Flight, S::SkyY, 0, [255, 128, 0]),
        ("swa", Scheme::Flight, S::SkyX, 0, [227, 127, 27]),
        ("swaj", Scheme::Flight, S::SkyY, 1, [227, 127, 27]),
        ("wingwar", Scheme::Flight, S::SkyX, 0, [255, 128, 0]),
        ("wingwarj", Scheme::Flight, S::SkyY, 1, [255, 128, 0]),
        ("wingwaru", Scheme::Flight, S::SkyY, 1, [255, 128, 0]),
        ("wingwar360", Scheme::Flight, S::SkyX, 0, [0, 128, 255]),
        ("wingwar360", Scheme::Flight, S::SkyY, 1, [0, 128, 255]),
        ("netmerc", Scheme::Flight, S::SkyX, 0, [0, 127, 255]),
        ("netmerc", Scheme::Flight, S::SkyY, 2, [0, 127, 255]),
        ("desert", Scheme::Racing, S::Elevation, 2, [255, 128, 0]),
        ("segawski", Scheme::Ski, S::WaterSlide, 0, [255, 128, 0]),
        ("topskatr", Scheme::Skate, S::Curving, 0, [255, 128, 0]),
        ("topskatrj", Scheme::Skate, S::Curving, 0, [255, 128, 0]),
        ("topskatru", Scheme::Skate, S::Curving, 0, [255, 128, 0]),
        ("topskatruo", Scheme::Skate, S::Curving, 0, [255, 128, 0]),
        ("topskatr", Scheme::Skate, S::SkaterSlide, 1, [0, 128, 255]),
        // Tested SM2 order overrides the MAME-generated metadata.
        ("skisuprg", Scheme::Ski, S::Swing, 1, [255, 128, 0]),
        ("skisuprg", Scheme::Ski, S::Inclining, 0, [0, 128, 255]),
        ("waverunr", Scheme::Jetski, S::Roll, 1, [0, 128, 255]),
        ("waverunr", Scheme::Jetski, S::Pitch, 3, [0, 128, 255]),
    ];
    for &(game, scheme, signal, channel, values) in specs {
        let mut input = state(game, scheme);
        input.set_analog_roles(db_roles(game));
        for &s in S::ALL {
            input.bindings.set_expression(s, "").unwrap();
        }
        input
            .bindings
            .set_expression(signal, "keys:F11/F12")
            .unwrap();
        let mut idle = Inputs::default();
        input.poll(&mut idle);
        for (key, value) in [
            (Some(KeyCode::F11), values[0]),
            (None, values[1]),
            (Some(KeyCode::F12), values[2]),
        ] {
            input.keys.clear();
            if let Some(k) = key {
                input.on_key(k, true);
            }
            let mut out = Inputs::default();
            for _ in 0..40 {
                input.poll(&mut out);
            }
            let mut expected = idle.analog;
            expected[channel] = value;
            assert_eq!(out.analog, expected, "{game} {signal:?} {key:?}");
            assert_eq!([out.steer, out.accel, out.brake], out.analog[..3]);
        }
    }
}

#[test]
fn gun_stick_moves_and_holds_the_same_cursor_that_is_drawn() {
    for game in [
        "vcop", "vcopa", "vcop2", "hotd", "hotdo", "hotdp", "gunblade", "bel", "rchase2",
        "rchase2a",
    ] {
        let mut input = state(game, Scheme::Gun);
        input.set_pad_stick(0.5, 0.5);
        let mut out = Inputs::default();
        input.poll(&mut out);
        let first = input.aim();
        assert!(first.0 > 0.5 && first.1 < 0.5);
        input.poll(&mut out);
        let moved = input.aim();
        assert!(moved.0 > first.0 && moved.1 < first.1);
        input.set_pad_stick(0.0, 0.0);
        input.poll(&mut out);
        assert_eq!(input.aim(), moved, "{game} release");
        input.on_cursor(0.25, 0.75);
        input.poll(&mut out);
        assert_eq!(input.aim(), (0.25, 0.75), "{game} mouse");
    }
}

#[test]
fn swa_view_uses_view1_defaults_not_action3() {
    for game in ["swa", "swaj"] {
        let mut input = state(game, Scheme::Flight);
        let mut out = Inputs::default();
        for (button, expected) in [
            (gilrs::Button::DPadDown, [255, 0xef, 255]),
            (gilrs::Button::DPadUp, [255, 0xef, 255]),
            (gilrs::Button::West, [255; 3]),
        ] {
            input.set_pad_button(button, true);
            input.poll(&mut out);
            assert_eq!(ports(&out), expected, "{game} {button:?}");
            input.set_pad_button(button, false);
            input.poll(&mut out);
            assert_eq!(ports(&out), [255; 3]);
        }
        for (key, expected) in [(KeyCode::KeyZ, [255, 0xef, 255]), (KeyCode::KeyL, [255; 3])] {
            input.on_key(key, true);
            input.poll(&mut out);
            assert_eq!(ports(&out), expected, "{game} {key:?}");
            input.on_key(key, false);
            input.poll(&mut out);
            assert_eq!(ports(&out), [255; 3]);
        }
    }
}

#[test]
fn single_view_pad_alias_preserves_multiview_and_other_cabinets() {
    use gilrs::Button as B;
    for cabinet in cabinets() {
        for game in cabinet.sets.split_whitespace() {
            let single = game.starts_with("srally")
                || matches!(game, "sgt24h" | "swa" | "swaj");
            let mut input = state(game, cabinet.scheme);
            if !cabinet.buttons.iter().any(|&(s, _, _)| matches!(s, S::View1 | S::View2 | S::View3 | S::View4)) {
                input.set_pad_button(B::DPadUp, true);
                assert_eq!(input.signal(S::View1), 0.0, "{game}: no view alias");
                continue;
            }
            for button in [B::DPadUp, B::DPadDown] {
                let mut baseline = Inputs::default();
                input.poll(&mut baseline);
                let mut expected = ports(&baseline);
                let signal = if button == B::DPadDown || single { S::View1 } else { S::View4 };
                // Other cabinet types can map Up/Down to joystick movement.
                let direction = if button == B::DPadUp { S::Up } else { S::Down };
                for &(s, port, mask) in cabinet.buttons {
                    if s == signal || s == direction { expected[port] &= !mask; }
                }
                input.set_pad_button(button, true);
                let mut out = Inputs::default();
                input.poll(&mut out);
                assert_eq!(ports(&out), expected, "{game}: {button:?}");
                input.set_pad_button(button, false);
            }
            if single {
                input.on_key(KeyCode::KeyV, true);
                let mut out = Inputs::default();
                input.poll(&mut out);
                assert_eq!(ports(&out), cabinet.idle, "{game}: no keyboard alias");
                input.on_key(KeyCode::KeyV, false);
                input.set_pad_button(B::DPadUp, true);
                input.set_pad_button(B::DPadDown, true);
                input.poll(&mut out);
                let mut expected = cabinet.idle;
                for &(s, port, mask) in cabinet.buttons {
                    if s == S::View1 { expected[port] &= !mask; }
                }
                assert_eq!(ports(&out), expected, "{game}: OR, not toggle/cancellation");
            }
        }
    }
}

#[test]
fn swa_throttle_default_keys_cover_both_directions_and_cancel() {
    for game in ["swa", "swaj"] {
        let mut input = state(game, Scheme::Flight);
        input.set_analog_roles(db_roles(game));
        let mut out = Inputs::default();
        for (accelerator, brake, expected) in [
            (false, false, 128),
            (true, false, 228),
            (false, true, 28),
            (true, true, 128),
        ] {
            input.on_key(KeyCode::KeyW, accelerator);
            input.on_key(KeyCode::KeyS, brake);
            input.poll(&mut out);
            assert_eq!(out.analog[2], expected, "{game} throttle");
        }
    }
}

#[test]
fn wave_runner_throttle_keeps_its_own_rest_and_range() {
    for (game, scheme, channel, rest, max) in [("waverunr", Scheme::Jetski, 2, 128, 0)] {
        let mut input = state(game, scheme);
        input.set_analog_roles(db_roles(game));
        input
            .bindings
            .set_expression(S::Accelerator, "F12")
            .unwrap();
        let mut out = Inputs::default();
        input.poll(&mut out);
        assert_eq!(out.analog[channel], rest, "{game} rest");
        input.on_key(KeyCode::F12, true);
        input.poll(&mut out);
        assert_eq!(out.analog[channel], max, "{game} throttle");
    }
}

#[test]
fn flight_throttle_half_axes_are_independent_of_pedals_and_cancel() {
    for (game, min, max, half_up, half_down) in [
        ("swa", 28, 228, 178, 78),
        ("swaj", 28, 228, 178, 78),
        ("wingwar", 1, 255, 192, 65),
        ("wingwaru", 1, 255, 192, 65),
        ("wingwarj", 1, 255, 192, 65),
        ("wingwar360", 1, 255, 192, 65),
    ] {
        let mut input = state(game, Scheme::Flight);
        input.set_analog_roles(db_roles(game));
        let mut out = Inputs::default();
        for (up, down, expected) in [
            (false, false, 128),
            (true, false, max),
            (false, true, min),
            (true, true, 128),
        ] {
            input.on_key(KeyCode::KeyW, up);
            input.on_key(KeyCode::KeyS, down);
            input.poll(&mut out);
            assert_eq!(out.analog[2], expected, "{game} keyboard");
        }
        input.keys.clear();
        // Rebinding the shared pedals must not change the dedicated throttle.
        input
            .bindings
            .set_expression(S::Accelerator, "F11")
            .unwrap();
        input.bindings.set_expression(S::Brake, "F12").unwrap();
        input.on_key(KeyCode::F11, true);
        input.poll(&mut out);
        assert_eq!(out.analog[2], 128, "{game} independent accelerator");
        input.keys.clear();
        input.on_key(KeyCode::F12, true);
        input.poll(&mut out);
        assert_eq!(out.analog[2], 128, "{game} independent brake");
        input.keys.clear();
        // Exercise assignable half-axes with synthetic analog travel, without
        // depending on a connected controller or its native trigger mapping.
        input
            .bindings
            .set_expression(S::ThrottleUp, "pad:LeftStickX+")
            .unwrap();
        input
            .bindings
            .set_expression(S::ThrottleDown, "pad:LeftStickY+")
            .unwrap();
        for (up, down, expected) in [
            (0.0, 0.0, 128),
            (1.0, 0.0, max),
            (0.0, 1.0, min),
            (0.5, 0.0, half_up),
            (0.0, 0.5, half_down),
            (0.5, 0.5, 128),
            (1.0, 1.0, 128),
        ] {
            input.set_pad_stick(up, down);
            input.poll(&mut out);
            assert_eq!(out.analog[2], expected, "{game} analog {up}/{down}");
        }
    }
}
