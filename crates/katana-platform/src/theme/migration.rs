const ERROR_R: u8 = 0xFF;
const ERROR_G: u8 = 0x55;
const ERROR_B: u8 = 0x55;

use crate::theme::types::*;
use serde::{Deserialize, Deserializer};

// ── Migration fallback color constants ──────────────────────────
// レガシーフォーマットから新3層構造への変換時に使用するデフォルト値。
// "is_dark" 分岐で使い分けるため dark/light のペアで定義する。

const LEGACY_SUCCESS_DARK: Rgb = Rgb {
    r: 80,
    g: 200,
    b: 80,
};
const LEGACY_SUCCESS_LIGHT: Rgb = Rgb {
    r: 20,
    g: 160,
    b: 20,
};
const LEGACY_BUTTON_BG_DARK: Rgba = Rgba {
    r: 80,
    g: 80,
    b: 80,
    a: 50,
};
const LEGACY_BUTTON_BG_LIGHT: Rgba = Rgba {
    r: 160,
    g: 160,
    b: 160,
    a: 50,
};
const LEGACY_BUTTON_ACTIVE_DARK: Rgba = Rgba {
    r: 120,
    g: 120,
    b: 120,
    a: 100,
};
const LEGACY_BUTTON_ACTIVE_LIGHT: Rgba = Rgba {
    r: 200,
    g: 200,
    b: 200,
    a: 100,
};
const LEGACY_SPLASH_BG_DARK: Rgb = Rgb {
    r: 25,
    g: 25,
    b: 25,
};
const LEGACY_SPLASH_BG_LIGHT: Rgb = Rgb {
    r: 240,
    g: 240,
    b: 240,
};
const LEGACY_SPLASH_PROGRESS_DARK: Rgb = Rgb {
    r: 80,
    g: 156,
    b: 214,
};
const LEGACY_SPLASH_PROGRESS_LIGHT: Rgb = Rgb {
    r: 0,
    g: 120,
    b: 212,
};
const LEGACY_LINE_NUMBER_DARK: Rgb = Rgb {
    r: 100,
    g: 100,
    b: 100,
};
const LEGACY_LINE_NUMBER_LIGHT: Rgb = Rgb {
    r: 160,
    g: 160,
    b: 160,
};
const LEGACY_CURRENT_LINE_DARK: Rgba = Rgba {
    r: 255,
    g: 255,
    b: 255,
    a: 15,
};
const LEGACY_CURRENT_LINE_LIGHT: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 15,
};
const LEGACY_HOVER_LINE_DARK: Rgba = Rgba {
    r: 255,
    g: 255,
    b: 255,
    a: 10,
};
const LEGACY_HOVER_LINE_LIGHT: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 10,
};
const LEGACY_FULLSCREEN_OVERLAY_DARK: Rgba = Rgba {
    r: 40,
    g: 40,
    b: 40,
    a: 200,
};
const LEGACY_FULLSCREEN_OVERLAY_LIGHT: Rgba = Rgba {
    r: 200,
    g: 200,
    b: 200,
    a: 200,
};

const LEGACY_DEFAULT_WARNING: Rgb = Rgb {
    r: 255,
    g: 140,
    b: 0,
};
const LEGACY_DEFAULT_BORDER: Rgb = Rgb {
    r: 60,
    g: 60,
    b: 60,
};
const LEGACY_DEFAULT_SELECTION: Rgb = Rgb {
    r: 80,
    g: 80,
    b: 80,
};
const LEGACY_DEFAULT_CODE_BG: Rgb = Rgb {
    r: 30,
    g: 30,
    b: 30,
};
const LEGACY_DEFAULT_PREVIEW_BG: Rgb = Rgb {
    r: 35,
    g: 35,
    b: 35,
};

// ── Deserialize implementation ──────────────────────────────────

impl<'de> Deserialize<'de> for ThemeColors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_theme_colors(deserializer)
    }
}

#[derive(Deserialize)]
#[serde(remote = "ThemeColors")]
struct ThemeColorsDef {
    pub name: String,
    pub mode: ThemeMode,
    pub system: SystemColors,
    pub code: CodeColors,
    pub preview: PreviewColors,
}

#[derive(Deserialize)]
struct ThemeColorsLegacyData {
    pub name: String,
    pub mode: ThemeMode,
    pub background: Rgb,
    pub panel_background: Rgb,
    pub text: Rgb,
    pub text_secondary: Rgb,
    pub accent: Rgb,
    pub title_bar_text: Rgb,
    pub file_tree_text: Rgb,
    pub active_file_highlight: Rgba,
    #[serde(default = "default_warning_text")]
    pub warning_text: Rgb,
    #[serde(default = "default_error_text")]
    pub error_text: Rgb,
    #[serde(default = "default_border")]
    pub border: Rgb,
    #[serde(default = "default_selection")]
    pub selection: Rgb,
    #[serde(default = "default_code_background")]
    pub code_background: Rgb,
    #[serde(default = "default_preview_background")]
    pub preview_background: Rgb,
}

fn default_error_text() -> Rgb {
    Rgb {
        r: ERROR_R,
        g: ERROR_G,
        b: ERROR_B,
    }
}

fn default_warning_text() -> Rgb {
    LEGACY_DEFAULT_WARNING
}
fn default_border() -> Rgb {
    LEGACY_DEFAULT_BORDER
}
fn default_selection() -> Rgb {
    LEGACY_DEFAULT_SELECTION
}
fn default_code_background() -> Rgb {
    LEGACY_DEFAULT_CODE_BG
}
fn default_preview_background() -> Rgb {
    LEGACY_DEFAULT_PREVIEW_BG
}

pub(crate) fn deserialize_theme_colors<'de, D>(deserializer: D) -> Result<ThemeColors, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Migrator {
        New(#[serde(with = "ThemeColorsDef")] ThemeColors),
        Legacy(ThemeColorsLegacyData),
    }

    match Migrator::deserialize(deserializer)? {
        Migrator::New(c) => Ok(c),
        Migrator::Legacy(l) => {
            let is_dark = l.mode == ThemeMode::Dark;
            Ok(ThemeColors {
                name: l.name,
                mode: l.mode,
                system: SystemColors {
                    background: l.background,
                    panel_background: l.panel_background,
                    text: l.text,
                    text_secondary: l.text_secondary,
                    success_text: if is_dark {
                        LEGACY_SUCCESS_DARK
                    } else {
                        LEGACY_SUCCESS_LIGHT
                    },
                    warning_text: l.warning_text,
                    error_text: l.error_text,
                    accent: l.accent,
                    title_bar_text: l.title_bar_text,
                    file_tree_text: l.file_tree_text,
                    active_file_highlight: l.active_file_highlight,
                    button_background: if is_dark {
                        LEGACY_BUTTON_BG_DARK
                    } else {
                        LEGACY_BUTTON_BG_LIGHT
                    },
                    button_active_background: if is_dark {
                        LEGACY_BUTTON_ACTIVE_DARK
                    } else {
                        LEGACY_BUTTON_ACTIVE_LIGHT
                    },
                    border: l.border,
                    selection: l.selection,
                    splash_background: if is_dark {
                        LEGACY_SPLASH_BG_DARK
                    } else {
                        LEGACY_SPLASH_BG_LIGHT
                    },
                    splash_progress: if is_dark {
                        LEGACY_SPLASH_PROGRESS_DARK
                    } else {
                        LEGACY_SPLASH_PROGRESS_LIGHT
                    },
                },
                code: CodeColors {
                    background: l.code_background,
                    text: l.text,
                    line_number_text: if is_dark {
                        LEGACY_LINE_NUMBER_DARK
                    } else {
                        LEGACY_LINE_NUMBER_LIGHT
                    },
                    line_number_active_text: l.text,
                    current_line_background: if is_dark {
                        LEGACY_CURRENT_LINE_DARK
                    } else {
                        LEGACY_CURRENT_LINE_LIGHT
                    },
                    hover_line_background: if is_dark {
                        LEGACY_HOVER_LINE_DARK
                    } else {
                        LEGACY_HOVER_LINE_LIGHT
                    },
                    selection: l.selection,
                },
                preview: PreviewColors {
                    background: l.preview_background,
                    text: l.text,
                    warning_text: l.warning_text,
                    border: l.border,
                    selection: l.selection,
                    fullscreen_overlay: if is_dark {
                        LEGACY_FULLSCREEN_OVERLAY_DARK
                    } else {
                        LEGACY_FULLSCREEN_OVERLAY_LIGHT
                    },
                    hover_line_background: if is_dark {
                        LEGACY_HOVER_LINE_DARK
                    } else {
                        LEGACY_HOVER_LINE_LIGHT
                    },
                },
            })
        }
    }
}
