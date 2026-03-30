use eframe::egui;

pub type ListItemNode<'a> = Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + 'a>;

pub struct ListItem<'a> {
    left_nodes: Vec<ListItemNode<'a>>,
    right_nodes: Vec<ListItemNode<'a>>,
    spacing: f32,
    interactive: bool,
    width: Option<f32>,
    shrink_to_fit: bool,
}

impl<'a> Default for ListItem<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ListItem<'a> {
    const DEFAULT_SPACING: f32 = 8.0;

    pub fn new() -> Self {
        Self {
            left_nodes: Vec::new(),
            right_nodes: Vec::new(),
            spacing: Self::DEFAULT_SPACING,
            interactive: false,
            width: None,
            shrink_to_fit: false,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn left(mut self, node: impl FnOnce(&mut egui::Ui) -> egui::Response + 'a) -> Self {
        self.left_nodes.push(Box::new(node));
        self
    }

    pub fn right(mut self, node: impl FnOnce(&mut egui::Ui) -> egui::Response + 'a) -> Self {
        self.right_nodes.push(Box::new(node));
        self
    }

    pub fn shrink_to_fit(mut self, shrink: bool) -> Self {
        self.shrink_to_fit = shrink;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let available_width = self.width.unwrap_or_else(|| ui.available_width());
        let row_height = ui.spacing().interact_size.y;

        let sense = if self.interactive {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };

        if self.shrink_to_fit {
            let final_response = ui
                .allocate_ui_with_layout(
                    egui::vec2(0.0, row_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |child_ui| {
                        child_ui.spacing_mut().item_spacing.x = self.spacing;
                        for node_fn in self.left_nodes {
                            node_fn(child_ui);
                        }
                        if !self.right_nodes.is_empty() {
                            child_ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |right_ui| {
                                    for node_fn in self.right_nodes.into_iter().rev() {
                                        node_fn(right_ui);
                                    }
                                },
                            );
                        }
                    },
                )
                .response;

            let mut rect = final_response.rect;
            if rect.height() < row_height {
                rect = rect.expand2(egui::vec2(0.0, (row_height - rect.height()) / 2.0));
            }

            let response = ui.interact(rect, ui.next_auto_id(), sense);

            if self.interactive && response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    ui.style().visuals.widgets.hovered.corner_radius,
                    ui.style().visuals.widgets.hovered.bg_fill,
                );
            }
            return response;
        }

        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, row_height), sense);

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.spacing_mut().item_spacing.x = self.spacing;

        for node_fn in self.left_nodes {
            node_fn(&mut child_ui);
        }

        if !self.right_nodes.is_empty() {
            child_ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |right_ui| {
                    for node_fn in self.right_nodes.into_iter().rev() {
                        node_fn(right_ui);
                    }
                },
            );
        }

        if self.interactive && response.hovered() {
            ui.painter().rect_filled(
                rect,
                ui.style().visuals.widgets.hovered.corner_radius,
                ui.style().visuals.widgets.hovered.bg_fill,
            );
        }

        response
    }
}