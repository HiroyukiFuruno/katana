use eframe::egui;

pub struct Accordion<'a> {
    id_source: egui::Id,
    label: egui::WidgetText,
    default_open: bool,
    force_open: Option<bool>,
    body: Box<dyn FnOnce(&mut egui::Ui) + 'a>,
}

impl<'a> Accordion<'a> {
    pub fn new(
        id_source: impl std::hash::Hash,
        label: impl Into<egui::WidgetText>,
        body: impl FnOnce(&mut egui::Ui) + 'a,
    ) -> Self {
        Self {
            id_source: egui::Id::new(id_source),
            label: label.into(),
            default_open: false,
            force_open: None,
            body: Box::new(body),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn open(mut self, open: Option<bool>) -> Self {
        self.force_open = open;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            self.id_source,
            self.default_open,
        );

        if let Some(force_open) = self.force_open {
            state.set_open(force_open);
        }

        let openness = state.openness(ui.ctx());

        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let galley =
            self.label
                .into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, font_id);

        let icon_size = ui.spacing().icon_width;
        let spacing = ui.spacing().item_spacing.x;

        let desired_size = egui::vec2(
            icon_size + spacing + galley.size().x,
            ui.spacing()
                .interact_size
                .y
                .max(icon_size)
                .max(galley.size().y),
        );

        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, true, galley.text())
        });

        let text_color = ui.style().interact(&response).text_color();

        if response.hovered() {
            ui.painter().rect_filled(
                rect,
                ui.style().visuals.widgets.hovered.corner_radius,
                ui.style().visuals.widgets.hovered.bg_fill,
            );
        }

        let icon_min_y = rect.center().y - icon_size / 2.0;
        let icon_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, icon_min_y),
            egui::pos2(rect.min.x + icon_size, icon_min_y + icon_size),
        );

        let icon_response = ui.interact(icon_rect, ui.next_auto_id(), egui::Sense::click());

        let stroke_color = if response.hovered() || response.has_focus() {
            ui.style().visuals.widgets.hovered.fg_stroke.color
        } else {
            ui.style().interact(&response).fg_stroke.color
        };

        const TRIANGLE_RADIUS_RATIO: f32 = 0.3;
        const TRIANGLE_BACK_RATIO: f32 = 0.6;
        
        let center = icon_rect.center();
        let triangle_radius = icon_size * TRIANGLE_RADIUS_RATIO; // WHY: Make the triangle nice and small!

        let rot = openness * std::f32::consts::FRAC_PI_2;
        let rot_mat = egui::emath::Rot2::from_angle(rot);
        let transform = |p: egui::Pos2| center + rot_mat * p.to_vec2();

        let points = vec![
            transform(egui::pos2(triangle_radius, 0.0)),
            transform(egui::pos2(-triangle_radius * TRIANGLE_BACK_RATIO, -triangle_radius)),
            transform(egui::pos2(-triangle_radius * TRIANGLE_BACK_RATIO, triangle_radius)),
        ];

        ui.painter().add(egui::Shape::convex_polygon(
            points,
            stroke_color,
            egui::Stroke::NONE,
        ));

        let text_min_y = rect.center().y - galley.size().y / 2.0;
        let text_pos = egui::pos2(rect.min.x + icon_size + spacing, text_min_y);
        ui.painter().galley(text_pos, galley, text_color);

        if response.clicked() || icon_response.clicked() {
            state.toggle(ui);
        }

        state.show_body_unindented(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(ui.spacing().indent);
                ui.vertical(|ui| {
                    (self.body)(ui);
                });
            });
        });
    }
}