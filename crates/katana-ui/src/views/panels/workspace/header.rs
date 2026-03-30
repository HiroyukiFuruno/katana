use crate::app_state::AppAction;
use crate::shell_ui::{invisible_label, LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG};
use eframe::egui;

pub(crate) struct WorkspaceHeader<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub recent_paths: &'a [String],
    pub action: &'a mut AppAction,
}

impl<'a> WorkspaceHeader<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        recent_paths: &'a [String],
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            recent_paths,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let (workspace, search, recent_paths, action) =
            (self.workspace, self.search, self.recent_paths, self.action);

        ui.horizontal(|ui| {
            ui.heading(crate::i18n::get().workspace.workspace_title.clone());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(egui::Button::image(
                        crate::icon::Icon::ChevronLeft.ui_image(ui, crate::icon::IconSize::Small),
                    ))
                    .on_hover_text(crate::i18n::get().action.collapse_sidebar.clone())
                    .clicked()
                {
                    *action = AppAction::ToggleWorkspace;
                }
            });
        });
        if workspace.data.is_some() {
            ui.horizontal(|ui| {
                let btn_resp = ui
                    .add(egui::Button::image(
                        crate::Icon::ExpandAll.ui_image(ui, crate::icon::IconSize::Small),
                    ))
                    .on_hover_text(crate::i18n::get().action.expand_all.clone());
                btn_resp
                    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "+"));
                if btn_resp.clicked() {
                    if let Some(ws) = &workspace.data {
                        workspace
                            .expanded_directories
                            .extend(ws.collect_all_directory_paths());
                    }
                }
                let btn_resp = ui
                    .add(egui::Button::image(
                        crate::Icon::CollapseAll.ui_image(ui, crate::icon::IconSize::Small),
                    ))
                    .on_hover_text(crate::i18n::get().action.collapse_all.clone());
                btn_resp
                    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "-"));
                if btn_resp.clicked() {
                    workspace.force_tree_open = Some(false);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let icon_bg = if ui.visuals().dark_mode {
                        crate::theme_bridge::TRANSPARENT
                    } else {
                        crate::theme_bridge::from_gray(LIGHT_MODE_ICON_BG)
                    };

                    if !recent_paths.is_empty() {
                        let ws_history_img =
                            crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Small);
                        ui.scope(|ui| {
                            ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;
                            ui.menu_image_button(ws_history_img, |ui| {
                                for path in recent_paths.iter().rev() {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add(egui::Button::image_and_text(
                                                crate::Icon::Remove
                                                    .ui_image(ui, crate::icon::IconSize::Small),
                                                invisible_label("x"),
                                            ))
                                            .on_hover_text(
                                                crate::i18n::get().action.remove_workspace.clone(),
                                            )
                                            .clicked()
                                        {
                                            *action = AppAction::RemoveWorkspace(path.clone());
                                            ui.close();
                                        }
                                        if ui.selectable_label(false, path).clicked() {
                                            *action = AppAction::OpenWorkspace(
                                                std::path::PathBuf::from(path),
                                            );
                                            ui.close();
                                        }
                                    });
                                }
                            })
                            .response
                            .on_hover_text(crate::i18n::get().workspace.recent_workspaces.clone());
                        });
                    }

                    if ui
                        .add(
                            egui::Button::image_and_text(
                                crate::Icon::Refresh.ui_image(ui, crate::icon::IconSize::Small),
                                invisible_label("🔄"),
                            )
                            .fill(icon_bg),
                        )
                        .on_hover_text(crate::i18n::get().action.refresh_workspace.clone())
                        .clicked()
                    {
                        *action = AppAction::RefreshWorkspace;
                    }

                    let filter_btn_color = if search.filter_enabled {
                        if ui.visuals().dark_mode {
                            ui.visuals().selection.bg_fill
                        } else {
                            crate::theme_bridge::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
                        }
                    } else {
                        icon_bg
                    };

                    if ui
                        .add(
                            egui::Button::image_and_text(
                                crate::Icon::Filter.ui_image(ui, crate::icon::IconSize::Small),
                                invisible_label("∇"),
                            )
                            .fill(filter_btn_color),
                        )
                        .on_hover_text(crate::i18n::get().action.toggle_filter.clone())
                        .clicked()
                    {
                        *action = AppAction::ToggleWorkspaceFilter;
                    }

                    if ui
                        .add(
                            egui::Button::image_and_text(
                                crate::Icon::Search.ui_image(ui, crate::icon::IconSize::Small),
                                invisible_label("🔍"),
                            )
                            .fill(icon_bg),
                        )
                        .on_hover_text(crate::i18n::get().search.modal_title.clone())
                        .clicked()
                    {
                        *action = AppAction::ToggleSearchModal;
                    }
                });
            });

            if search.filter_enabled {
                let mut is_valid_regex = true;
                if !search.filter_query.is_empty() {
                    is_valid_regex = regex::Regex::new(&search.filter_query).is_ok();
                }
                ui.horizontal(|ui| {
                    let text_color = if is_valid_regex {
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
                        egui::TextEdit::singleline(&mut search.filter_query)
                            .text_color(text_color)
                            .hint_text("Filter (Regex)...")
                            .desired_width(f32::INFINITY),
                    );
                });
            }
        }
    }
}
