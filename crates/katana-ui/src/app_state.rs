//! Shared application state.
//!
//! A single `AppState` container is owned by the egui application. UI
//! components render from this state and dispatch `AppAction` values back
//! through the update loop.

use katana_core::{
    ai::AiProviderRegistry, document::Document, plugin::PluginRegistry, workspace::Workspace,
};

/// UIレイアウトの表示モード
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
    /// タブを閉じる
    CloseDocument(usize),
    /// アクティブなドキュメントのバッファを更新
    UpdateBuffer(String),
    /// Explicitly save the active document.
    SaveDocument,
    /// ダイアグラムを含めてプレビューを完全再レンダリングする。
    RefreshDiagrams,
    /// 表示モードの変更
    SetViewMode(ViewMode),
    /// 言語変更
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
    /// Current view mode.
    pub view_mode: ViewMode,
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
            open_documents: Vec::new(),
            active_doc_idx: None,
            view_mode: ViewMode::PreviewOnly,
            ai_registry,
            _plugin_registry: plugin_registry,
            status_message: None,
        }
    }

    /// Whether the active document has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.active_document()
            .map(|d| d.is_dirty)
            .unwrap_or(false)
    }

    /// 現在アクティブなドキュメントを参照する
    pub fn active_document(&self) -> Option<&Document> {
        self.active_doc_idx.and_then(|idx| self.open_documents.get(idx))
    }

    /// 現在アクティブなドキュメントをミュータブルに参照する
    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_doc_idx.and_then(|idx| self.open_documents.get_mut(idx))
    }

    /// Whether the AI panel should be shown as available.
    pub fn ai_available(&self) -> bool {
        self.ai_registry.has_active_provider()
    }
}
