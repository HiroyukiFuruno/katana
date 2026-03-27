//! Application settings persistence.
//!
//! Settings are loaded from and saved to a JSON file via `JsonFileRepository`.
//! For tests, `InMemoryRepository` provides a no-op backend.
//!
//! ## Module structure
//!
//! | Module | Responsibility |
//! |---|---|
//! | `types` | All struct, enum, and constant type definitions |
//! | `defaults` | Serde default functions + Default impls |
//! | `impls` | `AppSettings` method implementations |
//! | `repository` | `SettingsRepository` trait + JSON/InMemory implementations |
//! | `service` | `SettingsService` (business logic) |
//! | `migration/` | Schema migration (trait + versioned strategies) |

pub mod defaults;
pub mod impls;
pub mod migration;
pub mod repository;
pub mod service;
pub mod types;

// Public API re-exports to preserve `use crate::settings::*` compatibility.
pub use defaults::default_true;
pub use repository::{InMemoryRepository, JsonFileRepository, SettingsRepository};
pub use service::SettingsService;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Rgb, ThemeColors, ThemeMode, ThemePreset};
    use defaults::select_preset_for_mode;
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
        assert!((s.font.size - 14.0).abs() < f32::EPSILON);
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
        custom.system.background = Rgb {
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
        assert!((loaded.font.size - 14.0).abs() < f32::EPSILON);
        assert_eq!(loaded.language, "en");
    }

    #[test]
    fn test_behavior_settings_fractional_auto_save_interval() {
        let mut b = BehaviorSettings::default();
        b.auto_save_interval_secs = 5.1;

        let json = serde_json::to_string(&b).unwrap();
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
        let tmp = TempDir::new().unwrap();
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let repo = JsonFileRepository::new(std::path::PathBuf::from("bare.json"));
        let settings = AppSettings::default();
        repo.save(&settings).unwrap();
        assert!(tmp.path().join("bare.json").exists());

        std::env::set_current_dir(cwd).unwrap();
    }

    #[test]
    fn test_json_file_repository_save_create_dir_fails() {
        let tmp = TempDir::new().unwrap();
        let blocker = tmp.path().join("blocker");
        std::fs::write(&blocker, "I am a file").unwrap();

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

        let mut settings = AppSettings::default();
        settings.theme.preset = ThemePreset::Dracula;
        let repo = JsonFileRepository::new(path.clone());
        repo.save(&settings).unwrap();

        let loaded = repo.load();
        assert_eq!(loaded.theme.preset, ThemePreset::Dracula);
        assert!(loaded.theme.custom_color_overrides.is_none());
    }

    #[test]
    fn test_custom_color_overrides_save_and_restore() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        let mut settings = AppSettings::default();
        settings.theme.preset = ThemePreset::Nord;
        let mut custom = ThemePreset::Nord.colors();
        custom.system.background = Rgb {
            r: 10,
            g: 20,
            b: 30,
        };
        settings.theme.custom_color_overrides = Some(custom.clone());
        let repo = JsonFileRepository::new(path.clone());
        repo.save(&settings).unwrap();

        let loaded = repo.load();
        assert_eq!(loaded.theme.preset, ThemePreset::Nord);
        assert_eq!(loaded.theme.custom_color_overrides, Some(custom));
        assert_eq!(
            loaded.effective_theme_colors().system.background,
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
        let mut service = SettingsService::new(Box::new(InMemoryRepository));
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
        let repo = FirstLaunchRepo {
            preset: ThemePreset::KatanaDark,
        };
        let mut service = SettingsService::new(Box::new(repo));
        service.apply_os_default_theme();
        let preset = &service.settings().theme.preset;
        assert!(
            *preset == ThemePreset::KatanaDark || *preset == ThemePreset::KatanaLight,
            "first launch must yield KatanaDark or KatanaLight, got {preset:?}"
        );
    }

    #[test]
    fn test_select_preset_for_mode_dark() {
        assert_eq!(select_preset_for_mode(Some(true)), ThemePreset::KatanaDark);
    }

    #[test]
    fn test_select_preset_for_mode_light() {
        assert_eq!(
            select_preset_for_mode(Some(false)),
            ThemePreset::KatanaLight
        );
    }

    #[test]
    fn test_select_preset_for_mode_unknown() {
        assert_eq!(select_preset_for_mode(None), ThemePreset::KatanaDark);
    }

    #[test]
    fn test_first_launch_repo_save_is_noop() {
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

        service.apply_os_default_language(None);
        assert_eq!(service.settings().language, "ja");
    }

    #[test]
    fn test_apply_os_default_language_updates_on_first_launch() {
        let repo = FirstLaunchRepo {
            preset: ThemePreset::KatanaDark,
        };
        let mut service = SettingsService::new(Box::new(repo));

        service.apply_os_default_language(None);
        assert_eq!(service.settings().language, "en");

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
        let json = r#"{"updates": {"interval": "Daily"}}"#;
        let loaded: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.updates.skipped_version, None);
    }

    #[test]
    fn test_legacy_theme_colors_migration() {
        let json = r#"{
            "name": "Legacy Dark",
            "mode": "Dark",
            "background":             { "r": 30, "g": 30, "b": 30 },
            "panel_background":       { "r": 37, "g": 37, "b": 38 },
            "text":                   { "r": 212, "g": 212, "b": 212 },
            "text_secondary":         { "r": 180, "g": 180, "b": 180 },
            "accent":                 { "r": 86, "g": 156, "b": 214 },
            "title_bar_text":         { "r": 180, "g": 180, "b": 180 },
            "file_tree_text":         { "r": 220, "g": 220, "b": 220 },
            "active_file_highlight":  { "r": 40, "g": 80, "b": 160, "a": 100 },
            "warning_text":           { "r": 255, "g": 165, "b": 0 },
            "border":                 { "r": 60, "g": 60, "b": 60 },
            "selection":              { "r": 38, "g": 79, "b": 120 },
            "code_background":        { "r": 25, "g": 25, "b": 40 },
            "preview_background":     { "r": 35, "g": 35, "b": 50 }
        }"#;
        let colors: ThemeColors = serde_json::from_str(json).unwrap();
        assert_eq!(colors.name, "Legacy Dark");
        assert_eq!(colors.mode, ThemeMode::Dark);
        assert_eq!(
            colors.system.background,
            Rgb {
                r: 30,
                g: 30,
                b: 30
            }
        );
        assert_eq!(
            colors.code.background,
            Rgb {
                r: 25,
                g: 25,
                b: 40
            }
        );
        assert_eq!(
            colors.preview.background,
            Rgb {
                r: 35,
                g: 35,
                b: 50
            }
        );
    }

    #[test]
    fn test_legacy_theme_colors_migration_light() {
        let json = r#"{
            "name": "Legacy Light",
            "mode": "Light",
            "background":             { "r": 255, "g": 255, "b": 255 },
            "panel_background":       { "r": 240, "g": 240, "b": 240 },
            "text":                   { "r": 30, "g": 30, "b": 30 },
            "text_secondary":         { "r": 100, "g": 100, "b": 100 },
            "accent":                 { "r": 0, "g": 122, "b": 204 },
            "title_bar_text":         { "r": 50, "g": 50, "b": 50 },
            "file_tree_text":         { "r": 40, "g": 40, "b": 40 },
            "active_file_highlight":  { "r": 200, "g": 220, "b": 255, "a": 120 }
        }"#;
        let colors: ThemeColors = serde_json::from_str(json).unwrap();
        assert_eq!(colors.name, "Legacy Light");
        assert_eq!(colors.mode, ThemeMode::Light);
        assert_eq!(
            colors.system.success_text,
            Rgb {
                r: 20,
                g: 160,
                b: 20
            }
        );
        assert_eq!(
            colors.system.splash_background,
            Rgb {
                r: 240,
                g: 240,
                b: 240
            }
        );
        assert_eq!(
            colors.code.line_number_text,
            Rgb {
                r: 160,
                g: 160,
                b: 160
            }
        );
        assert_eq!(
            colors.system.warning_text,
            Rgb {
                r: 255,
                g: 140,
                b: 0
            }
        );
        assert_eq!(
            colors.code.background,
            Rgb {
                r: 30,
                g: 30,
                b: 30
            }
        );
        assert_eq!(
            colors.preview.background,
            Rgb {
                r: 35,
                g: 35,
                b: 35
            }
        );
    }

    #[test]
    fn test_new_format_theme_colors_roundtrip() {
        let preset = ThemePreset::KatanaDark;
        let original = preset.colors();
        let json = serde_json::to_string(&original).unwrap();
        let roundtripped: ThemeColors = serde_json::from_str(&json).unwrap();
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_theme_mode_to_theme_string() {
        assert_eq!(ThemeMode::Dark.to_theme_string(), "dark");
        assert_eq!(ThemeMode::Light.to_theme_string(), "light");
    }
}
