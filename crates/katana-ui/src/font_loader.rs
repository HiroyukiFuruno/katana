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

        if let Some(name) = Self::load_first_valid(&mut fonts, emoji_candidates) {
            Self::append_fallback(&mut fonts, FontFamily::Proportional, &name);
            Self::append_fallback(&mut fonts, FontFamily::Monospace, &name);
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
        let Some(name) = Self::load_first_valid(fonts, candidates) else {
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
