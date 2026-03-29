use crate::filesystem::scanner::scan_directory;
use katana_core::{
    document::{Document, DocumentError},
    workspace::{Workspace, WorkspaceError},
};
use std::path::PathBuf;

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
    #[allow(clippy::too_many_arguments)]
    pub fn open_workspace(
        &self,
        path: impl Into<PathBuf>,
        ignored_directories: &[String],
        max_depth: usize,
        visible_extensions: &[String],
        extensionless_excludes: &[String],
        cancel_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
        in_memory_dirs: &std::collections::HashSet<PathBuf>,
    ) -> Result<Workspace, WorkspaceError> {
        let root: PathBuf = path.into();
        const ROOT_DEPTH: usize = 0;
        let tree = scan_directory(
            &root,
            ignored_directories,
            max_depth,
            ROOT_DEPTH,
            visible_extensions,
            extensionless_excludes,
            &cancel_token,
            in_memory_dirs,
        )
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
}

impl Default for FilesystemService {
    fn default() -> Self {
        Self::new()
    }
}
