use katana_core::plugin::*;

fn make_meta(id: &str, api_version: u32, points: Vec<ExtensionPoint>) -> PluginMeta {
    PluginMeta {
        id: id.to_string(),
        name: id.to_string(),
        api_version,
        extension_points: points,
    }
}

#[test]
fn compatible_plugin_becomes_active() {
    let mut registry = PluginRegistry::new();
    registry.register(
        make_meta(
            "my-renderer",
            PLUGIN_API_VERSION,
            vec![ExtensionPoint::RendererEnhancement],
        ),
        || Ok(()),
    );
    assert_eq!(registry.status("my-renderer"), Some(&PluginStatus::Active));
}

#[test]
fn incompatible_version_is_rejected() {
    let mut registry = PluginRegistry::new();
    registry.register(
        make_meta("old-plugin", 999, vec![ExtensionPoint::AiTool]),
        || Ok(()),
    );
    assert_eq!(
        registry.status("old-plugin"),
        Some(&PluginStatus::IncompatibleVersion)
    );
}

#[test]
fn failing_init_disables_plugin_without_panic() {
    let mut registry = PluginRegistry::new();
    registry.register(
        make_meta(
            "bad-plugin",
            PLUGIN_API_VERSION,
            vec![ExtensionPoint::UiPanel],
        ),
        || Err("simulated startup failure".to_string()),
    );
    assert_eq!(registry.status("bad-plugin"), Some(&PluginStatus::Disabled));
}

#[test]
fn active_plugins_for_returns_only_matching_active() {
    let mut registry = PluginRegistry::new();
    registry.register(
        make_meta(
            "r1",
            PLUGIN_API_VERSION,
            vec![ExtensionPoint::RendererEnhancement],
        ),
        || Ok(()),
    );
    registry.register(
        make_meta("a1", PLUGIN_API_VERSION, vec![ExtensionPoint::AiTool]),
        || Ok(()),
    );
    registry.register(
        make_meta(
            "bad",
            PLUGIN_API_VERSION,
            vec![ExtensionPoint::RendererEnhancement],
        ),
        || Err("fail".to_string()),
    );

    let renderers = registry.active_plugins_for(&ExtensionPoint::RendererEnhancement);
    assert_eq!(renderers.len(), 1);
    assert_eq!(renderers[0].id, "r1");
}
