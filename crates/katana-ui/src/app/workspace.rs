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

pub(crate) trait WorkspaceOps {
    fn handle_open_workspace(&mut self, path: std::path::PathBuf);
    fn finish_open_workspace(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    );
    fn handle_refresh_workspace(&mut self);
    fn poll_workspace_load(&mut self, ctx: &egui::Context);
    fn handle_remove_workspace(&mut self, path: String);
    fn save_workspace_state(&mut self);
}

impl WorkspaceOps for KatanaApp {
    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        // Save current workspace state (including open tabs) before unloading it
        if self.state.workspace.is_some() {
            self.save_workspace_state();
        }

        self.state.is_loading_workspace = true;
        // Temporary feedback
        self.state.status_message = Some((
            crate::i18n::tf(
                &crate::i18n::get().status.opened_workspace,
                &[("name", "...")],
            ),
            crate::app_state::StatusType::Info,
        ));

        let (tx, rx) = std::sync::mpsc::channel();
        self.workspace_rx = Some(rx);
        let path_clone = path.clone();

        // 1. Cancel any existing workspace scan
        if let Some(token) = &self.state.workspace_cancel_token {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // 2. Create a new cancellation token for this scan
        let new_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.workspace_cancel_token = Some(new_token.clone());

        let settings = self.state.settings.settings().workspace.clone();
        let in_memory_dirs = self.state.in_memory_dirs.clone();

        std::thread::spawn(move || {
            let fs = katana_platform::FilesystemService::new();
            let result = fs.open_workspace(
                &path_clone,
                &settings.ignored_directories,
                settings.max_depth,
                &settings.visible_extensions,
                &settings.extensionless_excludes,
                new_token,
                &in_memory_dirs,
            );
            let _ = tx.send((WorkspaceLoadType::Open, path_clone, result));
        });
    }
    fn finish_open_workspace(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    ) {
        let name = ws.name().unwrap_or("unknown").to_string();
        self.state.status_message = Some((
            crate::i18n::tf(
                &crate::i18n::get().status.opened_workspace,
                &[("name", name.as_str())],
            ),
            crate::app_state::StatusType::Success,
        ));
        self.state.workspace = Some(ws);
        self.state.open_documents.clear();
        self.state.active_doc_idx = None;
        self.state.filter_cache = None;
        let path_str = path.display().to_string();

        let mut to_open = Vec::new();
        let mut active_idx = None;

        let cache_key = format!("workspace_tabs:{}", path_str);
        if let Some(cache_json) = self.state.cache.get_persistent(&cache_key) {
            #[derive(serde::Deserialize)]
            struct TabState {
                tabs: Vec<String>,
                active_idx: Option<usize>,
                #[serde(default)]
                expanded_directories: std::collections::HashSet<String>,
            }
            if let Ok(state) = serde_json::from_str::<TabState>(&cache_json) {
                to_open = state.tabs;
                active_idx = state.active_idx;
                self.state.expanded_directories = state
                    .expanded_directories
                    .into_iter()
                    .map(std::path::PathBuf::from)
                    .collect();
            }
        } else {
            // Fallback for first time after migration
            let is_same_as_last = self
                .state
                .settings
                .settings()
                .workspace
                .last_workspace
                .as_deref()
                == Some(path_str.as_str());

            if is_same_as_last {
                let settings = self.state.settings.settings();
                to_open = settings.workspace.open_tabs.clone();
                active_idx = settings.workspace.active_tab_idx;
            }
        }

        // Persist the last opened workspace path.
        {
            let settings = self.state.settings.settings_mut();
            settings.workspace.last_workspace = Some(path_str.clone());

            // Move to end of history (most recent)
            settings.workspace.paths.retain(|p| p != &path_str);
            settings.workspace.paths.push(path_str);
        }

        // Restore tabs for the opened workspace
        if !to_open.is_empty() {
            // Retain only files that exist
            to_open.retain(|p| std::path::Path::new(p).exists());

            let active_idx_val = active_idx.unwrap_or(0).min(to_open.len().saturating_sub(1));

            for (i, p) in to_open.iter().enumerate() {
                let path = std::path::PathBuf::from(p);
                if i == active_idx_val {
                    if let Ok(doc) = self.fs.load_document(path) {
                        self.state.open_documents.push(doc);
                        self.state
                            .initialize_tab_split_state(std::path::PathBuf::from(p));
                    }
                } else {
                    // Lazy load non-active tabs
                    self.state
                        .open_documents
                        .push(katana_core::document::Document::new_empty(path));
                    self.state
                        .initialize_tab_split_state(std::path::PathBuf::from(p));
                }
            }
            if !self.state.open_documents.is_empty() {
                let idx = active_idx
                    .unwrap_or(0)
                    .min(self.state.open_documents.len() - 1);
                self.state.active_doc_idx = Some(idx);
                let active_doc = &self.state.open_documents[idx];
                let src = active_doc.buffer.clone();
                let doc_path = active_doc.path.clone();
                let concurrency = self
                    .state
                    .settings
                    .settings()
                    .performance
                    .diagram_concurrency;
                self.full_refresh_preview(&doc_path, &src, false, concurrency);
            }
        }

        if let Err(e) = self.state.settings.save() {
            tracing::warn!("Failed to save settings: {e}");
        }
    }
    fn handle_refresh_workspace(&mut self) {
        let Some(workspace) = &self.state.workspace else {
            return;
        };
        let root = workspace.root.clone();

        self.state.is_loading_workspace = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.workspace_rx = Some(rx);

        // 1. Cancel any existing workspace scan
        if let Some(token) = &self.state.workspace_cancel_token {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // 2. Create a new cancellation token for this scan
        let new_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.workspace_cancel_token = Some(new_token.clone());

        let settings = self.state.settings.settings().workspace.clone();
        let in_memory_dirs = self.state.in_memory_dirs.clone();

        std::thread::spawn(move || {
            let fs = katana_platform::FilesystemService::new();
            let result = fs.open_workspace(
                &root,
                &settings.ignored_directories,
                settings.max_depth,
                &settings.visible_extensions,
                &settings.extensionless_excludes,
                new_token,
                &in_memory_dirs,
            );
            let _ = tx.send((WorkspaceLoadType::Refresh, root, result));
        });
    }
    fn poll_workspace_load(&mut self, ctx: &egui::Context) {
        const WORKSPACE_LOAD_POLL_INTERVAL_MS: u64 = 50;
        let done = if let Some(rx) = &self.workspace_rx {
            match rx.try_recv() {
                Ok((WorkspaceLoadType::Open, path, Ok(ws))) => {
                    self.state.is_loading_workspace = false;
                    self.finish_open_workspace(path, ws);
                    true
                }
                Ok((WorkspaceLoadType::Refresh, _path, Ok(ws))) => {
                    self.state.is_loading_workspace = false;
                    self.state.workspace = Some(ws);
                    self.state.filter_cache = None;
                    true
                }
                Ok((_load_type, _path, Err(e))) => {
                    self.state.is_loading_workspace = false;
                    let error = e.to_string();
                    self.state.status_message = Some((
                        crate::i18n::tf(
                            &crate::i18n::get().status.cannot_open_workspace,
                            &[("error", error.as_str())],
                        ),
                        crate::app_state::StatusType::Error,
                    ));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    tracing::debug!("[CPU_LEAK] shell.rs Workspace load polling requested repaint");
                    ctx.request_repaint_after(std::time::Duration::from_millis(
                        WORKSPACE_LOAD_POLL_INTERVAL_MS,
                    ));
                    false
                }
                Err(_) => {
                    tracing::debug!("[CPU_LEAK] shell.rs workspace_rx disconnected (err)");
                    self.state.is_loading_workspace = false;
                    true
                }
            }
        } else {
            false
        };
        if done {
            self.workspace_rx = None;
        }

        // Feature: Automatically show the changelog after updating the app.
        // We defer this action until the initial workspace has finished loading so that
        // the newly opened changelog tab doesn't get wiped by `finish_open_workspace`.
        if self.needs_changelog_display
            && !self.state.is_loading_workspace
            && self.workspace_rx.is_none()
            && matches!(self.pending_action, AppAction::None)
        {
            self.needs_changelog_display = false;
            self.pending_action = AppAction::ShowReleaseNotes;
        }
    }
    fn handle_remove_workspace(&mut self, path: String) {
        let settings = self.state.settings.settings_mut();
        settings.workspace.paths.retain(|p| p != &path);

        if settings.workspace.last_workspace.as_deref() == Some(path.as_str()) {
            settings.workspace.last_workspace = None;
        }

        if let Err(e) = self.state.settings.save() {
            tracing::warn!("Failed to save settings after removing workspace: {e}");
        }
    }
    fn save_workspace_state(&mut self) {
        let open_tabs: Vec<String> = self
            .state
            .open_documents
            .iter()
            .map(|d| d.path.display().to_string())
            .filter(|p| !p.starts_with("Katana://"))
            .collect();
        let idx = self.state.active_doc_idx;
        let expanded: std::collections::HashSet<String> = self
            .state
            .expanded_directories
            .iter()
            .map(|p| p.display().to_string())
            .collect();

        let settings = self.state.settings.settings_mut();
        settings.workspace.open_tabs = open_tabs.clone();
        settings.workspace.active_tab_idx = idx;
        if let Err(e) = self.state.settings.save() {
            tracing::warn!("Failed to save workspace tab state: {e}");
        }

        if let Some(ws) = &self.state.workspace {
            let key = format!("workspace_tabs:{}", ws.root.display());
            #[derive(serde::Serialize)]
            struct TabState {
                tabs: Vec<String>,
                active_idx: Option<usize>,
                expanded_directories: std::collections::HashSet<String>,
            }
            let state = TabState {
                tabs: open_tabs,
                active_idx: idx,
                expanded_directories: expanded,
            };
            if let Ok(json) = serde_json::to_string(&state) {
                let _ = self.state.cache.set_persistent(&key, json);
            }
        }
    }
}
