use serde::{Deserialize, Serialize};

// WHY: Interval for checking for application updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UpdateInterval {
    // WHY: Skip automatic updates
    Never,
    // WHY: Check for updates daily
    #[default]
    Daily,
    // WHY: Check for updates weekly
    Weekly,
    // WHY: Check for updates monthly
    Monthly,
}

// WHY: Auto-updater configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSettings {
    // WHY: The interval at which the app should check for updates.
    #[serde(default)]
    pub interval: UpdateInterval,
    // WHY: The last time an update check was performed (UNIX timestamp in seconds).
    #[serde(default)]
    pub last_checked_timestamp_sec: Option<u64>,
    /* WHY: Version tag the user explicitly chose to skip (e.g. "v0.8.0").
    Auto-check will suppress notifications for this version. */
    #[serde(default)]
    pub skipped_version: Option<String>,
    /* WHY: The application version recorded during the previous launch.
    Used to determine whether to show the release notes after an update. */
    #[serde(default)]
    pub previous_app_version: Option<String>,
}