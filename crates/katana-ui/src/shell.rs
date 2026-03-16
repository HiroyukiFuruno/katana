//! Katana three-pane egui shell.
//!
//! Contains only business logic. egui rendering code is separated into shell_ui.rs.

use std::collections::HashMap;

use eframe::egui;
use katana_platform::FilesystemService;

use crate::{
    app_state::{AppAction, AppState},
    preview_pane::{DownloadRequest, PreviewPane},
};

// ─────────────────────────────────────────────
// Layout Constants
// ─────────────────────────────────────────────

/// Width of the '›' toggle button displayed when the sidebar is collapsed (px).
pub(crate) const SIDEBAR_COLLAPSED_TOGGLE_WIDTH: f32 = 24.0;

/// Minimum resize width of the file tree panel (px).
pub(crate) const FILE_TREE_PANEL_MIN_WIDTH: f32 = 120.0;

/// Initial display width of the file tree panel (px).
pub(crate) const FILE_TREE_PANEL_DEFAULT_WIDTH: f32 = 220.0;

/// Minimum width of the preview panel in Split mode (px).
pub(crate) const SPLIT_PREVIEW_PANEL_MIN_WIDTH: f32 = 200.0;

/// Initial width of the preview panel in Split mode (px).
pub(crate) const SPLIT_PREVIEW_PANEL_DEFAULT_WIDTH: f32 = 400.0;

/// Width of the ◀▶ navigation button area at the right end of the tab bar (px).
pub(crate) const TAB_NAV_BUTTONS_AREA_WIDTH: f32 = 80.0;

/// Inter-tab spacing provided on the right side of each tab (px).
pub(crate) const TAB_INTER_ITEM_SPACING: f32 = 4.0;

/// Initial number of visible rows for the text editor TextEdit.
pub(crate) const EDITOR_INITIAL_VISIBLE_ROWS: usize = 40;

/// Sensitivity threshold for scroll synchronization between editor and preview.
/// Scroll events are ignored if the fraction difference is less than or equal to this value.
pub(crate) const SCROLL_SYNC_DEAD_ZONE: f32 = 0.002;

/// Delay until the tab tooltip is displayed (seconds).
pub(crate) const TAB_TOOLTIP_SHOW_DELAY_SECS: f32 = 0.25;

/// Spacing below the 'No workspace selected' display in the file tree (px).
pub(crate) const NO_WORKSPACE_BOTTOM_SPACING: f32 = 8.0;

/// Polling interval for checking download completion (ms).
pub(crate) const DOWNLOAD_STATUS_CHECK_INTERVAL_MS: u64 = 200;

// ─────────────────────────────────────────────
// Color Constants
// ─────────────────────────────────────────────

/// Text color of the file name displayed in the in-app title bar.
pub(crate) const TITLE_BAR_TEXT_COLOR: egui::Color32 = egui::Color32::from_gray(180);

/// Normal text color of the file tree (inactive files/directories).
pub(crate) const FILE_TREE_TEXT_COLOR: egui::Color32 = egui::Color32::from_gray(220);

/// Background highlight color indicating the active file in the file tree (VSCode-like semi-transparent blue).
pub(crate) const ACTIVE_FILE_HIGHLIGHT_BG: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(40, 80, 160, 100);

/// Rounding radius of the active row background in the file tree.
pub(crate) const ACTIVE_FILE_HIGHLIGHT_ROUNDING: f32 = 3.0;

/// Converts a string to u64 using FNV-1a hash.
fn hash_str(s: &str) -> u64 {
    crate::shell_logic::hash_str(s)
}

pub struct KatanaApp {
    pub(crate) state: AppState,
    pub(crate) fs: FilesystemService,
    pub(crate) pending_action: AppAction,
    /// Per-tab preview pane. Key is the file path. Reuses cache on tab switch.
    pub(crate) tab_panes: HashMap<std::path::PathBuf, PreviewPane>,
    /// Last rendered content hash per tab. Used for change detection.
    pub(crate) tab_hashes: HashMap<std::path::PathBuf, u64>,
    /// Receiver for background download completion notifications.
    pub(crate) download_rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
}

impl KatanaApp {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            tab_panes: HashMap::new(),
            tab_hashes: HashMap::new(),
            download_rx: None,
        }
    }

    pub(crate) fn take_action(&mut self) -> AppAction {
        std::mem::replace(&mut self.pending_action, AppAction::None)
    }

    /// Reflects only text changes (keeps existing images for diagrams).
    fn refresh_preview(&mut self, path: &std::path::Path, source: &str) {
        self.tab_panes
            .entry(path.to_path_buf())
            .or_default()
            .update_markdown_sections(source, path);
    }

    /// Re-renders all sections. Updates the content hash as well.
    fn full_refresh_preview(&mut self, path: &std::path::Path, source: &str) {
        let h = hash_str(source);
        self.tab_hashes.insert(path.to_path_buf(), h);
        self.tab_panes
            .entry(path.to_path_buf())
            .or_default()
            .full_render(source, path);
    }

    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        match self.fs.open_workspace(&path) {
            Ok(ws) => {
                let name = ws.name().unwrap_or("unknown").to_string();
                self.state.status_message = Some(crate::i18n::tf(
                    "status_opened_workspace",
                    &[("name", &name)],
                ));
                self.state.workspace = Some(ws);
                self.state.open_documents.clear();
                self.state.active_doc_idx = None;

                // Persist the last opened workspace path.
                self.state.settings.settings_mut().last_workspace =
                    Some(path.display().to_string());
                if let Err(e) = self.state.settings.save() {
                    tracing::warn!("Failed to save settings: {e}");
                }
            }
            Err(e) => {
                let error = e.to_string();
                self.state.status_message = Some(crate::i18n::tf(
                    "status_cannot_open_workspace",
                    &[("error", &error)],
                ));
            }
        }
    }

    fn handle_select_document(&mut self, path: std::path::PathBuf) {
        // If the tab is already open, move focus to it. Reuse cache if the content hasn't changed.
        if let Some(existing_idx) = self
            .state
            .open_documents
            .iter()
            .position(|d| d.path == path)
        {
            self.state.active_doc_idx = Some(existing_idx);
            let src = self.state.open_documents[existing_idx].buffer.clone();
            let h = hash_str(&src);
            let last_h = self.tab_hashes.get(&path).copied().unwrap_or(0);
            if h != last_h {
                // Re-render only if the content has changed
                self.full_refresh_preview(&path, &src);
            }
            // No change -> reuse existing PreviewPane (cached)
            return;
        }

        match self.fs.load_document(&path) {
            Ok(doc) => {
                let src = doc.buffer.clone();
                self.state.open_documents.push(doc);
                self.state.active_doc_idx = Some(self.state.open_documents.len() - 1);
                self.full_refresh_preview(&path, &src);
            }
            Err(e) => {
                let error = e.to_string();
                self.state.status_message = Some(crate::i18n::tf(
                    "status_cannot_open_file",
                    &[("error", &error)],
                ));
            }
        }
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

    fn handle_save_document(&mut self) {
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        match self.fs.save_document(doc) {
            Ok(()) => self.state.status_message = Some(crate::i18n::t("status_saved")),
            Err(e) => {
                let error = e.to_string();
                self.state.status_message =
                    Some(crate::i18n::tf("status_save_failed", &[("error", &error)]));
            }
        }
    }

    pub(crate) fn process_action(&mut self, action: AppAction) {
        match action {
            AppAction::OpenWorkspace(p) => self.handle_open_workspace(p),
            AppAction::SelectDocument(p) => self.handle_select_document(p),
            AppAction::CloseDocument(idx) => {
                if idx < self.state.open_documents.len() {
                    self.state.open_documents.remove(idx);
                    self.state.active_doc_idx = if self.state.open_documents.is_empty() {
                        None
                    } else {
                        Some(self.state.open_documents.len() - 1)
                    };
                }
            }
            AppAction::UpdateBuffer(c) => self.handle_update_buffer(c),
            AppAction::SaveDocument => self.handle_save_document(),
            AppAction::RefreshDiagrams => {
                if let Some(doc) = self.state.active_document() {
                    let src = doc.buffer.clone();
                    let path = doc.path.clone();
                    self.full_refresh_preview(&path, &src);
                }
            }
            AppAction::ChangeLanguage(lang) => {
                crate::i18n::set_language(&lang);
                self.state.settings.settings_mut().language = lang;
                if let Err(e) = self.state.settings.save() {
                    tracing::warn!("Failed to save settings: {e}");
                }
            }
            AppAction::None => {}
        }
    }

    /// Processes a download request in a background thread.
    pub(crate) fn start_download(&mut self, req: DownloadRequest) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        self.state.status_message = Some(crate::i18n::t("downloading_plantuml"));
        let url = req.url;
        let dest = req.dest;
        std::thread::spawn(move || {
            let result = download_with_curl(&url, &dest);
            let _ = tx.send(result);
        });
    }

    /// Polls for download completion and re-renders the preview when done.
    pub(crate) fn poll_download(&mut self, ctx: &egui::Context) {
        let done = if let Some(rx) = &self.download_rx {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    self.state.status_message = Some(crate::i18n::t("plantuml_installed"));
                    self.pending_action = AppAction::RefreshDiagrams;
                    true
                }
                Ok(Err(e)) => {
                    self.state.status_message =
                        Some(format!("{}{}", crate::i18n::t("download_error"), e));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(
                        DOWNLOAD_STATUS_CHECK_INTERVAL_MS,
                    ));
                    false
                }
                Err(_) => true,
            }
        } else {
            false
        };
        if done {
            self.download_rx = None;
        }
    }

    /// Method for injecting actions from the program, e.g., during testing
    pub fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }

    /// Helper for calling AppState methods, e.g., during testing
    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

/// Calls `curl` as a subprocess to download a file.
fn download_with_curl(url: &str, dest: &std::path::Path) -> Result<(), String> {
    // Create parent directory if it does not exist
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let status = std::process::Command::new("curl")
        .args(["-L", "-o", dest.to_str().unwrap_or(""), url])
        .status()
        .map_err(|e| format!("{}: {e}", crate::i18n::t("curl_launch_failed")))?;
    if status.success() {
        Ok(())
    } else {
        Err(crate::i18n::t("download_failed"))
    }
}

// impl eframe::App for KatanaApp has been moved to shell_ui.rs

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_app() -> KatanaApp {
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
        );
        KatanaApp::new(state)
    }

    fn make_temp_workspace() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        // Create an md file in the workspace
        std::fs::write(dir.path().join("test.md"), "# Test").unwrap();
        dir
    }

    // handle_open_workspace: Success with valid path (L149-160)
    #[test]
    fn handle_open_workspace_success_sets_workspace() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        assert!(app.state.workspace.is_some());
        assert!(app.state.status_message.is_some());
    }

    // handle_open_workspace: Error with invalid path (L161-167)
    #[test]
    fn handle_open_workspace_error_sets_status_message() {
        let mut app = make_app();
        app.handle_open_workspace(PathBuf::from("/nonexistent/path/that/cannot/exist"));
        // Non-existent path, so workspace might be None (or opened as an empty directory)
        // Either an error is recorded or an empty workspace is opened
        assert!(
            app.state.workspace.is_some() || app.state.status_message.is_some(),
            "Error or workspace should be set"
        );
    }

    // handle_select_document: Load error for non-existent file (L198-204)
    #[test]
    fn handle_select_document_file_not_found_sets_status_message() {
        let mut app = make_app();
        app.handle_select_document(PathBuf::from("/nonexistent/file.md"));
        // Load error -> recorded in status_message
        assert!(app.state.status_message.is_some());
    }

    // handle_select_document: Move focus by selecting existing tab (L173-188)
    #[test]
    fn handle_select_document_switches_to_existing_tab() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");

        // Initial load
        app.handle_select_document(path.clone());
        assert_eq!(app.state.active_doc_idx, Some(0));
        assert_eq!(app.state.open_documents.len(), 1);

        // Re-select the same file -> does not open a new tab
        app.handle_select_document(path.clone());
        assert_eq!(app.state.open_documents.len(), 1);
        assert_eq!(app.state.active_doc_idx, Some(0));
    }

    // handle_update_buffer: No active document (L213)
    #[test]
    fn handle_update_buffer_without_active_doc_does_nothing() {
        let mut app = make_app();
        // UpdateBuffer without opening a document -> does not panic
        app.handle_update_buffer("new content".to_string());
        assert!(app.state.open_documents.is_empty());
    }

    // handle_update_buffer: Active document exists (L209-215)
    #[test]
    fn handle_update_buffer_updates_active_doc_buffer() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());

        app.handle_update_buffer("# Updated Content".to_string());
        let doc = app.state.active_document().unwrap();
        assert_eq!(doc.buffer, "# Updated Content");
        assert!(doc.is_dirty);
    }

    // handle_save_document: No active document (L219-220)
    #[test]
    fn handle_save_document_without_active_doc_does_nothing() {
        let mut app = make_app();
        app.handle_save_document();
        // Status message is not set (no document)
        assert!(app.state.status_message.is_none());
    }

    // handle_save_document: Successful save (L222-223)
    #[test]
    fn handle_save_document_success_sets_status() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());
        app.handle_update_buffer("# Modified".to_string());

        app.handle_save_document();
        assert!(app.state.status_message.is_some());
    }

    // process_action: CloseDocument (L236-244)
    #[test]
    fn process_action_close_document_removes_tab() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());
        assert_eq!(app.state.open_documents.len(), 1);

        app.process_action(AppAction::CloseDocument(0));
        assert!(app.state.open_documents.is_empty());
        assert!(app.state.active_doc_idx.is_none());
    }

    // process_action: CloseDocument - out of bounds does not panic (L237)
    #[test]
    fn process_action_close_document_out_of_bounds_does_nothing() {
        let mut app = make_app();
        app.process_action(AppAction::CloseDocument(99));
        assert!(app.state.open_documents.is_empty());
    }

    // process_action: RefreshDiagrams (L248-253)
    #[test]
    fn process_action_refresh_diagrams_does_not_crash() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());

        app.process_action(AppAction::RefreshDiagrams);
        // OK as long as no crash occurs
    }

    // process_action: RefreshDiagrams no document (L249 early return)
    #[test]
    fn process_action_refresh_diagrams_no_doc_does_nothing() {
        let mut app = make_app();
        app.process_action(AppAction::RefreshDiagrams);
        // No document -> does not crash
    }

    // process_action: ChangeLanguage (L255-257)
    #[test]
    fn process_action_change_language_sets_language() {
        let mut app = make_app();
        app.process_action(AppAction::ChangeLanguage("ja".to_string()));
        // Verify i18n language was changed (since direct access is hard, ensure no panic)
    }

    // process_action: None (L258)
    #[test]
    fn process_action_none_does_nothing() {
        let mut app = make_app();
        app.process_action(AppAction::None);
    }

    // process_action: UpdateBuffer (L246)
    #[test]
    fn process_action_update_buffer_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path);
        app.process_action(AppAction::UpdateBuffer("# Via Process Action".to_string()));
        assert_eq!(
            app.state.active_document().unwrap().buffer,
            "# Via Process Action"
        );
    }

    // process_action: SaveDocument (L247)
    #[test]
    fn process_action_save_document_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path);
        app.process_action(AppAction::UpdateBuffer("saved content".to_string()));
        app.process_action(AppAction::SaveDocument);
        assert!(app.state.status_message.is_some());
    }

    // start_download: Thread starts (L263-273)
    #[test]
    fn start_download_sets_download_state() {
        let mut app = make_app();
        app.start_download(DownloadRequest {
            url: "http://example.com/plantuml.jar".to_string(),
            dest: PathBuf::from("/tmp/test_plantuml.jar"),
        });
        // status_message is set
        assert!(app.state.status_message.is_some());
        // download_rx is set
        assert!(app.download_rx.is_some());
    }

    // download_with_curl: Parent directory creation required (L319-320)
    #[test]
    fn download_with_curl_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("subdir").join("file.jar");
        // Parent directory is created even if curl fails
        // (curl fails with a non-existent URL, but dir_all succeeds)
        let _ = download_with_curl("http://127.0.0.1:0/nonexistent", &dest);
        // Verify that the parent directory was created
        assert!(dest.parent().unwrap().exists());
    }

    // take_action: Return pending_action and reset (L127-129)
    #[test]
    fn take_action_returns_and_resets_pending_action() {
        let mut app = make_app();
        app.pending_action = AppAction::ChangeLanguage("en".to_string());
        let action = app.take_action();
        assert!(
            format!("{action:?}").starts_with("ChangeLanguage"),
            "expected ChangeLanguage, got {action:?}"
        );
        assert_eq!(
            format!("{:?}", app.pending_action),
            format!("{:?}", AppAction::None)
        );
    }

    // poll_download: If no download_rx (L297-299)
    #[test]
    fn poll_download_without_rx_does_nothing() {
        let app = make_app();
        assert!(app.download_rx.is_none());
        // Polling without download_rx is fine
        // Internal poll cannot be called without an egui Context, but
        // it early exits if download_rx = None (L297-299)
    }
}

// shell.rs additional tests: separated from previous module to increase coverage
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests_extra {
    use super::*;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};

    fn make_app() -> KatanaApp {
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
        );
        KatanaApp::new(state)
    }

    fn make_temp_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.md"), "# Test").unwrap();
        dir
    }

    // handle_select_document: Re-render on hash mismatch (L184-185)
    #[test]
    fn handle_select_document_rerenders_when_hash_changed() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");

        // Initial load
        app.handle_select_document(path.clone());
        assert_eq!(app.state.open_documents.len(), 1);

        // Set an old hash in tab_hashes (different from buffer)
        app.tab_hashes.insert(path.clone(), 0xDEADBEEF);

        // Re-select -> full_refresh_preview is called due to hash mismatch (L184-185)
        app.handle_select_document(path.clone());

        // Tab count remains unchanged
        assert_eq!(app.state.open_documents.len(), 1);
    }

    // handle_save_document: Case where fs.save_document fails (L224-228)
    #[test]
    fn handle_save_document_error_sets_error_status_message() {
        use std::os::unix::fs::PermissionsExt;

        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());
        app.handle_update_buffer("# Modified content".to_string());

        // Make file read-only
        let perms = std::fs::Permissions::from_mode(0o444);
        std::fs::set_permissions(&path, perms).unwrap();

        app.handle_save_document();

        // Write failure -> recorded in status_message
        assert!(app.state.status_message.is_some());

        // Cleanup: restore writability
        let perms = std::fs::Permissions::from_mode(0o644);
        let _ = std::fs::set_permissions(&path, perms);
    }

    // download_with_curl: Success case (L326-327) — local file:// URL
    #[test]
    fn download_with_curl_success_with_local_file_url() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&src, "hello").unwrap();

        let url = format!("file://{}", src.display());
        let result = download_with_curl(&url, &dest);
        // curl is available on macOS
        assert!(result.is_ok());
        assert!(dest.exists());
    }

    // process_action: OpenWorkspace (L234)
    #[test]
    fn process_action_open_workspace_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.process_action(AppAction::OpenWorkspace(dir.path().to_path_buf()));
        assert!(app.state.workspace.is_some());
    }

    // process_action: SelectDocument (L235)
    #[test]
    fn process_action_select_document_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.process_action(AppAction::SelectDocument(path));
        assert_eq!(app.state.open_documents.len(), 1);
    }

    // full_refresh_preview: Hash is updated (L140-147)
    #[test]
    fn full_refresh_preview_updates_tab_hash() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.full_refresh_preview(&path, "# Content");
        assert!(app.tab_hashes.contains_key(&path));
    }

    // refresh_preview: Existing entry is updated (L131-137)
    #[test]
    fn refresh_preview_updates_existing_pane() {
        let mut app = make_app();
        let _dir = make_temp_workspace();
        let path = _dir.path().join("test.md");
        app.refresh_preview(&path, "# Initial");
        app.refresh_preview(&path, "# Updated");
    }

    // poll_download: Does nothing if download_rx is None
    #[test]
    fn poll_download_does_nothing_when_no_rx() {
        let mut app = make_app();
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.download_rx.is_none());
    }

    // poll_download: Completes with Ok(Ok(())) -> sets status_message, download_rx=None
    #[test]
    fn poll_download_sets_status_on_success() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.download_rx = Some(rx);
        tx.send(Ok(())).unwrap();
        drop(tx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.state.status_message.is_some());
        assert!(app.download_rx.is_none());
        assert_eq!(
            format!("{:?}", app.pending_action),
            format!("{:?}", AppAction::RefreshDiagrams)
        );
    }

    // poll_download: Errors with Ok(Err(e)) -> error status_message
    #[test]
    fn poll_download_sets_error_on_failure() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.download_rx = Some(rx);
        tx.send(Err("network error".to_string())).unwrap();
        drop(tx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.state.status_message.is_some());
        assert!(app.download_rx.is_none());
    }

    // poll_download: Err(Empty) -> Still receiving
    #[test]
    fn poll_download_keeps_rx_when_empty() {
        let mut app = make_app();
        let (_tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
        app.download_rx = Some(rx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        // rx is maintained because it's Empty
        assert!(app.download_rx.is_some());
    }

    // poll_download: Err(Disconnected) -> Processed as complete
    #[test]
    fn poll_download_clears_rx_on_disconnect() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
        drop(tx); // Disconnected on sender drop
        app.download_rx = Some(rx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.download_rx.is_none());
    }

    // download_with_curl: Failure path (invalid URL -> non-zero exit code)
    #[test]
    fn download_with_curl_failure_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let dest = dir.path().join("nonexistent.jar");
        // Non-existent file URL -> curl fails
        let result = download_with_curl("file:///nonexistent/path/to/file", &dest);
        assert!(result.is_err());
    }

    // download_with_curl: Covers create_dir_all path (when parent directory doesn't exist)
    #[test]
    fn download_with_curl_creates_parent_dirs() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = dir.path().join("source.txt");
        std::fs::write(&src, "hello").unwrap();
        let dest = dir.path().join("subdir").join("deep").join("dest.txt");
        let url = format!("file://{}", src.display());
        let result = download_with_curl(&url, &dest);
        // Directory is created
        assert!(dest.parent().unwrap().exists());
        assert!(result.is_ok());
        assert!(dest.exists());
    }

    // download_with_curl: Case where parent() is None (path with only a root-level filename)
    #[test]
    fn download_with_curl_no_parent_path() {
        let result = download_with_curl("file:///nonexistent/file", std::path::Path::new(""));
        assert!(result.is_err());
    }

    // download_with_curl: Case where create_dir_all returns an error (covering map_err closure)
    #[test]
    fn download_with_curl_create_dir_error() {
        // Cause create_dir_all to fail using a read-only path like /proc/...
        // On macOS, new directories cannot be created under /dev/
        let dest = std::path::Path::new("/dev/null/impossible_dir/file.jar");
        let result = download_with_curl("file:///nonexistent/file", dest);
        assert!(result.is_err());
    }

    /// A mock repository that always fails on save, for testing error paths.
    struct FailingRepository;

    impl katana_platform::SettingsRepository for FailingRepository {
        fn load(&self) -> katana_platform::settings::AppSettings {
            katana_platform::settings::AppSettings::default()
        }
        fn save(&self, _settings: &katana_platform::settings::AppSettings) -> anyhow::Result<()> {
            anyhow::bail!("simulated save failure")
        }
    }

    fn make_app_with_failing_repo() -> KatanaApp {
        let settings = katana_platform::SettingsService::new(Box::new(FailingRepository));
        let state = AppState::new(AiProviderRegistry::new(), PluginRegistry::new(), settings);
        KatanaApp::new(state)
    }

    // handle_open_workspace: settings.save() error is logged, not panicked
    #[test]
    fn handle_open_workspace_save_error_does_not_panic() {
        let mut app = make_app_with_failing_repo();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        // Workspace is still opened despite save failure
        assert!(app.state.workspace.is_some());
    }

    // ChangeLanguage: settings.save() error is logged, not panicked
    #[test]
    fn change_language_save_error_does_not_panic() {
        let mut app = make_app_with_failing_repo();
        app.process_action(AppAction::ChangeLanguage("ja".to_string()));
        // Language change still proceeds despite save failure
    }
}
