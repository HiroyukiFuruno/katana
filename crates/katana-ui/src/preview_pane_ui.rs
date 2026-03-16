//! Pure egui UI rendering functions for the preview pane.
//!
//! This module contains code that depends entirely on the egui UI context (`egui::Ui`).
//! - Button click events (`button().clicked()`)
//! - Texture loading (`ui.ctx().load_texture()`)
//! - UI component rendering
//!
//! Since these cannot be executed without an egui frame context,
//! they are excluded from coverage measurement using `--ignore-filename-regex`.

use eframe::egui::{self, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_core::markdown::svg_rasterize::RasterizedSvg;

use crate::preview_pane::{DownloadRequest, RenderedSection};

/// Text color for the tool not installed warning (orange).
const WARNING_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 165, 0);

/// Renders a single section.
/// Returns `Some(DownloadRequest)` if the download button is pressed.
pub(crate) fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
) -> Option<DownloadRequest> {
    match section {
        RenderedSection::Markdown(md) => {
            let preset = DiagramColorPreset::current();
            // Boost the text color for dark-theme readability.
            let prev_override = ui.visuals().override_text_color;
            if let Some((r, g, b)) = DiagramColorPreset::parse_hex_rgb(preset.preview_text) {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(r, g, b));
            }
            CommonMarkViewer::new()
                .syntax_theme_dark(preset.syntax_theme_dark)
                .syntax_theme_light(preset.syntax_theme_light)
                .show(ui, cache, md);
            ui.visuals_mut().override_text_color = prev_override;
            None
        }
        RenderedSection::Image { svg_data, alt } => {
            show_rasterized(ui, svg_data, alt, id);
            None
        }
        RenderedSection::Error {
            kind,
            _source: _,
            message,
        } => {
            ui.label(
                egui::RichText::new(crate::i18n::tf(
                    "render_error",
                    &[("kind", kind), ("message", message)],
                ))
                .color(egui::Color32::YELLOW)
                .small(),
            );
            None
        }
        RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: _,
        } => {
            let msg = crate::i18n::t("missing_dependency")
                .replace("{tool_name}", tool_name)
                .replace("{install_hint}", install_hint);
            ui.label(
                egui::RichText::new(msg)
                    .color(egui::Color32::YELLOW)
                    .small(),
            );
            None
        }
        RenderedSection::NotInstalled {
            kind,
            download_url,
            install_path,
        } => show_not_installed(ui, kind, download_url, install_path),
        RenderedSection::Pending { kind } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new(crate::i18n::tf("rendering", &[("kind", kind)])).weak(),
                );
            });
            None
        }
    }
}

/// Download button UI for uninstalled tools.
pub(crate) fn show_not_installed(
    ui: &mut egui::Ui,
    kind: &str,
    download_url: &str,
    install_path: &std::path::Path,
) -> Option<DownloadRequest> {
    let mut request = None;
    ui.group(|ui| {
        ui.label(
            egui::RichText::new(crate::i18n::tf("tool_not_installed", &[("tool", kind)]))
                .color(WARNING_TEXT_COLOR),
        );
        ui.label(
            egui::RichText::new(crate::i18n::tf(
                "tool_install_path",
                &[("path", &install_path.display().to_string())],
            ))
            .small()
            .weak(),
        );
        if ui
            .button(crate::i18n::tf("tool_download", &[("tool", kind)]))
            .clicked()
        {
            request = Some(DownloadRequest {
                url: download_url.to_string(),
                dest: install_path.to_path_buf(),
            });
        }
    });
    request
}

/// Displays rasterized SVG as an egui texture.
pub(crate) fn show_rasterized(ui: &mut egui::Ui, img: &RasterizedSvg, _alt: &str, id: usize) {
    let color_img = egui::ColorImage::from_rgba_unmultiplied(
        [img.width as usize, img.height as usize],
        &img.rgba,
    );
    let texture = ui.ctx().load_texture(
        format!("diagram_{id}"),
        color_img,
        egui::TextureOptions::LINEAR,
    );
    let max_w = ui.available_width();
    let scale = (max_w / img.width as f32).min(1.0);
    let size = Vec2::new(img.width as f32 * scale, img.height as f32 * scale);
    ui.add(egui::Image::new((texture.id(), size)));
}

/// Renders the section list sequentially and returns a download request if any.
pub(crate) fn render_sections(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    sections: &[RenderedSection],
) -> Option<DownloadRequest> {
    let mut request: Option<DownloadRequest> = None;
    for (i, section) in sections.iter().enumerate() {
        ui.push_id(format!("section_{i}"), |ui| {
            if let Some(req) = show_section(ui, cache, section, i) {
                request = Some(req);
            }
            ui.separator();
        });
    }
    if sections.is_empty() {
        ui.label(egui::RichText::new(crate::i18n::t("no_preview")).weak());
    }
    request
}
