use super::dir_node::DirectoryEntryNode;
use super::file_node::FileEntryNode;
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct TreeEntryNode<'a, 'b, 'c> {
    pub entry: &'a katana_core::workspace::TreeEntry,
    pub ctx: &'a mut TreeRenderContext<'b, 'c>,
}

impl<'a, 'b, 'c> TreeEntryNode<'a, 'b, 'c> {
    pub fn new(
        entry: &'a katana_core::workspace::TreeEntry,
        ctx: &'a mut TreeRenderContext<'b, 'c>,
    ) -> Self {
        Self { entry, ctx }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let entry = self.entry;
        let ctx = self.ctx;
        use katana_core::workspace::TreeEntry;
        let entry_path = match entry {
            TreeEntry::Directory { path, .. } => path,
            TreeEntry::File { path } => path,
        };
        if let Some(fs) = ctx.filter_set {
            if !fs.contains(entry_path) {
                return;
            }
        }
        match entry {
            TreeEntry::Directory { path, children } => {
                DirectoryEntryNode::new(path, children, ctx).show(ui);
            }
            TreeEntry::File { path } => {
                FileEntryNode::new(entry, path, ctx).show(ui);
            }
        }
    }
}
