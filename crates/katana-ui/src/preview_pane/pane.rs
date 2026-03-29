use super::types::*;
use eframe::egui;
use egui_commonmark::CommonMarkCache;
use katana_core::markdown::diagram::DiagramKind;
use katana_core::markdown::outline::OutlineItem;

use super::viewer::ViewerState;
#[derive(Default)]
pub struct PreviewPane {
    pub(crate) commonmark_cache: CommonMarkCache,
    pub sections: Vec<RenderedSection>,
    pub outline_items: Vec<OutlineItem>,
    pub heading_anchors: Vec<(std::ops::Range<usize>, egui::Rect)>,
    pub content_top_y: f32,
    pub visible_rect: Option<egui::Rect>,
    pub scroll_request: Option<usize>,
    /// Channel for background rendering completion notifications.
    pub render_rx: Option<std::sync::mpsc::Receiver<RenderMessage>>,
    /// State flag indicating if background rendering is currently in progress.
    pub is_loading: bool,
    /// Token used to abort background rendering threads if the pane is dropped or reloaded.
    pub cancel_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Path to the currently previewed Markdown file (for resolving relative paths in render_html_fn).
    pub(crate) md_file_path: std::path::PathBuf,
    /// Signals shell to reduce diagram concurrency settings if a worker fails.
    pub concurrency_reduction_requested: bool,
    /// Queue of local image paths to preload in the background.
    pub image_preload_queue: Vec<std::path::PathBuf>,
    /// Cache tracking which local images have been requested to egui's background loader.
    pub image_cache: std::collections::HashSet<std::path::PathBuf>,
    /// Per-image/diagram zoom and pan state, indexed by section index.
    pub viewer_states: Vec<ViewerState>,
    /// Index of the section currently displayed in fullscreen modal, if any.
    pub fullscreen_image: Option<usize>,
    /// Independent viewer state for fullscreen modal (not synced with preview).
    pub fullscreen_viewer_state: ViewerState,
    /// Tracks whether the OS was already in fullscreen mode before opening the modal.
    pub was_os_fullscreen_before_modal: bool,
    /// Cached egui Context for background threads to signal repaint on completion.
    /// Set from `show()` / `show_content()` so threads can wake the UI without polling.
    pub(crate) repaint_ctx: Option<egui::Context>,
}

pub(crate) struct RenderJob {
    pub(crate) kind: DiagramKind,
    pub(crate) src: String,
    pub(crate) path: std::path::PathBuf,
    pub(crate) cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    pub(crate) force: bool,
    pub(crate) source_lines: usize,
}

pub enum RenderMessage {
    Section {
        kind: String,
        source: String,
        section: RenderedSection,
    },
    ReduceConcurrency,
}

impl Drop for PreviewPane {
    fn drop(&mut self) {
        // Automatically kill running background children explicitly if a pane is deleted (e.g., closing a tab)
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
