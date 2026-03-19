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

/// Ephemeral split-layout state cached per tab for the lifetime of the app.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SplitViewState {
    pub direction: katana_platform::SplitDirection,
    pub order: katana_platform::PaneOrder,
}

/// Tab within the settings window.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum SettingsTab {
    /// Theme preset and custom colour editing.
    #[default]
    Theme,
    /// Font size and family.
    Font,
    /// Editor / preview layout options.
    Layout,
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
    /// Toggle the settings window.
    ToggleSettings,
    /// Change the split view direction (Horizontal / Vertical).
    SetSplitDirection(katana_platform::SplitDirection),
    /// Change the pane order within the split view (EditorFirst / PreviewFirst).
    SetPaneOrder(katana_platform::PaneOrder),
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
    /// Split-layout cache per tab. Initialized from persisted defaults when a tab is first opened.
    pub tab_split_states: HashMap<std::path::PathBuf, SplitViewState>,

    /// Plugin registry (will be referenced during plugin widget integration in future Task 5.x).
    pub _plugin_registry: PluginRegistry,
    /// Non-fatal status message for the status bar.
    pub status_message: Option<String>,
    /// Show/hide the workspace panel.
    pub show_workspace: bool,
    /// Show/hide the settings window.
    pub show_settings: bool,
    /// Currently active tab in the settings window.
    pub active_settings_tab: SettingsTab,
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
            tab_split_states: HashMap::new(),
            _plugin_registry: plugin_registry,
            status_message: None,
            show_workspace: true,
            show_settings: false,
            active_settings_tab: SettingsTab::default(),
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

    fn split_defaults(&self) -> SplitViewState {
        SplitViewState {
            direction: self.settings.settings().split_direction,
            order: self.settings.settings().pane_order,
        }
    }

    pub fn initialize_tab_split_state(&mut self, path: impl Into<std::path::PathBuf>) {
        let defaults = self.split_defaults();
        self.tab_split_states.entry(path.into()).or_insert(defaults);
    }

    pub fn ensure_active_split_state(&mut self) {
        let Some(path) = self.active_path().map(std::path::Path::to_path_buf) else {
            return;
        };
        self.initialize_tab_split_state(path);
    }

    /// Returns the split direction for the active tab.
    pub fn active_split_direction(&self) -> katana_platform::SplitDirection {
        self.active_document()
            .and_then(|doc| self.tab_split_states.get(&doc.path))
            .map(|state| state.direction)
            .unwrap_or_else(|| self.split_defaults().direction)
    }

    /// Returns the pane order for the active tab.
    pub fn active_pane_order(&self) -> katana_platform::PaneOrder {
        self.active_document()
            .and_then(|doc| self.tab_split_states.get(&doc.path))
            .map(|state| state.order)
            .unwrap_or_else(|| self.split_defaults().order)
    }

    /// Sets the split direction for the active tab (temporary — not persisted to disk).
    pub fn set_active_split_direction(&mut self, dir: katana_platform::SplitDirection) {
        let Some(path) = self.active_path().map(std::path::Path::to_path_buf) else {
            return;
        };
        let defaults = self.split_defaults();
        let state = self.tab_split_states.entry(path).or_insert(defaults);
        state.direction = dir;
    }

    /// Sets the pane order for the active tab (temporary — not persisted to disk).
    pub fn set_active_pane_order(&mut self, order: katana_platform::PaneOrder) {
        let Some(path) = self.active_path().map(std::path::Path::to_path_buf) else {
            return;
        };
        let defaults = self.split_defaults();
        let state = self.tab_split_states.entry(path).or_insert(defaults);
        state.order = order;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_platform::{PaneOrder, SplitDirection};
    use std::path::PathBuf;

    fn make_state_with_doc(path: &str) -> AppState {
        let mut state = AppState::new(Default::default(), Default::default(), Default::default());
        let doc = Document {
            path: PathBuf::from(path),
            buffer: String::new(),
            is_dirty: false,
        };
        state.open_documents.push(doc);
        state.active_doc_idx = Some(0);
        state.initialize_tab_split_state(PathBuf::from(path));
        state.set_active_view_mode(ViewMode::Split);
        state
    }

    #[test]
    fn test_split_state_is_cached_per_tab_after_settings_change() {
        let mut state = make_state_with_doc("/tmp/a.md");
        state.settings.settings_mut().split_direction = SplitDirection::Horizontal;
        state.initialize_tab_split_state("/tmp/a.md");
        state.settings.settings_mut().split_direction = SplitDirection::Vertical;

        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);
    }

    #[test]
    fn test_pane_order_is_cached_per_tab_after_settings_change() {
        let mut state = make_state_with_doc("/tmp/b.md");
        state.settings.settings_mut().pane_order = PaneOrder::EditorFirst;
        state.initialize_tab_split_state("/tmp/b.md");
        state.settings.settings_mut().pane_order = PaneOrder::PreviewFirst;

        assert_eq!(state.active_pane_order(), PaneOrder::EditorFirst);
    }

    #[test]
    fn test_new_tab_uses_latest_persisted_split_defaults() {
        let mut state = make_state_with_doc("/tmp/a.md");
        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);

        state.settings.settings_mut().split_direction = SplitDirection::Vertical;
        state.settings.settings_mut().pane_order = PaneOrder::PreviewFirst;

        let doc = Document {
            path: PathBuf::from("/tmp/b.md"),
            buffer: String::new(),
            is_dirty: false,
        };
        state.open_documents.push(doc);
        state.active_doc_idx = Some(1);
        state.initialize_tab_split_state("/tmp/b.md");

        assert_eq!(state.active_split_direction(), SplitDirection::Vertical);
        assert_eq!(state.active_pane_order(), PaneOrder::PreviewFirst);

        state.active_doc_idx = Some(0);
        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);
        assert_eq!(state.active_pane_order(), PaneOrder::EditorFirst);
    }

    #[test]
    fn test_split_state_is_initialized_once_then_prefers_tab_local_state() {
        let mut state = make_state_with_doc("/tmp/a.md");
        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);
        assert_eq!(state.active_pane_order(), PaneOrder::EditorFirst);

        state.set_active_split_direction(SplitDirection::Vertical);
        state.set_active_pane_order(PaneOrder::EditorFirst);

        state.settings.settings_mut().split_direction = SplitDirection::Horizontal;
        state.settings.settings_mut().pane_order = PaneOrder::PreviewFirst;

        state.open_documents.push(Document {
            path: PathBuf::from("/tmp/b.md"),
            buffer: String::new(),
            is_dirty: false,
        });
        state.active_doc_idx = Some(1);
        state.initialize_tab_split_state("/tmp/b.md");

        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);
        assert_eq!(state.active_pane_order(), PaneOrder::PreviewFirst);

        state.active_doc_idx = Some(0);
        assert_eq!(state.active_split_direction(), SplitDirection::Vertical);
        assert_eq!(state.active_pane_order(), PaneOrder::EditorFirst);
    }

    #[test]
    fn test_ensure_active_split_state_no_active_doc() {
        let mut state = AppState::new(Default::default(), Default::default(), Default::default());
        // No documents open, active_doc_idx is None — should return early without panic
        state.ensure_active_split_state();
        assert!(state.tab_split_states.is_empty());
    }

    #[test]
    fn test_set_active_split_direction_no_active_doc() {
        let mut state = AppState::new(Default::default(), Default::default(), Default::default());
        // No documents open — should return early without panic
        state.set_active_split_direction(SplitDirection::Vertical);
        assert!(state.tab_split_states.is_empty());
    }

    #[test]
    fn test_set_active_pane_order_no_active_doc() {
        let mut state = AppState::new(Default::default(), Default::default(), Default::default());
        // No documents open — should return early without panic
        state.set_active_pane_order(PaneOrder::PreviewFirst);
        assert!(state.tab_split_states.is_empty());
    }
}
