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

pub(crate) fn render_create_fs_node_modal(
    ctx: &egui::Context,
    state: &mut crate::app_state::AppState,
    pending_action: &mut crate::app_state::AppAction,
) {
    let mut close = false;
    let mut do_create = false;

    if let Some((parent_dir, mut name, mut selected_ext, is_dir)) =
        state.layout.create_fs_node_modal.take()
    {
        let title = if is_dir {
            crate::i18n::get().dialog.new_directory_title.clone()
        } else {
            crate::i18n::get().dialog.new_file_title.clone()
        };

        let mut is_open = true;
        egui::Window::new(title)
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    const MODAL_INPUT_WIDTH: f32 = 200.0;
                    let re = ui.add(
                        egui::TextEdit::singleline(&mut name)
                            .hint_text("Name")
                            .desired_width(MODAL_INPUT_WIDTH),
                    );
                    re.request_focus();

                    if !is_dir {
                        if let Some(ref mut ext) = selected_ext {
                            const EXT_COMBOBOX_WIDTH: f32 = 80.0;
                            let options = state
                                .config
                                .settings
                                .settings()
                                .workspace
                                .visible_extensions
                                .clone();
                            crate::widgets::StyledComboBox::new("new_file_ext", ext.as_str())
                                .width(EXT_COMBOBOX_WIDTH)
                                .show(ui, |ui| {
                                    for opt in &options {
                                        ui.selectable_value(ext, opt.clone(), opt);
                                    }
                                });
                        }
                    }

                    if re.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_create = true;
                    }
                });
                const SPACING_SMALL: f32 = 8.0;
                ui.add_space(SPACING_SMALL);
                ui.horizontal(|ui| {
                    if ui
                        .button(crate::i18n::get().action.cancel.clone())
                        .clicked()
                    {
                        close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(crate::i18n::get().action.save.clone()).clicked() {
                            do_create = true;
                        }
                    });
                });
            });

        if !is_open {
            close = true;
        }

        if do_create && !name.is_empty() {
            let actual_name = if !is_dir {
                if let Some(ref ext) = selected_ext {
                    if name.ends_with(&format!(".{}", ext)) {
                        name.clone()
                    } else {
                        format!("{}.{}", name, ext)
                    }
                } else {
                    name.clone()
                }
            } else {
                name.clone()
            };

            let target_path = parent_dir.join(&actual_name);
            let res = if is_dir {
                std::fs::create_dir(&target_path)
            } else {
                std::fs::File::create(&target_path).map(|_| ())
            };
            if let Err(e) = res {
                tracing::error!("Failed to create fs node: {}", e);
            } else {
                if is_dir {
                    state.workspace.in_memory_dirs.insert(target_path);
                }
                *pending_action = crate::app_state::AppAction::RefreshWorkspace;
                state
                    .workspace
                    .expanded_directories
                    .insert(parent_dir.clone());
            }
            close = true;
        }

        if !close {
            state.layout.create_fs_node_modal = Some((parent_dir, name, selected_ext, is_dir));
        }
    }
}

pub(crate) fn render_rename_modal(
    ctx: &egui::Context,
    state: &mut crate::app_state::AppState,
    pending_action: &mut crate::app_state::AppAction,
) {
    let mut close = false;
    let mut do_rename = false;

    if let Some((target_path, mut new_name)) = state.layout.rename_modal.take() {
        let mut is_open = true;
        egui::Window::new(crate::i18n::get().dialog.rename_title.clone())
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    const MODAL_INPUT_WIDTH: f32 = 200.0;
                    let re = ui.add(
                        egui::TextEdit::singleline(&mut new_name)
                            .hint_text("New Name")
                            .desired_width(MODAL_INPUT_WIDTH),
                    );
                    re.request_focus();

                    if re.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_rename = true;
                    }
                });
                const SPACING_SMALL: f32 = 8.0;
                ui.add_space(SPACING_SMALL);
                ui.horizontal(|ui| {
                    if ui
                        .button(crate::i18n::get().action.cancel.clone())
                        .clicked()
                    {
                        close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(crate::i18n::get().action.save.clone()).clicked() {
                            do_rename = true;
                        }
                    });
                });
            });

        if !is_open {
            close = true;
        }

        if do_rename && !new_name.is_empty() {
            if let Some(parent) = target_path.parent() {
                let new_path = parent.join(&new_name);
                if let Err(e) = std::fs::rename(&target_path, &new_path) {
                    tracing::error!("Failed to rename file: {}", e);
                } else {
                    *pending_action = crate::app_state::AppAction::RefreshWorkspace;
                    for doc in &mut state.document.open_documents {
                        if doc.path == target_path {
                            doc.path = new_path.clone();
                            break;
                        }
                    }
                }
            }
            close = true;
        }

        if !close {
            state.layout.rename_modal = Some((target_path, new_name));
        }
    }
}

pub(crate) fn render_delete_modal(
    ctx: &egui::Context,
    state: &mut crate::app_state::AppState,
    pending_action: &mut crate::app_state::AppAction,
) {
    let mut close = false;

    if let Some(target_path) = state.layout.delete_modal.take() {
        let mut is_open = true;
        egui::Window::new(crate::i18n::get().dialog.delete_title.clone())
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let name = target_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                let msg = crate::i18n::tf(
                    &crate::i18n::get().dialog.delete_confirm_msg,
                    &[("name", name)],
                );
                ui.label(msg);

                const SPACING_SMALL: f32 = 8.0;
                ui.add_space(SPACING_SMALL);
                ui.horizontal(|ui| {
                    if ui
                        .button(crate::i18n::get().action.cancel.clone())
                        .clicked()
                    {
                        close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new(crate::i18n::get().action.delete.clone())
                                .color(ui.visuals().error_fg_color),
                        );
                        if ui.add(del_btn).clicked() {
                            let res = if target_path.is_dir() {
                                std::fs::remove_dir_all(&target_path)
                            } else {
                                std::fs::remove_file(&target_path)
                            };

                            if let Err(e) = res {
                                tracing::error!("Failed to delete path: {}", e);
                            } else {
                                *pending_action = crate::app_state::AppAction::RefreshWorkspace;
                                if let Some(idx) = state
                                    .document
                                    .open_documents
                                    .iter()
                                    .position(|d| d.path == target_path)
                                {
                                    state.document.open_documents.remove(idx);
                                    if let Some(active_idx) = state.document.active_doc_idx {
                                        if active_idx == idx {
                                            state.document.active_doc_idx =
                                                if state.document.open_documents.is_empty() {
                                                    None
                                                } else {
                                                    Some(if idx > 0 { idx - 1 } else { 0 })
                                                };
                                        } else if active_idx > idx {
                                            state.document.active_doc_idx = Some(active_idx - 1);
                                        }
                                    }
                                }
                            }
                            close = true;
                        }
                    });
                });
            });

        if !is_open {
            close = true;
        }

        if !close {
            state.layout.delete_modal = Some(target_path);
        }
    }
}
