use super::{
    behavior::BehaviorSettings, export::ExportSettings, font::FontSettings, layout::LayoutSettings,
    performance::PerformanceSettings, theme::ThemeSettings, update::UpdateSettings,
    workspace::WorkspaceSettings,
};
use serde::{Deserialize, Serialize};

// WHY: Application-level settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // WHY: Version string for schema migration.
    #[serde(default = "super::super::defaults::default_version")]
    pub version: String,
    // WHY: Theme settings (nesting).
    #[serde(default)]
    pub theme: ThemeSettings,
    // WHY: Font settings (nesting).
    #[serde(default)]
    pub font: FontSettings,
    // WHY: Layout settings (nesting).
    #[serde(default)]
    pub layout: LayoutSettings,

    // WHY: Workspace settings (nesting).
    #[serde(default)]
    pub workspace: WorkspaceSettings,

    // WHY: Performance and advanced tuning (nesting).
    #[serde(default)]
    pub performance: PerformanceSettings,

    // WHY: Export settings (nesting).
    #[serde(default)]
    pub export: ExportSettings,

    // WHY: Application update settings (nesting).
    #[serde(default)]
    pub updates: UpdateSettings,

    // WHY: Behavior / system-default settings (nesting).
    #[serde(default)]
    pub behavior: BehaviorSettings,

    // WHY: Terms of service accepted version (None = not accepted).
    #[serde(default)]
    pub terms_accepted_version: Option<String>,
    // WHY: UI language ("en" or "ja", etc).
    #[serde(default = "super::super::defaults::default_language")]
    pub language: String,
    // WHY: Additional key-value settings for future use.
    #[serde(default)]
    pub extra: Vec<ExtraSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtraSetting {
    pub key: String,
    pub value: String,
}

// WHY: Marker identifying whether settings were loaded from a persisted file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsLoadOrigin {
    // WHY: No settings file existed; defaults were used.
    FirstLaunch,
    // WHY: Settings file was read (even if partially corrupt).
    Persisted,
}