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

#[derive(Debug, PartialEq, Clone)]
pub struct TabViewMode {
    pub path: std::path::PathBuf,
    pub mode: ViewMode,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TabSplitState {
    pub path: std::path::PathBuf,
    pub state: SplitViewState,
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
    /// Open multiple files in the project tree at once.
    OpenMultipleDocuments(Vec<std::path::PathBuf>),
    /// Remove a workspace from the persistence list.
    RemoveWorkspace(String),
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
    /// Reload the workspace directory tree from disk.
    RefreshWorkspace,
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
    pub tab_view_modes: Vec<TabViewMode>,
    /// Split-layout cache per tab. Initialized from persisted defaults when a tab is first opened.
    pub tab_split_states: Vec<TabSplitState>,

    /// Plugin registry (will be referenced during plugin widget integration in future Task 5.x).
    pub _plugin_registry: PluginRegistry,
    /// Non-fatal status message for the status bar.
    pub status_message: Option<String>,
    /// Show/hide the workspace panel.
    pub show_workspace: bool,
    /// Whether the workspace file filter is enabled.
    pub filter_enabled: bool,
    /// The current regular expression query for the workspace folder filter.
    pub filter_query: String,
    /// Cache of visible paths for the current filter query. Tuple of (query, visible_paths_set)
    pub filter_cache: Option<(String, std::collections::HashSet<std::path::PathBuf>)>,
    /// Show/hide the settings window.
    pub show_settings: bool,
    /// Show/hide the search modal.
    pub show_search_modal: bool,
    /// The search query for the file search modal.
    pub search_query: String,
    /// Regular expression for including files/dirs in search results.
    pub search_include_pattern: String,
    /// Regular expression for excluding files/dirs in search results.
    pub search_exclude_pattern: String,
    /// Cached parameters used for the last search to detect programmatic changes.
    pub last_search_params: Option<(String, String, String)>,
    /// The cached search results.
    pub search_results: Vec<std::path::PathBuf>,
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
    /// Indicates if a workspace is currently being loaded asynchronously in the background.
    pub is_loading_workspace: bool,
    /// Facade for memory and persistent cache storage.
    pub cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    /// Set of manually expanded directories in the workspace tree.
    pub expanded_directories: std::collections::HashSet<std::path::PathBuf>,
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
        cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    ) -> Self {
        // ai_registry is planned for future AI integration. Currently unused.
        let _ = ai_registry;
        Self {
            workspace: None,
            open_documents: Vec::new(),
            active_doc_idx: None,
            tab_view_modes: Vec::new(),
            tab_split_states: Vec::new(),
            _plugin_registry: plugin_registry,
            status_message: None,
            show_workspace: true,
            filter_enabled: false,
            filter_query: String::new(),
            filter_cache: None,
            show_settings: false,
            show_search_modal: false,
            search_query: String::new(),
            search_include_pattern: String::new(),
            search_exclude_pattern: String::new(),
            last_search_params: None,
            search_results: Vec::new(),
            active_settings_tab: SettingsTab::default(),
            force_tree_open: None,
            scroll_fraction: 0.0,
            scroll_source: ScrollSource::Neither,
            editor_max_scroll: 0.0,
            preview_max_scroll: 0.0,
            settings,
            is_loading_workspace: false,
            cache,
            expanded_directories: std::collections::HashSet::new(),
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
            .and_then(|doc| {
                self.tab_view_modes
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.mode)
            })
            .unwrap_or(ViewMode::PreviewOnly)
    }

    /// Sets the view mode of the active tab.
    pub fn set_active_view_mode(&mut self, mode: ViewMode) {
        if let Some(doc) = self.active_document() {
            let path = doc.path.clone();
            if let Some(t) = self.tab_view_modes.iter_mut().find(|t| t.path == path) {
                t.mode = mode;
            } else {
                self.tab_view_modes.push(TabViewMode { path, mode });
            }
        }
    }

    fn split_defaults(&self) -> SplitViewState {
        SplitViewState {
            direction: self.settings.settings().layout.split_direction,
            order: self.settings.settings().layout.pane_order,
        }
    }

    pub fn initialize_tab_split_state(&mut self, path: impl Into<std::path::PathBuf>) {
        let p = path.into();
        if !self.tab_split_states.iter().any(|t| t.path == p) {
            let defaults = self.split_defaults();
            self.tab_split_states.push(TabSplitState {
                path: p,
                state: defaults,
            });
        }
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
            .and_then(|doc| {
                self.tab_split_states
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.state.direction)
            })
            .unwrap_or_else(|| self.split_defaults().direction)
    }

    /// Returns the pane order for the active tab.
    pub fn active_pane_order(&self) -> katana_platform::PaneOrder {
        self.active_document()
            .and_then(|doc| {
                self.tab_split_states
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.state.order)
            })
            .unwrap_or_else(|| self.split_defaults().order)
    }

    /// Sets the split direction for the active tab (temporary — not persisted to disk).
    pub fn set_active_split_direction(&mut self, dir: katana_platform::SplitDirection) {
        let Some(path) = self.active_path().map(std::path::Path::to_path_buf) else {
            return;
        };
        if let Some(t) = self.tab_split_states.iter_mut().find(|t| t.path == path) {
            t.state.direction = dir;
        } else {
            let mut defaults = self.split_defaults();
            defaults.direction = dir;
            self.tab_split_states.push(TabSplitState {
                path,
                state: defaults,
            });
        }
    }

    /// Sets the pane order for the active tab (temporary — not persisted to disk).
    pub fn set_active_pane_order(&mut self, order: katana_platform::PaneOrder) {
        let Some(path) = self.active_path().map(std::path::Path::to_path_buf) else {
            return;
        };
        if let Some(t) = self.tab_split_states.iter_mut().find(|t| t.path == path) {
            t.state.order = order;
        } else {
            let mut defaults = self.split_defaults();
            defaults.order = order;
            self.tab_split_states.push(TabSplitState {
                path,
                state: defaults,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_platform::{PaneOrder, SplitDirection};
    use std::path::PathBuf;

    fn make_state_with_doc(path: &str) -> AppState {
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
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
        state.settings.settings_mut().layout.split_direction = SplitDirection::Horizontal;
        state.initialize_tab_split_state("/tmp/a.md");
        state.settings.settings_mut().layout.split_direction = SplitDirection::Vertical;

        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);
    }

    #[test]
    fn test_pane_order_is_cached_per_tab_after_settings_change() {
        let mut state = make_state_with_doc("/tmp/b.md");
        state.settings.settings_mut().layout.pane_order = PaneOrder::EditorFirst;
        state.initialize_tab_split_state("/tmp/b.md");
        state.settings.settings_mut().layout.pane_order = PaneOrder::PreviewFirst;

        assert_eq!(state.active_pane_order(), PaneOrder::EditorFirst);
    }

    #[test]
    fn test_new_tab_uses_latest_persisted_split_defaults() {
        let mut state = make_state_with_doc("/tmp/a.md");
        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);

        state.settings.settings_mut().layout.split_direction = SplitDirection::Vertical;
        state.settings.settings_mut().layout.pane_order = PaneOrder::PreviewFirst;

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

        state.settings.settings_mut().layout.split_direction = SplitDirection::Horizontal;
        state.settings.settings_mut().layout.pane_order = PaneOrder::PreviewFirst;

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
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        // No documents open, active_doc_idx is None — should return early without panic
        state.ensure_active_split_state();
        assert!(state.tab_split_states.is_empty());
    }

    #[test]
    fn test_set_active_split_direction_no_active_doc() {
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        // No documents open — should return early without panic
        state.set_active_split_direction(SplitDirection::Vertical);
        assert!(state.tab_split_states.is_empty());
    }

    #[test]
    fn test_set_active_pane_order_no_active_doc() {
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        // No documents open — should return early without panic
        state.set_active_pane_order(PaneOrder::PreviewFirst);
        assert!(state.tab_split_states.is_empty());
    }
    #[test]
    fn test_set_active_split_direction_uninitialized() {
        let mut state = make_state_with_doc("/tmp/uninit.md");
        state.tab_split_states.clear();
        state.set_active_split_direction(SplitDirection::Vertical);
        assert_eq!(state.tab_split_states.len(), 1);
        assert_eq!(
            state.tab_split_states[0].state.direction,
            SplitDirection::Vertical
        );
    }

    #[test]
    fn test_set_active_pane_order_uninitialized() {
        let mut state = make_state_with_doc("/tmp/uninit.md");
        state.tab_split_states.clear();
        state.set_active_pane_order(PaneOrder::PreviewFirst);
        assert_eq!(state.tab_split_states.len(), 1);
        assert_eq!(
            state.tab_split_states[0].state.order,
            PaneOrder::PreviewFirst
        );
    }

    #[test]
    fn test_ensure_active_split_state() {
        let mut state = make_state_with_doc("/tmp/c.md");
        state.tab_split_states.clear();
        state.ensure_active_split_state();
        assert_eq!(state.tab_split_states.len(), 1);

        let mut empty_state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        empty_state.ensure_active_split_state(); // Should safely return early
        assert_eq!(empty_state.tab_split_states.len(), 0);
    }
}
