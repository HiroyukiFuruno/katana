use serde::{Deserialize, Serialize};

/// Whether a theme is visually dark or light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// System-wide / general UI colours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemColors {
    pub background: Rgb,
    pub panel_background: Rgb,
    pub text: Rgb,
    pub text_secondary: Rgb,
    /// Text color for success messages
    pub success_text: Rgb,
    /// Text color for warning messages
    pub warning_text: Rgb,
    /// Text color for error messages
    pub error_text: Rgb,
    pub accent: Rgb,
    pub title_bar_text: Rgb,
    pub file_tree_text: Rgb,
    pub active_file_highlight: Rgba,
    pub button_background: Rgba,
    pub button_active_background: Rgba,
    pub border: Rgb,
    pub selection: Rgb,
    pub splash_background: Rgb,
    pub splash_progress: Rgb,
}

/// Colours specific to code blocks and editors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeColors {
    pub background: Rgb,
    pub text: Rgb,
    pub line_number_text: Rgb,
    pub line_number_active_text: Rgb,
    pub current_line_background: Rgba,
    pub hover_line_background: Rgba,
    pub selection: Rgb,
}

/// Colours specific to the markdown preview.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewColors {
    pub background: Rgb,
    pub text: Rgb,
    pub warning_text: Rgb,
    pub border: Rgb,
    pub selection: Rgb,
    pub fullscreen_overlay: Rgba,
    #[serde(default = "default_hover_line_background")]
    pub hover_line_background: Rgba,
}

const DEFAULT_PREVIEW_LINE_BACKGROUND_RGB: u8 = 128;
const DEFAULT_PREVIEW_HOVER_LINE_BACKGROUND_ALPHA: u8 = 15;

fn default_hover_line_background() -> Rgba {
    Rgba {
        r: DEFAULT_PREVIEW_LINE_BACKGROUND_RGB,
        g: DEFAULT_PREVIEW_LINE_BACKGROUND_RGB,
        b: DEFAULT_PREVIEW_LINE_BACKGROUND_RGB,
        a: DEFAULT_PREVIEW_HOVER_LINE_BACKGROUND_ALPHA,
    }
}

/// Complete set of UI colours for the application.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThemeColors {
    /// Human-readable name (e.g. "KatanA-Dark").
    pub name: String,
    /// Whether this palette is dark or light.
    pub mode: ThemeMode,
    /// System-wide colours (panels, sidebars, borders, etc).
    pub system: SystemColors,
    /// Code block and syntax colours.
    pub code: CodeColors,
    /// Preview pane colours.
    pub preview: PreviewColors,
}

/// Built-in theme presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreset {
    #[default]
    KatanaDark,
    Dracula,
    GitHubDark,
    Nord,
    Monokai,
    OneDark,
    TokyoNight,
    CatppuccinMocha,
    MaterialDark,
    NightOwl,
    RosePine,
    Palenight,
    SynthWave84,
    Andromeda,
    OceanicNext,
    KatanaLight,
    GitHubLight,
    SolarizedLight,
    AyuLight,
    GruvboxLight,
    OneLight,
    RosePineDawn,
    CatppuccinLatte,
    MaterialLight,
    QuietLight,
    PaperColorLight,
    MinimalLight,
    Alabaster,
    FlatUILight,
    EverforestLight,
}

pub(crate) struct PresetColorData {
    pub mode: ThemeMode,
    pub system: SystemColors,
    pub code: CodeColors,
    pub preview: PreviewColors,
}

const OFFSET_ZERO: f32 = 0.0;
const OFFSET_DENOMINATOR: f32 = 100.0;
const ALPHA_MAX_F32: f32 = 255.0;
const ALPHA_MIN: f32 = 0.0;

impl Rgba {
    #[must_use]
    pub fn with_offset(mut self, offset_percent: f32) -> Self {
        if offset_percent == OFFSET_ZERO {
            return self;
        }
        let offset_val = ALPHA_MAX_F32 * (offset_percent / OFFSET_DENOMINATOR);
        let new_a = f32::from(self.a) + offset_val;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        {
            self.a = new_a.clamp(ALPHA_MIN, ALPHA_MAX_F32) as u8;
        }
        self
    }
}

impl ThemeColors {
    #[must_use]
    pub fn with_contrast_offset(mut self, offset_percent: f32) -> Self {
        self.code.current_line_background = self
            .code
            .current_line_background
            .with_offset(offset_percent);
        self.code.hover_line_background =
            self.code.hover_line_background.with_offset(offset_percent);

        self.preview.fullscreen_overlay =
            self.preview.fullscreen_overlay.with_offset(offset_percent);
        self.preview.hover_line_background = self
            .preview
            .hover_line_background
            .with_offset(offset_percent);

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_with_offset() {
        let same = Rgba {
            r: 255,
            g: 255,
            b: 255,
            a: 128,
        };
        assert_eq!(same.with_offset(0.0), same);
        assert_eq!(same.with_offset(100.0).a, 255);
        assert_eq!(same.with_offset(-100.0).a, 0);
    }
}

#[test]
fn test_preview_hover_line_background_default() {
    let hover = default_hover_line_background();
    assert_eq!(hover.r, DEFAULT_PREVIEW_LINE_BACKGROUND_RGB);
    assert_eq!(hover.a, DEFAULT_PREVIEW_HOVER_LINE_BACKGROUND_ALPHA);
}
