//! Application settings persistence.
//!
//! Settings are loaded from and saved to a JSON file via `JsonFileRepository`.
//! For tests, `InMemoryRepository` provides a no-op backend.

use crate::theme::{ThemeColors, ThemePreset};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod migration;
pub mod migration_0_1_2;
pub mod migration_0_1_3_to_0_1_4;
pub mod migration_0_1_4_to_0_2_0;
use migration::MigrationRunner;

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

const DEFAULT_FONT_SIZE: f32 = 14.0;
/// Minimum allowed font size in pixels.
pub const MIN_FONT_SIZE: f32 = 8.0;
/// Maximum allowed font size in pixels.
pub const MAX_FONT_SIZE: f32 = 32.0;

/// Application-level settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Version string for schema migration.
    #[serde(default = "default_version")]
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
    #[serde(default = "default_language")]
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

pub const MAX_CUSTOM_THEMES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomTheme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    #[serde(default = "default_theme")]
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

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            preset: ThemePreset::default(),
            custom_color_overrides: None,
            custom_themes: Vec::new(),
            active_custom_theme: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    #[serde(default = "default_font_size")]
    pub size: f32,
    #[serde(default = "default_font_family")]
    pub family: String,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            size: default_font_size(),
            family: default_font_family(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    #[serde(default)]
    pub split_direction: SplitDirection,
    #[serde(default)]
    pub pane_order: PaneOrder,
    #[serde(default = "crate::settings::default_true")]
    pub toc_visible: bool,
    #[serde(default)]
    pub toc_position: TocPosition,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            split_direction: Default::default(),
            pane_order: Default::default(),
            toc_visible: true,
            toc_position: Default::default(),
        }
    }
}

pub(crate) fn default_true() -> bool {
    true
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
    #[serde(default = "default_ignored_directories")]
    pub ignored_directories: Vec<String>,
    /// Maximum depth for recursive directory scanning.
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    /// Visible extensions in the workspace tree.
    #[serde(default = "default_visible_extensions")]
    pub visible_extensions: Vec<String>,

    /// Excluded exact file names when "no extension" files are visible.
    #[serde(default = "default_extensionless_excludes")]
    pub extensionless_excludes: Vec<String>,
}

fn default_visible_extensions() -> Vec<String> {
    vec![
        "md".to_string(),
        "markdown".to_string(),
        "mdx".to_string(),
        "txt".to_string(),
        "adr".to_string(),
    ]
}

fn default_extensionless_excludes() -> Vec<String> {
    vec![
        ".DS_Store".to_string(),
        ".gitignore".to_string(),
        ".gitattributes".to_string(),
        "Makefile".to_string(),
    ]
}
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

fn default_ignored_directories() -> Vec<String> {
    DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|&s| s.to_string())
        .collect()
}

fn default_max_depth() -> usize {
    DEFAULT_MAX_DEPTH
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            last_workspace: None,
            paths: vec![],
            open_tabs: vec![],
            active_tab_idx: None,
            ignored_directories: default_ignored_directories(),
            max_depth: DEFAULT_MAX_DEPTH,
            visible_extensions: default_visible_extensions(),
            extensionless_excludes: default_extensionless_excludes(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Number of concurrent diagram renders.
    pub diagram_concurrency: usize,
}

const DEFAULT_DIAGRAM_CONCURRENCY: usize = 4;

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            // Default 4 workers, reduces dynamically upon errors like OOM
            diagram_concurrency: DEFAULT_DIAGRAM_CONCURRENCY,
        }
    }
}

/// Export-related settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Directory for HTML export output. Defaults to the system temp directory.
    #[serde(default = "default_html_output_dir")]
    pub html_output_dir: String,
}

fn default_html_output_dir() -> String {
    std::env::temp_dir().to_string_lossy().to_string()
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            html_output_dir: default_html_output_dir(),
        }
    }
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

/// Default auto-save interval in seconds.
const DEFAULT_AUTO_SAVE_INTERVAL_SECS: f64 = 5.0;

fn default_auto_save_interval_secs() -> f64 {
    DEFAULT_AUTO_SAVE_INTERVAL_SECS
}

/// Application behavior settings controlling system-level defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    /// Show a confirmation dialog when closing a tab with unsaved changes.
    #[serde(default = "crate::settings::default_true")]
    pub confirm_close_dirty_tab: bool,
    /// Synchronise scroll position between editor and preview in split view.
    #[serde(default = "crate::settings::default_true")]
    pub scroll_sync_enabled: bool,
    /// Enable automatic saving of dirty documents.
    #[serde(default)]
    pub auto_save: bool,
    /// Interval in seconds between auto-save triggers.
    #[serde(default = "default_auto_save_interval_secs")]
    pub auto_save_interval_secs: f64,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            confirm_close_dirty_tab: true,
            scroll_sync_enabled: true,
            auto_save: false,
            auto_save_interval_secs: DEFAULT_AUTO_SAVE_INTERVAL_SECS,
        }
    }
}

fn default_version() -> String {
    "0.2.0".to_string()
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

/// Selects the initial theme preset based on the OS dark/light mode setting.
///
/// Called only on first launch. Returns `KatanaDark` when the OS is in dark mode
/// (or when detection is unavailable), and `KatanaLight` otherwise.
fn select_initial_preset() -> ThemePreset {
    select_preset_for_mode(crate::os_theme::is_dark_mode())
}

/// Pure helper: selects the preset for a given dark-mode query result.
/// Factored out to allow unit testing of both branches without OS dependency.
fn select_preset_for_mode(is_dark: Option<bool>) -> ThemePreset {
    match is_dark {
        Some(false) => ThemePreset::KatanaLight,
        _ => ThemePreset::KatanaDark, // dark mode or unknown -> dark by default
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: default_version(),
            theme: ThemeSettings::default(),
            font: FontSettings::default(),
            layout: LayoutSettings::default(),
            workspace: WorkspaceSettings::default(),
            performance: PerformanceSettings::default(),
            export: ExportSettings::default(),
            updates: UpdateSettings::default(),
            behavior: BehaviorSettings::default(),
            terms_accepted_version: None,
            language: default_language(),
            extra: Vec::new(),
        }
    }
}

impl AppSettings {
    /// Returns the effective theme colours.
    ///
    /// If the user has custom overrides, those are returned;
    /// otherwise the selected preset's palette is used.
    pub fn effective_theme_colors(&self) -> ThemeColors {
        self.theme
            .custom_color_overrides
            .clone()
            .unwrap_or_else(|| self.theme.preset.colors())
    }

    /// Sets font size, clamping to the allowed range [`MIN_FONT_SIZE`, `MAX_FONT_SIZE`].
    pub fn set_font_size(&mut self, size: f32) {
        self.font.size = size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
    }

    /// Returns the font size clamped to [`MIN_FONT_SIZE`, `MAX_FONT_SIZE`].
    ///
    /// Useful after deserialization where the raw value may be out of range.
    pub fn clamped_font_size(&self) -> f32 {
        self.font.size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE)
    }
}

// ── Repository trait ──

/// Marker identifying whether settings were loaded from a persisted file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsLoadOrigin {
    /// No settings file existed; defaults were used.
    FirstLaunch,
    /// Settings file was read (even if partially corrupt).
    Persisted,
}

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
    path: PathBuf,
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
                        runner.add_strategy(Box::new(migration_0_1_2::Migration0_1_2));
                        runner.add_strategy(Box::new(migration_0_1_3_to_0_1_4::Migration013To014));
                        runner.add_strategy(Box::new(migration_0_1_4_to_0_2_0::Migration014To020));
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
        // If the settings file does not exist, this is a first launch.
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

// ── Service ──

/// Platform settings service.
pub struct SettingsService {
    settings: AppSettings,
    repository: Box<dyn SettingsRepository>,
    /// `true` when the settings were first loaded without an existing settings file.
    is_first_launch: bool,
}

impl SettingsService {
    /// Create a new service backed by the given repository, loading initial settings.
    pub fn new(repository: Box<dyn SettingsRepository>) -> Self {
        let is_first_launch = repository.load_origin() == SettingsLoadOrigin::FirstLaunch;
        let settings = repository.load();
        Self {
            settings,
            repository,
            is_first_launch,
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

    /// Applies the OS-default theme preset on first launch only.
    ///
    /// If this is not a first launch (settings file already existed), this is a no-op
    /// to respect the user's saved theme preference.
    pub fn apply_os_default_theme(&mut self) {
        if !self.is_first_launch {
            return; // Existing users keep their saved preset unchanged.
        }
        let preset = select_initial_preset();
        self.settings.theme.preset = preset.clone();
        self.settings.theme.theme = preset.colors().mode.to_theme_string();
    }

    /// Applies the OS-default language on first launch.
    pub fn apply_os_default_language(&mut self, detected_lang: Option<String>) {
        if !self.is_first_launch {
            return;
        }
        if let Some(lang) = detected_lang {
            self.settings.language = lang;
        }
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
    use crate::theme::Rgb;
    use tempfile::TempDir;

    #[test]
    fn test_layout_settings_default_deserialization() {
        let json = "{}";
        let layout: LayoutSettings = serde_json::from_str(json).unwrap();
        assert!(layout.toc_visible);
    }

    #[test]
    fn test_workspace_settings_default_deserialization() {
        let json = "{}";
        let ws: WorkspaceSettings = serde_json::from_str(json).unwrap();
        assert_eq!(ws.max_depth, DEFAULT_MAX_DEPTH);
        assert!(!ws.visible_extensions.is_empty());
        assert!(!ws.extensionless_excludes.is_empty());
        assert!(!ws.ignored_directories.is_empty());
    }

    #[test]
    fn test_app_settings_default_values() {
        let s = AppSettings::default();
        assert_eq!(s.theme.theme, "dark");
        assert_eq!(s.theme.preset, ThemePreset::KatanaDark);
        assert!(s.theme.custom_color_overrides.is_none());
        assert!((s.font.size - DEFAULT_FONT_SIZE).abs() < f32::EPSILON);
        assert_eq!(s.font.family, "monospace");
        assert_eq!(s.language, "en");
        assert!(s.workspace.last_workspace.is_none());
        assert!(s.workspace.paths.is_empty());
        // Behavior defaults
        assert!(s.behavior.confirm_close_dirty_tab);
        assert!(s.behavior.scroll_sync_enabled);
        assert!(!s.behavior.auto_save);
        assert_eq!(s.behavior.auto_save_interval_secs, 5.0);
    }

    #[test]
    fn test_behavior_settings_defaults() {
        let b = BehaviorSettings::default();
        assert!(b.confirm_close_dirty_tab);
        assert!(b.scroll_sync_enabled);
        assert!(!b.auto_save);
        assert_eq!(b.auto_save_interval_secs, 5.0);
    }

    #[test]
    fn test_behavior_settings_serde_roundtrip() {
        let b = BehaviorSettings {
            confirm_close_dirty_tab: false,
            scroll_sync_enabled: false,
            auto_save: true,
            auto_save_interval_secs: 10.0,
        };
        let json = serde_json::to_string(&b).unwrap();
        let loaded: BehaviorSettings = serde_json::from_str(&json).unwrap();
        assert!(!loaded.confirm_close_dirty_tab);
        assert!(!loaded.scroll_sync_enabled);
        assert!(loaded.auto_save);
        assert_eq!(loaded.auto_save_interval_secs, 10.0);
    }

    #[test]
    fn test_behavior_settings_serde_missing_fields_use_defaults() {
        let json = "{}";
        let loaded: BehaviorSettings = serde_json::from_str(json).unwrap();
        assert!(loaded.confirm_close_dirty_tab);
        assert!(loaded.scroll_sync_enabled);
        assert!(!loaded.auto_save);
        assert_eq!(loaded.auto_save_interval_secs, 5.0);
    }

    #[test]
    fn test_effective_theme_colors_uses_preset_by_default() {
        let s = AppSettings::default();
        let colors = s.effective_theme_colors();
        assert_eq!(colors, ThemePreset::KatanaDark.colors());
    }

    #[test]
    fn test_effective_theme_colors_uses_custom_when_set() {
        let mut s = AppSettings::default();
        let mut custom = ThemePreset::Nord.colors();
        custom.background = Rgb {
            r: 10,
            g: 20,
            b: 30,
        };
        s.theme.custom_color_overrides = Some(custom.clone());
        assert_eq!(s.effective_theme_colors(), custom);
    }

    #[test]
    fn test_in_memory_repository_load_returns_defaults() {
        let repo = InMemoryRepository;
        let settings = repo.load();
        assert_eq!(settings.theme.theme, "dark");
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

        let settings = AppSettings {
            theme: ThemeSettings {
                theme: "light".to_string(),
                ..Default::default()
            },
            language: "ja".to_string(),
            ..Default::default()
        };
        repo.save(&settings).unwrap();

        let loaded = repo.load();
        assert_eq!(loaded.theme.theme, "light");
        assert_eq!(loaded.language, "ja");
    }

    #[test]
    fn test_json_file_repository_load_missing_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.json");
        let repo = JsonFileRepository::new(path);
        let settings = repo.load();
        assert_eq!(settings.theme.theme, "dark");
    }

    #[test]
    fn test_json_file_repository_load_corrupt_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("corrupt.json");
        std::fs::write(&path, "NOT VALID JSON").unwrap();
        let repo = JsonFileRepository::new(path);
        let settings = repo.load();
        assert_eq!(settings.theme.theme, "dark");
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
        assert_eq!(svc.settings().theme.theme, "dark");
    }

    #[test]
    fn test_settings_service_save_delegates_to_repository() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("svc.json");
        let mut svc = SettingsService::new(Box::new(JsonFileRepository::new(path.clone())));
        svc.settings_mut().theme.theme = "light".to_string();
        svc.save().unwrap();

        let loaded = JsonFileRepository::new(path).load();
        assert_eq!(loaded.theme.theme, "light");
    }

    #[test]
    fn test_settings_service_default_uses_in_memory() {
        let svc = SettingsService::default();
        assert_eq!(svc.settings().theme.theme, "dark");
        assert!(svc.save().is_ok());
    }

    #[test]
    fn test_app_settings_serde_roundtrip() {
        let mut s = AppSettings {
            theme: ThemeSettings {
                theme: "light".to_string(),
                ..Default::default()
            },
            font: FontSettings {
                size: 16.0,
                ..Default::default()
            },
            ..Default::default()
        };
        s.extra.push(ExtraSetting {
            key: "key".to_string(),
            value: "value".to_string(),
        });

        let json = serde_json::to_string(&s).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.theme.theme, "light");
        assert!((loaded.font.size - 16.0).abs() < f32::EPSILON);
        let ext = loaded.extra.iter().find(|e| e.key == "key").unwrap();
        assert_eq!(ext.value, "value");
    }

    #[test]
    fn test_app_settings_serde_missing_fields_use_defaults() {
        let json = r#"{"theme": {"theme": "custom"}}"#;
        let loaded: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.theme.theme, "custom");
        assert!((loaded.font.size - DEFAULT_FONT_SIZE).abs() < f32::EPSILON);
        assert_eq!(loaded.language, "en");
    }

    #[test]
    fn test_behavior_settings_fractional_auto_save_interval() {
        let mut b = BehaviorSettings::default();
        b.auto_save_interval_secs = 5.1;

        let json = serde_json::to_string(&b).unwrap();
        // Ensure standard string serialization maintains precision and dec places.
        assert!(
            json.contains("5.1"),
            "Should serialize as float with exactly 1 decimal representation for 0.1s"
        );

        let parsed: BehaviorSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.auto_save_interval_secs, 5.1,
            "Must roundtrip 0.1 float boundaries precisely to support egui interval sliding"
        );

        // Edge boundary testing
        b.auto_save_interval_secs = 0.0;
        let parsed: BehaviorSettings =
            serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
        assert_eq!(
            parsed.auto_save_interval_secs, 0.0,
            "Zero boundary strict matching"
        );

        b.auto_save_interval_secs = 300.0;
        let parsed: BehaviorSettings =
            serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
        assert_eq!(
            parsed.auto_save_interval_secs, 300.0,
            "Max boundary strict matching"
        );
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

    #[test]
    fn test_theme_preset_save_and_restore() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        // Save: change preset to Dracula
        let mut settings = AppSettings::default();
        settings.theme.preset = ThemePreset::Dracula;
        let repo = JsonFileRepository::new(path.clone());
        repo.save(&settings).unwrap();

        // Restore: Dracula should be persisted
        let loaded = repo.load();
        assert_eq!(loaded.theme.preset, ThemePreset::Dracula);
        assert!(loaded.theme.custom_color_overrides.is_none());
    }

    #[test]
    fn test_custom_color_overrides_save_and_restore() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        // Save custom colour overrides
        let mut settings = AppSettings::default();
        settings.theme.preset = ThemePreset::Nord;
        let mut custom = ThemePreset::Nord.colors();
        custom.background = Rgb {
            r: 10,
            g: 20,
            b: 30,
        };
        settings.theme.custom_color_overrides = Some(custom.clone());
        let repo = JsonFileRepository::new(path.clone());
        repo.save(&settings).unwrap();

        // Restore: custom colours should be persisted correctly
        let loaded = repo.load();
        assert_eq!(loaded.theme.preset, ThemePreset::Nord);
        assert_eq!(loaded.theme.custom_color_overrides, Some(custom));
        assert_eq!(
            loaded.effective_theme_colors().background,
            Rgb {
                r: 10,
                g: 20,
                b: 30
            }
        );
    }

    #[test]
    fn test_split_direction_defaults_to_horizontal() {
        let settings = AppSettings::default();
        assert_eq!(settings.layout.split_direction, SplitDirection::Horizontal);
    }

    #[test]
    fn test_pane_order_defaults_to_editor_first() {
        let settings = AppSettings::default();
        assert_eq!(settings.layout.pane_order, PaneOrder::EditorFirst);
    }

    #[test]
    fn test_layout_settings_serde_backward_compat() {
        // Existing JSON without split_direction/pane_order must deserialize
        // to the default values so that existing users' settings are not broken.
        let json = r#"{"theme": {"theme": "dark"}}"#;
        let loaded: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.layout.split_direction, SplitDirection::Horizontal);
        assert_eq!(loaded.layout.pane_order, PaneOrder::EditorFirst);
    }

    #[test]
    fn test_layout_settings_roundtrip() {
        let mut settings = AppSettings::default();
        settings.layout.split_direction = SplitDirection::Vertical;
        settings.layout.pane_order = PaneOrder::PreviewFirst;

        let json = serde_json::to_string(&settings).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.layout.split_direction, SplitDirection::Vertical);
        assert_eq!(loaded.layout.pane_order, PaneOrder::PreviewFirst);
    }

    // ── Task 5.3: OS theme auto-selection tests ──

    /// Helper: a test repository that reports `FirstLaunch` and holds a preset.
    struct FirstLaunchRepo {
        preset: ThemePreset,
    }

    impl SettingsRepository for FirstLaunchRepo {
        fn load(&self) -> AppSettings {
            let mut s = AppSettings::default();
            s.theme.preset = self.preset.clone();
            s
        }

        fn save(&self, _settings: &AppSettings) -> anyhow::Result<()> {
            Ok(())
        }

        fn load_origin(&self) -> SettingsLoadOrigin {
            SettingsLoadOrigin::FirstLaunch
        }
    }

    #[test]
    fn test_apply_os_default_theme_is_noop_for_existing_users() {
        // InMemoryRepository defaults to Persisted, so apply_os_default_theme
        // must not change the saved preset (user's choice is respected).
        let mut service = SettingsService::new(Box::new(InMemoryRepository));
        // Manually set a non-default preset to verify it is NOT overwritten.
        service.settings_mut().theme.preset = ThemePreset::Dracula;
        service.apply_os_default_theme();
        assert_eq!(
            service.settings().theme.preset,
            ThemePreset::Dracula,
            "existing user's preset must not be overwritten"
        );
    }

    #[test]
    fn test_apply_os_default_theme_on_first_launch_picks_katana_preset() {
        // On first launch, apply_os_default_theme selects KatanaDark or KatanaLight
        // depending on the OS theme (KatanaDark when unknown / non-macOS).
        let repo = FirstLaunchRepo {
            preset: ThemePreset::KatanaDark, // initial value before apply
        };
        let mut service = SettingsService::new(Box::new(repo));
        service.apply_os_default_theme();
        let preset = &service.settings().theme.preset;
        // Must be one of the two Katana presets — never a third-party preset.
        assert!(
            *preset == ThemePreset::KatanaDark || *preset == ThemePreset::KatanaLight,
            "first launch must yield KatanaDark or KatanaLight, got {preset:?}"
        );
    }

    #[test]
    fn test_select_preset_for_mode_dark() {
        // Explicit dark-mode input must yield KatanaDark.
        assert_eq!(select_preset_for_mode(Some(true)), ThemePreset::KatanaDark);
    }

    #[test]
    fn test_select_preset_for_mode_light() {
        // Explicit light-mode input must yield KatanaLight.
        assert_eq!(
            select_preset_for_mode(Some(false)),
            ThemePreset::KatanaLight
        );
    }

    #[test]
    fn test_select_preset_for_mode_unknown() {
        // Unknown (None) falls back to KatanaDark.
        assert_eq!(select_preset_for_mode(None), ThemePreset::KatanaDark);
    }

    #[test]
    fn test_first_launch_repo_save_is_noop() {
        // Covers the save() implementation of the test helper.
        let repo = FirstLaunchRepo {
            preset: ThemePreset::KatanaDark,
        };
        let settings = AppSettings::default();
        assert!(
            repo.save(&settings).is_ok(),
            "FirstLaunchRepo::save() must succeed"
        );
    }

    #[test]
    fn test_apply_os_default_language_is_noop_for_existing_users() {
        let mut service = SettingsService::new(Box::new(InMemoryRepository));
        service.settings_mut().language = "ja".to_string();
        service.apply_os_default_language(Some("fr".to_string()));
        assert_eq!(service.settings().language, "ja");

        // None case
        service.apply_os_default_language(None);
        assert_eq!(service.settings().language, "ja");
    }

    #[test]
    fn test_apply_os_default_language_updates_on_first_launch() {
        let repo = FirstLaunchRepo {
            preset: ThemePreset::KatanaDark,
        };
        let mut service = SettingsService::new(Box::new(repo));

        // Test with None to ensure it does not overwrite
        // Default AppSettings language is "en"
        service.apply_os_default_language(None);
        assert_eq!(service.settings().language, "en");

        // Test with Some
        service.apply_os_default_language(Some("fr".to_string()));
        assert_eq!(service.settings().language, "fr");
    }

    #[test]
    fn test_skipped_version_persistence_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        let mut settings = AppSettings::default();
        settings.updates.skipped_version = Some("v0.8.0".to_string());
        let repo = JsonFileRepository::new(path.clone());
        repo.save(&settings).unwrap();

        let loaded = repo.load();
        assert_eq!(loaded.updates.skipped_version, Some("v0.8.0".to_string()));
    }

    #[test]
    fn test_skipped_version_backward_compat() {
        // Existing JSON without skipped_version must deserialize to None
        let json = r#"{"updates": {"interval": "Daily"}}"#;
        let loaded: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.updates.skipped_version, None);
    }
}
