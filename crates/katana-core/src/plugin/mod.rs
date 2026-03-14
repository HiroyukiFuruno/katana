//! Plugin foundation: typed extension points, registry, and version contracts.
//!
//! MVP constraints:
//! - Built-in plugins only; no runtime manifest file required.
//! - All registrations happen at startup from compile-time definitions.
//! - Plugins failing to initialize are disabled, not fatal.

use std::collections::HashMap;

/// Plugin API contract version.
/// Plugins must declare compatibility with this version to be activated.
pub const PLUGIN_API_VERSION: u32 = 1;

/// The category of extension a plugin contributes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExtensionPoint {
    /// Contributes an additional diagram renderer.
    RendererEnhancement,
    /// Contributes an AI tool available in AI workflows.
    AiTool,
    /// Contributes a UI panel to the shell.
    UiPanel,
}

/// Metadata for a registered plugin.
#[derive(Debug, Clone)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub api_version: u32,
    pub extension_points: Vec<ExtensionPoint>,
}

/// Possible outcomes of initializing a plugin.
#[derive(Debug)]
pub enum PluginInitResult {
    /// Plugin initialized successfully.
    Ok,
    /// Plugin failed to initialize; it is disabled but does not crash the app.
    Failed(String),
    /// Plugin declared an incompatible API version.
    IncompatibleVersion { declared: u32, required: u32 },
}

/// Status of a plugin in the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Active,
    Disabled,
    IncompatibleVersion,
}

/// A registered plugin entry in the registry.
#[derive(Debug)]
struct PluginEntry {
    meta: PluginMeta,
    status: PluginStatus,
}

/// The plugin registry: assembled at startup from static built-in definitions.
#[derive(Default)]
pub struct PluginRegistry {
    entries: HashMap<String, PluginEntry>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register and initialize a plugin described by `meta`, using `init_fn`
    /// to perform any startup work.
    ///
    /// - Incompatible API version → marked `IncompatibleVersion`.
    /// - `init_fn` returns `Err` → marked `Disabled`, application continues.
    pub fn register<F>(&mut self, meta: PluginMeta, init_fn: F)
    where
        F: FnOnce() -> Result<(), String>,
    {
        let status = if meta.api_version != PLUGIN_API_VERSION {
            tracing::warn!(
                plugin_id = %meta.id,
                declared = meta.api_version,
                required = PLUGIN_API_VERSION,
                "Plugin rejected: incompatible API version"
            );
            PluginStatus::IncompatibleVersion
        } else {
            match init_fn() {
                Ok(()) => {
                    tracing::info!(plugin_id = %meta.id, "Plugin registered successfully");
                    PluginStatus::Active
                }
                Err(e) => {
                    tracing::warn!(plugin_id = %meta.id, error = %e, "Plugin disabled due to init failure");
                    PluginStatus::Disabled
                }
            }
        };
        self.entries
            .insert(meta.id.clone(), PluginEntry { meta, status });
    }

    /// Return metadata for all plugins that are active and contribute to `point`.
    pub fn active_plugins_for(&self, point: &ExtensionPoint) -> Vec<&PluginMeta> {
        self.entries
            .values()
            .filter(|e| e.status == PluginStatus::Active)
            .filter(|e| e.meta.extension_points.contains(point))
            .map(|e| &e.meta)
            .collect()
    }

    /// Status of a plugin by ID.
    pub fn status(&self, id: &str) -> Option<&PluginStatus> {
        self.entries.get(id).map(|e| &e.status)
    }

    /// Total number of active plugins.
    pub fn active_count(&self) -> usize {
        self.entries
            .values()
            .filter(|e| e.status == PluginStatus::Active)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
