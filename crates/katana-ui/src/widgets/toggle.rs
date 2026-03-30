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

    paint_toggle_switch(ui, rect, &response, *on);

    response
}

pub(crate) fn paint_toggle_switch(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    on: bool,
) {
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, on);
        let visuals = ui.style().interact_selectable(response, on);
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
    Attached(f32),
    SpaceBetween,
}

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
        let text_galley = self.text.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            ui.available_width(),
            egui::TextStyle::Body,
        );
        let text_size = text_galley.size();
        let toggle_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
        
        let margin = match self.alignment {
            ToggleAlignment::Attached(m) => m,
            ToggleAlignment::SpaceBetween => 0.0,
        };

        let desired_width = match self.alignment {
            ToggleAlignment::Attached(_) => {
                let h_pad = ui.spacing().button_padding.x * 2.0;
                text_size.x + toggle_size.x + margin + h_pad
            }
            ToggleAlignment::SpaceBetween => ui.available_width(),
        };

        let row_pad = ui.spacing().button_padding.y;
        let row_height = text_size.y.max(toggle_size.y).max(ui.spacing().interact_size.y) + row_pad * 2.0;
        
        let (rect, mut response) = ui.allocate_exact_size(
            egui::vec2(desired_width, row_height),
            egui::Sense::click(),
        );

        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }

        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *self.on, "")
        });

        if response.hovered() {
            ui.painter().rect_filled(
                rect,
                ui.style().visuals.widgets.hovered.corner_radius,
                ui.style().visuals.widgets.hovered.bg_fill,
            );
        }

        let (text_pos, toggle_pos) = match (self.position, self.alignment) {
            (TogglePosition::Right, ToggleAlignment::SpaceBetween) => {
                let text_x = rect.left();
                let toggle_x = rect.right() - toggle_size.x;
                (text_x, toggle_x)
            }
            (TogglePosition::Left, ToggleAlignment::SpaceBetween) => {
                let toggle_x = rect.left();
                let text_x = rect.right() - text_size.x;
                (text_x, toggle_x)
            }
            (TogglePosition::Right, ToggleAlignment::Attached(_)) => {
                let h_pad = ui.spacing().button_padding.x;
                let text_x = rect.left() + h_pad;
                let toggle_x = text_x + text_size.x + margin;
                (text_x, toggle_x)
            }
            (TogglePosition::Left, ToggleAlignment::Attached(_)) => {
                let h_pad = ui.spacing().button_padding.x;
                let toggle_x = rect.left() + h_pad;
                let text_x = toggle_x + toggle_size.x + margin;
                (text_x, toggle_x)
            }
        };

        let text_pos = egui::pos2(text_pos, rect.center().y - text_size.y / 2.0);
        let toggle_pos = egui::pos2(toggle_pos, rect.center().y - toggle_size.y / 2.0);

        let toggle_rect = egui::Rect::from_min_size(toggle_pos, toggle_size);

        let text_color = ui.style().interact(&response).text_color();
        ui.painter().galley(text_pos, text_galley, text_color);

        paint_toggle_switch(ui, toggle_rect, &response, *self.on);

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

                let _ = ui.add(
                    LabeledToggle::new("Toggle Space Right", &mut on)
                        .position(TogglePosition::Right)
                        .alignment(ToggleAlignment::SpaceBetween),
                );

                let _ = ui.add(
                    LabeledToggle::new("Toggle Space Left", &mut on)
                        .position(TogglePosition::Left)
                        .alignment(ToggleAlignment::SpaceBetween),
                );

                let _ = ui.add(
                    LabeledToggle::new("Toggle Attached Right", &mut on)
                        .position(TogglePosition::Right)
                        .alignment(ToggleAlignment::Attached(8.0)),
                );

                let _ = ui.add(
                    LabeledToggle::new("Toggle Attached Left", &mut on)
                        .position(TogglePosition::Left)
                        .alignment(ToggleAlignment::Attached(8.0)),
                );

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