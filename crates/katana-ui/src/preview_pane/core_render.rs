use egui_commonmark::CommonMarkCache;
use katana_core::markdown::diagram::{DiagramBlock, DiagramResult};
use katana_core::preview::{
    flatten_list_code_blocks, resolve_image_paths, split_into_sections, PreviewSection,
};


use super::pane::*;
use super::renderer::*;
use super::types::*;

impl PreviewPane {
    pub fn update_markdown_sections(&mut self, source: &str, md_file_path: &std::path::Path) {
        self.md_file_path = md_file_path.to_path_buf();
        self.outline_items = katana_core::markdown::outline::extract_outline(source);
        let (resolved, extracted_paths) = resolve_image_paths(source, md_file_path);

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

    pub fn abort_renders(&mut self) {
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.is_loading = false;
        self.render_rx = None;
    }

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
                    match serde_json::from_str::<DiagramResult>(&json) {
                        Ok(res) => res,
                        Err(_) => {
                            let res = dispatch_renderer(&DiagramBlock {
                                kind: job.kind.clone(),
                                source: job.src.clone(),
                            });
                            if matches!(res, DiagramResult::Err { .. }) {
                                let _ = tx.send(RenderMessage::ReduceConcurrency);
                            }
                            res
                        }
                    }
                } else {
                    let res = dispatch_renderer(&DiagramBlock {
                        kind: job.kind.clone(),
                        source: job.src.clone(),
                    });

                    match serde_json::to_string(&res) {
                        Ok(json) => {
                            if is_http {
                                job.cache.set_memory(&cache_key, json);
                            } else {
                                let _ = job.cache.set_persistent(&cache_key, json);
                            }
                        }
                        Err(_) => {}
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
                    break; // WHY: Receiver was dropped.
                }
                if let Some(ctx) = &repaint_ctx {
                    ctx.request_repaint();
                }
            });
        }
    }
}