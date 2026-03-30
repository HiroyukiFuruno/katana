use eframe::egui::{self, ScrollArea};


use super::pane::*;
use super::types::*;

impl PreviewPane {
    pub fn show(&mut self, ui: &mut egui::Ui) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        self.repaint_ctx = Some(ui.ctx().clone());
        self.poll_renders(ui.ctx());

        let mut request: Option<DownloadRequest> = None;
        let mut actions = Vec::new();
        let content_width = ui.available_width();
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let child_rect = egui::Rect::from_min_size(
                    ui.next_widget_position(),
                    egui::vec2(content_width, 0.0),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(child_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        let (req, act) = self.render_sections(ui, None, None);
                        request = req;
                        actions = act;
                    },
                );
            });
        self.render_fullscreen_modal(ui.ctx());
        (request, actions)
    }

    pub fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        active_editor_line: Option<usize>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
    ) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        self.repaint_ctx = Some(ui.ctx().clone());
        self.poll_renders(ui.ctx());
        let (request, actions) = self.render_sections(ui, active_editor_line, hovered_lines);
        self.render_fullscreen_modal(ui.ctx());
        (request, actions)
    }

    pub(crate) fn render_sections(
        &mut self,
        ui: &mut egui::Ui,
        active_editor_line: Option<usize>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
    ) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        self.visible_rect = Some(ui.clip_rect());
        self.content_top_y = ui.next_widget_position().y;
        self.heading_anchors.clear();
        let mut fullscreen_request: Option<usize> = None;
        let (request, actions) = crate::preview_pane::render_sections(
            ui,
            &mut self.commonmark_cache,
            &self.sections,
            &self.md_file_path,
            self.scroll_request,
            Some(&mut self.heading_anchors),
            Some(&mut self.viewer_states),
            Some(&mut fullscreen_request),
            active_editor_line,
            hovered_lines,
        );
        self.scroll_request = None;

        let ctx = ui.ctx().clone();
        self.handle_fullscreen_request(fullscreen_request, Some(&ctx));

        (request, actions)
    }

    pub(crate) fn render_fullscreen_modal(&mut self, ctx: &egui::Context) {
        let result = crate::preview_pane::render_fullscreen_if_active(
            ctx,
            &self.sections,
            self.fullscreen_image,
            &mut self.fullscreen_viewer_state,
        );
        self.apply_fullscreen_result(result, Some(ctx));
    }

    pub(crate) fn apply_fullscreen_result(
        &mut self,
        result: Option<usize>,
        ctx: Option<&egui::Context>,
    ) {
        if result.is_none() && self.fullscreen_image.is_some() {
            self.fullscreen_viewer_state.reset();
            if let Some(ctx) = ctx {
                if !self.was_os_fullscreen_before_modal {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                }
            }
        }
        self.fullscreen_image = result;
    }

    pub(crate) fn handle_fullscreen_request(
        &mut self,
        request: Option<usize>,
        ctx: Option<&egui::Context>,
    ) {
        if let Some(idx) = request {
            if self.fullscreen_image.is_none() {
                if let Some(ctx) = ctx {
                    let is_native_fs = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);
                    self.was_os_fullscreen_before_modal = is_native_fs;
                    if !is_native_fs {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    }
                }
            }
            self.fullscreen_image = Some(idx);
        }
        if let Some(idx) = self.fullscreen_image {
            match self.sections.get(idx) {
                Some(RenderedSection::Image { .. }) => {} // WHY: valid, keep open
                _ => self.fullscreen_image = None,
            }
        }
    }
}