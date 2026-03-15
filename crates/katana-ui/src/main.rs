#![deny(warnings)]
//! Katana UI application entry point.

#[cfg(not(test))]
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::{ExtensionPoint, PluginMeta, PluginRegistry, PLUGIN_API_VERSION};
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
fn main() -> eframe::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "katana_ui=info,katana_core=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting Katana");

    // Initialize AI provider registry (no providers configured in MVP).
    let ai_registry = AiProviderRegistry::new();

    // Initialize plugin registry with static built-in registrations.
    let mut plugin_registry = PluginRegistry::new();
    register_builtin_plugins(&mut plugin_registry);

    let state = AppState::new(ai_registry, plugin_registry);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Katana")
            .with_inner_size(INITIAL_WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        "Katana",
        native_options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);

            // macOS: Construct the native menu bar after eframe creates the window.
            #[cfg(target_os = "macos")]
            unsafe {
                shell_ui::native_menu_setup();
            }

            Ok(Box::new(KatanaApp::new(state)))
        }),
    )
}

/// Loads CJK fonts including Japanese and registers them with egui.
///
/// Adds bundled macOS fonts like AquaKana.ttc as fallback fonts.
pub fn setup_fonts(ctx: &egui::Context) {
    let candidates = [
        "/System/Library/Fonts/AquaKana.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
    ];
    setup_fonts_with_candidates(ctx, &candidates);
}

/// Receives a list of font candidates and sets the fonts. Testable.
pub fn setup_fonts_with_candidates(ctx: &egui::Context, candidates: &[&str]) {
    let mut fonts = egui::FontDefinitions::default();
    let loaded = load_first_font(candidates);
    if let Some((name, data)) = loaded {
        fonts.font_data.insert(
            name.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(data)),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            if let Some(list) = fonts.families.get_mut(&family) {
                list.push(name.clone());
            }
        }
        tracing::info!("Loaded Japanese font font={name}");
    } else {
        tracing::warn!("Japanese font not found. Garbled text may occur.");
    }
    ctx.set_fonts(fonts);

    ctx.style_mut(|style| {
        style.debug.debug_on_hover = false;
        style.debug.show_expand_width = false;
        style.debug.show_expand_height = false;
        style.debug.show_widget_hits = false;
    });
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
        if result.is_some() {
            let (name, data) = result.unwrap();
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
}
