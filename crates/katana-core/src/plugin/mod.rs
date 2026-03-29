/* WHY: Plugin foundation: typed extension points, registry, and version contracts.
MVP constraints:
- Built-in plugins only; no runtime manifest file required.
- All registrations happen at startup from compile-time definitions.
- Plugins failing to initialize are disabled, not fatal. */

pub const PLUGIN_API_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExtensionPoint {
    RendererEnhancement,
    AiTool,
    UiPanel,
}

#[derive(Debug, Clone)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub api_version: u32,
    pub extension_points: Vec<ExtensionPoint>,
}

#[derive(Debug)]
pub enum PluginInitResult {
    Ok,
    Failed(String),
    IncompatibleVersion { declared: u32, required: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Active,
    Disabled,
    IncompatibleVersion,
}

#[derive(Debug)]
struct PluginEntry {
    meta: PluginMeta,
    status: PluginStatus,
}

#[derive(Default)]
pub struct PluginRegistry {
    entries: Vec<PluginEntry>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /* WHY: Register and initialize a plugin described by `meta`, using `init_fn` to perform any startup work.
    Incompatible API version → marked `IncompatibleVersion`. `init_fn` returns `Err` → marked `Disabled`, application continues. */
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
        let id = meta.id.clone();
        if let Some(entry) = self.entries.iter_mut().find(|e| e.meta.id == id) {
            entry.meta = meta;
            entry.status = status;
        } else {
            self.entries.push(PluginEntry { meta, status });
        }
    }

    pub fn active_plugins_for(&self, point: &ExtensionPoint) -> Vec<&PluginMeta> {
        let mut result = Vec::new();
        for entry in &self.entries {
            if entry.status == PluginStatus::Active && entry.meta.extension_points.contains(point) {
                result.push(&entry.meta);
            }
        }
        result
    }

    pub fn status(&self, id: &str) -> Option<&PluginStatus> {
        match self.entries.iter().find(|e| e.meta.id == id) {
            Some(entry) => Some(&entry.status),
            None => None,
        }
    }

    pub fn active_count(&self) -> usize {
        let mut count = 0;
        for entry in &self.entries {
            if entry.status == PluginStatus::Active {
                count += 1;
            }
        }
        count
    }
}
