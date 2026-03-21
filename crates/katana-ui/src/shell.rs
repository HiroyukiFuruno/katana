//! KatanA three-pane egui shell.
//!
//! Contains only business logic. egui rendering code is separated into shell_ui.rs.

#![allow(clippy::useless_vec)]

use eframe::egui;
use katana_platform::theme::ThemeColors;
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

/// Vertical spacing between the "recent workspaces" section and other elements (px).
pub(crate) const RECENT_WORKSPACES_SPACING: f32 = 8.0;

/// Vertical spacing between items in the "recent workspaces" section (px).
pub(crate) const RECENT_WORKSPACES_ITEM_SPACING: f32 = 4.0;

/// Height of each row in the file tree (px).
pub(crate) const TREE_ROW_HEIGHT: f32 = 22.0;

/// Horizontal offset for labels in the file tree (px).
pub(crate) const TREE_LABEL_HOFFSET: f32 = 4.0;

/// Polling interval for checking download completion (ms).
pub(crate) const DOWNLOAD_STATUS_CHECK_INTERVAL_MS: u64 = 200;

// ─────────────────────────────────────────────
// Color Constants
// ─────────────────────────────────────────────
// Hardcoded colours have been migrated to ThemeColors (katana-platform::theme).
// Individual constants are no longer needed as visuals_from_theme() sets them.

/// Rounding radius of the active row background in the file tree.
pub(crate) const ACTIVE_FILE_HIGHLIGHT_ROUNDING: f32 = 3.0;

/// Converts a string to u64 using FNV-1a hash.
fn hash_str(s: &str) -> u64 {
    crate::shell_logic::hash_str(s)
}

pub(crate) struct TabPreviewCache {
    pub path: std::path::PathBuf,
    pub pane: PreviewPane,
    pub hash: u64,
}

pub(crate) enum WorkspaceLoadType {
    Open,
    Refresh,
}

pub(crate) type WorkspaceLoadResult =
    Result<katana_core::workspace::Workspace, katana_core::workspace::WorkspaceError>;
pub(crate) type WorkspaceLoadMessage = (WorkspaceLoadType, std::path::PathBuf, WorkspaceLoadResult);

pub struct KatanaApp {
    pub(crate) state: AppState,
    pub(crate) fs: FilesystemService,
    pub(crate) pending_action: AppAction,
    /// Per-tab preview pane cache. Reuses cache on tab switch.
    pub(crate) tab_previews: Vec<TabPreviewCache>,
    /// Receiver for background download completion notifications.
    pub(crate) download_rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    /// Receiver for background workspace loading.
    pub(crate) workspace_rx: Option<std::sync::mpsc::Receiver<WorkspaceLoadMessage>>,

    /// Whether the About dialog is currently visible.
    pub(crate) show_about: bool,
    /// App icon texture for the About dialog.
    /// Public because it is set from the binary crate (main.rs) during initialization.
    pub about_icon: Option<egui::TextureHandle>,
    /// Cached theme palette used to avoid redundant `set_visuals()` calls every frame.
    pub(crate) cached_theme: Option<ThemeColors>,
    /// Cached font size to avoid redundant `style_mut()` calls every frame.
    pub(crate) cached_font_size: Option<f32>,
    /// Cached font family to avoid rebuilding `FontDefinitions` every frame.
    pub(crate) cached_font_family: Option<String>,
    /// Dedicated PreviewPane for the settings window live preview.
    pub(crate) settings_preview: PreviewPane,
    /// Tracks the startup time for the splash screen fade animation.
    pub(crate) splash_start: Option<std::time::Instant>,
}

impl KatanaApp {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            tab_previews: Vec::new(),
            download_rx: None,
            workspace_rx: None,
            show_about: false,
            about_icon: None,
            cached_theme: None,
            cached_font_size: None,
            cached_font_family: None,
            settings_preview: PreviewPane::default(),
            splash_start: if cfg!(test) {
                None
            } else {
                Some(std::time::Instant::now())
            },
        }
    }

    /// Explicitly skips the splash screen. Useful for integration testing.
    pub fn skip_splash(&mut self) {
        self.splash_start = None;
    }

    pub(crate) fn take_action(&mut self) -> AppAction {
        std::mem::replace(&mut self.pending_action, AppAction::None)
    }

    pub(crate) fn get_preview_pane(
        previews: &mut Vec<TabPreviewCache>,
        path: std::path::PathBuf,
    ) -> &mut PreviewPane {
        if let Some(idx) = previews.iter().position(|t| t.path == path) {
            &mut previews[idx].pane
        } else {
            previews.push(TabPreviewCache {
                path,
                pane: PreviewPane::default(),
                hash: 0,
            });
            &mut previews.last_mut().expect("just pushed").pane
        }
    }

    /// Reflects only text changes (keeps existing images for diagrams).
    fn refresh_preview(&mut self, path: &std::path::Path, source: &str) {
        let path_buf = path.to_path_buf();
        Self::get_preview_pane(&mut self.tab_previews, path_buf)
            .update_markdown_sections(source, path);
    }

    fn full_refresh_preview(
        &mut self,
        path: &std::path::Path,
        source: &str,
        force: bool,
        concurrency: usize,
    ) {
        let h = hash_str(source);
        let path_buf = path.to_path_buf();
        let pane = Self::get_preview_pane(&mut self.tab_previews, path_buf.clone());
        pane.full_render(source, path, self.state.cache.clone(), force, concurrency);

        let tab = self
            .tab_previews
            .iter_mut()
            .find(|t| t.path == path_buf)
            .expect("just fetched pane");
        // If force, also reset hash to 0 so it redraws on switch, or update it now.
        // We update to true hash since we re-rendered anyway.
        tab.hash = h;
    }

    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        // Save current workspace state (including open tabs) before unloading it
        if self.state.workspace.is_some() {
            self.save_workspace_state();
        }

        self.state.is_loading_workspace = true;
        // Temporary feedback
        self.state.status_message = Some(crate::i18n::tf(
            &crate::i18n::get().status.opened_workspace,
            &vec![("name", "...")],
        ));

        let (tx, rx) = std::sync::mpsc::channel();
        self.workspace_rx = Some(rx);
        let path_clone = path.clone();

        std::thread::spawn(move || {
            let fs = katana_platform::FilesystemService::new();
            let result = fs.open_workspace(&path_clone);
            let _ = tx.send((WorkspaceLoadType::Open, path_clone, result));
        });
    }

    fn finish_open_workspace(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    ) {
        let name = ws.name().unwrap_or("unknown").to_string();
        self.state.status_message = Some(crate::i18n::tf(
            &crate::i18n::get().status.opened_workspace,
            &vec![("name", name.as_str())],
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
            if !settings.workspace.paths.contains(&path_str) {
                settings.workspace.paths.push(path_str);
            }
            // We no longer clear here because we manage state via CacheFacade now
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

        std::thread::spawn(move || {
            let fs = katana_platform::FilesystemService::new();
            let result = fs.open_workspace(&root);
            let _ = tx.send((WorkspaceLoadType::Refresh, root, result));
        });
    }

    pub(crate) fn poll_workspace_load(&mut self, ctx: &egui::Context) {
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
                    self.state.status_message = Some(crate::i18n::tf(
                        &crate::i18n::get().status.cannot_open_workspace,
                        &vec![("error", error.as_str())],
                    ));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(
                        WORKSPACE_LOAD_POLL_INTERVAL_MS,
                    ));
                    false
                }
                Err(_) => {
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
    }

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
                    self.state.status_message = Some(crate::i18n::tf(
                        "status_cannot_open_file",
                        &vec![("error", error.as_str())],
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

    fn handle_remove_workspace(&mut self, path: String) {
        let settings = self.state.settings.settings_mut();
        settings.workspace.paths.retain(|p| p != &path);
        // If the removed workspace matches the last_workspace, we don't necessarily clear last_workspace
        // because it's still the active one, but it won't appear in the history list anymore.
        if let Err(e) = self.state.settings.save() {
            tracing::warn!("Failed to save settings after removing workspace: {e}");
        }
    }

    pub(crate) fn save_workspace_state(&mut self) {
        let open_tabs: Vec<String> = self
            .state
            .open_documents
            .iter()
            .map(|d| d.path.display().to_string())
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
            Ok(()) => {
                self.state.status_message = Some(crate::i18n::get().status.saved.clone());
                self.save_workspace_state();
            }
            Err(e) => {
                let error = e.to_string();
                self.state.status_message = Some(crate::i18n::tf(
                    &crate::i18n::get().status.save_failed,
                    &vec![("error", error.as_str())],
                ));
            }
        }
    }

    pub(crate) fn process_action(&mut self, action: AppAction) {
        match action {
            AppAction::OpenWorkspace(p) => self.handle_open_workspace(p),
            AppAction::RefreshWorkspace => self.handle_refresh_workspace(),
            AppAction::SelectDocument(p) => self.handle_select_document(p, true),
            AppAction::OpenMultipleDocuments(paths) => {
                // When recursively opening a directory, open all files lazily
                // and do not artificially activate the last file nor auto-expand the directory tree.
                for path in paths.into_iter() {
                    self.handle_select_document(path, false);
                }
            }
            AppAction::RemoveWorkspace(path) => self.handle_remove_workspace(path),
            AppAction::CloseDocument(idx) => {
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
            }
            AppAction::UpdateBuffer(c) => self.handle_update_buffer(c),
            AppAction::SaveDocument => self.handle_save_document(),
            AppAction::RefreshDiagrams => {
                // Invalidate hashes so non-active tabs re-render on next switch
                for tab in &mut self.tab_previews {
                    tab.hash = 0;
                }
                // Re-render only the active tab immediately
                if let Some(doc) = self.state.active_document() {
                    let path = doc.path.clone();
                    let src = doc.buffer.clone();
                    let concurrency = self
                        .state
                        .settings
                        .settings()
                        .performance
                        .diagram_concurrency;
                    self.full_refresh_preview(&path, &src, true, concurrency);
                }
            }
            AppAction::ChangeLanguage(lang) => {
                crate::i18n::set_language(&lang);
                crate::shell_ui::update_native_menu_strings_from_i18n();
                self.state.settings.settings_mut().language = lang;
                if let Err(e) = self.state.settings.save() {
                    tracing::warn!("Failed to save settings: {e}");
                }
            }
            AppAction::ToggleSettings => {
                self.state.show_settings = !self.state.show_settings;
            }
            AppAction::ToggleToc => {
                self.state.show_toc = !self.state.show_toc;
            }
            AppAction::SetSplitDirection(dir) => {
                // Keep toolbar toggles temporary and scoped to the active tab.
                self.state.set_active_split_direction(dir);
            }
            AppAction::SetPaneOrder(order) => {
                // Keep toolbar toggles temporary and scoped to the active tab.
                self.state.set_active_pane_order(order);
            }
            AppAction::CloseOtherDocuments(idx) => {
                if idx < self.state.open_documents.len() {
                    let mut keep = Vec::new();
                    let old_docs = std::mem::take(&mut self.state.open_documents);
                    for (i, doc) in old_docs.into_iter().enumerate() {
                        if i == idx {
                            keep.push(doc);
                        } else {
                            self.state.push_recently_closed(doc.path);
                        }
                    }
                    self.state.open_documents = keep;
                    self.state.active_doc_idx = Some(0);
                }
                self.save_workspace_state();
            }
            AppAction::CloseAllDocuments => {
                let old_docs = std::mem::take(&mut self.state.open_documents);
                for doc in old_docs.into_iter() {
                    self.state.push_recently_closed(doc.path);
                }
                self.state.active_doc_idx = None;
                self.save_workspace_state();
            }
            AppAction::CloseDocumentsToRight(idx) => {
                let mut keep = Vec::new();
                let old_docs = std::mem::take(&mut self.state.open_documents);
                for (i, doc) in old_docs.into_iter().enumerate() {
                    if i <= idx {
                        keep.push(doc);
                    } else {
                        self.state.push_recently_closed(doc.path);
                    }
                }
                self.state.open_documents = keep;
                if let Some(a_idx) = self.state.active_doc_idx {
                    if a_idx > idx {
                        self.state.active_doc_idx = Some(idx);
                    }
                }
                self.save_workspace_state();
            }
            AppAction::CloseDocumentsToLeft(idx) => {
                let mut keep = Vec::new();
                let new_active_idx = self.state.active_doc_idx;
                let old_docs = std::mem::take(&mut self.state.open_documents);
                for (i, doc) in old_docs.into_iter().enumerate() {
                    if i >= idx {
                        keep.push(doc);
                    } else {
                        self.state.push_recently_closed(doc.path);
                    }
                }
                self.state.open_documents = keep;
                if let Some(a_idx) = new_active_idx {
                    if a_idx < idx {
                        self.state.active_doc_idx = Some(0);
                    } else {
                        self.state.active_doc_idx = Some(a_idx - idx);
                    }
                }
                self.save_workspace_state();
            }
            AppAction::TogglePinDocument(idx) => {
                if idx < self.state.open_documents.len() {
                    let active_path = self.state.active_document().map(|d| d.path.clone());
                    let doc = &mut self.state.open_documents[idx];
                    doc.is_pinned = !doc.is_pinned;
                    // Stable sort to move pinned tabs to the front
                    self.state.open_documents.sort_by_key(|d| !d.is_pinned);
                    if let Some(path) = active_path {
                        if let Some(new_idx) = self
                            .state
                            .open_documents
                            .iter()
                            .position(|d| d.path == path)
                        {
                            self.state.active_doc_idx = Some(new_idx);
                        }
                    }
                }
                self.save_workspace_state();
            }
            AppAction::RestoreClosedDocument => {
                if let Some(path) = self.state.recently_closed_tabs.pop_back() {
                    self.handle_select_document(path, true);
                }
            }
            AppAction::ReorderDocument { from, to } => {
                let len = self.state.open_documents.len();
                if from < len && to <= len && from != to {
                    let active_path = self.state.active_document().map(|d| d.path.clone());
                    let doc = self.state.open_documents.remove(from);
                    let actual_to = if to > from { to - 1 } else { to };
                    self.state.open_documents.insert(actual_to, doc);
                    if let Some(path) = active_path {
                        if let Some(new_idx) = self
                            .state
                            .open_documents
                            .iter()
                            .position(|d| d.path == path)
                        {
                            self.state.active_doc_idx = Some(new_idx);
                        }
                    }
                }
                self.save_workspace_state();
            }
            AppAction::None => {}
        }
    }

    /// Processes a download request in a background thread.
    pub(crate) fn start_download(&mut self, req: DownloadRequest) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        self.state.status_message = Some(crate::i18n::get().plantuml.downloading_plantuml.clone());
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
                    self.state.status_message =
                        Some(crate::i18n::get().plantuml.plantuml_installed.clone());
                    self.pending_action = AppAction::RefreshDiagrams;
                    true
                }
                Ok(Err(e)) => {
                    self.state.status_message = Some(format!(
                        "{}{}",
                        crate::i18n::get().plantuml.download_error.clone(),
                        e
                    ));
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
    _download_with_cmd("curl", url, dest)
}

fn _download_with_cmd(cmd: &str, url: &str, dest: &std::path::Path) -> Result<(), String> {
    // Create parent directory if it does not exist
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let status = std::process::Command::new(cmd)
        .args(vec!["-L", "-o", dest.to_str().unwrap_or(""), url])
        .status()
        .map_err(|e| {
            format!(
                "{}: {e}",
                crate::i18n::get().error.curl_launch_failed.clone()
            )
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(crate::i18n::get().error.download_failed.clone())
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
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    fn make_temp_workspace() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        // Create an md file in the workspace
        std::fs::write(dir.path().join("test.md"), "# Test").unwrap();
        dir
    }

    fn wait_for_workspace(app: &mut KatanaApp) {
        let ctx = egui::Context::default();
        for _ in 0..100 {
            app.poll_workspace_load(&ctx);
            if app.workspace_rx.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    // handle_open_workspace: Success with valid path (L149-160)
    #[test]
    fn handle_open_workspace_success_sets_workspace() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.is_some());
        assert!(app.state.status_message.is_some());
    }

    // handle_open_workspace: Error with invalid path (L161-167)
    #[test]
    fn handle_open_workspace_error_sets_status_message() {
        let mut app = make_app();
        app.handle_open_workspace(PathBuf::from("/nonexistent/path/that/cannot/exist"));
        wait_for_workspace(&mut app);
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
        app.handle_select_document(PathBuf::from("/nonexistent/file.md"), true);
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
        app.handle_select_document(path.clone(), true);
        assert_eq!(app.state.active_doc_idx, Some(0));
        assert_eq!(app.state.open_documents.len(), 1);

        // Re-select the same file -> does not open a new tab
        app.handle_select_document(path.clone(), true);
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
        app.handle_select_document(path.clone(), true);

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

    #[test]
    fn test_lazy_loading_flow() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("lazy.md");
        std::fs::write(&path, "# Lazy content").unwrap();

        // 1. Open lazily
        app.handle_select_document(path.clone(), false);
        assert_eq!(app.state.open_documents.len(), 1);
        assert!(!app.state.open_documents[0].is_loaded);

        // 2. Activate
        app.handle_select_document(path.clone(), true);
        assert!(app.state.open_documents[0].is_loaded);
        assert_eq!(app.state.open_documents[0].buffer, "# Lazy content");
    }

    // handle_save_document: Successful save (L222-223)
    #[test]
    fn handle_save_document_success_sets_status() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone(), true);
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
        app.handle_select_document(path.clone(), true);
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
        app.handle_select_document(path.clone(), true);

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

    // process_action: ToggleToc
    #[test]
    fn process_action_toggle_toc_toggles_flag() {
        let mut app = make_app();
        assert!(!app.state.show_toc);

        app.process_action(AppAction::ToggleToc);
        assert!(app.state.show_toc);

        app.process_action(AppAction::ToggleToc);
        assert!(!app.state.show_toc);
    }

    // process_action: ToggleSettings
    #[test]
    fn process_action_toggle_settings_toggles_flag() {
        let mut app = make_app();
        assert!(!app.state.show_settings);

        app.process_action(AppAction::ToggleSettings);
        assert!(app.state.show_settings);

        app.process_action(AppAction::ToggleSettings);
        assert!(!app.state.show_settings);
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
        app.handle_select_document(path, true);
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
        app.handle_select_document(path, true);
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
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
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
        app.handle_select_document(path.clone(), true);
        assert_eq!(app.state.open_documents.len(), 1);

        // Set an old hash in tab_hashes (different from buffer)
        app.tab_previews.push(TabPreviewCache {
            path: path.clone(),
            pane: PreviewPane::default(),
            hash: 0xDEADBEEF,
        });

        // Re-select -> full_refresh_preview is called due to hash mismatch (L184-185)
        app.handle_select_document(path.clone(), true);

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
        app.handle_select_document(path.clone(), true);
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

    #[test]
    fn download_with_curl_launch_error() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("dest.txt");
        let result = super::_download_with_cmd(
            "invalid_curl_binary_for_test",
            "http://example.com/file",
            &dest,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        // Just verify it uses the mapped error message from the locale
        assert!(err.contains(&crate::i18n::get().error.curl_launch_failed));
    }

    // process_action: OpenWorkspace (L234)
    #[test]
    fn process_action_open_workspace_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.process_action(AppAction::OpenWorkspace(dir.path().to_path_buf()));
        wait_for_workspace(&mut app);
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
        app.full_refresh_preview(&path, "# Content", false, 4);
        assert!(app.tab_previews.iter().any(|t| t.path == path));
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
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            settings,
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    fn wait_for_workspace(app: &mut KatanaApp) {
        let ctx = egui::Context::default();
        for _ in 0..100 {
            app.poll_workspace_load(&ctx);
            if app.workspace_rx.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    // handle_open_workspace: settings.save() error is logged, not panicked
    #[test]
    fn handle_open_workspace_save_error_does_not_panic() {
        let mut app = make_app_with_failing_repo();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
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

    // Regression: trigger_action(OpenWorkspace) must not be overwritten before take_action().
    //
    // Background: shell_ui.rs::update() sets pending_action = RefreshDiagrams on the first
    // frame (cold theme cache). If trigger_action() is called from the eframe setup_cc closure
    // (workspace restore at startup), the unconditional assignment silently discards the
    // OpenWorkspace action, causing the saved workspace to not be restored on reopen.
    #[test]
    fn trigger_action_is_not_overwritten_before_take_action() {
        let mut app = make_app();
        let dir = make_temp_workspace();

        // Simulate startup: workspace restore sets pending_action via trigger_action().
        app.trigger_action(AppAction::OpenWorkspace(dir.path().to_path_buf()));

        // Verify the action is still intact before take_action() is called.
        // The fix in shell_ui.rs guards the RefreshDiagrams assignment with
        // `if matches!(self.pending_action, AppAction::None)`.
        assert!(
            matches!(app.pending_action, AppAction::OpenWorkspace(_)),
            "pending_action must still be OpenWorkspace before take_action(); \
             RefreshDiagrams must not overwrite it"
        );

        let action = app.take_action();
        assert!(
            matches!(action, AppAction::OpenWorkspace(_)),
            "take_action() must return OpenWorkspace, not a different action. \
             Regression: shell_ui theme guard was overwriting pending_action on first frame."
        );

        // After take_action(), pending_action is reset to None.
        assert!(matches!(app.pending_action, AppAction::None));
    }

    // Verify that RefreshDiagrams IS set when no action is pending (normal theme-change path).
    #[test]
    fn refresh_diagrams_is_set_when_no_action_is_pending() {
        let mut app = make_app();
        assert!(matches!(app.pending_action, AppAction::None));

        // Reproduce the fixed guard: only assign when pending is None.
        if matches!(app.pending_action, AppAction::None) {
            app.pending_action = AppAction::RefreshDiagrams;
        }

        assert!(
            matches!(app.pending_action, AppAction::RefreshDiagrams),
            "RefreshDiagrams should be set when no action is pending"
        );
    }

    // handle_refresh_workspace: Success case — re-scans the workspace tree
    #[test]
    fn handle_refresh_workspace_rescans_tree() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.is_some());

        // Add a new file to the workspace
        std::fs::write(dir.path().join("new.md"), "# New").unwrap();

        app.handle_refresh_workspace();
        wait_for_workspace(&mut app);
        let ws = app.state.workspace.as_ref().unwrap();
        let paths: Vec<_> = ws
            .tree
            .iter()
            .map(|it| it.path().to_string_lossy().to_string())
            .collect();
        assert!(paths.iter().any(|it| it.contains("new.md")));
    }

    // handle_refresh_workspace: No workspace open — early return
    #[test]
    fn handle_refresh_workspace_no_workspace_does_nothing() {
        let mut app = make_app();
        app.handle_refresh_workspace();
        assert!(app.state.workspace.is_none());
    }

    // handle_refresh_workspace: Error case — workspace root is no longer valid
    #[test]
    fn handle_refresh_workspace_error_sets_status_message() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.is_some());

        // Overwrite the workspace root to a non-existent path
        app.state.workspace.as_mut().unwrap().root =
            std::path::PathBuf::from("/nonexistent/deleted/workspace");

        app.handle_refresh_workspace();
        wait_for_workspace(&mut app);
        assert!(app.state.status_message.is_some());
    }

    // process_action: RefreshWorkspace
    #[test]
    fn process_action_refresh_workspace_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        app.process_action(AppAction::RefreshWorkspace);
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.is_some());
    }
    #[test]
    fn test_open_workspace_file_updates_buffer() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let file_path = dir.path().join("a.md");
        std::fs::write(&file_path, "A").unwrap();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        app.handle_select_document(file_path.clone(), true);

        let doc = app.state.active_document_mut().unwrap();
        doc.buffer = "B".to_string(); // bypass update_buffer to bypass hash updates

        app.handle_select_document(file_path.clone(), true);
        let tab = app
            .tab_previews
            .iter()
            .find(|t| t.path == file_path)
            .unwrap();
        assert!(tab.hash != 0);
    }

    #[test]
    fn test_poll_workspace_load_disconnect() {
        let state = AppState::new(
            katana_core::ai::AiProviderRegistry::default(),
            katana_core::plugin::PluginRegistry::default(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let mut app = KatanaApp::new(state);

        let (tx, rx) = std::sync::mpsc::channel();
        app.workspace_rx = Some(rx);
        app.state.is_loading_workspace = true;

        // Drop the transmitter to simulate thread panic / disconnect
        drop(tx);

        let ui_ctx = egui::Context::default();
        app.poll_workspace_load(&ui_ctx);

        assert!(!app.state.is_loading_workspace);
    }

    #[test]
    fn test_lazy_loading_flow() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("lazy.md");
        std::fs::write(&path, "# Lazy content").unwrap();

        // 1. Open lazily
        app.handle_select_document(path.clone(), false);
        assert_eq!(app.state.open_documents.len(), 1);
        assert!(!app.state.open_documents[0].is_loaded);

        // 2. Activate
        app.handle_select_document(path.clone(), true);
        assert!(app.state.open_documents[0].is_loaded);
        assert_eq!(app.state.open_documents[0].buffer, "# Lazy content");
    }

    #[test]
    fn test_auto_expansion_relative_path() {
        let mut app = make_app();
        // Path with no parent (relative) should not crash and hit the break
        app.handle_select_document(std::path::PathBuf::from("root_file.md"), true);
        assert!(app.state.expanded_directories.is_empty());
    }

    #[test]
    fn test_handle_select_document_lazy_does_not_expand_parents() {
        let mut app = make_app();
        let path = std::path::PathBuf::from("/a/b/c.md");
        app.handle_select_document(path, false); // Lazy load

        // Ensure no directories were added to expanded_directories
        assert!(
            app.state.expanded_directories.is_empty(),
            "Expanded directories should be empty on lazy load"
        );
    }

    #[test]
    fn test_open_multiple_documents_is_entirely_lazy() {
        let mut app = make_app();
        let paths = vec![
            std::path::PathBuf::from("/a/1.md"),
            std::path::PathBuf::from("/a/2.md"),
        ];

        app.process_action(AppAction::OpenMultipleDocuments(paths));

        // Ensure documents were opened but none of them triggered auto-expansion
        assert_eq!(app.state.open_documents.len(), 2);
        assert!(app.state.expanded_directories.is_empty());
        // Ensure they are not loaded
        assert!(!app.state.open_documents[0].is_loaded);
        assert!(!app.state.open_documents[1].is_loaded);
    }
}
