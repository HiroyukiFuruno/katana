#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::hash_str;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub(crate) trait UpdateOps {
    fn start_update_check(&mut self, is_manual: bool);
    fn poll_update_install(&mut self, ctx: &egui::Context);
    fn poll_update_check(&mut self, _ctx: &egui::Context);
}

impl UpdateOps for KatanaApp {
    fn start_update_check(&mut self, is_manual: bool) {
        if self.state.update.checking {
            if is_manual {
                // Already checking, just show the dialog to let the user see the progress
                self.show_update_dialog = true;
            }
            return;
        }
        self.state.update.checking = true;
        self.state.update.check_error = None;
        self.state.update.available = None;

        if is_manual {
            self.show_update_dialog = true;
            self.update_notified = true; // Pretend we've notified them so it doesn't pop up AGAIN
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.update_rx = Some(rx);

        std::thread::spawn(move || {
            let result = katana_core::update::check_for_updates(env!("CARGO_PKG_VERSION"), None)
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }
    fn poll_update_install(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.update_install_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    UpdateInstallEvent::Progress(prog) => {
                        match prog {
                            katana_core::update::UpdateProgress::Downloading {
                                downloaded,
                                total,
                            } => {
                                let progress = if let Some(t) = total {
                                    if t > 0 {
                                        downloaded as f32 / t as f32
                                    } else {
                                        0.0
                                    }
                                } else {
                                    // Indeterminate progress if no Content-Length
                                    0.0
                                };
                                self.state.update.phase =
                                    Some(crate::app_state::UpdatePhase::Downloading { progress });
                            }
                            katana_core::update::UpdateProgress::Extracting { current, total } => {
                                let progress = if total > 0 {
                                    current as f32 / total as f32
                                } else {
                                    0.0
                                };
                                self.state.update.phase =
                                    Some(crate::app_state::UpdatePhase::Installing { progress });
                            }
                        }
                        ctx.request_repaint();
                    }
                    UpdateInstallEvent::Finished(Ok(prep)) => {
                        self.state.update.checking = false;
                        self.state.update.phase =
                            Some(crate::app_state::UpdatePhase::ReadyToRelaunch);
                        self.pending_relaunch = Some(prep);
                        self.show_update_dialog = true;
                        self.update_install_rx = None;
                        ctx.request_repaint();
                        break;
                    }
                    UpdateInstallEvent::Finished(Err(err)) => {
                        self.state.update.checking = false;
                        self.state.update.phase = None;
                        self.state.update.check_error = Some(err);
                        self.show_update_dialog = true;
                        self.update_install_rx = None;
                        ctx.request_repaint();
                        break;
                    }
                }
            }
        }
    }
    fn poll_update_check(&mut self, _ctx: &egui::Context) {
        if let Some(rx) = &self.update_rx {
            match rx.try_recv() {
                Ok(Ok(Some(release_info))) => {
                    self.state.update.checking = false;
                    self.update_rx = None;
                    if katana_core::update::is_newer_version(
                        env!("CARGO_PKG_VERSION"),
                        &release_info.tag_name,
                    ) {
                        self.state.update.available = Some(release_info);
                        if !self.update_notified {
                            self.show_update_dialog = true;
                            self.update_notified = true;
                        }
                    } else {
                        self.state.update.available = None;
                        if !self.update_notified {
                            self.update_notified = true;
                        }
                    }
                }
                Ok(Ok(None)) => {
                    self.state.update.checking = false;
                    self.update_rx = None;
                    self.state.update.available = None;
                    if !self.update_notified {
                        self.update_notified = true;
                    }
                }
                Ok(Err(err)) => {
                    self.state.update.checking = false;
                    self.state.update.check_error = Some(err);
                    self.update_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Update still in progress.
                }
                Err(_) => {
                    self.state.update.checking = false;
                    self.update_rx = None;
                }
            }
        }
    }
}
