#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::hash_str;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub(crate) trait DocumentOps {
    fn handle_select_document(&mut self, path: std::path::PathBuf, activate: bool);
    fn force_close_document(&mut self, idx: usize);
    fn handle_update_buffer(&mut self, content: String);
    fn handle_replace_text(&mut self, span: std::ops::Range<usize>, replacement: String);
    fn handle_save_document(&mut self);
}

impl DocumentOps for KatanaApp {
    fn handle_select_document(&mut self, path: std::path::PathBuf, activate: bool) {
        // Auto-expand parents only when a file is explicitly activated (not during lazy background loads)
        if activate {
            let mut parent = path.parent();
            while let Some(p) = parent {
                if p == std::path::Path::new("") {
                    break;
                }
                self.state.expanded_directories.insert(p.to_path_buf());
                parent = p.parent();
            }
        }

        if let Some(idx) = self
            .state
            .open_documents
            .iter()
            .position(|d| d.path == path)
        {
            if activate {
                self.state.active_doc_idx = Some(idx);
                let doc = &mut self.state.open_documents[idx];
                if !doc.is_loaded {
                    if let Ok(loaded_doc) = self.fs.load_document(&path) {
                        *doc = loaded_doc;
                    }
                }
                let src = self.state.open_documents[idx].buffer.clone();
                let concurrency = self
                    .state
                    .settings
                    .settings()
                    .performance
                    .diagram_concurrency;
                self.full_refresh_preview(&path, &src, false, concurrency);
            }
            return;
        }

        if activate {
            match self.fs.load_document(&path) {
                Ok(doc) => {
                    let src = doc.buffer.clone();
                    let concurrency = self
                        .state
                        .settings
                        .settings()
                        .performance
                        .diagram_concurrency;
                    self.full_refresh_preview(&path, &src, false, concurrency);
                    self.state.open_documents.push(doc);
                    self.state.active_doc_idx = Some(self.state.open_documents.len() - 1);
                    self.state.initialize_tab_split_state(path.clone());
                    self.save_workspace_state();
                }
                Err(e) => {
                    let error = e.to_string();
                    self.state.status_message = Some((
                        crate::i18n::tf(
                            &crate::i18n::get().status.cannot_open_file,
                            &[("error", error.as_str())],
                        ),
                        crate::app_state::StatusType::Error,
                    ));
                }
            }
        } else {
            // Lazy load: just add to tabs
            self.state
                .open_documents
                .push(katana_core::document::Document::new_empty(path.clone()));
            self.state.initialize_tab_split_state(path);
            self.save_workspace_state();
        }
    }
    fn force_close_document(&mut self, idx: usize) {
        if idx < self.state.open_documents.len() {
            let closed_doc = self.state.open_documents.remove(idx);
            self.state.push_recently_closed(closed_doc.path);
            self.state.active_doc_idx = if self.state.open_documents.is_empty() {
                None
            } else {
                Some(
                    self.state
                        .active_doc_idx
                        .unwrap_or(0)
                        .saturating_sub(if self.state.active_doc_idx == Some(idx) {
                            1
                        } else {
                            0
                        })
                        .min(self.state.open_documents.len().saturating_sub(1)),
                )
            };
        }
        self.save_workspace_state();
        self.cleanup_closed_tab_previews();
    }
    fn handle_update_buffer(&mut self, content: String) {
        let path = if let Some(doc) = self.state.active_document_mut() {
            doc.update_buffer(content.clone());
            doc.path.clone()
        } else {
            return;
        };
        self.refresh_preview(&path, &content);
    }
    fn handle_replace_text(&mut self, span: std::ops::Range<usize>, replacement: String) {
        let (path, content) = if let Some(doc) = self.state.active_document_mut() {
            // Ensure the span is within bounds
            if span.start <= span.end && span.end <= doc.buffer.len() {
                doc.buffer.replace_range(span, &replacement);
                doc.is_dirty = true;
            }
            (doc.path.clone(), doc.buffer.clone())
        } else {
            return;
        };
        self.refresh_preview(&path, &content);
    }
    fn handle_save_document(&mut self) {
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        match self.fs.save_document(doc) {
            Ok(()) => {
                self.state.status_message = Some((
                    crate::i18n::get().status.saved.clone(),
                    crate::app_state::StatusType::Success,
                ));
                self.save_workspace_state();
            }
            Err(e) => {
                let error = e.to_string();
                self.state.status_message = Some((
                    crate::i18n::tf(
                        &crate::i18n::get().status.save_failed,
                        &[("error", error.as_str())],
                    ),
                    crate::app_state::StatusType::Error,
                ));
            }
        }
    }
}
