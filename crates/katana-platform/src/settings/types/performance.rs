use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Number of concurrent diagram renders.
    pub diagram_concurrency: usize,
    /// Number of days to retain HTTP image cache.
    #[serde(default = "super::super::defaults::default_cache_retention")]
    pub http_image_cache_retention_days: u32,
}
