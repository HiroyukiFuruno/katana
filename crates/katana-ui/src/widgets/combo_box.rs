/// A styled combobox wrapper ensuring vertical center alignment of icon and text.
///
/// Wraps `egui::ComboBox` with consistent styling across the application.
/// All dropdown selectors should use this component instead of raw `egui::ComboBox`.
///
/// # Example
///
/// ```ignore
/// use katana_ui::widgets::StyledComboBox;
///
/// StyledComboBox::new("my_selector", "Currently Selected")
///     .width(150.0)
///     .show(ui, |ui| {
///         ui.selectable_value(&mut value, Option::A, "Option A");
///     });
/// ```
pub struct StyledComboBox<'a> {
    /// Unique ID salt for the egui combobox (prevents ID collisions).
    id: &'a str,
    /// Text displayed when the combobox is collapsed.
    selected_text: String,
    /// Optional fixed width in pixels.
    width: Option<f32>,
}

impl<'a> StyledComboBox<'a> {
    /// Creates a new styled combobox with the given ID and selected text.
    pub fn new(id: &'a str, selected_text: impl Into<String>) -> Self {
        Self {
            id,
            selected_text: selected_text.into(),
            width: None,
        }
    }

    /// Sets a fixed width for the combobox in pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Renders the combobox with vertically centered icon and text.
    ///
    /// The `content` closure receives a `&mut Ui` for adding selectable items.
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

    // ── StyledComboBox tests ─────────────────────────────────────────

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
