use crate::theme::migration::constants::*;
use crate::theme::types::{CodeColors, PreviewColors, Rgb, Rgba, SystemColors, ThemeMode};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(remote = "crate::theme::types::ThemeColors")]
pub(crate) struct ThemeColorsDef {
    pub name: String,
    pub mode: ThemeMode,
    pub system: SystemColors,
    pub code: CodeColors,
    pub preview: PreviewColors,
}

#[derive(Deserialize)]
pub(crate) struct ThemeColorsLegacyData {
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
