//! KatanA three-pane egui shell.
//!
//! Contains only business logic. egui rendering code is separated into shell_ui.rs.

#![allow(clippy::useless_vec)]

use eframe::egui;
use katana_platform::theme::ThemeColors;
use katana_platform::FilesystemService;

use crate::app::*;

use crate::{
    app_state::{AppAction, AppState},
    preview_pane::PreviewPane,
};

// ─────────────────────────────────────────────
// Layout Constants
// ─────────────────────────────────────────────

/// Width of the '›' toggle button displayed when the sidebar is collapsed (px).
pub(crate) const SIDEBAR_COLLAPSED_TOGGLE_WIDTH: f32 = 24.0;

/// Minimum resize width of the file tree panel (px).
pub(crate) const FILE_TREE_PANEL_MIN_WIDTH: f32 = 120.0;

/// Initial display width of the file tree panel (px).
pub(crate) const FILE_TREE_PANEL_DEFAULT_WIDTH: f32 = 220.0;

/// Minimum width of the preview panel in Split mode (px).
pub(crate) const SPLIT_PREVIEW_PANEL_MIN_WIDTH: f32 = 200.0;

/// Width of the ◀▶ navigation button area at the right end of the tab bar (px).
pub(crate) const TAB_NAV_BUTTONS_AREA_WIDTH: f32 = 80.0;

/// Inter-tab spacing provided on the right side of each tab (px).
pub(crate) const TAB_INTER_ITEM_SPACING: f32 = 4.0;

/// Lerp animation duration for the tab drop indicator line (seconds).
pub(crate) const TAB_DROP_ANIMATION_TIME: f32 = 0.1;

/// Width of the vertical line indicating where a tab will be dropped (px).
pub(crate) const TAB_DROP_INDICATOR_WIDTH: f32 = 2.5;

/// Initial number of visible rows for the text editor TextEdit.
pub(crate) const EDITOR_INITIAL_VISIBLE_ROWS: usize = 40;

/// Sensitivity threshold for scroll synchronization between editor and preview.
/// Scroll events are ignored if the fraction difference is less than or equal to this value.
pub(crate) const SCROLL_SYNC_DEAD_ZONE: f32 = 0.002;

/// Delay until the tab tooltip is displayed (seconds).
pub(crate) const TAB_TOOLTIP_SHOW_DELAY_SECS: f32 = 0.25;

/// Spacing below the 'No workspace selected' display in the file tree (px).
pub(crate) const NO_WORKSPACE_BOTTOM_SPACING: f32 = 8.0;

/// Vertical spacing between the "recent workspaces" section and other elements (px).
pub(crate) const RECENT_WORKSPACES_SPACING: f32 = 8.0;

/// Vertical spacing between items in the "recent workspaces" section (px).
pub(crate) const RECENT_WORKSPACES_ITEM_SPACING: f32 = 4.0;

/// Height of each row in the file tree (px).
pub(crate) const TREE_ROW_HEIGHT: f32 = 22.0;

/// Horizontal offset for labels in the file tree (px).
pub(crate) const TREE_LABEL_HOFFSET: f32 = 4.0;

/// Polling interval for checking download completion (ms).
pub(crate) const DOWNLOAD_STATUS_CHECK_INTERVAL_MS: u64 = 200;

// ─────────────────────────────────────────────
// Color Constants
// ─────────────────────────────────────────────
// Hardcoded colours have been migrated to ThemeColors (katana-platform::theme).
// Individual constants are no longer needed as visuals_from_theme() sets them.

/// Rounding radius of the active row background in the file tree.
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

/// A single background export task with its communication channel.
pub(crate) struct ExportTask {
    /// Display filename for the progress indicator.
    pub filename: String,
    /// Receiver for the background thread result.
    pub rx: std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>,
    /// Whether to open the result file in the browser after completion (HTML exports).
    pub open_on_complete: bool,
}

pub enum UpdateInstallEvent {
    Progress(katana_core::update::UpdateProgress),
    Finished(Result<katana_core::update::UpdatePreparation, String>),
}

pub struct KatanaApp {
    /// The global application state object. Contains all reactive data models.
    pub(crate) state: AppState,
    pub(crate) fs: FilesystemService,
    pub(crate) pending_action: AppAction,
    /// Per-tab preview pane cache. Reuses cache on tab switch.
    pub(crate) tab_previews: Vec<TabPreviewCache>,
    /// Receiver for background download completion notifications.
    pub(crate) download_rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    /// Receiver for background workspace loading.
    pub(crate) workspace_rx: Option<std::sync::mpsc::Receiver<WorkspaceLoadMessage>>,
    /// Receiver for background update checks.
    pub(crate) update_rx:
        Option<std::sync::mpsc::Receiver<Result<Option<katana_core::update::ReleaseInfo>, String>>>,
    pub(crate) changelog_rx: Option<std::sync::mpsc::Receiver<crate::changelog::ChangelogEvent>>,
    pub(crate) update_install_rx: Option<std::sync::mpsc::Receiver<UpdateInstallEvent>>,
    /// Active background export tasks.
    pub(crate) export_tasks: Vec<ExportTask>,
    /// Queue for aggressively opening multiple documents without freezing UI
    pub(crate) pending_document_loads: std::collections::VecDeque<std::path::PathBuf>,

    /// Whether the About dialog is currently visible.
    pub(crate) show_about: bool,
    /// Whether the Update dialog is currently visible.
    pub(crate) show_update_dialog: bool,
    pub(crate) update_markdown_cache: egui_commonmark::CommonMarkCache,
    /// Tracks if we have already automatically shown the update dialog on startup.
    pub(crate) update_notified: bool,
    /// App icon texture for the About dialog.
    /// Public because it is set from the binary crate (main.rs) during initialization.
    pub about_icon: Option<egui::TextureHandle>,
    /// Cached theme palette used to avoid redundant `set_visuals()` calls every frame.
    pub(crate) cached_theme: Option<ThemeColors>,
    /// Cached font size to avoid redundant `style_mut()` calls every frame.
    pub(crate) cached_font_size: Option<f32>,
    /// Cached font family to avoid rebuilding `FontDefinitions` every frame.
    pub(crate) cached_font_family: Option<String>,
    /// Dedicated PreviewPane for the settings window live preview.
    pub(crate) settings_preview: PreviewPane,
    /// Whether the splash screen needs to start on the first frame.
    pub(crate) needs_splash: bool,
    /// Tracks the startup time for the splash screen fade animation.
    pub(crate) splash_start: Option<std::time::Instant>,
    /// Path for the currently active metadata dialog.
    pub(crate) show_meta_info_for: Option<std::path::PathBuf>,
    /// Prepared update ready to be relaunched after user confirmation.
    pub(crate) pending_relaunch: Option<katana_core::update::UpdatePreparation>,
    /// Parsed changelog sections for the release notes window.
    pub(crate) changelog_sections: Vec<crate::changelog::ChangelogSection>,
    /// Flags whether the changelog tab needs to be displayed after startup.
    pub(crate) needs_changelog_display: bool,
    /// The previous app version, used to determine which changelog sections to open by default.
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
                // First launch ever or first launch since v0.8.0
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

    /// Explicitly skips the splash screen. Useful for integration testing.
    pub fn skip_splash(&mut self) {
        self.needs_splash = false;
        self.splash_start = None;
    }

    /// Test-only helper: opens the update dialog without going through the menu action.
    #[doc(hidden)]
    pub fn open_update_dialog_for_test(&mut self) {
        self.show_update_dialog = true;
    }

    /// Test-only helper: disables automatic changelog display.
    #[doc(hidden)]
    pub fn disable_changelog_display_for_test(&mut self) {
        self.needs_changelog_display = false;
    }

    /// Test-only helper to inspect app state.
    #[doc(hidden)]
    pub fn app_state_for_test(&self) -> &AppState {
        &self.state
    }

    /// Test-only helper to set changelog sections.
    #[doc(hidden)]
    pub fn set_changelog_sections_for_test(
        &mut self,
        sections: Vec<crate::changelog::ChangelogSection>,
    ) {
        self.changelog_sections = sections;
    }

    /// Test-only helper to clear the changelog receiver.
    pub fn clear_changelog_rx_for_test(&mut self) {
        self.changelog_rx = None;
    }
}

/// Converts markdown source to HTML and writes it to the system temp directory.
///
/// Returns the absolute path of the written HTML file.
///
/// This is a pure function (no UI state) suitable for background threads and unit tests.
/// Steps: 1) markdown→HTML  2) write to /tmp  3) return path
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
    /// Method for injecting actions from the program, e.g., during testing
    pub fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }

    /// Helper for calling AppState methods, e.g., during testing
    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

/// Calls `curl` as a subprocess to download a file.
pub(crate) fn download_with_curl(url: &str, dest: &std::path::Path) -> Result<(), String> {
    _download_with_cmd("curl", url, dest)
}

fn _download_with_cmd(cmd: &str, url: &str, dest: &std::path::Path) -> Result<(), String> {
    // Create parent directory if it does not exist
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

// impl eframe::App for KatanaApp has been moved to shell_ui.rs

#[cfg(test)]
include!("shell_tests.rs");
