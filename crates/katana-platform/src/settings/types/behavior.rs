use serde::{Deserialize, Serialize};

// WHY: Application behavior settings controlling system-level defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    // WHY: Show a confirmation dialog when closing a tab with unsaved changes.
    #[serde(default = "super::super::defaults::default_true")]
    pub confirm_close_dirty_tab: bool,
    // WHY: Synchronise scroll position between editor and preview in split view.
    #[serde(default = "super::super::defaults::default_true")]
    pub scroll_sync_enabled: bool,
    // WHY: Enable automatic saving of dirty documents.
    #[serde(default)]
    pub auto_save: bool,
    // WHY: Interval in seconds between auto-save triggers.
    #[serde(default = "super::super::defaults::default_auto_save_interval_secs")]
    pub auto_save_interval_secs: f64,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            confirm_close_dirty_tab: true,
            scroll_sync_enabled: true,
            auto_save: false,
            auto_save_interval_secs: crate::settings::defaults::DEFAULT_AUTO_SAVE_INTERVAL_SECS,
        }
    }
}
