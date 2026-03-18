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

/// Helper to configure new tab vs same tab behavior. We currently default to new_tab.
pub(crate) fn open_tab(ctx: &egui::Context, url: &str) {
    ctx.open_url(egui::OpenUrl::new_tab(url));
}

/// Renders a single section.
/// Returns `Some(DownloadRequest)` if the download button is pressed.
pub(crate) fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
    md_file_path: &std::path::Path,
) -> Option<DownloadRequest> {
    match section {
        RenderedSection::Markdown(md) => {
            let preset = if ui.visuals().dark_mode {
                &DiagramColorPreset::DARK
            } else {
                &DiagramColorPreset::LIGHT
            };
            let text_color = ui.visuals().override_text_color;
            let md_path_owned = md_file_path.to_path_buf();
            CommonMarkViewer::new()
                .syntax_theme_dark(preset.syntax_theme_dark)
                .syntax_theme_light(preset.syntax_theme_light)
                .render_html_fn(Some(&move |ui: &mut egui::Ui, html: &str| {
                    render_html_block(ui, html, text_color, &md_path_owned);
                }))
                .show(ui, cache, md);
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
///
/// No automatic separators are inserted between sections.
/// Horizontal rules (`---`) are rendered by CommonMarkViewer as part of the markdown content.
pub(crate) fn render_sections(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    sections: &[RenderedSection],
    md_file_path: &std::path::Path,
) -> Option<DownloadRequest> {
    let mut request: Option<DownloadRequest> = None;
    for (i, section) in sections.iter().enumerate() {
        ui.push_id(format!("section_{i}"), |ui| {
            if let Some(req) = show_section(ui, cache, section, i, md_file_path) {
                request = Some(req);
            }
        });
    }
    if sections.is_empty() {
        ui.label(egui::RichText::new(crate::i18n::t("no_preview")).weak());
    }
    request
}

/// Renders an HTML block using our HtmlParser + HtmlRenderer pipeline.
///
/// This function is called from the `render_html_fn` callback registered with
/// `egui_commonmark::CommonMarkViewer`. `pulldown-cmark` identifies HTML blocks
/// per CommonMark spec and passes them to this handler. This means we no longer
/// need custom regex-based HTML block extraction in `split_into_sections`.
/// Vertical spacing (in points) added before and after each HTML block.
const HTML_BLOCK_VERTICAL_SPACING: f32 = 4.0;

fn render_html_block(
    ui: &mut egui::Ui,
    html: &str,
    text_color: Option<egui::Color32>,
    md_file_path: &std::path::Path,
) {
    // Add breathing room above the HTML block
    ui.add_space(HTML_BLOCK_VERTICAL_SPACING);

    // Resolve relative image paths in the HTML to absolute file:// URIs
    let resolved_html = katana_core::preview::resolve_html_image_paths(html, md_file_path);
    let base_dir = md_file_path.parent().unwrap_or(std::path::Path::new("."));
    let parser = katana_core::html::HtmlParser::new(base_dir);
    let nodes = parser.parse(&resolved_html);
    let mut renderer = crate::html_renderer::HtmlRenderer::new(ui, base_dir);
    if let Some(c) = text_color {
        renderer = renderer.text_color(c);
    }
    if let Some(action) = renderer.render(&nodes) {
        match action {
            katana_core::html::LinkAction::OpenInBrowser(url) => {
                open_tab(ui.ctx(), &url);
            }
            katana_core::html::LinkAction::NavigateCurrentTab(path) => {
                open_tab(ui.ctx(), &path.to_string_lossy());
            }
        }
    }

    // Add breathing room below the HTML block
    ui.add_space(HTML_BLOCK_VERTICAL_SPACING);
}
