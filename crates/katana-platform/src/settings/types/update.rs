use serde::{Deserialize, Serialize};

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
    /// The application version recorded during the previous launch.
    /// Used to determine whether to show the release notes after an update.
    #[serde(default)]
    pub previous_app_version: Option<String>,
}
