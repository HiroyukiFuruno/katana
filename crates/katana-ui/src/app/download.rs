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

pub(crate) trait DownloadOps {
    fn start_download(&mut self, req: DownloadRequest);
    fn poll_download(&mut self, ctx: &egui::Context);
}

impl DownloadOps for KatanaApp {
    fn start_download(&mut self, req: DownloadRequest) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        self.state.layout.status_message = Some((
            crate::i18n::get().plantuml.downloading_plantuml.clone(),
            crate::app_state::StatusType::Info,
        ));
        let url = req.url;
        let dest = req.dest;
        std::thread::spawn(move || {
            let result = download_with_curl(&url, &dest);
            let _ = tx.send(result);
        });
    }
    fn poll_download(&mut self, ctx: &egui::Context) {
        let done = if let Some(rx) = &self.download_rx {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    self.state.layout.status_message = Some((
                        crate::i18n::get().plantuml.plantuml_installed.clone(),
                        crate::app_state::StatusType::Success,
                    ));
                    self.pending_action = AppAction::RefreshDiagrams;
                    true
                }
                Ok(Err(e)) => {
                    self.state.layout.status_message = Some((
                        format!(
                            "{}{}",
                            crate::i18n::get().plantuml.download_error.clone(),
                            e
                        ),
                        crate::app_state::StatusType::Error,
                    ));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(
                        DOWNLOAD_STATUS_CHECK_INTERVAL_MS,
                    ));
                    false
                }
                Err(_) => true,
            }
        } else {
            false
        };
        if done {
            self.download_rx = None;
        }
    }
}