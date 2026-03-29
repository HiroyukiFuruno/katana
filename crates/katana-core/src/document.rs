use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub path: PathBuf,
    pub buffer: String,
    pub is_dirty: bool,
    pub is_loaded: bool,
    pub is_pinned: bool,
}

impl Document {
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            buffer: content.into(),
            is_dirty: false,
            is_loaded: true,
            is_pinned: false,
        }
    }

    pub fn new_empty(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            buffer: String::new(),
            is_dirty: false,
            is_loaded: false,
            is_pinned: false,
        }
    }

    pub fn update_buffer(&mut self, content: impl Into<String>) {
        let new = content.into();
        if self.buffer != new {
            self.buffer = new;
            self.is_dirty = true;
        }
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new_empty() {
        let path = PathBuf::from("test.md");
        let doc = Document::new_empty(path.clone());
        assert_eq!(doc.path, path);
        assert!(doc.buffer.is_empty());
        assert!(!doc.is_loaded);
    }
}
