use crate::app_state::{AppAction, ScrollSource};
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell::SCROLL_SYNC_DEAD_ZONE;
use crate::shell_ui::{
    invisible_label, LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG, PREVIEW_CONTENT_PADDING,
};
use eframe::egui;

pub(crate) fn preview_panel_id(path: Option<&std::path::Path>, base: &'static str) -> egui::Id {
    match path {
        Some(path) => egui::Id::new((base, path)),
        None => egui::Id::new(base),
    }
}

pub(crate) fn invalidate_preview_image_cache(ctx: &egui::Context, action: &AppAction) {
    if matches!(action, AppAction::RefreshDiagrams) {
        crate::icon::IconRegistry::install(ctx);
    }
}

pub(crate) struct PreviewContent<'a> {
    pub preview: &'a mut PreviewPane,
    pub document: Option<&'a katana_core::document::Document>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub toc_visible: bool,
    pub show_toc: bool,
    pub action: &'a mut AppAction,
    pub scroll_sync: bool,
    pub scroll_state: &'a mut (f32, ScrollSource, f32),
}

impl<'a> PreviewContent<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        preview: &'a mut PreviewPane,
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut crate::app_state::ScrollState,
        toc_visible: bool,
        show_toc: bool,
        action: &'a mut AppAction,
        scroll_sync: bool,
        scroll_state: &'a mut (f32, ScrollSource, f32),
    ) -> Self {
        Self {
            preview,
            document,
            scroll,
            toc_visible,
            show_toc,
            action,
            scroll_sync,
            scroll_state,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let preview = self.preview;
        let document = self.document;
        let scroll = self.scroll;
        let toc_visible = self.toc_visible;
        let show_toc = self.show_toc;
        let action = self.action;
        let scroll_sync = self.scroll_sync;
        let scroll_state = self.scroll_state;
        let mut download_req = None;
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        let outer_rect = ui.available_rect_before_wrap();
        ui.allocate_rect(outer_rect, egui::Sense::hover());

        let (fraction, source, prev_max_scroll) = scroll_state;
        let mut scroll_area = egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .auto_shrink(std::array::from_fn(|_| false));

        let mut target_scroll_offset = *fraction * (*prev_max_scroll).max(1.0);
        let consuming_editor = scroll_sync && *source == ScrollSource::Editor;
        if consuming_editor {
            if let Some(doc) = document {
                let _buffer = &doc.buffer;
                let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                let editor_y = *fraction * scroll.editor_max.max(1.0);

                let mut points = Vec::new();
                points.push((0.0, 0.0));
                for (span, rect) in &preview.heading_anchors {
                    let e_y = span.start as f32 * row_height;
                    let p_y = (rect.min.y - preview.content_top_y).max(0.0);
                    points.push((e_y, p_y));
                }
                points.push((scroll.editor_max.max(1.0), (*prev_max_scroll).max(1.0)));

                let mut mapped_y = 0.0;
                for i in 0..points.len() - 1 {
                    let (e_y1, p_y1) = points[i];
                    let (e_y2, p_y2) = points[i + 1];
                    if editor_y >= e_y1 && editor_y <= e_y2 {
                        if e_y2 > e_y1 {
                            let t = (editor_y - e_y1) / (e_y2 - e_y1);
                            mapped_y = p_y1 + t * (p_y2 - p_y1);
                        } else {
                            mapped_y = p_y1;
                        }
                        break;
                    }
                }
                if editor_y > points.last().unwrap().0 {
                    mapped_y = points.last().unwrap().1;
                }
                target_scroll_offset = mapped_y;
            }

            scroll_area = scroll_area.vertical_scroll_offset(target_scroll_offset);
        }

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(outer_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        content_ui.set_clip_rect(outer_rect);

        let output = scroll_area.show(&mut content_ui, |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(
                    PREVIEW_CONTENT_PADDING,
                    PREVIEW_CONTENT_PADDING,
                ))
                .show(ui, |ui| {
                    let content_width = ui.available_width();
                    let child_rect = egui::Rect::from_min_size(
                        ui.next_widget_position(),
                        egui::vec2(content_width, 0.0),
                    );
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .max_rect(child_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                        |ui| {
                            const PREVIEW_PANE_TOP_BOTTOM_PADDING: f32 = 4.0; // 0.25rem padding
                            ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                            let mut hovered_lines = Vec::new();
                            let (req, actions) = preview.show_content(
                                ui,
                                scroll.active_editor_line,
                                Some(&mut hovered_lines),
                            );
                            if scroll_sync && *source != ScrollSource::Preview {
                                scroll.hovered_preview_lines = hovered_lines.clone();
                            }

                            if ui.rect_contains_pointer(ui.min_rect())
                                && ui.input(|i| i.pointer.primary_clicked())
                            {
                                if let Some(hovered) = hovered_lines.first() {
                                    scroll.scroll_to_line = Some(hovered.start);
                                }
                            }
                            download_req = req;
                            if let Some((global_index, new_state)) = actions.into_iter().next() {
                                *action = AppAction::ToggleTaskList {
                                    global_index,
                                    new_state,
                                };
                            }
                            ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                        },
                    );
                });
        });

        if scroll_sync {
            let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
            *prev_max_scroll = max_scroll;

            if consuming_editor {
                *source = ScrollSource::Neither;
                if max_scroll > 0.0 {
                    // If mapped via piecewise, we should inversely map the actual offset back to fraction to stay stable.
                    // However, holding the fraction is generally safer.
                    // We leave fraction alone when consuming editor input.
                }
            } else {
                if max_scroll > 0.0 {
                    let preview_y = output.state.offset.y;
                    let mut editor_target_y = preview_y;

                    if let Some(doc) = document {
                        let _buffer = &doc.buffer;
                        let row_height = ui.text_style_height(&egui::TextStyle::Monospace);

                        let mut points = Vec::new();
                        points.push((0.0, 0.0));
                        for (span, rect) in &preview.heading_anchors {
                            let e_y = span.start as f32 * row_height;
                            let p_y = (rect.min.y - preview.content_top_y).max(0.0);
                            points.push((e_y, p_y));
                        }
                        points.push((scroll.editor_max.max(1.0), max_scroll));

                        let mut mapped_y = 0.0;
                        for i in 0..points.len() - 1 {
                            let (e_y1, p_y1) = points[i];
                            let (e_y2, p_y2) = points[i + 1];
                            if preview_y >= p_y1 && preview_y <= p_y2 {
                                if p_y2 > p_y1 {
                                    let t = (preview_y - p_y1) / (p_y2 - p_y1);
                                    mapped_y = e_y1 + t * (e_y2 - e_y1);
                                } else {
                                    mapped_y = e_y1;
                                }
                                break;
                            }
                        }
                        if preview_y > points.last().unwrap().1 {
                            mapped_y = points.last().unwrap().0;
                        }
                        editor_target_y = mapped_y;
                    }

                    let current_fraction =
                        (editor_target_y / scroll.editor_max.max(1.0)).clamp(0.0, 1.0);
                    let diff = (current_fraction - *fraction).abs();
                    if diff > SCROLL_SYNC_DEAD_ZONE {
                        *fraction = current_fraction;
                        *source = ScrollSource::Preview;
                    }
                }
            }
        }

        PreviewHeader::new(document.is_some(), toc_visible, show_toc, action).show(ui);

        download_req
    }
}

pub(crate) struct PreviewHeader<'a> {
    pub has_doc: bool,
    pub toc_visible: bool,
    pub show_toc: bool,
    pub action: &'a mut AppAction,
}

impl<'a> PreviewHeader<'a> {
    pub fn new(
        has_doc: bool,
        toc_visible: bool,
        show_toc: bool,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            has_doc,
            toc_visible,
            show_toc,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let has_doc = self.has_doc;
        let action = self.action;
        let button_size = egui::vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y);
        let margin = f32::from(PREVIEW_CONTENT_PADDING);
        let spacing = ui.spacing().item_spacing.x;
        let mut button_count = 2.0; // Refresh + Export
        if self.toc_visible {
            button_count += 1.0;
        }
        let total_width = (button_size.x * button_count) + (spacing * (button_count - 1.0));

        let button_rect = egui::Rect::from_min_size(
            egui::pos2(
                ui.max_rect().right() - margin - total_width,
                ui.max_rect().top() + margin,
            ),
            egui::vec2(total_width, button_size.y),
        );
        let mut overlay_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(button_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );

        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::from_gray(LIGHT_MODE_ICON_BG)
        };

        if overlay_ui
            .add_enabled(
                has_doc,
                egui::Button::image_and_text(
                    crate::Icon::Refresh.ui_image(ui, crate::icon::IconSize::Medium),
                    invisible_label("🔄"),
                )
                .min_size(button_size)
                .fill(icon_bg),
            )
            .on_hover_text(crate::i18n::get().preview.refresh_diagrams.clone())
            .clicked()
        {
            *action = AppAction::RefreshDiagrams;
        }

        let export_img = egui::Image::new(crate::icon::Icon::Export.uri())
            .tint(overlay_ui.visuals().text_color());
        overlay_ui.scope(|ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;
            ui.menu_image_button(export_img, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                if ui
                    .button(crate::i18n::get().menu.export_html.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Html);
                    ui.close();
                }
                if ui
                    .button(crate::i18n::get().menu.export_pdf.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Pdf);
                    ui.close();
                }
                if ui
                    .button(crate::i18n::get().menu.export_png.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Png);
                    ui.close();
                }
                if ui
                    .button(crate::i18n::get().menu.export_jpg.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Jpg);
                    ui.close();
                }
            });
        });

        if self.toc_visible {
            let toc_bg = if self.show_toc {
                if ui.visuals().dark_mode {
                    ui.visuals().selection.bg_fill
                } else {
                    crate::theme_bridge::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
                }
            } else {
                icon_bg
            };
            if overlay_ui
                .add_enabled(
                    has_doc,
                    egui::Button::image_and_text(
                        crate::Icon::Toc.ui_image(ui, crate::icon::IconSize::Medium),
                        invisible_label("toggle_toc"),
                    )
                    .min_size(button_size)
                    .fill(toc_bg),
                )
                .on_hover_text(crate::i18n::get().action.toggle_toc.clone())
                .clicked()
            {
                *action = AppAction::ToggleToc;
            }
        }
    }
}
