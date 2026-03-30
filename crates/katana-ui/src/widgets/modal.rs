
pub struct Modal<'a> {
    id: &'a str,
    title: &'a str,
    progress: Option<f32>,
    show_pct: bool,
    bar_width: f32,
    width: Option<f32>,
}

pub(crate) const DEFAULT_BAR_WIDTH: f32 = 280.0;
const DEFAULT_DIALOG_WIDTH: f32 = 450.0;
const BODY_TO_BAR_SPACING: f32 = 12.0;
const BAR_TO_FOOTER_SPACING: f32 = 16.0;

impl<'a> Modal<'a> {
    pub fn new(id: &'a str, title: &'a str) -> Self {
        Self {
            id,
            title,
            progress: None,
            show_pct: false,
            bar_width: DEFAULT_BAR_WIDTH,
            width: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn progress(mut self, ratio: f32) -> Self {
        self.progress = Some(ratio.clamp(0.0, 1.0));
        self
    }

    pub fn maybe_progress(mut self, ratio: Option<f32>) -> Self {
        self.progress = ratio.map(|r| r.clamp(0.0, 1.0));
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_pct = show;
        self
    }

    pub fn bar_width(mut self, width: f32) -> Self {
        self.bar_width = width;
        self
    }

    pub fn show<T>(
        self,
        ctx: &egui::Context,
        body: impl FnOnce(&mut egui::Ui),
        footer: impl FnOnce(&mut egui::Ui) -> Option<T>,
    ) -> Option<T> {
        let mut result: Option<T> = None;

        let dialog_width = self.width.unwrap_or(DEFAULT_DIALOG_WIDTH);

        egui::Window::new(self.title)
            .id(egui::Id::new(self.id))
            .collapsible(false)
            .resizable(false)
            .default_width(dialog_width)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_max_width(dialog_width);

                ui.vertical_centered(|ui| {
                    ui.set_max_width(dialog_width);
                    body(ui);

                    if let Some(ratio) = self.progress {
                        ui.add_space(BODY_TO_BAR_SPACING);
                        let mut bar = egui::ProgressBar::new(ratio).desired_width(self.bar_width);
                        if self.show_pct {
                            bar = bar.show_percentage();
                        }
                        ui.add(bar);
                    }
                });

                ui.add_space(BAR_TO_FOOTER_SPACING);
                ui.horizontal(|ui| {
                    ui.set_max_width(dialog_width);
                    result = footer(ui);
                });
            });

        result
    }

    pub fn show_body_only(self, ctx: &egui::Context, body: impl FnOnce(&mut egui::Ui)) {
        self.show(ctx, body, |_ui| None::<()>);
    }
}

#[cfg(test)]
mod tests {
    use super::*;


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