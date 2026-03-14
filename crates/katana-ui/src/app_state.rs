//! Shared application state.
//!
//! A single `AppState` container is owned by the egui application. UI
//! components render from this state and dispatch `AppAction` values back
//! through the update loop.

use katana_core::{
    ai::AiProviderRegistry, document::Document, plugin::PluginRegistry, workspace::Workspace,
};

/// User-visible actions dispatched from UI components to the core update loop.
#[derive(Debug)]
pub enum AppAction {
    /// Open a workspace at the given path.
    OpenWorkspace(std::path::PathBuf),
    /// Select a file in the project tree.
    SelectDocument(std::path::PathBuf),
    /// Update the active document buffer.
    UpdateBuffer(String),
    /// Explicitly save the active document.
    SaveDocument,
    /// ダイアグラムを含めてプレビューを完全再レンダリングする。
    RefreshDiagrams,
    /// No-op (used internally).
    None,
}

/// Top-level application state shared across all UI components.
pub struct AppState {
    /// The currently open workspace, if any.
    pub workspace: Option<Workspace>,
    /// The currently active document, if any.
    pub active_document: Option<Document>,
    /// AI provider registry.
    pub ai_registry: AiProviderRegistry,
    /// Plugin registry（将来の Task 5.x でプラグインウィジェット統合時に参照する）。
    pub _plugin_registry: PluginRegistry,
    /// Non-fatal status message for the status bar.
    pub status_message: Option<String>,
}

impl AppState {
    pub fn new(ai_registry: AiProviderRegistry, plugin_registry: PluginRegistry) -> Self {
        Self {
            workspace: None,
            active_document: None,
            ai_registry,
            _plugin_registry: plugin_registry,
            status_message: None,
        }
    }

    /// Whether the active document has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.active_document
            .as_ref()
            .map(|d| d.is_dirty)
            .unwrap_or(false)
    }

    /// Whether the AI panel should be shown as available.
    pub fn ai_available(&self) -> bool {
        self.ai_registry.has_active_provider()
    }
}
