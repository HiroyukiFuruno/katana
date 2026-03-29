use crate::theme::palettes::*;
use crate::theme::types::{
    CodeColors, PresetColorData, PreviewColors, Rgb, Rgba, SystemColors, ThemeMode,
};

pub(crate) const fn lighten(color: Rgb, amount: u8) -> Rgb {
    Rgb {
        r: color.r.saturating_add(amount),
        g: color.g.saturating_add(amount),
        b: color.b.saturating_add(amount),
    }
}

pub(crate) const fn darken(color: Rgb, amount: u8) -> Rgb {
    Rgb {
        r: color.r.saturating_sub(amount),
        g: color.g.saturating_sub(amount),
        b: color.b.saturating_sub(amount),
    }
}

pub(crate) const fn to_rgba(rgb: Rgb, alpha: u8) -> Rgba {
    Rgba {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        a: alpha,
    }
}

pub(crate) struct ThemePresetBuilder {
    mode: ThemeMode,
    background: Rgb,
    text: Rgb,
    accent: Rgb,
    panel_background: Option<Rgb>,
    code_background: Option<Rgb>,
    text_secondary: Option<Rgb>,
    success: Option<Rgb>,
    warning: Option<Rgb>,
    error: Option<Rgb>,
    border: Option<Rgb>,
    selection: Option<Rgb>,
}

impl ThemePresetBuilder {
    pub(crate) const fn new(mode: ThemeMode, background: Rgb, text: Rgb, accent: Rgb) -> Self {
        Self {
            mode,
            background,
            text,
            accent,
            panel_background: None,
            code_background: None,
            text_secondary: None,
            success: None,
            warning: None,
            error: None,
            border: None,
            selection: None,
        }
    }

    pub(crate) const fn with_panel_bg(mut self, c: Rgb) -> Self {
        self.panel_background = Some(c);
        self
    }
    pub(crate) const fn with_code_bg(mut self, c: Rgb) -> Self {
        self.code_background = Some(c);
        self
    }
    pub(crate) const fn with_text_sec(mut self, c: Rgb) -> Self {
        self.text_secondary = Some(c);
        self
    }
    pub(crate) const fn with_success(mut self, c: Rgb) -> Self {
        self.success = Some(c);
        self
    }
    pub(crate) const fn with_warning(mut self, c: Rgb) -> Self {
        self.warning = Some(c);
        self
    }
    pub(crate) const fn with_error(mut self, c: Rgb) -> Self {
        self.error = Some(c);
        self
    }
    pub(crate) const fn with_border(mut self, c: Rgb) -> Self {
        self.border = Some(c);
        self
    }
    pub(crate) const fn with_selection(mut self, c: Rgb) -> Self {
        self.selection = Some(c);
        self
    }

    pub(crate) const fn build(self) -> PresetColorData {
        let is_dark = matches!(self.mode, ThemeMode::Dark);

        let p_bg = match self.panel_background {
            Some(c) => c,
            None => {
                if is_dark {
                    darken(self.background, DEFAULT_PANEL_BG_DARKEN_DARK)
                } else {
                    darken(self.background, DEFAULT_PANEL_BG_DARKEN_LIGHT)
                }
            }
        };
        let c_bg = match self.code_background {
            Some(c) => c,
            None => {
                if is_dark {
                    lighten(self.background, DEFAULT_CODE_BG_LIGHTEN_DARK)
                } else {
                    darken(self.background, DEFAULT_CODE_BG_DARKEN_LIGHT)
                }
            }
        };
        let t_sec = match self.text_secondary {
            Some(c) => c,
            None => {
                if is_dark {
                    darken(self.text, DEFAULT_TEXT_SECONDARY_DARKEN)
                } else {
                    lighten(self.text, DEFAULT_TEXT_SECONDARY_LIGHTEN)
                }
            }
        };
        let border = match self.border {
            Some(c) => c,
            None => {
                if is_dark {
                    lighten(self.background, DEFAULT_BORDER_LIGHTEN)
                } else {
                    darken(self.background, DEFAULT_BORDER_DARKEN)
                }
            }
        };
        let selection = match self.selection {
            Some(c) => c,
            None => {
                if is_dark {
                    lighten(self.background, DEFAULT_SELECTION_LIGHTEN)
                } else {
                    darken(self.background, DEFAULT_SELECTION_DARKEN)
                }
            }
        };

        let success = match self.success {
            Some(c) => c,
            None => {
                if is_dark {
                    DEFAULT_SUCCESS_DARK
                } else {
                    DEFAULT_SUCCESS_LIGHT
                }
            }
        };
        let warning = match self.warning {
            Some(c) => c,
            None => {
                if is_dark {
                    DEFAULT_WARNING_DARK
                } else {
                    DEFAULT_WARNING_LIGHT
                }
            }
        };
        let error = match self.error {
            Some(c) => c,
            None => {
                if is_dark {
                    DEFAULT_ERROR_DARK
                } else {
                    DEFAULT_ERROR_LIGHT
                }
            }
        };

        PresetColorData {
            mode: self.mode,
            system: SystemColors {
                background: self.background,
                panel_background: p_bg,
                text: self.text,
                text_secondary: t_sec,
                success_text: success,
                warning_text: warning,
                error_text: error,
                accent: self.accent,
                title_bar_text: self.text,
                file_tree_text: t_sec,
                active_file_highlight: to_rgba(self.accent, DEFAULT_ACTIVE_FILE_HIGHLIGHT_ALPHA),
                button_background: to_rgba(p_bg, DEFAULT_BUTTON_BACKGROUND_ALPHA),
                button_active_background: to_rgba(self.accent, DEFAULT_BUTTON_ACTIVE_ALPHA),
                border,
                selection,
            },
            code: CodeColors {
                background: c_bg,
                text: self.text,
                line_number_text: t_sec,
                line_number_active_text: self.text,
                current_line_background: if is_dark {
                    Rgba {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: DEFAULT_CODE_CURRENT_LINE_DARK_ALPHA,
                    }
                } else {
                    Rgba {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: DEFAULT_CODE_CURRENT_LINE_LIGHT_ALPHA,
                    }
                },
                hover_line_background: to_rgba(self.accent, DEFAULT_HOVER_LINE_HIGHLIGHT_ALPHA),
                selection,
            },
            preview: PreviewColors {
                background: self.background,
                text: self.text,
                warning_text: warning,
                border,
                selection,
                hover_line_background: to_rgba(self.accent, DEFAULT_HOVER_LINE_HIGHLIGHT_ALPHA),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighten() {
        let color = Rgb {
            r: 100,
            g: 100,
            b: 100,
        };
        let lightened = lighten(color, 50);
        assert_eq!(lightened.r, 150);
        assert_eq!(lightened.g, 150);
        assert_eq!(lightened.b, 150);

        // Saturating add test
        let color2 = Rgb {
            r: 250,
            g: 250,
            b: 250,
        };
        let lightened2 = lighten(color2, 50);
        assert_eq!(lightened2.r, 255);
        assert_eq!(lightened2.g, 255);
        assert_eq!(lightened2.b, 255);
    }

    #[test]
    fn test_darken() {
        let color = Rgb {
            r: 100,
            g: 100,
            b: 100,
        };
        let darkened = darken(color, 50);
        assert_eq!(darkened.r, 50);
        assert_eq!(darkened.g, 50);
        assert_eq!(darkened.b, 50);

        // Saturating sub test
        let color2 = Rgb {
            r: 10,
            g: 10,
            b: 10,
        };
        let darkened2 = darken(color2, 50);
        assert_eq!(darkened2.r, 0);
        assert_eq!(darkened2.g, 0);
        assert_eq!(darkened2.b, 0);
    }

    #[test]
    fn test_to_rgba() {
        let color = Rgb {
            r: 100,
            g: 150,
            b: 200,
        };
        let rgba = to_rgba(color, 128);
        assert_eq!(rgba.r, 100);
        assert_eq!(rgba.g, 150);
        assert_eq!(rgba.b, 200);
        assert_eq!(rgba.a, 128);
    }

    #[test]
    fn test_builder_dark_defaults() {
        let builder = ThemePresetBuilder::new(
            ThemeMode::Dark,
            Rgb {
                r: 30,
                g: 30,
                b: 30,
            },
            Rgb {
                r: 200,
                g: 200,
                b: 200,
            },
            Rgb {
                r: 100,
                g: 100,
                b: 255,
            },
        );
        let preset = builder.build();
        assert_eq!(preset.mode, ThemeMode::Dark);
        assert_eq!(preset.system.background.r, 30);
        assert_eq!(preset.system.text.r, 200);
        assert_eq!(preset.system.accent.r, 100);
    }

    #[test]
    fn test_builder_light_defaults() {
        let builder = ThemePresetBuilder::new(
            ThemeMode::Light,
            Rgb {
                r: 240,
                g: 240,
                b: 240,
            },
            Rgb {
                r: 10,
                g: 10,
                b: 10,
            },
            Rgb {
                r: 50,
                g: 50,
                b: 200,
            },
        );
        let preset = builder.build();
        assert_eq!(preset.mode, ThemeMode::Light);
        assert_eq!(preset.system.background.r, 240);
        assert_eq!(preset.system.text.r, 10);
        assert_eq!(preset.system.accent.r, 50);
    }

    #[test]
    fn test_builder_custom_colors() {
        let preset = ThemePresetBuilder::new(
            ThemeMode::Dark,
            Rgb {
                r: 30,
                g: 30,
                b: 30,
            },
            Rgb {
                r: 200,
                g: 200,
                b: 200,
            },
            Rgb {
                r: 100,
                g: 100,
                b: 255,
            },
        )
        .with_panel_bg(Rgb {
            r: 40,
            g: 40,
            b: 40,
        })
        .with_code_bg(Rgb {
            r: 20,
            g: 20,
            b: 20,
        })
        .with_text_sec(Rgb {
            r: 150,
            g: 150,
            b: 150,
        })
        .with_success(Rgb {
            r: 10,
            g: 200,
            b: 10,
        })
        .with_warning(Rgb {
            r: 200,
            g: 200,
            b: 10,
        })
        .with_error(Rgb {
            r: 200,
            g: 10,
            b: 10,
        })
        .with_border(Rgb {
            r: 80,
            g: 80,
            b: 80,
        })
        .with_selection(Rgb {
            r: 60,
            g: 60,
            b: 100,
        })
        .build();

        assert_eq!(preset.system.panel_background.r, 40);
        assert_eq!(preset.code.background.r, 20);
        assert_eq!(preset.system.text_secondary.r, 150);
        assert_eq!(preset.system.success_text.r, 10);
        assert_eq!(preset.system.warning_text.r, 200);
        assert_eq!(preset.system.error_text.r, 200);
        assert_eq!(preset.system.border.r, 80);
        assert_eq!(preset.system.selection.r, 60);
    }
}
