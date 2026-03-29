use egui_commonmark::CommonMarkCache;
use katana_core::markdown::diagram::{DiagramBlock, DiagramResult};
use katana_core::preview::{
    flatten_list_code_blocks, resolve_image_paths, split_into_sections, PreviewSection,
};

// ─────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────

use super::pane::*;
use super::renderer::*;
use super::types::*;

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

                let cached_result: Option<String> = if !job.force {
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
}
