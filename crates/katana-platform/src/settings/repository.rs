//! Settings persistence layer.
//!
//! Provides the `SettingsRepository` trait with JSON file and in-memory implementations.

use std::path::PathBuf;

use super::migration::MigrationRunner;
use super::migration::{v0_1_2, v0_1_3_to_0_1_4, v0_1_4_to_0_2_0};
use super::types::{AppSettings, SettingsLoadOrigin};

/// Minimal interface for loading and saving settings.
pub trait SettingsRepository: Send {
    fn load(&self) -> AppSettings;
    fn save(&self, settings: &AppSettings) -> anyhow::Result<()>;
    /// Returns the load origin for detecting first launch.
    fn load_origin(&self) -> SettingsLoadOrigin {
        // Default: assume persisted to avoid false positives in tests.
        SettingsLoadOrigin::Persisted
    }
}

// ── JSON file repository ──

/// Persists settings as a JSON file on disk.
pub struct JsonFileRepository {
    pub(crate) path: PathBuf,
}

impl JsonFileRepository {
    /// Create a repository targeting the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Create a repository using the platform-standard config directory.
    ///
    /// On macOS: `~/Library/Application Support/KatanA/settings.json`
    pub fn with_default_path() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("KatanA").join("settings.json"))
    }
}

impl SettingsRepository for JsonFileRepository {
    fn load(&self) -> AppSettings {
        match std::fs::read_to_string(&self.path) {
            Ok(json_str) => {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
                match parsed {
                    Ok(mut value) => {
                        let mut runner = MigrationRunner::new();
                        runner.add_strategy(Box::new(v0_1_2::Migration0_1_2));
                        runner.add_strategy(Box::new(v0_1_3_to_0_1_4::Migration013To014));
                        runner.add_strategy(Box::new(v0_1_4_to_0_2_0::Migration014To020));
                        value = runner.migrate(value);
                        serde_json::from_value(value).unwrap_or_default()
                    }
                    Err(_) => AppSettings::default(),
                }
            }
            Err(_) => AppSettings::default(),
        }
    }

    fn save(&self, settings: &AppSettings) -> anyhow::Result<()> {
        // Ensure the parent directory exists. filter(|p| !p.as_os_str().is_empty())
        // skips the no-op case when the path has no parent component.
        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.path, json)?;
        tracing::info!("Settings saved to {}", self.path.display());
        Ok(())
    }

    fn load_origin(&self) -> SettingsLoadOrigin {
        if self.path.exists() {
            SettingsLoadOrigin::Persisted
        } else {
            SettingsLoadOrigin::FirstLaunch
        }
    }
}

// ── In-memory repository (for tests) ──

/// No-op repository that never touches the filesystem.
pub struct InMemoryRepository;

impl SettingsRepository for InMemoryRepository {
    fn load(&self) -> AppSettings {
        AppSettings::default()
    }

    fn save(&self, _settings: &AppSettings) -> anyhow::Result<()> {
        Ok(())
    }
}
