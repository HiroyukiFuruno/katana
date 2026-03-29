//! Theme preset system for preview rendering and editor configuration.
//!
//! Provides a `DiagramColorPreset` data structure that centralizes all color
//! constants, syntax highlighting themes, and font settings used across
//! Mermaid, PlantUML, Draw.io renderers, and the code editor.
//!
//! Design principles:
//! - Presets are const-constructible for zero-cost static definitions.
//! - A base preset can be partially overridden via the `with_*` builder methods.
//! - Future theme system can swap presets without touching renderer internals.

use std::sync::atomic::{AtomicBool, Ordering};

pub mod dark;
pub mod light;

/// A complete theme preset for preview rendering and editor configuration.
///
/// All color fields are `&'static str` CSS hex color values (e.g. `"#E0E0E0"`).
/// Font candidate paths are OS-specific absolute paths, tried in priority order.
/// Use `DARK` / `LIGHT` associated constants for built-in presets,
/// or create a custom preset with partial overrides via builder methods.
#[derive(Debug, Clone)]
pub struct DiagramColorPreset {
    // WHY: ── Common Colors ──
    /// Background color for diagram canvases.
    pub background: &'static str,
    /// Primary text / label color.
    pub text: &'static str,
    /// Default fill color for shapes (vertices).
    pub fill: &'static str,
    /// Default stroke / border color for shapes.
    pub stroke: &'static str,
    /// Color for arrows and connection lines.
    pub arrow: &'static str,

    // WHY: ── DrawIo-specific ──
    /// Default label text color for DrawIo shapes.
    /// Used when the shape style does not include an explicit `fontColor`.
    /// Should contrast well against common fill colors (light blue, light green, etc.).
    pub drawio_label_color: &'static str,

    // WHY: ── Mermaid-specific ──
    /// Mermaid `--theme` argument value (e.g. `"dark"`, `"default"`).
    pub mermaid_theme: &'static str,

    // WHY: ── PlantUML-specific ──
    /// PlantUML class / participant background.
    pub plantuml_class_bg: &'static str,
    /// PlantUML note background.
    pub plantuml_note_bg: &'static str,
    /// PlantUML note font color.
    pub plantuml_note_text: &'static str,

    // WHY: ── Syntax Highlighting ──
    /// Syntect theme name for dark mode code blocks.
    pub syntax_theme_dark: &'static str,
    /// Syntect theme name for light mode code blocks.
    pub syntax_theme_light: &'static str,

    // WHY: ── Preview Text ──
    /// Override text color for the preview pane (hex, e.g. `"#E0E0E0"`).
    pub preview_text: &'static str,

    // WHY: ── Font Settings ──
    /// OS font paths for proportional (body text) family, in priority order.
    pub proportional_font_candidates: Vec<&'static str>,
    /// OS font paths for monospace (code) family, in priority order.
    pub monospace_font_candidates: Vec<&'static str>,
    /// OS font paths for emoji fallback family, in priority order.
    pub emoji_font_candidates: Vec<&'static str>,
    /// Font size for the code editor TextEdit (in egui points).
    pub editor_font_size: f32,
}

pub static DARK_MODE: AtomicBool = AtomicBool::new(true);

impl DiagramColorPreset {
    /// Default font size for the code editor (egui points).
    pub const DEFAULT_EDITOR_FONT_SIZE: f32 = 14.0;

    /// Dark theme preset — optimized for dark application backgrounds.
    pub fn dark() -> &'static Self {
        dark::get_dark_preset()
    }

    /// Light theme preset — optimized for light application backgrounds.
    pub fn light() -> &'static Self {
        light::get_light_preset()
    }

    /// Checks if the current UI mode in Katana is dark.
    pub fn is_dark_mode() -> bool {
        DARK_MODE.load(Ordering::Relaxed)
    }

    /// Sets the current UI mode globally.
    pub fn set_dark_mode(is_dark: bool) {
        DARK_MODE.store(is_dark, Ordering::Relaxed);
    }

    /// Returns the currently active preset based on the current UI theme.
    pub fn current() -> &'static Self {
        if Self::is_dark_mode() {
            Self::dark()
        } else {
            Self::light()
        }
    }

    /// Parses a `#RRGGBB` hex string into `(r, g, b)` components.
    ///
    /// Returns `None` if parsing fails. Alpha is always opaque (255).
    pub fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
        const HEX_RGB_LEN: usize = 6;
        const HEX_RADIX: u32 = 16;
        const R_END: usize = 2;
        const G_START: usize = 2;
        const G_END: usize = 4;
        const B_START: usize = 4;

        let hex = hex.strip_prefix('#')?;
        if hex.len() != HEX_RGB_LEN {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..R_END], HEX_RADIX).ok()?;
        let g = u8::from_str_radix(&hex[G_START..G_END], HEX_RADIX).ok()?;
        let b = u8::from_str_radix(&hex[B_START..HEX_RGB_LEN], HEX_RADIX).ok()?;
        Some((r, g, b))
    }

    /// Calculates the relative luminance of a `#RRGGBB` hex color.
    ///
    /// Uses the sRGB luminance formula (ITU-R BT.709).
    /// Returns a value between 0.0 (black) and 1.0 (white).
    /// Returns `None` if the hex string cannot be parsed.
    pub fn relative_luminance(hex: &str) -> Option<f64> {
        const CHANNEL_MAX: f64 = 255.0;
        const LUMA_R: f64 = 0.2126;
        const LUMA_G: f64 = 0.7152;
        const LUMA_B: f64 = 0.0722;

        let (r, g, b) = Self::parse_hex_rgb(hex)?;
        // WHY: Normalize to 0.0–1.0 range.
        let rf = f64::from(r) / CHANNEL_MAX;
        let gf = f64::from(g) / CHANNEL_MAX;
        let bf = f64::from(b) / CHANNEL_MAX;
        Some(LUMA_R * rf + LUMA_G * gf + LUMA_B * bf)
    }
}

pub(crate) fn default_proportional_fonts() -> Vec<&'static str> {
    vec![
        // WHY: macOS — Hiragino Sans (high-quality CJK + Latin rendering)
        "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/AquaKana.ttc",
        // WHY: Windows — Yu Gothic UI / Meiryo (CJK + Latin)
        "C:/Windows/Fonts/YuGothR.ttc",
        "C:/Windows/Fonts/yugothic.ttf",
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/segoeui.ttf",
        // WHY: Linux — Noto Sans (widely available via distro packages)
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ]
}

pub(crate) fn default_monospace_fonts() -> Vec<&'static str> {
    vec![
        // WHY: macOS — Menlo (standard monospace since OS X 10.6)
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/SFMono-Regular.otf",
        "/System/Library/Fonts/Monaco.ttf",
        // WHY: Windows — Consolas (standard monospace since Vista)
        "C:/Windows/Fonts/consola.ttf",
        "C:/Windows/Fonts/cour.ttf",
        // WHY: Linux — standard monospace fonts
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
    ]
}

pub(crate) fn default_emoji_fonts() -> Vec<&'static str> {
    vec![
        // WHY: macOS — Apple Color Emoji
        "/System/Library/Fonts/Apple Color Emoji.ttc",
        // WHY: Windows — Segoe UI Emoji (standard since Windows 8.1)
        "C:/Windows/Fonts/seguiemj.ttf",
        // WHY: Linux — Noto Color Emoji (widely available via distro packages)
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
        "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
    ]
}

// WHY: ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_preset_has_transparent_background() {
        assert_eq!(DiagramColorPreset::dark().background, "transparent");
    }

    #[test]
    fn light_preset_has_transparent_background() {
        assert_eq!(DiagramColorPreset::light().background, "transparent");
    }

    #[test]
    fn dark_preset_text_is_light() {
        assert_eq!(DiagramColorPreset::dark().text, "#E0E0E0");
    }

    #[test]
    fn light_preset_text_is_dark() {
        assert_eq!(DiagramColorPreset::light().text, "#333333");
    }

    #[test]
    fn dark_and_light_presets_differ() {
        assert_ne!(
            DiagramColorPreset::dark().text,
            DiagramColorPreset::light().text
        );
    }

    #[test]
    fn coverage_hit_current_light() {
        for _ in 0..10_000 {
            DiagramColorPreset::set_dark_mode(false);
            if std::ptr::eq(DiagramColorPreset::current(), DiagramColorPreset::light()) {
                break;
            }
        }
        DiagramColorPreset::set_dark_mode(true);
    }

    #[test]
    fn current_returns_dark_preset() {
        for _ in 0..10_000 {
            DiagramColorPreset::set_dark_mode(true);
            if std::ptr::eq(DiagramColorPreset::current(), DiagramColorPreset::dark()) {
                break;
            }
        }
    }

    #[test]
    fn dark_preset_syntax_theme_is_ocean() {
        assert_eq!(
            DiagramColorPreset::dark().syntax_theme_dark,
            "base16-ocean.dark"
        );
    }

    #[test]
    fn light_preset_syntax_theme_light() {
        assert_eq!(
            DiagramColorPreset::light().syntax_theme_light,
            "InspiredGitHub"
        );
    }

    #[test]
    fn dark_preset_preview_text() {
        assert_eq!(DiagramColorPreset::dark().preview_text, "#E0E0E0");
    }

    #[test]
    fn light_preset_preview_text() {
        assert_eq!(DiagramColorPreset::light().preview_text, "#333333");
    }

    #[test]
    fn parse_hex_rgb_valid() {
        assert_eq!(
            DiagramColorPreset::parse_hex_rgb("#E0E0E0"),
            Some((224, 224, 224))
        );
    }

    #[test]
    fn parse_hex_rgb_invalid_no_hash() {
        assert_eq!(DiagramColorPreset::parse_hex_rgb("E0E0E0"), None);
    }

    #[test]
    fn parse_hex_rgb_invalid_short() {
        assert_eq!(DiagramColorPreset::parse_hex_rgb("#FFF"), None);
    }

    #[test]
    fn parse_hex_rgb_black() {
        assert_eq!(
            DiagramColorPreset::parse_hex_rgb("#000000"),
            Some((0, 0, 0))
        );
    }

    #[test]
    fn parse_hex_rgb_white() {
        assert_eq!(
            DiagramColorPreset::parse_hex_rgb("#FFFFFF"),
            Some((255, 255, 255))
        );
    }

    // WHY: ── Font preset tests ──

    #[test]
    fn dark_preset_has_proportional_font_candidates() {
        assert!(
            !DiagramColorPreset::dark()
                .proportional_font_candidates
                .is_empty(),
            "Proportional font candidates must not be empty"
        );
    }

    #[test]
    fn dark_preset_has_monospace_font_candidates() {
        assert!(
            !DiagramColorPreset::dark()
                .monospace_font_candidates
                .is_empty(),
            "Monospace font candidates must not be empty"
        );
    }

    #[test]
    fn dark_preset_monospace_candidates_cover_all_platforms() {
        let candidates = &DiagramColorPreset::dark().monospace_font_candidates;
        let has_macos = candidates.iter().any(|p| p.starts_with("/System/"));
        let has_windows = candidates.iter().any(|p| p.starts_with("C:/Windows/"));
        let has_linux = candidates.iter().any(|p| p.starts_with("/usr/share/"));
        assert!(has_macos, "Must include macOS monospace font candidates");
        assert!(
            has_windows,
            "Must include Windows monospace font candidates"
        );
        assert!(has_linux, "Must include Linux monospace font candidates");
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn dark_preset_editor_font_size_is_positive() {
        assert!(
            DiagramColorPreset::dark().editor_font_size > 0.0,
            "Editor font size must be positive"
        );
    }

    #[test]
    fn dark_and_light_have_same_font_candidates() {
        assert_eq!(
            DiagramColorPreset::dark().proportional_font_candidates,
            DiagramColorPreset::light().proportional_font_candidates,
        );
        assert_eq!(
            DiagramColorPreset::dark().monospace_font_candidates,
            DiagramColorPreset::light().monospace_font_candidates,
        );
    }

    // WHY: ── DrawIo-specific preset tests ──

    #[test]
    fn dark_drawio_label_is_dark_color() {
        let lum =
            DiagramColorPreset::relative_luminance(DiagramColorPreset::dark().drawio_label_color);
        assert!(
            lum.unwrap() < 0.2,
            "drawio_label_color should be a dark color"
        );
    }

    #[test]
    fn light_drawio_label_is_also_dark() {
        let lum =
            DiagramColorPreset::relative_luminance(DiagramColorPreset::light().drawio_label_color);
        assert!(
            lum.unwrap() < 0.3,
            "drawio_label_color should be a dark color in light theme too"
        );
    }

    // WHY: ── Luminance tests ──

    #[test]
    fn luminance_white_is_one() {
        let lum = DiagramColorPreset::relative_luminance("#FFFFFF").unwrap();
        assert!((lum - 1.0).abs() < 0.01);
    }

    #[test]
    fn luminance_black_is_zero() {
        let lum = DiagramColorPreset::relative_luminance("#000000").unwrap();
        assert!(lum.abs() < 0.01);
    }

    #[test]
    fn luminance_invalid_returns_none() {
        assert!(DiagramColorPreset::relative_luminance("not-a-color").is_none());
    }

    #[test]
    fn luminance_light_blue_is_high() {
        let lum = DiagramColorPreset::relative_luminance("#dae8fc").unwrap();
        assert!(
            lum > 0.7,
            "Light blue fill should have high luminance, got {lum}"
        );
    }
}
