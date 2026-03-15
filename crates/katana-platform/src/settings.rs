//! Application settings persistence.
//!
//! Provides `SettingsRepository` trait (DIP) for flexible backend switching,
//! with `JsonFileRepository` as the default implementation.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Application-level settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// ID of the last opened workspace root path, restored on next launch.
    #[serde(default)]
    pub last_workspace: Option<String>,
    /// Active AI provider identifier.
    #[serde(default)]
    pub ai_provider: Option<String>,
    /// Theme selection ("dark" or "light"). Default: "dark".
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Font size in pixels. Default: 14.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Font family name.
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// Whether to show Table of Contents by default.
    #[serde(default)]
    pub toc_visible: bool,
    /// Workspace directory paths.
    #[serde(default)]
    pub workspace_paths: Vec<String>,
    /// Terms of service accepted version (None = not accepted).
    #[serde(default)]
    pub terms_accepted_version: Option<String>,
    /// UI language ("en" or "ja").
    #[serde(default = "default_language")]
    pub language: String,
    /// Additional key-value settings for future use.
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_font_size() -> f32 {
    14.0
}

fn default_font_family() -> String {
    "monospace".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_workspace: None,
            ai_provider: None,
            theme: default_theme(),
            font_size: default_font_size(),
            font_family: default_font_family(),
            toc_visible: false,
            workspace_paths: Vec::new(),
            terms_accepted_version: None,
            language: default_language(),
            extra: HashMap::new(),
        }
    }
}
/// Repository trait for settings persistence (DIP).
///
/// Implementations can back onto JSON files, databases, cloud storage, etc.
pub trait SettingsRepository: Send + Sync {
    /// Load settings from the backing store.
    /// Returns default settings if not found or on parse error.
    fn load(&self) -> anyhow::Result<AppSettings>;

    /// Save settings to the backing store.
    fn save(&self, settings: &AppSettings) -> anyhow::Result<()>;
}

/// JSON file-backed repository implementation.
pub struct JsonFileRepository {
    path: PathBuf,
}

impl JsonFileRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Create a repository using the platform-standard config directory.
    /// macOS: ~/Library/Application Support/katana/config.json
    pub fn default_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or(PathBuf::from("."));
        base.join("katana").join("config.json")
    }

    /// Create a repository at the platform-standard location.
    pub fn with_default_path() -> Self {
        Self::new(Self::default_path())
    }
}

impl SettingsRepository for JsonFileRepository {
    fn load(&self) -> anyhow::Result<AppSettings> {
        if !self.path.exists() {
            tracing::info!("Settings file not found at {:?}, using defaults", self.path);
            return Ok(AppSettings::default());
        }

        let content = std::fs::read_to_string(&self.path)?;
        match serde_json::from_str::<AppSettings>(&content) {
            Ok(settings) => Ok(settings),
            Err(e) => {
                tracing::warn!(
                    "Failed to parse settings file {:?}: {}. Using defaults.",
                    self.path,
                    e
                );
                Ok(AppSettings::default())
            }
        }
    }

    fn save(&self, settings: &AppSettings) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.path, json)?;
        tracing::info!("Settings saved to {:?}", self.path);
        Ok(())
    }
}

/// Platform settings service.
pub struct SettingsService {
    settings: AppSettings,
    repository: Box<dyn SettingsRepository>,
}

impl SettingsService {
    pub fn new(repository: Box<dyn SettingsRepository>) -> Self {
        let settings = repository.load().unwrap_or_default();
        Self {
            settings,
            repository,
        }
    }

    /// Create with in-memory only (no persistence).
    pub fn in_memory() -> Self {
        Self {
            settings: AppSettings::default(),
            repository: Box::new(InMemoryRepository),
        }
    }

    /// Load settings from the given path (legacy compatibility).
    pub fn load_from(_path: &str) -> Self {
        Self::in_memory()
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /// Persist current settings to the backing store.
    pub fn save(&self) -> anyhow::Result<()> {
        self.repository.save(&self.settings)
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::in_memory()
    }
}

/// In-memory repository (no persistence). Used for testing and defaults.
pub struct InMemoryRepository;

impl SettingsRepository for InMemoryRepository {
    fn load(&self) -> anyhow::Result<AppSettings> {
        Ok(AppSettings::default())
    }

    fn save(&self, _settings: &AppSettings) -> anyhow::Result<()> {
        Ok(())
    }
}
