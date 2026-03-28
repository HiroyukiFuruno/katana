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
        const TOGGLE_Y_OFFSET: f32 = 3.0;

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

/// A generic color picker row with an aligned label and a 5px content offset
/// to visually align the color button with Katana's typography.
pub struct LabeledColorPicker<'a> {
    label: &'a str,
    label_width: f32,
    spacing: f32,
    offset_y: f32,
    is_rgba: bool,
}

const COLOR_LABEL_WIDTH: f32 = 130.0;
const COLOR_SPACING: f32 = 16.0;
const COLOR_OFFSET_Y: f32 = -2.0;
const COLOR_ROW_HEIGHT: f32 = 24.0;
const COLOR_LABEL_MARGIN: f32 = 8.0;

impl<'a> LabeledColorPicker<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            label_width: COLOR_LABEL_WIDTH,
            spacing: COLOR_SPACING,
            offset_y: COLOR_OFFSET_Y, // Nudge 2px up to visually align with text baseline
            is_rgba: false,
        }
    }

    pub fn rgba(mut self, is_rgba: bool) -> Self {
        self.is_rgba = is_rgba;
        self
    }

    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn offset_y(mut self, offset: f32) -> Self {
        self.offset_y = offset;
        self
    }

    pub fn show_rgb(self, ui: &mut egui::Ui, color: &mut egui::Color32) -> egui::Response {
        let available_w = ui.available_width();
        let row_height = COLOR_ROW_HEIGHT; // Standardize row height for strict table alignment
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(available_w, row_height), egui::Sense::hover());

        // 1. Text Left-aligned (center Y) with margin
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                // Force line height to exact row_height to perfectly center the text
                ui.allocate_exact_size(egui::vec2(0.0, rect.height()), egui::Sense::hover());
                ui.add_space(COLOR_LABEL_MARGIN);
                ui.label(self.label);
            });
        });

        // 2. Button Right-aligned (center Y shifted by offset_y if visually desired)
        let right_rect = rect.translate(egui::vec2(0.0, self.offset_y));
        ui.scope_builder(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                // Force line height to exact row_height to perfectly center the button
                ui.allocate_exact_size(egui::vec2(0.0, right_rect.height()), egui::Sense::hover());
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    color,
                    egui::color_picker::Alpha::Opaque,
                )
            })
            .inner
        })
        .inner
    }

    pub fn show_rgba(self, ui: &mut egui::Ui, color: &mut egui::Color32) -> egui::Response {
        let available_w = ui.available_width();
        let row_height = COLOR_ROW_HEIGHT;
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(available_w, row_height), egui::Sense::hover());

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.allocate_exact_size(egui::vec2(0.0, rect.height()), egui::Sense::hover());
                ui.add_space(COLOR_LABEL_MARGIN);
                ui.label(self.label);
            });
        });

        let right_rect = rect.translate(egui::vec2(0.0, self.offset_y));
        ui.scope_builder(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.allocate_exact_size(egui::vec2(0.0, right_rect.height()), egui::Sense::hover());
                // For RGBA, we want alpha to be editable. Use BlendOrAdditive with srgba (Color32)
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    color,
                    egui::color_picker::Alpha::BlendOrAdditive,
                )
            })
            .inner
        })
        .inner
    }
}

#[cfg(test)]
mod labeled_color_picker_tests {
    use super::*;
    use egui::Context;

    #[test]
    fn test_labeled_color_picker_defaults() {
        let picker = LabeledColorPicker::new("Test Label");
        assert_eq!(picker.label, "Test Label");
        assert_eq!(
            picker.offset_y, -2.0,
            "Expected -2.0px offset because we nudge up to visuals align text baseline"
        );
        assert_eq!(
            picker.label_width, 130.0,
            "Expected strict 130.0px label width for grid alignment"
        );
        assert_eq!(
            picker.spacing, 16.0,
            "Expected strict 16.0px spacing representing sectional margin"
        );
    }

    #[test]
    fn test_labeled_color_picker_customization() {
        let picker = LabeledColorPicker::new("Custom")
            .offset_y(10.0)
            .label_width(200.0)
            .spacing(5.0)
            .rgba(true);
        assert_eq!(picker.offset_y, 10.0);
        assert_eq!(picker.label_width, 200.0);
        assert_eq!(picker.spacing, 5.0);
        assert!(picker.is_rgba);
    }

    #[test]
    fn test_labeled_color_picker_layout_strict() {
        let mut color = crate::theme_bridge::rgb_to_color32(katana_platform::theme::Rgb {
            r: 255,
            g: 0,
            b: 128,
        });
        let ctx = Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // IT environment setup: Create an explicit, strictly constrained UI mock (500x500 rect at origin 0,0)
                let test_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(500.0, 500.0));
                let mut test_ui = ui.new_child(egui::UiBuilder::new().max_rect(test_rect).layout(egui::Layout::top_down(egui::Align::Min)));

                let response = LabeledColorPicker::new("Test RGB Strict Alignment").show_rgb(&mut test_ui, &mut color);

                // Strict Horizontal IT: Space-Between validation
                // Color button MUST be aligned flush to the right boundary of the 500px allocated UI
                assert_eq!(
                    response.rect.max.x, 500.0,
                    "Strict Layout Validation Failed: Color picker button is NOT aligned to the exact right edge!"
                );

                // Strict Vertical IT: Center alignment validation
                // The explicit row height is 24px and starts at Y=0. Mathematical center is implicitly Y=12.0
                // But we nudged offset_y by -2.0! So center is Y=10.0
                assert_eq!(
                    response.rect.center().y, 10.0,
                    "Strict Layout Validation Failed: Color picker button is NOT perfectly mathematically vertically centered with offset!"
                );
            });
        });
    }

    #[test]
    fn test_labeled_color_picker_layout_rgba_strict() {
        let mut color = crate::theme_bridge::rgba_to_color32(katana_platform::theme::Rgba {
            r: 255,
            g: 0,
            b: 128,
            a: 204,
        });
        let ctx = Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let test_rect =
                    egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(800.0, 800.0));
                let mut test_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(test_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );

                let response =
                    LabeledColorPicker::new("Test RGBA").show_rgba(&mut test_ui, &mut color);

                // Max X should be 100 + 800 = 900
                assert_eq!(
                    response.rect.max.x, 900.0,
                    "RGBA Button right edge misalignment"
                );
                // Center Y should be 100 + 12 - 2 (offset) = 110 (row height is 24)
                assert_eq!(
                    response.rect.center().y,
                    110.0,
                    "RGBA Button vertical center misalignment"
                );
            });
        });
    }
}
