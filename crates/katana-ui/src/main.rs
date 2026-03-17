#![deny(warnings)]
//! KatanA UI application entry point.

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
const INITIAL_WINDOW_SIZE: [f32; 2] = [1280.0, 800.0];

#[cfg(not(test))]
const MIN_WINDOW_SIZE: [f32; 2] = [800.0, 500.0];

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

#[cfg(not(test))]
fn main() -> eframe::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "katana_ui=info,katana_core=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting KatanA");

    // macOS: Set process name before any window is created.
    // This ensures the Dock label shows "KatanA" instead of "katana".
    #[cfg(target_os = "macos")]
    unsafe {
        shell_ui::native_set_process_name();
    }

    // Initialize AI provider registry (no providers configured in MVP).
    let ai_registry = AiProviderRegistry::new();

    // Initialize plugin registry with static built-in registrations.
    let mut plugin_registry = PluginRegistry::new();
    register_builtin_plugins(&mut plugin_registry);

    // Initialize settings with JSON file persistence.
    let repo = JsonFileRepository::with_default_path();
    let settings = SettingsService::new(Box::new(repo));

    // Read saved values before moving settings into AppState.
    let saved_language = settings.settings().language.clone();
    let saved_workspace = settings.settings().last_workspace.clone();

    let state = AppState::new(ai_registry, plugin_registry, settings);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("KatanA")
            .with_icon(load_icon())
            .with_inner_size(INITIAL_WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        "KatanA",
        native_options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            // Install lazy, parallel image loaders for file:// URIs in preview.
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // macOS: Construct the native menu bar and set app icon.
            #[cfg(target_os = "macos")]
            unsafe {
                shell_ui::native_menu_setup();
                let png_bytes = include_bytes!("../../../assets/icon.iconset/icon_512x512.png");
                shell_ui::native_set_app_icon_png(png_bytes.as_ptr(), png_bytes.len());
            }

            // Restore saved language.
            katana_ui::i18n::set_language(&saved_language);

            let mut app = KatanaApp::new(state);

            // Load icon texture for About dialog.
            let icon_png = include_bytes!("../../../assets/icon.iconset/icon_128x128.png");
            if let Ok(icon_image) = image::load_from_memory(icon_png) {
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

            // Restore last opened workspace.
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

/// Loads system fonts from the current preset and registers them with egui.
///
/// - **Proportional**: Uses `proportional_font_candidates` from preset (inserted at front).
/// - **Monospace**: Uses `monospace_font_candidates` from preset (inserted at front).
/// - Each loaded font is also added as a fallback in the OTHER family for CJK coverage.
pub fn setup_fonts(ctx: &egui::Context) {
    let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
    setup_fonts_from_preset(ctx, preset);
}

/// Configures fonts from a given preset. Testable.
pub fn setup_fonts_from_preset(
    ctx: &egui::Context,
    preset: &katana_core::markdown::color_preset::DiagramColorPreset,
) {
    let fonts = build_font_definitions(
        preset.proportional_font_candidates,
        preset.monospace_font_candidates,
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

/// Receives a list of font candidates and sets the fonts. Testable.
/// Kept for backward compatibility with existing tests.
pub fn setup_fonts_with_candidates(ctx: &egui::Context, candidates: &[&str]) {
    let fonts = build_font_definitions(candidates, &[]);
    ctx.set_fonts(fonts);

    #[cfg(debug_assertions)]
    ctx.style_mut(|style| {
        style.debug.debug_on_hover = false;
        style.debug.show_expand_width = false;
        style.debug.show_expand_height = false;
        style.debug.show_widget_hits = false;
    });
}

/// Builds `FontDefinitions` with system fonts inserted at the correct family positions.
///
/// - **Proportional**: loaded from `proportional_candidates`, inserted at position 0.
///   Also appended to Monospace as CJK fallback.
/// - **Monospace**: loaded from `monospace_candidates`, inserted at position 0 in Monospace family.
///   Also appended to Proportional as fallback.
///
/// This is a pure function for testability.
pub fn build_font_definitions(
    proportional_candidates: &[&str],
    monospace_candidates: &[&str],
) -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    // Load proportional font (e.g. Hiragino Sans for Japanese UI text).
    if let Some((name, data)) = load_first_font(proportional_candidates) {
        fonts.font_data.insert(
            name.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(data)),
        );
        // Primary in Proportional.
        if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            list.insert(0, name.clone());
        }
        // CJK fallback in Monospace.
        if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            list.push(name.clone());
        }
        tracing::info!("Loaded proportional font={name}");
    } else {
        tracing::warn!(
            "Proportional system font not found. Text rendering quality may be degraded."
        );
    }

    // Load monospace font (e.g. Menlo for code blocks).
    if let Some((name, data)) = load_first_font(monospace_candidates) {
        fonts.font_data.insert(
            name.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(data)),
        );
        // Primary in Monospace (inserted BEFORE CJK fallback).
        if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            list.insert(0, name.clone());
        }
        tracing::info!("Loaded monospace font={name}");
    } else {
        tracing::warn!("Monospace system font not found. Code blocks may render poorly.");
    }

    fonts
}

/// Returns the first readable font from the list of candidate paths.
pub fn load_first_font(candidates: &[&str]) -> Option<(String, Vec<u8>)> {
    for &path in candidates {
        if let Ok(data) = std::fs::read(path) {
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cjk_font")
                .to_string();
            return Some((name, data));
        }
    }
    None
}

/// Register all built-in plugins at startup.
///
/// In the MVP, all plugin registrations are static compile-time definitions.
/// No runtime manifest file is required.
pub fn register_builtin_plugins(registry: &mut PluginRegistry) {
    // Built-in Mermaid renderer plugin (placeholder; actual renderer bound in Task 4.2).
    registry.register(
        PluginMeta {
            id: "builtin-mermaid-renderer".to_string(),
            name: "Built-in Mermaid Renderer".to_string(),
            api_version: PLUGIN_API_VERSION,
            extension_points: vec![ExtensionPoint::RendererEnhancement],
        },
        || Ok(()), // Renderer logic is wired directly in the markdown pipeline.
    );

    // Built-in PlantUML renderer plugin (placeholder; actual renderer bound in Task 4.4).
    registry.register(
        PluginMeta {
            id: "builtin-plantuml-renderer".to_string(),
            name: "Built-in PlantUML Renderer".to_string(),
            api_version: PLUGIN_API_VERSION,
            extension_points: vec![ExtensionPoint::RendererEnhancement],
        },
        || Ok(()),
    );

    // Built-in Draw.io renderer plugin (placeholder; actual renderer bound in Task 4.3).
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
        let candidates = ["/invalid/path/to/never/found/font.ttc"];
        let result = load_first_font(&candidates);
        assert!(result.is_none());
    }

    #[test]
    fn test_load_first_font_found() {
        // Cover the success path using fonts that exist on macOS
        let candidates = [
            "/System/Library/Fonts/AquaKana.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ];
        let result = load_first_font(&candidates);
        // One of them should be found in a macOS environment
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
        // Only non-existent paths -> take the else (warn) path
        setup_fonts_with_candidates(&ctx, &["/nonexistent/font.ttc"]);
    }

    #[test]
    fn test_register_builtin_plugins() {
        init_tracing();
        let mut registry = PluginRegistry::new();
        register_builtin_plugins(&mut registry);
        assert_eq!(registry.active_count(), 3);
    }

    // ── Smoke Test: image loaders can be installed ──

    #[test]
    fn test_install_image_loaders_does_not_panic() {
        let ctx = egui::Context::default();
        // This must not panic. If egui_extras features are missing, it would.
        egui_extras::install_image_loaders(&ctx);
    }

    // ── Font Family Ordering Tests ──

    const PROP_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/AquaKana.ttc",
    ];

    const MONO_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Monaco.ttf",
    ];

    #[test]
    fn test_proportional_font_is_first_in_proportional_family() {
        init_tracing();
        if load_first_font(PROP_CANDIDATES).is_none() {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES);
        let proportional = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("Proportional family missing");
        let loaded_name = load_first_font(PROP_CANDIDATES).unwrap().0;
        assert_eq!(
            proportional[0], loaded_name,
            "Proportional font must be at position 0 in Proportional family"
        );
    }

    #[test]
    fn test_monospace_font_is_first_in_monospace_family() {
        init_tracing();
        if load_first_font(MONO_CANDIDATES).is_none() {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES);
        let monospace = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let mono_name = load_first_font(MONO_CANDIDATES).unwrap().0;
        assert_eq!(
            monospace[0], mono_name,
            "Monospace font (e.g. Menlo) must be at position 0 in Monospace family"
        );
    }

    #[test]
    fn test_proportional_font_is_cjk_fallback_in_monospace() {
        init_tracing();
        if load_first_font(PROP_CANDIDATES).is_none() || load_first_font(MONO_CANDIDATES).is_none()
        {
            return;
        }
        let fonts = build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES);
        let monospace = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let prop_name = load_first_font(PROP_CANDIDATES).unwrap().0;
        assert!(
            monospace.contains(&prop_name),
            "Proportional font should be in Monospace family as CJK fallback"
        );
        let mono_name = load_first_font(MONO_CANDIDATES).unwrap().0;
        let mono_pos = monospace.iter().position(|n| n == &mono_name).unwrap();
        let prop_pos = monospace.iter().position(|n| n == &prop_name).unwrap();
        assert!(
            mono_pos < prop_pos,
            "Monospace font must appear before proportional (CJK fallback)"
        );
    }

    #[test]
    fn test_build_font_definitions_without_candidates_returns_defaults() {
        init_tracing();
        let fonts = build_font_definitions(&["/nonexistent/font.ttc"], &[]);
        let proportional = fonts
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

    // ── Preset Integration Tests ──

    #[test]
    fn test_preset_syntax_themes_are_valid_identifiers() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        let preset = DiagramColorPreset::current();
        // Syntect themes must not be empty.
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
            DiagramColorPreset::DARK.preview_text,
            DiagramColorPreset::LIGHT.preview_text,
            "DARK and LIGHT presets should have different preview text colors"
        );
    }
}
