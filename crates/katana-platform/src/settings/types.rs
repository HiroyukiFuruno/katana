//! Settings type definitions.
//!
//! All settings-related structs, enums, and constants are consolidated here.

use crate::theme::{ThemeColors, ThemePreset};
use serde::{Deserialize, Serialize};

/// Split direction for editor/preview layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SplitDirection {
    /// Editor on left, preview on right.
    #[default]
    Horizontal,
    /// Editor on top, preview on bottom.
    Vertical,
}

/// Position of the Table of Contents panel in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TocPosition {
    /// Left side of the workspace.
    #[default]
    Left,
    /// Right side of the workspace.
    Right,
}

/// Pane order within the split view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PaneOrder {
    /// Editor first (left or top), preview second.
    #[default]
    EditorFirst,
    /// Preview first (left or top), editor second.
    PreviewFirst,
}

/// Minimum allowed font size in pixels.
pub const MIN_FONT_SIZE: f32 = 8.0;
/// Maximum allowed font size in pixels.
pub const MAX_FONT_SIZE: f32 = 32.0;

pub const MAX_CUSTOM_THEMES: usize = 10;

/// Default maximum recursion depth for workspace scanning.
pub const DEFAULT_MAX_DEPTH: usize = 10;

/// Default list of directory names to ignore during workspace scanning.
pub const DEFAULT_IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    ".terraform",
    "node_modules",
    "target",
    ".idea",
    ".vscode",
];

/// Application-level settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Version string for schema migration.
    #[serde(default = "super::defaults::default_version")]
    pub version: String,
    /// Theme settings (nesting).
    #[serde(default)]
    pub theme: ThemeSettings,
    /// Font settings (nesting).
    #[serde(default)]
    pub font: FontSettings,
    /// Layout settings (nesting).
    #[serde(default)]
    pub layout: LayoutSettings,

    /// Workspace settings (nesting).
    #[serde(default)]
    pub workspace: WorkspaceSettings,

    /// Performance and advanced tuning (nesting).
    #[serde(default)]
    pub performance: PerformanceSettings,

    /// Export settings (nesting).
    #[serde(default)]
    pub export: ExportSettings,

    /// Application update settings (nesting).
    #[serde(default)]
    pub updates: UpdateSettings,

    /// Behavior / system-default settings (nesting).
    #[serde(default)]
    pub behavior: BehaviorSettings,

    /// Terms of service accepted version (None = not accepted).
    #[serde(default)]
    pub terms_accepted_version: Option<String>,
    /// UI language ("en" or "ja", etc).
    #[serde(default = "super::defaults::default_language")]
    pub language: String,
    /// Additional key-value settings for future use.
    #[serde(default)]
    pub extra: Vec<ExtraSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtraSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomTheme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    #[serde(default = "super::defaults::default_theme")]
    pub theme: String,
    #[serde(default)]
    pub preset: ThemePreset,
    #[serde(default)]
    pub custom_color_overrides: Option<ThemeColors>,
    #[serde(default)]
    pub custom_themes: Vec<CustomTheme>,
    #[serde(default)]
    pub active_custom_theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    #[serde(default = "super::defaults::default_font_size")]
    pub size: f32,
    #[serde(default = "super::defaults::default_font_family")]
    pub family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    #[serde(default)]
    pub split_direction: SplitDirection,
    #[serde(default)]
    pub pane_order: PaneOrder,
    #[serde(default = "super::defaults::default_true")]
    pub toc_visible: bool,
    #[serde(default)]
    pub toc_position: TocPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// ID of the last opened workspace root path, restored on next launch.
    #[serde(default)]
    pub last_workspace: Option<String>,
    /// Workspace directory paths.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Previously opened document tabs.
    #[serde(default)]
    pub open_tabs: Vec<String>,
    /// Index of the actively selected tab.
    #[serde(default)]
    pub active_tab_idx: Option<usize>,
    /// Directories to ignore during workspace scanning.
    #[serde(default = "super::defaults::default_ignored_directories")]
    pub ignored_directories: Vec<String>,
    /// Maximum depth for recursive directory scanning.
    #[serde(default = "super::defaults::default_max_depth")]
    pub max_depth: usize,
    /// Visible extensions in the workspace tree.
    #[serde(default = "super::defaults::default_visible_extensions")]
    pub visible_extensions: Vec<String>,

    /// Excluded exact file names when "no extension" files are visible.
    #[serde(default = "super::defaults::default_extensionless_excludes")]
    pub extensionless_excludes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Number of concurrent diagram renders.
    pub diagram_concurrency: usize,
    /// Number of days to retain HTTP image cache.
    #[serde(default = "super::defaults::default_cache_retention")]
    pub http_image_cache_retention_days: u32,
}

/// Export-related settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Directory for HTML export output. Defaults to the system temp directory.
    #[serde(default = "super::defaults::default_html_output_dir")]
    pub html_output_dir: String,
}

/// Interval for checking for application updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UpdateInterval {
    /// Skip automatic updates
    Never,
    /// Check for updates daily
    #[default]
    Daily,
    /// Check for updates weekly
    Weekly,
    /// Check for updates monthly
    Monthly,
}

/// Auto-updater configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSettings {
    /// The interval at which the app should check for updates.
    #[serde(default)]
    pub interval: UpdateInterval,
    /// The last time an update check was performed (UNIX timestamp in seconds).
    #[serde(default)]
    pub last_checked_timestamp_sec: Option<u64>,
    /// Version tag the user explicitly chose to skip (e.g. "v0.8.0").
    /// Auto-check will suppress notifications for this version.
    #[serde(default)]
    pub skipped_version: Option<String>,
}

/// Application behavior settings controlling system-level defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    /// Show a confirmation dialog when closing a tab with unsaved changes.
    #[serde(default = "super::defaults::default_true")]
    pub confirm_close_dirty_tab: bool,
    /// Synchronise scroll position between editor and preview in split view.
    #[serde(default = "super::defaults::default_true")]
    pub scroll_sync_enabled: bool,
    /// Enable automatic saving of dirty documents.
    #[serde(default)]
    pub auto_save: bool,
    /// Interval in seconds between auto-save triggers.
    #[serde(default = "super::defaults::default_auto_save_interval_secs")]
    pub auto_save_interval_secs: f64,
}

/// Marker identifying whether settings were loaded from a persisted file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsLoadOrigin {
    /// No settings file existed; defaults were used.
    FirstLaunch,
    /// Settings file was read (even if partially corrupt).
    Persisted,
}
