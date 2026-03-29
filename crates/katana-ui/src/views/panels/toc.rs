use crate::shell_ui::{
    TOC_HEADING_VISIBILITY_THRESHOLD, TOC_INDENT_PER_LEVEL, TOC_PANEL_DEFAULT_WIDTH,
    TOC_PANEL_MARGIN,
};
use eframe::egui;

pub(crate) fn render_toc_panel(
    ctx: &egui::Context,
    preview: &mut crate::preview_pane::PreviewPane,
    state: &crate::app_state::AppState,
) {
    use katana_platform::settings::TocPosition;
    let position = state.config.settings.settings().layout.toc_position;

    let panel = match position {
        TocPosition::Left => egui::SidePanel::left("toc_panel"),
        TocPosition::Right => egui::SidePanel::right("toc_panel"),
    };

    let frame = egui::Frame::side_top_panel(&ctx.style()).inner_margin(TOC_PANEL_MARGIN);

    panel
        .frame(frame)
        .resizable(true)
        .default_width(TOC_PANEL_DEFAULT_WIDTH)
        .show(ctx, |ui| {
            ui.heading(crate::i18n::get().toc.title.clone());
            ui.separator();

            // Prevent text from wrapping or pushing the SidePanel width. Text will truncate with `...`
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    if preview.outline_items.is_empty() {
                        ui.label(
                            egui::RichText::new(crate::i18n::get().toc.empty.clone())
                                .weak()
                                .italics(),
                        );
                    } else {
                        let mut active_index = 0;
                        if let Some(visible_rect) = preview.visible_rect {
                            let threshold = visible_rect.min.y + TOC_HEADING_VISIBILITY_THRESHOLD;
                            for (i, (_, rect)) in preview.heading_anchors.iter().enumerate() {
                                if rect.min.y <= threshold {
                                    active_index = i;
                                } else {
                                    break;
                                }
                            }
                        }

                        let mut next_scroll = None;
                        for (i, item) in preview.outline_items.iter().enumerate() {
                            let indent = (item.level as f32 - 1.0) * TOC_INDENT_PER_LEVEL;
                            ui.horizontal(|ui| {
                                ui.add_space(indent);
                                let is_active = i == active_index;
                                let mut text = egui::RichText::new(&item.text);
                                if is_active {
                                    text = text
                                        .strong()
                                        .color(ui.visuals().widgets.active.text_color());
                                }
                                if ui.selectable_label(is_active, text).clicked() {
                                    next_scroll = Some(item.index);
                                }
                            });
                        }
                        if next_scroll.is_some() {
                            preview.scroll_request = next_scroll;
                        }
                    }
                });
        });
}
