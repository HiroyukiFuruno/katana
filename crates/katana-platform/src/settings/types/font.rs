use serde::{Deserialize, Serialize};

/// Minimum allowed font size in pixels.
pub const MIN_FONT_SIZE: f32 = 8.0;
/// Maximum allowed font size in pixels.
pub const MAX_FONT_SIZE: f32 = 32.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    #[serde(default = "super::super::defaults::default_font_size")]
    pub size: f32,
    #[serde(default = "super::super::defaults::default_font_family")]
    pub family: String,
}
