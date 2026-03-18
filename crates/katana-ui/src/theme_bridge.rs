//! Bridge between `ThemeColors` (katana-platform) and `egui::Visuals`.
//!
//! Converts a complete colour palette into egui's visual style so that the
//! entire application can be themed cohesively.

use eframe::egui;
use katana_platform::theme::{Rgb, Rgba, ThemeColors, ThemeMode};

// ── Stroke width constants ──
const STROKE_THIN: f32 = 0.5;
const STROKE_NORMAL: f32 = 1.0;
const STROKE_MEDIUM: f32 = 1.5;
const STROKE_BOLD: f32 = 2.0;

/// Converts a `ThemeColors` palette into an `egui::Visuals` struct.
pub fn visuals_from_theme(colors: &ThemeColors) -> egui::Visuals {
    let dark = colors.mode == ThemeMode::Dark;

    let bg = rgb_to_color32(colors.background);
    let panel_bg = rgb_to_color32(colors.panel_background);
    let text = rgb_to_color32(colors.text);
    let text_secondary = rgb_to_color32(colors.text_secondary);
    let accent = rgb_to_color32(colors.accent);
    let border = rgb_to_color32(colors.border);
    let selection_bg = rgb_to_color32(colors.selection);
    let highlight_bg = rgba_to_color32(colors.active_file_highlight);
    let code_bg = rgb_to_color32(colors.code_background);
    let warning = rgb_to_color32(colors.warning_text);

    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.override_text_color = Some(text);
    visuals.panel_fill = panel_bg;
    visuals.window_fill = bg;
    visuals.extreme_bg_color = code_bg;
    visuals.faint_bg_color = panel_bg;
    visuals.warn_fg_color = warning;

    visuals.selection.bg_fill = selection_bg;
    visuals.selection.stroke = egui::Stroke::new(STROKE_NORMAL, accent);

    visuals.widgets.noninteractive.bg_fill = panel_bg;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text_secondary);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(STROKE_THIN, border);

    visuals.widgets.inactive.bg_fill = panel_bg;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text_secondary);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(STROKE_THIN, border);

    visuals.widgets.hovered.bg_fill = highlight_bg;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(STROKE_MEDIUM, accent);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(STROKE_NORMAL, accent);

    visuals.widgets.active.bg_fill = accent;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(STROKE_BOLD, bg);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(STROKE_NORMAL, accent);

    visuals
}

/// Convert `Rgb` to `egui::Color32`.
pub fn rgb_to_color32(c: Rgb) -> egui::Color32 {
    egui::Color32::from_rgb(c.r, c.g, c.b)
}

/// Convert `Rgba` to `egui::Color32` (premultiplied alpha).
pub fn rgba_to_color32(c: Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
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
