#![allow(clippy::useless_vec)]
#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::cognitive_complexity
)]

#[cfg(not(test))]
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::{ExtensionPoint, PluginMeta, PluginRegistry, PLUGIN_API_VERSION};
#[cfg(not(test))]
use katana_platform::{JsonFileRepository, SettingsService};
#[cfg(not(test))]
use katana_ui::app_state::AppState;
#[cfg(not(test))]
use katana_ui::shell::KatanaApp;
#[cfg(all(target_os = "macos", not(test)))]
use katana_ui::shell_ui;

#[cfg(not(test))]
const INITIAL_WIDTH: f32 = 1280.0;
#[cfg(not(test))]
const INITIAL_HEIGHT: f32 = 800.0;
#[cfg(not(test))]
const MIN_WIDTH: f32 = 800.0;
#[cfg(not(test))]
const MIN_HEIGHT: f32 = 500.0;
#[cfg(all(target_os = "macos", not(test)))]
const LOCALE_BUF_SIZE: usize = 32;

#[cfg(not(test))]
fn initial_window_size() -> egui::Vec2 {
    egui::vec2(INITIAL_WIDTH, INITIAL_HEIGHT)
}

#[cfg(not(test))]
fn min_window_size() -> egui::Vec2 {
    egui::vec2(MIN_WIDTH, MIN_HEIGHT)
}

#[cfg(not(test))]
fn load_icon() -> std::sync::Arc<egui::IconData> {
    let icon_bytes = include_bytes!("../../../assets/icon.iconset/icon_512x512.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon byte map")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    std::sync::Arc::new(egui::IconData {
        rgba,
        width,
        height,
    })
}

#[cfg(all(target_os = "macos", not(test)))]
fn resolve_locale_to_lang(locale: &str) -> String {
    let lower = locale.to_lowercase();

    if lower.starts_with("zh-hans") || lower.contains("hans") {
        return "zh-CN".to_string();
    }
    if lower.starts_with("zh-hant")
        || lower.contains("hant")
        || lower.starts_with("zh-tw")
        || lower.starts_with("zh-hk")
    {
        return "zh-TW".to_string();
    }

    const PREFIX_MAP: &[(&str, &str)] = &[
        ("ja", "ja"),
        ("ko", "ko"),
        ("pt", "pt"),
        ("fr", "fr"),
        ("de", "de"),
        ("es", "es"),
        ("it", "it"),
    ];
    for &(prefix, lang) in PREFIX_MAP {
        if lower.starts_with(prefix) {
            return lang.to_string();
        }
    }
    "en".to_string()
}

#[cfg(not(test))]
fn detect_initial_language() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn katana_get_mac_locale(buf: *mut std::ffi::c_char, max_len: usize);
        }
        let mut buf = [0u8; LOCALE_BUF_SIZE];
        unsafe { katana_get_mac_locale(buf.as_mut_ptr() as _, buf.len()) };
        let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as _) };
        let locale = c_str.to_string_lossy().to_string();
        return Some(resolve_locale_to_lang(&locale));
    }
    #[allow(unreachable_code)]
    None
}

#[cfg(not(test))]
fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "katana_ui=info,katana_core=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting KatanA");

    #[cfg(target_os = "macos")]
    unsafe {
        shell_ui::native_set_process_name();
    }

    let ai_registry = AiProviderRegistry::new();

    let mut plugin_registry = PluginRegistry::new();
    register_builtin_plugins(&mut plugin_registry);

    let repo = JsonFileRepository::with_default_path();
    let mut settings = SettingsService::new(Box::new(repo));

    settings.apply_os_default_theme();
    settings.apply_os_default_language(detect_initial_language());

    let saved_language = settings.settings().language.clone();
    let saved_workspace = settings.settings().workspace.last_workspace.clone();

    let cache = std::sync::Arc::new(katana_platform::DefaultCacheService::default());
    let state = AppState::new(ai_registry, plugin_registry, settings, cache);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("KatanA")
            .with_icon(load_icon())
            .with_inner_size(initial_window_size())
            .with_min_inner_size(min_window_size())
            .with_maximized(true),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "KatanA",
        native_options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            katana_ui::svg_loader::install_image_loaders(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            katana_ui::icon::IconRegistry::install(&cc.egui_ctx);

            #[cfg(target_os = "macos")]
            unsafe {
                shell_ui::native_menu_setup();
                let png_bytes = include_bytes!("../../../assets/icon.iconset/icon_512x512.png");
                shell_ui::native_set_app_icon_png(png_bytes.as_ptr(), png_bytes.len());
            }

            katana_ui::i18n::set_language(&saved_language);
            katana_ui::shell_ui::update_native_menu_strings_from_i18n();

            let mut app = KatanaApp::new(state);

            let icon_png = include_bytes!("../../../assets/icon.iconset/icon_128x128.png");
            match image::load_from_memory(icon_png) {
                Ok(icon_image) => {
                    let rgba = icon_image.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    let texture = cc.egui_ctx.load_texture(
                        "about_icon",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    app.about_icon = Some(texture);
                }
                Err(_) => {}
            }

            if let Some(ws_path) = saved_workspace {
                let path = std::path::PathBuf::from(&ws_path);
                if path.is_dir() {
                    app.trigger_action(katana_ui::app_state::AppAction::OpenWorkspace(path));
                    tracing::info!("Restored workspace: {ws_path}");
                }
            }

            Ok(Box::new(app))
        }),
    )
}

pub fn setup_fonts(ctx: &egui::Context) {
    let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
    setup_fonts_from_preset(ctx, preset);
}

pub fn setup_fonts_from_preset(
    ctx: &egui::Context,
    preset: &katana_core::markdown::color_preset::DiagramColorPreset,
) {
    katana_ui::font_loader::SystemFontLoader::setup_fonts(ctx, preset, None, None);
}

pub fn setup_fonts_with_candidates(ctx: &egui::Context, candidates: &[&str]) {
    let normalized = build_font_definitions(candidates, &[], &[]);
    ctx.set_fonts(normalized.into_inner());

    #[cfg(debug_assertions)]
    ctx.style_mut(|style| {
        style.debug.debug_on_hover = false;
        style.debug.show_expand_width = false;
        style.debug.show_expand_height = false;
        style.debug.show_widget_hits = false;
    });
}

pub fn build_font_definitions(
    proportional_candidates: &[&str],
    monospace_candidates: &[&str],
    emoji_candidates: &[&str],
) -> katana_ui::font_loader::NormalizeFonts {
    katana_ui::font_loader::SystemFontLoader::build_font_definitions(
        proportional_candidates,
        monospace_candidates,
        emoji_candidates,
        None,
        None,
    )
}

pub fn load_first_font(candidates: &[&str]) -> Option<(String, Vec<u8>)> {
    for &path in candidates {
        let Ok(data) = std::fs::read(path) else { continue };
        let name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("cjk_font")
            .to_string();
        return Some((name, data));
    }
    None
}

pub fn register_builtin_plugins(registry: &mut PluginRegistry) {
    registry.register(
        PluginMeta {
            id: "builtin-mermaid-renderer".to_string(),
            name: "Built-in Mermaid Renderer".to_string(),
            api_version: PLUGIN_API_VERSION,
            extension_points: vec![ExtensionPoint::RendererEnhancement],
        },
        || Ok(()), // WHY: Renderer logic is wired directly in the markdown pipeline.
    );

    registry.register(
        PluginMeta {
            id: "builtin-plantuml-renderer".to_string(),
            name: "Built-in PlantUML Renderer".to_string(),
            api_version: PLUGIN_API_VERSION,
            extension_points: vec![ExtensionPoint::RendererEnhancement],
        },
        || Ok(()),
    );

    registry.register(
        PluginMeta {
            id: "builtin-drawio-renderer".to_string(),
            name: "Built-in Draw.io Renderer".to_string(),
            api_version: PLUGIN_API_VERSION,
            extension_points: vec![ExtensionPoint::RendererEnhancement],
        },
        || Ok(()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    #[test]
    fn test_load_first_font_not_found() {
        let candidates = vec!["/invalid/path/to/never/found/font.ttc"];
        let result = load_first_font(&candidates);
        assert!(result.is_none());
    }

    #[test]
    fn test_load_first_font_found() {
        let candidates = vec![
            "/System/Library/Fonts/AquaKana.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ];
        let result = load_first_font(&candidates);
        if let Some((name, data)) = result {
            assert!(!name.is_empty());
            assert!(!data.is_empty());
        }
    }

    #[test]
    fn test_setup_fonts_with_cjk() {
        init_tracing();
        let ctx = egui::Context::default();
        setup_fonts(&ctx);
    }

    #[test]
    fn test_setup_fonts_without_cjk() {
        init_tracing();
        let ctx = egui::Context::default();
        setup_fonts_with_candidates(&ctx, &["/nonexistent/font.ttc"]);
    }

    #[test]
    fn test_register_builtin_plugins() {
        init_tracing();
        let mut registry = PluginRegistry::new();
        register_builtin_plugins(&mut registry);
        assert_eq!(registry.active_count(), 3);
    }


    #[test]
    fn test_install_image_loaders_does_not_panic() {
        let ctx = egui::Context::default();
        katana_ui::svg_loader::install_image_loaders(&ctx);
        assert!(ctx.is_loader_installed(katana_ui::svg_loader::KatanaSvgLoader::ID));
    }


    const PROP_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
        "/System/Library/Fonts/AquaKana.ttc",
    ];

    const MONO_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Monaco.ttf",
    ];

    #[test]
    fn test_proportional_font_is_primary_in_proportional_family() {
        init_tracing();
        if load_first_font(PROP_CANDIDATES).is_none() {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, &[]);
        let proportional = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("Proportional family missing");
        let loaded_name = load_first_font(PROP_CANDIDATES).unwrap().0;
        assert_eq!(
            proportional.first().unwrap(),
            &loaded_name,
            "CJK font SHOULD be at position 0 to dictate proper row height and fix jitter"
        );
    }

    #[test]
    fn test_monospace_font_is_primary_in_monospace_family() {
        init_tracing();
        if load_first_font(MONO_CANDIDATES).is_none() {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, &[]);
        let monospace = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let mono_name = load_first_font(MONO_CANDIDATES).unwrap().0;
        assert_eq!(
            monospace.first().unwrap(),
            &mono_name,
            "Monospace CJK font SHOULD be at position 0 to provide correct line height"
        );
    }

    #[test]
    fn test_proportional_font_is_cjk_fallback_in_monospace() {
        init_tracing();
        if load_first_font(PROP_CANDIDATES).is_none() || load_first_font(MONO_CANDIDATES).is_none()
        {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, &[]);
        let monospace = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let prop_name = load_first_font(PROP_CANDIDATES).unwrap().0;
        let mono_fallback_name = format!("{}_mono_fallback", prop_name);
        assert!(
            monospace.contains(&mono_fallback_name),
            "Proportional font should be in Monospace family as CJK fallback"
        );
        let mono_name = load_first_font(MONO_CANDIDATES).unwrap().0;
        let mono_pos = monospace.iter().position(|n| n == &mono_name).unwrap();
        let prop_pos = monospace
            .iter()
            .position(|n| n == &mono_fallback_name)
            .unwrap();
        assert!(
            mono_pos < prop_pos,
            "Monospace font must appear before proportional (CJK fallback)"
        );
    }

    #[test]
    fn test_build_font_definitions_without_candidates_returns_defaults() {
        init_tracing();
        let fonts = build_font_definitions(&["/nonexistent/font.ttc"], &[], &[]);
        let proportional = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("Proportional family missing");
        assert!(
            !proportional.is_empty(),
            "Proportional family should have default egui fonts"
        );
    }

    #[test]
    fn test_setup_fonts_from_preset_does_not_panic() {
        init_tracing();
        let ctx = egui::Context::default();
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
        setup_fonts_from_preset(&ctx, preset);
    }


    #[test]
    fn test_preset_syntax_themes_are_valid_identifiers() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        let preset = DiagramColorPreset::current();
        assert!(
            !preset.syntax_theme_dark.is_empty(),
            "syntax_theme_dark must not be empty"
        );
        assert!(
            !preset.syntax_theme_light.is_empty(),
            "syntax_theme_light must not be empty"
        );
    }

    #[test]
    fn test_preset_preview_text_is_valid_hex_color() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        let preset = DiagramColorPreset::current();
        let parsed = DiagramColorPreset::parse_hex_rgb(preset.preview_text);
        assert!(
            parsed.is_some(),
            "preview_text '{}' must be a valid #RRGGBB hex",
            preset.preview_text
        );
    }

    #[test]
    fn test_preset_dark_and_light_have_different_preview_text() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        assert_ne!(
            DiagramColorPreset::dark().preview_text,
            DiagramColorPreset::light().preview_text,
            "DARK and LIGHT presets should have different preview text colors"
        );
    }


    const EMOJI_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Apple Color Emoji.ttc",
        "C:/Windows/Fonts/seguiemj.ttf",
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
    ];

    #[test]
    #[cfg(target_os = "macos")]
    fn test_emoji_font_available_on_macos() {
        let result = load_first_font(EMOJI_CANDIDATES);
        assert!(
            result.is_some(),
            "Apple Color Emoji font should be available on macOS"
        );
    }

    #[test]
    fn test_emoji_font_is_not_in_proportional_family() {
        init_tracing();
        if load_first_font(EMOJI_CANDIDATES).is_none() {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, EMOJI_CANDIDATES);
        let proportional = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("Proportional family missing");
        let emoji_name = load_first_font(EMOJI_CANDIDATES).unwrap().0;
        assert!(
            !proportional.contains(&emoji_name),
            "Preview emoji should not replace UI fallback fonts in Proportional family"
        );
    }

    #[test]
    fn test_emoji_font_is_not_in_monospace_family() {
        init_tracing();
        if load_first_font(EMOJI_CANDIDATES).is_none() {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, EMOJI_CANDIDATES);
        let monospace = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let emoji_name = load_first_font(EMOJI_CANDIDATES).unwrap().0;
        assert!(
            !monospace.contains(&emoji_name),
            "Preview emoji should not replace UI fallback fonts in Monospace family"
        );
    }

    #[test]
    fn test_preset_has_emoji_font_candidates() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        let preset = DiagramColorPreset::current();
        assert!(
            !preset.emoji_font_candidates.is_empty(),
            "Preset must have at least one emoji font candidate"
        );
    }
}