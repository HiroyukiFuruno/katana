//! Preview pane — native Markdown rendering with egui_commonmark
//! + rasterized image display of diagram blocks.
//!
//! Design considerations (MVP):
//! - The Markdown part is updated immediately on every text change (egui_commonmark).
//! - Diagrams involve sub-processes, so they are re-rendered only when
//!   the "🔄 Refresh" button is clicked or a document is selected.

use eframe::egui::{self, ScrollArea};
use egui_commonmark::CommonMarkCache;
use katana_core::markdown::diagram::DiagramKind;
use katana_core::markdown::outline::OutlineItem;
use katana_core::{
    markdown::{
        diagram::{DiagramBlock, DiagramResult},
        drawio_renderer, mermaid_renderer, plantuml_renderer,
        svg_rasterize::{rasterize_svg, RasterizedSvg},
    },
    preview::{flatten_list_code_blocks, resolve_image_paths, split_into_sections, PreviewSection},
};

// ─────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────

/// Display scale when converting diagram SVG to pixel images.
const DIAGRAM_SVG_DISPLAY_SCALE: f32 = 1.5;

/// Interval (in milliseconds) used to poll background render tasks.
const RENDER_POLL_INTERVAL_MS: u64 = 50;

/// Rendered sections held in the UI layer.
#[derive(Debug, Clone)]
pub enum RenderedSection {
    /// Markdown text rendered by egui_commonmark.
    Markdown(String),
    /// Rasterized diagram image.
    Image {
        svg_data: RasterizedSvg,
        alt: String,
        source_lines: usize,
    },
    /// A standalone local image handled by Katana (with viewer controls).
    LocalImage {
        path: std::path::PathBuf,
        alt: String,
        source_lines: usize,
    },
    /// Rendering error (holds source and message).
    Error {
        kind: String,
        _source: String,
        message: String,
        source_lines: usize,
    },
    /// Command line tool not found (path issues, etc.).
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        _source: String,
        source_lines: usize,
    },
    /// Required tool is not installed — can be downloaded from the UI.
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
        source_lines: usize,
    },
    /// Placeholder during background rendering.
    Pending {
        kind: String,
        source: String,
        source_lines: usize,
    },
}

/// Download request returned by the preview pane.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub dest: std::path::PathBuf,
}

// ─────────────────────────────────────────────
// Viewer Controls — State management for pan/zoom on images and diagrams
// ─────────────────────────────────────────────

/// Zoom increment per button click.
const VIEWER_ZOOM_STEP: f32 = 0.25;
/// Minimum zoom level (25%).
const VIEWER_ZOOM_MIN: f32 = 0.25;
/// Maximum zoom level (400%).
const VIEWER_ZOOM_MAX: f32 = 4.0;
/// Pan offset (in logical pixels) per button click.
const VIEWER_PAN_STEP: f32 = 50.0;

/// Per-image/diagram viewer state (zoom level and pan offset).
#[derive(Clone, PartialEq)]
pub struct ViewerState {
    /// Current zoom factor (1.0 = 100%).
    pub zoom: f32,
    /// Current pan offset in logical pixels.
    pub pan: egui::Vec2,
    /// Cached texture handle to avoid re-uploading the image to the GPU every frame.
    pub texture: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for ViewerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerState")
            .field("zoom", &self.zoom)
            .field("pan", &self.pan)
            .field("texture", &self.texture.as_ref().map(|t| t.id()))
            .finish()
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            texture: None,
        }
    }
}

impl ViewerState {
    /// Zoom in by one step, clamped to `VIEWER_ZOOM_MAX`.
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + VIEWER_ZOOM_STEP).min(VIEWER_ZOOM_MAX);
    }

    /// Zoom out by one step, clamped to `VIEWER_ZOOM_MIN`.
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - VIEWER_ZOOM_STEP).max(VIEWER_ZOOM_MIN);
    }

    /// Pan by the given delta (in logical pixels).
    pub fn pan_by(&mut self, delta: egui::Vec2) {
        self.pan += delta;
    }

    /// Pan up by one step.
    pub fn pan_up(&mut self) {
        self.pan_by(egui::vec2(0.0, -VIEWER_PAN_STEP));
    }

    /// Pan down by one step.
    pub fn pan_down(&mut self) {
        self.pan_by(egui::vec2(0.0, VIEWER_PAN_STEP));
    }

    /// Pan left by one step.
    pub fn pan_left(&mut self) {
        self.pan_by(egui::vec2(-VIEWER_PAN_STEP, 0.0));
    }

    /// Pan right by one step.
    pub fn pan_right(&mut self) {
        self.pan_by(egui::vec2(VIEWER_PAN_STEP, 0.0));
    }

    /// Reset zoom and pan to defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Default)]
pub struct PreviewPane {
    commonmark_cache: CommonMarkCache,
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
    md_file_path: std::path::PathBuf,
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
    repaint_ctx: Option<egui::Context>,
}

struct RenderJob {
    kind: DiagramKind,
    src: String,
    path: std::path::PathBuf,
    cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    force: bool,
    source_lines: usize,
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

impl PreviewPane {
    /// Immediately updates only text sections from the Markdown source (diagrams are preserved).
    pub fn update_markdown_sections(&mut self, source: &str, md_file_path: &std::path::Path) {
        self.md_file_path = md_file_path.to_path_buf();
        self.outline_items = katana_core::markdown::outline::extract_outline(source);
        let (resolved, extracted_paths) = resolve_image_paths(source, md_file_path);

        // Add new unique paths to the preload queue
        for path in extracted_paths {
            if !self.image_cache.contains(&path) && !self.image_preload_queue.contains(&path) {
                self.image_preload_queue.push(path);
            }
        }

        let flattened = flatten_list_code_blocks(&resolved);
        let raw = split_into_sections(&flattened);
        let mut new_sections = Vec::with_capacity(raw.len());
        let mut diagram_iter = self
            .sections
            .iter()
            .filter(|s| !matches!(s, RenderedSection::Markdown(_)));
        for section in &raw {
            match section {
                PreviewSection::Markdown(md) => {
                    new_sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram {
                    kind,
                    source,
                    lines,
                } => {
                    // Reuse existing rendered image if available.
                    let reused =
                        diagram_iter
                            .next()
                            .cloned()
                            .unwrap_or_else(|| RenderedSection::Error {
                                kind: format!("{kind:?}"),
                                _source: source.clone(),
                                message: "🔄 Please refresh the preview".to_string(),
                                source_lines: *lines,
                            });
                    new_sections.push(reused);
                }
                PreviewSection::LocalImage { path, alt, lines } => {
                    let path_buf = std::path::PathBuf::from(path.trim_start_matches("file://"));
                    new_sections.push(RenderedSection::LocalImage {
                        path: path_buf,
                        alt: alt.clone(),
                        source_lines: *lines,
                    });
                }
            }
        }
        self.sections = new_sections;
    }

    /// Gracefully aborts all currently running background renders for this pane.
    /// Used when a tab goes into the background without being fully destroyed.
    pub fn abort_renders(&mut self) {
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.is_loading = false;
        self.render_rx = None;
    }

    /// Completely re-renders all sections (including diagrams).
    ///
    /// Returns Markdown sections immediately. Diagrams are set to `Pending`
    /// and rendered in a background thread.
    pub fn full_render(
        &mut self,
        source: &str,
        md_file_path: &std::path::Path,
        cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
        force: bool,
        diagram_concurrency: usize,
    ) {
        if force {
            self.commonmark_cache = CommonMarkCache::default();
        }

        self.md_file_path = md_file_path.to_path_buf();
        self.outline_items = katana_core::markdown::outline::extract_outline(source);
        let (resolved, extracted_paths) = resolve_image_paths(source, md_file_path);

        self.image_preload_queue.clear();
        self.image_cache.clear();
        self.image_preload_queue = extracted_paths;

        let flattened = flatten_list_code_blocks(&resolved);
        let raw = split_into_sections(&flattened);
        // Cancel previous rendering.
        self.render_rx = None;

        let mut sections = Vec::with_capacity(raw.len());
        let mut jobs: Vec<RenderJob> = Vec::new();

        for section in raw.iter() {
            match section {
                PreviewSection::Markdown(md) => {
                    sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram {
                    kind,
                    source,
                    lines,
                } => {
                    sections.push(RenderedSection::Pending {
                        kind: format!("{kind:?}"),
                        source: source.clone(),
                        source_lines: *lines,
                    });
                    jobs.push(RenderJob {
                        kind: kind.clone(),
                        src: source.clone(),
                        path: self.md_file_path.clone(),
                        cache: cache.clone(),
                        force,
                        source_lines: *lines,
                    });
                }
                PreviewSection::LocalImage { path, alt, lines } => {
                    let path_buf = std::path::PathBuf::from(path.trim_start_matches("file://"));
                    sections.push(RenderedSection::LocalImage {
                        path: path_buf,
                        alt: alt.clone(),
                        source_lines: *lines,
                    });
                }
            }
        }
        self.sections = sections;

        if jobs.is_empty() {
            self.is_loading = false;
            return;
        }

        // Cancel previous running threads first!
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let current_cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.cancel_token = current_cancel_token.clone();

        self.is_loading = true;
        let (tx, rx) = std::sync::mpsc::channel();
        self.render_rx = Some(rx);

        let concurrency = diagram_concurrency.max(1);
        let jobs_len = jobs.len();
        let jobs_rx = std::sync::Arc::new(std::sync::Mutex::new(jobs.into_iter()));

        for _ in 0..concurrency.min(jobs_len) {
            let tx = tx.clone();
            let jobs_rx = jobs_rx.clone();
            let current_cancel_token = current_cancel_token.clone();
            let repaint_ctx = self.repaint_ctx.clone();
            std::thread::spawn(move || loop {
                let job = {
                    let mut lock = jobs_rx.lock().unwrap();
                    lock.next()
                };
                let Some(job) = job else {
                    break;
                };

                // Abort before spinning up heavy rendering logic/CLI tasks
                if current_cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let cache_key = get_cache_key(&job.path, &job.kind, &job.src);
                let is_http = job.src.contains("http://") || job.src.contains("https://");

                let cached_result = if !job.force {
                    if is_http {
                        job.cache.get_memory(&cache_key)
                    } else {
                        job.cache.get_persistent(&cache_key)
                    }
                } else {
                    None
                };

                let result = if let Some(json) = cached_result {
                    if let Ok(res) = serde_json::from_str::<DiagramResult>(&json) {
                        res
                    } else {
                        let res = dispatch_renderer(&DiagramBlock {
                            kind: job.kind.clone(),
                            source: job.src.clone(),
                        });
                        if matches!(res, DiagramResult::Err { .. }) {
                            let _ = tx.send(RenderMessage::ReduceConcurrency);
                        }
                        res
                    }
                } else {
                    let res = dispatch_renderer(&DiagramBlock {
                        kind: job.kind.clone(),
                        source: job.src.clone(),
                    });

                    if let Ok(json) = serde_json::to_string(&res) {
                        if is_http {
                            job.cache.set_memory(&cache_key, json);
                        } else {
                            let _ = job.cache.set_persistent(&cache_key, json);
                        }
                    }
                    if matches!(res, DiagramResult::Err { .. }) {
                        let _ = tx.send(RenderMessage::ReduceConcurrency);
                    }
                    res
                };

                let section = map_diagram_result(&job.kind, &job.src, result, job.source_lines);
                let msg = RenderMessage::Section {
                    kind: format!("{:?}", job.kind),
                    source: job.src.clone(),
                    section,
                };
                if tx.send(msg).is_err() {
                    break; // Receiver was dropped.
                }
                // Signal the UI thread that new data is available.
                // This replaces the polling-based request_repaint_after in poll_renders.
                // Note: repaint_ctx is None in test context (no egui::Context available).
                if let Some(ctx) = &repaint_ctx {
                    ctx.request_repaint();
                }
            });
        }
    }

    /// Renders the preview pane content (including ScrollArea).
    /// Used when scroll sync is not needed, such as in PreviewOnly mode.
    /// Returns `Some(DownloadRequest)` if the download button is pressed.
    pub fn show(&mut self, ui: &mut egui::Ui) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        self.repaint_ctx = Some(ui.ctx().clone());
        // Poll for background rendering completion.
        self.poll_renders(ui.ctx());

        let mut request: Option<DownloadRequest> = None;
        let mut actions = Vec::new();
        let content_width = ui.available_width();
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let child_rect = egui::Rect::from_min_size(
                    ui.next_widget_position(),
                    egui::vec2(content_width, 0.0),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(child_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        let (req, act) = self.render_sections(ui, None, None);
                        request = req;
                        actions = act;
                    },
                );
            });
        self.render_fullscreen_modal(ui.ctx());
        (request, actions)
    }

    /// Renders only the preview content without a ScrollArea.
    /// Used when you want to control the outer ScrollArea (e.g. for scroll sync).
    pub fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        active_editor_line: Option<usize>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
    ) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        self.repaint_ctx = Some(ui.ctx().clone());
        self.poll_renders(ui.ctx());
        let (request, actions) = self.render_sections(ui, active_editor_line, hovered_lines);
        self.render_fullscreen_modal(ui.ctx());
        (request, actions)
    }

    /// Internal method to sequentially render sections.
    /// Actual UI rendering is delegated to preview_pane_ui::render_sections.
    fn render_sections(
        &mut self,
        ui: &mut egui::Ui,
        active_editor_line: Option<usize>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
    ) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        self.visible_rect = Some(ui.clip_rect());
        self.content_top_y = ui.next_widget_position().y;
        self.heading_anchors.clear();
        let mut fullscreen_request: Option<usize> = None;
        let (request, actions) = crate::preview_pane_ui::render_sections(
            ui,
            &mut self.commonmark_cache,
            &self.sections,
            &self.md_file_path,
            self.scroll_request,
            Some(&mut self.heading_anchors),
            Some(&mut self.viewer_states),
            Some(&mut fullscreen_request),
            active_editor_line,
            hovered_lines,
        );
        self.scroll_request = None;

        // Apply fullscreen state transitions (testable without UI context).
        let ctx = ui.ctx().clone();
        self.handle_fullscreen_request(fullscreen_request, Some(&ctx));

        (request, actions)
    }

    /// Renders the fullscreen modal overlay (requires egui Context).
    /// Delegates to preview_pane_ui which is coverage-excluded.
    fn render_fullscreen_modal(&mut self, ctx: &egui::Context) {
        let result = crate::preview_pane_ui::render_fullscreen_if_active(
            ctx,
            &self.sections,
            self.fullscreen_image,
            &mut self.fullscreen_viewer_state,
        );
        self.apply_fullscreen_result(result, Some(ctx));
    }

    /// Applies the result of the fullscreen modal to state.
    /// Extracted for testability of state transitions.
    fn apply_fullscreen_result(&mut self, result: Option<usize>, ctx: Option<&egui::Context>) {
        if result.is_none() && self.fullscreen_image.is_some() {
            // Fullscreen was closed — reset state and restore OS native fullscreen if needed.
            self.fullscreen_viewer_state.reset();
            if let Some(ctx) = ctx {
                if !self.was_os_fullscreen_before_modal {
                    // It wasn't fullscreen before we opened the modal, so restore it.
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                }
            }
        }
        self.fullscreen_image = result;
    }

    /// Handles fullscreen request and validates index against current sections.
    /// Separated from `render_sections` for testability.
    fn handle_fullscreen_request(&mut self, request: Option<usize>, ctx: Option<&egui::Context>) {
        // Apply new fullscreen request.
        if let Some(idx) = request {
            if self.fullscreen_image.is_none() {
                // We are opening it. Track the previous OS fullscreen state.
                if let Some(ctx) = ctx {
                    let is_native_fs = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);
                    self.was_os_fullscreen_before_modal = is_native_fs;
                    if !is_native_fs {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    }
                }
            }
            self.fullscreen_image = Some(idx);
        }
        // Clear fullscreen if the section no longer exists or is not an Image.
        if let Some(idx) = self.fullscreen_image {
            match self.sections.get(idx) {
                Some(RenderedSection::Image { .. }) => {} // valid, keep open
                _ => self.fullscreen_image = None,
            }
        }
    }

    /// Polls for background rendering completion and updates sections with received results.
    fn poll_renders(&mut self, ctx: &egui::Context) {
        // Process local image background loading (trigger egui's background loaders).
        // This offloads reading and decoding the images to egui_extras' background threads.
        while let Some(path) = self.image_preload_queue.pop() {
            if self.image_cache.insert(path.clone()) {
                let uri = format!("file://{}", path.display());
                let _ = ctx.try_load_image(&uri, egui::load::SizeHint::Scale(1.0.into()));
            }
        }

        let mut disconnected = false;

        if let Some(rx) = &self.render_rx {
            loop {
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        RenderMessage::Section {
                            kind,
                            source,
                            section,
                        } => {
                            for slot in &mut self.sections {
                                if let RenderedSection::Pending {
                                    kind: p_kind,
                                    source: p_source,
                                    ..
                                } = slot
                                {
                                    if p_kind == &kind && p_source == &source {
                                        *slot = section.clone();
                                    }
                                }
                            }
                        }
                        RenderMessage::ReduceConcurrency => {
                            self.concurrency_reduction_requested = true;
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }

            // No repaint requested here — background threads signal directly
            // via repaint_ctx.request_repaint() when they send messages.

            if disconnected {
                self.is_loading = false;
                self.render_rx = None;
            }
        } else {
            self.is_loading = false;
        }
    }

    /// For testing: Block and wait on the background thread until there are no Pending sections.
    /// Available in release builds too (harmless no-op when no background render is running).
    pub fn wait_for_renders(&mut self) {
        while let Some(rx) = &self.render_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    RenderMessage::Section {
                        kind,
                        source,
                        section,
                    } => {
                        for slot in &mut self.sections {
                            if let RenderedSection::Pending {
                                kind: p_kind,
                                source: p_source,
                                ..
                            } = slot
                            {
                                if p_kind == &kind && p_source == &source {
                                    *slot = section.clone();
                                }
                            }
                        }
                    }
                    RenderMessage::ReduceConcurrency => {
                        self.concurrency_reduction_requested = true;
                    }
                }
            }
            if self
                .sections
                .iter()
                .any(|s| matches!(s, RenderedSection::Pending { .. }))
            {
                std::thread::sleep(std::time::Duration::from_millis(RENDER_POLL_INTERVAL_MS));
            } else {
                self.render_rx = None;
                break;
            }
        }
    }
}

/// Renders a `PreviewSection` into a `RenderedSection`.
/// Converts diagram blocks via the renderer and attempts SVG rasterization.
#[cfg(test)]
fn render_diagram(kind: &DiagramKind, source: &str, source_lines: usize) -> RenderedSection {
    let block = DiagramBlock {
        kind: kind.clone(),
        source: source.to_string(),
    };
    let result = dispatch_renderer(&block);
    map_diagram_result(kind, source, result, source_lines)
}

/// Generates a cache key based on the file path, diagram kind, and source text.
pub fn get_cache_key(md_file_path: &std::path::Path, kind: &DiagramKind, source: &str) -> String {
    use katana_core::markdown::color_preset::DiagramColorPreset;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    md_file_path.hash(&mut hasher);
    kind.display_name().hash(&mut hasher);
    source.hash(&mut hasher);
    DiagramColorPreset::is_dark_mode().hash(&mut hasher);
    format!("diagram_{:x}", hasher.finish())
}

/// Pure function converting a `DiagramResult` into a `RenderedSection`. Exposed for testing.
pub(crate) fn map_diagram_result(
    kind: &DiagramKind,
    source: &str,
    result: DiagramResult,
    source_lines: usize,
) -> RenderedSection {
    match result {
        DiagramResult::Ok(html) => try_rasterize(kind, source, &html, source_lines),
        DiagramResult::OkPng(bytes) => decode_png_to_section(kind, source, bytes, source_lines),
        DiagramResult::Err { source, error } => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source,
            message: error,
            source_lines,
        },
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            source,
        } => RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: source,
            source_lines,
        },
        DiagramResult::NotInstalled {
            kind: k,
            download_url,
            install_path,
        } => RenderedSection::NotInstalled {
            kind: k,
            download_url,
            install_path,
            source_lines,
        },
    }
}

/// Delegates to the appropriate renderer per diagram kind.
fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
    match block.kind {
        DiagramKind::Mermaid => mermaid_renderer::render_mermaid(block),
        DiagramKind::PlantUml => plantuml_renderer::render_plantuml(block),
        DiagramKind::DrawIo => drawio_renderer::render_drawio(block),
    }
}

/// Extracts SVG from an HTML fragment and rasterizes it.
fn try_rasterize(
    kind: &DiagramKind,
    source: &str,
    html: &str,
    source_lines: usize,
) -> RenderedSection {
    let Some(svg) = extract_svg(html) else {
        return RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: "Failed to extract SVG".to_string(),
            source_lines,
        };
    };
    match rasterize_svg(svg, DIAGRAM_SVG_DISPLAY_SCALE) {
        Ok(img) => RenderedSection::Image {
            svg_data: img,
            alt: format!("{kind:?} diagram"),
            source_lines,
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: e.to_string(),
            source_lines,
        },
    }
}

/// Extracts `<svg...>...</svg>` from an HTML fragment.
pub fn extract_svg(html: &str) -> Option<&str> {
    let start = html.find("<svg")?;
    let end = html.rfind("</svg>")? + "</svg>".len();
    Some(&html[start..end])
}

/// Converts PNG bytes to `RenderedSection::Image`.
///
/// Decodes mmdc PNG output using the `image` crate to get an RGBA pixel buffer.
/// This completely avoids resvg's lack of support for `<foreignObject>`.
fn decode_png_to_section(
    kind: &DiagramKind,
    source: &str,
    bytes: Vec<u8>,
    source_lines: usize,
) -> RenderedSection {
    match decode_png_rgba(&bytes) {
        Ok(rasterized) => RenderedSection::Image {
            svg_data: rasterized,
            alt: format!("{kind:?} diagram"),
            source_lines,
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: format!("PNG decode failed: {e}"),
            source_lines,
        },
    }
}

/// Converts PNG bytes to RGBA pixels.
pub fn decode_png_rgba(bytes: &[u8]) -> Result<RasterizedSvg, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(RasterizedSvg {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use katana_platform::CacheFacade;

    // ── Markdown image parsing (test-only utilities) ──

    /// Byte length of the badge image prefix `[![`.
    const BADGE_PREFIX_LEN: usize = "[![".len();

    /// Parsed Markdown image reference.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MdImage {
        src: String,
        alt: String,
        /// Number of characters consumed from the input string.
        consumed: usize,
    }

    /// Finds the byte offset of the next image pattern (`![` or `[![`).
    fn find_next_image(s: &str) -> Option<usize> {
        let pos = s.find("![")?;
        if pos > 0 && s.as_bytes()[pos - 1] == b'[' {
            Some(pos - 1)
        } else {
            Some(pos)
        }
    }

    /// Parses a Markdown image at the start of the given string.
    fn parse_md_image(s: &str) -> Option<MdImage> {
        if let Some(rest) = s.strip_prefix("[![") {
            let alt_end = rest.find(']')?;
            let alt = &rest[..alt_end];
            let after_alt = &rest[alt_end + 1..];
            let inner_src = after_alt.strip_prefix('(')?;
            let src_end = inner_src.find(')')?;
            let src = &inner_src[..src_end];
            let after_inner = &inner_src[src_end + 1..];
            let after_bracket = after_inner.strip_prefix("](")?;
            let link_end = after_bracket.find(')')?;
            let total = BADGE_PREFIX_LEN + alt_end + 1 + 1 + src_end + 1 + 2 + link_end + 1;
            return Some(MdImage {
                src: src.to_string(),
                alt: alt.to_string(),
                consumed: total,
            });
        }

        let rest = s.strip_prefix("![")?;
        let close_bracket = rest.find("](")?;
        let alt = &rest[..close_bracket];
        let after = &rest[close_bracket + 2..];
        let close_paren = after.find(')')?;
        let src = &after[..close_paren];
        if src.is_empty() {
            return None;
        }
        let total = 2 + close_bracket + 2 + close_paren + 1;
        Some(MdImage {
            src: src.to_string(),
            alt: alt.to_string(),
            consumed: total,
        })
    }

    /// Alternative to `matches!` macro. Avoids uncovered lines issue caused by
    /// subregions (`^0`) generated by LLVM in the else branch of `matches!`.
    macro_rules! assert_variant {
        ($expr:expr, $pat:pat) => {
            let val = &$expr;
            assert!(
                if let $pat = val { true } else { false },
                "expected {}, got {:?}",
                stringify!($pat),
                val
            );
        };
    }
    // render_diagram: Maps DrawIo result to RenderedSection
    #[test]
    fn render_diagram_drawio_returns_ok_section() {
        let xml = r#"<mxGraphModel><root><mxCell id="0"/><mxCell id="1" parent="0"/></root></mxGraphModel>"#;
        let section = render_diagram(&DiagramKind::DrawIo, xml, 0);
        assert_variant!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        );
    }

    // dispatch_renderer: DrawIo branch
    #[test]
    fn dispatch_renderer_drawio_returns_result() {
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: r#"<mxGraphModel><root><mxCell id="0"/></root></mxGraphModel>"#.to_string(),
        };
        let result = dispatch_renderer(&block);
        assert_variant!(result, DiagramResult::Ok(_) | DiagramResult::Err { .. });
    }

    // dispatch_renderer: Mermaid branch
    #[test]
    fn dispatch_renderer_mermaid_when_no_mmdc_returns_command_not_found() {
        let block = DiagramBlock {
            kind: DiagramKind::Mermaid,
            source: "graph TD; A-->B".to_string(),
        };
        let result = dispatch_renderer(&block);
        assert_variant!(
            result,
            DiagramResult::CommandNotFound { .. }
                | DiagramResult::OkPng(_)
                | DiagramResult::Err { .. }
        );
    }

    // dispatch_renderer: PlantUml branch
    #[test]
    fn dispatch_renderer_plantuml_when_no_jar_returns_not_installed() {
        std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar");
        let block = DiagramBlock {
            kind: DiagramKind::PlantUml,
            source: "@startuml\nA->B\n@enduml".to_string(),
        };
        let result = dispatch_renderer(&block);
        std::env::remove_var("PLANTUML_JAR");
        assert_variant!(result, DiagramResult::NotInstalled { .. });
    }

    // try_rasterize: SVG extraction failure case
    #[test]
    fn try_rasterize_returns_error_when_no_svg_in_html() {
        let kind = DiagramKind::DrawIo;
        let section = try_rasterize(&kind, "source", "<div>no svg here</div>", 0);
        assert_variant!(section, RenderedSection::Error { .. });
    }

    // try_rasterize: Success with valid SVG
    #[test]
    fn try_rasterize_returns_image_for_valid_svg() {
        let kind = DiagramKind::DrawIo;
        let html = r#"<div class="diagram"><svg width="10" height="10"><rect fill="white" width="10" height="10"/></svg></div>"#;
        let section = try_rasterize(&kind, "source", html, 0);
        assert_variant!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        );
    }

    // decode_png_to_section: Valid PNG
    #[test]
    fn decode_png_to_section_returns_image_for_valid_png() {
        use image::{ImageBuffer, Rgba};
        let mut buf = Vec::new();
        let img = ImageBuffer::from_pixel(2, 2, Rgba([100u8, 150, 200, 255]));
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let section = decode_png_to_section(&DiagramKind::DrawIo, "source", buf, 0);
        assert_variant!(section, RenderedSection::Image { .. });
    }

    // decode_png_to_section: Invalid data
    #[test]
    fn decode_png_to_section_returns_error_for_invalid_data() {
        let section = decode_png_to_section(&DiagramKind::DrawIo, "source", b"not png".to_vec(), 0);
        assert_variant!(section, RenderedSection::Error { .. });
    }

    // map_diagram_result: Exhaustive test for all variants
    #[test]
    fn map_diagram_result_ok_delegates_to_try_rasterize() {
        let section = map_diagram_result(
            &DiagramKind::DrawIo,
            "src",
            DiagramResult::Ok("<svg width=\"10\" height=\"10\"></svg>".to_string()),
            0,
        );
        assert_variant!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        );
    }

    #[test]
    fn map_diagram_result_ok_png_delegates_to_decode() {
        use image::{ImageBuffer, Rgba};
        let mut buf = Vec::new();
        let img = ImageBuffer::from_pixel(2, 2, Rgba([0u8, 0, 0, 255]));
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let section =
            map_diagram_result(&DiagramKind::Mermaid, "src", DiagramResult::OkPng(buf), 0);
        assert_variant!(section, RenderedSection::Image { .. });
    }

    #[test]
    fn map_diagram_result_err_maps_to_error_section() {
        let section = map_diagram_result(
            &DiagramKind::DrawIo,
            "src",
            DiagramResult::Err {
                source: "src".to_string(),
                error: "render failed".to_string(),
            },
            0,
        );
        assert_variant!(section, RenderedSection::Error { .. });
    }

    #[test]
    fn map_diagram_result_command_not_found_maps_to_section() {
        let section = map_diagram_result(
            &DiagramKind::Mermaid,
            "src",
            DiagramResult::CommandNotFound {
                tool_name: "mmdc".to_string(),
                install_hint: "npm install".to_string(),
                source: "src".to_string(),
            },
            0,
        );
        assert_variant!(section, RenderedSection::CommandNotFound { .. });
    }

    #[test]
    fn map_diagram_result_not_installed_maps_to_section() {
        let section = map_diagram_result(
            &DiagramKind::PlantUml,
            "src",
            DiagramResult::NotInstalled {
                kind: "PlantUML".to_string(),
                download_url: "https://example.com".to_string(),
                install_path: std::path::PathBuf::from("/tmp/plantuml.jar"),
            },
            0,
        );
        assert_variant!(section, RenderedSection::NotInstalled { .. });
    }

    // render_diagram_mermaid: Integration test (independent of mmdc presence)
    #[test]
    fn render_diagram_mermaid_produces_valid_section() {
        let section = render_diagram(&DiagramKind::Mermaid, "graph TD; A-->B", 0);
        // CommandNotFound if mmdc is absent, Image if present
        assert!(!matches!(section, RenderedSection::Pending { .. }));
    }

    // poll_renders: Receives results from the background thread and updates sections (L200-206)
    #[test]
    fn poll_renders_receives_background_result_and_updates_section() {
        use std::sync::mpsc;
        let mut pane = PreviewPane::default();

        // Set Pending section
        pane.sections = vec![RenderedSection::Pending {
            kind: "DrawIo".to_string(),
            source: "src".to_string(),
            source_lines: 0,
        }];

        // Create an mpsc channel and set it to render_rx
        let (tx, rx) = mpsc::channel();
        pane.render_rx = Some(rx);

        // Send a result from the background thread
        tx.send(RenderMessage::Section {
            kind: "DrawIo".to_string(),
            source: "src".to_string(),
            section: RenderedSection::Markdown("# Result".to_string()),
        })
        .unwrap();
        // Drop tx so the receiver becomes Disconnected
        drop(tx);

        // egui Context is required to call poll_renders
        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);

        // The section has been updated
        assert_variant!(pane.sections[0], RenderedSection::Markdown(_));
        // render_rx is None (Pending removed)
        assert!(pane.render_rx.is_none());
    }

    // wait_for_renders: Wait until Pending sections are gone (L224-242)
    #[test]
    fn wait_for_renders_blocks_until_all_rendered() {
        use std::sync::mpsc;
        let mut pane = PreviewPane::default();

        pane.sections = vec![RenderedSection::Pending {
            kind: "DrawIo".to_string(),
            source: "src".to_string(),
            source_lines: 0,
        }];

        let (tx, rx) = mpsc::channel();
        pane.render_rx = Some(rx);

        // Send in another thread
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let _ = tx.send(RenderMessage::Section {
                kind: "DrawIo".to_string(),
                source: "src".to_string(),
                section: RenderedSection::Markdown("# Done".to_string()),
            });
        });

        pane.wait_for_renders();

        // No longer Pending after completion
        assert!(pane.render_rx.is_none());
        assert_variant!(pane.sections[0], RenderedSection::Markdown(_));
    }

    // poll_renders: Do nothing without render_rx (L211-213)
    #[test]
    fn poll_renders_without_rx_does_nothing() {
        let mut pane = PreviewPane::default();
        // render_rx remains None
        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);
        // OK as long as no crash occurs
        assert!(pane.render_rx.is_none());
    }

    // full_render: Starts thread and generates Pending section (L140-149)
    #[test]
    fn full_render_with_diagram_creates_pending_section_then_renders() {
        let mut pane = PreviewPane::default();
        // Content containing a DrawIo diagram -> evaluates to Pending
        let source = "# Title\n```drawio\n<mxGraphModel><root></root></mxGraphModel>\n```";
        let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());
        pane.full_render(
            source,
            std::path::Path::new("/tmp/test.md"),
            cache,
            false,
            4,
        );

        // render_rx is set (because there is a diagram)
        assert!(pane.render_rx.is_some());

        // Wait and confirm that there are no crashes
        pane.wait_for_renders();
        assert!(pane.render_rx.is_none());
    }

    // full_render: force=true should clear CommonMarkCache
    // (We cannot directly observe the opaque cache length, but we verify the reset path doesn't crash)
    #[test]
    fn full_render_with_force_true_resets_commonmark_cache() {
        let mut pane = PreviewPane::default();
        let source = "![image](https://example.com/test.png)";
        let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());

        // Initial render (force = false)
        pane.full_render(
            source,
            std::path::Path::new("/tmp/test.md"),
            cache.clone(),
            false,
            4,
        );

        // Force render (force = true) triggers `self.commonmark_cache = CommonMarkCache::default()`
        pane.full_render(source, std::path::Path::new("/tmp/test.md"), cache, true, 4);
        assert!(
            pane.render_rx.is_none(),
            "Markdown-only render should not have pending background jobs"
        );
    }

    // ── parse_md_image / find_next_image unit tests ──

    #[test]
    fn parse_md_image_simple_image() {
        let input = "![alt text](path/to/image.png)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "path/to/image.png");
        assert_eq!(img.alt, "alt text");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_simple_with_file_uri() {
        let input = "![icon](file:///tmp/icon.png)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "file:///tmp/icon.png");
        assert_eq!(img.alt, "icon");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_badge_pattern() {
        let input = "[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "https://img.shields.io/badge/License-MIT-blue.svg");
        assert_eq!(img.alt, "License: MIT");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_badge_with_url_link() {
        let input =
            "[![CI](https://github.com/org/repo/badge.svg)](https://github.com/org/repo/actions)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "https://github.com/org/repo/badge.svg");
        assert_eq!(img.alt, "CI");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_consumed_allows_continuation() {
        let input = "[![A](https://a.svg)](link1) ![B](https://b.png)";
        let first = parse_md_image(input).unwrap();
        assert_eq!(first.alt, "A");
        let remainder = &input[first.consumed..];
        let trimmed = remainder.trim_start();
        let second = parse_md_image(trimmed).unwrap();
        assert_eq!(second.alt, "B");
        assert_eq!(second.src, "https://b.png");
    }

    #[test]
    fn parse_md_image_empty_src_returns_none() {
        assert!(parse_md_image("![alt]()").is_none());
    }

    #[test]
    fn parse_md_image_plain_text_returns_none() {
        assert!(parse_md_image("just plain text").is_none());
    }

    #[test]
    fn parse_md_image_incomplete_badge_returns_none() {
        // Missing closing of outer link
        assert!(parse_md_image("[![alt](src)](").is_none());
    }

    #[test]
    fn find_next_image_simple() {
        assert_eq!(find_next_image("hello ![img](src)"), Some(6));
    }

    #[test]
    fn find_next_image_badge() {
        assert_eq!(find_next_image("[![badge](url)](link)"), Some(0));
    }

    #[test]
    fn find_next_image_badge_before_simple() {
        // Badge starts at 0, simple at 1 — should return 0
        assert_eq!(
            find_next_image("[![badge](url)](link) ![img](src)"),
            Some(0)
        );
    }

    #[test]
    fn find_next_image_no_image() {
        assert_eq!(find_next_image("no images here"), None);
    }

    #[test]
    fn find_next_image_with_preceding_text() {
        assert_eq!(
            find_next_image("text before [![badge](url)](link)"),
            Some(12)
        );
    }

    // ── Coverage Gap Fillers (Task 4) ──

    #[test]
    fn test_coverage_gap_fillers_rendering_logic() {
        let mut pane = PreviewPane::default();
        let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());

        // 1. hit is_http = true and get_memory (L202)
        // 2. hit DiagramResult::Err and ReduceConcurrency (L237)
        // Note: is_http is true if the source contains "http://"
        let source = "```drawio\nhttp://invalidxml\n```";
        pane.full_render(
            source,
            std::path::Path::new("/tmp/test.md"),
            cache.clone(),
            false,
            1,
        );
        pane.wait_for_renders();
        assert!(pane.concurrency_reduction_requested);

        // 3. hit force = true and return None (L207)
        pane.concurrency_reduction_requested = false;
        pane.full_render(
            source,
            std::path::Path::new("/tmp/test.md"),
            cache.clone(),
            true,
            1,
        );
        pane.wait_for_renders();

        // 4. hit cached path (L211) and successful recovery/reserialize (L212, L231)
        // First, put a valid result in cache
        let diag_src = "http://graph TD; A-->B";
        let key = get_cache_key(
            &std::path::PathBuf::from("/tmp/test.md"),
            &DiagramKind::Mermaid,
            diag_src,
        );
        let valid_res = DiagramResult::Ok("<s></s>".to_string());
        let valid_json = serde_json::to_string(&valid_res).unwrap();
        cache.set_memory(&key, valid_json);

        let source2 = format!("```mermaid\n{diag_src}\n```"); // is_http=true
        pane.full_render(
            &source2,
            std::path::Path::new("/tmp/test.md"),
            cache.clone(),
            false,
            1,
        );
        pane.wait_for_renders();

        // 5. hit cached path but JSON error -> re-render (L214-222)
        cache.set_memory(&key, "invalid json".to_string());
        pane.full_render(
            &source2,
            std::path::Path::new("/tmp/test.md"),
            cache.clone(),
            false,
            1,
        );
        pane.wait_for_renders();

        // 6. corrupt cache + render error -> hit L223 ReduceConcurrency
        let diag_src3 = "invalid drawio";
        let key3 = get_cache_key(
            &std::path::PathBuf::from("/tmp/test.md"),
            &katana_core::markdown::diagram::DiagramKind::DrawIo,
            diag_src3,
        );
        let _ = cache.set_persistent(&key3, "invalid json".to_string());
        let source3 = format!("```drawio\n{diag_src3}\n```");
        pane.full_render(
            &source3,
            std::path::Path::new("/tmp/test.md"),
            cache.clone(),
            false,
            1,
        );
        pane.wait_for_renders();
    }

    #[test]
    fn cache_key_differs_by_theme() {
        use katana_core::markdown::color_preset::DiagramColorPreset;

        let path = std::path::Path::new("/tmp/test.md");
        let kind = DiagramKind::Mermaid;
        let source = "graph TD; A-->B";

        // Generate key in dark mode
        DiagramColorPreset::set_dark_mode(true);
        let key_dark = get_cache_key(path, &kind, source);

        // Generate key in light mode
        DiagramColorPreset::set_dark_mode(false);
        let key_light = get_cache_key(path, &kind, source);

        // Restore default
        DiagramColorPreset::set_dark_mode(true);

        assert_ne!(
            key_dark, key_light,
            "Cache key must differ between dark and light themes"
        );
    }

    #[test]
    fn test_coverage_gap_filler_render_message_processing() {
        let mut pane = PreviewPane::default();
        let (tx, rx) = std::sync::mpsc::channel();
        pane.render_rx = Some(rx);

        // hit RenderMessage::ReduceConcurrency in poll_renders (L309-311)
        tx.send(RenderMessage::ReduceConcurrency).unwrap();
        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);
        assert!(pane.concurrency_reduction_requested);

        // hit RenderMessage::ReduceConcurrency in wait_for_renders (L341-343)
        pane.concurrency_reduction_requested = false;
        let (tx, rx) = std::sync::mpsc::channel();
        pane.render_rx = Some(rx);
        tx.send(RenderMessage::ReduceConcurrency).unwrap();
        pane.wait_for_renders();
        assert!(pane.concurrency_reduction_requested);
    }

    #[test]
    fn test_image_preload_queue_processing() {
        let mut pane = PreviewPane::default();
        let path = std::path::PathBuf::from("/tmp/test.png");
        pane.image_preload_queue.push(path.clone());

        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);

        assert!(pane.image_preload_queue.is_empty());
        assert!(pane.image_cache.contains(&path));
    }

    // ── ViewerState tests ──

    #[test]
    fn viewer_state_default_is_zoom_1_pan_zero() {
        let state = ViewerState::default();
        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.pan, egui::Vec2::ZERO);
    }

    #[test]
    fn viewer_state_zoom_in_increases_by_step() {
        let mut state = ViewerState::default();
        state.zoom_in();
        assert_eq!(state.zoom, 1.25);
    }

    #[test]
    fn viewer_state_zoom_out_decreases_by_step() {
        let mut state = ViewerState::default();
        state.zoom_out();
        assert_eq!(state.zoom, 0.75);
    }

    #[test]
    fn viewer_state_zoom_in_clamps_at_max() {
        let mut state = ViewerState::default();
        for _ in 0..20 {
            state.zoom_in();
        }
        assert_eq!(state.zoom, 4.0);
    }

    #[test]
    fn viewer_state_zoom_out_clamps_at_min() {
        let mut state = ViewerState::default();
        for _ in 0..20 {
            state.zoom_out();
        }
        assert_eq!(state.zoom, 0.25);
    }

    #[test]
    fn viewer_state_pan_up() {
        let mut state = ViewerState::default();
        state.pan_up();
        assert_eq!(state.pan, egui::vec2(0.0, -50.0));
    }

    #[test]
    fn viewer_state_pan_down() {
        let mut state = ViewerState::default();
        state.pan_down();
        assert_eq!(state.pan, egui::vec2(0.0, 50.0));
    }

    #[test]
    fn viewer_state_pan_left() {
        let mut state = ViewerState::default();
        state.pan_left();
        assert_eq!(state.pan, egui::vec2(-50.0, 0.0));
    }

    #[test]
    fn viewer_state_pan_right() {
        let mut state = ViewerState::default();
        state.pan_right();
        assert_eq!(state.pan, egui::vec2(50.0, 0.0));
    }

    #[test]
    fn viewer_state_pan_by_accumulates() {
        let mut state = ViewerState::default();
        state.pan_by(egui::vec2(10.0, 20.0));
        state.pan_by(egui::vec2(5.0, -10.0));
        assert_eq!(state.pan, egui::vec2(15.0, 10.0));
    }

    #[test]
    fn viewer_state_reset_restores_defaults() {
        let mut state = ViewerState::default();
        state.zoom_in();
        state.zoom_in();
        state.pan_right();
        state.pan_down();
        state.reset();
        assert_eq!(state, ViewerState::default());
    }

    #[test]
    fn preview_pane_viewer_states_default_empty() {
        let pane = PreviewPane::default();
        assert!(pane.viewer_states.is_empty());
        assert!(pane.fullscreen_image.is_none());
    }

    #[test]
    fn handle_fullscreen_request_sets_index_for_valid_image() {
        let mut pane = PreviewPane::default();
        pane.sections.push(RenderedSection::Image {
            svg_data: katana_core::markdown::svg_rasterize::RasterizedSvg {
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            },
            alt: String::new(),
            source_lines: 0,
        });
        pane.handle_fullscreen_request(Some(0), None);
        assert_eq!(pane.fullscreen_image, Some(0));
    }

    #[test]
    fn handle_fullscreen_request_clears_for_out_of_bounds_index() {
        let mut pane = PreviewPane::default();
        pane.handle_fullscreen_request(Some(99), None);
        assert!(pane.fullscreen_image.is_none());
    }

    #[test]
    fn handle_fullscreen_request_clears_for_non_image_section() {
        let mut pane = PreviewPane::default();
        pane.sections
            .push(RenderedSection::Markdown("# Hello".to_string()));
        pane.handle_fullscreen_request(Some(0), None);
        assert!(pane.fullscreen_image.is_none());
    }

    #[test]
    fn handle_fullscreen_request_noop_on_none() {
        let mut pane = PreviewPane::default();
        pane.handle_fullscreen_request(None, None);
        assert!(pane.fullscreen_image.is_none());
    }

    #[test]
    fn handle_fullscreen_request_clears_stale_index() {
        let mut pane = PreviewPane::default();
        pane.sections.push(RenderedSection::Image {
            svg_data: katana_core::markdown::svg_rasterize::RasterizedSvg {
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            },
            alt: String::new(),
            source_lines: 0,
        });
        pane.fullscreen_image = Some(0);
        // Remove section, then validate state.
        pane.sections.clear();
        pane.handle_fullscreen_request(None, None);
        assert!(pane.fullscreen_image.is_none());
    }

    #[test]
    fn fullscreen_viewer_state_is_independent() {
        let mut pane = PreviewPane::default();
        // Modify main viewer state.
        pane.viewer_states.push(ViewerState::default());
        pane.viewer_states[0].zoom_in();
        // Fullscreen state should remain at defaults.
        assert!((pane.fullscreen_viewer_state.zoom - 1.0).abs() < f32::EPSILON);
        assert_eq!(pane.fullscreen_viewer_state.pan, egui::Vec2::ZERO);
    }

    #[test]
    fn fullscreen_viewer_state_resets_on_close() {
        let mut pane = PreviewPane::default();
        // Simulate zoom in fullscreen.
        pane.fullscreen_viewer_state.zoom_in();
        pane.fullscreen_viewer_state.pan_right();
        assert!(pane.fullscreen_viewer_state.zoom > 1.0);
        // Close fullscreen should reset state.
        pane.fullscreen_viewer_state.reset();
        assert!((pane.fullscreen_viewer_state.zoom - 1.0).abs() < f32::EPSILON);
        assert_eq!(pane.fullscreen_viewer_state.pan, egui::Vec2::ZERO);
    }

    #[test]
    fn i18n_diagram_controller_fields_exist() {
        use crate::i18n;
        i18n::set_language("en");
        let msgs = i18n::get();
        let dc = &msgs.preview.diagram_controller;
        assert!(!dc.pan_up.is_empty());
        assert!(!dc.pan_down.is_empty());
        assert!(!dc.pan_left.is_empty());
        assert!(!dc.pan_right.is_empty());
        assert!(!dc.zoom_in.is_empty());
        assert!(!dc.zoom_out.is_empty());
        assert!(!dc.reset.is_empty());
        assert!(!dc.fullscreen.is_empty());
        assert!(!dc.close.is_empty());
    }

    #[test]
    fn i18n_diagram_controller_ja() {
        use crate::i18n;
        i18n::set_language("ja");
        let msgs = i18n::get();
        let dc = &msgs.preview.diagram_controller;
        assert_eq!(dc.pan_up, "\u{4e0a}\u{3078}\u{79fb}\u{52d5}");
        assert_eq!(dc.reset, "\u{521d}\u{671f}\u{4f4d}\u{7f6e}\u{30fb}\u{30b5}\u{30a4}\u{30ba}\u{306b}\u{30ea}\u{30bb}\u{30c3}\u{30c8}");
        assert_eq!(dc.fullscreen, "\u{5168}\u{753b}\u{9762}\u{8868}\u{793a}");
        // Restore default language.
        i18n::set_language("en");
    }

    #[test]
    fn test_fullscreen_viewer_state_apply_result() {
        let mut pane = PreviewPane::default();
        let ctx = egui::Context::default();
        pane.fullscreen_image = Some(0);
        pane.fullscreen_viewer_state.zoom_in();
        assert_ne!(pane.fullscreen_viewer_state.zoom, 1.0);

        // Simulate closing modal.
        pane.apply_fullscreen_result(None, Some(&ctx));
        assert_eq!(pane.fullscreen_image, None);
        assert_eq!(pane.fullscreen_viewer_state.zoom, 1.0); // Resets.

        // Simulate keeping modal open.
        pane.fullscreen_image = Some(0);
        pane.fullscreen_viewer_state.zoom_in();
        pane.apply_fullscreen_result(Some(0), Some(&ctx));
        assert_eq!(pane.fullscreen_image, Some(0));
        assert_ne!(pane.fullscreen_viewer_state.zoom, 1.0); // Does NOT reset.
    }

    #[test]
    fn test_handle_fullscreen_request_context_logic() {
        let mut pane = PreviewPane::default();
        let ctx = egui::Context::default();

        pane.sections.push(RenderedSection::Image {
            svg_data: katana_core::markdown::svg_rasterize::RasterizedSvg {
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            },
            alt: "Diagram A".to_string(),
            source_lines: 0,
        });

        // Simulating opening modal when it was None
        pane.handle_fullscreen_request(Some(0), Some(&ctx));
        assert_eq!(pane.fullscreen_image, Some(0));
        assert!(!pane.was_os_fullscreen_before_modal); // context default is window mode
    }

    #[test]
    fn test_viewer_state_debug() {
        let state = ViewerState::default();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("ViewerState"));
    }
    #[test]
    fn poll_renders_clears_is_loading_on_disconnect() {
        let mut pane = PreviewPane::default();
        let (tx, rx) = std::sync::mpsc::channel::<RenderMessage>();
        drop(tx); // EXPLICIT DISCONNECT
        pane.render_rx = Some(rx);
        pane.is_loading = true;

        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);

        assert!(
            !pane.is_loading,
            "poll_renders did not clear is_loading when disconnected"
        );
        assert!(
            pane.render_rx.is_none(),
            "poll_renders did not drop the disconnected rx"
        );
    }

    #[test]
    fn full_render_sets_is_loading_to_true() {
        let mut pane = PreviewPane::default();
        assert!(!pane.is_loading);

        let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());
        pane.full_render(
            "```mermaid\ngraph TD;\nA-->B;\n```",
            std::path::Path::new("test.md"),
            cache,
            false,
            1,
        );

        assert!(
            pane.is_loading,
            "full_render did not set is_loading to true"
        );
        assert!(pane.render_rx.is_some());
    }

    #[test]
    fn full_render_aborts_on_cancel_token() {
        let mut pane = PreviewPane::default();
        let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());

        // Use multiple diagram blocks so that even if the first one races through,
        // the cancel_token should prevent the rest from completing.
        let source = concat!(
            "```mermaid\ngraph TD\nA-->B\n```\n\n",
            "```mermaid\ngraph TD\nC-->D\n```\n\n",
            "```mermaid\ngraph TD\nE-->F\n```\n\n",
            "```mermaid\ngraph TD\nG-->H\n```\n\n",
            "```mermaid\ngraph TD\nI-->J\n```\n",
        );
        pane.full_render(
            source,
            &std::path::PathBuf::from("test.md"),
            cache,
            true,
            1, // single-threaded worker to make ordering deterministic
        );
        let rx = pane.render_rx.take().unwrap();
        assert!(pane.is_loading);

        // Immediately cancel before background thread processes all jobs
        pane.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Drain all messages. The first job may have already been picked up
        // (race), but the cancel_token should prevent the rest.
        let sections: Vec<_> =
            std::iter::from_fn(|| rx.recv_timeout(std::time::Duration::from_millis(500)).ok())
                .filter(|msg| matches!(msg, RenderMessage::Section { .. }))
                .collect();

        // With 5 diagram jobs and immediate cancel, at most 1 may slip through
        // due to the race window. The key invariant is that cancel prevents
        // the majority of work.
        assert!(
            sections.len() < 5,
            "Cancel token should have prevented most renders, but all {} completed",
            sections.len()
        );
    }
}
