use crate::icon::Icon;
use crate::preview_pane::{RenderedSection, ViewerState};
use eframe::egui::{self, Vec2};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) const FULLSCREEN_PADDING: f32 = 40.0;
pub(crate) const FULLSCREEN_CLOSE_SIZE: f32 = 32.0;
pub(crate) const FULLSCREEN_CLOSE_MARGIN: f32 = 20.0;
pub(crate) const MIN_ZOOM: f32 = 0.5;
pub(crate) const MAX_ZOOM: f32 = 5.0;

pub(crate) fn render_fullscreen_if_active(
    ctx: &egui::Context,
    sections: &[RenderedSection],
    fullscreen_image: Option<usize>,
    fullscreen_state: &mut ViewerState,
) -> Option<usize> {
    let idx = fullscreen_image?;
    if let Some(RenderedSection::Image { svg_data, alt, .. }) = sections.get(idx) {
        if show_fullscreen_modal(ctx, svg_data, alt, fullscreen_state, idx) {
            Some(idx) // WHY: keep open
        } else {
            None // WHY: user closed
        }
    } else if let Some(RenderedSection::LocalImage { path, alt, .. }) = sections.get(idx) {
        if show_fullscreen_local_image(ctx, path, alt, fullscreen_state, idx) {
            Some(idx) // WHY: keep open
        } else {
            None // WHY: user closed
        }
    } else {
        None // WHY: section gone
    }
}

pub(crate) fn show_fullscreen_modal(
    ctx: &egui::Context,
    img: &RasterizedSvg,
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
                    Icon::CloseModal
                        .image(crate::icon::IconSize::Large)
                        .tint(crate::theme_bridge::WHITE),
                )
                .fill(
                    crate::theme_bridge::TRANSPARENT, /* WHY: Handled by theme overlay */
                )
                .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
            );
            if close_resp.on_hover_text(&dc.close).clicked() {
                keep_open = false;
            }
        });

    keep_open
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
                match std::fs::read(path) {
                    Ok(bytes) => match image::load_from_memory(&bytes) {
                        Ok(dyn_img) => {
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
                        Err(_) => {}
                    },
                    Err(_) => {}
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
                    crate::theme_bridge::TRANSPARENT, /* WHY: Handled by theme overlay */
                )
                .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
            );
            if close_resp.on_hover_text(&dc.close).clicked() {
                keep_open = false;
            }
        });

    keep_open
}