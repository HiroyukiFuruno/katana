//! Katana UI application entry point.

mod app_state;
mod preview_pane;
mod shell;

use app_state::AppState;
use katana_core::{
    ai::AiProviderRegistry,
    plugin::{ExtensionPoint, PluginMeta, PluginRegistry, PLUGIN_API_VERSION},
};
use shell::KatanaApp;

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
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Katana",
        native_options,
        Box::new(|_cc| Ok(Box::new(KatanaApp::new(state)))),
    )
}

/// Register all built-in plugins at startup.
///
/// In the MVP, all plugin registrations are static compile-time definitions.
/// No runtime manifest file is required.
fn register_builtin_plugins(registry: &mut PluginRegistry) {
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
