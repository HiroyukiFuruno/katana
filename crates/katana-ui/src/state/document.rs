use katana_core::document::Document;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewMode {
    PreviewOnly,
    CodeOnly,
    Split,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SplitViewState {
    pub direction: katana_platform::SplitDirection,
    pub order: katana_platform::PaneOrder,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TabViewMode {
    pub path: PathBuf,
    pub mode: ViewMode,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TabSplitState {
    pub path: PathBuf,
    pub state: SplitViewState,
}

pub struct DocumentState {
    pub open_documents: Vec<Document>,
    pub active_doc_idx: Option<usize>,
    pub tab_view_modes: Vec<TabViewMode>,
    pub tab_split_states: Vec<TabSplitState>,
    pub recently_closed_tabs: VecDeque<PathBuf>,
    pub last_auto_save: Option<Instant>,
}

impl Default for DocumentState {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentState {
    pub const MAX_RECENTLY_CLOSED_TABS: usize = 10;

    pub fn new() -> Self {
        Self {
            open_documents: Vec::new(),
            active_doc_idx: None,
            tab_view_modes: Vec::new(),
            tab_split_states: Vec::new(),
            recently_closed_tabs: VecDeque::with_capacity(Self::MAX_RECENTLY_CLOSED_TABS),
            last_auto_save: None,
        }
    }

    pub fn active_document(&self) -> Option<&Document> {
        self.active_doc_idx
            .and_then(|idx| self.open_documents.get(idx))
    }
}
