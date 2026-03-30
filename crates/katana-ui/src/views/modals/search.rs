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

pub(crate) struct SearchModal<'a> {
    pub search: &'a mut crate::app_state::SearchState,
    pub workspace: Option<&'a katana_core::workspace::Workspace>,
    pub is_open: &'a mut bool,
    pub action: &'a mut AppAction,
}

impl<'a> SearchModal<'a> {
    pub fn new(
        search: &'a mut crate::app_state::SearchState,
        workspace: Option<&'a katana_core::workspace::Workspace>,
        is_open: &'a mut bool,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            search,
            workspace,
            is_open,
            action,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        let search = self.search;
        let workspace = self.workspace;
        let action = self.action;
        let mut local_is_open = *self.is_open;
        egui::Window::new(crate::i18n::get().search.modal_title.clone())
            .open(&mut local_is_open)
            .collapsible(false)
            .resizable(true)
            .default_size(egui::vec2(SEARCH_MODAL_WIDTH, SEARCH_MODAL_HEIGHT))
            .show(ctx, |ui| {
                // Focus on the text edit automatically
                let response = ui.add(
                    egui::TextEdit::singleline(&mut search.query)
                        .hint_text(crate::i18n::get().search.query_hint.clone())
                        .desired_width(f32::INFINITY),
                );
                response.request_focus();

                let mut include_regexes = Vec::new();
                let mut include_valid = true;
                if !search.include_pattern.is_empty() {
                    for pat in search.include_pattern.split(',') {
                        let pat = pat.trim();
                        if !pat.is_empty() {
                            match regex::Regex::new(pat) {
                                Ok(re) => include_regexes.push(re),
                                Err(_) => include_valid = false,
                            }
                        }
                    }
                }

                let mut exclude_regexes = Vec::new();
                let mut exclude_valid = true;
                if !search.exclude_pattern.is_empty() {
                    for pat in search.exclude_pattern.split(',') {
                        let pat = pat.trim();
                        if !pat.is_empty() {
                            match regex::Regex::new(pat) {
                                Ok(re) => exclude_regexes.push(re),
                                Err(_) => exclude_valid = false,
                            }
                        }
                    }
                }

                let include_color = if include_valid {
                    ui.visuals().text_color()
                } else {
                    ui.ctx()
                        .data(|d| {
                            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                "katana_theme_colors",
                            ))
                        })
                        .map_or(crate::theme_bridge::WHITE, |tc| {
                            crate::theme_bridge::rgb_to_color32(tc.system.error_text)
                        })
                };

                let exclude_color = if exclude_valid {
                    ui.visuals().text_color()
                } else {
                    ui.ctx()
                        .data(|d| {
                            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                "katana_theme_colors",
                            ))
                        })
                        .map_or(crate::theme_bridge::WHITE, |tc| {
                            crate::theme_bridge::rgb_to_color32(tc.system.error_text)
                        })
                };

                ui.add(
                    egui::TextEdit::singleline(&mut search.include_pattern)
                        .hint_text(crate::i18n::get().search.include_pattern_hint.clone())
                        .text_color(include_color)
                        .desired_width(f32::INFINITY),
                );

                ui.add(
                    egui::TextEdit::singleline(&mut search.exclude_pattern)
                        .hint_text(crate::i18n::get().search.exclude_pattern_hint.clone())
                        .text_color(exclude_color)
                        .desired_width(f32::INFINITY),
                );

                let current_params = (
                    search.query.clone(),
                    search.include_pattern.clone(),
                    search.exclude_pattern.clone(),
                );

                if search.last_params.as_ref() != Some(&current_params) {
                    search.last_params = Some(current_params);

                    let query = search.query.to_lowercase();
                    if query.is_empty() && include_regexes.is_empty() && exclude_regexes.is_empty()
                    {
                        search.results.clear();
                    } else if let Some(ws) = workspace {
                        let mut results = Vec::new();
                        crate::shell_logic::collect_matches(
                            &ws.tree,
                            &query,
                            &include_regexes,
                            &exclude_regexes,
                            &ws.root,
                            &mut results,
                        );
                        search.results = results;
                    }
                }

                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if search.results.is_empty() && !search.query.is_empty() {
                            ui.label(crate::i18n::get().search.no_results.clone());
                        } else {
                            let ws_root = workspace.map(|ws| ws.root.clone());
                            for path in &search.results {
                                let rel = crate::shell_logic::relative_full_path(
                                    path,
                                    ws_root.as_deref(),
                                );
                                if ui.selectable_label(false, rel).clicked() && path.exists() {
                                    *action = AppAction::SelectDocument(path.to_path_buf());
                                }
                            }
                        }
                    });
            });

        *self.is_open = local_is_open;
    }
}
