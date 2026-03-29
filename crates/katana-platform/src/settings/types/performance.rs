use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    // WHY: Number of concurrent diagram renders.
    pub diagram_concurrency: usize,
    // WHY: Number of days to retain HTTP image cache.
    #[serde(default = "super::super::defaults::default_cache_retention")]
    pub http_image_cache_retention_days: u32,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            diagram_concurrency: crate::settings::defaults::DEFAULT_DIAGRAM_CONCURRENCY,
            http_image_cache_retention_days: crate::settings::defaults::default_cache_retention(),
        }
    }
}
