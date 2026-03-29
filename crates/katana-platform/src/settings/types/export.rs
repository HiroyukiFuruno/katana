use serde::{Deserialize, Serialize};

// WHY: Export-related settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    // WHY: Directory for HTML export output. Defaults to the system temp directory.
    #[serde(default = "super::super::defaults::default_html_output_dir")]
    pub html_output_dir: String,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            html_output_dir: crate::settings::defaults::default_html_output_dir(),
        }
    }
}
