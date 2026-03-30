pub struct StyledComboBox<'a> {
    id: &'a str,
    selected_text: String,
    width: Option<f32>,
}

impl<'a> StyledComboBox<'a> {
    pub fn new(id: &'a str, selected_text: impl Into<String>) -> Self {
        Self {
            id,
            selected_text: selected_text.into(),
            width: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
        let mut combo = egui::ComboBox::from_id_salt(self.id).selected_text(self.selected_text);

        if let Some(width) = self.width {
            combo = combo.width(width);
        }

        combo.show_ui(ui, |ui| {
            content(ui);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_styled_combobox_builder_defaults() {
        let combo = StyledComboBox::new("test_id", "Selected");
        assert_eq!(combo.id, "test_id");
        assert_eq!(combo.selected_text, "Selected");
        assert!(combo.width.is_none());
    }

    #[test]
    fn test_styled_combobox_builder_with_width() {
        let combo = StyledComboBox::new("test_id", "Selected").width(150.0);
        assert_eq!(combo.width, Some(150.0));
    }

    #[test]
    fn test_styled_combobox_renders_without_panic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                StyledComboBox::new("render_test", "Value").show(ui, |ui| {
                    ui.label("item_a");
                });
            });
        });
    }

    #[test]
    fn test_styled_combobox_renders_with_width() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                StyledComboBox::new("width_test", "Value")
                    .width(200.0)
                    .show(ui, |ui| {
                        ui.label("item_a");
                    });
            });
        });
    }
}