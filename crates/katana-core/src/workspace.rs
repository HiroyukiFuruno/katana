use std::path::{Path, PathBuf};
use thiserror::Error;

/// An entry in the workspace project tree.
#[derive(Debug, Clone, PartialEq)]
pub enum TreeEntry {
    File {
        path: PathBuf,
    },
    Directory {
        path: PathBuf,
        children: Vec<TreeEntry>,
    },
}

impl TreeEntry {
    pub fn path(&self) -> &Path {
        match self {
            Self::File { path } => path,
            Self::Directory { path, .. } => path,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.path().file_name()?.to_str()
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn is_markdown(&self) -> bool {
        match self {
            Self::File { path } => path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false),
            _ => false,
        }
    }
}

/// An open workspace rooted at a local directory.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Absolute path to the workspace root directory.
    pub root: PathBuf,
    /// Flat snapshot of the directory tree under `root`.
    pub tree: Vec<TreeEntry>,
}

impl Workspace {
    /// Build a workspace from a root path and a pre-scanned tree.
    pub fn new(root: impl Into<PathBuf>, tree: Vec<TreeEntry>) -> Self {
        Self {
            root: root.into(),
            tree,
        }
    }

    /// Returns workspace name (the root directory's base name), if available.
    pub fn name(&self) -> Option<&str> {
        self.root.file_name()?.to_str()
    }
}

/// Errors related to workspace operations.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Cannot read workspace directory at {path}: {source}")]
    UnreadableRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("No workspace is currently open")]
    NoWorkspace,
}

impl WorkspaceError {
    pub fn unreadable_root(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::UnreadableRoot {
            path: path.into(),
            source,
        }
    }
}
