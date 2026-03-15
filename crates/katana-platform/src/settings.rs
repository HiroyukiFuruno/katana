//! Application settings persistence.
//!
//! Settings are loaded from and saved to a JSON file via `JsonFileRepository`.
//! For tests, `InMemoryRepository` provides a no-op backend.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const DEFAULT_FONT_SIZE: f32 = 14.0;

/// Application-level settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Theme name ("dark" or "light").
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Font size in pixels.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Font family name.
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// ID of the last opened workspace root path, restored on next launch.
    #[serde(default)]
    pub last_workspace: Option<String>,
    /// Whether the table of contents panel is visible.
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
    DEFAULT_FONT_SIZE
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
            theme: default_theme(),
            font_size: default_font_size(),
            font_family: default_font_family(),
            last_workspace: None,
            toc_visible: false,
            workspace_paths: Vec::new(),
            terms_accepted_version: None,
            language: default_language(),
            extra: HashMap::new(),
        }
    }
}

// ── Repository trait ──

/// Abstraction for loading/saving settings (enables test doubles).
pub trait SettingsRepository: Send {
    /// Load settings from the backing store. Returns defaults on any error.
    fn load(&self) -> AppSettings;
    /// Persist settings to the backing store.
    fn save(&self, settings: &AppSettings) -> anyhow::Result<()>;
}

// ── JSON file repository ──

/// Persists settings as a JSON file on disk.
pub struct JsonFileRepository {
    path: PathBuf,
}

impl JsonFileRepository {
    /// Create a repository targeting the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Create a repository using the platform-standard config directory.
    ///
    /// On macOS: `~/Library/Application Support/katana/settings.json`
    pub fn with_default_path() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("katana").join("settings.json"))
    }
}

impl SettingsRepository for JsonFileRepository {
    fn load(&self) -> AppSettings {
        match std::fs::read_to_string(&self.path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
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

// ── Service ──

/// Platform settings service.
pub struct SettingsService {
    settings: AppSettings,
    repository: Box<dyn SettingsRepository>,
}

impl SettingsService {
    /// Create a new service backed by the given repository, loading initial settings.
    pub fn new(repository: Box<dyn SettingsRepository>) -> Self {
        let settings = repository.load();
        Self {
            settings,
            repository,
        }
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /// Persist current settings via the repository.
    pub fn save(&self) -> anyhow::Result<()> {
        self.repository.save(&self.settings)
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new(Box::new(InMemoryRepository))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_settings_default_values() {
        let s = AppSettings::default();
        assert_eq!(s.theme, "dark");
        assert!((s.font_size - DEFAULT_FONT_SIZE).abs() < f32::EPSILON);
        assert_eq!(s.font_family, "monospace");
        assert_eq!(s.language, "en");
        assert!(s.last_workspace.is_none());
    }

    #[test]
    fn test_in_memory_repository_load_returns_defaults() {
        let repo = InMemoryRepository;
        let settings = repo.load();
        assert_eq!(settings.theme, "dark");
    }

    #[test]
    fn test_in_memory_repository_save_succeeds() {
        let repo = InMemoryRepository;
        let settings = AppSettings::default();
        assert!(repo.save(&settings).is_ok());
    }

    #[test]
    fn test_json_file_repository_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        let repo = JsonFileRepository::new(path);

        let mut settings = AppSettings::default();
        settings.theme = "light".to_string();
        settings.language = "ja".to_string();
        repo.save(&settings).unwrap();

        let loaded = repo.load();
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.language, "ja");
    }

    #[test]
    fn test_json_file_repository_load_missing_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.json");
        let repo = JsonFileRepository::new(path);
        let settings = repo.load();
        assert_eq!(settings.theme, "dark");
    }

    #[test]
    fn test_json_file_repository_load_corrupt_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("corrupt.json");
        std::fs::write(&path, "NOT VALID JSON").unwrap();
        let repo = JsonFileRepository::new(path);
        let settings = repo.load();
        assert_eq!(settings.theme, "dark");
    }

    #[test]
    fn test_json_file_repository_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested").join("dir").join("settings.json");
        let repo = JsonFileRepository::new(path.clone());
        let settings = AppSettings::default();
        repo.save(&settings).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_json_file_repository_with_default_path() {
        let repo = JsonFileRepository::with_default_path();
        // Just ensure it doesn't panic and path ends with settings.json
        assert!(repo.path.ends_with("settings.json"));
    }

    #[test]
    fn test_settings_service_new_loads_from_repository() {
        let svc = SettingsService::new(Box::new(InMemoryRepository));
        assert_eq!(svc.settings().theme, "dark");
    }

    #[test]
    fn test_settings_service_save_delegates_to_repository() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("svc.json");
        let mut svc = SettingsService::new(Box::new(JsonFileRepository::new(path.clone())));
        svc.settings_mut().theme = "light".to_string();
        svc.save().unwrap();

        let loaded = JsonFileRepository::new(path).load();
        assert_eq!(loaded.theme, "light");
    }

    #[test]
    fn test_settings_service_default_uses_in_memory() {
        let svc = SettingsService::default();
        assert_eq!(svc.settings().theme, "dark");
        assert!(svc.save().is_ok());
    }

    #[test]
    fn test_app_settings_serde_roundtrip() {
        let mut s = AppSettings::default();
        s.theme = "light".to_string();
        s.font_size = 16.0;
        s.extra.insert("key".to_string(), "value".to_string());

        let json = serde_json::to_string(&s).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.theme, "light");
        assert!((loaded.font_size - 16.0).abs() < f32::EPSILON);
        assert_eq!(loaded.extra.get("key").unwrap(), "value");
    }

    #[test]
    fn test_app_settings_serde_missing_fields_use_defaults() {
        let json = r#"{"theme": "custom"}"#;
        let loaded: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.theme, "custom");
        assert!((loaded.font_size - DEFAULT_FONT_SIZE).abs() < f32::EPSILON);
        assert_eq!(loaded.language, "en");
    }

    #[test]
    fn test_json_file_repository_save_bare_filename_no_parent() {
        // PathBuf::from("settings.json").parent() returns Some("") which is
        // still handled by create_dir_all(""), covering the closing brace of
        // the if-let block.
        let tmp = TempDir::new().unwrap();
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let repo = JsonFileRepository::new(std::path::PathBuf::from("bare.json"));
        let settings = AppSettings::default();
        // This exercises the parent="" path inside save()
        repo.save(&settings).unwrap();
        assert!(tmp.path().join("bare.json").exists());

        std::env::set_current_dir(cwd).unwrap();
    }

    #[test]
    fn test_json_file_repository_save_create_dir_fails() {
        // create_dir_all fails when the parent path component is an existing file
        let tmp = TempDir::new().unwrap();
        let blocker = tmp.path().join("blocker");
        std::fs::write(&blocker, "I am a file").unwrap();

        // Try to create "blocker/nested/settings.json" — blocker is a file, not a dir
        let path = blocker.join("nested").join("settings.json");
        let repo = JsonFileRepository::new(path);
        let settings = AppSettings::default();
        let result = repo.save(&settings);
        assert!(
            result.is_err(),
            "save should fail when create_dir_all fails"
        );
    }
}
