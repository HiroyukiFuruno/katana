use egui::{Context, FontData, FontDefinitions, FontFamily, FontTweak};
use katana_core::markdown::color_preset::DiagramColorPreset;
use std::fs;

const EMOJI_FONT_SCALE: f32 = 0.9;

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
            preset.proportional_font_candidates,
            preset.monospace_font_candidates,
            preset.emoji_font_candidates,
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
        emoji_candidates: &[&str],
        custom_font_path: Option<&str>,
        custom_font_name: Option<&str>,
    ) -> FontDefinitions {
        let mut fonts = FontDefinitions::default();

        Self::load_system_candidates(
            &mut fonts,
            FontFamily::Proportional,
            proportional_candidates,
        );
        Self::load_system_candidates(&mut fonts, FontFamily::Monospace, monospace_candidates);

        if let Some(name) = Self::load_first_valid(&mut fonts, emoji_candidates, true) {
            if Self::should_prioritize_system_emoji(&name) {
                Self::insert_fallback_after_primary(&mut fonts, FontFamily::Proportional, &name);
                Self::insert_fallback_after_primary(&mut fonts, FontFamily::Monospace, &name);
            } else {
                Self::append_fallback(&mut fonts, FontFamily::Proportional, &name);
                Self::append_fallback(&mut fonts, FontFamily::Monospace, &name);
            }
        }

        if let (Some(path), Some(name)) = (custom_font_path, custom_font_name) {
            Self::inject_custom_font(&mut fonts, path, name);
        }

        fonts
    }

    fn load_system_candidates(
        fonts: &mut FontDefinitions,
        primary_family: FontFamily,
        candidates: &[&str],
    ) {
        let Some(name) = Self::load_first_valid(fonts, candidates, false) else {
            return;
        };
        Self::prepend_primary(fonts, primary_family.clone(), &name);

        let fallback_family = if primary_family == FontFamily::Proportional {
            FontFamily::Monospace
        } else {
            FontFamily::Proportional
        };
        Self::append_fallback(fonts, fallback_family, &name);
    }

    fn load_first_valid(
        fonts: &mut FontDefinitions,
        candidates: &[&str],
        emoji_tweak: bool,
    ) -> Option<String> {
        for &path in candidates {
            if let Ok(data) = fs::read(path) {
                let name = std::path::Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("cjk_font")
                    .to_string();
                let font_data = if emoji_tweak {
                    FontData::from_owned(data).tweak(FontTweak {
                        scale: EMOJI_FONT_SCALE,
                        ..Default::default()
                    })
                } else {
                    FontData::from_owned(data)
                };
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

    fn should_prioritize_system_emoji(_name: &str) -> bool {
        true
    }

    fn insert_fallback_after_primary(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            let insert_at = usize::from(!list.is_empty());
            list.insert(insert_at, name.to_string());
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
    fn test_build_font_definitions_with_fallback() {
        let tmp = TempDir::new().unwrap();
        let font_path = tmp.path().join("emoji.ttf");
        fs::write(&font_path, "").unwrap();
        let path_str = font_path.to_str().unwrap();

        let fonts = SystemFontLoader::build_font_definitions(&[], &[], &[path_str], None, None);

        let emoji_name = "emoji"; // from file_stem
        assert!(fonts.font_data.contains_key(emoji_name));

        // Ensure the font was added to the fallback list for both families
        let prop_list = fonts.families.get(&FontFamily::Proportional).unwrap();
        assert!(prop_list.contains(&emoji_name.to_string()));

        let mono_list = fonts.families.get(&FontFamily::Monospace).unwrap();
        assert!(mono_list.contains(&emoji_name.to_string()));
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
    fn test_load_system_candidates_empty_returns_early() {
        let mut fonts = FontDefinitions::empty();
        SystemFontLoader::load_system_candidates(&mut fonts, FontFamily::Proportional, &[]);
        assert!(fonts.font_data.is_empty());
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
    fn test_apple_color_emoji_has_extended_coverage_beyond_builtin_fallbacks() {
        let mut fonts = FontDefinitions::default();
        for name in [
            APPLE_COLOR_EMOJI_FONT_NAME,
            "NotoEmoji-Regular",
            "emoji-icon-font",
        ] {
            if fonts.font_data.contains_key(name) {
                fonts
                    .families
                    .insert(FontFamily::Name(name.into()), vec![name.to_string()]);
            }
        }
        if !fonts.font_data.contains_key(APPLE_COLOR_EMOJI_FONT_NAME) {
            let data =
                fs::read("/System/Library/Fonts/Apple Color Emoji.ttc").expect("apple emoji font");
            fonts.font_data.insert(
                APPLE_COLOR_EMOJI_FONT_NAME.to_string(),
                Arc::new(FontData::from_owned(data)),
            );
            fonts.families.insert(
                FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into()),
                vec![APPLE_COLOR_EMOJI_FONT_NAME.to_string()],
            );
        }

        let ctx = Context::default();
        ctx.set_fonts(fonts);

        let mut apple_results = Vec::new();
        let mut noto_results = Vec::new();
        let mut icon_results = Vec::new();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.ctx().fonts_mut(|fonts| {
                    for emoji in ["🌍", "🧪", "📖", "⚔️", "❤️"] {
                        apple_results.push(fonts.has_glyphs(
                            &FontId::new(
                                24.0,
                                FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into()),
                            ),
                            emoji,
                        ));
                        noto_results.push(fonts.has_glyphs(
                            &FontId::new(24.0, FontFamily::Name("NotoEmoji-Regular".into())),
                            emoji,
                        ));
                        icon_results.push(fonts.has_glyphs(
                            &FontId::new(24.0, FontFamily::Name("emoji-icon-font".into())),
                            emoji,
                        ));
                    }
                });
            });
        });

        assert!(
            apple_results.iter().all(|covered| *covered),
            "Apple Color Emoji should cover the macOS emoji regression set"
        );
        assert!(
            noto_results
                .iter()
                .zip(icon_results.iter())
                .any(|(noto, icon)| !noto && !icon),
            "built-in emoji fallbacks should miss at least one regression emoji, otherwise system priority is not justified"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_emoji_fallback_produces_rasterized_glyphs() {
        let ctx = Context::default();
        let preset = DiagramColorPreset::current();
        SystemFontLoader::setup_fonts(&ctx, preset, None, None);

        let mut glyph = None;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    "🙂".to_owned(),
                    FontId::proportional(24.0),
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
            "emoji fallback was registered but produced no rasterized glyph"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_system_emoji_precedes_builtin_fallbacks() {
        let preset = DiagramColorPreset::current();
        let fonts = SystemFontLoader::build_font_definitions(
            preset.proportional_font_candidates,
            preset.monospace_font_candidates,
            preset.emoji_font_candidates,
            None,
            None,
        );
        let proportional = fonts
            .families
            .get(&FontFamily::Proportional)
            .expect("proportional family");
        let emoji_idx = proportional
            .iter()
            .position(|name| name == APPLE_COLOR_EMOJI_FONT_NAME)
            .expect("Apple Color Emoji should be registered on macOS");

        for builtin in ["NotoEmoji-Regular", "emoji-icon-font"] {
            if let Some(builtin_idx) = proportional.iter().position(|name| name == builtin) {
                assert!(
                    emoji_idx < builtin_idx,
                    "Apple Color Emoji must precede built-in emoji fallbacks on macOS, family order was {proportional:?}"
                );
            }
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_common_emoji_sequences_produce_rasterized_glyphs() {
        let ctx = Context::default();
        let preset = DiagramColorPreset::current();
        SystemFontLoader::setup_fonts(&ctx, preset, None, None);

        for emoji in ["🌍", "🦀", "⚡", "📝", "🔧", "✅", "❌", "💡"] {
            let mut glyph = None;
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let galley = ui.painter().layout_no_wrap(
                        emoji.to_owned(),
                        FontId::proportional(24.0),
                        Color32::WHITE,
                    );
                    glyph = galley
                        .rows
                        .first()
                        .and_then(|row| row.glyphs.first())
                        .copied();
                });
            });
            let glyph = glyph.unwrap_or_else(|| panic!("{emoji} should be laid out"));
            assert!(
                !glyph.uv_rect.is_nothing(),
                "{emoji} produced no rasterized glyph"
            );
        }
    }
}
