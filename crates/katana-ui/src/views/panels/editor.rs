use crate::app_state::{AppAction, ScrollSource};
use crate::shell::{EDITOR_INITIAL_VISIBLE_ROWS, SCROLL_SYNC_DEAD_ZONE};
use eframe::egui;

pub(crate) struct EditorContent<'a> {
    pub document: Option<&'a katana_core::document::Document>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub action: &'a mut AppAction,
    pub sync_scroll: bool,
}

impl<'a> EditorContent<'a> {
    pub fn new(
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut crate::app_state::ScrollState,
        action: &'a mut AppAction,
        sync_scroll: bool,
    ) -> Self {
        Self {
            document,
            scroll,
            action,
            sync_scroll,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let action = self.action;
        let sync_scroll = self.sync_scroll;
        let scroll = self.scroll;
        if let Some(doc) = self.document {
            let mut buffer = doc.buffer.clone();

            let (
                code_bg,
                code_text,
                code_selection,
                current_line_bg,
                hover_line_bg,
                ln_text,
                ln_active_text,
            ) = ui.ctx().data(|d| {
                if let Some(tc) = d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                )) {
                    (
                        crate::theme_bridge::rgb_to_color32(tc.code.background),
                        crate::theme_bridge::rgb_to_color32(tc.code.text),
                        Some(crate::theme_bridge::rgb_to_color32(tc.code.selection)),
                        Some(crate::theme_bridge::rgba_to_color32(
                            tc.code.current_line_background,
                        )),
                        Some(crate::theme_bridge::rgba_to_color32(
                            tc.code.hover_line_background,
                        )),
                        Some(crate::theme_bridge::rgb_to_color32(
                            tc.code.line_number_text,
                        )),
                        Some(crate::theme_bridge::rgb_to_color32(
                            tc.code.line_number_active_text,
                        )),
                    )
                } else {
                    (
                        ui.visuals().extreme_bg_color,
                        ui.visuals().text_color(),
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                }
            });

            let mut scroll_area = egui::ScrollArea::vertical().id_salt("editor_scroll");

            let consuming_preview = sync_scroll && scroll.source == ScrollSource::Preview;
            if consuming_preview {
                scroll_area = scroll_area
                    .vertical_scroll_offset(scroll.fraction * scroll.editor_max.max(1.0));
            }

            let output = egui::Frame::NONE.fill(code_bg).show(ui, |ui| {
                ui.style_mut().visuals.override_text_color = Some(code_text);
                ui.style_mut().visuals.extreme_bg_color = code_bg;
                if let Some(sel) = code_selection {
                    ui.style_mut().visuals.selection.bg_fill = sel;
                }

                scroll_area.show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        const LINE_NUMBER_MARGIN: f32 = 40.0;
                        const LINE_NUMBER_PAD_RIGHT: f32 = 8.0;
                        let left_margin = LINE_NUMBER_MARGIN;
                        let (ln_rect, _) = ui.allocate_exact_size(
                            egui::vec2(left_margin, 0.0),
                            egui::Sense::hover(),
                        );
                        let text_output = egui::TextEdit::multiline(&mut buffer)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(EDITOR_INITIAL_VISIBLE_ROWS)
                            .margin(egui::Margin {
                                left: 0,
                                right: LINE_NUMBER_MARGIN as i8,
                                top: 0,
                                bottom: 0,
                            })
                            .frame(false)
                            .show(ui);
                        let response = text_output.response;
                        let galley = text_output.galley;

                        if response.clicked() {
                            if let Some(c) = text_output.cursor_range {
                                let char_idx = c.primary.index;
                                let line = galley
                                    .text()
                                    .chars()
                                    .take(char_idx)
                                    .filter(|&ch| ch == '\n')
                                    .count();
                                scroll.scroll_to_line = Some(line);
                            }
                        }

                        let mut current_cursor_y = None;
                        if let Some(c) = text_output.cursor_range {
                            let char_idx = c.primary.index;
                            let paragraph = galley
                                .text()
                                .chars()
                                .take(char_idx)
                                .filter(|&ch| ch == '\n')
                                .count();
                            scroll.active_editor_line = Some(paragraph);

                            let cursor_rect = galley.pos_from_cursor(c.primary);
                            current_cursor_y = Some(cursor_rect.min.y);

                            let min_y = cursor_rect.min.y;
                            let max_y = cursor_rect.max.y;

                            let highlight_rect = egui::Rect::from_min_max(
                                egui::pos2(ln_rect.min.x, response.rect.min.y + min_y),
                                egui::pos2(response.rect.max.x, response.rect.min.y + max_y),
                            );

                            const HIGHLIGHT_ALPHA: u8 = 15;
                            let highlight_color = current_line_bg.unwrap_or_else(|| {
                                if ui.visuals().dark_mode {
                                    crate::theme_bridge::from_white_alpha(HIGHLIGHT_ALPHA)
                                } else {
                                    crate::theme_bridge::from_black_alpha(HIGHLIGHT_ALPHA)
                                }
                            });
                            ui.painter()
                                .rect_filled(highlight_rect, 1.0, highlight_color);
                        } else {
                            scroll.active_editor_line = None;
                        }

                        // Hover highlights from preview pane
                        const HOVER_HIGHLIGHT_ALPHA: u8 = 10;
                        let hover_color = hover_line_bg.unwrap_or_else(|| {
                            if ui.visuals().dark_mode {
                                crate::theme_bridge::from_white_alpha(HOVER_HIGHLIGHT_ALPHA)
                            } else {
                                crate::theme_bridge::from_black_alpha(HOVER_HIGHLIGHT_ALPHA)
                            }
                        });

                        for line_range in &scroll.hovered_preview_lines {
                            let mut current_line = 0;
                            let mut start_char = None;
                            let mut end_char = None;

                            for (char_idx, c) in buffer.chars().enumerate() {
                                if current_line == line_range.start && start_char.is_none() {
                                    start_char = Some(char_idx);
                                }
                                if current_line == line_range.end + 1 {
                                    end_char = Some(char_idx.saturating_sub(1));
                                    break;
                                }
                                if c == '\n' {
                                    current_line += 1;
                                }
                            }
                            if start_char.is_some() && end_char.is_none() {
                                end_char = Some(buffer.chars().count().saturating_sub(1));
                            }

                            if let (Some(start_idx), Some(end_idx)) = (start_char, end_char) {
                                let cursor_start = egui::text::CCursor {
                                    index: start_idx,
                                    prefer_next_row: false,
                                };
                                // Ensure we don't highlight beyond the actual characters
                                let cursor_end = egui::text::CCursor {
                                    index: end_idx.saturating_sub(1),
                                    prefer_next_row: false,
                                };

                                let pos_start = galley.pos_from_cursor(cursor_start);
                                let pos_end = galley.pos_from_cursor(cursor_end);

                                let highlight_rect = egui::Rect::from_min_max(
                                    egui::pos2(
                                        ln_rect.min.x,
                                        response.rect.min.y + pos_start.min.y,
                                    ),
                                    egui::pos2(
                                        response.rect.max.x,
                                        response.rect.min.y + pos_end.max.y,
                                    ),
                                );
                                ui.painter().rect_filled(highlight_rect, 1.0, hover_color);
                            }
                        }

                        // Draw line numbers
                        let clip_rect = ui.clip_rect().expand(100.0);
                        let mut p = 0;
                        let mut is_start_of_para = true;

                        for row in &galley.rows {
                            let top_y = row.rect().min.y;
                            let y = response.rect.min.y + top_y;
                            let is_visible = is_start_of_para
                                && y <= clip_rect.max.y
                                && (y + row.rect().height()) >= clip_rect.min.y;

                            if is_visible {
                                let is_current = current_cursor_y == Some(top_y);
                                let text = format!("{}", p + 1);
                                let color = if is_current {
                                    ln_active_text.unwrap_or_else(|| ui.visuals().text_color())
                                } else {
                                    const LINE_NUMBER_INACTIVE_ALPHA: f32 = 0.3;
                                    ln_text.unwrap_or_else(|| {
                                        ui.visuals()
                                            .text_color()
                                            .linear_multiply(LINE_NUMBER_INACTIVE_ALPHA)
                                    })
                                };
                                let font_id = egui::TextStyle::Monospace.resolve(ui.style());

                                let label_rect = egui::Rect::from_min_size(
                                    egui::pos2(ln_rect.min.x, y),
                                    egui::vec2(
                                        left_margin - LINE_NUMBER_PAD_RIGHT,
                                        row.rect().height(),
                                    ),
                                );
                                let mut text_rt =
                                    egui::RichText::new(text).color(color).font(font_id);
                                if is_current {
                                    text_rt = text_rt.strong();
                                }

                                let label_for_measuring =
                                    egui::Label::new(text_rt.clone()).selectable(false);
                                // align right
                                let galley_ln = label_for_measuring.layout_in_ui(ui);
                                let offset_x =
                                    (label_rect.width() - galley_ln.1.rect.width()).max(0.0);
                                let tight_rect = egui::Rect::from_min_size(
                                    label_rect.min + egui::vec2(offset_x, 0.0),
                                    galley_ln.1.rect.size(),
                                );

                                let resp =
                                    ui.interact(label_rect, ui.id().with(p), egui::Sense::click());
                                if resp.clicked() {
                                    scroll.scroll_to_line = Some(p);
                                }
                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                ui.put(tight_rect, egui::Label::new(text_rt).selectable(false));
                            }

                            is_start_of_para = row.ends_with_newline;
                            if row.ends_with_newline {
                                p += 1;
                            }
                        }

                        if response.changed() {
                            *action = AppAction::UpdateBuffer(buffer.clone());
                        }

                        if let Some(target_line) = scroll.scroll_to_line.take() {
                            let mut current_line = 0;
                            let mut target_char = None;
                            for (char_idx, c) in buffer.chars().enumerate() {
                                if current_line == target_line && target_char.is_none() {
                                    target_char = Some(char_idx);
                                    break;
                                }
                                if c == '\n' {
                                    current_line += 1;
                                }
                            }
                            if let Some(idx) = target_char {
                                let cursor = egui::text::CCursor {
                                    index: idx,
                                    prefer_next_row: false,
                                };
                                let pos = galley.pos_from_cursor(cursor);
                                let rect = egui::Rect::from_min_max(
                                    egui::pos2(
                                        response.rect.min.x,
                                        response.rect.min.y + pos.min.y,
                                    ),
                                    egui::pos2(
                                        response.rect.max.x,
                                        response.rect.min.y + pos.max.y,
                                    ),
                                );
                                ui.scroll_to_rect(rect, Some(egui::Align::Center));
                            }
                        }
                        response
                    })
                    .inner
                })
            });

            if sync_scroll {
                let max_scroll =
                    (output.inner.content_size.y - output.inner.inner_rect.height()).max(0.0);
                scroll.editor_max = max_scroll;

                if consuming_preview {
                    scroll.source = ScrollSource::Neither;
                    if max_scroll > 0.0 {
                        scroll.fraction =
                            (output.inner.state.offset.y / max_scroll).clamp(0.0, 1.0);
                    }
                } else {
                    if max_scroll > 0.0 {
                        let current_fraction =
                            (output.inner.state.offset.y / max_scroll).clamp(0.0, 1.0);
                        let diff = (current_fraction - scroll.fraction).abs();
                        if diff > SCROLL_SYNC_DEAD_ZONE {
                            scroll.fraction = current_fraction;
                            scroll.source = ScrollSource::Editor;
                        }
                    }
                }
            }
        }
    }
}
