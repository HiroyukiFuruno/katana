use super::content::WorkspaceContent;
use super::header::WorkspaceHeader;
use crate::app_state::AppAction;
use crate::shell_ui::{
    WORKSPACE_SPINNER_INNER_MARGIN, WORKSPACE_SPINNER_OUTER_MARGIN, WORKSPACE_SPINNER_TEXT_MARGIN,
};
use eframe::egui;

pub(crate) struct WorkspacePanel<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub recent_paths: &'a [String],
    pub active_path: Option<&'a std::path::Path>,
    pub action: &'a mut AppAction,
}

impl<'a> WorkspacePanel<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        recent_paths: &'a [String],
        active_path: Option<&'a std::path::Path>,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            recent_paths,
            active_path,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let workspace = self.workspace;
        let search = self.search;
        let recent_paths = self.recent_paths;
        let active_path = self.active_path;
        let action = self.action;
        let panel_width = ui.available_width();
        ui.set_max_width(panel_width);
        ui.set_min_width(panel_width);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

        let is_loading = workspace.is_loading;

        WorkspaceHeader::new(workspace, search, recent_paths, action).show(ui);

        ui.separator();

        if is_loading {
            ui.add_space(WORKSPACE_SPINNER_OUTER_MARGIN);
            ui.horizontal(|ui| {
                ui.add_space(WORKSPACE_SPINNER_INNER_MARGIN);
                ui.spinner();
                ui.add_space(WORKSPACE_SPINNER_TEXT_MARGIN);
                ui.label(crate::i18n::get().action.refresh_workspace.clone());
            });
        } else {
            WorkspaceContent::new(workspace, search, recent_paths, active_path, action).show(ui);
        }
    }
}
