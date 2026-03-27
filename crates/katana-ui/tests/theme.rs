//! Integration tests for theme switching.
//!
//! Verifies that changing the selected preset in AppSettings is correctly
//! reflected in the effective theme colours and egui Visuals.

use katana_platform::theme::{Rgb, ThemeMode, ThemePreset};
use katana_platform::{InMemoryRepository, SettingsService};
use katana_ui::theme_bridge::visuals_from_theme;

/// Switching theme preset correctly changes effective_theme_colors
#[test]
fn switching_preset_changes_effective_colors() {
    let mut svc = SettingsService::new(Box::new(InMemoryRepository));

    // Default is KatanaDark
    let default_colors = svc.settings().effective_theme_colors();
    assert_eq!(default_colors.mode, ThemeMode::Dark);

    // Switch to Dracula
    svc.settings_mut().theme.preset = ThemePreset::Dracula;
    let dracula_colors = svc.settings().effective_theme_colors();
    assert_eq!(dracula_colors.name, "Dracula");
    assert_eq!(dracula_colors.mode, ThemeMode::Dark);
    assert_ne!(
        dracula_colors.system.background,
        default_colors.system.background
    );

    // Switch to KatanaLight
    svc.settings_mut().theme.preset = ThemePreset::KatanaLight;
    let light_colors = svc.settings().effective_theme_colors();
    assert_eq!(light_colors.mode, ThemeMode::Light);
}

/// Theme switching is correctly reflected in egui::Visuals
#[test]
fn switching_preset_changes_visuals() {
    let mut svc = SettingsService::new(Box::new(InMemoryRepository));

    // Dark → visuals.dark_mode == true
    let dark_visuals = visuals_from_theme(&svc.settings().effective_theme_colors());
    assert!(dark_visuals.dark_mode);

    // Switch to Light → visuals.dark_mode == false
    svc.settings_mut().theme.preset = ThemePreset::GitHubLight;
    let light_visuals = visuals_from_theme(&svc.settings().effective_theme_colors());
    assert!(!light_visuals.dark_mode);

    // panel_fill should follow the theme
    let expected_panel = ThemePreset::GitHubLight.colors().system.panel_background;
    assert_eq!(
        light_visuals.panel_fill,
        eframe::egui::Color32::from_rgb(expected_panel.r, expected_panel.g, expected_panel.b)
    );
}

/// Custom colour overrides take precedence over preset
#[test]
fn custom_overrides_take_precedence_over_preset() {
    let mut svc = SettingsService::new(Box::new(InMemoryRepository));

    // Apply customisation on top of a preset
    svc.settings_mut().theme.preset = ThemePreset::Nord;
    let mut custom = ThemePreset::Nord.colors();
    custom.system.background = Rgb { r: 1, g: 2, b: 3 };
    svc.settings_mut().theme.custom_color_overrides = Some(custom.clone());

    let effective = svc.settings().effective_theme_colors();
    assert_eq!(effective.system.background, Rgb { r: 1, g: 2, b: 3 });
    // Other fields should remain unchanged from Nord
    assert_eq!(
        effective.system.accent,
        ThemePreset::Nord.colors().system.accent
    );
}

/// All presets switch correctly
#[test]
fn all_presets_switch_correctly() {
    let mut svc = SettingsService::new(Box::new(InMemoryRepository));

    for preset in ThemePreset::builtins() {
        svc.settings_mut().theme.preset = preset;
        svc.settings_mut().theme.custom_color_overrides = None;

        let colors = svc.settings().effective_theme_colors();
        assert_eq!(
            colors.name,
            preset.display_name(),
            "Preset: {}",
            preset.display_name()
        );

        let visuals = visuals_from_theme(&colors);
        let expected_dark = colors.mode == ThemeMode::Dark;
        assert_eq!(
            visuals.dark_mode,
            expected_dark,
            "Preset: {}",
            preset.display_name()
        );
    }
}
