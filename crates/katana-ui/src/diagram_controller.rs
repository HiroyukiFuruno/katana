//! Reusable overlay control grid for diagram/image pan, zoom, reset, and fullscreen.
//!
//! This module is a self-contained UI component (similar to a React component)
//! that can be rendered on top of any image/diagram container.

use crate::i18n;
use crate::icon::{Icon, IconSize};
use crate::preview_pane::ViewerState;
use egui::Vec2;

/// Size of overlay control buttons.
const BUTTON_SIZE: f32 = 28.0;
/// Margin from container edge to overlay buttons.
const MARGIN: f32 = 8.0;
/// Semi-transparent background for overlay buttons.
const BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(40, 40, 40, 200);
/// Gap between grid buttons.
const GAP: f32 = 2.0;
/// Number of columns/rows in the control grid.
const GRID_DIM: f32 = 3.0;

/// Action returned by the overlay controls.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ControlAction {
    /// No action taken.
    None,
}

/// Draws the overlay control grid on the given container rect.
///
/// Layout (matching GitHub-style reference):
/// ```text
///         [PanUp]  [ZoomIn]
/// [PanLeft] [Reset] [PanRight]
///         [PanDown] [ZoomOut]
/// ```
///
/// Returns `ControlAction::Fullscreen` if the fullscreen button was clicked.
pub(crate) fn draw_controls(
    ui: &mut egui::Ui,
    state: &mut ViewerState,
    container_rect: egui::Rect,
) -> ControlAction {
    let msgs = i18n::get();
    let dc = &msgs.preview.diagram_controller;

    let grid_w = BUTTON_SIZE * GRID_DIM + GAP * (GRID_DIM - 1.0);
    let grid_h = BUTTON_SIZE * GRID_DIM + GAP * (GRID_DIM - 1.0);
    let grid_origin = egui::pos2(
        container_rect.right() - grid_w - MARGIN,
        container_rect.bottom() - grid_h - MARGIN,
    );

    let btn_rect = |col: f32, row: f32| -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(
                grid_origin.x + col * (BUTTON_SIZE + GAP),
                grid_origin.y + row * (BUTTON_SIZE + GAP),
            ),
            Vec2::splat(BUTTON_SIZE),
        )
    };

    // Row 0: (empty), PanUp, ZoomIn
    if ui
        .put(
            btn_rect(1.0, 0.0),
            egui::Button::image(Icon::PanUp.image(IconSize::Large)).fill(BG),
        )
        .on_hover_text(&dc.pan_up)
        .clicked()
    {
        state.pan_up();
    }
    if ui
        .put(
            btn_rect(2.0, 0.0),
            egui::Button::image(Icon::ZoomIn.image(IconSize::Large)).fill(BG),
        )
        .on_hover_text(&dc.zoom_in)
        .clicked()
    {
        state.zoom_in();
    }

    // Row 1: PanLeft, Reset, PanRight
    if ui
        .put(
            btn_rect(0.0, 1.0),
            egui::Button::image(Icon::PanLeft.image(IconSize::Large)).fill(BG),
        )
        .on_hover_text(&dc.pan_left)
        .clicked()
    {
        state.pan_left();
    }
    if ui
        .put(
            btn_rect(1.0, 1.0),
            egui::Button::image(Icon::ResetView.image(IconSize::Large)).fill(BG),
        )
        .on_hover_text(&dc.reset)
        .clicked()
    {
        state.reset();
    }
    if ui
        .put(
            btn_rect(2.0, 1.0),
            egui::Button::image(Icon::PanRight.image(IconSize::Large)).fill(BG),
        )
        .on_hover_text(&dc.pan_right)
        .clicked()
    {
        state.pan_right();
    }

    // Row 2: Info (trackpad help), PanDown, ZoomOut
    ui.put(
        btn_rect(0.0, 2.0),
        egui::Button::image(Icon::Info.image(IconSize::Large)).fill(BG),
    )
    .on_hover_text(&dc.trackpad_help);

    if ui
        .put(
            btn_rect(1.0, 2.0),
            egui::Button::image(Icon::PanDown.image(IconSize::Large)).fill(BG),
        )
        .on_hover_text(&dc.pan_down)
        .clicked()
    {
        state.pan_down();
    }
    if ui
        .put(
            btn_rect(2.0, 2.0),
            egui::Button::image(Icon::ZoomOut.image(IconSize::Large)).fill(BG),
        )
        .on_hover_text(&dc.zoom_out)
        .clicked()
    {
        state.zoom_out();
    }

    ControlAction::None
}

/// Draws a fullscreen button at the top-right of the container.
///
/// Returns `true` if clicked.
pub(crate) fn draw_fullscreen_button(ui: &mut egui::Ui, container_rect: egui::Rect) -> bool {
    let msgs = i18n::get();
    let dc = &msgs.preview.diagram_controller;

    let btn_rect = egui::Rect::from_min_size(
        egui::pos2(
            container_rect.right() - BUTTON_SIZE - MARGIN,
            container_rect.top() + MARGIN,
        ),
        Vec2::splat(BUTTON_SIZE),
    );
    ui.put(
        btn_rect,
        egui::Button::image(Icon::Fullscreen.image(IconSize::Large)).fill(BG),
    )
    .on_hover_text(&dc.fullscreen)
    .clicked()
}
