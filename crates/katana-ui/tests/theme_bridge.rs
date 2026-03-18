//! Unit tests for theme_bridge module (visuals_from_theme, rgb_to_color32, rgba_to_color32).

use katana_platform::theme::{Rgb, Rgba, ThemeMode, ThemePreset};
use katana_ui::theme_bridge::{rgb_to_color32, rgba_to_color32, visuals_from_theme};

#[test]
fn dark_preset_produces_dark_visuals() {
    let colors = ThemePreset::KatanaDark.colors();
    let visuals = visuals_from_theme(&colors);
    assert!(visuals.dark_mode);
}

#[test]
fn light_preset_produces_light_visuals() {
    let colors = ThemePreset::KatanaLight.colors();
    let visuals = visuals_from_theme(&colors);
    assert!(!visuals.dark_mode);
}

#[test]
fn panel_fill_matches_theme_panel_bg() {
    let colors = ThemePreset::Nord.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(visuals.panel_fill, rgb_to_color32(colors.panel_background));
}

#[test]
fn text_color_override_is_set() {
    let colors = ThemePreset::Dracula.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(
        visuals.override_text_color,
        Some(rgb_to_color32(colors.text))
    );
}

#[test]
fn all_presets_produce_valid_visuals() {
    for preset in ThemePreset::all() {
        let colors = preset.colors();
        let visuals = visuals_from_theme(&colors);
        assert!(visuals.override_text_color.is_some());
        let is_dark = colors.mode == ThemeMode::Dark;
        assert_eq!(visuals.dark_mode, is_dark, "{}", preset.display_name());
    }
}

#[test]
fn rgb_to_color32_converts_correctly() {
    let c = Rgb {
        r: 255,
        g: 128,
        b: 0,
    };
    assert_eq!(
        rgb_to_color32(c),
        eframe::egui::Color32::from_rgb(255, 128, 0)
    );
}

#[test]
fn rgba_to_color32_converts_correctly() {
    let c = Rgba {
        r: 40,
        g: 80,
        b: 160,
        a: 100,
    };
    assert_eq!(
        rgba_to_color32(c),
        eframe::egui::Color32::from_rgba_premultiplied(40, 80, 160, 100)
    );
}

#[test]
fn warn_fg_color_matches_warning_text() {
    let colors = ThemePreset::Monokai.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(visuals.warn_fg_color, rgb_to_color32(colors.warning_text));
}

#[test]
fn selection_bg_uses_selection_color() {
    let colors = ThemePreset::Dracula.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(visuals.selection.bg_fill, rgb_to_color32(colors.selection));
}

#[test]
fn hovered_bg_uses_active_file_highlight() {
    let colors = ThemePreset::Dracula.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(
        visuals.widgets.hovered.bg_fill,
        rgba_to_color32(colors.active_file_highlight)
    );
}

#[test]
fn noninteractive_fg_uses_text_secondary() {
    let colors = ThemePreset::Nord.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(
        visuals.widgets.noninteractive.fg_stroke.color,
        rgb_to_color32(colors.text_secondary)
    );
}
