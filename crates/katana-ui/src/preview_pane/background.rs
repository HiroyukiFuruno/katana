use super::pane::RenderMessage;
use super::types::*;
use eframe::egui;

use super::pane::PreviewPane;
impl PreviewPane {
    /// Polls for background rendering completion and updates sections with received results.
    pub(crate) fn poll_renders(&mut self, ctx: &egui::Context) {
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
