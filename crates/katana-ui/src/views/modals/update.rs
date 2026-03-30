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

pub(crate) struct UpdateModal<'a> {
    pub open: &'a mut bool,
    pub state: &'a AppState,
    pub markdown_cache: &'a mut egui_commonmark::CommonMarkCache,
    pub pending_action: &'a mut AppAction,
}

impl<'a> UpdateModal<'a> {
    pub fn new(
        open: &'a mut bool,
        state: &'a AppState,
        markdown_cache: &'a mut egui_commonmark::CommonMarkCache,
        pending_action: &'a mut AppAction,
    ) -> Self {
        Self {
            open,
            state,
            markdown_cache,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        let open = self.open;
        let state = self.state;
        let markdown_cache = self.markdown_cache;
        let pending_action = self.pending_action;
        use crate::app_state::UpdatePhase;
        use crate::widgets::Modal;

        const SPACING_SMALL: f32 = 4.0;
        const SPACING_MEDIUM: f32 = 8.0;
        const SPACING_LARGE: f32 = 12.0;
        const MAX_SCROLL_HEIGHT: f32 = 250.0;
        const UPDATE_DIALOG_WIDTH: f32 = 600.0;

        let msgs = &crate::i18n::get().update;

        match &state.update.phase {
            Some(UpdatePhase::Downloading { progress }) => {
                Modal::new("katana_update_dialog_v6", &msgs.title)
                    .width(UPDATE_DIALOG_WIDTH)
                    .show_body_only(ctx, |ui| {
                        ui.add_space(SPACING_SMALL);
                        ui.add(
                            egui::ProgressBar::new(*progress)
                                .animate(true)
                                .text(format!("{:.0}%", progress * 100.0)),
                        );
                        ui.add_space(SPACING_MEDIUM);
                        ui.label(&msgs.downloading);
                    });
                return;
            }
            Some(UpdatePhase::Installing { progress }) => {
                Modal::new("katana_update_dialog_v6", &msgs.title)
                    .width(UPDATE_DIALOG_WIDTH)
                    .show_body_only(ctx, |ui| {
                        ui.add_space(SPACING_SMALL);
                        ui.add(
                            egui::ProgressBar::new(*progress)
                                .animate(true)
                                .text(format!("{:.0}%", progress * 100.0)),
                        );
                        ui.add_space(SPACING_MEDIUM);
                        ui.label(&msgs.installing);
                    });
                return;
            }
            Some(UpdatePhase::ReadyToRelaunch) => {
                let action = Modal::new("katana_update_dialog_v6", &msgs.title)
                    .width(UPDATE_DIALOG_WIDTH)
                    .show(
                        ctx,
                        |ui| {
                            ui.add_space(SPACING_LARGE);
                            ui.label(egui::RichText::new(&msgs.restart_confirm).heading());
                            ui.add_space(SPACING_LARGE);
                        },
                        |ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new(&msgs.action_restart)
                                            .color(ui.visuals().widgets.active.text_color())
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    return Some(AppAction::ConfirmRelaunch);
                                }
                                if ui.button(&msgs.action_later).clicked() {
                                    return Some(AppAction::DismissUpdate);
                                }
                                None
                            })
                            .inner
                        },
                    );
                if let Some(action) = action {
                    *pending_action = action;
                    if matches!(pending_action, AppAction::DismissUpdate) {
                        *open = false;
                    }
                }
                return;
            }
            None => {} // WHY: Fall through to the standard update dialog
        }

        if state.update.checking {
            Modal::new("katana_update_dialog_v6", &msgs.title)
                .width(UPDATE_DIALOG_WIDTH)
                .show_body_only(ctx, |ui| {
                    ui.add(egui::Spinner::new());
                    ui.add_space(SPACING_MEDIUM);
                    ui.label(msgs.checking_for_updates.clone());
                });
        } else if let Some(err) = &state.update.check_error {
            let close = {
                let err: String = err.clone();
                Modal::new("katana_update_dialog_v6", &msgs.title)
                    .width(UPDATE_DIALOG_WIDTH)
                    .show(
                        ctx,
                        |ui| {
                            ui.colored_label(
                                ui.ctx()
                                    .data(|d| {
                                        d.get_temp::<katana_platform::theme::ThemeColors>(
                                            egui::Id::new("katana_theme_colors"),
                                        )
                                    })
                                    .map_or(crate::theme_bridge::WHITE, |tc| {
                                        crate::theme_bridge::rgb_to_color32(tc.system.error_text)
                                    }),
                                msgs.failed_to_check.clone(),
                            );
                            ui.add_space(SPACING_SMALL);
                            ui.label(&err);
                        },
                        |ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(msgs.action_close.clone()).clicked() {
                                    return Some(true);
                                }
                                None
                            })
                            .inner
                        },
                    )
            };
            if close == Some(true) {
                *open = false;
            }
        } else if let Some(latest) = &state.update.available {
            let tag = latest.tag_name.clone();
            let body_text = latest.body.clone();
            let desc = msgs
                .update_available_desc
                .replace("{version}", tag.as_str());
            let action = Modal::new("katana_update_dialog_v6", &msgs.title)
                .width(UPDATE_DIALOG_WIDTH)
                .show(
                    ctx,
                    |ui| {
                        ui.label(
                            egui::RichText::new(msgs.update_available.clone())
                                .heading()
                                .color(ui.visuals().widgets.active.text_color()),
                        );
                        ui.add_space(SPACING_MEDIUM);
                        ui.label(&desc);
                        ui.add_space(SPACING_LARGE);

                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(MAX_SCROLL_HEIGHT)
                                .auto_shrink([true, true])
                                .show(ui, |ui| {
                                    egui_commonmark::CommonMarkViewer::new().show(
                                        ui,
                                        markdown_cache,
                                        &body_text,
                                    );
                                });
                        });
                        ui.add_space(SPACING_LARGE);
                    },
                    |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(
                                    egui::RichText::new(msgs.install_update.clone())
                                        .color(ui.visuals().widgets.active.text_color())
                                        .strong(),
                                )
                                .clicked()
                            {
                                return Some(AppAction::InstallUpdate);
                            }
                            if ui
                                .button(crate::i18n::get().menu.release_notes.clone())
                                .clicked()
                            {
                                return Some(AppAction::ShowReleaseNotes);
                            }
                            if ui.button(msgs.action_skip_version.clone()).clicked() {
                                return Some(AppAction::SkipVersion(tag.clone()));
                            }
                            if ui.button(msgs.action_later.clone()).clicked() {
                                return Some(AppAction::DismissUpdate);
                            }
                            None
                        })
                        .inner
                    },
                );
            if let Some(action) = action {
                *pending_action = action;
                if matches!(
                    *pending_action,
                    AppAction::DismissUpdate
                        | AppAction::SkipVersion(_)
                        | AppAction::ShowReleaseNotes
                ) {
                    *open = false;
                }
            }
        } else {
            let close = Modal::new("katana_update_dialog_v6", &msgs.title)
                .width(UPDATE_DIALOG_WIDTH)
                .show(
                    ctx,
                    |ui| {
                        ui.heading(msgs.up_to_date.clone());
                        ui.add_space(SPACING_SMALL);
                        ui.label(msgs.up_to_date_desc.clone());
                    },
                    |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(msgs.action_close.clone()).clicked() {
                                return Some(true);
                            }
                            None
                        })
                        .inner
                    },
                );
            if close == Some(true) {
                *open = false;
            }
        }
    }
}