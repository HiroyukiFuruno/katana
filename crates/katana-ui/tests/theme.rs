
use katana_platform::theme::{Rgb, ThemeMode, ThemePreset};
use katana_platform::{InMemoryRepository, SettingsService};
use katana_ui::theme_bridge::visuals_from_theme;

#[test]
fn switching_preset_changes_effective_colors() {
    let mut svc = SettingsService::new(Box::new(InMemoryRepository));

    let default_colors = svc.settings().effective_theme_colors();
    assert_eq!(default_colors.mode, ThemeMode::Dark);

    svc.settings_mut().theme.preset = ThemePreset::Dracula;
    let dracula_colors = svc.settings().effective_theme_colors();
    assert_eq!(dracula_colors.name, "Dracula");
    assert_eq!(dracula_colors.mode, ThemeMode::Dark);
    assert_ne!(
        dracula_colors.system.background,
        default_colors.system.background
    );

    svc.settings_mut().theme.preset = ThemePreset::KatanaLight;
    let light_colors = svc.settings().effective_theme_colors();
    assert_eq!(light_colors.mode, ThemeMode::Light);
}

#[test]
fn switching_preset_changes_visuals() {
    let mut svc = SettingsService::new(Box::new(InMemoryRepository));

    let dark_visuals = visuals_from_theme(&svc.settings().effective_theme_colors());
    assert!(dark_visuals.dark_mode);

    svc.settings_mut().theme.preset = ThemePreset::GitHubLight;
    let light_visuals = visuals_from_theme(&svc.settings().effective_theme_colors());
    assert!(!light_visuals.dark_mode);

    let expected_panel = ThemePreset::GitHubLight.colors().system.panel_background;
    assert_eq!(
        light_visuals.panel_fill,
        eframe::egui::Color32::from_rgb(expected_panel.r, expected_panel.g, expected_panel.b)
    );
}

#[test]
fn custom_overrides_take_precedence_over_preset() {
    let mut svc = SettingsService::new(Box::new(InMemoryRepository));

    svc.settings_mut().theme.preset = ThemePreset::Nord;
    let mut custom = ThemePreset::Nord.colors();
    custom.system.background = Rgb { r: 1, g: 2, b: 3 };
    svc.settings_mut().theme.custom_color_overrides = Some(custom.clone());

    let effective = svc.settings().effective_theme_colors();
    assert_eq!(effective.system.background, Rgb { r: 1, g: 2, b: 3 });
    assert_eq!(
        effective.system.accent,
        ThemePreset::Nord.colors().system.accent
    );
}

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