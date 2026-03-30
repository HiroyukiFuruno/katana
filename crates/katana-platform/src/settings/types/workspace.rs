use serde::{Deserialize, Serialize};

// WHY: Default maximum recursion depth for workspace scanning.
pub const DEFAULT_MAX_DEPTH: usize = 10;

// WHY: Default list of directory names to ignore during workspace scanning.
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
    // WHY: ID of the last opened workspace root path, restored on next launch.
    #[serde(default)]
    pub last_workspace: Option<String>,
    // WHY: Workspace directory paths.
    #[serde(default)]
    pub paths: Vec<String>,
    // WHY: Previously opened document tabs.
    #[serde(default)]
    pub open_tabs: Vec<String>,
    // WHY: Index of the actively selected tab.
    #[serde(default)]
    pub active_tab_idx: Option<usize>,
    // WHY: Directories to ignore during workspace scanning.
    #[serde(default = "super::super::defaults::default_ignored_directories")]
    pub ignored_directories: Vec<String>,
    // WHY: Maximum depth for recursive directory scanning.
    #[serde(default = "super::super::defaults::default_max_depth")]
    pub max_depth: usize,
    // WHY: Visible extensions in the workspace tree.
    #[serde(default = "super::super::defaults::default_visible_extensions")]
    pub visible_extensions: Vec<String>,

    // WHY: Excluded exact file names when "no extension" files are visible.
    #[serde(default = "super::super::defaults::default_extensionless_excludes")]
    pub extensionless_excludes: Vec<String>,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            last_workspace: None,
            paths: vec![],
            open_tabs: vec![],
            active_tab_idx: None,
            ignored_directories: crate::settings::defaults::default_ignored_directories(),
            max_depth: DEFAULT_MAX_DEPTH,
            visible_extensions: crate::settings::defaults::default_visible_extensions(),
            extensionless_excludes: crate::settings::defaults::default_extensionless_excludes(),
        }
    }
}