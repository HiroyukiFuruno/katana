//! Serde default value functions and `Default` implementations for each struct.

use crate::theme::ThemePreset;

use super::types::*;

// ── Constants ──

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_DIAGRAM_CONCURRENCY: usize = 4;
const DEFAULT_AUTO_SAVE_INTERVAL_SECS: f64 = 5.0;

// ── Serde default functions ──

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
    vec![
        "md".to_string(),
        "markdown".to_string(),
        "mdx".to_string(),
        "txt".to_string(),
        "adr".to_string(),
    ]
}

pub(crate) fn default_extensionless_excludes() -> Vec<String> {
    vec![
        ".DS_Store".to_string(),
        ".gitignore".to_string(),
        ".gitattributes".to_string(),
        "Makefile".to_string(),
    ]
}

pub(crate) fn default_ignored_directories() -> Vec<String> {
    DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|&s| s.to_string())
        .collect()
}

pub(crate) fn default_max_depth() -> usize {
    DEFAULT_MAX_DEPTH
}

// ── OS theme auto-detection ──

/// Selects the initial theme preset based on the OS dark/light mode setting.
///
/// Called only on first launch. Returns `KatanaDark` when the OS is in dark mode
/// (or when detection is unavailable), and `KatanaLight` otherwise.
pub(crate) fn select_initial_preset() -> ThemePreset {
    select_preset_for_mode(crate::os_theme::is_dark_mode())
}

/// Pure helper: selects the preset for a given dark-mode query result.
/// Factored out to allow unit testing of both branches without OS dependency.
pub(crate) fn select_preset_for_mode(is_dark: Option<bool>) -> ThemePreset {
    match is_dark {
        Some(false) => ThemePreset::KatanaLight,
        _ => ThemePreset::KatanaDark, // dark mode or unknown -> dark by default
    }
}

// ── Default impls ──

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

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            split_direction: Default::default(),
            pane_order: Default::default(),
            toc_visible: true,
            toc_position: Default::default(),
        }
    }
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            last_workspace: None,
            paths: vec![],
            open_tabs: vec![],
            active_tab_idx: None,
            ignored_directories: default_ignored_directories(),
            max_depth: DEFAULT_MAX_DEPTH,
            visible_extensions: default_visible_extensions(),
            extensionless_excludes: default_extensionless_excludes(),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            diagram_concurrency: DEFAULT_DIAGRAM_CONCURRENCY,
            http_image_cache_retention_days: default_cache_retention(),
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            html_output_dir: default_html_output_dir(),
        }
    }
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            confirm_close_dirty_tab: true,
            scroll_sync_enabled: true,
            auto_save: false,
            auto_save_interval_secs: DEFAULT_AUTO_SAVE_INTERVAL_SECS,
        }
    }
}

pub const DEFAULT_CACHE_RETENTION_DAYS: u32 = 7;

pub fn default_cache_retention() -> u32 {
    DEFAULT_CACHE_RETENTION_DAYS
}
