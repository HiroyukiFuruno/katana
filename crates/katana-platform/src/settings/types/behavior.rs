use serde::{Deserialize, Serialize};

/// Application behavior settings controlling system-level defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    /// Show a confirmation dialog when closing a tab with unsaved changes.
    #[serde(default = "super::super::defaults::default_true")]
    pub confirm_close_dirty_tab: bool,
    /// Synchronise scroll position between editor and preview in split view.
    #[serde(default = "super::super::defaults::default_true")]
    pub scroll_sync_enabled: bool,
    /// Enable automatic saving of dirty documents.
    #[serde(default)]
    pub auto_save: bool,
    /// Interval in seconds between auto-save triggers.
    #[serde(default = "super::super::defaults::default_auto_save_interval_secs")]
    pub auto_save_interval_secs: f64,
}
