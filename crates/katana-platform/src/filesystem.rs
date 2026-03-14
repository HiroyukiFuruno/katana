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

    /// Recursively scan a directory and return a flat tree.
    fn scan_directory(&self, dir: &Path) -> std::io::Result<Vec<TreeEntry>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            // Skip hidden files and common non-content directories.
            if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                continue;
            }
            if path.is_dir() {
                let children = self.scan_directory(&path).unwrap_or_default();
                entries.push(TreeEntry::Directory { path, children });
            } else {
                entries.push(TreeEntry::File { path });
            }
        }
        // Sort: directories first, then files, both alphabetically.
        entries.sort_by(|a, b| match (a, b) {
            (TreeEntry::Directory { .. }, TreeEntry::File { .. }) => std::cmp::Ordering::Less,
            (TreeEntry::File { .. }, TreeEntry::Directory { .. }) => std::cmp::Ordering::Greater,
            (a, b) => a.path().cmp(b.path()),
        });
        Ok(entries)
    }
}

impl Default for FilesystemService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_workspace() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("README.md"), "# Workspace").unwrap();
        fs::write(dir.path().join("spec.md"), "## Spec").unwrap();
        fs::create_dir(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("docs").join("arch.md"), "## Architecture").unwrap();
        dir
    }

    #[test]
    fn open_valid_workspace_returns_workspace() {
        let tmp = setup_workspace();
        let svc = FilesystemService::new();
        let ws = svc.open_workspace(tmp.path()).unwrap();
        assert_eq!(ws.root, tmp.path());
        assert!(!ws.tree.is_empty());
    }

    #[test]
    fn open_invalid_workspace_returns_error() {
        let svc = FilesystemService::new();
        let result = svc.open_workspace("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn load_document_reads_content() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.md");
        fs::write(&path, "# Hello World").unwrap();
        let svc = FilesystemService::new();
        let doc = svc.load_document(&path).unwrap();
        assert_eq!(doc.buffer, "# Hello World");
        assert!(!doc.is_dirty);
    }

    #[test]
    fn save_document_writes_buffer_and_marks_clean() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("doc.md");
        fs::write(&path, "original").unwrap();
        let svc = FilesystemService::new();
        let mut doc = svc.load_document(&path).unwrap();
        doc.update_buffer("# Updated Content");
        assert!(doc.is_dirty);
        svc.save_document(&mut doc).unwrap();
        assert!(!doc.is_dirty);
        assert_eq!(fs::read_to_string(&path).unwrap(), "# Updated Content");
    }

    #[test]
    fn edit_without_explicit_save_does_not_change_disk() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("doc.md");
        fs::write(&path, "original").unwrap();
        let svc = FilesystemService::new();
        let mut doc = svc.load_document(&path).unwrap();
        doc.update_buffer("edited but not saved");
        // Disk unchanged.
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    }

    #[test]
    fn load_nonexistent_document_returns_error() {
        let svc = FilesystemService::new();
        let result = svc.load_document("/this/does/not/exist.md");
        assert!(result.is_err());
    }
}
