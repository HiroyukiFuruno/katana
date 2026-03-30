
use eframe::egui;

const SPLASH_REPAINT_INTERVAL_MS: u64 = 32;

const SPLASH_BG_DARK: u8 = 30;
const SPLASH_BG_LIGHT: u8 = 240;
const SPLASH_ICON_SIZE: f32 = 128.0;
const SPLASH_ICON_SPACING: f32 = 16.0;
const SPLASH_HEADING_SIZE: f32 = 32.0;
const SPLASH_HEADING_SPACING: f32 = 8.0;
const SPLASH_VERSION_SIZE: f32 = 16.0;
const SPLASH_PROGRESS_SPACING: f32 = 24.0;
const SPLASH_PROGRESS_WIDTH: f32 = 240.0;
const SPLASH_PROGRESS_PHASE1: f32 = 0.25;
const SPLASH_PROGRESS_PHASE2: f32 = 0.6;
const SPLASH_PROGRESS_PHASE3: f32 = 0.95;
const SPLASH_PROGRESS_TEXT_SIZE: f32 = 12.0;
const SPLASH_PROGRESS_TEXT_DIM: f32 = 0.7;
const SPLASH_PROGRESS_BAR_MARGIN: f32 = 4.0;
const SPLASH_PROGRESS_BG_LIGHT: u8 = 100;
const SPLASH_PROGRESS_BG_DARK: u8 = 200;

const SPLASH_CONTENT_HEIGHT: f32 = SPLASH_ICON_SIZE
    + SPLASH_ICON_SPACING
    + SPLASH_HEADING_SIZE
    + SPLASH_HEADING_SPACING
    + SPLASH_VERSION_SIZE
    + SPLASH_PROGRESS_SPACING
    + SPLASH_PROGRESS_TEXT_SIZE
    + SPLASH_PROGRESS_BAR_MARGIN
    + SPLASH_PROGRESS_SPACING;

pub(crate) struct SplashOverlay<'a> {
    pub elapsed: f32,
    pub about_icon: Option<&'a egui::TextureHandle>,
}

impl<'a> SplashOverlay<'a> {
    pub fn new(elapsed: f32, about_icon: Option<&'a egui::TextureHandle>) -> Self {
        Self {
            elapsed,
            about_icon,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let elapsed = self.elapsed;
        let about_icon = self.about_icon;
        let opacity = crate::shell_logic::calculate_splash_opacity(elapsed);
        let any_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        if opacity <= 0.0 || any_pressed {
            return true; // WHY: splash dismissed
        }

        egui::Area::new(egui::Id::new("splash_screen_area"))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                let is_dark = ctx.style().visuals.dark_mode;
                #[allow(deprecated)]
                let content_rect = ctx.screen_rect();
                let bg_color = if is_dark {
                    crate::theme_bridge::from_rgb(SPLASH_BG_DARK, SPLASH_BG_DARK, SPLASH_BG_DARK)
                } else {
                    crate::theme_bridge::from_rgb(SPLASH_BG_LIGHT, SPLASH_BG_LIGHT, SPLASH_BG_LIGHT)
                };
                let fill_color = bg_color.gamma_multiply(opacity);
                ui.painter().rect_filled(content_rect, 1.0, fill_color);

                let text_color = if is_dark {
                    crate::theme_bridge::WHITE
                } else {
                    crate::theme_bridge::TRANSPARENT
                }
                .gamma_multiply(opacity);

                let center = content_rect.center();
                let centered_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(content_rect.width(), SPLASH_CONTENT_HEIGHT),
                );

                ui.scope_builder(egui::UiBuilder::new().max_rect(centered_rect), |ui| {
                    ui.vertical_centered(|ui| {
                        if let Some(tex) = about_icon {
                            ui.image(egui::load::SizedTexture::new(
                                tex.id(),
                                egui::vec2(SPLASH_ICON_SIZE, SPLASH_ICON_SIZE),
                            ));
                            ui.add_space(SPLASH_ICON_SPACING);
                        }
                        let heading = egui::RichText::new(crate::about_info::APP_DISPLAY_NAME)
                            .strong()
                            .size(SPLASH_HEADING_SIZE)
                            .color(text_color);
                        ui.label(heading);

                        ui.add_space(SPLASH_HEADING_SPACING);

                        let version_str = format!("Version {}", env!("CARGO_PKG_VERSION"));
                        let version = egui::RichText::new(version_str)
                            .size(SPLASH_VERSION_SIZE)
                            .color(text_color);
                        ui.label(version);

                        ui.add_space(SPLASH_PROGRESS_SPACING);
                        let progress = crate::shell_logic::calculate_splash_progress(elapsed);

                        let progress_text = if progress < SPLASH_PROGRESS_PHASE1 {
                            "Initializing Katana engine..."
                        } else if progress < SPLASH_PROGRESS_PHASE2 {
                            "Parsing workspace structure..."
                        } else if progress < SPLASH_PROGRESS_PHASE3 {
                            "Increasing context size... w"
                        } else {
                            "Ready."
                        };

                        ui.label(
                            egui::RichText::new(progress_text)
                                .size(SPLASH_PROGRESS_TEXT_SIZE)
                                .color(text_color.gamma_multiply(SPLASH_PROGRESS_TEXT_DIM)),
                        );
                        ui.add_space(SPLASH_PROGRESS_BAR_MARGIN);
                        let progress_bar = egui::ProgressBar::new(progress)
                            .desired_width(SPLASH_PROGRESS_WIDTH)
                            .show_percentage();

                        if !is_dark {
                            ui.visuals_mut().selection.bg_fill = crate::theme_bridge::from_rgb(
                                SPLASH_PROGRESS_BG_LIGHT,
                                SPLASH_PROGRESS_BG_LIGHT,
                                SPLASH_PROGRESS_BG_LIGHT,
                            )
                            .gamma_multiply(opacity);
                        } else {
                            ui.visuals_mut().selection.bg_fill = crate::theme_bridge::from_rgb(
                                SPLASH_PROGRESS_BG_DARK,
                                SPLASH_PROGRESS_BG_DARK,
                                SPLASH_PROGRESS_BG_DARK,
                            )
                            .gamma_multiply(opacity);
                        }
                        ui.add(progress_bar);
                    });
                });
            });

        ctx.request_repaint_after(std::time::Duration::from_millis(SPLASH_REPAINT_INTERVAL_MS));

        false // WHY: splash still active
    }
}