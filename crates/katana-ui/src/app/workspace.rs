#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;
use katana_platform::CacheFacade;

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
        if self.state.workspace.data.is_some() {
            self.save_workspace_state();
        }

        self.state.workspace.is_loading = true;
        self.state.layout.status_message = Some((
            crate::i18n::tf(
                &crate::i18n::get().status.opened_workspace,
                &[("name", "...")],
            ),
            crate::app_state::StatusType::Info,
        ));

        let (tx, rx) = std::sync::mpsc::channel();
        self.workspace_rx = Some(rx);
        let path_clone = path.clone();

        if let Some(token) = &self.state.workspace.cancel_token {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        let new_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.workspace.cancel_token = Some(new_token.clone());

        let settings = self.state.config.settings.settings().workspace.clone();
        let in_memory_dirs = self.state.workspace.in_memory_dirs.clone();

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
        self.state.layout.status_message = Some((
            crate::i18n::tf(
                &crate::i18n::get().status.opened_workspace,
                &[("name", name.as_str())],
            ),
            crate::app_state::StatusType::Success,
        ));
        self.state.workspace.data = Some(ws);
        self.state.document.open_documents.clear();
        self.state.document.active_doc_idx = None;
        self.state.search.filter_cache = None;
        let path_str = path.display().to_string();

        let mut to_open = Vec::new();
        let mut active_idx = None;

        let cache_key = format!("workspace_tabs:{}", path_str);
        if let Some(cache_json) = self.state.config.cache.get_persistent(&cache_key) {
            #[derive(serde::Deserialize)]
            struct TabState {
                tabs: Vec<String>,
                active_idx: Option<usize>,
                #[serde(default)]
                expanded_directories: std::collections::HashSet<String>,
            }
            match serde_json::from_str::<TabState>(&cache_json) {
                Ok(state) => {
                    to_open = state.tabs;
                    active_idx = state.active_idx;
                    self.state.workspace.expanded_directories = state
                        .expanded_directories
                        .into_iter()
                        .map(std::path::PathBuf::from)
                        .collect();
                }
                Err(e) => {
                    tracing::warn!("Failed to deserialize tab state: {}", e);
                }
            }
        } else {
            let is_same_as_last = self
                .state
                .config
                .settings
                .settings()
                .workspace
                .last_workspace
                .as_deref()
                == Some(path_str.as_str());

            if is_same_as_last {
                let settings = self.state.config.settings.settings();
                to_open = settings.workspace.open_tabs.clone();
                active_idx = settings.workspace.active_tab_idx;
            }
        }

        {
            let settings = self.state.config.settings.settings_mut();
            settings.workspace.last_workspace = Some(path_str.clone());

            settings.workspace.paths.retain(|p| p != &path_str);
            settings.workspace.paths.push(path_str);
        }

        if !to_open.is_empty() {
            to_open.retain(|p| std::path::Path::new(p).exists());

            let active_idx_val = active_idx.unwrap_or(0).min(to_open.len().saturating_sub(1));

            for (i, p) in to_open.iter().enumerate() {
                let path = std::path::PathBuf::from(p);
                if i == active_idx_val {
                    match self.fs.load_document(path) {
                        Ok(doc) => {
                            self.state.document.open_documents.push(doc);
                            self.state
                                .initialize_tab_split_state(std::path::PathBuf::from(p));
                        }
                        Err(e) => {
                            tracing::error!("Failed to load document: {}", e);
                        }
                    }
                } else {
                    self.state
                        .document
                        .open_documents
                        .push(katana_core::document::Document::new_empty(path));
                    self.state
                        .initialize_tab_split_state(std::path::PathBuf::from(p));
                }
            }
            if !self.state.document.open_documents.is_empty() {
                let idx = active_idx
                    .unwrap_or(0)
                    .min(self.state.document.open_documents.len() - 1);
                self.state.document.active_doc_idx = Some(idx);
                let active_doc = &self.state.document.open_documents[idx];
                let src = active_doc.buffer.clone();
                let doc_path = active_doc.path.clone();
                let concurrency = self
                    .state
                    .config
                    .settings
                    .settings()
                    .performance
                    .diagram_concurrency;
                self.full_refresh_preview(&doc_path, &src, false, concurrency);
            }
        }

        if let Err(e) = self.state.config.settings.save() {
            tracing::warn!("Failed to save settings: {e}");
        }
    }
    fn handle_refresh_workspace(&mut self) {
        let Some(workspace) = &self.state.workspace.data else {
            return;
        };
        let root = workspace.root.clone();

        self.state.workspace.is_loading = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.workspace_rx = Some(rx);

        if let Some(token) = &self.state.workspace.cancel_token {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        let new_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.workspace.cancel_token = Some(new_token.clone());

        let settings = self.state.config.settings.settings().workspace.clone();
        let in_memory_dirs = self.state.workspace.in_memory_dirs.clone();

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
                    self.state.workspace.is_loading = false;
                    self.finish_open_workspace(path, ws);
                    true
                }
                Ok((WorkspaceLoadType::Refresh, _path, Ok(ws))) => {
                    self.state.workspace.is_loading = false;
                    self.state.workspace.data = Some(ws);
                    self.state.search.filter_cache = None;
                    true
                }
                Ok((_load_type, _path, Err(e))) => {
                    self.state.workspace.is_loading = false;
                    let error = e.to_string();
                    self.state.layout.status_message = Some((
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
                    self.state.workspace.is_loading = false;
                    true
                }
            }
        } else {
            false
        };
        if done {
            self.workspace_rx = None;
        }

        if self.needs_changelog_display
            && !self.state.workspace.is_loading
            && self.workspace_rx.is_none()
            && matches!(self.pending_action, AppAction::None)
        {
            self.needs_changelog_display = false;
            self.pending_action = AppAction::ShowReleaseNotes;
        }
    }
    fn handle_remove_workspace(&mut self, path: String) {
        let settings = self.state.config.settings.settings_mut();
        settings.workspace.paths.retain(|p| p != &path);

        if settings.workspace.last_workspace.as_deref() == Some(path.as_str()) {
            settings.workspace.last_workspace = None;
        }

        if let Err(e) = self.state.config.settings.save() {
            tracing::warn!("Failed to save settings after removing workspace: {e}");
        }
    }
    fn save_workspace_state(&mut self) {
        let open_tabs: Vec<String> = self
            .state
            .document
            .open_documents
            .iter()
            .map(|d| d.path.display().to_string())
            .filter(|p| !p.starts_with("Katana://"))
            .collect();
        let idx = self.state.document.active_doc_idx;
        let expanded: std::collections::HashSet<String> = self
            .state
            .workspace
            .expanded_directories
            .iter()
            .map(|p| p.display().to_string())
            .collect();

        let settings = self.state.config.settings.settings_mut();
        settings.workspace.open_tabs = open_tabs.clone();
        settings.workspace.active_tab_idx = idx;
        if let Err(e) = self.state.config.settings.save() {
            tracing::warn!("Failed to save workspace tab state: {e}");
        }

        if let Some(ws) = &self.state.workspace.data {
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
            match serde_json::to_string(&state) {
                Ok(json) => {
                    let _ = self.state.config.cache.set_persistent(&key, json);
                }
                Err(e) => {
                    tracing::warn!("Failed to serialize tab state: {}", e);
                }
            }
        }
    }
}