use std::path::PathBuf;
use thiserror::Error;

/// Represents a single document in a workspace.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Absolute path to the source file on disk.
    pub path: PathBuf,
    /// In-memory buffer content (may differ from disk if dirty).
    pub buffer: String,
    /// Whether the buffer has unsaved changes.
    pub is_dirty: bool,
}

impl Document {
    /// Create a new document with `content` loaded from `path`.
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            buffer: content.into(),
            is_dirty: false,
        }
    }

    /// Update the in-memory buffer content. Marks the document as dirty.
    pub fn update_buffer(&mut self, content: impl Into<String>) {
        let new = content.into();
        if self.buffer != new {
            self.buffer = new;
            self.is_dirty = true;
        }
    }

    /// Mark the document as clean (called after a successful save).
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    /// Returns the file name of this document, if available.
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }
}

/// Errors related to document operations.
#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Failed to read document at {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to save document to {path}: {source}")]
    SaveFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl DocumentError {
    pub fn read_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::ReadFailed {
            path: path.into(),
            source,
        }
    }

    pub fn save_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::SaveFailed {
            path: path.into(),
            source,
        }
    }
}
