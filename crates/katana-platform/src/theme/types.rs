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

#[test]
fn test_preview_hover_line_background_default() {
    let hover = default_hover_line_background();
    assert_eq!(hover.r, DEFAULT_PREVIEW_LINE_BACKGROUND_RGB);
    assert_eq!(hover.a, DEFAULT_PREVIEW_HOVER_LINE_BACKGROUND_ALPHA);
}
