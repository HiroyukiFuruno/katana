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

use crate::{
    app_state::{AppAction, AppState, ViewMode},
    preview_pane::DownloadRequest,
    widgets::StyledComboBox,
};

const INVISIBLE_LABEL_SIZE: f32 = 0.1;
/// Splash screen animation repaint interval (~30fps).
const SPLASH_REPAINT_INTERVAL_MS: u64 = 32;

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
// Split layout helpers
// ─────────────────────────────────────────────

// ─────────────────────────────────────────────
// eframe::App implementation (egui main rendering loop)
// ─────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod native_menu {
    // Must match the tag constants defined in macos_menu.m (Objective-C).
    pub const TAG_OPEN_WORKSPACE: i32 = 1;
    pub const TAG_SAVE: i32 = 2;
    pub const TAG_LANG_EN: i32 = 3;
    pub const TAG_LANG_JA: i32 = 4;
    pub const TAG_ABOUT: i32 = 5;
    pub const TAG_SETTINGS: i32 = 6;
    pub const TAG_LANG_ZH_CN: i32 = 7;
    pub const TAG_LANG_ZH_TW: i32 = 8;
    pub const TAG_LANG_KO: i32 = 9;
    pub const TAG_LANG_PT: i32 = 10;
    pub const TAG_LANG_FR: i32 = 11;
    pub const TAG_LANG_DE: i32 = 12;
    pub const TAG_LANG_ES: i32 = 13;
    pub const TAG_LANG_IT: i32 = 14;
    pub const TAG_CHECK_UPDATES: i32 = 15;
    pub const TAG_RELEASE_NOTES: i32 = 16;

    // These FFI symbols are linked from Objective-C (macos_menu.m) and called
    // only at runtime; the Rust compiler cannot see the call sites.
    #[allow(dead_code)]
    extern "C" {
        pub fn katana_setup_native_menu();
        pub fn katana_poll_menu_action() -> i32;
        pub fn katana_set_app_icon_png(png_data: *const u8, png_len: std::ffi::c_ulong);
        pub fn katana_set_process_name();
        pub fn native_free_menu_actions();
        pub fn katana_update_menu_strings(
            file: *const std::ffi::c_char,
            open_workspace: *const std::ffi::c_char,
            save: *const std::ffi::c_char,
            settings: *const std::ffi::c_char,
            preferences: *const std::ffi::c_char,
            language: *const std::ffi::c_char,
            about: *const std::ffi::c_char,
            quit: *const std::ffi::c_char,
            hide: *const std::ffi::c_char,
            hide_others: *const std::ffi::c_char,
            show_all: *const std::ffi::c_char,
            check_updates: *const std::ffi::c_char,
            help: *const std::ffi::c_char,
            release_notes: *const std::ffi::c_char,
        );
    }
}

/// Initializes the macOS native menu bar.
/// Called from main.rs after eframe creates the window.
///
/// # Safety
/// Contains Objective-C runtime calls. Must be called only once from the main thread.
#[cfg(all(target_os = "macos", not(test)))]
pub unsafe fn native_menu_setup() {
    native_menu::katana_setup_native_menu();
}

/// Sets the macOS process name to "KatanA".
/// Must be called at the very start of main(), BEFORE eframe creates the window,
/// so that the Dock label shows "KatanA" instead of the binary name.
///
/// # Safety
/// Contains Objective-C runtime calls. Must be called from the main thread.
#[cfg(all(target_os = "macos", not(test)))]
pub unsafe fn native_set_process_name() {
    native_menu::katana_set_process_name();
}

/// Sets the macOS application icon dynamically from PNG data.
///
/// # Safety
/// Contains Objective-C runtime calls. Must be called from the main thread.
#[cfg(all(target_os = "macos", not(test)))]
pub unsafe fn native_set_app_icon_png(png_data: *const u8, png_len: usize) {
    native_menu::katana_set_app_icon_png(png_data, png_len as std::ffi::c_ulong);
}

#[cfg(all(target_os = "macos", not(test)))]
#[allow(clippy::too_many_arguments)]
unsafe fn native_update_menu_strings(
    file: &str,
    open_workspace: &str,
    save: &str,
    settings: &str,
    preferences: &str,
    language: &str,
    about: &str,
    quit: &str,
    hide: &str,
    hide_others: &str,
    show_all: &str,
    check_updates: &str,
    help: &str,
    release_notes: &str,
) {
    let f = std::ffi::CString::new(file).unwrap_or_default();
    let ow = std::ffi::CString::new(open_workspace).unwrap_or_default();
    let s = std::ffi::CString::new(save).unwrap_or_default();
    let st = std::ffi::CString::new(settings).unwrap_or_default();
    let p = std::ffi::CString::new(preferences).unwrap_or_default();
    let l = std::ffi::CString::new(language).unwrap_or_default();
    let a = std::ffi::CString::new(about).unwrap_or_default();
    let q = std::ffi::CString::new(quit).unwrap_or_default();
    let h = std::ffi::CString::new(hide).unwrap_or_default();
    let ho = std::ffi::CString::new(hide_others).unwrap_or_default();
    let sa = std::ffi::CString::new(show_all).unwrap_or_default();
    let cu = std::ffi::CString::new(check_updates).unwrap_or_default();
    let hlp = std::ffi::CString::new(help).unwrap_or_default();
    let rn = std::ffi::CString::new(release_notes).unwrap_or_default();
    native_menu::katana_update_menu_strings(
        f.as_ptr(),
        ow.as_ptr(),
        s.as_ptr(),
        st.as_ptr(),
        p.as_ptr(),
        l.as_ptr(),
        a.as_ptr(),
        q.as_ptr(),
        h.as_ptr(),
        ho.as_ptr(),
        sa.as_ptr(),
        cu.as_ptr(),
        hlp.as_ptr(),
        rn.as_ptr(),
    );
}

#[cfg(all(target_os = "macos", not(test)))]
pub fn update_native_menu_strings_from_i18n() {
    let msgs = crate::i18n::get();
    let preferences = format!("{}…", msgs.menu.settings);
    unsafe {
        native_update_menu_strings(
            &msgs.menu.file,
            &msgs.menu.open_workspace,
            &msgs.menu.save,
            &msgs.menu.settings,
            &preferences,
            &msgs.menu.language,
            &msgs.menu.about,
            &msgs.menu.quit,
            &msgs.menu.hide,
            &msgs.menu.hide_others,
            &msgs.menu.show_all,
            &msgs.menu.check_updates,
            &msgs.menu.help,
            &msgs.menu.release_notes,
        );
    }
}

#[cfg(any(not(target_os = "macos"), test))]
pub fn update_native_menu_strings_from_i18n() {}

// ─────────────────────────────────────────────
// eframe::App Implementation (egui Main Rendering Loop)
// ─────────────────────────────────────────────

use crate::shell::{KatanaApp, SIDEBAR_COLLAPSED_TOGGLE_WIDTH};

// Half-panel ratio for responsive 50/50 split.
pub(crate) const SPLIT_HALF_RATIO: f32 = 0.5;
/// Maximum ratio for TopBottomPanel in vertical split.
/// Prevents preview from consuming more than 70% of the available height,
/// guaranteeing the editor retains at least 30% for scrolling.
pub(crate) const SPLIT_PANEL_MAX_RATIO: f32 = 0.7;
pub(crate) const PREVIEW_CONTENT_PADDING: i8 = 12;

impl KatanaApp {
    const TERMS_MODAL_WIDTH: f32 = 600.0;
    const TERMS_TITLE_SIZE: f32 = 28.0;
    const TERMS_INNER_MARGIN: f32 = 24.0;
    const TERMS_CONVAS_MARGIN: f32 = 16.0;
    const TERMS_ROUNDING_LARGE: f32 = 12.0;
    const TERMS_ROUNDING_SMALL: f32 = 8.0;
    const TERMS_SPACING_SMALL: f32 = 8.0;
    const TERMS_SPACING_MEDIUM: f32 = 20.0;
    const TERMS_SPACING_XLARGE: f32 = 32.0;
    const TERMS_BUTTON_WIDTH: f32 = 120.0;
    const TERMS_BUTTON_HEIGHT: f32 = 40.0;
    const TERMS_BUTTON_TEXT_SIZE: f32 = 16.0;
    const TERMS_BUTTON_SPACING: f32 = 24.0;
    const TERMS_SCROLL_HEIGHT_RATIO: f32 = 0.5;
    const TERMS_CENTER_OFFSET_RATIO: f32 = 0.1;
    const TERMS_LANG_SELECT_WIDTH: f32 = 140.0;

    fn render_terms_modal(&mut self, ctx: &egui::Context, version: &str) {
        let terms = crate::i18n::get().terms.clone();

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
                let width = ui.available_width();
                let height = ui.available_height();
                let content_width = width.min(Self::TERMS_MODAL_WIDTH);

                ui.vertical_centered(|ui| {
                    ui.add_space(height * Self::TERMS_CENTER_OFFSET_RATIO);

                    ui.set_width(content_width);

                    egui::Frame::window(ui.style())
                        .inner_margin(Self::TERMS_INNER_MARGIN)
                        .corner_radius(Self::TERMS_ROUNDING_LARGE)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading(
                                    egui::RichText::new(&terms.title)
                                        .size(Self::TERMS_TITLE_SIZE)
                                        .strong()
                                        .color(ui.visuals().strong_text_color()),
                                );
                                ui.add_space(Self::TERMS_SPACING_SMALL);

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(crate::i18n::tf(
                                            &terms.version_label,
                                            &[("version", version)],
                                        ))
                                        .weak(),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let current_lang = crate::i18n::get_language();
                                            let current_name = crate::i18n::supported_languages()
                                                .iter()
                                                .find(|(code, _)| *code == current_lang)
                                                .map(|(_, name)| name.as_str())
                                                .unwrap_or("English");

                                            StyledComboBox::new("terms_lang_select", current_name)
                                                .width(Self::TERMS_LANG_SELECT_WIDTH)
                                                .show(ui, |ui| {
                                                    for (code, name) in
                                                        crate::i18n::supported_languages()
                                                    {
                                                        if ui
                                                            .selectable_label(
                                                                current_lang == *code,
                                                                name,
                                                            )
                                                            .clicked()
                                                        {
                                                            self.pending_action =
                                                                AppAction::ChangeLanguage(
                                                                    code.clone(),
                                                                );
                                                        }
                                                    }
                                                });
                                        },
                                    );
                                });

                                ui.add_space(Self::TERMS_SPACING_MEDIUM);
                                ui.separator();
                                ui.add_space(Self::TERMS_SPACING_MEDIUM);

                                egui::Frame::canvas(ui.style())
                                    .inner_margin(Self::TERMS_CONVAS_MARGIN)
                                    .corner_radius(Self::TERMS_ROUNDING_SMALL)
                                    .show(ui, |ui| {
                                        ui.set_min_height(
                                            ui.available_height() * Self::TERMS_SCROLL_HEIGHT_RATIO,
                                        );
                                        egui::ScrollArea::vertical()
                                            .max_height(
                                                ui.available_height()
                                                    * Self::TERMS_SCROLL_HEIGHT_RATIO,
                                            )
                                            .show(ui, |ui| {
                                                ui.add(egui::Label::new(&terms.content).wrap());
                                            });
                                    });

                                ui.add_space(Self::TERMS_SPACING_XLARGE);

                                ui.horizontal(|ui| {
                                    let total_buttons_width =
                                        Self::TERMS_BUTTON_WIDTH * 2.0 + Self::TERMS_BUTTON_SPACING;
                                    let available = ui.available_width();
                                    let outer_spacing = (available - total_buttons_width) / 2.0;

                                    if outer_spacing > 0.0 {
                                        ui.add_space(outer_spacing);
                                    }

                                    let accept_btn = egui::Button::new(
                                        egui::RichText::new(&terms.accept)
                                            .strong()
                                            .size(Self::TERMS_BUTTON_TEXT_SIZE),
                                    )
                                    .min_size(egui::vec2(
                                        Self::TERMS_BUTTON_WIDTH,
                                        Self::TERMS_BUTTON_HEIGHT,
                                    ))
                                    .corner_radius(Self::TERMS_ROUNDING_SMALL);

                                    if ui
                                        .add(accept_btn)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        self.pending_action =
                                            AppAction::AcceptTerms(version.to_string());
                                    }

                                    ui.add_space(Self::TERMS_BUTTON_SPACING);

                                    let decline_btn = egui::Button::new(
                                        egui::RichText::new(&terms.decline)
                                            .size(Self::TERMS_BUTTON_TEXT_SIZE),
                                    )
                                    .min_size(egui::vec2(
                                        Self::TERMS_BUTTON_WIDTH,
                                        Self::TERMS_BUTTON_HEIGHT,
                                    ))
                                    .corner_radius(Self::TERMS_ROUNDING_SMALL);

                                    if ui
                                        .add(decline_btn)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        self.pending_action = AppAction::DeclineTerms;
                                    }
                                });
                                ui.add_space(Self::TERMS_SPACING_MEDIUM);
                            });
                        });
                });
            });
    }
}

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
        #[cfg(target_os = "macos")]
        {
            let action = unsafe { native_menu::katana_poll_menu_action() };
            match action {
                native_menu::TAG_OPEN_WORKSPACE => {
                    if let Some(path) = open_folder_dialog() {
                        self.pending_action = AppAction::OpenWorkspace(path);
                    }
                }
                native_menu::TAG_SAVE => {
                    self.pending_action = AppAction::SaveDocument;
                }
                native_menu::TAG_LANG_EN => {
                    self.pending_action = AppAction::ChangeLanguage("en".to_string());
                }
                native_menu::TAG_LANG_JA => {
                    self.pending_action = AppAction::ChangeLanguage("ja".to_string());
                }
                native_menu::TAG_LANG_ZH_CN => {
                    self.pending_action = AppAction::ChangeLanguage("zh-CN".to_string());
                }
                native_menu::TAG_LANG_ZH_TW => {
                    self.pending_action = AppAction::ChangeLanguage("zh-TW".to_string());
                }
                native_menu::TAG_LANG_KO => {
                    self.pending_action = AppAction::ChangeLanguage("ko".to_string());
                }
                native_menu::TAG_LANG_PT => {
                    self.pending_action = AppAction::ChangeLanguage("pt".to_string());
                }
                native_menu::TAG_LANG_FR => {
                    self.pending_action = AppAction::ChangeLanguage("fr".to_string());
                }
                native_menu::TAG_LANG_DE => {
                    self.pending_action = AppAction::ChangeLanguage("de".to_string());
                }
                native_menu::TAG_LANG_ES => {
                    self.pending_action = AppAction::ChangeLanguage("es".to_string());
                }
                native_menu::TAG_LANG_IT => {
                    self.pending_action = AppAction::ChangeLanguage("it".to_string());
                }
                native_menu::TAG_ABOUT => {
                    self.show_about = !self.show_about;
                }
                native_menu::TAG_CHECK_UPDATES => {
                    self.pending_action = AppAction::CheckForUpdates;
                }
                native_menu::TAG_RELEASE_NOTES => {
                    self.pending_action = AppAction::ShowReleaseNotes;
                }
                native_menu::TAG_SETTINGS => {
                    self.pending_action = AppAction::ToggleSettings;
                }
                _ => {}
            }
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
                self.render_terms_modal(ctx, &terms_ver);
                return;
            }
        }

        if !splash_is_opaque {
            // On macOS, the egui menu is hidden because the native menu bar is used.
            crate::views::top_bar::render_menu_bar(ctx, &mut self.state, &mut self.pending_action);
            let export_filenames: Vec<String> = self
                .export_tasks
                .iter()
                .map(|t| t.filename.clone())
                .collect();
            crate::views::top_bar::render_status_bar(ctx, &self.state, &export_filenames);

            // Reflect the file name in the window title
            let ws_root_for_title = self.state.workspace.data.as_ref().map(|ws| ws.root.clone());
            let title_text = match self.state.active_document() {
                Some(doc) => {
                    let fname = doc.file_name().unwrap_or("");
                    let rel = relative_full_path(&doc.path, ws_root_for_title.as_deref());
                    crate::shell_logic::format_window_title(
                        fname,
                        &rel,
                        &crate::i18n::get().menu.release_notes,
                    )
                }
                None => "KatanA".to_string(),
            };
            if self.state.layout.last_window_title != title_text {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));
                self.state.layout.last_window_title = title_text.clone();
            }

            // In-app title bar
            egui::TopBottomPanel::top("app_title_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.centered_and_justified(|ui| {
                        let title_color =
                            theme_bridge::rgb_to_color32(theme_colors.system.title_bar_text);
                        ui.label(egui::RichText::new(&title_text).small().color(title_color));
                    });
                });
            });

            // Show the collapse toggle button even when the workspace is hidden.
            if !self.state.layout.show_workspace {
                egui::SidePanel::left("workspace_collapsed")
                    .resizable(false)
                    .exact_width(SIDEBAR_COLLAPSED_TOGGLE_WIDTH)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            if ui
                                .add(egui::Button::image(
                                    crate::Icon::ChevronRight
                                        .ui_image(ui, crate::icon::IconSize::Medium),
                                ))
                                .on_hover_text(crate::i18n::get().workspace.workspace_title.clone())
                                .clicked()
                            {
                                self.state.layout.show_workspace = true;
                            }
                        });
                    });
            } else {
                crate::views::panels::workspace::render_workspace_panel(
                    ctx,
                    &mut self.state,
                    &mut self.pending_action,
                );
            }

            // Tab row + breadcrumbs + view mode row
            egui::TopBottomPanel::top("tab_toolbar").show(ctx, |ui| {
                crate::views::top_bar::render_tab_bar(ui, &mut self.state, &mut self.pending_action);
                let active_doc_props = self.state.active_document();
                if let Some(doc) = active_doc_props {
                    let d_path = doc.path.to_string_lossy();
                    let is_changelog = d_path.starts_with("Katana://ChangeLog");

                    if !is_changelog {
                        let doc_path = doc.path.clone();
                        let ws_root = self.state.workspace.data.as_ref().map(|ws| ws.root.clone());
                        let rel = relative_full_path(&doc_path, ws_root.as_deref());
                        let mut breadcrumb_action = None;
                        ui.horizontal(|ui| {
                            let segments: Vec<&str> = rel.split('/').collect();
                            let mut current_path = ws_root.clone().unwrap_or_default();
                            for (i, seg) in segments.iter().enumerate() {
                                if i > 0 {
                                    const CHEVRON_ICON_SIZE: f32 = 10.0;
                                    ui.add(
                                        egui::Image::new(crate::Icon::ChevronRight.uri())
                                            .tint(ui.visuals().text_color())
                                            .max_height(CHEVRON_ICON_SIZE),
                                    );
                                }

                                if ws_root.is_none() {
                                    ui.label(egui::RichText::new(*seg).small());
                                    continue;
                                }

                                current_path = current_path.join(seg);
                                let is_last = i == segments.len() - 1;

                                // The doc_path_clone is only needed for the original `render_tree_entry` logic,
                                // which is being replaced.
                                // let doc_path_clone = doc_path.clone();

                                if is_last {
                                    // Last segment is the file itself, just render it as text
                                    ui.add(
                                        egui::Label::new(egui::RichText::new(*seg).small())
                                            .sense(egui::Sense::hover()),
                                    );
                                } else {
                                    // Dropdown mini-workspace for Breadcrumbs
                                    ui.menu_button(egui::RichText::new(*seg).small(), |ui| {
                                        let mut ctx_action = crate::app_state::AppAction::None;

                                        if let Some(ws) = &self.state.workspace.data {
                                            if let Some(
                                                katana_core::workspace::TreeEntry::Directory {
                                                    children,
                                                    ..
                                                },
                                            ) = crate::views::panels::tree::find_node_in_tree(&ws.tree, &current_path)
                                            {
                                                crate::views::panels::workspace::render_breadcrumb_menu(
                                                    ui,
                                                    children,
                                                    &mut ctx_action,
                                                );
                                            }
                                        }

                                        if !matches!(ctx_action, crate::app_state::AppAction::None)
                                        {
                                            breadcrumb_action = Some(ctx_action);
                                            ui.close();
                                        }
                                    });
                                }
                            }
                        });
                        if let Some(a) = breadcrumb_action {
                            self.pending_action = a;
                        }
                    } // End if !is_changelog
                    crate::views::top_bar::render_view_mode_bar(ui, &mut self.state, &mut self.pending_action);
                }
            });

            let mut download_req: Option<DownloadRequest> = None;
            let current_mode = self.state.active_view_mode();
            let is_split = current_mode == ViewMode::Split;
            let mut is_changelog_tab = false;

            if let Some(doc) = self.state.active_document() {
                if doc.path.to_string_lossy().starts_with("Katana://ChangeLog") {
                    is_changelog_tab = true;
                }
            }

            if self.state.layout.show_toc
                && self.state.config.settings.settings().layout.toc_visible
            {
                if let Some(doc) = self.state.active_document() {
                    if let Some(preview) = self.tab_previews.iter_mut().find(|p| p.path == doc.path)
                    {
                        crate::views::panels::toc::render_toc_panel(
                            ctx,
                            &mut preview.pane,
                            &self.state,
                        );
                    }
                }
            }

            if is_changelog_tab {
                egui::CentralPanel::default().show(ctx, |ui| {
                    crate::changelog::render_release_notes_tab(
                        ui,
                        &self.changelog_sections,
                        self.changelog_rx.is_some(),
                    );
                });
            } else {
                if is_split {
                    let split_dir = self.state.active_split_direction();
                    let pane_order = self.state.active_pane_order();
                    download_req = crate::views::layout::split::render_split_mode(
                        ctx, self, split_dir, pane_order,
                    );
                }

                if !is_split {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                        .show(ctx, |ui| match current_mode {
                            ViewMode::CodeOnly => {
                                crate::views::panels::editor::render_editor_content(
                                    ui,
                                    &mut self.state,
                                    &mut self.pending_action,
                                    false,
                                );
                            }
                            ViewMode::PreviewOnly => {
                                crate::views::layout::split::render_preview_only(ui, self);
                            }
                            ViewMode::Split => {}
                        });
                }
            }

            if let Some(req) = download_req {
                self.start_download(req);
            }
        }

        // Settings window
        if let Some(settings_action) = crate::settings_window::render_settings_window(
            ctx,
            &mut self.state,
            &mut self.settings_preview,
        ) {
            self.pending_action = settings_action;
        }

        if self.state.layout.show_search_modal {
            crate::views::modals::search::render_search_modal(
                ctx,
                &mut self.state,
                &mut self.pending_action,
            );
        }

        // About dialog
        if self.show_about {
            render_about_window(
                ctx,
                &mut self.show_about,
                self.about_icon.as_ref(),
                &mut self.pending_action,
            );
            if matches!(self.pending_action, AppAction::ShowReleaseNotes) {
                self.show_about = false;
            }
        }

        // Meta info dialog
        if let Some(path) = self.show_meta_info_for.clone() {
            let mut is_open = true;
            render_meta_info_window(ctx, &mut is_open, &path);
            if !is_open {
                self.show_meta_info_for = None;
            }
        }

        // File system operation modals
        if self.state.layout.create_fs_node_modal.is_some() {
            render_create_fs_node_modal(ctx, &mut self.state, &mut self.pending_action);
        }
        if self.state.layout.rename_modal.is_some() {
            render_rename_modal(ctx, &mut self.state, &mut self.pending_action);
        }
        if self.state.layout.delete_modal.is_some() {
            render_delete_modal(ctx, &mut self.state, &mut self.pending_action);
        }

        // Update notification dialog
        if self.show_update_dialog {
            render_update_window(
                ctx,
                &mut self.show_update_dialog,
                &self.state,
                &mut self.update_markdown_cache,
                &mut self.pending_action,
            );
        }

        // Intercept all URL opening requests globally
        let commands = ctx.output_mut(|o| std::mem::take(&mut o.commands));
        let mut unprocessed_commands = Vec::new();

        for cmd in commands {
            if let egui::OutputCommand::OpenUrl(open) = &cmd {
                let url = &open.url;
                if url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with("mailto:")
                {
                    // Let eframe natively handle external URLs so it respects same_tab vs new_tab
                    unprocessed_commands.push(cmd);
                } else {
                    let mut path = std::path::PathBuf::from(url);
                    if path.is_relative() {
                        // Resolve relative link against current active document's parent char
                        if let Some(doc) = self.state.active_document() {
                            if let Some(parent) = doc.path.parent() {
                                path = parent.join(path);
                            }
                        }
                    }
                    self.process_action(ctx, AppAction::SelectDocument(path));
                }
            } else {
                unprocessed_commands.push(cmd);
            }
        }

        // Put back the commands we didn't handle
        if !unprocessed_commands.is_empty() {
            ctx.output_mut(|o| o.commands.extend(unprocessed_commands));
        }

        // --- Splash Screen Overlay ---
        if let Some(start) = self.splash_start {
            let elapsed = start.elapsed().as_secs_f32();
            let opacity = crate::shell_logic::calculate_splash_opacity(elapsed);
            let any_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

            if opacity <= 0.0 || any_pressed {
                self.splash_start = None;
            } else {
                egui::Area::new(egui::Id::new("splash_screen_area"))
                    .order(egui::Order::Foreground)
                    .interactable(true) // Consume interactions directly falling through
                    .show(ctx, |ui| {
                        const SPLASH_BG_DARK: u8 = 30;
                        const SPLASH_BG_LIGHT: u8 = 240;
                        const SPLASH_ICON_SIZE: f32 = 128.0;
                        const SPLASH_ICON_SPACING: f32 = 16.0;
                        const SPLASH_HEADING_SIZE: f32 = 32.0;
                        const SPLASH_HEADING_SPACING: f32 = 8.0;
                        const SPLASH_VERSION_SIZE: f32 = 16.0;
                        const SPLASH_PROGRESS_SPACING: f32 = 24.0;
                        const SPLASH_PROGRESS_WIDTH: f32 = 240.0;
                        const SPLASH_PROGRESS_PHASE1: f32 = 0.25;
                        const SPLASH_PROGRESS_PHASE2: f32 = 0.6;
                        const SPLASH_PROGRESS_PHASE3: f32 = 0.95;
                        const SPLASH_PROGRESS_TEXT_SIZE: f32 = 12.0;
                        const SPLASH_PROGRESS_TEXT_DIM: f32 = 0.7;
                        const SPLASH_PROGRESS_BAR_MARGIN: f32 = 4.0;
                        const SPLASH_PROGRESS_BG_LIGHT: u8 = 100;
                        const SPLASH_PROGRESS_BG_DARK: u8 = 200;

                        let is_dark = ctx.style().visuals.dark_mode;
                        #[allow(deprecated)]
                        let content_rect = ctx.screen_rect();
                        let bg_color = if is_dark {
                            crate::theme_bridge::from_rgb(
                                SPLASH_BG_DARK,
                                SPLASH_BG_DARK,
                                SPLASH_BG_DARK,
                            )
                        } else {
                            crate::theme_bridge::from_rgb(
                                SPLASH_BG_LIGHT,
                                SPLASH_BG_LIGHT,
                                SPLASH_BG_LIGHT,
                            )
                        };
                        let fill_color = bg_color.gamma_multiply(opacity);
                        ui.painter().rect_filled(content_rect, 1.0, fill_color);

                        let text_color = if is_dark {
                            crate::theme_bridge::WHITE
                        } else {
                            crate::theme_bridge::TRANSPARENT
                        }
                        .gamma_multiply(opacity);

                        // Calculate total content height to vertically center the splash content
                        const SPLASH_CONTENT_HEIGHT: f32 = SPLASH_ICON_SIZE
                            + SPLASH_ICON_SPACING
                            + SPLASH_HEADING_SIZE
                            + SPLASH_HEADING_SPACING
                            + SPLASH_VERSION_SIZE
                            + SPLASH_PROGRESS_SPACING
                            + SPLASH_PROGRESS_TEXT_SIZE
                            + SPLASH_PROGRESS_BAR_MARGIN
                            + SPLASH_PROGRESS_SPACING; // approximate total height

                        let center = content_rect.center();
                        let centered_rect = egui::Rect::from_center_size(
                            center,
                            egui::vec2(content_rect.width(), SPLASH_CONTENT_HEIGHT),
                        );

                        ui.scope_builder(egui::UiBuilder::new().max_rect(centered_rect), |ui| {
                            ui.vertical_centered(|ui| {
                                if let Some(tex) = &self.about_icon {
                                    ui.image(egui::load::SizedTexture::new(
                                        tex.id(),
                                        egui::vec2(SPLASH_ICON_SIZE, SPLASH_ICON_SIZE),
                                    ));
                                    ui.add_space(SPLASH_ICON_SPACING);
                                }
                                let heading =
                                    egui::RichText::new(crate::about_info::APP_DISPLAY_NAME)
                                        .strong()
                                        .size(SPLASH_HEADING_SIZE)
                                        .color(text_color);
                                ui.label(heading);

                                ui.add_space(SPLASH_HEADING_SPACING);

                                let version_str = format!("Version {}", env!("CARGO_PKG_VERSION"));
                                let version = egui::RichText::new(version_str)
                                    .size(SPLASH_VERSION_SIZE)
                                    .color(text_color);
                                ui.label(version);

                                ui.add_space(SPLASH_PROGRESS_SPACING);
                                let progress =
                                    crate::shell_logic::calculate_splash_progress(elapsed);

                                let progress_text = if progress < SPLASH_PROGRESS_PHASE1 {
                                    "Initializing Katana engine..."
                                } else if progress < SPLASH_PROGRESS_PHASE2 {
                                    "Parsing workspace structure..."
                                } else if progress < SPLASH_PROGRESS_PHASE3 {
                                    "Increasing context size... w"
                                } else {
                                    "Ready."
                                };

                                ui.label(
                                    egui::RichText::new(progress_text)
                                        .size(SPLASH_PROGRESS_TEXT_SIZE)
                                        .color(text_color.gamma_multiply(SPLASH_PROGRESS_TEXT_DIM)),
                                );
                                ui.add_space(SPLASH_PROGRESS_BAR_MARGIN);
                                let progress_bar = egui::ProgressBar::new(progress)
                                    .desired_width(SPLASH_PROGRESS_WIDTH)
                                    .show_percentage();

                                // Add a little visual flair to the progress bar by tinting it based on the theme
                                if !is_dark {
                                    ui.visuals_mut().selection.bg_fill =
                                        crate::theme_bridge::from_rgb(
                                            SPLASH_PROGRESS_BG_LIGHT,
                                            SPLASH_PROGRESS_BG_LIGHT,
                                            SPLASH_PROGRESS_BG_LIGHT,
                                        )
                                        .gamma_multiply(opacity);
                                } else {
                                    ui.visuals_mut().selection.bg_fill =
                                        crate::theme_bridge::from_rgb(
                                            SPLASH_PROGRESS_BG_DARK,
                                            SPLASH_PROGRESS_BG_DARK,
                                            SPLASH_PROGRESS_BG_DARK,
                                        )
                                        .gamma_multiply(opacity);
                                }
                                ui.add(progress_bar);
                            });
                        });
                    });
                // Animate splash screen (fade in/out, progress text)
                ctx.request_repaint_after(std::time::Duration::from_millis(
                    SPLASH_REPAINT_INTERVAL_MS,
                ));
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Persist the open tabs state right before the application process is terminated
        self.save_workspace_state();
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    use crate::app_state::ScrollSource;
    use crate::preview_pane::PreviewPane;
    use katana_platform::PaneOrder;

    use eframe::egui::{self, pos2, Rect};
    use eframe::App as _;
    use egui::load::{BytesLoadResult, BytesLoader, LoadError};
    use katana_core::{document::Document, workspace::TreeEntry};
    use std::path::{Path, PathBuf};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    pub(crate) const PREVIEW_CONTENT_PADDING: f32 = 12.0;

    /// Custom testing egui Context that pre-populates dummy font mappings for Markdown
    /// layout families. PreviewPane panics if these are missing natively.
    fn test_context() -> egui::Context {
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        let md_prop = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .cloned()
            .unwrap_or_default();
        let md_mono = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .cloned()
            .unwrap_or_default();
        fonts.families.insert(
            egui::FontFamily::Name("MarkdownProportional".into()),
            md_prop,
        );
        fonts
            .families
            .insert(egui::FontFamily::Name("MarkdownMonospace".into()), md_mono);
        ctx.set_fonts(fonts);
        ctx
    }

    fn test_input(size: egui::Vec2) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), size)),
            ..Default::default()
        }
    }

    fn flatten_shapes<'a>(
        shapes: impl IntoIterator<Item = &'a egui::epaint::ClippedShape>,
    ) -> Vec<&'a egui::epaint::Shape> {
        fn visit<'a>(shape: &'a egui::epaint::Shape, acc: &mut Vec<&'a egui::epaint::Shape>) {
            match shape {
                egui::epaint::Shape::Vec(children) => {
                    for child in children {
                        visit(child, acc);
                    }
                }
                _ => acc.push(shape),
            }
        }

        let mut flat = Vec::new();
        for clipped in shapes {
            visit(&clipped.shape, &mut flat);
        }
        flat
    }

    fn state_with_active_doc(path: &std::path::Path) -> AppState {
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state
            .document
            .open_documents
            .push(Document::new(path, "# Heading\n\nBody"));
        state.document.active_doc_idx = Some(0);
        state
    }

    fn app_with_preview_doc(path: &Path, markdown: &str) -> KatanaApp {
        let mut app = KatanaApp::new(state_with_active_doc(path));
        if let Some(doc) = app.state.active_document_mut() {
            doc.buffer = markdown.to_string();
        }
        let mut pane = PreviewPane::default();
        let cache = app.state.config.cache.clone();
        let concurrency = app
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        pane.full_render(markdown, path, cache, false, concurrency);
        pane.wait_for_renders();
        app.tab_previews.push(crate::shell::TabPreviewCache {
            path: path.to_path_buf(),
            pane,
            hash: 0,
        });
        app
    }

    struct CountingBytesLoader {
        forget_all_calls: Arc<AtomicUsize>,
    }

    impl BytesLoader for CountingBytesLoader {
        fn id(&self) -> &str {
            egui::generate_loader_id!(CountingBytesLoader)
        }

        fn load(&self, _ctx: &egui::Context, _uri: &str) -> BytesLoadResult {
            Err(LoadError::NotSupported)
        }

        fn forget(&self, _uri: &str) {}

        fn forget_all(&self) {
            self.forget_all_calls.fetch_add(1, Ordering::SeqCst);
        }

        fn byte_size(&self) -> usize {
            0
        }

        fn has_pending(&self) -> bool {
            false
        }
    }

    #[test]
    fn preview_header_leaves_height_for_preview_body() {
        let ctx = test_context();
        let state = state_with_active_doc(std::path::Path::new("/tmp/preview.md"));
        let mut action = AppAction::None;
        let mut before_height = 0.0;
        let mut remaining_height = 0.0;

        let _ = ctx.run(test_input(egui::vec2(800.0, 600.0)), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                .show(ctx, |ui| {
                    before_height = ui.available_height();
                    crate::views::panels::preview::render_preview_header(ui, &state, &mut action);
                    remaining_height = ui.available_height();
                });
        });

        assert!(
            (before_height - remaining_height).abs() <= 1.0,
            "preview header must overlay without consuming layout height, before={before_height}, after={remaining_height}"
        );
    }

    #[test]
    fn active_file_highlight_is_painted_before_text() {
        let ctx = test_context();
        let path = std::path::PathBuf::from("/tmp/CHANGELOG.md");
        let entry = TreeEntry::File { path: path.clone() };
        let mut action = AppAction::None;
        let mut expanded_directories = std::collections::HashSet::new();

        let output = ctx.run(test_input(egui::vec2(320.0, 200.0)), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                .show(ctx, |ui| {
                    let mut render_ctx = TreeRenderContext {
                        action: &mut action,
                        depth: 0,
                        active_path: Some(path.as_path()),
                        filter_set: None,
                        expanded_directories: &mut expanded_directories,
                        disable_context_menu: false,
                    };
                    crate::views::panels::workspace::render_file_entry(
                        ui,
                        &entry,
                        &path,
                        &mut render_ctx,
                    );
                });
        });

        let shapes = flatten_shapes(output.shapes.iter());
        let highlight_idx = shapes.iter().position(|shape| {
            matches!(
                shape,
                egui::epaint::Shape::Rect(rect)
                    if rect.fill == ctx.style().visuals.selection.bg_fill
            )
        });
        let text_idx = shapes.iter().position(|shape| {
            matches!(
                shape,
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains("CHANGELOG.md")
            )
        });

        let highlight_idx = highlight_idx.expect("active row highlight was not painted");
        let text_idx = text_idx.expect("active row label text was not painted");

        assert!(
            highlight_idx < text_idx,
            "active row background must be behind its text, got rect index {highlight_idx} and text index {text_idx}"
        );
    }

    #[test]
    fn split_preview_left_padding_is_consistent() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/padding.md");
        let mut app = app_with_preview_doc(&path, "# PaddingHeading\n\nBody");
        let output = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let heading_rect = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains("PaddingHeading") =>
                {
                    let rect = text.visual_bounding_rect();
                    if rect.left() >= preview_rect.left() - 1.0 {
                        Some(rect)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .expect("heading text shape");

        let left_padding = heading_rect.left() - preview_rect.left();
        assert!(
            (left_padding - PREVIEW_CONTENT_PADDING).abs() <= 2.0,
            "preview left padding must be {}px, got {left_padding}",
            PREVIEW_CONTENT_PADDING
        );
    }

    #[test]
    fn new_horizontal_split_starts_at_half_width_even_if_another_tab_has_panel_state() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/active.md");
        let stale = PathBuf::from("/tmp/stale.md");
        let mut app = app_with_preview_doc(&active, "Body");

        ctx.data_mut(|data| {
            data.insert_persisted(
                crate::views::panels::preview::preview_panel_id(
                    Some(stale.as_path()),
                    "preview_panel_h_right",
                ),
                egui::containers::panel::PanelState {
                    rect: Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(240.0, 800.0)),
                },
            );
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        assert!(
            (preview_rect.width() - 600.0).abs() <= 4.0,
            "fresh horizontal split must start at 50%, got {}",
            preview_rect.width()
        );
    }

    #[test]
    fn horizontal_split_width_stays_stable_across_initial_frames() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/active.md");
        let mut app = app_with_preview_doc(&active, "# Title\n\nBody");

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after second frame")
        .rect;

        assert!(
            (first_rect.width() - 600.0).abs() <= 4.0,
            "first frame must start at 50%, got {}",
            first_rect.width()
        );
        assert!(
            (second_rect.width() - first_rect.width()).abs() <= 4.0,
            "horizontal split width must remain stable across frames, first={} second={}",
            first_rect.width(),
            second_rect.width()
        );
    }

    #[test]
    fn horizontal_split_width_stays_stable_with_readme_like_preview_content() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/readme.md");
        let markdown = concat!(
            "# KatanA Desktop\n\n",
            "> Note: On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple.\n",
            "> Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n\n",
            "Current Status\n\n",
            "KatanA Desktop is under active development. See the Releases page for the latest version and changelog.\n"
        );
        let mut app = app_with_preview_doc(&active, markdown);

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after second frame")
        .rect;

        assert!(
            (first_rect.width() - 600.0).abs() <= 4.0,
            "first frame must start at 50%, got {}",
            first_rect.width()
        );
        assert!(
            (second_rect.width() - first_rect.width()).abs() <= 4.0,
            "horizontal split width must remain stable with README-like preview content, first={} second={}",
            first_rect.width(),
            second_rect.width()
        );
    }

    #[test]
    fn horizontal_split_width_stays_stable_with_changelog_like_list_content() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/changelog.md");
        let markdown = concat!(
            "## Fixes\n\n",
            "- Dark theme DrawIO contrast fix using `drawio_label_color`\n",
            "- Fixed `mmdc` lookup when launched from `.dmg/.app` without a standard PATH\n",
            "- Stabilized i18n tests under parallel execution\n\n",
            "## Improvements\n\n",
            "- Expanded `mmdc` binary resolution across Homebrew, env vars, and shell fallback\n",
            "- Extracted `CHANNEL_MAX`, `LUMA_R/G/B`, and `RENDER_POLL_INTERVAL_MS`\n"
        );
        let mut app = app_with_preview_doc(&active, markdown);

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after second frame")
        .rect;

        assert!(
            (first_rect.width() - 600.0).abs() <= 4.0,
            "first frame must start at 50%, got {}",
            first_rect.width()
        );
        assert!(
            (second_rect.width() - first_rect.width()).abs() <= 4.0,
            "horizontal split width must remain stable with changelog-like list content, first={} second={}",
            first_rect.width(),
            second_rect.width()
        );
    }

    #[test]
    fn new_vertical_split_starts_at_half_height_even_if_another_tab_has_panel_state() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/active.md");
        let stale = PathBuf::from("/tmp/stale.md");
        let mut app = app_with_preview_doc(&active, "Body");

        ctx.data_mut(|data| {
            data.insert_persisted(
                crate::views::panels::preview::preview_panel_id(
                    Some(stale.as_path()),
                    "preview_panel_v_bottom",
                ),
                egui::containers::panel::PanelState {
                    rect: Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1200.0, 180.0)),
                },
            );
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_vertical_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_v_bottom",
            ),
        )
        .expect("preview panel rect")
        .rect;
        assert!(
            (preview_rect.height() - 400.0).abs() <= 4.0,
            "fresh vertical split must start at 50%, got {}",
            preview_rect.height()
        );
    }

    #[test]
    fn split_preview_wraps_long_lines_without_horizontal_overflow() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/long-line.md");
        let long_line = "\u{3042}".repeat(240);
        let mut app = app_with_preview_doc(&path, &long_line);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let text_shape = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains(&long_line[..60]) =>
                {
                    Some(text)
                }
                _ => None,
            })
            .expect("long preview text shape");

        assert!(
            text_shape.galley.rows.len() > 1,
            "long preview line must wrap instead of staying on a single row"
        );
        assert!(
            text_shape.visual_bounding_rect().right()
                <= preview_rect.right() - PREVIEW_CONTENT_PADDING + 4.0,
            "wrapped preview text must stay within the preview panel"
        );
    }

    #[test]
    fn split_preview_wraps_long_inline_code_without_horizontal_overflow() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/long-inline-code.md");
        let inline_code = format!("`{}`", "\u{3042}".repeat(240));
        let mut app = app_with_preview_doc(&path, &inline_code);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let text_shape = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains(&"\u{3042}".repeat(60)) =>
                {
                    Some(text)
                }
                _ => None,
            })
            .expect("long inline code text shape");

        assert!(
            text_shape.galley.rows.len() > 1,
            "long inline code must wrap instead of staying on a single row"
        );
        assert!(
            text_shape.visual_bounding_rect().right()
                <= preview_rect.right() - PREVIEW_CONTENT_PADDING + 4.0,
            "wrapped inline code must stay within the preview panel"
        );
    }

    #[test]
    fn split_preview_wraps_long_markdown_with_mixed_inline_styles() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/blockquote-strong.md");
        let markdown = concat!(
            "> **Note:** On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple. ",
            "Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n"
        );
        let mut app = app_with_preview_doc(&path, markdown);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let text_shapes: Vec<&egui::epaint::TextShape> = shapes
            .iter()
            .filter_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains("Note:")
                        || text.galley.job.text.contains("Gatekeeper requires") =>
                {
                    Some(text)
                }
                _ => None,
            })
            .collect();

        assert!(
            !text_shapes.is_empty(),
            "expected mixed-style blockquote text shapes"
        );

        let max_right = text_shapes
            .iter()
            .map(|text| text.visual_bounding_rect().right())
            .fold(f32::NEG_INFINITY, f32::max);
        let max_rows = text_shapes
            .iter()
            .map(|text| text.galley.rows.len())
            .max()
            .unwrap_or(0);

        assert!(
            max_rows > 1,
            "mixed-style blockquote must wrap to multiple rows"
        );
        assert!(
            max_right <= preview_rect.right() - PREVIEW_CONTENT_PADDING + 4.0,
            "mixed-style blockquote must stay within preview width, got right edge {max_right}"
        );
    }

    // ── TDD(RED): Vertical split must leave sufficient height for editor scrolling ──

    /// When the split direction is vertical (top/bottom), the editor's
    /// CentralPanel must occupy at least 30% of the total height so that
    /// the TextEdit inside can scroll.
    ///
    /// The bug: `render_preview_content` calls `allocate_rect(outer_rect)` which
    /// consumes the full available height of the TopBottomPanel. Combined with
    /// no `max_height` constraint, the preview panel grows beyond its `default_height`,
    /// starving the CentralPanel.
    #[test]
    fn vertical_split_editor_has_sufficient_height_for_scrolling() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_scroll.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);
        let total_height = 800.0_f32;

        // Run 3 frames for layout stabilization
        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, total_height)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_v_bottom",
            ),
        )
        .expect("preview panel rect")
        .rect;

        // The preview panel should not consume more than 70% of the total height.
        // The remaining >= 30% is the editor's CentralPanel.
        let editor_height = total_height - preview_rect.height();
        let min_editor_ratio = 0.30;

        assert!(
            editor_height >= total_height * min_editor_ratio,
            "Editor panel in vertical split must have at least {:.0}% of total height for scrolling. \
             Got editor_height={editor_height:.1}, preview_height={:.1}, total={total_height:.1}",
            min_editor_ratio * 100.0,
            preview_rect.height(),
        );
    }

    // ── TDD(RED): Bidirectional scroll sync in vertical split ──
    //
    // Scenario 3: Scroll sync works bidirectionally in vertical split.
    // Scenario 5: Scroll sync works bidirectionally after order swap.

    /// When the editor reports a scroll (scroll_source=Editor, fraction=0.5),
    /// the preview must consume it within the next frame, transitioning
    /// scroll_source to Neither. This verifies editor→preview sync works.
    #[test]
    fn vertical_split_editor_to_preview_scroll_sync() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_sync_e2p.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        // Stabilize layout (5 frames)
        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        // Simulate editor scroll by setting scroll state
        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Editor;

        // Run 3 frames for sync to propagate
        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        // After sync, scroll_source must settle to Neither.
        // If it bounces to Preview, the sync is creating an oscillation loop.
        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither after consumption. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    /// Same editor→preview sync test for horizontal split — expected to PASS.
    #[test]
    fn horizontal_split_editor_to_preview_scroll_sync() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/hsplit_sync_e2p.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_horizontal_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Editor;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_horizontal_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither in horizontal split. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    /// Scenario 5: After swapping order (PreviewFirst), the same
    /// editor→preview sync must work in vertical split.
    #[test]
    fn vertical_split_editor_to_preview_scroll_sync_after_swap() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_sync_swap.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        // Use PreviewFirst (swapped order)
        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::PreviewFirst,
                );
            });
        }

        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Editor;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::PreviewFirst,
                );
            });
        }

        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither after order swap. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    /// Verify preview→editor sync direction also works in vertical split.
    /// Set scroll_source=Preview and verify it transitions to Neither.
    #[test]
    fn vertical_split_preview_to_editor_scroll_sync() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_sync_p2e.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        // Simulate preview scroll
        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Preview;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Preview→Editor sync must settle to Neither in vertical split. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    #[test]
    fn refresh_diagrams_update_clears_image_caches() {
        let ctx = test_context();
        let mut frame = eframe::Frame::_new_kittest();
        let path = PathBuf::from("/tmp/refresh-cache.md");
        let mut app = app_with_preview_doc(&path, "# Refresh cache");
        let forget_all_calls = Arc::new(AtomicUsize::new(0));

        ctx.add_bytes_loader(Arc::new(CountingBytesLoader {
            forget_all_calls: Arc::clone(&forget_all_calls),
        }));
        app.pending_action = AppAction::RefreshDiagrams;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            app.update(ctx, &mut frame);
        });

        assert_eq!(
            forget_all_calls.load(Ordering::SeqCst),
            1,
            "RefreshDiagrams must clear image caches before rerendering preview"
        );
    }
}

/// Renders the Meta Info window popup.
fn render_meta_info_window(ctx: &egui::Context, open: &mut bool, path: &std::path::Path) {
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

/// Renders the custom About window with all required OSS project information.
fn render_about_window(
    ctx: &egui::Context,
    open: &mut bool,
    icon: Option<&egui::TextureHandle>,
    action: &mut AppAction,
) {
    const ABOUT_WINDOW_WIDTH: f32 = 400.0;
    const INNER_PADDING: f32 = 8.0;
    const ICON_SIZE: f32 = 64.0;
    const HEADING_SIZE: f32 = 20.0;
    const DESCRIPTION_SIZE: f32 = 12.0;
    const SECTION_HEADER_SIZE: f32 = 13.0;
    const SECTION_SPACING: f32 = 8.0;
    const HEADING_SPACING: f32 = 8.0;
    const SECTION_HEADER_BOTTOM: f32 = 2.0;

    let info = crate::about_info::about_info();

    egui::Window::new(format!("About {}", crate::about_info::APP_DISPLAY_NAME))
        .open(open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(ABOUT_WINDOW_WIDTH)
        .frame(egui::Frame::window(&ctx.style()).inner_margin(INNER_PADDING))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(HEADING_SPACING);
                // App icon
                if let Some(tex) = icon {
                    ui.image(egui::load::SizedTexture::new(
                        tex.id(),
                        egui::vec2(ICON_SIZE, ICON_SIZE),
                    ));
                    ui.add_space(SECTION_SPACING);
                }
                ui.heading(
                    egui::RichText::new(info.product_name)
                        .strong()
                        .size(HEADING_SIZE),
                );
                ui.label(
                    egui::RichText::new(info.description)
                        .weak()
                        .size(DESCRIPTION_SIZE),
                );
                ui.add_space(HEADING_SPACING);
            });

            let i18n_about = &crate::i18n::get().about;

            // ── 1. Basic Info ──
            about_section_header(
                ui,
                &i18n_about.basic_info,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_row(ui, &i18n_about.version, &format!("v{}", info.version));
            about_row(ui, &i18n_about.build, info.build);
            about_row(ui, &i18n_about.copyright, info.copyright);
            ui.add_space(SECTION_SPACING);

            // ── 2. Runtime ──
            about_section_header(
                ui,
                &i18n_about.runtime,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_row(ui, &i18n_about.platform, &info.system.os);
            about_row(ui, &i18n_about.architecture, &info.system.arch);
            about_row(ui, &i18n_about.rust, &info.system.rustc_version);
            ui.add_space(SECTION_SPACING);

            // ── 3. License ──
            about_section_header(
                ui,
                &i18n_about.license,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_row(ui, &i18n_about.license, info.license);
            ui.add_space(SECTION_SPACING);

            // ── 4-6. Links ──
            about_section_header(
                ui,
                &i18n_about.links,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_link_row(
                ui,
                &i18n_about.source_code,
                info.repository,
                crate::Icon::Github,
            );
            about_link_row(
                ui,
                &i18n_about.documentation,
                info.docs_url,
                crate::Icon::Document,
            );
            about_link_row(
                ui,
                &i18n_about.report_issue,
                info.issues_url,
                crate::Icon::Bug,
            );
            ui.add_space(SECTION_SPACING);

            // ── 7. Support / Sponsor ──
            about_section_header(
                ui,
                &i18n_about.support,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            if info.sponsor_url.is_empty() {
                ui.horizontal(|ui| {
                    ui.add(crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Medium));
                    ui.label(
                        egui::RichText::new(crate::i18n::get().menu.release_notes.clone()).weak(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::image(
                                    crate::Icon::ExternalLink
                                        .ui_image(ui, crate::icon::IconSize::Small),
                                )
                                .frame(false),
                            )
                            .on_hover_text(crate::i18n::get().menu.release_notes.clone())
                            .clicked()
                        {
                            *action = AppAction::ShowReleaseNotes;
                        }
                    });
                });
            } else {
                about_link_row(
                    ui,
                    &i18n_about.sponsor,
                    info.sponsor_url,
                    crate::Icon::Heart,
                );
            }
            ui.add_space(SECTION_SPACING);
        });
}

/// Section header for the About dialog.
fn about_section_header(ui: &mut egui::Ui, title: &str, size: f32, bottom: f32) {
    ui.separator();
    ui.label(egui::RichText::new(title).strong().size(size));
    ui.add_space(bottom);
}

/// Key-value row (non-clickable).
fn about_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
    });
}

fn about_link_row(ui: &mut egui::Ui, label: &str, url: &str, icon: crate::Icon) {
    ui.horizontal(|ui| {
        ui.add(icon.ui_image(ui, crate::icon::IconSize::Medium));
        ui.label(egui::RichText::new(label).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::image(
                        crate::Icon::ExternalLink.ui_image(ui, crate::icon::IconSize::Small),
                    )
                    .frame(false),
                )
                .on_hover_text(url)
                .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
        });
    });
}

pub(crate) const SEARCH_MODAL_WIDTH: f32 = 500.0;
pub(crate) const SEARCH_MODAL_HEIGHT: f32 = 400.0;

pub(crate) const TOC_PANEL_DEFAULT_WIDTH: f32 = 200.0;
pub(crate) const TOC_PANEL_MARGIN: f32 = 8.0;
pub(crate) const TOC_HEADING_VISIBILITY_THRESHOLD: f32 = 40.0;
pub(crate) const TOC_INDENT_PER_LEVEL: f32 = 12.0;

pub(crate) const LIGHT_MODE_ICON_BG: u8 = 235;
pub(crate) const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 200;

fn render_update_window(
    ctx: &egui::Context,
    open: &mut bool,
    state: &AppState,
    markdown_cache: &mut egui_commonmark::CommonMarkCache,
    pending_action: &mut AppAction,
) {
    use crate::app_state::UpdatePhase;
    use crate::widgets::Modal;

    const SPACING_SMALL: f32 = 4.0;
    const SPACING_MEDIUM: f32 = 8.0;
    const SPACING_LARGE: f32 = 12.0;
    const MAX_SCROLL_HEIGHT: f32 = 250.0;
    const UPDATE_DIALOG_WIDTH: f32 = 600.0;

    let msgs = &crate::i18n::get().update;

    // Phase-aware modals (Downloading / Installing / ReadyToRelaunch)
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
        None => {} // Fall through to the standard update dialog
    }

    // Standard update dialog — use Modal to avoid vertical stretch bug.
    // (egui::Window::open() stores resize state, causing unbounded height growth.)
    if state.update.checking {
        // Checking spinner — no footer, no close button
        Modal::new("katana_update_dialog_v6", &msgs.title)
            .width(UPDATE_DIALOG_WIDTH)
            .show_body_only(ctx, |ui| {
                ui.add(egui::Spinner::new());
                ui.add_space(SPACING_MEDIUM);
                ui.label(msgs.checking_for_updates.clone());
            });
    } else if let Some(err) = &state.update.check_error {
        // Error state — OK button to close
        let close = {
            let err = err.clone();
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
        // Update available — Install/Skip/Later buttons
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
                        // Primary: Install
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
                        // Release Notes
                        if ui
                            .button(crate::i18n::get().menu.release_notes.clone())
                            .clicked()
                        {
                            return Some(AppAction::ShowReleaseNotes);
                        }
                        // Skip
                        if ui.button(msgs.action_skip_version.clone()).clicked() {
                            return Some(AppAction::SkipVersion(tag.clone()));
                        }
                        // Later
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
                AppAction::DismissUpdate | AppAction::SkipVersion(_) | AppAction::ShowReleaseNotes
            ) {
                *open = false;
            }
        }
    } else {
        // Up to date — OK button to close
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

fn render_create_fs_node_modal(
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

fn render_rename_modal(
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

fn render_delete_modal(
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
