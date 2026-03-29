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

pub(crate) trait PreviewOps {
    fn get_preview_pane(
        previews: &mut Vec<TabPreviewCache>,
        path: std::path::PathBuf,
    ) -> &mut PreviewPane;
    fn refresh_preview(&mut self, path: &std::path::Path, source: &str);
    fn full_refresh_preview(
        &mut self,
        path: &std::path::Path,
        source: &str,
        force: bool,
        concurrency: usize,
    );
}

impl PreviewOps for KatanaApp {
    fn get_preview_pane(
        previews: &mut Vec<TabPreviewCache>,
        path: std::path::PathBuf,
    ) -> &mut PreviewPane {
        if let Some(idx) = previews.iter().position(|t| t.path == path) {
            &mut previews[idx].pane
        } else {
            previews.push(TabPreviewCache {
                path,
                pane: PreviewPane::default(),
                hash: 0,
            });
            &mut previews.last_mut().expect("just pushed").pane
        }
    }
    fn refresh_preview(&mut self, path: &std::path::Path, source: &str) {
        let h = hash_str(source);
        let path_buf = path.to_path_buf();

        let current_hash = self
            .tab_previews
            .iter()
            .find(|t| t.path == path_buf)
            .map(|t| t.hash)
            .unwrap_or(0);

        if current_hash != 0 && current_hash == h {
            return;
        }

        Self::get_preview_pane(&mut self.tab_previews, path_buf.clone())
            .update_markdown_sections(source, path);

        if let Some(tab) = self.tab_previews.iter_mut().find(|t| t.path == path_buf) {
            tab.hash = h;
        }
    }
    fn full_refresh_preview(
        &mut self,
        path: &std::path::Path,
        source: &str,
        force: bool,
        concurrency: usize,
    ) {
        let h = hash_str(source);
        let path_buf = path.to_path_buf();
        let current_hash = self
            .tab_previews
            .iter()
            .find(|t| t.path == path_buf)
            .map(|t| t.hash)
            .unwrap_or(0);

        if !force && current_hash != 0 && current_hash == h {
            return;
        }

        tracing::debug!(
            "[DEBUG-HASH] MISMATCH or FORCE. Running full_render for path: {:?}. force={}, current_hash={}, new_hash={}",
            path_buf,
            force,
            current_hash,
            h
        );

        let pane = Self::get_preview_pane(&mut self.tab_previews, path_buf.clone());
        pane.full_render(
            source,
            path,
            self.state.config.cache.clone(),
            force,
            concurrency,
        );

        let tab = self
            .tab_previews
            .iter_mut()
            .find(|t| t.path == path_buf)
            .expect("just fetched pane");
        // If force, also reset hash to 0 so it redraws on switch, or update it now.
        // We update to true hash since we re-rendered anyway.
        tab.hash = h;
    }
}
