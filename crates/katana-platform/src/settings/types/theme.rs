use crate::theme::{ThemeColors, ThemePreset};
use serde::{Deserialize, Serialize};

pub const MAX_CUSTOM_THEMES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomTheme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    #[serde(default = "super::super::defaults::default_theme")]
    pub theme: String,
    #[serde(default = "super::super::defaults::default_ui_contrast_offset")]
    pub ui_contrast_offset: f32,
    #[serde(default)]
    pub preset: ThemePreset,
    #[serde(default)]
    pub custom_color_overrides: Option<ThemeColors>,
    #[serde(default)]
    pub custom_themes: Vec<CustomTheme>,
    #[serde(default)]
    pub active_custom_theme: Option<String>,
}
