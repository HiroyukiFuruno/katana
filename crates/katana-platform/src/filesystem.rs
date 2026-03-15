//! Filesystem service: reading workspace directories and loading/saving documents.

use katana_core::{
    document::{Document, DocumentError},
    workspace::{TreeEntry, Workspace, WorkspaceError},
};
use std::path::{Path, PathBuf};

/// Platform-layer filesystem service.
///
/// Implements all disk I/O so that higher layers (core, ui) remain free of
/// direct `std::fs` calls.
pub struct FilesystemService;

impl FilesystemService {
    pub fn new() -> Self {
        Self
    }

    /// Attempt to open `path` as a workspace root.
    ///
    /// On success, returns a [`Workspace`] with the directory tree populated.  
    /// On failure (unreadable path), returns a recoverable [`WorkspaceError`].
    pub fn open_workspace(&self, path: impl Into<PathBuf>) -> Result<Workspace, WorkspaceError> {
        let root: PathBuf = path.into();
        let tree = self
            .scan_directory(&root)
            .map_err(|e| WorkspaceError::unreadable_root(root.clone(), e))?;
        Ok(Workspace::new(root, tree))
    }

    /// Load a document from `path`, returning its in-memory representation.
    pub fn load_document(&self, path: impl Into<PathBuf>) -> Result<Document, DocumentError> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| DocumentError::read_failed(path.clone(), e))?;
        Ok(Document::new(path, content))
    }

    /// Save a document's current buffer to its source path on disk.
    ///
    /// This is the *only* path that writes to the source file. There is no
    /// implicit or background save.
    pub fn save_document(&self, doc: &mut Document) -> Result<(), DocumentError> {
        std::fs::write(&doc.path, &doc.buffer)
            .map_err(|e| DocumentError::save_failed(doc.path.clone(), e))?;
        doc.mark_clean();
        Ok(())
    }

    /// Recursively and in parallel scans a directory, returning a tree containing only `.md` files.
    fn scan_directory(&self, dir: &Path) -> std::io::Result<Vec<TreeEntry>> {
        use rayon::prelude::*;

        let iter = std::fs::read_dir(dir)?;
        let child_entries: Vec<_> = iter.filter_map(Result::ok).collect();

        let mut entries: Vec<TreeEntry> = child_entries
            .into_par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str())?;

                // Skip hidden files, build artifacts, and Node.js modules.
                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                {
                    return None;
                }

                if path.is_dir() {
                    let children = self.scan_directory(&path).unwrap_or_default();
                    // Do not show directories that contain no `.md` files underneath.
                    if has_any_markdown(&children) {
                        Some(TreeEntry::Directory { path, children })
                    } else {
                        None
                    }
                } else if path
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("md"))
                    .unwrap_or(false)
                {
                    Some(TreeEntry::File { path })
                } else {
                    None
                }
            })
            .collect();

        // Sort: directories first, then files, both alphabetically.
        entries.sort_by(|a, b| match (a, b) {
            (TreeEntry::Directory { .. }, TreeEntry::File { .. }) => std::cmp::Ordering::Less,
            (TreeEntry::File { .. }, TreeEntry::Directory { .. }) => std::cmp::Ordering::Greater,
            (a, b) => a.path().cmp(b.path()),
        });
        Ok(entries)
    }
}

/// Recursively checks if there is at least one `.md` file in the tree.
fn has_any_markdown(entries: &[TreeEntry]) -> bool {
    entries.iter().any(|e| match e {
        TreeEntry::File { .. } => e.is_markdown(),
        TreeEntry::Directory { children, .. } => has_any_markdown(children),
    })
}
impl Default for FilesystemService {
    fn default() -> Self {
        Self::new()
    }
}
