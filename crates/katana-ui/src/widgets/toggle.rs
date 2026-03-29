/// A custom generic toggle switch widget.
///
/// Designed to visually represent boolean states as an iOS-style switch
/// instead of a traditional checkbox.
///
/// # Example
///
/// ```ignore
/// use katana_ui::widgets::toggle_switch;
///
/// let mut is_enabled = true;
/// if toggle_switch(ui, &mut is_enabled).clicked() {
///     // Handle state change
/// }
/// ```
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        const TOGGLE_RADIUS_RATIO: f32 = 0.5;
        let radius = TOGGLE_RADIUS_RATIO * rect.height();
        ui.painter().rect(
            rect,
            radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        const TOGGLE_CIRCLE_RATIO: f32 = 0.75;
        ui.painter().circle(
            center,
            TOGGLE_CIRCLE_RATIO * radius,
            visuals.bg_fill,
            visuals.fg_stroke,
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_switch_clicked() {
        let mut on = false;
        let ctx = egui::Context::default();

        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rect = toggle_switch(ui, &mut on).rect;
            });
        });

        assert!(!on);

        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::PointerMoved(rect.center()));
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        });

        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = toggle_switch(ui, &mut on);
            });
        });

        assert!(on);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TogglePosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToggleAlignment {
    /// Attach the toggle to the text with a specific margin.
    Attached(f32),
    /// Push the toggle and text to opposite ends of the available width.
    SpaceBetween,
}

/// A labeled toggle switch that ensures perfect vertical centering between text and the toggle.
/// It supports flexible alignment and positioning (e.g. left vs right).
pub struct LabeledToggle<'a> {
    text: egui::WidgetText,
    on: &'a mut bool,
    position: TogglePosition,
    alignment: ToggleAlignment,
}

impl<'a> LabeledToggle<'a> {
    pub fn new(text: impl Into<egui::WidgetText>, on: &'a mut bool) -> Self {
        Self {
            text: text.into(),
            on,
            position: TogglePosition::Right,
            alignment: ToggleAlignment::SpaceBetween,
        }
    }

    pub fn position(mut self, position: TogglePosition) -> Self {
        self.position = position;
        self
    }

    pub fn alignment(mut self, alignment: ToggleAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<'a> egui::Widget for LabeledToggle<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        const TOGGLE_Y_OFFSET: f32 = 0.0;

        let toggle_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
        let text_galley = self.text.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        );
        let text_size = text_galley.size();

        let available_width = ui.available_width();
        let total_width = match self.alignment {
            ToggleAlignment::Attached(margin) => text_size.x + margin + toggle_size.x,
            ToggleAlignment::SpaceBetween => available_width.max(text_size.x + toggle_size.x),
        };

        let height = text_size.y.max(toggle_size.y);
        let desired_size = egui::vec2(total_width, height);

        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }

        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *self.on, "")
        });

        if ui.is_rect_visible(rect) {
            let (text_rect, toggle_rect) = match (self.position, self.alignment) {
                (TogglePosition::Right, ToggleAlignment::SpaceBetween) => {
                    let text_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, rect.center().y - text_size.y / 2.0),
                        text_size,
                    );
                    let toggle_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            rect.max.x - toggle_size.x,
                            rect.center().y - toggle_size.y / 2.0 - TOGGLE_Y_OFFSET,
                        ),
                        toggle_size,
                    );
                    (text_rect, toggle_rect)
                }
                (TogglePosition::Left, ToggleAlignment::SpaceBetween) => {
                    let toggle_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            rect.min.x,
                            rect.center().y - toggle_size.y / 2.0 - TOGGLE_Y_OFFSET,
                        ),
                        toggle_size,
                    );
                    let text_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            rect.max.x - text_size.x,
                            rect.center().y - text_size.y / 2.0,
                        ),
                        text_size,
                    );
                    (text_rect, toggle_rect)
                }
                (TogglePosition::Right, ToggleAlignment::Attached(margin)) => {
                    let text_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, rect.center().y - text_size.y / 2.0),
                        text_size,
                    );
                    let toggle_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            rect.min.x + text_size.x + margin,
                            rect.center().y - toggle_size.y / 2.0 - TOGGLE_Y_OFFSET,
                        ),
                        toggle_size,
                    );
                    (text_rect, toggle_rect)
                }
                (TogglePosition::Left, ToggleAlignment::Attached(margin)) => {
                    let toggle_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            rect.min.x,
                            rect.center().y - toggle_size.y / 2.0 - TOGGLE_Y_OFFSET,
                        ),
                        toggle_size,
                    );
                    let text_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            rect.min.x + toggle_size.x + margin,
                            rect.center().y - text_size.y / 2.0,
                        ),
                        text_size,
                    );
                    (text_rect, toggle_rect)
                }
            };

            let text_color = ui.style().interact(&response).text_color();
            let text_pos = egui::Align2::LEFT_TOP
                .align_size_within_rect(text_galley.size(), text_rect)
                .min;
            ui.painter()
                .galley_with_override_text_color(text_pos, text_galley, text_color);

            let how_on = ui.ctx().animate_bool(response.id, *self.on);
            let visuals = ui.style().interact_selectable(&response, *self.on);
            let expanded_toggle_rect = toggle_rect.expand(visuals.expansion);
            const TOGGLE_RADIUS_RATIO: f32 = 0.5;
            let radius = TOGGLE_RADIUS_RATIO * expanded_toggle_rect.height();
            ui.painter().rect(
                expanded_toggle_rect,
                radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );
            let circle_x = egui::lerp(
                (expanded_toggle_rect.left() + radius)..=(expanded_toggle_rect.right() - radius),
                how_on,
            );
            let center = egui::pos2(circle_x, expanded_toggle_rect.center().y);
            const TOGGLE_CIRCLE_RATIO: f32 = 0.75;
            ui.painter().circle(
                center,
                TOGGLE_CIRCLE_RATIO * radius,
                visuals.bg_fill,
                visuals.fg_stroke,
            );
        }

        response
    }
}

#[cfg(test)]
mod labeled_toggle_tests {
    use super::*;
    use egui::Context;

    #[test]
    fn test_labeled_toggle_alignments() {
        let ctx = Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut on = false;

                // SpaceBetween Right
                let _ = ui.add(
                    LabeledToggle::new("Toggle Space Right", &mut on)
                        .position(TogglePosition::Right)
                        .alignment(ToggleAlignment::SpaceBetween),
                );

                // SpaceBetween Left
                let _ = ui.add(
                    LabeledToggle::new("Toggle Space Left", &mut on)
                        .position(TogglePosition::Left)
                        .alignment(ToggleAlignment::SpaceBetween),
                );

                // Attached Right
                let _ = ui.add(
                    LabeledToggle::new("Toggle Attached Right", &mut on)
                        .position(TogglePosition::Right)
                        .alignment(ToggleAlignment::Attached(8.0)),
                );

                // Attached Left
                let _ = ui.add(
                    LabeledToggle::new("Toggle Attached Left", &mut on)
                        .position(TogglePosition::Left)
                        .alignment(ToggleAlignment::Attached(8.0)),
                );

                // Click interaction
                let mut on_click = false;
                let _response = ui.add(LabeledToggle::new("Clickable", &mut on_click));
            });
        });
    }

    #[test]
    fn test_labeled_toggle_click() {
        let mut on = false;
        let ctx = Context::default();

        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rect = ui.add(LabeledToggle::new("Clickable", &mut on)).rect;
            });
        });

        assert!(!on);

        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::PointerMoved(rect.center()));
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });

        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _response = ui.add(LabeledToggle::new("Clickable", &mut on));
            });
        });

        assert!(on);
    }
}
