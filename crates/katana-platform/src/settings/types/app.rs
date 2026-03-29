use super::{
    behavior::BehaviorSettings, export::ExportSettings, font::FontSettings, layout::LayoutSettings,
    performance::PerformanceSettings, theme::ThemeSettings, update::UpdateSettings,
    workspace::WorkspaceSettings,
};
use serde::{Deserialize, Serialize};

/// Application-level settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Version string for schema migration.
    #[serde(default = "super::super::defaults::default_version")]
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
    #[serde(default = "super::super::defaults::default_language")]
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

/// Marker identifying whether settings were loaded from a persisted file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsLoadOrigin {
    /// No settings file existed; defaults were used.
    FirstLaunch,
    /// Settings file was read (even if partially corrupt).
    Persisted,
}
