//! Reusable UI widgets for the KatanA application.
//!
//! Designed as shared components (analogous to React components) for use
//! across splash screens, update dialogs, export flows, and any future
//! feature requiring a modal overlay.
//!
//! # Design Philosophy (React-inspired)
//!
//! - **Content injection**: Body and footer are separate closures,
//!   analogous to React's `children` and render-prop patterns.
//! - **Optional progress bar**: Shown only when `progress` is `Some`.
//! - **Footer actions**: The footer closure returns `Option<T>`, enabling
//!   the caller to propagate button-click results (e.g., `AppAction`).

/// A generic modal window with optional progress bar and footer buttons.
///
/// # Slots (React-inspired)
///
/// | Slot     | Method          | Purpose                              |
/// |----------|-----------------|--------------------------------------|
/// | Body     | `.show()`       | Main content (labels, rich text)     |
/// | Progress | `.progress()`   | Optional progress bar (0.0–1.0)      |
/// | Footer   | `.show()` return| Buttons that return `Option<T>`      |
///
/// # Example
///
/// ```ignore
/// use katana_ui::widgets::Modal;
///
/// // Full modal with progress + footer buttons
/// let action: Option<AppAction> = Modal::new("update_modal", "Updating")
///     .progress(0.42)
///     .show_percentage(true)
///     .show(ctx, |ui| {
///         ui.label("Downloading update...");
///     }, |ui| {
///         if ui.button("Cancel").clicked() {
///             return Some(AppAction::DismissUpdate);
///         }
///         None
///     });
///
/// // Plain modal (no progress, no footer)
/// Modal::new("info_modal", "Notice")
///     .show_body_only(ctx, |ui| {
///         ui.label("Something happened.");
///     });
/// ```
pub struct Modal<'a> {
    /// Unique ID for the egui window (prevents ID collisions).
    id: &'a str,
    /// Window title.
    title: &'a str,
    /// Progress ratio (0.0–1.0). `None` hides the progress bar entirely.
    progress: Option<f32>,
    /// Whether to show percentage text on the progress bar.
    show_pct: bool,
    /// Width of the progress bar in pixels.
    bar_width: f32,
}

/// Default width of the progress bar inside the modal.
const DEFAULT_BAR_WIDTH: f32 = 280.0;
/// Spacing between body content and the progress bar.
const BODY_TO_BAR_SPACING: f32 = 12.0;
/// Spacing between progress bar (or body) and the footer buttons.
const BAR_TO_FOOTER_SPACING: f32 = 16.0;

impl<'a> Modal<'a> {
    /// Creates a new modal with the given unique ID and title.
    pub fn new(id: &'a str, title: &'a str) -> Self {
        Self {
            id,
            title,
            progress: None,
            show_pct: false,
            bar_width: DEFAULT_BAR_WIDTH,
        }
    }

    /// Sets a determinate progress ratio (0.0–1.0).
    pub fn progress(mut self, ratio: f32) -> Self {
        self.progress = Some(ratio.clamp(0.0, 1.0));
        self
    }

    /// Optionally sets the progress ratio. `None` hides the bar.
    pub fn maybe_progress(mut self, ratio: Option<f32>) -> Self {
        self.progress = ratio.map(|r| r.clamp(0.0, 1.0));
        self
    }

    /// Shows percentage text on the progress bar.
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_pct = show;
        self
    }

    /// Sets the progress bar width in pixels.
    pub fn bar_width(mut self, width: f32) -> Self {
        self.bar_width = width;
        self
    }

    /// Renders the modal with body content and footer buttons.
    ///
    /// - `body`: Closure for main content (labels, spinners, etc.).
    /// - `footer`: Closure for action buttons. Returns `Option<T>` to
    ///   propagate user actions back to the caller (callback pattern).
    pub fn show<T>(
        self,
        ctx: &egui::Context,
        body: impl FnOnce(&mut egui::Ui),
        footer: impl FnOnce(&mut egui::Ui) -> Option<T>,
    ) -> Option<T> {
        let mut result: Option<T> = None;

        // Use a reasonable dialogue width.
        // We do not use auto_sized() alone because right_to_left layouts
        // in the footer will cause the window to expand to full screen width.
        const DIALOG_WIDTH: f32 = 450.0;

        egui::Window::new(self.title)
            .id(egui::Id::new(self.id))
            .collapsible(false)
            .resizable(false)
            .default_width(DIALOG_WIDTH)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                // Ensure the content area does not expand beyond our desired dialog width
                ui.set_max_width(DIALOG_WIDTH);

                ui.vertical_centered(|ui| {
                    ui.set_max_width(DIALOG_WIDTH);
                    // Slot 1: Body content
                    body(ui);

                    // Slot 2: Optional progress bar
                    if let Some(ratio) = self.progress {
                        ui.add_space(BODY_TO_BAR_SPACING);
                        let mut bar = egui::ProgressBar::new(ratio).desired_width(self.bar_width);
                        if self.show_pct {
                            bar = bar.show_percentage();
                        }
                        ui.add(bar);
                    }
                });

                // Slot 3: Footer buttons
                ui.add_space(BAR_TO_FOOTER_SPACING);
                ui.horizontal(|ui| {
                    ui.set_max_width(DIALOG_WIDTH);
                    result = footer(ui);
                });
            });

        result
    }

    /// Renders the modal with body content only (no footer buttons).
    pub fn show_body_only(self, ctx: &egui::Context, body: impl FnOnce(&mut egui::Ui)) {
        self.show(ctx, body, |_ui| None::<()>);
    }
}

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

    // ── Modal tests ──────────────────────────────────────────────────

    #[test]
    fn test_modal_builder_defaults() {
        let modal = Modal::new("t", "Title");
        assert_eq!(modal.progress, None);
        assert!(!modal.show_pct);
        assert!((modal.bar_width - DEFAULT_BAR_WIDTH).abs() < f32::EPSILON);
    }

    #[test]
    fn test_modal_builder_with_progress() {
        let modal = Modal::new("t", "T")
            .progress(0.5)
            .show_percentage(true)
            .bar_width(200.0);
        assert_eq!(modal.progress, Some(0.5));
        assert!(modal.show_pct);
        assert!((modal.bar_width - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_modal_maybe_progress() {
        let with = Modal::new("t", "T").maybe_progress(Some(0.7));
        assert_eq!(with.progress, Some(0.7));

        let without = Modal::new("t", "T").maybe_progress(None);
        assert_eq!(without.progress, None);
    }

    #[test]
    fn test_modal_progress_clamps() {
        let over = Modal::new("t", "T").progress(1.5);
        assert_eq!(over.progress, Some(1.0));

        let under = Modal::new("t", "T").progress(-0.5);
        assert_eq!(under.progress, Some(0.0));
    }

    #[test]
    fn test_show_body_only_renders() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            Modal::new("test_body", "Body Only").show_body_only(ctx, |ui| {
                ui.label("hello");
            });
        });
    }

    #[test]
    fn test_show_with_footer_returns_action() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let _result = Modal::new("test_footer", "Footer").show(
                ctx,
                |ui| {
                    ui.label("body");
                },
                |_ui| Some(42_i32),
            );
        });
    }

    #[test]
    fn test_show_with_progress_and_percentage() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let _ = Modal::new("test_prog", "Progress")
                .progress(0.5)
                .show_percentage(true)
                .show(
                    ctx,
                    |ui| {
                        ui.label("downloading");
                    },
                    |_ui| None::<()>,
                );
        });
    }

    #[test]
    fn test_show_without_progress_footer_none() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let _ = Modal::new("test_no_prog", "No Progress").show(
                ctx,
                |ui| {
                    ui.label("content");
                },
                |_ui| None::<()>,
            );
        });
    }
}
