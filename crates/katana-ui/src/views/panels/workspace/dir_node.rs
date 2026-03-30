use super::tree_node::TreeEntryNode;
use crate::shell::{TREE_LABEL_HOFFSET, TREE_ROW_HEIGHT};
use crate::shell_ui::{indent_prefix, TreeRenderContext};
use eframe::egui;

pub(crate) struct DirectoryEntryNode<'a, 'b, 'c> {
    pub path: &'a std::path::Path,
    pub children: &'a [katana_core::workspace::TreeEntry],
    pub ctx: &'a mut TreeRenderContext<'b, 'c>,
}

impl<'a, 'b, 'c> DirectoryEntryNode<'a, 'b, 'c> {
    pub fn new(
        path: &'a std::path::Path,
        children: &'a [katana_core::workspace::TreeEntry],
        ctx: &'a mut TreeRenderContext<'b, 'c>,
    ) -> Self {
        Self {
            path,
            children,
            ctx,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let path = self.path;
        let children = self.children;
        let ctx = self.ctx;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let id = ui.make_persistent_id(format!("dir:{}", path.display()));

        let is_open = ctx.expanded_directories.contains(path);

        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, is_open);
        state.set_open(is_open);
        let file_tree_color = ui.visuals().text_color();
        let (rect, mut resp) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), TREE_ROW_HEIGHT),
            egui::Sense::click(),
        );
        resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

        let accessible_label = format!("dir {}", name);
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &accessible_label)
        });

        if resp.clicked() {
            if is_open {
                ctx.expanded_directories.remove(path);
            } else {
                ctx.expanded_directories.insert(path.to_path_buf());
            }
        }

        if ui.is_rect_visible(rect) {
            if ui.rect_contains_pointer(rect) && ui.is_enabled() {
                ui.painter()
                    .rect_filled(rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
            }

            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            child_ui.add_space(TREE_LABEL_HOFFSET);
            let prefix = indent_prefix(ctx.depth);
            let arrow_icon = if is_open {
                crate::icon::Icon::PanDown
            } else {
                crate::icon::Icon::PanRight
            };
            let folder_icon = if is_open {
                crate::icon::Icon::FolderOpen
            } else {
                crate::icon::Icon::FolderClosed
            };

            child_ui.add(egui::Label::new(prefix).selectable(false));

            child_ui.add(
                arrow_icon
                    .image(crate::icon::IconSize::Small)
                    .tint(file_tree_color),
            );
            child_ui.add(
                folder_icon
                    .image(crate::icon::IconSize::Medium)
                    .tint(file_tree_color),
            );
            child_ui.add(
                egui::Label::new(egui::RichText::new(name).color(file_tree_color))
                    .selectable(false)
                    .truncate(),
            );
        }

        if !ctx.disable_context_menu {
            resp.context_menu(|ui| {
                crate::views::panels::tree::TreeContextMenu::new(
                    path,
                    true,
                    Some(children),
                    None,
                    ctx,
                )
                .show(ui);
            });
        }

        if resp.clicked() {
            let new_state = !is_open;
            state.set_open(new_state);
            if new_state {
                ctx.expanded_directories.insert(path.to_path_buf());
            } else {
                ctx.expanded_directories.remove(path);
            }
        }
        state.store(ui.ctx());

        if state.is_open() {
            let prev_depth = ctx.depth;
            ctx.depth += 1;
            for child in children {
                TreeEntryNode::new(child, ctx).show(ui);
            }
            ctx.depth = prev_depth;
        }
    }
}
