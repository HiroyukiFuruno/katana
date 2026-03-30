use crate::preview_pane::{DownloadRequest, ViewerState};
use eframe::egui::{self, Vec2};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

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
                &[("tool", kind)],
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
                &[("path", path_str.as_str())],
            ))
            .small()
            .weak(),
        );
        if ui
            .button(crate::i18n::tf(
                &crate::i18n::get().tool.download,
                &[("tool", kind)],
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

const MIN_ZOOM: f32 = 0.1;
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

    let zoom = state.as_ref().map_or(1.0, |s| s.zoom);
    let pan = state.as_ref().map_or(egui::Vec2::ZERO, |s| s.pan);

    let base_size = Vec2::new(
        img.width as f32 * base_scale,
        img.height as f32 * base_scale,
    );

    let zoomed_size = base_size * zoom;

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

    let x_offset = (max_w - base_size.x).max(0.0) / 2.0;

    let image_pos = container_rect.min + egui::vec2(x_offset, 0.0) + pan;
    let image_rect = egui::Rect::from_min_size(image_pos, zoomed_size);
    ui.painter().with_clip_rect(container_rect).image(
        texture_handle.id(),
        image_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        crate::theme_bridge::WHITE,
    );

    if let Some(state) = state {
        if crate::diagram_controller::draw_fullscreen_button(ui, container_rect) {
            if let Some(req) = fullscreen_request {
                *req = Some(idx);
            }
        }

        crate::diagram_controller::draw_controls(ui, state, container_rect);
    }
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
                        state.texture = Some(ui.ctx().load_texture(
                            format!("local_image_{id}"),
                            color_img,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                    Err(_) => {}
                },
                Err(_) => {}
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
        None => return, // WHY: Could not load image
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