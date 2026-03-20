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
            Self::File { path } => {
                let ext = path.extension();
                ext.map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn collect_all_directory_paths(&self, paths: &mut Vec<PathBuf>) {
        if let Self::Directory { path, children } = self {
            paths.push(path.clone());
            for child in children {
                child.collect_all_directory_paths(paths);
            }
        }
    }

    pub fn collect_all_markdown_file_paths(&self, paths: &mut Vec<PathBuf>) {
        match self {
            Self::File { path } => {
                if self.is_markdown() {
                    paths.push(path.clone());
                }
            }
            Self::Directory { children, .. } => {
                for child in children {
                    child.collect_all_markdown_file_paths(paths);
                }
            }
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

    pub fn collect_all_directory_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for entry in &self.tree {
            entry.collect_all_directory_paths(&mut paths);
        }
        paths
    }

    pub fn collect_all_markdown_file_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for entry in &self.tree {
            entry.collect_all_markdown_file_paths(&mut paths);
        }
        paths
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_collection() {
        let root = PathBuf::from("/root");
        let file1 = root.join("a.md");
        let sub = root.join("sub");
        let file2 = sub.join("b.md");

        let workspace = Workspace::new(
            root.clone(),
            vec![
                TreeEntry::File {
                    path: file1.clone(),
                },
                TreeEntry::Directory {
                    path: sub.clone(),
                    children: vec![TreeEntry::File {
                        path: file2.clone(),
                    }],
                },
            ],
        );

        let mds = workspace.collect_all_markdown_file_paths();
        assert_eq!(mds.len(), 2);
        assert!(mds.contains(&file1));
        assert!(mds.contains(&file2));

        let dirs = workspace.collect_all_directory_paths();
        assert_eq!(dirs.len(), 1);
        assert!(dirs.contains(&sub));
    }
}
