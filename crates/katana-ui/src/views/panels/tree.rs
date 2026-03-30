use crate::shell_ui::TreeRenderContext;

pub(crate) struct TreeContextMenu<'a, 'b, 'c> {
    pub path: &'a std::path::Path,
    pub is_dir: bool,
    pub children: Option<&'a [katana_core::workspace::TreeEntry]>,
    pub entry: Option<&'a katana_core::workspace::TreeEntry>,
    pub ctx: &'a mut TreeRenderContext<'b, 'c>,
}

impl<'a, 'b, 'c> TreeContextMenu<'a, 'b, 'c> {
    pub fn new(
        path: &'a std::path::Path,
        is_dir: bool,
        children: Option<&'a [katana_core::workspace::TreeEntry]>,
        entry: Option<&'a katana_core::workspace::TreeEntry>,
        ctx: &'a mut TreeRenderContext<'b, 'c>,
    ) -> Self {
        Self {
            path,
            is_dir,
            children,
            entry,
            ctx,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let path = self.path;
        let is_dir = self.is_dir;
        let children = self.children;
        let entry = self.entry;
        let ctx = self.ctx;
        let msg = &crate::i18n::get().action;

        let target_dir = if is_dir {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };

        if is_dir {
            if ui.button(msg.new_file.clone()).clicked() {
                *ctx.action = crate::app_state::AppAction::RequestNewFile(target_dir.clone());
                ui.close();
            }
            if ui.button(msg.new_directory.clone()).clicked() {
                *ctx.action = crate::app_state::AppAction::RequestNewDirectory(target_dir);
                ui.close();
            }
            ui.separator();
        }

        if is_dir {
            if let Some(children) = children {
                if ui.button(msg.recursive_expand.clone()).clicked() {
                    let mut to_expand = Vec::new();
                    for child in children {
                        child.collect_all_directory_paths(&mut to_expand);
                    }
                    ctx.expanded_directories.insert(path.to_path_buf());
                    ctx.expanded_directories.extend(to_expand);
                    ui.close();
                }
                if ui.button(msg.recursive_open_all.clone()).clicked() {
                    let mut to_open = Vec::new();
                    for child in children {
                        child.collect_all_markdown_file_paths(&mut to_open);
                    }
                    if !to_open.is_empty() {
                        *ctx.action = crate::app_state::AppAction::OpenMultipleDocuments(to_open);
                    }
                    ui.close();
                }
            }
        } else if entry.is_some() {
            #[allow(clippy::collapsible_if)]
            if ui.button(msg.open.clone()).clicked() {
                *ctx.action = crate::app_state::AppAction::SelectDocument(path.to_path_buf());
                ui.close();
            }
        }

        ui.separator();

        if ui.button(msg.reveal_in_os.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::RevealInOs(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.copy_path.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::CopyPathToClipboard(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.copy_relative_path.clone()).clicked() {
            *ctx.action =
                crate::app_state::AppAction::CopyRelativePathToClipboard(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.show_meta_info.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::ShowMetaInfo(path.to_path_buf());
            ui.close();
        }

        ui.separator();

        if ui.button(msg.rename.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::RequestRename(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.delete.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::RequestDelete(path.to_path_buf());
            ui.close();
        }
    }
}

pub(crate) fn gather_visible_paths(
    entries: &[katana_core::workspace::TreeEntry],
    regex: &regex::Regex,
    is_negated: bool,
    ws_root: &std::path::Path,
    visible: &mut std::collections::HashSet<std::path::PathBuf>,
) -> bool {
    let mut any_visible = false;
    for entry in entries {
        match entry {
            katana_core::workspace::TreeEntry::File { path } => {
                let rel = crate::shell_logic::relative_full_path(path, Some(ws_root));
                let is_match = regex.is_match(&rel);
                let should_show = if is_negated { !is_match } else { is_match };

                if should_show {
                    visible.insert(path.clone());
                    any_visible = true;
                }
            }
            katana_core::workspace::TreeEntry::Directory { path, children } => {
                if gather_visible_paths(children, regex, is_negated, ws_root, visible) {
                    visible.insert(path.clone());
                    any_visible = true;
                }
            }
        }
    }
    any_visible
}

pub(crate) fn find_node_in_tree<'a>(
    entries: &'a [katana_core::workspace::TreeEntry],
    target: &std::path::Path,
) -> Option<&'a katana_core::workspace::TreeEntry> {
    for entry in entries {
        match entry {
            katana_core::workspace::TreeEntry::Directory { path, children } => {
                if path == target {
                    return Some(entry);
                }
                if target.starts_with(path) {
                    if let Some(found) =
                        crate::views::panels::tree::find_node_in_tree(children, target)
                    {
                        return Some(found);
                    }
                }
            }
            katana_core::workspace::TreeEntry::File { path } => {
                if path == target {
                    return Some(entry);
                }
            }
        }
    }
    None
}