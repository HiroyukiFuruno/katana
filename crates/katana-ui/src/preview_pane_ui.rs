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

use crate::icon::Icon;
use crate::preview_pane::{DownloadRequest, RenderedSection, ViewerState};

/// Text color for the tool not installed warning (orange).
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
    mut heading_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    heading_offset: usize,
    global_task_list_idx: &mut usize,
    active_editor_line: Option<usize>,
    hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
    global_line_offset: usize,
) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
    let mut actions = Vec::new();
    match section {
        RenderedSection::Markdown(md) => {
            with_preview_text_style(ui, |ui| {
                // Negative margin offset is no longer needed since we removed forced newlines from egui_commonmark
                let preset = if ui.visuals().dark_mode {
                    DiagramColorPreset::dark()
                } else {
                    DiagramColorPreset::light()
                };
                // Retrieve the preview-specific text colour from the cached ThemeColors.
                // egui::Visuals::override_text_color was global to all UI,
                // so we use preview.text to achieve independent colour assignment.
                let theme_colors = ui.ctx().data(|d| {
                    d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                        "katana_theme_colors",
                    ))
                });
                let text_color = theme_colors
                    .as_ref()
                    .map(|tc| crate::theme_bridge::rgb_to_color32(tc.preview.text));
                let hover_bg_color = theme_colors.as_ref().map(|tc| {
                    crate::theme_bridge::rgba_to_color32(tc.preview.hover_line_background)
                });
                let border_color = theme_colors
                    .as_ref()
                    .map(|tc| crate::theme_bridge::rgb_to_color32(tc.preview.border));
                let selection_color = theme_colors
                    .as_ref()
                    .map(|tc| crate::theme_bridge::rgb_to_color32(tc.preview.selection));

                let md_path_owned = md_file_path.to_path_buf();

                let binding = move |ui: &mut egui::Ui, html: &str| {
                    render_html_block(ui, html, text_color, &md_path_owned);
                };

                let math_binding = |ui: &mut egui::Ui, tex: &str, is_inline: bool| {
                    render_math(ui, tex, is_inline);
                };

                let mut viewer = CommonMarkViewer::new()
                    .syntax_theme_dark(preset.syntax_theme_dark)
                    .syntax_theme_light(preset.syntax_theme_light)
                    .heading_offset(heading_offset)
                    .render_html_fn(Some(&binding))
                    .render_math_fn(Some(&math_binding))
                    .hover_bg_color(hover_bg_color);

                if let Some(idx) = scroll_to_heading_index {
                    viewer = viewer.scroll_to_heading_index(idx);
                }

                let previous_anchor_count = heading_anchors.as_ref().map(|a| a.len()).unwrap_or(0);
                if let Some(anchors) = heading_anchors.as_mut() {
                    viewer = viewer.heading_anchors(anchors);
                }

                if let Some(global_line) = active_editor_line {
                    let lines_in_md = md.chars().filter(|c| *c == '\n').count();
                    if global_line >= global_line_offset
                        && global_line <= global_line_offset + lines_in_md
                    {
                        let local_line = global_line - global_line_offset;
                        let mut current_line = 0;
                        let mut start_byte = None;
                        let mut end_byte = None;
                        for (i, c) in md.char_indices() {
                            if current_line == local_line && start_byte.is_none() {
                                start_byte = Some(i);
                            }
                            if current_line == local_line + 1 {
                                end_byte = Some(i);
                                break;
                            }
                            if c == '\n' {
                                current_line += 1;
                            }
                        }
                        if current_line == local_line && start_byte.is_none() {
                            start_byte = Some(0);
                        }
                        if let Some(s) = start_byte {
                            viewer = viewer.active_char_range(s..end_byte.unwrap_or(md.len()));
                        }
                    }
                }

                let mut local_hovered_spans = Vec::new();
                if hovered_lines.is_some() {
                    viewer = viewer.hovered_spans(&mut local_hovered_spans);
                }

                let (_, newly_captured) = ui
                    .scope(|ui| {
                        if let Some(color) = text_color {
                            ui.visuals_mut().override_text_color = Some(color);
                        }
                        if let Some(border) = border_color {
                            ui.visuals_mut().widgets.noninteractive.bg_stroke.color = border;
                        }
                        const TABLE_STRIPE_ALPHA: f32 = 0.1;
                        if let Some(sel) = selection_color {
                            ui.visuals_mut().selection.bg_fill = sel;
                            ui.visuals_mut().faint_bg_color =
                                sel.gamma_multiply(TABLE_STRIPE_ALPHA);
                        }
                        viewer.show_with_events(ui, cache, md)
                    })
                    .inner;

                if let Some(anchors) = heading_anchors {
                    for anchor in &mut anchors[previous_anchor_count..] {
                        let local_span = &anchor.0;
                        let start_line = global_line_offset
                            + md[..local_span.start]
                                .chars()
                                .filter(|c| *c == '\n')
                                .count();
                        let end_line = global_line_offset
                            + md[..local_span.end].chars().filter(|c| *c == '\n').count();
                        anchor.0 = start_line..end_line;
                    }
                }

                if let Some(hovered) = hovered_lines {
                    for local_span in local_hovered_spans {
                        let start_line = global_line_offset
                            + md[..local_span.start]
                                .chars()
                                .filter(|c| *c == '\n')
                                .count();
                        let end_line = global_line_offset
                            + md[..local_span.end].chars().filter(|c| *c == '\n').count();
                        hovered.push(start_line..end_line);
                    }
                }
                let spans = egui_commonmark::extract_task_list_spans(md);
                for action in newly_captured {
                    if let Some(local_idx) = spans.iter().position(|s| s == &action.span) {
                        actions.push((*global_task_list_idx + local_idx, action.new_state));
                    }
                }
                *global_task_list_idx += spans.len();
            });
            (None, actions)
        }
        RenderedSection::Image { svg_data, alt, .. } => {
            show_rasterized(ui, svg_data, alt, id, None, None);
            (None, vec![])
        }
        RenderedSection::LocalImage { path, alt, .. } => {
            show_local_image(ui, path, alt, id, None, None);
            (None, vec![])
        }
        RenderedSection::Error { kind, message, .. } => {
            ui.label(
                egui::RichText::new(crate::i18n::tf(
                    &crate::i18n::get().error.render_error,
                    &vec![("kind", kind.as_str()), ("message", message.as_str())],
                ))
                .color(
                    ui.ctx()
                        .data(|d| {
                            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                "katana_theme_colors",
                            ))
                        })
                        .map_or(crate::theme_bridge::WHITE, |tc| {
                            crate::theme_bridge::rgb_to_color32(tc.preview.warning_text)
                        }),
                )
                .small(),
            );
            (None, vec![])
        }
        RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            ..
        } => {
            let msg = crate::i18n::get()
                .error
                .missing_dependency
                .clone()
                .replace("{tool_name}", tool_name)
                .replace("{install_hint}", install_hint);
            ui.label(
                egui::RichText::new(msg)
                    .color(
                        ui.ctx()
                            .data(|d| {
                                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                    "katana_theme_colors",
                                ))
                            })
                            .map_or(crate::theme_bridge::WHITE, |tc| {
                                crate::theme_bridge::rgb_to_color32(tc.preview.warning_text)
                            }),
                    )
                    .small(),
            );
            (None, vec![])
        }
        RenderedSection::NotInstalled {
            kind,
            download_url,
            install_path,
            ..
        } => (
            show_not_installed(ui, kind, download_url, install_path),
            vec![],
        ),
        RenderedSection::Pending { kind, .. } => {
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
            (None, vec![])
        }
    }
}

fn with_preview_text_style<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.scope(|ui| {
        let fonts_loaded = ui.ctx().data(|d| {
            d.get_temp::<bool>(egui::Id::new("katana_fonts_loaded"))
                .unwrap_or(false)
        });
        if fonts_loaded {
            set_preview_body_family(ui, egui::FontFamily::Name("MarkdownProportional".into()));
        } else {
            set_preview_body_family(ui, egui::FontFamily::Proportional);
        }

        if let Some(color) = ui.ctx().data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        }) {
            ui.visuals_mut().override_text_color =
                Some(crate::theme_bridge::rgb_to_color32(color.preview.text));
            ui.visuals_mut().selection.bg_fill =
                crate::theme_bridge::rgb_to_color32(color.preview.selection);
        }

        add_contents(ui)
    })
    .inner
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
            .color(
                ui.ctx()
                    .data(|d| {
                        d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                            "katana_theme_colors",
                        ))
                    })
                    .map_or(crate::theme_bridge::WHITE, |tc| {
                        crate::theme_bridge::rgb_to_color32(tc.preview.warning_text)
                    }),
            ),
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
/// Padding around fullscreen image.
const FULLSCREEN_PADDING: f32 = 40.0;
/// Size of the fullscreen close button.
const FULLSCREEN_CLOSE_SIZE: f32 = 32.0;
/// Margin for close button from screen edge.
const FULLSCREEN_CLOSE_MARGIN: f32 = 16.0;

/// Minimum zoom limit for trackpad pinches.
const MIN_ZOOM: f32 = 0.1;
/// Maximum zoom limit for trackpad pinches.
const MAX_ZOOM: f32 = 10.0;

pub(crate) fn show_rasterized(
    ui: &mut egui::Ui,
    img: &RasterizedSvg,
    _alt_text: &str,
    idx: usize,
    mut state: Option<&mut ViewerState>,
    fullscreen_request: Option<&mut Option<usize>>,
) {
    let max_w = ui.available_width();
    let base_scale = (max_w / img.width as f32).min(1.0);

    // Apply viewer zoom/pan if state is provided.
    let zoom = state.as_ref().map_or(1.0, |s| s.zoom);
    let pan = state.as_ref().map_or(egui::Vec2::ZERO, |s| s.pan);

    // The base display size (zoom = 1.0)
    let base_size = Vec2::new(
        img.width as f32 * base_scale,
        img.height as f32 * base_scale,
    );

    // Zoomed image size
    let zoomed_size = base_size * zoom;

    // Reserve space for the image container (invariant to zoom!).
    let (container_rect, response) =
        ui.allocate_exact_size(Vec2::new(max_w, base_size.y), egui::Sense::click_and_drag());

    if let Some(state) = state.as_mut() {
        if response.hovered() {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                state.zoom = (state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
            }
            if response.dragged() {
                state.pan += response.drag_delta();
            }
        }
    }

    let texture_handle = if let Some(state) = state.as_mut() {
        if state.texture.is_none() {
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
            state.texture = Some(ui.ctx().load_texture(
                format!("diagram_{idx}"),
                color_img,
                egui::TextureOptions::LINEAR,
            ));
        }
        state.texture.clone().unwrap()
    } else {
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
        ui.ctx().load_texture(
            format!("diagram_{idx}"),
            color_img,
            egui::TextureOptions::LINEAR,
        )
    };

    // Paint the image with pan offset, clipped to the fixed container.
    // Center the base image horizontally if it's smaller than max_w.
    let x_offset = (max_w - base_size.x).max(0.0) / 2.0;

    let image_pos = container_rect.min + egui::vec2(x_offset, 0.0) + pan;
    let image_rect = egui::Rect::from_min_size(image_pos, zoomed_size);
    ui.painter().with_clip_rect(container_rect).image(
        texture_handle.id(),
        image_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        crate::theme_bridge::WHITE,
    );

    // Draw overlay controls only when viewer_state is provided.
    if let Some(state) = state {
        // Fullscreen button (top-right).
        if crate::diagram_controller::draw_fullscreen_button(ui, container_rect) {
            if let Some(req) = fullscreen_request {
                *req = Some(idx);
            }
        }

        // Control grid (bottom-right).
        crate::diagram_controller::draw_controls(ui, state, container_rect);
    }
}

/// Renders the section list sequentially and returns a download request if any.
///
/// No automatic separators are inserted between sections.
/// Horizontal rules (`---`) are rendered by CommonMarkViewer as part of the markdown content.
/// Extended version of `render_sections` with viewer state support.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_sections(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    sections: &[RenderedSection],
    md_file_path: &std::path::Path,
    scroll_to_heading_index: Option<usize>,
    mut heading_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    mut viewer_states: Option<&mut Vec<ViewerState>>,
    mut fullscreen_request: Option<&mut Option<usize>>,
    active_editor_line: Option<usize>,
    mut hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
    let mut request: Option<DownloadRequest> = None;
    let mut actions = Vec::new();
    let mut current_heading_offset = 0;
    let mut global_task_list_idx = 0;
    let mut global_line_offset = 0;

    for (i, section) in sections.iter().enumerate() {
        ui.push_id(format!("section_{i}"), |ui| {
            let mut offset = 0;
            let lines_in_section = if let RenderedSection::Markdown(md) = section {
                offset = current_heading_offset;
                current_heading_offset += katana_core::markdown::outline::extract_outline(md).len();
                md.chars().filter(|c| *c == '\n').count()
            } else {
                match section {
                    RenderedSection::Image { source_lines, .. } => *source_lines,
                    RenderedSection::LocalImage { source_lines, .. } => *source_lines,
                    RenderedSection::Error { source_lines, .. } => *source_lines,
                    RenderedSection::NotInstalled { source_lines, .. } => *source_lines,
                    RenderedSection::Pending { source_lines, .. } => *source_lines,
                    RenderedSection::CommandNotFound { source_lines, .. } => *source_lines,
                    _ => 0,
                }
            };
            match section {
                RenderedSection::Image { svg_data, alt, .. } => {
                    // Auto-extend viewer_states Vec if needed and take a mutable reference.
                    let state = viewer_states.as_mut().map(|vs| {
                        if vs.len() <= i {
                            vs.resize_with(i + 1, ViewerState::default);
                        }
                        &mut vs[i]
                    });
                    show_rasterized(
                        ui,
                        svg_data,
                        alt,
                        i,
                        state,
                        fullscreen_request.as_deref_mut(),
                    );
                }
                RenderedSection::LocalImage { path, alt, .. } => {
                    let state = viewer_states.as_mut().map(|vs| {
                        if vs.len() <= i {
                            vs.resize_with(i + 1, ViewerState::default);
                        }
                        &mut vs[i]
                    });
                    show_local_image(ui, path, alt, i, state, fullscreen_request.as_deref_mut());
                }
                _ => {
                    let (req, mut event_actions) = show_section(
                        ui,
                        cache,
                        section,
                        i,
                        md_file_path,
                        scroll_to_heading_index,
                        heading_anchors.as_deref_mut(),
                        offset,
                        &mut global_task_list_idx,
                        active_editor_line,
                        hovered_lines.as_deref_mut(),
                        global_line_offset,
                    );
                    if let Some(r) = req {
                        request = Some(r);
                    }
                    actions.append(&mut event_actions);
                }
            }
            global_line_offset += lines_in_section;
        });
    }
    if sections.is_empty() {
        ui.label(egui::RichText::new(crate::i18n::get().preview.no_preview.clone()).weak());
    }
    (request, actions)
}

pub fn open_fullscreen_viewer(
    ui: &mut egui::Ui,
    idx: usize,
    sections: &[RenderedSection],
    opened_fullscreen_idx: &mut Option<usize>,
) {
    if let Some(RenderedSection::Image { svg_data, alt, .. }) = sections.get(idx) {
        show_fullscreen_modal(ui.ctx(), svg_data, alt, &mut ViewerState::default(), idx);
        *opened_fullscreen_idx = Some(idx);
    } else if let Some(RenderedSection::LocalImage { path, alt, .. }) = sections.get(idx) {
        show_fullscreen_local_image(ui.ctx(), path, alt, &mut ViewerState::default(), idx);
        *opened_fullscreen_idx = Some(idx);
    }
}

/// Renders the fullscreen modal overlay if `fullscreen_image` is `Some`.
/// Returns the updated fullscreen state: `None` if closed, otherwise unchanged.
pub(crate) fn render_fullscreen_if_active(
    ctx: &egui::Context,
    sections: &[RenderedSection],
    fullscreen_image: Option<usize>,
    fullscreen_state: &mut ViewerState,
) -> Option<usize> {
    let idx = fullscreen_image?;
    if let Some(RenderedSection::Image { svg_data, alt, .. }) = sections.get(idx) {
        if show_fullscreen_modal(ctx, svg_data, alt, fullscreen_state, idx) {
            Some(idx) // keep open
        } else {
            None // user closed
        }
    } else if let Some(RenderedSection::LocalImage { path, alt, .. }) = sections.get(idx) {
        if show_fullscreen_local_image(ctx, path, alt, fullscreen_state, idx) {
            Some(idx) // keep open
        } else {
            None // user closed
        }
    } else {
        None // section gone
    }
}

/// Renders a fullscreen modal overlay displaying a single image with controls.
/// Returns `true` if the modal should remain open, `false` if closed.
pub(crate) fn show_fullscreen_modal(
    ctx: &egui::Context,
    img: &RasterizedSvg,
    _alt: &str,
    viewer_state: &mut ViewerState,
    idx: usize,
) -> bool {
    let msgs = crate::i18n::get();
    let dc = &msgs.preview.diagram_controller;

    // Close on Escape.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return false;
    }

    let screen = ctx.content_rect();

    // Input blocker — consume all clicks/drags on the backdrop so nothing behind is interactive.
    let mut keep_open = true;
    egui::Area::new(egui::Id::new("fs_input_blocker"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (blocker_rect, response) =
                ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());

            if response.hovered() {
                let zoom_delta = ui.input(|i| i.zoom_delta());
                if zoom_delta != 1.0 {
                    viewer_state.zoom = (viewer_state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
                }
                if response.dragged() {
                    viewer_state.pan += response.drag_delta();
                } else {
                    viewer_state.pan += ui.input(|i| i.smooth_scroll_delta);
                }
            }

            // Fully opaque backdrop — blocks all visual content behind the modal.
            let bg_color = crate::theme_bridge::IMAGE_VIEWER_OVERLAY_COLOR;
            ui.painter().rect_filled(blocker_rect, 0.0, bg_color);

            // Fit image to screen with padding, applying viewer zoom/pan.
            let avail = Vec2::new(
                screen.width() - FULLSCREEN_PADDING * 2.0,
                screen.height() - FULLSCREEN_PADDING * 2.0,
            );
            let scale_x = avail.x / img.width as f32;
            let scale_y = avail.y / img.height as f32;
            let base_scale = scale_x.min(scale_y).min(1.0);
            let zoom = viewer_state.zoom;
            let pan = viewer_state.pan;
            let size = Vec2::new(
                img.width as f32 * base_scale * zoom,
                img.height as f32 * base_scale * zoom,
            );
            let texture_handle = if viewer_state.texture.is_none() {
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
                let th = ctx.load_texture(
                    format!("diagram_fs_{idx}"),
                    color_img,
                    egui::TextureOptions::LINEAR,
                );
                viewer_state.texture = Some(th.clone());
                th
            } else {
                viewer_state.texture.clone().unwrap()
            };

            // Center the image with pan offset.
            let img_pos = screen.center() - size / 2.0 + pan;
            let img_rect = egui::Rect::from_min_size(img_pos, size);
            ui.painter().with_clip_rect(blocker_rect).image(
                texture_handle.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                crate::theme_bridge::WHITE,
            );

            // Overlay controls (bottom-right of screen).
            crate::diagram_controller::draw_controls(ui, viewer_state, blocker_rect);

            // Close button [✕] (top-right).
            let close_btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);
            let close_btn_rect = egui::Rect::from_min_size(
                egui::pos2(
                    blocker_rect.right() - close_btn_size.x - FULLSCREEN_CLOSE_MARGIN,
                    blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
                ),
                close_btn_size,
            );
            let close_resp = ui.put(
                close_btn_rect,
                egui::Button::image(
                    Icon::CloseModal
                        .image(crate::icon::IconSize::Large)
                        .tint(crate::theme_bridge::WHITE),
                )
                .fill(
                    crate::theme_bridge::TRANSPARENT, /* Handled by theme overlay */
                )
                .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
            );
            if close_resp.on_hover_text(&dc.close).clicked() {
                keep_open = false;
            }
        });

    keep_open
}

pub(crate) fn show_local_image(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    _alt: &str,
    id: usize,
    mut viewer_state: Option<&mut ViewerState>,
    fullscreen_request: Option<&mut Option<usize>>,
) {
    let texture_handle = if let Some(state) = viewer_state.as_mut() {
        if state.texture.is_none() {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                    let rgba = dyn_img.into_rgba8();
                    let size = std::array::from_fn(|i| {
                        if i == 0 {
                            rgba.width() as usize
                        } else {
                            rgba.height() as usize
                        }
                    });
                    let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    state.texture = Some(ui.ctx().load_texture(
                        format!("local_image_{id}"),
                        color_img,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
        }
        state.texture.clone()
    } else {
        None
    };

    let (texture_handle, width, height) = match texture_handle {
        Some(t) => {
            let size = t.size();
            (t, size[0], size[1])
        }
        None => return, // Could not load image
    };

    let max_w = ui.available_width();
    let base_scale = (max_w / width as f32).min(1.0);

    let zoom = viewer_state.as_ref().map_or(1.0, |s| s.zoom);
    let pan = viewer_state.as_ref().map_or(egui::Vec2::ZERO, |s| s.pan);

    let base_size = Vec2::new(width as f32 * base_scale, height as f32 * base_scale);
    let zoomed_size = base_size * zoom;

    let (container_rect, response) =
        ui.allocate_exact_size(Vec2::new(max_w, base_size.y), egui::Sense::click_and_drag());

    if let Some(state) = viewer_state.as_mut() {
        if response.hovered() {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                state.zoom = (state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
            }
            if response.dragged() {
                state.pan += response.drag_delta();
            }
        }
    }

    let x_offset = (max_w - base_size.x).max(0.0) / 2.0;
    let image_pos = container_rect.min + egui::vec2(x_offset, 0.0) + pan;
    let image_rect = egui::Rect::from_min_size(image_pos, zoomed_size);

    ui.painter().with_clip_rect(container_rect).image(
        texture_handle.id(),
        image_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        crate::theme_bridge::WHITE,
    );

    if let Some(state) = viewer_state {
        if crate::diagram_controller::draw_fullscreen_button(ui, container_rect) {
            if let Some(req) = fullscreen_request {
                *req = Some(id);
            }
        }
        crate::diagram_controller::draw_controls(ui, state, container_rect);
    }
}

pub(crate) fn show_fullscreen_local_image(
    ctx: &egui::Context,
    path: &std::path::Path,
    _alt: &str,
    viewer_state: &mut ViewerState,
    idx: usize,
) -> bool {
    let msgs = crate::i18n::get();
    let dc = &msgs.preview.diagram_controller;

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return false;
    }

    let screen = ctx.content_rect();
    let mut keep_open = true;

    egui::Area::new(egui::Id::new("fs_input_blocker"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (blocker_rect, response) =
                ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());

            if response.hovered() {
                let zoom_delta = ui.input(|i| i.zoom_delta());
                if zoom_delta != 1.0 {
                    viewer_state.zoom = (viewer_state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
                }
                if response.dragged() {
                    viewer_state.pan += response.drag_delta();
                } else {
                    viewer_state.pan += ui.input(|i| i.smooth_scroll_delta);
                }
            }

            let bg_color = crate::theme_bridge::IMAGE_VIEWER_OVERLAY_COLOR;
            ui.painter().rect_filled(blocker_rect, 0.0, bg_color);

            let texture_handle = if viewer_state.texture.is_none() {
                if let Ok(bytes) = std::fs::read(path) {
                    if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                        let rgba = dyn_img.into_rgba8();
                        let size = std::array::from_fn(|i| {
                            if i == 0 {
                                rgba.width() as usize
                            } else {
                                rgba.height() as usize
                            }
                        });
                        let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                        viewer_state.texture = Some(ui.ctx().load_texture(
                            format!("local_image_fs_{idx}"),
                            color_img,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                }
                viewer_state.texture.clone()
            } else {
                viewer_state.texture.clone()
            };

            let (texture_handle, width, height) = match texture_handle {
                Some(t) => {
                    let size = t.size();
                    (t, size[0], size[1])
                }
                None => return,
            };

            let avail = Vec2::new(
                screen.width() - FULLSCREEN_PADDING * 2.0,
                screen.height() - FULLSCREEN_PADDING * 2.0,
            );
            let scale_x = avail.x / width as f32;
            let scale_y = avail.y / height as f32;
            let base_scale = scale_x.min(scale_y).min(1.0);

            let zoom = viewer_state.zoom;
            let pan = viewer_state.pan;
            let size = Vec2::new(
                width as f32 * base_scale * zoom,
                height as f32 * base_scale * zoom,
            );

            let img_pos = screen.center() - size / 2.0 + pan;
            let img_rect = egui::Rect::from_min_size(img_pos, size);
            ui.painter().with_clip_rect(blocker_rect).image(
                texture_handle.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                crate::theme_bridge::WHITE,
            );

            crate::diagram_controller::draw_controls(ui, viewer_state, blocker_rect);

            let close_btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);
            let close_btn_rect = egui::Rect::from_min_size(
                egui::pos2(
                    blocker_rect.right() - close_btn_size.x - FULLSCREEN_CLOSE_MARGIN,
                    blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
                ),
                close_btn_size,
            );

            let close_resp = ui.put(
                close_btn_rect,
                egui::Button::image(
                    crate::icon::Icon::CloseModal
                        .image(crate::icon::IconSize::Large)
                        .tint(crate::theme_bridge::WHITE),
                )
                .fill(
                    crate::theme_bridge::TRANSPARENT, /* Handled by theme overlay */
                )
                .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
            );
            if close_resp.on_hover_text(&dc.close).clicked() {
                keep_open = false;
            }
        });

    keep_open
}
#[derive(Clone)]
struct MathJaxCache(std::sync::Arc<egui::mutex::Mutex<std::collections::BTreeMap<String, String>>>);

impl Default for MathJaxCache {
    fn default() -> Self {
        Self(std::sync::Arc::new(egui::mutex::Mutex::new(
            Default::default(),
        )))
    }
}

/// Renders a LaTeX mathematical formula as an SVG image.
///
/// This function is registered as the `math_fn` callback in `egui_commonmark::CommonMarkViewer`.
pub fn render_math(ui: &mut egui::Ui, tex: &str, is_inline: bool) {
    /// Horizontal padding (left/right) inside the block math frame.
    const MATH_BLOCK_H_MARGIN: i8 = 8;
    /// Vertical padding (top/bottom) inside the block math frame.
    const MATH_BLOCK_V_MARGIN: i8 = 4;
    /// Corner radius for the block math frame.
    const MATH_BLOCK_CORNER_RADIUS: u8 = 4;
    /// Conversion ratio roughly equalizing 1 `ex` to pixel height in our font rendering context.
    const EX_TO_PX: f32 = 8.5;
    /// Negative top margin used to perfectly align inline math center with vertical text layout tops.
    const INLINE_MATH_MARGIN_TOP: i8 = -8;
    /// Negative top/bottom margin used to tighten the visual gap for block math spacing natively rendered.
    const BLOCK_MATH_MARGIN_VERTICAL: i8 = -10;
    let tex = tex.trim();
    if tex.is_empty() {
        return;
    }

    // MathJax paths must perfectly match the visual text colour (since math is inherently 'text')
    let text_color = ui.visuals().text_color();
    let hex_color = format!(
        "#{:02x}{:02x}{:02x}",
        text_color.r(),
        text_color.g(),
        text_color.b()
    );

    // Hash map key differentiates block/inline and the EXACT textual math colour requested,
    // to force re-rendering if the user changes the preview.text setting dynamically.
    let is_dark = ui.visuals().dark_mode;
    let cache_key = format!(
        "{}:{}:{}:{}",
        if is_dark { "dark" } else { "light" },
        hex_color,
        if is_inline { "inline" } else { "block" },
        tex
    );

    // Retrieve or create the cache in egui's temporary state storage.
    let cache = ui.ctx().memory_mut(|mem| {
        mem.data
            .get_temp_mut_or_default::<MathJaxCache>(egui::Id::new("katana_mathjax_cache"))
            .clone()
    });

    let uri = {
        let mut map = cache.0.lock();
        if let Some(cached_uri) = map.get(&cache_key) {
            cached_uri.clone()
        } else {
            // MathJax V8 initialization panics if called from multiple test threads concurrently.
            // In the actual app, egui rendering is single-threaded, but `cargo test` runs in parallel.
            static MATHJAX_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

            let svg_result = {
                let _lock = MATHJAX_LOCK.lock().unwrap();
                if is_inline {
                    mathjax_svg::convert_to_svg_inline(tex)
                } else {
                    mathjax_svg::convert_to_svg(tex)
                }
            };

            let data_uri = match svg_result {
                Ok(svg_string) => {
                    // `mathjax_svg` returns dimensions in `ex` units (e.g. width="8.6ex").
                    // `usvg` in KatanaSvgLoader does not support `.ex` parsing and will fail with a Bad px_scale_factor zero crash.
                    // We extract the `ex` value, scale it (1 ex ~ 8.5px aligns nicely with our font), and replace it with `px`.
                    let mut processed_svg = svg_string;
                    let width_re = regex::Regex::new(r#"width="([\d\.]+)ex""#).unwrap();
                    let height_re = regex::Regex::new(r#"height="([\d\.]+)ex""#).unwrap();

                    if let Some(caps) = width_re.captures(&processed_svg) {
                        if let Ok(w_ex) = caps.get(1).unwrap().as_str().parse::<f32>() {
                            let w_px = w_ex * EX_TO_PX;
                            processed_svg = width_re
                                .replace(&processed_svg, format!("width=\"{w_px}px\""))
                                .into_owned();
                        }
                    }
                    if let Some(caps) = height_re.captures(&processed_svg) {
                        if let Ok(h_ex) = caps.get(1).unwrap().as_str().parse::<f32>() {
                            let h_px = h_ex * EX_TO_PX;
                            processed_svg = height_re
                                .replace(&processed_svg, format!("height=\"{h_px}px\""))
                                .into_owned();
                        }
                    }

                    // `usvg` doesn't automatically inherit css `currentColor`.
                    // MathJax emits generic currentColor fields which we must explicitly colorize.
                    // (Already retrieved `hex_color` string earlier to form our cache key)
                    processed_svg = processed_svg.replace("currentColor", &hex_color);

                    use base64::{engine::general_purpose, Engine as _};
                    let b64 = general_purpose::STANDARD.encode(processed_svg.as_bytes());
                    format!("data:image/svg+xml;base64,{}", b64)
                }
                Err(e) => {
                    tracing::error!("MathJax rendering failed for {:?}: {}", tex, e);
                    // Return an empty string to trigger fallback
                    String::new()
                }
            };
            map.insert(cache_key.clone(), data_uri.clone());
            data_uri
        }
    };

    if uri.is_empty() {
        // Fallback display if rendering failed
        if is_inline {
            ui.label(
                egui::RichText::new(tex)
                    .monospace()
                    .color(ui.visuals().error_fg_color),
            );
        } else {
            egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin::symmetric(
                    MATH_BLOCK_H_MARGIN,
                    MATH_BLOCK_V_MARGIN,
                ))
                .corner_radius(egui::CornerRadius::same(MATH_BLOCK_CORNER_RADIUS))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(tex)
                            .monospace()
                            .color(ui.visuals().error_fg_color),
                    );
                });
        }
        return;
    }

    // Provide the original tex to AccessKit by putting a visually hidden label,
    // or by adding a tooltip (accessible name).
    let response = if is_inline {
        // To gracefully align the math graphics with the surrounding Japanese characters,
        // we use a negative top margin. Since egui horizontally aligns inline widgets
        // by their tops, this shrinks the bounding box at the top, physically shifting
        // the drawn image UP relative to the text by exactly the margin value.
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: INLINE_MATH_MARGIN_TOP,
                bottom: 0,
            })
            .show(ui, |ui| {
                ui.add(egui::Image::new(&uri).fit_to_original_size(1.0))
            })
            .inner
    } else {
        // Block math: MathJax display mode natively includes generous vertical spacing.
        // To tighten the flow without cropping the SVG, we use negative margins (-10px top/bottom).
        // This brings the surrounding text 10px closer to the integral/block equation.
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: BLOCK_MATH_MARGIN_VERTICAL,
                bottom: BLOCK_MATH_MARGIN_VERTICAL,
            })
            .show(ui, |ui| {
                ui.add(egui::Image::new(&uri).fit_to_original_size(1.0))
            })
            .inner
    };

    // To ensure UI integration tests (accesskit get_by_label) can find this math element,
    // add an invisible label right after the image if it wasn't intercepted by tooltip.
    response.on_hover_text(tex);

    // Add a tiny transparent label so `get_by_label` locates it correctly inside the dom tree.
    let mut rect = ui.cursor();
    rect.max = rect.min;
    ui.put(
        rect,
        egui::Label::new(
            egui::RichText::new(tex)
                .size(1.0)
                .color(crate::theme_bridge::TRANSPARENT),
        ),
    );
}

/// Renders an HTML block using our HtmlParser + HtmlRenderer pipeline.
///
/// This function is called from the `render_html_fn` callback registered with
/// `egui_commonmark::CommonMarkViewer`. `pulldown-cmark` identifies HTML blocks
/// per CommonMark spec and passes them to this handler. This means we no longer
/// need custom regex-based HTML block extraction in `split_into_sections`.
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
        egui::vec2(ui.max_rect().width(), ui.available_height()),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(block_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |block_ui| {
            block_ui.set_clip_rect(clip_rect);

            // Adjust margin: shift text 2px up and reduce top padding by 5px
            const HTML_BLOCK_MARGIN_TOP_ADJUST: f32 = -7.0;
            block_ui.add_space(HTML_BLOCK_MARGIN_TOP_ADJUST);

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

            // Adjust margin: reduce bottom padding by 5px (with 2px offset applied)
            const HTML_BLOCK_MARGIN_BOTTOM_ADJUST: f32 = -3.0;
            block_ui.add_space(HTML_BLOCK_MARGIN_BOTTOM_ADJUST);
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
            after_html_y.get() >= 28.0,
            "HTML badge block must advance cursor by a meaningful height, got {:.1}",
            after_html_y.get()
        );
        assert!(
            gap_from_top >= 28.0,
            "Following text must render below the badge row, got Y={gap_from_top:.1}"
        );
    }
}
