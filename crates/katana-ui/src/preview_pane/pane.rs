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
    pub render_rx: Option<std::sync::mpsc::Receiver<RenderMessage>>,
    pub is_loading: bool,
    pub cancel_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub(crate) md_file_path: std::path::PathBuf,
    pub concurrency_reduction_requested: bool,
    pub image_preload_queue: Vec<std::path::PathBuf>,
    pub image_cache: std::collections::HashSet<std::path::PathBuf>,
    pub viewer_states: Vec<ViewerState>,
    pub fullscreen_image: Option<usize>,
    pub fullscreen_viewer_state: ViewerState,
    pub was_os_fullscreen_before_modal: bool,
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
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}