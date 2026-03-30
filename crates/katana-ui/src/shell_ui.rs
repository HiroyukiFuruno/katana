//! Pure egui UI rendering functions for the KatanA shell.
//!
//! This module contains code that depends entirely on the egui frame context
//! and UI events (e.g., clicks).
//! - Rendering functions that can only be called within `eframe::App::update`
//! - Branches that are not executed without user click events
//! - OS UI dependent code like `rfd` file dialogs
//!
//! Therefore, it is excluded from code coverage measurement using `--ignore-filename-regex`.

use crate::app::*;

use eframe::egui;

use crate::app_state::AppAction;

const INVISIBLE_LABEL_SIZE: f32 = 0.1;

pub(crate) fn invisible_label(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(INVISIBLE_LABEL_SIZE)
        .color(crate::theme_bridge::TRANSPARENT)
}

use crate::theme_bridge;

pub(crate) fn open_folder_dialog() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

pub(crate) const WORKSPACE_SPINNER_OUTER_MARGIN: f32 = 10.0;
pub(crate) const WORKSPACE_SPINNER_INNER_MARGIN: f32 = 10.0;
pub(crate) const WORKSPACE_SPINNER_TEXT_MARGIN: f32 = 5.0;
/// Green channel value for the success status bar color.
pub(crate) const STATUS_SUCCESS_GREEN: u8 = 200;
/// Spacing before the icon in the status bar.
pub(crate) const STATUS_BAR_ICON_SPACING: f32 = 4.0;

pub(crate) const SEARCH_MODAL_WIDTH: f32 = 500.0;
pub(crate) const SEARCH_MODAL_HEIGHT: f32 = 400.0;
pub(crate) const TOC_PANEL_DEFAULT_WIDTH: f32 = 200.0;
pub(crate) const TOC_PANEL_MARGIN: f32 = 8.0;
pub(crate) const TOC_HEADING_VISIBILITY_THRESHOLD: f32 = 40.0;
pub(crate) const TOC_INDENT_PER_LEVEL: f32 = 12.0;
pub(crate) const LIGHT_MODE_ICON_BG: u8 = 235;
pub(crate) const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 200;

pub(crate) fn relative_full_path(
    path: &std::path::Path,
    ws_root: Option<&std::path::Path>,
) -> String {
    crate::shell_logic::relative_full_path(path, ws_root)
}

pub(crate) struct TreeRenderContext<'a, 'b> {
    pub action: &'a mut AppAction,
    pub depth: usize,
    pub active_path: Option<&'b std::path::Path>,
    pub filter_set: Option<&'b std::collections::HashSet<std::path::PathBuf>>,
    pub expanded_directories: &'a mut std::collections::HashSet<std::path::PathBuf>,
    pub disable_context_menu: bool,
}

pub(crate) fn indent_prefix(depth: usize) -> String {
    "  ".repeat(depth)
}

// ─────────────────────────────────────────────
// Native menu re-exports (implementation in native_menu.rs)
// ─────────────────────────────────────────────

pub use crate::native_menu::update_native_menu_strings_from_i18n;

#[cfg(all(target_os = "macos", not(test)))]
pub use crate::native_menu::{native_menu_setup, native_set_app_icon_png, native_set_process_name};
use crate::shell::KatanaApp;

// Half-panel ratio for responsive 50/50 split.
pub(crate) const SPLIT_HALF_RATIO: f32 = 0.5;
/// Maximum ratio for TopBottomPanel in vertical split.
/// Prevents preview from consuming more than 70% of the available height,
/// guaranteeing the editor retains at least 30% for scrolling.
pub(crate) const SPLIT_PANEL_MAX_RATIO: f32 = 0.7;
pub(crate) const PREVIEW_CONTENT_PADDING: i8 = 12;

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start the splash screen timer exactly when the first frame is requested,
        // rather than during App allocation, to prevent it from expiring during loading.
        if self.needs_splash {
            self.splash_start = Some(std::time::Instant::now());
            self.needs_splash = false;
        }

        // --- Auto-Save Timer ---
        let auto_save_enabled = self.state.config.settings.settings().behavior.auto_save;
        let auto_save_interval = self
            .state
            .config
            .settings
            .settings()
            .behavior
            .auto_save_interval_secs;
        if auto_save_enabled && auto_save_interval > 0.0 {
            let now = std::time::Instant::now();
            if let Some(last) = self.state.document.last_auto_save {
                if now.duration_since(last).as_secs_f64() >= auto_save_interval {
                    if let Some(doc) = self.state.active_document() {
                        if doc.is_dirty {
                            self.pending_action = crate::app_state::AppAction::SaveDocument;
                        }
                    }
                    self.state.document.last_auto_save = Some(now);
                }
            } else {
                self.state.document.last_auto_save = Some(now);
            }
        } else {
            self.state.document.last_auto_save = None;
        }

        // Pre-calculate splash state to prevent flickering of the background UI.
        // If the splash is fully opaque (the first 1.5s), we skip drawing the panels.
        let splash_opacity = self
            .splash_start
            .map(|s| crate::shell_logic::calculate_splash_opacity(s.elapsed().as_secs_f32()))
            .unwrap_or(0.0);
        let splash_is_opaque = self.splash_start.is_some() && splash_opacity >= 1.0;

        // Apply theme colours to egui Visuals (only when the palette changed)
        let theme_colors = self
            .state
            .config
            .settings
            .settings()
            .effective_theme_colors();
        if self.cached_theme.as_ref() != Some(&theme_colors) {
            let dark = theme_colors.mode == katana_platform::theme::ThemeMode::Dark;
            ctx.set_visuals(theme_bridge::visuals_from_theme(&theme_colors));
            // Cache the full ThemeColors in the context so that each
            // rendering path (preview/code) can access its own colours independently.
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new("katana_theme_colors"), theme_colors.clone());
            });
            // Disable floating scrollbar animation — egui's animate_bool_responsive
            // for floating scrollbar hover detection triggers continuous repaints (~90fps).
            ctx.style_mut(|s| s.spacing.scroll.floating = false);
            katana_core::markdown::color_preset::DiagramColorPreset::set_dark_mode(dark);
            self.cached_theme = Some(theme_colors.clone());
            // Re-render diagrams with the new theme colours.
            // Only set if no action is already pending (e.g. OpenWorkspace from startup restore).
            if matches!(self.pending_action, AppAction::None) {
                self.pending_action = AppAction::RefreshDiagrams;
            }
        }

        // Apply font size to egui text styles (only when the size changed)
        let font_size = self.state.config.settings.settings().clamped_font_size();
        if self.cached_font_size != Some(font_size) {
            theme_bridge::apply_font_size(ctx, font_size);
            self.cached_font_size = Some(font_size);
        }

        // Apply font family by rebuilding FontDefinitions (only when family changed)
        let font_family = self.state.config.settings.settings().font.family.clone();
        if self.cached_font_family.as_deref() != Some(&font_family) {
            theme_bridge::apply_font_family(ctx, &font_family);
            self.cached_font_family = Some(font_family);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers {
                    command: true,
                    shift: true,
                    ..Default::default()
                },
                egui::Key::T,
            ))
        }) && !self.state.document.recently_closed_tabs.is_empty()
        {
            self.pending_action = AppAction::RestoreClosedDocument;
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::P,
            ))
        }) {
            self.state.layout.show_search_modal = true;
            // The query will persist across invocations as per standard fuzzy finders
        }

        self.poll_download(ctx);
        self.poll_workspace_load(ctx);

        // Process pending document loads (1 per frame to avoid UI freeze)
        if let Some(path) = self.pending_document_loads.pop_front() {
            self.handle_select_document(path, false);
            ctx.request_repaint();
        }

        self.poll_update_install(ctx);
        self.poll_update_check(ctx);
        self.poll_changelog(ctx);
        self.poll_export(ctx);

        // macOS: Poll actions from the native menu.
        let native_action =
            crate::native_menu::poll_native_menu(&mut self.show_about, open_folder_dialog);
        if !matches!(native_action, AppAction::None) {
            self.pending_action = native_action;
        }

        let action = self.take_action();
        crate::views::panels::preview::invalidate_preview_image_cache(ctx, &action);
        self.process_action(ctx, action);

        if !splash_is_opaque {
            // Terms of Service check (Blocking UI)
            let terms_ver = crate::about_info::APP_VERSION.to_string();
            let accepted_ver = self
                .state
                .config
                .settings
                .settings()
                .terms_accepted_version
                .as_ref();
            if accepted_ver != Some(&terms_ver) {
                crate::views::modals::terms::TermsModal::new(&terms_ver, &mut self.pending_action)
                    .show(ctx);
                return;
            }
        }

        if !splash_is_opaque {
            let download_req =
                crate::views::app_frame::MainPanels::new(self, &theme_colors).show(ctx);
            if let Some(req) = download_req {
                self.start_download(req);
            }
        }

        // Settings window
        if let Some(settings_action) =
            crate::settings::SettingsWindow::new(&mut self.state, &mut self.settings_preview)
                .show(ctx)
        {
            self.pending_action = settings_action;
        }

        if self.state.layout.show_search_modal {
            let mut is_open = true;
            let workspace_data = self.state.workspace.data.as_ref();
            crate::views::modals::search::SearchModal::new(
                &mut self.state.search,
                workspace_data,
                &mut is_open,
                &mut self.pending_action,
            )
            .show(ctx);
            if !is_open {
                self.pending_action = AppAction::ToggleSearchModal;
            }
        }

        // About dialog
        if self.show_about {
            crate::views::modals::about::AboutModal::new(
                &mut self.show_about,
                self.about_icon.as_ref(),
                &mut self.pending_action,
            )
            .show(ctx);
            if matches!(self.pending_action, AppAction::ShowReleaseNotes) {
                self.show_about = false;
            }
        }

        // Meta info dialog
        if let Some(path) = self.show_meta_info_for.clone() {
            let mut is_open = true;
            crate::views::modals::meta_info::MetaInfoModal::new(&mut is_open, &path).show(ctx);
            if !is_open {
                self.show_meta_info_for = None;
            }
        }

        // File system operation modals
        if let Some(mut modal_data) = self.state.layout.create_fs_node_modal.take() {
            let visible_extensions = &self
                .state
                .config
                .settings
                .settings()
                .workspace
                .visible_extensions;
            let close = crate::views::modals::file_ops::CreateFsNodeModal::new(
                &mut modal_data,
                visible_extensions,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.create_fs_node_modal = Some(modal_data);
            }
        }
        if let Some(mut modal_data) = self.state.layout.rename_modal.take() {
            let close = crate::views::modals::file_ops::RenameModal::new(
                &mut modal_data,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.rename_modal = Some(modal_data);
            }
        }
        if let Some(modal_data) = self.state.layout.delete_modal.take() {
            let close = crate::views::modals::file_ops::DeleteModal::new(
                &modal_data,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.delete_modal = Some(modal_data);
            }
        }

        // Update notification dialog
        if self.show_update_dialog {
            crate::views::modals::update::UpdateModal::new(
                &mut self.show_update_dialog,
                &self.state,
                &mut self.update_markdown_cache,
                &mut self.pending_action,
            )
            .show(ctx);
        }

        // Intercept all URL opening requests globally
        crate::views::app_frame::intercept_url_commands(ctx, self);

        // --- Splash Screen Overlay ---
        if let Some(start) = self.splash_start {
            let elapsed = start.elapsed().as_secs_f32();
            let dismissed =
                crate::views::splash::SplashOverlay::new(elapsed, self.about_icon.as_ref())
                    .show(ctx);
            if dismissed {
                self.splash_start = None;
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Persist the open tabs state right before the application process is terminated
        self.save_workspace_state();
    }
}

#[cfg(test)]
include!("shell_ui_tests.rs");
