use egui::{Context, FontData, FontDefinitions, FontFamily};
use katana_core::markdown::color_preset::DiagramColorPreset;
use std::fs;

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
        let fonts = Self::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            custom_font_path,
            custom_font_name,
        );
        ctx.set_fonts(fonts);

        #[cfg(debug_assertions)]
        ctx.style_mut(|style| {
            style.debug.debug_on_hover = false;
            style.debug.show_expand_width = false;
            style.debug.show_expand_height = false;
            style.debug.show_widget_hits = false;
        });
    }

    /// Builds `FontDefinitions` pulling in system fonts and fallback chains.
    pub fn build_font_definitions(
        proportional_candidates: &[&str],
        monospace_candidates: &[&str],
        _emoji_candidates: &[&str],
        custom_font_path: Option<&str>,
        custom_font_name: Option<&str>,
    ) -> FontDefinitions {
        let mut fonts = FontDefinitions::default();

        let prop_name = Self::load_first_valid(&mut fonts, proportional_candidates);
        let mono_name = Self::load_first_valid(&mut fonts, monospace_candidates);

        // Add primary fonts mapped to their corresponding families
        if let Some(name) = &prop_name {
            Self::prepend_primary(&mut fonts, FontFamily::Proportional, name);
        }
        if let Some(name) = &mono_name {
            Self::prepend_primary(&mut fonts, FontFamily::Monospace, name);
        }

        // Cross-fallbacks for comprehensive CJK coverage in code blocks, and vice versa
        if let Some(name) = &prop_name {
            Self::append_fallback(&mut fonts, FontFamily::Monospace, name);
        }
        if let Some(name) = &mono_name {
            Self::append_fallback(&mut fonts, FontFamily::Proportional, name);
        }

        if let (Some(path), Some(name)) = (custom_font_path, custom_font_name) {
            Self::inject_custom_font(&mut fonts, path, name);
        }

        fonts
    }

    fn load_first_valid(fonts: &mut FontDefinitions, candidates: &[&str]) -> Option<String> {
        for &path in candidates {
            if let Ok(data) = fs::read(path) {
                let name = std::path::Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("cjk_font")
                    .to_string();
                fonts.font_data.insert(
                    name.clone(),
                    std::sync::Arc::new(FontData::from_owned(data)),
                );
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
    use egui::{Color32, FontId};
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;

    const APPLE_COLOR_EMOJI_FONT_NAME: &str = "Apple Color Emoji";

    #[test]
    fn test_build_font_definitions_no_candidates() {
        let fonts = SystemFontLoader::build_font_definitions(&[], &[], &[], None, None);
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

        let fonts = SystemFontLoader::build_font_definitions(&[], &[], &[path_str], None, None);

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

        assert!(fonts.font_data.contains_key("MyCustomFont"));
        let prop_list = fonts.families.get(&FontFamily::Proportional).unwrap();
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
        assert!(!fonts.font_data.contains_key("MyCustomFont"));
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
                    Color32::WHITE,
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
            .families
            .get(&FontFamily::Proportional)
            .expect("proportional family");
        assert!(
            !proportional.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
            "UI symbol glyphs should keep using egui/built-in fallback fonts"
        );

        let monospace = fonts
            .families
            .get(&FontFamily::Monospace)
            .expect("monospace family");
        assert!(
            !monospace.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
            "UI symbol glyphs should keep using egui/built-in fallback fonts"
        );
    }
    // --- TDD: Font Jitter (ガタツキ) Reproduction Tests ---

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
        ctx.set_fonts(fonts);

        let text = format!("Katana — {} Lambdaアップデート手順.md", context_name);
        let mut eng_glyph = None;
        let mut jpn_glyph = None;

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    text.clone(),
                    FontId::proportional(font_size),
                    Color32::WHITE,
                );
                eng_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'L').copied();
                jpn_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'ア').copied();
            });
        });

        let eng_glyph = eng_glyph.expect("English char not found");
        let jpn_glyph = jpn_glyph.expect("Japanese char not found");

        assert_eq!(
            eng_glyph.pos.y, jpn_glyph.pos.y,
            "ガタツキ (Jitter) in {}: English 'L' y={} vs Japanese 'ア' y={}",
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
}
