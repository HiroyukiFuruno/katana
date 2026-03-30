/* WHY: Serde default value functions and `Default` implementations for each struct.

Consolidates all settings default value generators to keep types.rs clean. Extracted to handle Serde defaults safely.
SAFETY: Contains no stateful logic or new type definitions, only purely functional value generation and `Default` trait implementations. */
use crate::theme::ThemePreset;

use super::types::{
    AppSettings, BehaviorSettings, ExportSettings, FontSettings, LayoutSettings,
    PerformanceSettings, ThemeSettings, UpdateSettings, WorkspaceSettings,
    DEFAULT_IGNORED_DIRECTORIES, DEFAULT_MAX_DEPTH,
};

// WHY: ── Constants ──

pub(crate) const DEFAULT_FONT_SIZE: f32 = 14.0;
pub(crate) const DEFAULT_DIAGRAM_CONCURRENCY: usize = 4;
pub(crate) const DEFAULT_AUTO_SAVE_INTERVAL_SECS: f64 = 5.0;

// WHY: ── Serde default functions ──

pub(crate) fn default_version() -> String {
    "0.2.0".to_string()
}

pub(crate) fn default_theme() -> String {
    "dark".to_string()
}

pub(crate) fn default_ui_contrast_offset() -> f32 {
    0.0
}

pub(crate) fn default_font_size() -> f32 {
    DEFAULT_FONT_SIZE
}

pub(crate) fn default_font_family() -> String {
    "monospace".to_string()
}

pub(crate) fn default_language() -> String {
    "en".to_string()
}

pub fn default_true() -> bool {
    true
}

pub(crate) fn default_auto_save_interval_secs() -> f64 {
    DEFAULT_AUTO_SAVE_INTERVAL_SECS
}

pub(crate) fn default_html_output_dir() -> String {
    std::env::temp_dir().to_string_lossy().to_string()
}

pub(crate) fn default_visible_extensions() -> Vec<String> {
    ["md", "markdown", "mdx", "txt", "adr"]
        .iter()
        .map(|&s| s.into())
        .collect()
}

pub(crate) fn default_extensionless_excludes() -> Vec<String> {
    [".DS_Store", ".gitignore", ".gitattributes", "Makefile"]
        .iter()
        .map(|&s| s.into())
        .collect()
}

pub(crate) fn default_ignored_directories() -> Vec<String> {
    DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|&s| s.into())
        .collect()
}

pub(crate) fn default_max_depth() -> usize {
    DEFAULT_MAX_DEPTH
}

// WHY: ── OS theme auto-detection ──

/* WHY: Selects the initial theme preset based on the OS dark/light mode setting.

Called only on first launch. Returns `KatanaDark` when the OS is in dark mode
(or when detection is unavailable), and `KatanaLight` otherwise. */
pub(crate) fn select_initial_preset() -> ThemePreset {
    select_preset_for_mode(crate::os_theme::is_dark_mode())
}

/* WHY: Pure helper: selects the preset for a given dark-mode query result.
Factored out to allow unit testing of both branches without OS dependency. */
pub(crate) fn select_preset_for_mode(is_dark: Option<bool>) -> ThemePreset {
    match is_dark {
        Some(false) => ThemePreset::KatanaLight,
        _ => ThemePreset::KatanaDark, // WHY: dark mode or unknown -> dark by default
    }
}

// WHY: ── Default impls ──

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: default_version(),
            theme: ThemeSettings::default(),
            font: FontSettings::default(),
            layout: LayoutSettings::default(),
            workspace: WorkspaceSettings::default(),
            performance: PerformanceSettings::default(),
            export: ExportSettings::default(),
            updates: UpdateSettings::default(),
            behavior: BehaviorSettings::default(),
            terms_accepted_version: None,
            language: default_language(),
            extra: Vec::new(),
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            ui_contrast_offset: default_ui_contrast_offset(),
            preset: ThemePreset::default(),
            custom_color_overrides: None,
            custom_themes: Vec::new(),
            active_custom_theme: None,
        }
    }
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            size: default_font_size(),
            family: default_font_family(),
        }
    }
}

pub const DEFAULT_CACHE_RETENTION_DAYS: u32 = 7;

pub const fn default_cache_retention() -> u32 {
    DEFAULT_CACHE_RETENTION_DAYS
}