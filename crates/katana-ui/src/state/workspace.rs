use katana_core::workspace::Workspace;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct WorkspaceState {
    pub data: Option<Workspace>,
    pub cancel_token: Option<Arc<AtomicBool>>,
    pub is_loading: bool,
    pub expanded_directories: HashSet<PathBuf>,
    pub in_memory_dirs: HashSet<PathBuf>,
    pub force_tree_open: Option<bool>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceState {
    pub fn new() -> Self {
        Self {
            data: None,
            cancel_token: None,
            is_loading: false,
            expanded_directories: HashSet::new(),
            in_memory_dirs: HashSet::new(),
            force_tree_open: None,
        }
    }
}
