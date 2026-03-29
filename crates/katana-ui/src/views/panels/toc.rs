#![allow(unused_imports, dead_code)]
use crate::app_state::{AppAction, AppState, ScrollSource, ViewMode};
use crate::i18n;
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, EDITOR_INITIAL_VISIBLE_ROWS, FILE_TREE_PANEL_DEFAULT_WIDTH,
    FILE_TREE_PANEL_MIN_WIDTH, NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_ITEM_SPACING,
    RECENT_WORKSPACES_SPACING, SCROLL_SYNC_DEAD_ZONE, TAB_DROP_ANIMATION_TIME,
    TAB_DROP_INDICATOR_WIDTH, TAB_INTER_ITEM_SPACING, TAB_NAV_BUTTONS_AREA_WIDTH,
    TAB_TOOLTIP_SHOW_DELAY_SECS, TREE_LABEL_HOFFSET, TREE_ROW_HEIGHT,
};
use crate::shell_ui::{
    indent_prefix, invisible_label, open_folder_dialog, relative_full_path, TreeRenderContext,
    LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG, PREVIEW_CONTENT_PADDING, SEARCH_MODAL_HEIGHT,
    SEARCH_MODAL_WIDTH, STATUS_BAR_ICON_SPACING, STATUS_SUCCESS_GREEN,
    TOC_HEADING_VISIBILITY_THRESHOLD, TOC_INDENT_PER_LEVEL, TOC_PANEL_DEFAULT_WIDTH,
    TOC_PANEL_MARGIN, WORKSPACE_SPINNER_INNER_MARGIN, WORKSPACE_SPINNER_OUTER_MARGIN,
    WORKSPACE_SPINNER_TEXT_MARGIN,
};
use crate::theme_bridge;
use crate::Icon;
use eframe::egui;
use katana_core::workspace::{TreeEntry, Workspace};
use std::path::{Path, PathBuf};

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
