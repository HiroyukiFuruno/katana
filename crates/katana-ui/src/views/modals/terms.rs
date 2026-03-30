
use crate::app_state::AppAction;
use crate::widgets::StyledComboBox;
use eframe::egui;

const TERMS_MODAL_WIDTH: f32 = 600.0;
const TERMS_TITLE_SIZE: f32 = 28.0;
const TERMS_INNER_MARGIN: f32 = 24.0;
const TERMS_CONVAS_MARGIN: f32 = 16.0;
const TERMS_ROUNDING_LARGE: f32 = 12.0;
const TERMS_ROUNDING_SMALL: f32 = 8.0;
const TERMS_SPACING_SMALL: f32 = 8.0;
const TERMS_SPACING_MEDIUM: f32 = 20.0;
const TERMS_SPACING_XLARGE: f32 = 32.0;
const TERMS_BUTTON_WIDTH: f32 = 120.0;
const TERMS_BUTTON_HEIGHT: f32 = 40.0;
const TERMS_BUTTON_TEXT_SIZE: f32 = 16.0;
const TERMS_BUTTON_SPACING: f32 = 24.0;
const TERMS_SCROLL_HEIGHT_RATIO: f32 = 0.5;
const TERMS_CENTER_OFFSET_RATIO: f32 = 0.1;
const TERMS_LANG_SELECT_WIDTH: f32 = 140.0;

pub(crate) struct TermsModal<'a> {
    pub version: &'a str,
    pub pending_action: &'a mut AppAction,
}

impl<'a> TermsModal<'a> {
    pub fn new(version: &'a str, pending_action: &'a mut AppAction) -> Self {
        Self {
            version,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        let version = self.version;
        let pending_action = self.pending_action;
        let terms = crate::i18n::get().terms.clone();

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
                let width = ui.available_width();
                let height = ui.available_height();
                let content_width = width.min(TERMS_MODAL_WIDTH);

                ui.vertical_centered(|ui| {
                    ui.add_space(height * TERMS_CENTER_OFFSET_RATIO);

                    ui.set_width(content_width);

                    egui::Frame::window(ui.style())
                        .inner_margin(TERMS_INNER_MARGIN)
                        .corner_radius(TERMS_ROUNDING_LARGE)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading(
                                    egui::RichText::new(&terms.title)
                                        .size(TERMS_TITLE_SIZE)
                                        .strong()
                                        .color(ui.visuals().strong_text_color()),
                                );
                                ui.add_space(TERMS_SPACING_SMALL);

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(crate::i18n::tf(
                                            &terms.version_label,
                                            &[("version", version)],
                                        ))
                                        .weak(),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let current_lang = crate::i18n::get_language();
                                            let current_name = crate::i18n::supported_languages()
                                                .iter()
                                                .find(|(code, _)| *code == current_lang)
                                                .map(|(_, name)| name.as_str())
                                                .unwrap_or("English");

                                            StyledComboBox::new("terms_lang_select", current_name)
                                                .width(TERMS_LANG_SELECT_WIDTH)
                                                .show(ui, |ui| {
                                                    for (code, name) in
                                                        crate::i18n::supported_languages()
                                                    {
                                                        if ui
                                                            .selectable_label(
                                                                current_lang == *code,
                                                                name,
                                                            )
                                                            .clicked()
                                                        {
                                                            *pending_action =
                                                                AppAction::ChangeLanguage(
                                                                    code.clone(),
                                                                );
                                                        }
                                                    }
                                                });
                                        },
                                    );
                                });

                                ui.add_space(TERMS_SPACING_MEDIUM);
                                ui.separator();
                                ui.add_space(TERMS_SPACING_MEDIUM);

                                egui::Frame::canvas(ui.style())
                                    .inner_margin(TERMS_CONVAS_MARGIN)
                                    .corner_radius(TERMS_ROUNDING_SMALL)
                                    .show(ui, |ui| {
                                        ui.set_min_height(
                                            ui.available_height() * TERMS_SCROLL_HEIGHT_RATIO,
                                        );
                                        egui::ScrollArea::vertical()
                                            .max_height(
                                                ui.available_height() * TERMS_SCROLL_HEIGHT_RATIO,
                                            )
                                            .show(ui, |ui| {
                                                ui.add(egui::Label::new(&terms.content).wrap());
                                            });
                                    });

                                ui.add_space(TERMS_SPACING_XLARGE);

                                ui.horizontal(|ui| {
                                    let total_buttons_width =
                                        TERMS_BUTTON_WIDTH * 2.0 + TERMS_BUTTON_SPACING;
                                    let available = ui.available_width();
                                    let outer_spacing = (available - total_buttons_width) / 2.0;

                                    if outer_spacing > 0.0 {
                                        ui.add_space(outer_spacing);
                                    }

                                    let accept_btn = egui::Button::new(
                                        egui::RichText::new(&terms.accept)
                                            .strong()
                                            .size(TERMS_BUTTON_TEXT_SIZE),
                                    )
                                    .min_size(egui::vec2(TERMS_BUTTON_WIDTH, TERMS_BUTTON_HEIGHT))
                                    .corner_radius(TERMS_ROUNDING_SMALL);

                                    if ui
                                        .add(accept_btn)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        *pending_action =
                                            AppAction::AcceptTerms(version.to_string());
                                    }

                                    ui.add_space(TERMS_BUTTON_SPACING);

                                    let decline_btn = egui::Button::new(
                                        egui::RichText::new(&terms.decline)
                                            .size(TERMS_BUTTON_TEXT_SIZE),
                                    )
                                    .min_size(egui::vec2(TERMS_BUTTON_WIDTH, TERMS_BUTTON_HEIGHT))
                                    .corner_radius(TERMS_ROUNDING_SMALL);

                                    if ui
                                        .add(decline_btn)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        *pending_action = AppAction::DeclineTerms;
                                    }
                                });
                                ui.add_space(TERMS_SPACING_MEDIUM);
                            });
                        });
                });
            });
    }
}