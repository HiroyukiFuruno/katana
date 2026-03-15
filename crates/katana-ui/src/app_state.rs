//! Shared application state.
//!
//! `scroll_fraction` — State for bidirectionally synchronizing scroll position
//! between editor and preview in Split mode, using a ratio (0.0–1.0).
//!
//! A single `AppState` container is owned by the egui application. UI
//! components render from this state and dispatch `AppAction` values back
//! through the update loop.

use katana_core::{
    ai::AiProviderRegistry, document::Document, plugin::PluginRegistry, workspace::Workspace,
};
use katana_platform::SettingsService;
use std::collections::HashMap;

/// Display mode for the UI layout
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewMode {
    PreviewOnly,
    CodeOnly,
    Split,
}

/// User-visible actions dispatched from UI components to the core update loop.
#[derive(Debug)]
pub enum AppAction {
    /// Open a workspace at the given path.
    OpenWorkspace(std::path::PathBuf),
    /// Select a file in the project tree.
    SelectDocument(std::path::PathBuf),
    /// Close a tab.
    CloseDocument(usize),
    /// Update the buffer of the active document.
    UpdateBuffer(String),
    /// Explicitly save the active document.
    SaveDocument,
    /// Fully re-render the preview, including diagrams.
    RefreshDiagrams,
    /// Change language.
    ChangeLanguage(String),
    /// No-op (used internally).
    None,
}

/// Top-level application state shared across all UI components.
pub struct AppState {
    /// The currently open workspace, if any.
    pub workspace: Option<Workspace>,
    /// Currently open documents (tabs).
    pub open_documents: Vec<Document>,
    /// Index of the currently active document, if any.
    pub active_doc_idx: Option<usize>,
    /// View mode per tab. The key is the file path (to prevent index shifts after closing a tab).
    pub tab_view_modes: HashMap<std::path::PathBuf, ViewMode>,

    /// Plugin registry (will be referenced during plugin widget integration in future Task 5.x).
    pub _plugin_registry: PluginRegistry,
    /// Non-fatal status message for the status bar.
    pub status_message: Option<String>,
    /// Show/hide the workspace panel.
    pub show_workspace: bool,
    /// Trigger to expand/collapse the entire workspace tree. Some(true)=expand all, Some(false)=collapse all.
    pub force_tree_open: Option<bool>,
    /// Split mode scroll sync: Normalized scroll position (0.0–1.0).
    pub scroll_fraction: f32,
    /// Source of the scroll operation. Prevents chain reactions (infinite loops).
    pub scroll_source: ScrollSource,
    /// Previous frame's editor-side max_scroll (content_height - viewport_height).
    pub editor_max_scroll: f32,
    /// Previous frame's preview-side max_scroll (content_height - viewport_height).
    pub preview_max_scroll: f32,
    /// Settings persistence service.
    pub settings: SettingsService,
}

/// Indicates the source of a scroll operation. Used to prevent chain reactions.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ScrollSource {
    /// No change from either (initial state).
    #[default]
    Neither,
    /// Scroll from the editor pane.
    Editor,
    /// Scroll from the preview pane.
    Preview,
}

impl AppState {
    pub fn new(
        ai_registry: AiProviderRegistry,
        plugin_registry: PluginRegistry,
        settings: SettingsService,
    ) -> Self {
        // ai_registry is planned for future AI integration. Currently unused.
        let _ = ai_registry;
        Self {
            workspace: None,
            open_documents: Vec::new(),
            active_doc_idx: None,
            tab_view_modes: HashMap::new(),
            _plugin_registry: plugin_registry,
            status_message: None,
            show_workspace: true,
            force_tree_open: None,
            scroll_fraction: 0.0,
            scroll_source: ScrollSource::Neither,
            editor_max_scroll: 0.0,
            preview_max_scroll: 0.0,
            settings,
        }
    }

    /// Whether the active document has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.active_document().map(|d| d.is_dirty).unwrap_or(false)
    }

    /// Get a reference to the currently active document.
    pub fn active_document(&self) -> Option<&Document> {
        self.active_doc_idx
            .and_then(|idx| self.open_documents.get(idx))
    }

    /// Get a mutable reference to the currently active document.
    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_doc_idx
            .and_then(|idx| self.open_documents.get_mut(idx))
    }

    /// Returns the path of the currently active document (used for highlighting in the workspace tree).
    pub fn active_path(&self) -> Option<&std::path::Path> {
        self.active_document().map(|d| d.path.as_path())
    }

    /// Returns the view mode of the active tab. Default value if no tab is selected.
    pub fn active_view_mode(&self) -> ViewMode {
        self.active_document()
            .and_then(|doc| self.tab_view_modes.get(&doc.path))
            .copied()
            .unwrap_or(ViewMode::PreviewOnly)
    }

    /// Sets the view mode of the active tab.
    pub fn set_active_view_mode(&mut self, mode: ViewMode) {
        if let Some(doc) = self.active_document() {
            let path = doc.path.clone();
            self.tab_view_modes.insert(path, mode);
        }
    }
}
