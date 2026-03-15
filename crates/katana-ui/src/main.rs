#![deny(warnings)]
//! Katana UI application entry point.

#[cfg(not(test))]
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::{ExtensionPoint, PluginMeta, PluginRegistry, PLUGIN_API_VERSION};
#[cfg(not(test))]
use katana_platform::{JsonFileRepository, SettingsService};
#[cfg(not(test))]
use katana_ui::app_state::AppState;
#[cfg(not(test))]
use katana_ui::shell::KatanaApp;

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

    // Initialize settings with JSON file persistence.
    let repo = JsonFileRepository::with_default_path();
    let settings = SettingsService::new(Box::new(repo));

    let state = AppState::new(ai_registry, plugin_registry, settings);

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
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);

            // macOS: eframe がウィンドウを生成した後にネイティブメニューバーを構築する。
            #[cfg(target_os = "macos")]
            unsafe {
                katana_ui::shell_ui::native_menu_setup();
            }

            Ok(Box::new(KatanaApp::new(state)))
        }),
    )
}

/// 日本語を含む CJK フォントを読み込んで egui に登録する。
///
/// macOS バンドルの AquaKana.ttc などを倪側フォントとして追加する。
pub fn setup_fonts(ctx: &egui::Context) {
    let candidates = [
        "/System/Library/Fonts/AquaKana.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
    ];
    setup_fonts_with_candidates(ctx, &candidates);
}

/// 候補パスを指定してフォントを読み込む内部関数（テスト可能）。
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
                // 追加フォントはフォールバックとして末尾に追加（日本語文字のみがこのフォントで描画される）。
                list.push(name.clone());
            }
        }
        tracing::info!("日本語フォントを読み込みました font={name}");
    } else {
        tracing::warn!("日本語フォントが見つかりませんでした。文字化けが発生する場合があります。");
    }
    ctx.set_fonts(fonts);

    // デバッグビルドで "First use of Grid ID XXXX" などの赤いオーバーレイが
    // プレビュー領域に表示される問題を防ぐ。
    ctx.style_mut(|style| {
        style.debug.debug_on_hover = false;
        style.debug.show_expand_width = false;
        style.debug.show_expand_height = false;
        style.debug.show_widget_hits = false;
    });
}

/// 候補パスの先頭から読めたフォントを返す。
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

    #[test]
    fn test_load_first_font_not_found() {
        let candidates = ["/invalid/path/to/never/found/font.ttc"];
        let result = load_first_font(&candidates);
        assert!(result.is_none());
    }

    #[test]
    fn test_setup_fonts() {
        let ctx = egui::Context::default();
        setup_fonts(&ctx);
        // Ensure no panics
    }

    #[test]
    fn test_setup_fonts_with_no_candidates() {
        // フォント候補が存在しないパスのみ → 警告パス (else ブランチ) がカバーされる
        let ctx = egui::Context::default();
        setup_fonts_with_candidates(&ctx, &["/nonexistent/font1.ttc", "/nonexistent/font2.ttc"]);
        // パニックしないこと、フォント設定がデフォルトのまま完了すること
    }

    #[test]
    fn test_register_builtin_plugins() {
        let mut registry = PluginRegistry::new();
        register_builtin_plugins(&mut registry);
        // Ensure it doesn't panic. No public field to check length easily.
    }
}
