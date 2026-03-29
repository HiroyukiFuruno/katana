use serde::{Deserialize, Serialize};

/// Export-related settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Directory for HTML export output. Defaults to the system temp directory.
    #[serde(default = "super::super::defaults::default_html_output_dir")]
    pub html_output_dir: String,
}
