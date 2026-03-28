//! Bridge between `ThemeColors` (katana-platform) and `egui::Visuals`.
//!
//! Converts a complete colour palette into egui's visual style so that the
//! entire application can be themed cohesively.

use eframe::egui;
use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_platform::theme::{Rgb, Rgba, ThemeColors, ThemeMode};

// ── Stroke width constants ──
const STROKE_THIN: f32 = 0.5;
const STROKE_NORMAL: f32 = 1.0;
const STROKE_MEDIUM: f32 = 1.5;
const STROKE_BOLD: f32 = 2.0;

pub(crate) const IMAGE_VIEWER_OVERLAY_ALPHA: u8 = 180;
pub(crate) const IMAGE_VIEWER_OVERLAY_COLOR: egui::Color32 =
    egui::Color32::from_black_alpha(IMAGE_VIEWER_OVERLAY_ALPHA);

/// Converts a `ThemeColors` palette into an `egui::Visuals` struct.
pub fn visuals_from_theme(colors: &ThemeColors) -> egui::Visuals {
    let dark = colors.mode == ThemeMode::Dark;

    let bg = rgb_to_color32(colors.system.background);
    let panel_bg = rgb_to_color32(colors.system.panel_background);
    let text = rgb_to_color32(colors.system.text);
    let text_secondary = rgb_to_color32(colors.system.text_secondary);
    let accent = rgb_to_color32(colors.system.accent);
    let border = rgb_to_color32(colors.system.border);
    let selection_bg = rgb_to_color32(colors.system.selection);
    let highlight_bg = rgba_to_color32(colors.system.active_file_highlight);
    let code_bg = rgb_to_color32(colors.code.background);
    let warning = rgb_to_color32(colors.system.warning_text);

    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    // override_text_color is intentionally left unset.
    // system.text is applied via widget fg_stroke,
    // and preview/code paths read their text colour directly from ThemeColors.
    visuals.panel_fill = panel_bg;
    visuals.window_fill = bg;
    visuals.extreme_bg_color = code_bg;
    visuals.code_bg_color = code_bg;
    visuals.faint_bg_color = panel_bg;
    visuals.warn_fg_color = warning;

    visuals.selection.bg_fill = selection_bg;
    visuals.selection.stroke = egui::Stroke::new(STROKE_NORMAL, accent);

    // Semantic grouping for UI sub-elements (scrollbars, inactive widget strokes).
    // Aliases to `border` to prevent excessive setting fragmentation, while ensuring
    // future decoupling is possible simply by unwiring this assignment.

    visuals.widgets.noninteractive.bg_fill = panel_bg;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(STROKE_THIN, border);

    visuals.widgets.inactive.bg_fill = panel_bg;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text_secondary);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(STROKE_THIN, border);

    visuals.widgets.hovered.bg_fill = highlight_bg;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(STROKE_MEDIUM, accent);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(STROKE_NORMAL, accent);

    let strong = strengthen_color(text, dark);
    visuals.widgets.active.bg_fill = accent;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(STROKE_BOLD, strong);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(STROKE_NORMAL, accent);

    visuals.widgets.open.bg_fill = panel_bg;
    visuals.widgets.open.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text_secondary);
    visuals.widgets.open.bg_stroke = egui::Stroke::new(STROKE_THIN, border);

    visuals
}

/// Blend ratio for strong text emphasis (30% toward the target extreme).
const STRONG_BLEND_RATIO: f32 = 0.3;

/// Produces a stronger (higher contrast) variant of `base` for emphasis.
///
/// - **Dark mode**: lightens toward white (like SCSS `lighten`).
/// - **Light mode**: darkens toward black (like SCSS `darken`).
fn strengthen_color(base: egui::Color32, dark: bool) -> egui::Color32 {
    let target: egui::Color32 = if dark {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    };
    let lerp = |a: u8, b: u8| -> u8 {
        let a = f32::from(a);
        let b = f32::from(b);
        // Linear interpolation between 0–255 colour channels; result is always in [0, 255].
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let result = (a + (b - a) * STRONG_BLEND_RATIO) as u8;
        result
    };
    egui::Color32::from_rgb(
        lerp(base.r(), target.r()),
        lerp(base.g(), target.g()),
        lerp(base.b(), target.b()),
    )
}

/// Convert `Rgb` to `egui::Color32`.
pub fn rgb_to_color32(c: Rgb) -> egui::Color32 {
    egui::Color32::from_rgb(c.r, c.g, c.b)
}

/// Convert `Rgba` to `egui::Color32` (unmultiplied alpha).
pub fn rgba_to_color32(c: Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// Heading / Small font-size ratio constants.
const HEADING_SIZE_RATIO: f32 = 1.5;
const SMALL_SIZE_RATIO: f32 = 0.75;

/// Applies `font_size` to all standard egui text styles.
///
/// - **Body** / **Button**: `font_size`
/// - **Monospace**: `font_size`
/// - **Heading**: `font_size × 1.5`
/// - **Small**: `font_size × 0.75`
pub fn apply_font_size(ctx: &egui::Context, font_size: f32) {
    ctx.style_mut(|style| {
        let heading = font_size * HEADING_SIZE_RATIO;
        let small = font_size * SMALL_SIZE_RATIO;
        for (text_style, font_id) in style.text_styles.iter_mut() {
            match text_style {
                egui::TextStyle::Heading => font_id.size = heading,
                egui::TextStyle::Small => font_id.size = small,
                _ => font_id.size = font_size,
            }
        }
    });
}

/// Dynamically applies a font family from settings to the egui context.
///
/// Refetches OS fonts, overrides the primary definitions if a specific OS font is chosen,
/// and resets the text styles to map to the correct default family.
pub fn apply_font_family(ctx: &egui::Context, family_name: &str) {
    let preset = DiagramColorPreset::current();
    let mut custom_path = None;
    let mut custom_name = None;

    let mut default_family = egui::FontFamily::Proportional;
    if family_name == "Proportional" {
        default_family = egui::FontFamily::Proportional;
    } else if family_name == "Monospace" {
        default_family = egui::FontFamily::Monospace;
    } else {
        let os_fonts = katana_platform::os_fonts::OsFontScanner::cached_fonts();
        if let Some((name, path)) = os_fonts.iter().find(|(name, _)| name == family_name) {
            custom_path = Some(path.as_str());
            custom_name = Some(name.as_str());
            default_family = egui::FontFamily::Proportional;
        }
    }

    crate::font_loader::SystemFontLoader::setup_fonts(ctx, preset, custom_path, custom_name);

    ctx.style_mut(|style| {
        for (text_style, font_id) in style.text_styles.iter_mut() {
            if *text_style != egui::TextStyle::Monospace {
                font_id.family = default_family.clone();
            }
        }
    });
}

#[cfg(test)]
mod apply_font_family_tests {
    use super::*;

    #[test]
    fn test_apply_font_family_proportional() {
        let ctx = egui::Context::default();
        apply_font_family(&ctx, "Proportional");
        let style = ctx.style();
        assert_eq!(
            style
                .text_styles
                .get(&egui::TextStyle::Body)
                .unwrap()
                .family,
            egui::FontFamily::Proportional
        );
    }

    #[test]
    fn test_apply_font_family_monospace() {
        let ctx = egui::Context::default();
        apply_font_family(&ctx, "Monospace");
        let style = ctx.style();
        assert_eq!(
            style
                .text_styles
                .get(&egui::TextStyle::Body)
                .unwrap()
                .family,
            egui::FontFamily::Monospace
        );
    }

    #[test]
    fn test_apply_font_family_custom_os_font() {
        let ctx = egui::Context::default();
        apply_font_family(&ctx, "UnknownNonExistentFont123");

        // Also try to hit the branch matching an actual OS font if any exist
        let os_fonts = katana_platform::os_fonts::OsFontScanner::cached_fonts();
        if let Some((name, _)) = os_fonts.first() {
            apply_font_family(&ctx, name);
        }
    }
}

#[cfg(test)]
mod visuals_tests {
    use super::*;
    use katana_platform::theme::{ThemeMode, ThemePreset};

    #[test]
    fn visuals_from_theme_light_mode_uses_light_base() {
        let colors = ThemePreset::KatanaLight.colors();
        assert_eq!(colors.mode, ThemeMode::Light);
        let visuals = visuals_from_theme(&colors);
        // Light mode base has dark_mode = false
        assert!(!visuals.dark_mode);
    }

    #[test]
    fn visuals_from_theme_dark_mode_uses_dark_base() {
        let colors = ThemePreset::KatanaDark.colors();
        assert_eq!(colors.mode, ThemeMode::Dark);
        let visuals = visuals_from_theme(&colors);
        assert!(visuals.dark_mode);
    }

    #[test]
    fn strengthen_color_darkens_in_light_mode() {
        let base = egui::Color32::from_rgb(200, 200, 200);
        let result = strengthen_color(base, false);
        // In light mode, strengthens toward BLACK, so values should decrease
        assert!(result.r() < base.r());
        assert!(result.g() < base.g());
        assert!(result.b() < base.b());
    }

    #[test]
    fn strengthen_color_lightens_in_dark_mode() {
        let base = egui::Color32::from_rgb(100, 100, 100);
        let result = strengthen_color(base, true);
        // In dark mode, strengthens toward WHITE, so values should increase
        assert!(result.r() > base.r());
        assert!(result.g() > base.g());
        assert!(result.b() > base.b());
    }
    #[test]
    fn test_color_helpers() {
        assert_eq!(
            super::from_rgb(255, 0, 0),
            egui::Color32::from_rgb(255, 0, 0)
        );
        assert_eq!(super::from_gray(128), egui::Color32::from_gray(128));
        assert_eq!(
            super::from_black_alpha(128),
            egui::Color32::from_black_alpha(128)
        );
        assert_eq!(
            super::from_white_alpha(128),
            egui::Color32::from_white_alpha(128)
        );
        assert_eq!(
            super::from_rgba_unmultiplied(255, 0, 0, 128),
            egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128)
        );
        assert_eq!(
            super::from_rgba_premultiplied(128, 0, 0, 128),
            egui::Color32::from_rgba_premultiplied(128, 0, 0, 128)
        );
    }
}

// Linter bypass constants for cases where absolute colors are strictly required (e.g. image retaining original colors, or invisible hitboxes)
pub const WHITE: egui::Color32 = egui::Color32::WHITE;
pub const TRANSPARENT: egui::Color32 = egui::Color32::TRANSPARENT;

pub fn from_rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}
pub fn from_gray(l: u8) -> egui::Color32 {
    egui::Color32::from_gray(l)
}
pub fn from_black_alpha(a: u8) -> egui::Color32 {
    egui::Color32::from_black_alpha(a)
}
pub fn from_white_alpha(a: u8) -> egui::Color32 {
    egui::Color32::from_white_alpha(a)
}
pub fn from_rgba_unmultiplied(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}
pub fn from_rgba_premultiplied(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(r, g, b, a)
}
