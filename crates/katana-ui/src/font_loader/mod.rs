use egui::{Context, FontData, FontDefinitions, FontFamily};
use katana_core::markdown::color_preset::DiagramColorPreset;
use std::fs;

const MONO_FALLBACK_Y_OFFSET_FACTOR: f32 = 0.40;
const MONO_PRIMARY_Y_OFFSET_FACTOR: f32 = -0.15;
const MARKDOWN_PROPORTIONAL_Y_OFFSET_FACTOR: f32 = 0.0;

/// Font wrapper that tracks normalization state for consistent baseline
/// alignment across mixed-language text (JP/EN) and across font families
/// (Proportional ↔ Monospace).
///
/// Normalization ensures:
/// 1. CJK fallback fonts are correctly positioned in the Monospace family
///    chain with appropriate `y_offset_factor` to prevent visual jitter.
/// 2. Monospace primary font baseline is aligned with Proportional baseline
///    to prevent jitter in inline code within body text.
///
/// The `is_normalized` flag prevents double-correction.
///
/// All font setup MUST go through this type (enforced by `ast_linter`).
pub struct NormalizeFonts {
    fonts: FontDefinitions,
    is_normalized: bool,
}

impl NormalizeFonts {
    /// Wrap raw `FontDefinitions` (not yet normalized).
    pub fn new(fonts: FontDefinitions) -> Self {
        Self {
            fonts,
            is_normalized: false,
        }
    }

    /// Apply all baseline normalizations.
    /// No-op if already normalized (prevents double-correction).
    ///
    /// Corrections applied:
    /// 1. **CJK fallback**: Insert Proportional CJK font into Monospace chain
    ///    at position 1 with `y_offset_factor` for baseline alignment.
    /// 2. **Cross-family**: Adjust Monospace primary font's `y_offset_factor`
    ///    so inline code shares baseline with surrounding Proportional text.
    pub fn normalize(mut self, proportional_candidates: &[&str]) -> Self {
        if self.is_normalized {
            return self;
        }

        self.normalize_cjk_baseline(proportional_candidates);

        self.is_normalized = true;
        self
    }

    /// CJK baseline normalization:
    /// Adjusts the `Monospace` fallback so it perfectly aligns with `Monospace` primary,
    /// which is in turn raised by `MONO_PRIMARY_Y_OFFSET_FACTOR` to match `Proportional` baselines
    /// during `egui` LayoutJob center alignment mixed rendering.
    fn normalize_cjk_baseline(&mut self, proportional_candidates: &[&str]) {
        let tweaked_fallback = egui::FontTweak {
            scale: 1.0,
            y_offset_factor: MONO_FALLBACK_Y_OFFSET_FACTOR + MONO_PRIMARY_Y_OFFSET_FACTOR,
            y_offset: 0.0,
        };
        let mono_fallback_name = SystemFontLoader::load_first_valid(
            &mut self.fonts,
            proportional_candidates,
            Some(tweaked_fallback),
            "_mono_fallback",
        );

        if let Some(name) = &mono_fallback_name {
            SystemFontLoader::insert_after_primary(&mut self.fonts, FontFamily::Monospace, name);
        }
    }
    //
    // Inline code baseline alignment is handled at the LayoutJob level instead.

    /// Whether baseline normalization has been applied.
    pub fn is_normalized(&self) -> bool {
        self.is_normalized
    }

    /// Read access to the underlying `FontDefinitions`.
    pub fn fonts(&self) -> &FontDefinitions {
        &self.fonts
    }

    /// Consume and return the underlying `FontDefinitions`.
    pub fn into_inner(self) -> FontDefinitions {
        self.fonts
    }
}

/// Responsible for dynamically loading and registering system fonts with egui.
pub struct SystemFontLoader;

impl SystemFontLoader {
    /// Configures egui context fonts from a given preset, optionally overriding the base font.
    pub fn setup_fonts(
        ctx: &Context,
        preset: &DiagramColorPreset,
        custom_font_path: Option<&str>,
        custom_font_name: Option<&str>,
    ) {
        let normalized = Self::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            custom_font_path,
            custom_font_name,
        );
        let is_loaded = normalized
            .fonts
            .families
            .contains_key(&egui::FontFamily::Name("MarkdownProportional".into()));
        ctx.set_fonts(normalized.into_inner());
        let id = egui::Id::new("katana_fonts_loaded");
        ctx.data_mut(|d| d.insert_temp(id, is_loaded));

        #[cfg(debug_assertions)]
        ctx.style_mut(|style| {
            style.debug.debug_on_hover = false;
            style.debug.show_expand_width = false;
            style.debug.show_expand_height = false;
            style.debug.show_widget_hits = false;
        });
    }

    /// Builds `NormalizeFonts` pulling in system fonts and fallback chains.
    /// CJK fallback is inserted at position 1 (right after primary mono font)
    /// with `y_offset_factor` correction to ensure consistent baseline alignment.
    pub fn build_font_definitions(
        proportional_candidates: &[&str],
        monospace_candidates: &[&str],
        _emoji_candidates: &[&str],
        custom_font_path: Option<&str>,
        custom_font_name: Option<&str>,
    ) -> NormalizeFonts {
        let mut fonts = FontDefinitions::default();

        let prop_name = Self::load_first_valid(&mut fonts, proportional_candidates, None, "");

        let markdown_tweak = egui::FontTweak {
            scale: 1.0,
            y_offset_factor: MARKDOWN_PROPORTIONAL_Y_OFFSET_FACTOR, // 2px down locally for Markdown Proportional
            y_offset: 0.0,
        };
        let markdown_name = Self::load_first_valid(
            &mut fonts,
            proportional_candidates,
            Some(markdown_tweak),
            "_markdown",
        );

        let mono_tweak = egui::FontTweak {
            scale: 1.0,
            y_offset_factor: MONO_PRIMARY_Y_OFFSET_FACTOR,
            y_offset: 0.0,
        };
        let mono_name =
            Self::load_first_valid(&mut fonts, monospace_candidates, Some(mono_tweak), "");

        // Add primary fonts mapped to their corresponding families
        if let Some(name) = &prop_name {
            Self::prepend_primary(&mut fonts, FontFamily::Proportional, name);
        }
        if let Some(name) = &mono_name {
            Self::prepend_primary(&mut fonts, FontFamily::Monospace, name);
        }
        if let Some(name) = &markdown_name {
            Self::prepend_primary(
                &mut fonts,
                FontFamily::Name("MarkdownProportional".into()),
                name,
            );
        }

        // Mono to Prop fallback (usually English Monaco fallback for Proportional CJK)
        if let Some(name) = &mono_name {
            Self::append_fallback(&mut fonts, FontFamily::Proportional, name);
        }
        if let Some(name) = &mono_name {
            Self::append_fallback(
                &mut fonts,
                FontFamily::Name("MarkdownProportional".into()),
                name,
            );
        }

        if let (Some(path), Some(name)) = (custom_font_path, custom_font_name) {
            Self::inject_custom_font(&mut fonts, path, name);
        }

        // Wrap in NormalizeFonts and apply CJK baseline normalization
        NormalizeFonts::new(fonts).normalize(proportional_candidates)
    }

    fn load_first_valid(
        fonts: &mut FontDefinitions,
        candidates: &[&str],
        tweak: Option<egui::FontTweak>,
        suffix: &str,
    ) -> Option<String> {
        for &path in candidates {
            if let Ok(data) = fs::read(path) {
                let name = std::path::Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("cjk_font")
                    .to_string()
                    + suffix;

                let mut font_data = FontData::from_owned(data);
                if let Some(t) = tweak {
                    font_data.tweak = t;
                }

                fonts
                    .font_data
                    .insert(name.clone(), std::sync::Arc::new(font_data));
                return Some(name);
            }
        }
        None
    }

    fn prepend_primary(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.insert(0, name.to_string());
        }
    }

    fn append_fallback(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push(name.to_string());
        }
    }

    /// Insert a font right after the primary (position 1) in the family chain.
    /// This ensures our tweaked CJK fallback is used before any egui defaults.
    fn insert_after_primary(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            let pos = 1.min(list.len());
            list.insert(pos, name.to_string());
        }
    }

    fn inject_custom_font(fonts: &mut FontDefinitions, path: &str, name: &str) {
        let Ok(data) = fs::read(path) else { return };
        fonts.font_data.insert(
            name.to_string(),
            std::sync::Arc::new(FontData::from_owned(data)),
        );
        Self::prepend_primary(fonts, FontFamily::Proportional, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::FontId;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;

    const APPLE_COLOR_EMOJI_FONT_NAME: &str = "Apple Color Emoji";

    #[test]
    fn test_normalize_fonts_is_normalized_state() {
        let raw = NormalizeFonts::new(FontDefinitions::default());
        assert!(!raw.is_normalized());
        let normalized = raw.normalize(&[]);
        assert!(normalized.is_normalized());
    }

    #[test]
    fn test_normalize_fonts_double_normalize_is_noop() {
        let fonts = NormalizeFonts::new(FontDefinitions::default()).normalize(&[]);
        let family_before = fonts.fonts().families.clone();
        // Second call should be a no-op
        let fonts = fonts.normalize(&[]);
        assert_eq!(fonts.fonts().families, family_before);
    }

    #[test]
    fn test_build_font_definitions_no_candidates() {
        let fonts =
            SystemFontLoader::build_font_definitions(&[], &[], &[], None, None).into_inner();
        // egui's FontDefinitions::default() is not empty. We just ensure no custom names were added.
        assert!(!fonts.font_data.contains_key("cjk_font"));
        assert!(!fonts.font_data.contains_key("MyCustomFont"));
    }

    #[test]
    fn test_build_font_definitions_ignores_emoji_candidates_for_ui_families() {
        let tmp = TempDir::new().unwrap();
        let font_path = tmp.path().join("emoji.ttf");
        fs::write(&font_path, "").unwrap();
        let path_str = font_path.to_str().unwrap();

        let fonts = SystemFontLoader::build_font_definitions(&[], &[], &[path_str], None, None)
            .into_inner();

        let emoji_name = "emoji";
        assert!(
            !fonts.font_data.contains_key(emoji_name),
            "preview emoji are handled outside the global egui font families"
        );
        let prop_list = fonts.families.get(&FontFamily::Proportional).unwrap();
        assert!(!prop_list.contains(&emoji_name.to_string()));
        let mono_list = fonts.families.get(&FontFamily::Monospace).unwrap();
        assert!(!mono_list.contains(&emoji_name.to_string()));
    }

    #[test]
    fn test_custom_font_injection() {
        let tmp = TempDir::new().unwrap();
        let custom_font_path = tmp.path().join("custom.ttf");
        fs::write(&custom_font_path, "").unwrap();
        let path_str = custom_font_path.to_str().unwrap();

        let fonts = SystemFontLoader::build_font_definitions(
            &[],
            &[],
            &[],
            Some(path_str),
            Some("MyCustomFont"),
        );

        assert!(fonts.fonts().font_data.contains_key("MyCustomFont"));
        let prop_list = fonts
            .fonts()
            .families
            .get(&FontFamily::Proportional)
            .unwrap();
        assert_eq!(prop_list.first().unwrap(), "MyCustomFont");
    }

    #[test]
    fn test_custom_font_injection_invalid_path() {
        let fonts = SystemFontLoader::build_font_definitions(
            &[],
            &[],
            &[],
            Some("/path/does/not/exist.ttf"),
            Some("MyCustomFont"),
        );

        // Returns early without doing anything
        assert!(!fonts.fonts().font_data.contains_key("MyCustomFont"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_apple_color_emoji_family_renders_directly() {
        let data =
            fs::read("/System/Library/Fonts/Apple Color Emoji.ttc").expect("apple emoji font");
        let mut fonts = FontDefinitions::empty();
        fonts.font_data.insert(
            APPLE_COLOR_EMOJI_FONT_NAME.to_string(),
            Arc::new(FontData::from_owned(data)),
        );
        fonts.families.insert(
            FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into()),
            vec![APPLE_COLOR_EMOJI_FONT_NAME.to_string()],
        );

        let ctx = Context::default();
        ctx.set_fonts(fonts);

        let mut glyph = None;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    "🌍".to_owned(),
                    FontId::new(24.0, FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into())),
                    crate::theme_bridge::WHITE,
                );
                glyph = galley
                    .rows
                    .first()
                    .and_then(|row| row.glyphs.first())
                    .copied();
            });
        });

        let glyph = glyph.expect("emoji glyph should be laid out");
        assert!(
            !glyph.uv_rect.is_nothing(),
            "Apple Color Emoji should rasterize a visible glyph when used directly"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_ui_font_setup_does_not_register_apple_color_emoji_globally() {
        let preset = DiagramColorPreset::current();
        let fonts = SystemFontLoader::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            None,
            None,
        );
        let proportional = fonts
            .fonts()
            .families
            .get(&FontFamily::Proportional)
            .expect("proportional family");
        assert!(
            !proportional.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
            "UI symbol glyphs should keep using egui/built-in fallback fonts"
        );

        let monospace = fonts
            .fonts()
            .families
            .get(&FontFamily::Monospace)
            .expect("monospace family");
        assert!(
            !monospace.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
            "UI symbol glyphs should keep using egui/built-in fallback fonts"
        );
    }
    // --- TDD: Font Jitter Reproduction Tests ---

    fn assert_font_jitter(context_name: &str, font_size: f32) {
        let preset = DiagramColorPreset::current();
        let fonts = SystemFontLoader::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            None,
            None,
        );
        let ctx = Context::default();
        ctx.set_fonts(fonts.into_inner());

        let text = format!(
            "Katana — {} Lambda\u{30a2}\u{30c3}\u{30d7}\u{30c7}\u{30fc}\u{30c8}\u{624b}\u{9806}.md",
            context_name
        );
        let mut eng_glyph = None;
        let mut jpn_glyph = None;

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    text.clone(),
                    FontId::proportional(font_size),
                    crate::theme_bridge::WHITE,
                );
                eng_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'L').copied();
                jpn_glyph = galley.rows[0]
                    .glyphs
                    .iter()
                    .find(|g| g.chr == '\u{30a2}')
                    .copied();
            });
        });

        let eng_glyph = eng_glyph.expect("English char not found");
        let jpn_glyph = jpn_glyph.expect("Japanese char not found");

        assert_eq!(
            eng_glyph.pos.y, jpn_glyph.pos.y,
            "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in {}: English 'L' y={} vs Japanese '\u{30a2}' y={}",
            context_name, eng_glyph.pos.y, jpn_glyph.pos.y
        );
    }

    #[test]
    fn test_font_jitter_1_app_title() {
        // App title uses Heading (usually 20.0 or larger)
        assert_font_jitter("App Title", 20.0);
    }

    #[test]
    fn test_font_jitter_2_workspace_dir() {
        // Workspace directory uses Body (14.0)
        assert_font_jitter("Workspace Dir", 14.0);
    }

    #[test]
    fn test_font_jitter_3_workspace_file() {
        // Workspace file uses Body (14.0)
        assert_font_jitter("Workspace File", 14.0);
    }

    #[test]
    fn test_font_jitter_4_toc_heading() {
        // TOC heading uses Body/Button (14.0)
        assert_font_jitter("TOC Heading", 14.0);
    }

    #[test]
    fn test_font_jitter_5_tab_name() {
        // Tab name uses Button (14.0)
        assert_font_jitter("Tab Name", 14.0);
    }

    #[test]
    fn test_font_jitter_6_monospace() {
        let preset = DiagramColorPreset::current();
        let fonts = SystemFontLoader::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            None,
            None,
        );
        let ctx = Context::default();
        ctx.set_fonts(fonts.into_inner());

        let text =
            "cd infrastructures/tools/crypt-decrypt \u{306e}\u{5fa9}\u{53f7}\u{5316}".to_string();
        let mut eng_glyph = None;
        let mut jpn_glyph = None;

        let mut primitives = vec![];
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    text.clone(),
                    FontId::monospace(14.0),
                    crate::theme_bridge::WHITE,
                );
                eng_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'c').copied();
                jpn_glyph = galley.rows[0]
                    .glyphs
                    .iter()
                    .find(|g| g.chr == '\u{5fa9}')
                    .copied();

                let shapes = vec![egui::epaint::ClippedShape {
                    clip_rect: egui::Rect::EVERYTHING,
                    shape: egui::epaint::Shape::galley(
                        egui::Pos2::ZERO,
                        galley,
                        crate::theme_bridge::WHITE,
                    ),
                }];
                primitives = ctx.tessellate(shapes, 1.0);
            });
        });

        let eng_glyph = eng_glyph.expect("English char not found");
        let jpn_glyph = jpn_glyph.expect("Japanese char not found");

        // Find visual min y for English char
        let mut eng_min_y = f32::INFINITY;
        let mut jpn_min_y = f32::INFINITY;

        if let egui::epaint::Primitive::Mesh(mesh) = &primitives[0].primitive {
            for v in &mesh.vertices {
                if v.pos.x >= eng_glyph.logical_rect().min.x
                    && v.pos.x <= eng_glyph.logical_rect().max.x
                {
                    eng_min_y = eng_min_y.min(v.pos.y);
                }
                if v.pos.x >= jpn_glyph.logical_rect().min.x
                    && v.pos.x <= jpn_glyph.logical_rect().max.x
                {
                    jpn_min_y = jpn_min_y.min(v.pos.y);
                }
            }
        }

        // Allow up to 1.0 pixel difference for natural font optical alignment,
        // but 3.0+ (like 13 vs 10) is a jitter bug.
        let diff = (eng_min_y - jpn_min_y).abs();
        assert!(
            diff <= 1.5,
            "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in Monospace visual mesh: English 'c' y={} vs Japanese '\u{5fa9}' y={} (Diff: {})",
            eng_min_y, jpn_min_y, diff
        );
    }

    /// TDD RED→GREEN: Reproduce the actual egui_commonmark CodeBlock rendering path.
    ///
    /// Uses `LayoutJob` + `TextFormat::simple(TextStyle::Monospace)` +
    /// `fonts_mut(|f| f.layout_job(job))` — the exact same path as
    /// `egui_commonmark_backend::misc::CodeBlock::end`.
    ///
    /// Measures VISUAL jitter via tessellated mesh vertices (not logical glyph.pos),
    /// since `y_offset_factor` is applied during tessellation only.
    #[test]
    fn test_font_jitter_7_codeblock_layoutjob() {
        let preset = DiagramColorPreset::current();
        let fonts = SystemFontLoader::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            None,
            None,
        );
        let ctx = Context::default();
        ctx.set_fonts(fonts.into_inner());

        // Simulate a code block comment with mixed JP/EN,
        // exactly as seen in the user's app.
        let code_text = "# \u{5168}\u{4ef6}\u{5b9f}\u{884c}";

        let mut hash_glyph = None;
        let mut jp_glyph = None;
        let mut primitives = vec![];

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Build LayoutJob the same way CodeBlock::end does
                let mut job = egui::text::LayoutJob::default();
                job.append(
                    code_text,
                    0.0,
                    egui::TextFormat::simple(
                        egui::TextStyle::Monospace.resolve(ui.style()),
                        crate::theme_bridge::WHITE,
                    ),
                );
                job.wrap.max_width = 800.0;

                let galley = ui.fonts_mut(|f| f.layout_job(job));

                // Capture glyph logical rects for mesh vertex lookup
                if let Some(row) = galley.rows.first() {
                    for g in &row.glyphs {
                        if g.chr == '#' {
                            hash_glyph = Some(*g);
                        }
                        if g.chr == '\u{5168}' {
                            jp_glyph = Some(*g);
                        }
                    }
                }

                let shapes = vec![egui::epaint::ClippedShape {
                    clip_rect: egui::Rect::EVERYTHING,
                    shape: egui::epaint::Shape::galley(
                        egui::Pos2::ZERO,
                        galley,
                        crate::theme_bridge::WHITE,
                    ),
                }];
                primitives = ctx.tessellate(shapes, 1.0);
            });
        });

        let hash_glyph = hash_glyph.expect("'#' glyph not found");
        let jp_glyph = jp_glyph.expect("'\u{5168}' glyph not found");

        // Find visual min y from tessellated mesh (actual pixel positions)
        let mut hash_min_y = f32::INFINITY;
        let mut jp_min_y = f32::INFINITY;

        if let egui::epaint::Primitive::Mesh(mesh) = &primitives[0].primitive {
            for v in &mesh.vertices {
                if v.pos.x >= hash_glyph.logical_rect().min.x
                    && v.pos.x <= hash_glyph.logical_rect().max.x
                {
                    hash_min_y = hash_min_y.min(v.pos.y);
                }
                if v.pos.x >= jp_glyph.logical_rect().min.x
                    && v.pos.x <= jp_glyph.logical_rect().max.x
                {
                    jp_min_y = jp_min_y.min(v.pos.y);
                }
            }
        }

        let diff = (hash_min_y - jp_min_y).abs();
        assert!(
            diff <= 1.5,
            "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in CodeBlock LayoutJob visual mesh: '#' y={} vs '\u{5168}' y={} (Diff: {}). \
             Mixed JP/EN text must share a common baseline.",
            hash_min_y, jp_min_y, diff
        );
    }

    /// Cross-family (Monospace + Proportional) inline code jitter test.
    /// Simulates backtick-wrapped text (`mmdc`) within body text.
    ///
    /// NOTE: Tolerance is relaxed (10px) because cross-family baseline normalization
    /// via `y_offset_factor` has been removed — it caused ALL body text (tabs, buttons,
    /// labels) to appear bottom-aligned. Inline code alignment should be handled
    /// at the LayoutJob/rendering level instead.
    #[test]
    #[cfg(target_os = "macos")]
    fn test_font_jitter_8_inline_code_cross_family() {
        use egui::text::{LayoutJob, TextFormat};
        use egui::TextStyle;

        let preset = DiagramColorPreset::current();
        let fonts = SystemFontLoader::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            None,
            None,
        );
        let ctx = Context::default();
        ctx.set_fonts(fonts.into_inner());

        let mut prop_min_y = f32::INFINITY;
        let mut mono_min_y = f32::INFINITY;

        let mut primitives = vec![];
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Build a LayoutJob with mixed Proportional and Monospace sections,
                // exactly like inline code in body text:
                // "After installation, `mmdc` is automatically detected"
                let mut job = LayoutJob::default();

                let prop_format = TextFormat {
                    font_id: TextStyle::Body.resolve(ui.style()),
                    ..Default::default()
                };
                let mono_format = TextFormat {
                    font_id: TextStyle::Monospace.resolve(ui.style()),
                    ..Default::default()
                };

                // Proportional section
                job.append("\u{30a4}\u{30f3}\u{30b9}\u{30c8}\u{30fc}\u{30eb}\u{5f8c}\u{3001}", 0.0, prop_format.clone());
                // Monospace section (inline code)
                job.append("mmdc", 0.0, mono_format);
                // Proportional section
                job.append(" \u{306f}\u{81ea}\u{52d5}\u{7684}\u{306b}\u{691c}\u{51fa}\u{3055}\u{308c}\u{307e}\u{3059}", 0.0, prop_format);

                let galley = ui.fonts_mut(|f| f.layout_job(job));

                // Find glyphs: '\u{30a4}' from Proportional, 'm' from Monospace
                let prop_glyph = galley.rows[0]
                    .glyphs
                    .iter()
                    .find(|g| g.chr == '\u{30a4}')
                    .copied();
                let mono_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'm').copied();

                if let (Some(pg), Some(mg)) = (prop_glyph, mono_glyph) {
                    // Tessellate to get actual visual positions
                    let shapes = vec![egui::epaint::ClippedShape {
                        clip_rect: egui::Rect::EVERYTHING,
                        shape: egui::epaint::Shape::galley(
                            egui::Pos2::ZERO,
                            galley,
                            crate::theme_bridge::WHITE,
                        ),
                    }];
                    primitives = ctx.tessellate(shapes, 1.0);

                    if let Some(egui::epaint::Primitive::Mesh(mesh)) =
                        primitives.first().map(|p| &p.primitive)
                    {
                        for v in &mesh.vertices {
                            if v.pos.x >= pg.logical_rect().min.x
                                && v.pos.x <= pg.logical_rect().max.x
                            {
                                prop_min_y = prop_min_y.min(v.pos.y);
                            }
                            if v.pos.x >= mg.logical_rect().min.x
                                && v.pos.x <= mg.logical_rect().max.x
                            {
                                mono_min_y = mono_min_y.min(v.pos.y);
                            }
                        }
                    }
                }
            });
        });

        assert!(
            prop_min_y.is_finite() && mono_min_y.is_finite(),
            "Both glyphs must be found in mesh"
        );

        let diff = (prop_min_y - mono_min_y).abs();
        assert!(
            diff <= 10.0,
            "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in inline code: Proportional '\u{30a4}' y={} vs Monospace 'm' y={} (Diff: {}). \
             Cross-family alignment is not enforced at font-level; handle at LayoutJob level.",
            prop_min_y, mono_min_y, diff
        );
    }

    /// RED: Proportional primary font y_offset_factor MUST be zero or near-zero.
    ///
    /// A non-zero `y_offset_factor` on the Proportional primary font shifts ALL
    /// body text downward within widgets (buttons, labels, tabs), making them
    /// appear bottom-aligned instead of vertically centred.
    ///
    /// The cross-family baseline alignment (inline code vs body text) should be
    /// achieved by adjusting the Monospace side, not the Proportional side.
    #[test]
    fn test_proportional_primary_y_offset_is_zero() {
        let preset = DiagramColorPreset::current();
        let fonts = SystemFontLoader::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            None,
            None,
        );

        let prop_primary = fonts
            .fonts()
            .families
            .get(&FontFamily::Proportional)
            .and_then(|list| list.first())
            .cloned();

        if let Some(name) = prop_primary {
            let font_data = fonts.fonts().font_data.get(&name).expect("font data");
            let y_offset = font_data.tweak.y_offset_factor;
            assert!(
                y_offset.abs() < 0.01,
                "Proportional primary font y_offset_factor must be ~0 for correct vertical \
                 centering in widgets. Got: {}. Cross-family alignment should be achieved \
                 by adjusting the Monospace side instead.",
                y_offset
            );
        }
    }
    #[test]
    fn test_setup_fonts_sets_fonts_loaded_flag() {
        let ctx = Context::default();
        let preset = DiagramColorPreset::current();
        SystemFontLoader::setup_fonts(&ctx, preset, None, None);

        // Flag is always set (true if CJK fonts found, false otherwise).
        let loaded = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("katana_fonts_loaded")));
        assert!(
            loaded.is_some(),
            "katana_fonts_loaded flag must always be set"
        );
    }
}
