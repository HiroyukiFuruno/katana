//! `AppSettings` のメソッド実装。

use crate::theme::ThemeColors;

use super::types::*;

impl AppSettings {
    /// Returns the effective theme colours.
    ///
    /// If the user has custom overrides, those are returned;
    /// otherwise the selected preset's palette is used.
    pub fn effective_theme_colors(&self) -> ThemeColors {
        self.theme
            .custom_color_overrides
            .clone()
            .unwrap_or_else(|| self.theme.preset.colors())
    }

    /// Returns the theme colours adjusted for rendering.
    ///
    /// This applies the `ui_contrast_offset` to all RGBa layers. Use this method
    /// for UI generation, while `effective_theme_colors` should strictly be used
    /// for data manipulation and the Settings Window to avoid corrupting user overrides.
    pub fn rendered_theme_colors(&self) -> ThemeColors {
        self.effective_theme_colors()
            .with_contrast_offset(self.theme.ui_contrast_offset)
    }

    /// Sets font size, clamping to the allowed range [`MIN_FONT_SIZE`, `MAX_FONT_SIZE`].
    pub fn set_font_size(&mut self, size: f32) {
        self.font.size = size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
    }

    /// Returns the font size clamped to [`MIN_FONT_SIZE`, `MAX_FONT_SIZE`].
    ///
    /// Useful after deserialization where the raw value may be out of range.
    pub fn clamped_font_size(&self) -> f32 {
        self.font.size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE)
    }
}
