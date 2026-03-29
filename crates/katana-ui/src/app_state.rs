//! Shared application state.
//!
//! A single `AppState` container is owned by the egui application. UI
//! components render from this state and dispatch `AppAction` values back
//! through the update loop.

pub use crate::state::config::{ConfigState, SettingsSection, SettingsTab};
pub use crate::state::document::{
    DocumentState, SplitViewState, TabSplitState, TabViewMode, ViewMode,
};
pub use crate::state::layout::LayoutState;
pub use crate::state::scroll::{ScrollSource, ScrollState};
pub use crate::state::search::SearchState;
pub use crate::state::update::{UpdatePhase, UpdateState};
pub use crate::state::workspace::WorkspaceState;

pub use katana_platform::CacheFacade;

use katana_core::{ai::AiProviderRegistry, document::Document, plugin::PluginRegistry};
use katana_platform::SettingsService;
use std::path::PathBuf;

/// User-visible actions dispatched from UI components to the core update loop.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExportFormat {
    Html,
    Pdf,
    Png,
    Jpg,
}

#[derive(Debug)]
pub enum AppAction {
    InstallUpdate,
    OpenWorkspace(PathBuf),
    SelectDocument(PathBuf),
    OpenMultipleDocuments(Vec<PathBuf>),
    RemoveWorkspace(String),
    CloseDocument(usize),
    ForceCloseDocument(usize),
    UpdateBuffer(String),
    ReplaceText {
        span: std::ops::Range<usize>,
        replacement: String,
    },
    ToggleTaskList {
        global_index: usize,
        new_state: char,
    },
    SaveDocument,
    RefreshDiagrams,
    ChangeLanguage(String),
    ToggleSettings,
    ToggleAbout,
    ToggleToc,
    CheckForUpdates,
    SetSplitDirection(katana_platform::SplitDirection),
    SetPaneOrder(katana_platform::PaneOrder),
    SetViewMode(ViewMode),
    ToggleScrollSync(bool),
    RefreshWorkspace,
    CloseOtherDocuments(usize),
    CloseAllDocuments,
    CloseDocumentsToRight(usize),
    CloseDocumentsToLeft(usize),
    TogglePinDocument(usize),
    RestoreClosedDocument,
    ReorderDocument {
        from: usize,
        to: usize,
    },
    ExportDocument(ExportFormat),
    AcceptTerms(String),
    DeclineTerms,
    ShowMetaInfo(PathBuf),
    RequestNewFile(PathBuf),
    RequestNewDirectory(PathBuf),
    RequestRename(PathBuf),
    RequestDelete(PathBuf),
    CopyPathToClipboard(PathBuf),
    CopyRelativePathToClipboard(PathBuf),
    RevealInOs(PathBuf),
    SkipVersion(String),
    DismissUpdate,
    ConfirmRelaunch,
    ShowReleaseNotes,
    ClearAllCaches,
    None,
}

#[derive(Debug, PartialEq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

/// Top-level application state shared across all UI components.
pub struct AppState {
    pub document: DocumentState,
    pub workspace: WorkspaceState,
    pub layout: LayoutState,
    pub search: SearchState,
    pub scroll: ScrollState,
    pub update: UpdateState,
    pub config: ConfigState,
}

impl AppState {
    pub fn new(
        ai_registry: AiProviderRegistry,
        plugin_registry: PluginRegistry,
        settings: SettingsService,
        cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    ) -> Self {
        let _ = ai_registry;
        Self {
            document: DocumentState::new(),
            workspace: WorkspaceState::new(),
            layout: LayoutState::new(),
            search: SearchState::new(),
            scroll: ScrollState::new(),
            update: UpdateState::new(),
            config: ConfigState::new(plugin_registry, settings, cache),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.active_document().map(|d| d.is_dirty).unwrap_or(false)
    }

    pub fn active_document(&self) -> Option<&Document> {
        self.document
            .active_doc_idx
            .and_then(|idx| self.document.open_documents.get(idx))
    }

    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.document
            .active_doc_idx
            .and_then(|idx| self.document.open_documents.get_mut(idx))
    }

    pub fn active_path(&self) -> Option<std::path::PathBuf> {
        self.active_document().map(|d| d.path.clone())
    }

    pub fn active_view_mode(&self) -> ViewMode {
        self.active_document()
            .and_then(|doc| {
                self.document
                    .tab_view_modes
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.mode)
            })
            .unwrap_or(ViewMode::PreviewOnly)
    }

    pub fn set_active_view_mode(&mut self, mode: ViewMode) {
        if let Some(doc) = self.active_document() {
            let path = doc.path.clone();
            if let Some(t) = self
                .document
                .tab_view_modes
                .iter_mut()
                .find(|t| t.path == path)
            {
                t.mode = mode;
            } else {
                self.document
                    .tab_view_modes
                    .push(TabViewMode { path, mode });
            }
        }
    }

    fn split_defaults(&self) -> SplitViewState {
        SplitViewState {
            direction: self.config.settings.settings().layout.split_direction,
            order: self.config.settings.settings().layout.pane_order,
        }
    }

    pub fn initialize_tab_split_state(&mut self, path: impl Into<std::path::PathBuf>) {
        let p = path.into();
        if !self.document.tab_split_states.iter().any(|t| t.path == p) {
            let defaults = self.split_defaults();
            self.document.tab_split_states.push(TabSplitState {
                path: p,
                state: defaults,
            });
        }
    }

    pub fn ensure_active_split_state(&mut self) {
        let Some(path) = self.active_path() else {
            return;
        };
        self.initialize_tab_split_state(path);
    }

    pub fn active_split_direction(&self) -> katana_platform::SplitDirection {
        self.active_document()
            .and_then(|doc| {
                self.document
                    .tab_split_states
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.state.direction)
            })
            .unwrap_or_else(|| self.split_defaults().direction)
    }

    pub fn active_pane_order(&self) -> katana_platform::PaneOrder {
        self.active_document()
            .and_then(|doc| {
                self.document
                    .tab_split_states
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.state.order)
            })
            .unwrap_or_else(|| self.split_defaults().order)
    }

    pub fn set_active_split_direction(&mut self, dir: katana_platform::SplitDirection) {
        let Some(path) = self.active_path() else {
            return;
        };
        if let Some(t) = self
            .document
            .tab_split_states
            .iter_mut()
            .find(|t| t.path == path)
        {
            t.state.direction = dir;
        } else {
            let mut defaults = self.split_defaults();
            defaults.direction = dir;
            self.document.tab_split_states.push(TabSplitState {
                path,
                state: defaults,
            });
        }
    }

    pub fn set_active_pane_order(&mut self, order: katana_platform::PaneOrder) {
        let Some(path) = self.active_path() else {
            return;
        };
        if let Some(t) = self
            .document
            .tab_split_states
            .iter_mut()
            .find(|t| t.path == path)
        {
            t.state.order = order;
        } else {
            let mut defaults = self.split_defaults();
            defaults.order = order;
            self.document.tab_split_states.push(TabSplitState {
                path,
                state: defaults,
            });
        }
    }

    pub fn push_recently_closed(&mut self, path: std::path::PathBuf) {
        if self.document.recently_closed_tabs.len() >= DocumentState::MAX_RECENTLY_CLOSED_TABS {
            self.document.recently_closed_tabs.pop_front();
        }
        self.document.recently_closed_tabs.push_back(path);
    }
}
