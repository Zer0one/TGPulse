//! Desktop presentation only; the emulated board owns the LCD character state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Off,
    Overlay,
    Window,
}

impl Mode {
    pub fn key(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Window => "window",
            Self::Overlay => "overlay",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Window => "Dedicated Window",
            Self::Overlay => "Overlay",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        [Self::Off, Self::Overlay, Self::Window]
            .into_iter()
            .find(|m| m.key() == value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rendering {
    Hd44780,
    Text,
}
impl Rendering {
    pub fn key(self) -> &'static str {
        match self {
            Self::Hd44780 => "hd44780",
            Self::Text => "text",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Hd44780 => "HD44780",
            Self::Text => "Text",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        [Self::Hd44780, Self::Text]
            .into_iter()
            .find(|mode| mode.key() == value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
    pub rendering: Rendering,
    /// Clockwise: top left, top right, bottom right, bottom left.
    pub corner: u32,
    pub opacity: u32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            mode: Mode::Off,
            rendering: Rendering::Hd44780,
            corner: 1,
            opacity: 80,
        }
    }
}
impl Options {
    pub fn overlay(self, fullscreen: bool) -> bool {
        self.mode == Mode::Overlay || self.mode == Mode::Window && fullscreen
    }
}
pub const CORNERS: [&str; 4] = ["Top Left", "Top Right", "Bottom Right", "Bottom Left"];
pub const CORNER_KEYS: [&str; 4] = ["top_left", "top_right", "bottom_right", "bottom_left"];

pub fn controls(ui: &imgui::Ui, options: &mut Options) -> bool {
    let mut changed = false;
    if let Some(_combo) = ui.begin_combo("Diagnostic Display", options.mode.label()) {
        for mode in [Mode::Off, Mode::Overlay, Mode::Window] {
            if ui
                .selectable_config(mode.label())
                .selected(options.mode == mode)
                .build()
            {
                changed |= options.mode != mode;
                options.mode = mode;
            }
        }
    }
    let _disabled = ui.begin_disabled(options.mode == Mode::Off);
    if let Some(_combo) = ui.begin_combo("Diagnostic Rendering", options.rendering.label()) {
        for rendering in [Rendering::Hd44780, Rendering::Text] {
            if ui
                .selectable_config(rendering.label())
                .selected(options.rendering == rendering)
                .build()
            {
                changed |= options.rendering != rendering;
                options.rendering = rendering;
            }
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("HD44780 uses hd44780_a00.bin in the game ZIP or adjacent hd44780.zip; Text is the fallback if missing.");
    }
    if let Some(_combo) = ui.begin_combo("Overlay Position", CORNERS[options.corner as usize]) {
        for (index, label) in CORNERS.into_iter().enumerate() {
            if ui
                .selectable_config(label)
                .selected(options.corner == index as u32)
                .build()
            {
                changed |= options.corner != index as u32;
                options.corner = index as u32;
            }
        }
    }
    changed |= ui.slider(
        "Overlay Background Opacity (%)",
        0,
        100,
        &mut options.opacity,
    );
    changed
}

/// Foreground primitives never capture input and remain visible with F1/fullscreen.
/// HD44780 pixels or fixed text cells; both retain spaces/column alignment.
pub fn draw(
    ui: &imgui::Ui,
    lines: [[u8; 20]; 2],
    pixels: Option<tgpulse_core::model1io2::DiagnosticPixels>,
    options: Options,
    dedicated: bool,
) {
    let font = ui.current_font_size();
    let cell = font * 0.75;
    let pad = font;
    let viewport = ui.io().display_size;
    let pixels = pixels.filter(|_| options.rendering == Rendering::Hd44780);
    let desired_scale = (font / 5.0).round().max(1.0);
    let scale = if dedicated {
        desired_scale
            .min(((viewport[0] - 2.0 * pad) / 121.0).floor().max(1.0))
            .min(((viewport[1] - 2.0 * pad) / 19.0).floor().max(1.0))
    } else {
        desired_scale
    };
    let size = if pixels.is_some() {
        [121.0 * scale + 2.0 * pad, 19.0 * scale + 2.0 * pad]
    } else {
        [20.0 * cell + 2.0 * pad, 2.0 * font * 1.4 + 2.0 * pad]
    };
    let margin = font * 1.5;
    let pos = if dedicated {
        [
            (viewport[0] - size[0]).max(0.0) * 0.5,
            (viewport[1] - size[1]).max(0.0) * 0.5,
        ]
    } else {
        [
            if matches!(options.corner, 1 | 2) {
                (viewport[0] - size[0] - margin).max(0.0)
            } else {
                margin
            },
            if options.corner >= 2 {
                (viewport[1] - size[1] - margin).max(0.0)
            } else {
                margin
            },
        ]
    };
    let draw = ui.get_foreground_draw_list();
    draw.add_rect(
        pos,
        [pos[0] + size[0], pos[1] + size[1]],
        [
            0.025,
            0.06,
            0.04,
            if pixels.is_some() {
                0.0 // LCD pens below supply the background once, without double alpha.
            } else if dedicated {
                1.0
            } else {
                options.opacity as f32 / 100.0
            },
        ],
    )
    .filled(true)
    .rounding(5.0)
    .build();
    if let Some(pixels) = pixels {
        // Same palette as MAME's Model 1 I/O board LCD. No smoothed font or texture.
        let alpha = if dedicated {
            1.0
        } else {
            options.opacity as f32 / 100.0
        };
        let colors = [
            [138.0 / 255.0, 146.0 / 255.0, 148.0 / 255.0, alpha],
            [92.0 / 255.0, 83.0 / 255.0, 88.0 / 255.0, 1.0],
            [131.0 / 255.0, 136.0 / 255.0, 139.0 / 255.0, alpha],
        ];
        for (y, row) in pixels.into_iter().enumerate() {
            for (x, pen) in row.into_iter().enumerate() {
                let dot = [
                    pos[0] + pad + x as f32 * scale,
                    pos[1] + pad + y as f32 * scale,
                ];
                draw.add_rect(dot, [dot[0] + scale, dot[1] + scale], colors[pen as usize])
                    .filled(true)
                    .build();
            }
        }
        return;
    }
    for (row, line) in lines.into_iter().enumerate() {
        for (column, byte) in line.into_iter().enumerate() {
            let glyph = if (32..=126).contains(&byte) {
                byte as char
            } else {
                '·'
            };
            draw.add_text(
                [
                    pos[0] + pad + column as f32 * cell,
                    pos[1] + pad + row as f32 * font * 1.4,
                ],
                [0.65, 0.95, 0.7, 1.0],
                glyph.to_string(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fullscreen_uses_overlay_without_changing_the_saved_mode() {
        let options = Options {
            mode: Mode::Window,
            ..Default::default()
        };
        assert!(!options.overlay(false));
        assert!(options.overlay(true));
        assert_eq!(options.mode, Mode::Window);
        assert!(!Options::default().overlay(true));
        let _lock = super::super::tests::IMGUI_TEST_LOCK.lock().unwrap();
        {
            let mut gui = super::super::Gui::new(
                &tgpulse_core::config::Config::default(),
                Default::default(),
            );
            gui.context.io_mut().want_capture_keyboard = true;
            let mut auxiliary = Some(imgui::SuspendedContext::create());
            gui.with_diagnostic_context(&mut auxiliary, |context| {
                context.set_ini_filename(None);
                context.io_mut().display_size = [360.0, 130.0];
                context.fonts().build_rgba32_texture();
                draw(
                    context.frame(),
                    [[b' '; 20]; 2],
                    Some([[2; 121]; 19]),
                    options,
                    true,
                );
                assert!(context.render().total_vtx_count > 0);
            });
            assert!(gui.context.io().want_capture_keyboard);
            assert_eq!(gui.context.io().display_size, [992.0, 768.0]);
            drop(auxiliary);
            // Closing the auxiliary must leave the main context usable.
            gui.context.fonts().build_rgba32_texture();
            let _ = gui.context.frame();
            gui.context.render();
        }
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        context.io_mut().display_size = [800.0, 600.0];
        context.fonts().build_rgba32_texture();
        for corner in 0..4 {
            let ui = context.frame();
            draw(
                ui,
                [*b"ABCDEFGHIJKLMNOPQRST", *b"  Diagnostic LCD    "],
                None,
                Options { corner, ..options },
                false,
            );
            assert!(context.render().total_vtx_count > 0);
        }
    }
}
