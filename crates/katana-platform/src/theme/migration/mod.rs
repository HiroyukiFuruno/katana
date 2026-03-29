mod constants;
mod legacy_types;

use crate::theme::migration::constants::*;
use crate::theme::migration::legacy_types::{ThemeColorsDef, ThemeColorsLegacyData};
use crate::theme::types::{CodeColors, PreviewColors, SystemColors, ThemeColors, ThemeMode};
use serde::{Deserialize, Deserializer};

// WHY: ── Deserialize implementation ──────────────────────────────────

impl<'de> Deserialize<'de> for ThemeColors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_theme_colors(deserializer)
    }
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
        Migrator::Legacy(l) => Ok(l.into_theme_colors()),
    }
}

impl ThemeColorsLegacyData {
    fn to_system_colors(&self, is_dark: bool) -> SystemColors {
        SystemColors {
            background: self.background,
            panel_background: self.panel_background,
            text: self.text,
            text_secondary: self.text_secondary,
            success_text: if is_dark {
                LEGACY_SUCCESS_DARK
            } else {
                LEGACY_SUCCESS_LIGHT
            },
            warning_text: self.warning_text,
            error_text: self.error_text,
            accent: self.accent,
            title_bar_text: self.title_bar_text,
            file_tree_text: self.file_tree_text,
            active_file_highlight: self.active_file_highlight,
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
            border: self.border,
            selection: self.selection,
        }
    }

    fn to_code_colors(&self, is_dark: bool) -> CodeColors {
        CodeColors {
            background: self.code_background,
            text: self.text,
            line_number_text: if is_dark {
                LEGACY_LINE_NUMBER_DARK
            } else {
                LEGACY_LINE_NUMBER_LIGHT
            },
            line_number_active_text: self.text,
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
            selection: self.selection,
        }
    }

    fn to_preview_colors(&self, is_dark: bool) -> PreviewColors {
        PreviewColors {
            background: self.preview_background,
            text: self.text,
            warning_text: self.warning_text,
            border: self.border,
            selection: self.selection,
            hover_line_background: if is_dark {
                LEGACY_HOVER_LINE_DARK
            } else {
                LEGACY_HOVER_LINE_LIGHT
            },
        }
    }

    fn into_theme_colors(self) -> ThemeColors {
        let is_dark = self.mode == ThemeMode::Dark;
        let system = self.to_system_colors(is_dark);
        let code = self.to_code_colors(is_dark);
        let preview = self.to_preview_colors(is_dark);

        ThemeColors {
            name: self.name,
            mode: self.mode,
            system,
            code,
            preview,
        }
    }
}
