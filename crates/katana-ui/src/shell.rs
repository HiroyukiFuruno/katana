
#![allow(clippy::useless_vec)]

use eframe::egui;
use katana_platform::theme::ThemeColors;
use katana_platform::FilesystemService;

use crate::app::*;

use crate::{
    app_state::{AppAction, AppState},
    preview_pane::PreviewPane,
};


pub(crate) const SIDEBAR_COLLAPSED_TOGGLE_WIDTH: f32 = 24.0;

pub(crate) const FILE_TREE_PANEL_MIN_WIDTH: f32 = 120.0;

pub(crate) const FILE_TREE_PANEL_DEFAULT_WIDTH: f32 = 220.0;

pub(crate) const SPLIT_PREVIEW_PANEL_MIN_WIDTH: f32 = 200.0;

pub(crate) const TAB_NAV_BUTTONS_AREA_WIDTH: f32 = 80.0;

pub(crate) const TAB_INTER_ITEM_SPACING: f32 = 4.0;

pub(crate) const TAB_DROP_ANIMATION_TIME: f32 = 0.1;

pub(crate) const TAB_DROP_INDICATOR_WIDTH: f32 = 2.5;

pub(crate) const EDITOR_INITIAL_VISIBLE_ROWS: usize = 40;

pub(crate) const SCROLL_SYNC_DEAD_ZONE: f32 = 0.002;

pub(crate) const TAB_TOOLTIP_SHOW_DELAY_SECS: f32 = 0.25;

pub(crate) const NO_WORKSPACE_BOTTOM_SPACING: f32 = 8.0;

pub(crate) const RECENT_WORKSPACES_SPACING: f32 = 8.0;

pub(crate) const RECENT_WORKSPACES_ITEM_SPACING: f32 = 4.0;

pub(crate) const TREE_ROW_HEIGHT: f32 = 22.0;

pub(crate) const TREE_LABEL_HOFFSET: f32 = 4.0;

pub(crate) const DOWNLOAD_STATUS_CHECK_INTERVAL_MS: u64 = 200;


pub(crate) const ACTIVE_FILE_HIGHLIGHT_ROUNDING: f32 = 3.0;

pub(crate) struct TabPreviewCache {
    pub path: std::path::PathBuf,
    pub pane: PreviewPane,
    pub hash: u64,
}

pub(crate) enum WorkspaceLoadType {
    Open,
    Refresh,
}

pub(crate) type WorkspaceLoadResult =
    Result<katana_core::workspace::Workspace, katana_core::workspace::WorkspaceError>;
pub(crate) type WorkspaceLoadMessage = (WorkspaceLoadType, std::path::PathBuf, WorkspaceLoadResult);

pub(crate) struct ExportTask {
    pub filename: String,
    pub rx: std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>,
    pub open_on_complete: bool,
}

pub enum UpdateInstallEvent {
    Progress(katana_core::update::UpdateProgress),
    Finished(Result<katana_core::update::UpdatePreparation, String>),
}

pub struct KatanaApp {
    pub(crate) state: AppState,
    pub(crate) fs: FilesystemService,
    pub(crate) pending_action: AppAction,
    pub(crate) tab_previews: Vec<TabPreviewCache>,
    pub(crate) download_rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    pub(crate) workspace_rx: Option<std::sync::mpsc::Receiver<WorkspaceLoadMessage>>,
    pub(crate) update_rx:
        Option<std::sync::mpsc::Receiver<Result<Option<katana_core::update::ReleaseInfo>, String>>>,
    pub(crate) changelog_rx: Option<std::sync::mpsc::Receiver<crate::changelog::ChangelogEvent>>,
    pub(crate) update_install_rx: Option<std::sync::mpsc::Receiver<UpdateInstallEvent>>,
    pub(crate) export_tasks: Vec<ExportTask>,
    pub(crate) pending_document_loads: std::collections::VecDeque<std::path::PathBuf>,

    pub(crate) show_about: bool,
    pub(crate) show_update_dialog: bool,
    pub(crate) update_markdown_cache: egui_commonmark::CommonMarkCache,
    pub(crate) update_notified: bool,
    pub about_icon: Option<egui::TextureHandle>,
    pub(crate) cached_theme: Option<ThemeColors>,
    pub(crate) cached_font_size: Option<f32>,
    pub(crate) cached_font_family: Option<String>,
    pub(crate) settings_preview: PreviewPane,
    pub(crate) needs_splash: bool,
    pub(crate) splash_start: Option<std::time::Instant>,
    pub(crate) show_meta_info_for: Option<std::path::PathBuf>,
    pub(crate) pending_relaunch: Option<katana_core::update::UpdatePreparation>,
    pub(crate) changelog_sections: Vec<crate::changelog::ChangelogSection>,
    pub(crate) needs_changelog_display: bool,
    pub(crate) old_app_version: Option<String>,
}

impl KatanaApp {
    pub fn new(state: AppState) -> Self {
        let mut app = Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            tab_previews: Vec::new(),
            download_rx: None,
            workspace_rx: None,
            update_rx: None,
            changelog_rx: None,
            update_install_rx: None,
            export_tasks: Vec::new(),
            pending_document_loads: std::collections::VecDeque::new(),
            show_about: false,
            show_update_dialog: false,
            update_markdown_cache: egui_commonmark::CommonMarkCache::default(),
            update_notified: false,
            about_icon: None,
            cached_theme: None,
            cached_font_size: None,
            cached_font_family: None,
            settings_preview: PreviewPane::default(),
            needs_splash: !cfg!(test),
            splash_start: None,
            show_meta_info_for: None,
            pending_relaunch: None,
            changelog_sections: Vec::new(),
            needs_changelog_display: false,
            old_app_version: None,
        };
        let current_version = env!("CARGO_PKG_VERSION");
        let mut show_changelog = false;

        {
            let settings_mut = app.state.config.settings.settings_mut();
            if let Some(prev) = &settings_mut.updates.previous_app_version {
                app.old_app_version = Some(prev.clone());
                if prev != current_version {
                    show_changelog = true;
                }
            } else {
                show_changelog = true;
            }
            if show_changelog {
                settings_mut.updates.previous_app_version = Some(current_version.to_string());
            }
        }

        if show_changelog {
            if let Err(e) = app.state.config.settings.save() {
                tracing::warn!("Failed to save previous_app_version: {e}");
            }
            app.needs_changelog_display = true;
        }

        app.start_update_check(false);
        app
    }

    pub fn skip_splash(&mut self) {
        self.needs_splash = false;
        self.splash_start = None;
    }

    #[doc(hidden)]
    pub fn open_update_dialog_for_test(&mut self) {
        self.show_update_dialog = true;
    }

    #[doc(hidden)]
    pub fn disable_changelog_display_for_test(&mut self) {
        self.needs_changelog_display = false;
    }

    #[doc(hidden)]
    pub fn app_state_for_test(&self) -> &AppState {
        &self.state
    }

    #[doc(hidden)]
    pub fn set_changelog_sections_for_test(
        &mut self,
        sections: Vec<crate::changelog::ChangelogSection>,
    ) {
        self.changelog_sections = sections;
    }

    pub fn clear_changelog_rx_for_test(&mut self) {
        self.changelog_rx = None;
    }
}

pub(crate) fn export_html_to_tmp(
    source: &str,
    filename: &str,
    preset: &katana_core::markdown::color_preset::DiagramColorPreset,
    base_dir: Option<&std::path::Path>,
) -> Result<std::path::PathBuf, String> {
    let renderer = katana_core::markdown::KatanaRenderer;
    let html = katana_core::markdown::HtmlExporter::export(source, &renderer, preset, base_dir)
        .map_err(|e| e.to_string())?;
    let output_path = std::path::PathBuf::from("/tmp").join(filename);
    std::fs::write(&output_path, html.as_bytes()).map_err(|e| e.to_string())?;
    Ok(output_path)
}

impl KatanaApp {
    pub fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }

    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

pub(crate) fn download_with_curl(url: &str, dest: &std::path::Path) -> Result<(), String> {
    _download_with_cmd("curl", url, dest)
}

fn _download_with_cmd(cmd: &str, url: &str, dest: &std::path::Path) -> Result<(), String> {
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let status = std::process::Command::new(cmd)
        .args(vec!["-L", "-o", dest.to_str().unwrap_or(""), url])
        .status()
        .map_err(|e| {
            format!(
                "{}: {e}",
                crate::i18n::get().error.curl_launch_failed.clone()
            )
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(crate::i18n::get().error.download_failed.clone())
    }
}


#[cfg(test)]
include!("shell_tests.rs");