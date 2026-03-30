use crate::preview_pane::*;
use crate::preview_pane::{DownloadRequest, RenderedSection, ViewerState};
use eframe::egui::{self};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::color_preset::DiagramColorPreset;

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
                let preset = if ui.visuals().dark_mode {
                    DiagramColorPreset::dark()
                } else {
                    DiagramColorPreset::light()
                };
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
                    super::math::render_math(ui, tex, is_inline);
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
                    &[("kind", kind.as_str()), ("message", message.as_str())],
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
                        &[("kind", kind.as_str())],
                    ))
                    .weak(),
                );
            });
            (None, vec![])
        }
    }
}

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