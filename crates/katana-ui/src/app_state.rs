//! Shared application state.
//!
//! `scroll_fraction` — Splitモードでエディタ/プレビュー間の
//! スクロール位置を比率 (0.0–1.0) で双方向同期するためのステート。
//!
//! A single `AppState` container is owned by the egui application. UI
//! components render from this state and dispatch `AppAction` values back
//! through the update loop.

use katana_core::{
    ai::AiProviderRegistry, document::Document, plugin::PluginRegistry, workspace::Workspace,
};
use std::collections::HashMap;

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
    /// タブごとの表示モード。キーはファイルパス（タブ閉じ後のインデックスずれを防ぐ）。
    pub tab_view_modes: HashMap<std::path::PathBuf, ViewMode>,

    /// Plugin registry（将来の Task 5.x でプラグインウィジェット統合時に参照する）。
    pub _plugin_registry: PluginRegistry,
    /// Non-fatal status message for the status bar.
    pub status_message: Option<String>,
    /// ワークスペースパネルの表示・非表示。
    pub show_workspace: bool,
    /// ワークスペースツリーの全展開/全折畳トリガー。Some(true)=全展開, Some(false)=全折畳。
    pub force_tree_open: Option<bool>,
    /// Splitモード スクロール同期: 正規化されたスクロール位置 (0.0–1.0)。
    pub scroll_fraction: f32,
    /// スクロール操作の発生元。連鎖反応（無限ループ）を防ぐ。
    pub scroll_source: ScrollSource,
    /// 前フレームのエディタ側 max_scroll (content_height - viewport_height)。
    pub editor_max_scroll: f32,
    /// 前フレームのプレビュー側 max_scroll (content_height - viewport_height)。
    pub preview_max_scroll: f32,
}

/// スクロール操作の発生元を示す。連鎖反応を防ぐために使用。
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ScrollSource {
    /// どちらからも変更なし（初期状態）。
    #[default]
    Neither,
    /// エディタペインからのスクロール。
    Editor,
    /// プレビューペインからのスクロール。
    Preview,
}

impl AppState {
    pub fn new(ai_registry: AiProviderRegistry, plugin_registry: PluginRegistry) -> Self {
        // ai_registry は将来 AI 統合時に使用予定。現時点では未使用。
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
        }
    }

    /// Whether the active document has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.active_document().map(|d| d.is_dirty).unwrap_or(false)
    }

    /// 現在アクティブなドキュメントを参照する
    pub fn active_document(&self) -> Option<&Document> {
        self.active_doc_idx
            .and_then(|idx| self.open_documents.get(idx))
    }

    /// 現在アクティブなドキュメントをミュータブルに参照する
    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_doc_idx
            .and_then(|idx| self.open_documents.get_mut(idx))
    }

    /// 現在アクティブなドキュメントのパスを返す（ワークスペースツリーのハイライトに使用）。
    pub fn active_path(&self) -> Option<&std::path::Path> {
        self.active_document().map(|d| d.path.as_path())
    }

    /// アクティブなタブの表示モードを返す。タブ未選択時はデフォルト値。
    pub fn active_view_mode(&self) -> ViewMode {
        self.active_document()
            .and_then(|doc| self.tab_view_modes.get(&doc.path))
            .copied()
            .unwrap_or(ViewMode::PreviewOnly)
    }

    /// アクティブなタブの表示モードを設定する。
    pub fn set_active_view_mode(&mut self, mode: ViewMode) {
        if let Some(doc) = self.active_document() {
            let path = doc.path.clone();
            self.tab_view_modes.insert(path, mode);
        }
    }
}
