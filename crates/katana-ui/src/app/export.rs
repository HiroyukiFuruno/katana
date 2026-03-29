#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::hash_str;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub(crate) trait ExportOps {
    fn handle_export_document(&mut self, ctx: &egui::Context, fmt: crate::app_state::ExportFormat);
    fn export_filename(&self, doc_path: &std::path::Path, ext: &str) -> String;
    fn export_as_html(&mut self, _ctx: &egui::Context, source: &str, doc_path: &std::path::Path);
    fn export_with_tool(
        &mut self,
        _ctx: &egui::Context,
        source: &str,
        ext: &str,
        doc_path: &std::path::Path,
    );
    fn perform_tool_export(
        &mut self,
        source: &str,
        ext: &str,
        output_path: std::path::PathBuf,
        doc_path: &std::path::Path,
    );
    fn poll_export(&mut self, ctx: &egui::Context);
}

impl ExportOps for KatanaApp {
    fn handle_export_document(&mut self, ctx: &egui::Context, fmt: crate::app_state::ExportFormat) {
        tracing::info!("Export document requested: {:?}", fmt);

        let Some(doc) = self.state.active_document() else {
            return;
        };
        let buffer = doc.buffer.clone();
        let doc_path = doc.path.clone();

        match fmt {
            crate::app_state::ExportFormat::Html => self.export_as_html(ctx, &buffer, &doc_path),
            crate::app_state::ExportFormat::Pdf => {
                self.export_with_tool(ctx, &buffer, "pdf", &doc_path)
            }
            crate::app_state::ExportFormat::Png => {
                self.export_with_tool(ctx, &buffer, "png", &doc_path)
            }
            crate::app_state::ExportFormat::Jpg => {
                self.export_with_tool(ctx, &buffer, "jpg", &doc_path)
            }
        }
    }
    fn export_filename(&self, doc_path: &std::path::Path, ext: &str) -> String {
        let (prefix, relative) = if let Some(ws) = &self.state.workspace.data {
            // Build prefix from workspace root path initials
            let initials: String = ws
                .root
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(s) => s.to_string_lossy().chars().next(),
                    _ => None, // skip RootDir, Prefix, CurDir, ParentDir
                })
                .collect();

            let rel = doc_path.strip_prefix(&ws.root).unwrap_or(doc_path);
            (initials, rel.to_path_buf())
        } else {
            (String::new(), doc_path.to_path_buf())
        };

        let stem = relative
            .with_extension("")
            .to_string_lossy()
            .replace([std::path::MAIN_SEPARATOR, '/'], "_");

        if stem.is_empty() {
            format!("export.{}", ext)
        } else if prefix.is_empty() {
            format!("{}.{}", stem, ext)
        } else {
            format!("{}_{}.{}", prefix, stem, ext)
        }
    }
    fn export_as_html(&mut self, _ctx: &egui::Context, source: &str, doc_path: &std::path::Path) {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::current().clone();
        let source = source.to_string();
        let base_dir = doc_path.parent().map(|p| p.to_path_buf());
        let filename = self.export_filename(doc_path, "html");

        let (tx, rx) = std::sync::mpsc::channel();

        let fname = filename.clone();
        std::thread::spawn(move || {
            let result = export_html_to_tmp(&source, &fname, &preset, base_dir.as_deref());
            let _ = tx.send(result);
        });

        self.export_tasks.push(ExportTask {
            filename,
            rx,
            open_on_complete: true,
        });
    }
    fn export_with_tool(
        &mut self,
        _ctx: &egui::Context,
        source: &str,
        ext: &str,
        doc_path: &std::path::Path,
    ) {
        let (is_available, tool_name) = match ext {
            "pdf" => (true, "headless_chrome"),
            _ => (true, "headless_chrome"),
        };

        if !is_available {
            let msg = crate::i18n::tf(
                &crate::i18n::get().export.tool_missing,
                &[("tool", tool_name), ("format", &ext.to_uppercase())],
            );
            self.state.layout.status_message = Some((msg, crate::app_state::StatusType::Error));
            return;
        }

        let default_name = self.export_filename(doc_path, ext);
        let path = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter(ext, &[ext])
            .save_file();

        if let Some(output_path) = path {
            self.perform_tool_export(source, ext, output_path, doc_path);
        }
    }
    fn perform_tool_export(
        &mut self,
        source: &str,
        ext: &str,
        output_path: std::path::PathBuf,
        doc_path: &std::path::Path,
    ) {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::current().clone();
        let source = source.to_string();
        let ext = ext.to_string();
        let base_dir = doc_path.parent().map(|p| p.to_path_buf());
        let filename = output_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "export".to_string());

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let renderer = katana_core::markdown::KatanaRenderer;
            let html = match katana_core::markdown::HtmlExporter::export(
                &source,
                &renderer,
                &preset,
                base_dir.as_deref(),
            ) {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };

            let result = match ext.as_str() {
                "pdf" => katana_core::markdown::PdfExporter::export(&html, &output_path),
                _ => katana_core::markdown::ImageExporter::export(&html, &output_path),
            };

            let _ = tx.send(
                result
                    .map(|()| output_path.clone())
                    .map_err(|e| e.to_string()),
            );
        });

        self.export_tasks.push(ExportTask {
            filename,
            rx,
            open_on_complete: false,
        });
    }
    fn poll_export(&mut self, ctx: &egui::Context) {
        const EXPORT_POLL_INTERVAL_MS: u64 = 50;
        let mut has_pending = false;

        // Collect completed tasks first (borrow checker workaround).
        let mut completed: Vec<(usize, Result<std::path::PathBuf, String>)> = Vec::new();
        for (i, task) in self.export_tasks.iter().enumerate() {
            match task.rx.try_recv() {
                Ok(result) => {
                    completed.push((i, result));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    has_pending = true;
                }
                Err(_) => {
                    completed.push((i, Err("Export thread disconnected".to_string())));
                }
            }
        }

        // Process completed tasks in reverse order to maintain correct indices.
        for (i, result) in completed.into_iter().rev() {
            let task = self.export_tasks.remove(i);
            match result {
                Ok(output_path) => {
                    let ext = output_path
                        .extension()
                        .map(|e| e.to_string_lossy().to_uppercase())
                        .unwrap_or_default();
                    let msg = crate::i18n::tf(
                        &crate::i18n::get().export.success,
                        &[
                            ("format", &ext),
                            ("path", &output_path.display().to_string()),
                        ],
                    );
                    self.state.layout.status_message =
                        Some((msg, crate::app_state::StatusType::Success));
                    if task.open_on_complete {
                        if let Err(e) = open::that(&output_path) {
                            tracing::warn!("Failed to open {}: {e}", output_path.display());
                        }
                    }
                    tracing::info!(
                        "Export complete: {} → {}",
                        task.filename,
                        output_path.display()
                    );
                }
                Err(error) => {
                    let msg = crate::i18n::tf(
                        &crate::i18n::get().export.failed,
                        &[("format", &task.filename), ("error", &error)],
                    );
                    self.state.layout.status_message =
                        Some((msg, crate::app_state::StatusType::Error));
                }
            }
        }

        if has_pending {
            ctx.request_repaint_after(std::time::Duration::from_millis(EXPORT_POLL_INTERVAL_MS));
        }
    }
}
