use super::tree_node::TreeEntryNode;
use crate::app_state::AppAction;
use crate::shell::{
    NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_ITEM_SPACING, RECENT_WORKSPACES_SPACING,
};
use crate::shell_ui::{open_folder_dialog, TreeRenderContext};
use eframe::egui;

pub(crate) struct WorkspaceContent<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub recent_paths: &'a [String],
    pub active_path: Option<&'a std::path::Path>,
    pub action: &'a mut AppAction,
}

impl<'a> WorkspaceContent<'a> {
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
        if let Some(ws) = &workspace.data {
            let entries = ws.tree.clone();
            if let Some(force) = workspace.force_tree_open {
                if force {
                    workspace
                        .expanded_directories
                        .extend(ws.collect_all_directory_paths());
                } else {
                    workspace.expanded_directories.clear();
                }
            }

            let ws_root = ws.root.clone();
            if search.filter_enabled && !search.filter_query.is_empty() {
                let is_negated = search.filter_query.starts_with('!');
                let query_str = if is_negated {
                    &search.filter_query[1..]
                } else {
                    &search.filter_query
                };

                match regex::Regex::new(query_str) {
                    Ok(regex) => {
                        if search.filter_cache.as_ref().map(|(q, _)| q)
                            != Some(&search.filter_query)
                        {
                            let mut visible = std::collections::HashSet::new();
                            crate::views::panels::tree::gather_visible_paths(
                                &entries,
                                &regex,
                                is_negated,
                                &ws_root,
                                &mut visible,
                            );
                            search.filter_cache = Some((search.filter_query.clone(), visible));
                        }
                    }
                    Err(_) => {
                        search.filter_cache = None;
                    }
                }
            } else {
                search.filter_cache = None;
            }
            let filter_set = search.filter_cache.as_ref().map(|(_, v)| v);

            egui::ScrollArea::vertical()
                .id_salt("workspace_tree_scroll")
                .show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    let mut ctx = TreeRenderContext {
                        action,
                        depth: 0,
                        active_path,
                        filter_set,
                        expanded_directories: &mut workspace.expanded_directories,
                        disable_context_menu: false,
                    };
                    for entry in &entries {
                        TreeEntryNode::new(entry, &mut ctx).show(ui);
                    }
                });
            workspace.force_tree_open = None;
        } else {
            ui.label(crate::i18n::get().workspace.no_workspace_open.clone());
            ui.add_space(NO_WORKSPACE_BOTTOM_SPACING);
            if ui
                .button(crate::i18n::get().menu.open_workspace.clone())
                .clicked()
            {
                if let Some(path) = open_folder_dialog() {
                    *action = AppAction::OpenWorkspace(path);
                }
            }

            if !recent_paths.is_empty() {
                ui.add_space(RECENT_WORKSPACES_SPACING);
                ui.separator();
                ui.add_space(RECENT_WORKSPACES_SPACING);
                ui.heading(crate::i18n::get().workspace.recent_workspaces.clone());
                ui.add_space(RECENT_WORKSPACES_ITEM_SPACING);
                for path in recent_paths.iter().rev() {
                    ui.horizontal(|ui| {
                        if ui
                            .button("×")
                            .on_hover_text(crate::i18n::get().action.remove_workspace.clone())
                            .clicked()
                        {
                            *action = AppAction::RemoveWorkspace(path.clone());
                        }
                        if ui.selectable_label(false, path).clicked() {
                            *action = AppAction::OpenWorkspace(std::path::PathBuf::from(path));
                        }
                    });
                }
            }
        }
    }
}
