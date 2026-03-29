pub mod centering {
    use egui::{remap, vec2, Rect, Response, Shape, Stroke, Ui};
    use std::f32::consts::TAU;

    /// Constant for optical Y-offset applied to the accordion triangle to counter native egui geometrical centering.
    /// Changed from -4.0 to 0.0 to fix vertical misalignment against text per user feedback.
    const OPTICAL_Y_OFFSET: f32 = 0.0;

    /// Helper struct for custom UI drawing with optical corrections.
    pub struct AccordionIcon;

    impl AccordionIcon {
        /// Draws the default egui triangle icon, but applies a negative Y-offset to perfectly align the 
        /// geometric center of the icon to the optical crossbar cap height of typical Japanese body fonts.
        /// This resolves the issue where the native egui implementation causes the triangle to appear 
        /// completely bottom-aligned (touching the baseline) due to the large descender metrics in IMGUI parsing.
        pub fn paint_optically_centered(ui: &mut Ui, openness: f32, response: &Response) {
            let visuals = ui.style().interact(response);
            let mut rect = response.rect;

            // Optical offset: Shift the geometric box UP by OPTICAL_Y_OFFSET pixels.
            rect.set_center(egui::pos2(rect.center().x, rect.center().y + OPTICAL_Y_OFFSET));

            let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height()) * 0.75);
            let rect = rect.expand(visuals.expansion);
            let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
            
            let rotation = egui::emath::Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
            for p in &mut points {
                *p = rect.center() + rotation * (*p - rect.center());
            }

            ui.painter().add(Shape::convex_polygon(
                points,
                visuals.fg_stroke.color,
                Stroke::NONE,
            ));
        }
    }
}

pub mod task_list {
    use egui::Ui;
    use std::ops::Range;

    pub fn katana_task_box(
        ui: &mut Ui,
        state: char,
        span: Range<usize>,
        mutable: bool,
        events: &mut Vec<crate::TaskListAction>,
    ) {
        let is_checked = state == 'x' || state == 'X';
        let is_progress = state == '/' || state == '-' || state == '~';
        let is_active = is_checked || is_progress;

        let icon_width = ui.spacing().icon_width;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(icon_width, icon_width),
            if mutable { egui::Sense::click() } else { egui::Sense::hover() },
        );

        if ui.is_rect_visible(rect) {
            // Use `interact_selectable` to colorize background if it's checked or in progress
            let visuals = ui.style().interact_selectable(&response, is_active);
            let rounding = ui.visuals().widgets.noninteractive.corner_radius;
            
            ui.painter().rect(
                rect.expand(visuals.expansion),
                rounding,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );

            let stroke_width = ui.visuals().widgets.noninteractive.fg_stroke.width.max(1.5);
            let stroke = egui::Stroke::new(stroke_width, visuals.fg_stroke.color);
            let center = rect.center();
            let width = rect.width();

            if is_checked {
                // Draw checkmark
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-width * 0.25, 0.0),
                        center + egui::vec2(-width * 0.05, width * 0.25),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-width * 0.05, width * 0.25),
                        center + egui::vec2(width * 0.3, -width * 0.25),
                    ],
                    stroke,
                );
            } else if is_progress {
                if state == '/' {
                    // Slight diagonal for [/]
                    let half_w = width * 0.35;
                    ui.painter().line_segment(
                        [
                            center + egui::vec2(-half_w, half_w),
                            center + egui::vec2(half_w, -half_w),
                        ],
                        stroke,
                    );
                } else {
                    // Horizontal for [-]
                    let half_w = width * 0.35;
                    ui.painter().line_segment(
                        [
                            center - egui::vec2(half_w, 0.0),
                            center + egui::vec2(half_w, 0.0),
                        ],
                        stroke,
                    );
                }
            }
        }

        if mutable {
            if response.clicked() {
                let new_state = match state {
                    ' ' => 'x',
                    '/' | '-' | '~' => 'x',
                    'x' | 'X' => ' ',
                    _ => ' ',
                };
                events.push(crate::TaskListAction {
                    span: span.clone(),
                    new_state,
                });
            }

            response.context_menu(|ui| {
                if ui.button("未実施 [ ]").clicked() {
                    events.push(crate::TaskListAction {
                        span: span.clone(),
                        new_state: ' ',
                    });
                    ui.close();
                }
                if ui.button("実施中 [/]").clicked() {
                    events.push(crate::TaskListAction {
                        span: span.clone(),
                        new_state: '/',
                    });
                    ui.close();
                }
                if ui.button("完了 [x]").clicked() {
                    events.push(crate::TaskListAction {
                        span: span.clone(),
                        new_state: 'x',
                    });
                    ui.close();
                }
            });
        }
        
        // Add margin between checkbox and text
        ui.add_space(8.0);
    }
}
