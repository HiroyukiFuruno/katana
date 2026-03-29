#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app_state::{AppAction, AppState};
use crate::shell::KatanaApp;
use crate::state::update::UpdatePhase;
use crate::Icon;
use katana_core::update::ReleaseInfo;

use crate::i18n;
use egui::{Align, Layout};
use std::path::{Path, PathBuf};

pub(crate) fn render_meta_info_window(
    ctx: &egui::Context,
    open: &mut bool,
    path: &std::path::Path,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let meta_text = crate::shell_logic::format_tree_tooltip(name, path);

    const META_INFO_WINDOW_WIDTH: f32 = 400.0;
    egui::Window::new(crate::i18n::get().action.show_meta_info.clone())
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_width(META_INFO_WINDOW_WIDTH)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(meta_text);
            });
        });
}
