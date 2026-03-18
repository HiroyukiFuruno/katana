//! Theme color definitions and presets.
//!
//! `ThemeColors` aggregates all UI colors. Ten presets (5 dark, 5 light) ship
//! out of the box, and users can customise individual colors on top of any
//! preset.
//!
//! # Architectural Note
//!
//! `Rgb` / `Rgba` are general-purpose colour value types that live here for
//! simplicity.  Should the number of crates grow (e.g. a separate renderer),
//! consider extracting them into a dedicated `katana-types` crate so that
//! every layer can depend on shared value objects without pulling in the
//! full platform crate.

use serde::{Deserialize, Serialize};

/// Whether a theme is visually dark or light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
}

/// Opaque RGB colour value (0–255 per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Opaque RGBA colour value with premultiplied alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Complete set of UI colours for the application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Human-readable name (e.g. "KatanA-Dark").
    pub name: String,
    /// Whether this palette is dark or light.
    pub mode: ThemeMode,
    /// Primary background colour (main panes).
    pub background: Rgb,
    /// Secondary background colour (panels, sidebars).
    pub panel_background: Rgb,
    /// Primary text colour.
    pub text: Rgb,
    /// Subdued text colour (secondary labels, hints).
    pub text_secondary: Rgb,
    /// Accent / highlight colour (buttons, links, active items).
    pub accent: Rgb,
    /// Title bar text colour.
    pub title_bar_text: Rgb,
    /// File tree normal text colour.
    pub file_tree_text: Rgb,
    /// Active file highlight background (semi-transparent).
    pub active_file_highlight: Rgba,
    /// Warning / alert text colour.
    pub warning_text: Rgb,
    /// Border / separator colour.
    pub border: Rgb,
    /// Selection background colour.
    pub selection: Rgb,
    /// Code block background colour.
    pub code_background: Rgb,
    /// Preview pane background colour.
    pub preview_background: Rgb,
}

impl ThemeColors {
    /// Returns the syntect theme name fitting this palette's mode.
    pub fn syntax_theme_name(&self) -> &str {
        match self.mode {
            ThemeMode::Dark => "base16-ocean.dark",
            ThemeMode::Light => "base16-ocean.light",
        }
    }
}

/// Identifies a built-in preset.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    #[default]
    KatanaDark,
    Dracula,
    GitHubDark,
    Nord,
    Monokai,
    KatanaLight,
    GitHubLight,
    SolarizedLight,
    AyuLight,
    GruvboxLight,
}

impl ThemePreset {
    /// All available presets, ordered dark-first.
    pub fn all() -> &'static [ThemePreset] {
        &[
            Self::KatanaDark,
            Self::Dracula,
            Self::GitHubDark,
            Self::Nord,
            Self::Monokai,
            Self::KatanaLight,
            Self::GitHubLight,
            Self::SolarizedLight,
            Self::AyuLight,
            Self::GruvboxLight,
        ]
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &str {
        match self {
            Self::KatanaDark => "KatanA Dark",
            Self::Dracula => "Dracula",
            Self::GitHubDark => "GitHub Dark",
            Self::Nord => "Nord",
            Self::Monokai => "Monokai",
            Self::KatanaLight => "KatanA Light",
            Self::GitHubLight => "GitHub Light",
            Self::SolarizedLight => "Solarized Light",
            Self::AyuLight => "Ayu Light",
            Self::GruvboxLight => "Gruvbox Light",
        }
    }

    /// Build the full `ThemeColors` palette for this preset.
    pub fn colors(&self) -> ThemeColors {
        let data = match self {
            Self::KatanaDark => &KATANA_DARK,
            Self::Dracula => &DRACULA,
            Self::GitHubDark => &GITHUB_DARK,
            Self::Nord => &NORD,
            Self::Monokai => &MONOKAI,
            Self::KatanaLight => &KATANA_LIGHT,
            Self::GitHubLight => &GITHUB_LIGHT,
            Self::SolarizedLight => &SOLARIZED_LIGHT,
            Self::AyuLight => &AYU_LIGHT,
            Self::GruvboxLight => &GRUVBOX_LIGHT,
        };
        data.to_theme_colors(self.display_name())
    }
}

// ── Const-friendly color data ──

/// Raw colour data without `String` so it can be `const`.
struct PresetColorData {
    mode: ThemeMode,
    background: Rgb,
    panel_background: Rgb,
    text: Rgb,
    text_secondary: Rgb,
    accent: Rgb,
    title_bar_text: Rgb,
    file_tree_text: Rgb,
    active_file_highlight: Rgba,
    warning_text: Rgb,
    border: Rgb,
    selection: Rgb,
    code_background: Rgb,
    preview_background: Rgb,
}

impl PresetColorData {
    fn to_theme_colors(&self, name: &str) -> ThemeColors {
        ThemeColors {
            name: name.to_string(),
            mode: self.mode,
            background: self.background,
            panel_background: self.panel_background,
            text: self.text,
            text_secondary: self.text_secondary,
            accent: self.accent,
            title_bar_text: self.title_bar_text,
            file_tree_text: self.file_tree_text,
            active_file_highlight: self.active_file_highlight,
            warning_text: self.warning_text,
            border: self.border,
            selection: self.selection,
            code_background: self.code_background,
            preview_background: self.preview_background,
        }
    }
}

// ── Dark presets (const) ──

const KATANA_DARK: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    background: Rgb {
        r: 30,
        g: 30,
        b: 30,
    },
    panel_background: Rgb {
        r: 37,
        g: 37,
        b: 38,
    },
    text: Rgb {
        r: 212,
        g: 212,
        b: 212,
    },
    text_secondary: Rgb {
        r: 180,
        g: 180,
        b: 180,
    },
    accent: Rgb {
        r: 86,
        g: 156,
        b: 214,
    },
    title_bar_text: Rgb {
        r: 180,
        g: 180,
        b: 180,
    },
    file_tree_text: Rgb {
        r: 220,
        g: 220,
        b: 220,
    },
    active_file_highlight: Rgba {
        r: 40,
        g: 80,
        b: 160,
        a: 100,
    },
    warning_text: Rgb {
        r: 255,
        g: 165,
        b: 0,
    },
    border: Rgb {
        r: 60,
        g: 60,
        b: 60,
    },
    selection: Rgb {
        r: 38,
        g: 79,
        b: 120,
    },
    code_background: Rgb {
        r: 40,
        g: 40,
        b: 40,
    },
    preview_background: Rgb {
        r: 30,
        g: 30,
        b: 30,
    },
};

const DRACULA: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    background: Rgb {
        r: 40,
        g: 42,
        b: 54,
    },
    panel_background: Rgb {
        r: 44,
        g: 44,
        b: 58,
    },
    text: Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    text_secondary: Rgb {
        r: 189,
        g: 147,
        b: 249,
    },
    accent: Rgb {
        r: 139,
        g: 233,
        b: 253,
    },
    title_bar_text: Rgb {
        r: 189,
        g: 147,
        b: 249,
    },
    file_tree_text: Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    active_file_highlight: Rgba {
        r: 68,
        g: 71,
        b: 90,
        a: 120,
    },
    warning_text: Rgb {
        r: 255,
        g: 184,
        b: 108,
    },
    border: Rgb {
        r: 68,
        g: 71,
        b: 90,
    },
    selection: Rgb {
        r: 68,
        g: 71,
        b: 90,
    },
    code_background: Rgb {
        r: 50,
        g: 52,
        b: 66,
    },
    preview_background: Rgb {
        r: 40,
        g: 42,
        b: 54,
    },
};

const GITHUB_DARK: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    background: Rgb {
        r: 13,
        g: 17,
        b: 23,
    },
    panel_background: Rgb {
        r: 22,
        g: 27,
        b: 34,
    },
    text: Rgb {
        r: 201,
        g: 209,
        b: 217,
    },
    text_secondary: Rgb {
        r: 139,
        g: 148,
        b: 158,
    },
    accent: Rgb {
        r: 88,
        g: 166,
        b: 255,
    },
    title_bar_text: Rgb {
        r: 139,
        g: 148,
        b: 158,
    },
    file_tree_text: Rgb {
        r: 201,
        g: 209,
        b: 217,
    },
    active_file_highlight: Rgba {
        r: 33,
        g: 38,
        b: 45,
        a: 130,
    },
    warning_text: Rgb {
        r: 210,
        g: 153,
        b: 34,
    },
    border: Rgb {
        r: 48,
        g: 54,
        b: 61,
    },
    selection: Rgb {
        r: 23,
        g: 74,
        b: 130,
    },
    code_background: Rgb {
        r: 22,
        g: 27,
        b: 34,
    },
    preview_background: Rgb {
        r: 13,
        g: 17,
        b: 23,
    },
};

const NORD: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    background: Rgb {
        r: 46,
        g: 52,
        b: 64,
    },
    panel_background: Rgb {
        r: 59,
        g: 66,
        b: 82,
    },
    text: Rgb {
        r: 216,
        g: 222,
        b: 233,
    },
    text_secondary: Rgb {
        r: 163,
        g: 190,
        b: 140,
    },
    accent: Rgb {
        r: 136,
        g: 192,
        b: 208,
    },
    title_bar_text: Rgb {
        r: 163,
        g: 190,
        b: 140,
    },
    file_tree_text: Rgb {
        r: 216,
        g: 222,
        b: 233,
    },
    active_file_highlight: Rgba {
        r: 67,
        g: 76,
        b: 94,
        a: 120,
    },
    warning_text: Rgb {
        r: 235,
        g: 203,
        b: 139,
    },
    border: Rgb {
        r: 67,
        g: 76,
        b: 94,
    },
    selection: Rgb {
        r: 67,
        g: 76,
        b: 94,
    },
    code_background: Rgb {
        r: 59,
        g: 66,
        b: 82,
    },
    preview_background: Rgb {
        r: 46,
        g: 52,
        b: 64,
    },
};

const MONOKAI: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    background: Rgb {
        r: 39,
        g: 40,
        b: 34,
    },
    panel_background: Rgb {
        r: 49,
        g: 50,
        b: 44,
    },
    text: Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    text_secondary: Rgb {
        r: 117,
        g: 113,
        b: 94,
    },
    accent: Rgb {
        r: 102,
        g: 217,
        b: 239,
    },
    title_bar_text: Rgb {
        r: 117,
        g: 113,
        b: 94,
    },
    file_tree_text: Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    active_file_highlight: Rgba {
        r: 73,
        g: 72,
        b: 62,
        a: 120,
    },
    warning_text: Rgb {
        r: 253,
        g: 151,
        b: 31,
    },
    border: Rgb {
        r: 73,
        g: 72,
        b: 62,
    },
    selection: Rgb {
        r: 73,
        g: 72,
        b: 62,
    },
    code_background: Rgb {
        r: 49,
        g: 50,
        b: 44,
    },
    preview_background: Rgb {
        r: 39,
        g: 40,
        b: 34,
    },
};

// ── Light presets (const) ──

const KATANA_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    background: Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
    panel_background: Rgb {
        r: 243,
        g: 243,
        b: 243,
    },
    text: Rgb {
        r: 36,
        g: 36,
        b: 36,
    },
    text_secondary: Rgb {
        r: 100,
        g: 100,
        b: 100,
    },
    accent: Rgb {
        r: 0,
        g: 120,
        b: 212,
    },
    title_bar_text: Rgb {
        r: 100,
        g: 100,
        b: 100,
    },
    file_tree_text: Rgb {
        r: 36,
        g: 36,
        b: 36,
    },
    active_file_highlight: Rgba {
        r: 0,
        g: 120,
        b: 212,
        a: 40,
    },
    warning_text: Rgb {
        r: 200,
        g: 120,
        b: 0,
    },
    border: Rgb {
        r: 220,
        g: 220,
        b: 220,
    },
    selection: Rgb {
        r: 173,
        g: 214,
        b: 255,
    },
    code_background: Rgb {
        r: 243,
        g: 243,
        b: 243,
    },
    preview_background: Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
};

const GITHUB_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    background: Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
    panel_background: Rgb {
        r: 246,
        g: 248,
        b: 250,
    },
    text: Rgb {
        r: 31,
        g: 35,
        b: 40,
    },
    text_secondary: Rgb {
        r: 101,
        g: 109,
        b: 118,
    },
    accent: Rgb {
        r: 9,
        g: 105,
        b: 218,
    },
    title_bar_text: Rgb {
        r: 101,
        g: 109,
        b: 118,
    },
    file_tree_text: Rgb {
        r: 31,
        g: 35,
        b: 40,
    },
    active_file_highlight: Rgba {
        r: 9,
        g: 105,
        b: 218,
        a: 30,
    },
    warning_text: Rgb {
        r: 191,
        g: 135,
        b: 0,
    },
    border: Rgb {
        r: 216,
        g: 222,
        b: 228,
    },
    selection: Rgb {
        r: 218,
        g: 234,
        b: 247,
    },
    code_background: Rgb {
        r: 246,
        g: 248,
        b: 250,
    },
    preview_background: Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
};

const SOLARIZED_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    background: Rgb {
        r: 253,
        g: 246,
        b: 227,
    },
    panel_background: Rgb {
        r: 238,
        g: 232,
        b: 213,
    },
    text: Rgb {
        r: 101,
        g: 123,
        b: 131,
    },
    text_secondary: Rgb {
        r: 147,
        g: 161,
        b: 161,
    },
    accent: Rgb {
        r: 38,
        g: 139,
        b: 210,
    },
    title_bar_text: Rgb {
        r: 147,
        g: 161,
        b: 161,
    },
    file_tree_text: Rgb {
        r: 101,
        g: 123,
        b: 131,
    },
    active_file_highlight: Rgba {
        r: 38,
        g: 139,
        b: 210,
        a: 40,
    },
    warning_text: Rgb {
        r: 203,
        g: 75,
        b: 22,
    },
    border: Rgb {
        r: 238,
        g: 232,
        b: 213,
    },
    selection: Rgb {
        r: 238,
        g: 232,
        b: 213,
    },
    code_background: Rgb {
        r: 238,
        g: 232,
        b: 213,
    },
    preview_background: Rgb {
        r: 253,
        g: 246,
        b: 227,
    },
};

const AYU_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    background: Rgb {
        r: 250,
        g: 250,
        b: 250,
    },
    panel_background: Rgb {
        r: 242,
        g: 242,
        b: 242,
    },
    text: Rgb {
        r: 92,
        g: 101,
        b: 112,
    },
    text_secondary: Rgb {
        r: 157,
        g: 170,
        b: 182,
    },
    accent: Rgb {
        r: 255,
        g: 170,
        b: 51,
    },
    title_bar_text: Rgb {
        r: 157,
        g: 170,
        b: 182,
    },
    file_tree_text: Rgb {
        r: 92,
        g: 101,
        b: 112,
    },
    active_file_highlight: Rgba {
        r: 255,
        g: 170,
        b: 51,
        a: 40,
    },
    warning_text: Rgb {
        r: 255,
        g: 106,
        b: 0,
    },
    border: Rgb {
        r: 218,
        g: 218,
        b: 218,
    },
    selection: Rgb {
        r: 224,
        g: 224,
        b: 224,
    },
    code_background: Rgb {
        r: 242,
        g: 242,
        b: 242,
    },
    preview_background: Rgb {
        r: 250,
        g: 250,
        b: 250,
    },
};

const GRUVBOX_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    background: Rgb {
        r: 251,
        g: 241,
        b: 199,
    },
    panel_background: Rgb {
        r: 235,
        g: 219,
        b: 178,
    },
    text: Rgb {
        r: 60,
        g: 56,
        b: 54,
    },
    text_secondary: Rgb {
        r: 102,
        g: 92,
        b: 84,
    },
    accent: Rgb {
        r: 69,
        g: 133,
        b: 136,
    },
    title_bar_text: Rgb {
        r: 102,
        g: 92,
        b: 84,
    },
    file_tree_text: Rgb {
        r: 60,
        g: 56,
        b: 54,
    },
    active_file_highlight: Rgba {
        r: 69,
        g: 133,
        b: 136,
        a: 50,
    },
    warning_text: Rgb {
        r: 214,
        g: 93,
        b: 14,
    },
    border: Rgb {
        r: 213,
        g: 196,
        b: 161,
    },
    selection: Rgb {
        r: 213,
        g: 196,
        b: 161,
    },
    code_background: Rgb {
        r: 235,
        g: 219,
        b: 178,
    },
    preview_background: Rgb {
        r: 251,
        g: 241,
        b: 199,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_presets_returns_ten_items() {
        assert_eq!(ThemePreset::all().len(), 10);
    }

    #[test]
    fn default_preset_is_katana_dark() {
        assert_eq!(ThemePreset::default(), ThemePreset::KatanaDark);
    }

    #[test]
    fn each_preset_has_matching_name() {
        for preset in ThemePreset::all() {
            let colors = preset.colors();
            assert_eq!(colors.name, preset.display_name());
        }
    }

    #[test]
    fn dark_presets_have_dark_mode() {
        let dark = [
            ThemePreset::KatanaDark,
            ThemePreset::Dracula,
            ThemePreset::GitHubDark,
            ThemePreset::Nord,
            ThemePreset::Monokai,
        ];
        for preset in &dark {
            let name = preset.display_name();
            assert_eq!(preset.colors().mode, ThemeMode::Dark, "{}", name);
        }
    }

    #[test]
    fn light_presets_have_light_mode() {
        let light = [
            ThemePreset::KatanaLight,
            ThemePreset::GitHubLight,
            ThemePreset::SolarizedLight,
            ThemePreset::AyuLight,
            ThemePreset::GruvboxLight,
        ];
        for preset in &light {
            let name = preset.display_name();
            assert_eq!(preset.colors().mode, ThemeMode::Light, "{}", name);
        }
    }

    #[test]
    fn syntax_theme_name_matches_mode() {
        assert_eq!(
            ThemePreset::KatanaDark.colors().syntax_theme_name(),
            "base16-ocean.dark"
        );
        assert_eq!(
            ThemePreset::KatanaLight.colors().syntax_theme_name(),
            "base16-ocean.light"
        );
    }

    #[test]
    fn theme_colors_clone_is_equal() {
        let original = ThemePreset::KatanaDark.colors();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn theme_preset_serde_roundtrip() {
        let preset = ThemePreset::Dracula;
        let json = serde_json::to_string(&preset).expect("serialize preset");
        let loaded: ThemePreset = serde_json::from_str(&json).expect("deserialize preset");
        assert_eq!(loaded, preset);
    }

    #[test]
    fn theme_colors_serde_roundtrip() {
        let colors = ThemePreset::Nord.colors();
        let json = serde_json::to_string(&colors).expect("serialize colors");
        let loaded: ThemeColors = serde_json::from_str(&json).expect("deserialize colors");
        assert_eq!(loaded, colors);
    }
}
