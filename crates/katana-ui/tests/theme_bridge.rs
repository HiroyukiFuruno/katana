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
    assert_eq!(
        visuals.panel_fill,
        rgb_to_color32(colors.system.panel_background)
    );
}

#[test]
fn text_color_override_is_not_set() {
    // override_text_color is intentionally None.
    // system.text is applied via widget fg_stroke,
    // and preview/code paths read their own text colour from ThemeColors.
    let colors = ThemePreset::Dracula.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(visuals.override_text_color, None);
    // system.text is the primary text colour applied to noninteractive widget fg_stroke
    assert_eq!(
        visuals.widgets.noninteractive.fg_stroke.color,
        rgb_to_color32(colors.system.text)
    );
}

#[test]
fn all_presets_produce_valid_visuals() {
    for preset in ThemePreset::builtins() {
        let colors = preset.colors();
        let visuals = visuals_from_theme(&colors);
        // override_text_color must be None (prevents scope contamination)
        assert_eq!(
            visuals.override_text_color,
            None,
            "{}: override_text_color must be None",
            preset.display_name()
        );
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
        eframe::egui::Color32::from_rgba_unmultiplied(40, 80, 160, 100)
    );
}

#[test]
fn warn_fg_color_matches_warning_text() {
    let colors = ThemePreset::Monokai.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(
        visuals.warn_fg_color,
        rgb_to_color32(colors.system.warning_text)
    );
}

#[test]
fn selection_bg_uses_selection_color() {
    let colors = ThemePreset::Dracula.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(
        visuals.selection.bg_fill,
        rgb_to_color32(colors.system.selection)
    );
}

#[test]
fn hovered_bg_uses_active_file_highlight() {
    let colors = ThemePreset::Dracula.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(
        visuals.widgets.hovered.bg_fill,
        rgba_to_color32(colors.system.active_file_highlight)
    );
}

#[test]
fn noninteractive_fg_uses_primary_text() {
    let colors = ThemePreset::Nord.colors();
    let visuals = visuals_from_theme(&colors);
    assert_eq!(
        visuals.widgets.noninteractive.fg_stroke.color,
        rgb_to_color32(colors.system.text)
    );
}
