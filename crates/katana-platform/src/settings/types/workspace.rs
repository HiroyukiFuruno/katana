use serde::{Deserialize, Serialize};

/// Default maximum recursion depth for workspace scanning.
pub const DEFAULT_MAX_DEPTH: usize = 10;

/// Default list of directory names to ignore during workspace scanning.
pub const DEFAULT_IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    ".terraform",
    "node_modules",
    "target",
    ".idea",
    ".vscode",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// ID of the last opened workspace root path, restored on next launch.
    #[serde(default)]
    pub last_workspace: Option<String>,
    /// Workspace directory paths.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Previously opened document tabs.
    #[serde(default)]
    pub open_tabs: Vec<String>,
    /// Index of the actively selected tab.
    #[serde(default)]
    pub active_tab_idx: Option<usize>,
    /// Directories to ignore during workspace scanning.
    #[serde(default = "super::super::defaults::default_ignored_directories")]
    pub ignored_directories: Vec<String>,
    /// Maximum depth for recursive directory scanning.
    #[serde(default = "super::super::defaults::default_max_depth")]
    pub max_depth: usize,
    /// Visible extensions in the workspace tree.
    #[serde(default = "super::super::defaults::default_visible_extensions")]
    pub visible_extensions: Vec<String>,

    /// Excluded exact file names when "no extension" files are visible.
    #[serde(default = "super::super::defaults::default_extensionless_excludes")]
    pub extensionless_excludes: Vec<String>,
}
