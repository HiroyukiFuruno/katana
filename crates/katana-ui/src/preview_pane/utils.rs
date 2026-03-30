use eframe::egui::{self};

pub(crate) fn open_tab(ctx: &egui::Context, url: &str) {
    ctx.open_url(egui::OpenUrl::new_tab(url));
}

pub(crate) fn with_preview_text_style<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.scope(|ui| {
        let fonts_loaded = ui.ctx().data(|d| {
            d.get_temp::<bool>(egui::Id::new("katana_fonts_loaded"))
                .unwrap_or(false)
        });
        if fonts_loaded {
            set_preview_body_family(ui, egui::FontFamily::Name("MarkdownProportional".into()));
        } else {
            set_preview_body_family(ui, egui::FontFamily::Proportional);
        }

        if let Some(color) = ui.ctx().data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        }) {
            ui.visuals_mut().override_text_color =
                Some(crate::theme_bridge::rgb_to_color32(color.preview.text));
            ui.visuals_mut().selection.bg_fill =
                crate::theme_bridge::rgb_to_color32(color.preview.selection);
        }

        add_contents(ui)
    })
    .inner
}

pub(crate) fn set_preview_body_family(ui: &mut egui::Ui, family: egui::FontFamily) {
    let style = ui.style_mut();
    style.override_font_id = None;
    style.override_text_style = None;
    for text_style in [
        egui::TextStyle::Body,
        egui::TextStyle::Button,
        egui::TextStyle::Heading,
        egui::TextStyle::Small,
    ] {
        if let Some(font_id) = style.text_styles.get_mut(&text_style) {
            font_id.family = family.clone();
        }
    }
}