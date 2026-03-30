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

pub(crate) struct CreateFsNodeModal<'a> {
    pub modal_data: &'a mut (PathBuf, String, Option<String>, bool),
    pub visible_extensions: &'a [String],
    pub pending_action: &'a mut crate::app_state::AppAction,
}

impl<'a> CreateFsNodeModal<'a> {
    pub fn new(
        modal_data: &'a mut (PathBuf, String, Option<String>, bool),
        visible_extensions: &'a [String],
        pending_action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self {
            modal_data,
            visible_extensions,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let (parent_dir, name, selected_ext, is_dir) = self.modal_data;
        let pending_action = self.pending_action;
        let mut close = false;
        let mut do_create = false;

        let title = if *is_dir {
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
                        egui::TextEdit::singleline(name)
                            .hint_text("Name")
                            .desired_width(MODAL_INPUT_WIDTH),
                    );
                    re.request_focus();

                    if !*is_dir {
                        if let Some(ref mut ext) = selected_ext {
                            const EXT_COMBOBOX_WIDTH: f32 = 80.0;
                            let options = self.visible_extensions;
                            crate::widgets::StyledComboBox::new("new_file_ext", ext.as_str())
                                .width(EXT_COMBOBOX_WIDTH)
                                .show(ui, |ui| {
                                    for opt in options {
                                        ui.selectable_value(ext, opt.clone(), opt.clone());
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
            let actual_name = if !*is_dir {
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
            *pending_action = crate::app_state::AppAction::CreateFsNode {
                target_path,
                is_dir: *is_dir,
                parent_dir: parent_dir.clone(),
            };
            close = true;
        }

        close
    }
}

pub(crate) struct RenameModal<'a> {
    pub modal_data: &'a mut (PathBuf, String),
    pub pending_action: &'a mut crate::app_state::AppAction,
}

impl<'a> RenameModal<'a> {
    pub fn new(
        modal_data: &'a mut (PathBuf, String),
        pending_action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self {
            modal_data,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let (target_path, new_name) = self.modal_data;
        let pending_action = self.pending_action;
        let mut close = false;
        let mut do_rename = false;

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
                        egui::TextEdit::singleline(new_name)
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
                let new_path = parent.join(&*new_name);
                *pending_action = crate::app_state::AppAction::RenameFsNode {
                    target_path: target_path.clone(),
                    new_path,
                };
            }
            close = true;
        }

        close
    }
}

pub(crate) struct DeleteModal<'a> {
    pub target_path: &'a PathBuf,
    pub pending_action: &'a mut crate::app_state::AppAction,
}

impl<'a> DeleteModal<'a> {
    pub fn new(
        target_path: &'a PathBuf,
        pending_action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self {
            target_path,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let target_path = self.target_path;
        let pending_action = self.pending_action;
        let mut close = false;

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
                            *pending_action = crate::app_state::AppAction::DeleteFsNode {
                                target_path: target_path.clone(),
                            };
                            close = true;
                        }
                    });
                });
            });

        if !is_open {
            close = true;
        }

        close
    }
}