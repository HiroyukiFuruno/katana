#![allow(clippy::useless_vec)]
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
    md_file_path: &std::path::Path,
    scroll_to_heading_index: Option<usize>,
    populate_heading_rects: Option<&mut Vec<egui::Rect>>,
    heading_offset: usize,
) -> Option<DownloadRequest> {
    match section {
        RenderedSection::Markdown(md) => {
            with_preview_text_style(ui, |ui| {
                if id == 0 && starts_with_markdown_heading(md) {
                    // egui_commonmark inserts a forced leading newline before the first heading.
                    // Compensate that so preview padding stays consistent with non-heading content.
                    ui.add_space(-leading_heading_offset(ui));
                }
                let preset = if ui.visuals().dark_mode {
                    DiagramColorPreset::dark()
                } else {
                    DiagramColorPreset::light()
                };
                let text_color = ui.visuals().override_text_color;
                let md_path_owned = md_file_path.to_path_buf();

                let binding = move |ui: &mut egui::Ui, html: &str| {
                    render_html_block(ui, html, text_color, &md_path_owned);
                };

                let mut viewer = CommonMarkViewer::new()
                    .syntax_theme_dark(preset.syntax_theme_dark)
                    .syntax_theme_light(preset.syntax_theme_light)
                    .heading_offset(heading_offset)
                    .render_html_fn(Some(&binding));

                if let Some(idx) = scroll_to_heading_index {
                    viewer = viewer.scroll_to_heading_index(idx);
                }
                if let Some(rects) = populate_heading_rects {
                    viewer = viewer.populate_heading_rects(rects);
                }

                viewer.show(ui, cache, md);
            });
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
                    &crate::i18n::get().error.render_error,
                    &vec![("kind", kind.as_str()), ("message", message.as_str())],
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
            let msg = crate::i18n::get()
                .error
                .missing_dependency
                .clone()
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
                    egui::RichText::new(crate::i18n::tf(
                        &crate::i18n::get().preview.rendering,
                        &vec![("kind", kind.as_str())],
                    ))
                    .weak(),
                );
            });
            None
        }
    }
}

fn starts_with_markdown_heading(markdown: &str) -> bool {
    markdown
        .trim_start_matches(char::is_whitespace)
        .starts_with('#')
}

const HEADING_SPACING_MULTIPLIER: f32 = 1.0;

fn leading_heading_offset(ui: &egui::Ui) -> f32 {
    ui.text_style_height(&egui::TextStyle::Body)
        + (ui.spacing().item_spacing.y * HEADING_SPACING_MULTIPLIER)
}

fn with_preview_text_style<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.scope(|ui| {
        if body_uses_monospace(ui) {
            set_preview_body_family(ui, egui::FontFamily::Proportional);
        }
        add_contents(ui)
    })
    .inner
}

fn body_uses_monospace(ui: &egui::Ui) -> bool {
    ui.style()
        .text_styles
        .get(&egui::TextStyle::Body)
        .is_some_and(|font_id| font_id.family == egui::FontFamily::Monospace)
}

fn set_preview_body_family(ui: &mut egui::Ui, family: egui::FontFamily) {
    let style = ui.style_mut();
    style.override_font_id = None;
    style.override_text_style = None;
    for text_style in vec![
        egui::TextStyle::Body,
        egui::TextStyle::Button,
        egui::TextStyle::Heading,
        egui::TextStyle::Small,
    ] {
        if let Some(font_id) = style.text_styles.get_mut(&text_style) {
            font_id.family = family.clone();
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
            egui::RichText::new(crate::i18n::tf(
                &crate::i18n::get().tool.not_installed,
                &vec![("tool", kind)],
            ))
            .color(WARNING_TEXT_COLOR),
        );
        let path_str = install_path.display().to_string();
        ui.label(
            egui::RichText::new(crate::i18n::tf(
                &crate::i18n::get().tool.install_path,
                &vec![("path", path_str.as_str())],
            ))
            .small()
            .weak(),
        );
        if ui
            .button(crate::i18n::tf(
                &crate::i18n::get().tool.download,
                &vec![("tool", kind)],
            ))
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
        std::array::from_fn(|i| {
            if i == 0 {
                img.width as usize
            } else {
                img.height as usize
            }
        }),
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
    scroll_to_heading_index: Option<usize>,
    mut populate_heading_rects: Option<&mut Vec<egui::Rect>>,
) -> Option<DownloadRequest> {
    let mut request: Option<DownloadRequest> = None;
    let mut current_heading_offset = 0;

    for (i, section) in sections.iter().enumerate() {
        ui.push_id(format!("section_{i}"), |ui| {
            let mut offset = 0;
            if let RenderedSection::Markdown(md) = section {
                offset = current_heading_offset;
                current_heading_offset += katana_core::markdown::outline::extract_outline(md).len();
            }
            if let Some(req) = show_section(
                ui,
                cache,
                section,
                i,
                md_file_path,
                scroll_to_heading_index,
                populate_heading_rects.as_deref_mut(),
                offset,
            ) {
                request = Some(req);
            }
        });
    }
    if sections.is_empty() {
        ui.label(egui::RichText::new(crate::i18n::get().preview.no_preview.clone()).weak());
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
const HTML_BLOCK_VERTICAL_SPACING: f32 = 1.0;

fn render_html_block(
    ui: &mut egui::Ui,
    html: &str,
    text_color: Option<egui::Color32>,
    md_file_path: &std::path::Path,
) {
    let clip_rect = ui.clip_rect();
    let ctx = ui.ctx().clone();
    let block_rect = egui::Rect::from_min_size(
        egui::pos2(ui.max_rect().left(), ui.next_widget_position().y),
        egui::vec2(ui.max_rect().width(), 0.0),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(block_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |block_ui| {
            block_ui.set_clip_rect(clip_rect);
            block_ui.add_space(HTML_BLOCK_VERTICAL_SPACING);

            let resolved_html = katana_core::preview::resolve_html_image_paths(html, md_file_path);
            let base_dir = md_file_path.parent().unwrap_or(std::path::Path::new("."));
            let parser = katana_core::html::HtmlParser::new(base_dir);
            let nodes = parser.parse(&resolved_html);
            let mut renderer = crate::html_renderer::HtmlRenderer::new(block_ui, base_dir);
            if let Some(c) = text_color {
                renderer = renderer.text_color(c);
            }
            if let Some(action) = renderer.render(&nodes) {
                match action {
                    katana_core::html::LinkAction::OpenInBrowser(url) => {
                        open_tab(&ctx, &url);
                    }
                    katana_core::html::LinkAction::NavigateCurrentTab(path) => {
                        open_tab(&ctx, &path.to_string_lossy());
                    }
                }
            }

            block_ui.add_space(HTML_BLOCK_VERTICAL_SPACING);
        },
    );
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::path::Path;
    use std::rc::Rc;

    use eframe::egui;
    use egui_kittest::{
        kittest::{NodeT, Queryable},
        Harness,
    };

    use super::render_html_block;

    #[test]
    fn html_block_badge_advances_cursor_before_following_text() {
        let after_html_y = Rc::new(Cell::new(0.0_f32));
        let after_html_y_capture = Rc::clone(&after_html_y);
        let html = concat!(
            "<p align=\"center\">",
            "<a href=\"https://github.com/sponsors/HiroyukiFuruno\">",
            "<img src=\"https://img.shields.io/badge/Sponsor-%E2%9D%A4%EF%B8%8F-ea4aaa?style=for-the-badge&logo=github-sponsors\" alt=\"Sponsor\">",
            "</a>",
            "</p>",
        );

        let mut harness = Harness::builder()
            .with_size(egui::vec2(800.0, 240.0))
            .build_ui(move |ui| {
                render_html_block(ui, html, None, Path::new("/tmp/README.md"));
                after_html_y_capture.set(ui.next_widget_position().y);
                ui.label("Support helps cover:");
            });
        harness.step();
        harness.run();

        let label = harness.get_by_label("Support helps cover:");
        let bounds = label
            .accesskit_node()
            .raw_bounds()
            .expect("following label should have bounds");
        let gap_from_top = bounds.y0 as f32;

        assert!(
            after_html_y.get() >= 40.0,
            "HTML badge block must advance cursor by a meaningful height, got {:.1}",
            after_html_y.get()
        );
        assert!(
            gap_from_top >= 40.0,
            "Following text must render below the badge row, got Y={gap_from_top:.1}"
        );
    }
}
