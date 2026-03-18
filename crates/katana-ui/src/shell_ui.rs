//! Pure egui UI rendering functions for the KatanA shell.
//!
//! This module contains code that depends entirely on the egui frame context
//! and UI events (e.g., clicks).
//! - Rendering functions that can only be called within `eframe::App::update`
//! - Branches that are not executed without user click events
//! - OS UI dependent code like `rfd` file dialogs
//!
//! Therefore, it is excluded from code coverage measurement using `--ignore-filename-regex`.

use eframe::egui;

use crate::{
    app_state::{AppAction, AppState, ScrollSource, ViewMode},
    i18n,
    preview_pane::{DownloadRequest, PreviewPane},
};

use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, EDITOR_INITIAL_VISIBLE_ROWS, FILE_TREE_PANEL_DEFAULT_WIDTH,
    FILE_TREE_PANEL_MIN_WIDTH, NO_WORKSPACE_BOTTOM_SPACING, SCROLL_SYNC_DEAD_ZONE,
    TAB_INTER_ITEM_SPACING, TAB_NAV_BUTTONS_AREA_WIDTH, TAB_TOOLTIP_SHOW_DELAY_SECS,
};
use crate::theme_bridge;

pub(crate) fn open_folder_dialog() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_menu_bar(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(crate::i18n::t("menu_file"), |ui| {
                render_file_menu(ui, state, action);
            });
            ui.menu_button(crate::i18n::t("menu_settings"), |ui| {
                render_settings_menu(ui, state, action);
            });
            render_header_right(ui, state);
        });
    });
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_file_menu(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    if ui.button(crate::i18n::t("menu_open_workspace")).clicked() {
        if let Some(path) = open_folder_dialog() {
            *action = AppAction::OpenWorkspace(path);
        }
        ui.close();
    }
    ui.separator();
    if ui
        .add_enabled(
            state.is_dirty(),
            egui::Button::new(crate::i18n::t("menu_save")),
        )
        .clicked()
    {
        *action = AppAction::SaveDocument;
        ui.close();
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_settings_menu(ui: &mut egui::Ui, _state: &AppState, action: &mut AppAction) {
    ui.menu_button(crate::i18n::t("menu_language"), |ui| {
        let mut reset_layout = false;
        for (code, display_name) in crate::i18n::supported_languages() {
            if ui.button(display_name.as_str()).clicked() {
                *action = AppAction::ChangeLanguage(code.to_string());
                reset_layout = true;
            }
        }
        if reset_layout {
            ui.close();
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_header_right(ui: &mut egui::Ui, state: &AppState) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if state.is_dirty() {
            ui.label("*");
        }
    });
}

pub(crate) fn render_status_bar(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let ready = crate::i18n::t("status_ready");
            let msg = state.status_message.as_deref().unwrap_or(&ready);
            ui.label(msg);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if state.is_dirty() {
                    ui.label("●");
                }
            });
        });
    });
}

pub(crate) fn render_workspace_panel(
    ctx: &egui::Context,
    state: &mut AppState,
    action: &mut AppAction,
) {
    egui::SidePanel::left("workspace_tree")
        .resizable(true)
        .min_width(FILE_TREE_PANEL_MIN_WIDTH)
        .default_width(FILE_TREE_PANEL_DEFAULT_WIDTH)
        .show(ctx, |ui| {
            let panel_width = ui.available_width();
            ui.set_max_width(panel_width);
            ui.set_min_width(panel_width);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.horizontal(|ui| {
                ui.heading(crate::i18n::t("workspace_title"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("‹")
                        .on_hover_text(crate::i18n::t("collapse_sidebar"))
                        .clicked()
                    {
                        state.show_workspace = false;
                    }
                });
            });
            if state.workspace.is_some() {
                ui.horizontal(|ui| {
                    if ui
                        .small_button("+")
                        .on_hover_text(crate::i18n::t("expand_all"))
                        .clicked()
                    {
                        state.force_tree_open = Some(true);
                    }
                    if ui
                        .small_button("-")
                        .on_hover_text(crate::i18n::t("collapse_all"))
                        .clicked()
                    {
                        state.force_tree_open = Some(false);
                    }
                });
            }
            ui.separator();
            render_workspace_content(ui, state, action);
        });
}

pub(crate) fn render_workspace_content(
    ui: &mut egui::Ui,
    state: &mut AppState,
    action: &mut AppAction,
) {
    if let Some(ws) = &state.workspace {
        let entries = ws.tree.clone();
        let mut selected: Option<std::path::PathBuf> = None;
        let force = state.force_tree_open;
        let active_path = state.active_path().map(|p| p.to_path_buf());
        egui::ScrollArea::vertical()
            .id_salt("workspace_tree_scroll")
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                for entry in &entries {
                    render_tree_entry(ui, entry, &mut selected, force, 0, active_path.as_deref());
                }
            });
        state.force_tree_open = None;
        if let Some(path) = selected {
            *action = AppAction::SelectDocument(path);
        }
    } else {
        ui.label(crate::i18n::t("no_workspace_open"));
        ui.add_space(NO_WORKSPACE_BOTTOM_SPACING);
        if ui.button(crate::i18n::t("menu_open_workspace")).clicked() {
            if let Some(path) = open_folder_dialog() {
                *action = AppAction::OpenWorkspace(path);
            }
        }
    }
}

pub(crate) fn render_preview_content(
    ui: &mut egui::Ui,
    preview: &mut PreviewPane,
    state: &AppState,
    action: &mut AppAction,
    scroll_sync: bool,
    scroll_state: &mut (f32, ScrollSource, f32),
) -> Option<DownloadRequest> {
    let mut download_req = None;
    render_preview_header(ui, state, action);
    ui.separator();

    let (fraction, source, prev_max_scroll) = scroll_state;
    let mut scroll_area = egui::ScrollArea::both().id_salt("preview_scroll");

    let consuming_editor = scroll_sync && *source == ScrollSource::Editor;
    if consuming_editor {
        scroll_area = scroll_area.vertical_scroll_offset(*fraction * (*prev_max_scroll).max(1.0));
    }

    let output = scroll_area.show(ui, |ui| {
        download_req = preview.show_content(ui);
    });

    if scroll_sync {
        let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
        *prev_max_scroll = max_scroll;

        if consuming_editor {
            *source = ScrollSource::Neither;
            if max_scroll > 0.0 {
                *fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
            }
        } else {
            if max_scroll > 0.0 {
                let current_fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
                let diff = (current_fraction - *fraction).abs();
                if diff > SCROLL_SYNC_DEAD_ZONE {
                    *fraction = current_fraction;
                    *source = ScrollSource::Preview;
                }
            }
        }
    }

    download_req
}

pub(crate) fn render_preview_header(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    ui.horizontal(|ui| {
        ui.heading(crate::i18n::t("preview_title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let has_doc = state.active_document().is_some();
            if ui
                .add_enabled(has_doc, egui::Button::new("\u{1F504}"))
                .on_hover_text(crate::i18n::t("refresh_diagrams"))
                .clicked()
            {
                *action = AppAction::RefreshDiagrams;
            }
        });
    });
}

/// Tab bar: Displays tabs of open documents side-by-side.
pub(crate) fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState, action: &mut AppAction) {
    const MAX_TAB_WIDTH: f32 = 200.0;

    let mut close_idx: Option<usize> = None;
    let mut tab_action: Option<AppAction> = None;

    let ws_root = state.workspace.as_ref().map(|ws| ws.root.clone());
    let doc_count = state.open_documents.len();

    ui.style_mut().interaction.tooltip_delay = TAB_TOOLTIP_SHOW_DELAY_SECS;

    ui.horizontal(|ui| {
        let nav_button_width = TAB_NAV_BUTTONS_AREA_WIDTH;
        let scroll_width = ui.available_width() - nav_button_width;

        egui::ScrollArea::horizontal()
            .max_width(scroll_width)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .id_salt("tab_scroll")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (idx, doc) in state.open_documents.iter().enumerate() {
                        let is_active = state.active_doc_idx == Some(idx);
                        let filename = doc.file_name().unwrap_or("untitled").to_string();
                        let dirty_suffix = if doc.is_dirty { " *" } else { "" };
                        let title = format!("{filename}{dirty_suffix}");
                        let tooltip_path = relative_full_path(&doc.path, ws_root.as_deref());

                        let resp = ui
                            .push_id(format!("tab_{idx}"), |ui| {
                                ui.set_max_width(MAX_TAB_WIDTH);
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                ui.selectable_label(is_active, &title)
                            })
                            .inner;

                        let clicked = resp.clicked();
                        resp.on_hover_text(&tooltip_path);
                        if clicked && !is_active {
                            tab_action = Some(AppAction::SelectDocument(doc.path.clone()));
                        }

                        if ui.small_button("x").clicked() {
                            close_idx = Some(idx);
                        }
                        ui.add_space(TAB_INTER_ITEM_SPACING);
                    }
                });
            });

        ui.separator();

        let nav_enabled = doc_count > 1;
        if ui
            .add_enabled(nav_enabled, egui::Button::new("◀").small())
            .clicked()
        {
            if let Some(idx) = state.active_doc_idx {
                let new_idx = crate::shell_logic::prev_tab_index(idx, doc_count);
                tab_action = Some(AppAction::SelectDocument(
                    state.open_documents[new_idx].path.clone(),
                ));
            }
        }
        if ui
            .add_enabled(nav_enabled, egui::Button::new("▶").small())
            .clicked()
        {
            if let Some(idx) = state.active_doc_idx {
                let new_idx = crate::shell_logic::next_tab_index(idx, doc_count);
                tab_action = Some(AppAction::SelectDocument(
                    state.open_documents[new_idx].path.clone(),
                ));
            }
        }
    });

    if let Some(action_val) = tab_action {
        *action = action_val;
    } else if let Some(idx) = close_idx {
        *action = AppAction::CloseDocument(idx);
    }
}

pub(crate) fn relative_full_path(
    path: &std::path::Path,
    ws_root: Option<&std::path::Path>,
) -> String {
    crate::shell_logic::relative_full_path(path, ws_root)
}

pub(crate) fn render_view_mode_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let mut mode = state.active_view_mode();
    let prev = mode;
    let bar_height = ui.spacing().interact_size.y;
    let available_width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(available_width, bar_height),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            ui.selectable_value(&mut mode, ViewMode::Split, i18n::t("view_mode_split"));
            ui.selectable_value(&mut mode, ViewMode::CodeOnly, i18n::t("view_mode_code"));
            ui.selectable_value(
                &mut mode,
                ViewMode::PreviewOnly,
                i18n::t("view_mode_preview"),
            );
        },
    );
    if mode != prev {
        state.set_active_view_mode(mode);
    }
}

pub(crate) fn render_editor_content(
    ui: &mut egui::Ui,
    state: &mut AppState,
    action: &mut AppAction,
    sync_scroll: bool,
) {
    if let Some(doc) = state.active_document() {
        let mut buffer = doc.buffer.clone();

        let mut scroll_area = egui::ScrollArea::vertical().id_salt("editor_scroll");

        let consuming_preview = sync_scroll && state.scroll_source == ScrollSource::Preview;
        if consuming_preview {
            scroll_area = scroll_area
                .vertical_scroll_offset(state.scroll_fraction * state.editor_max_scroll.max(1.0));
        }

        let output = scroll_area.show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut buffer)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(EDITOR_INITIAL_VISIBLE_ROWS),
            );
            if response.changed() {
                *action = AppAction::UpdateBuffer(buffer);
            }
        });

        if sync_scroll {
            let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
            state.editor_max_scroll = max_scroll;

            if consuming_preview {
                state.scroll_source = ScrollSource::Neither;
                if max_scroll > 0.0 {
                    state.scroll_fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
                }
            } else {
                if max_scroll > 0.0 {
                    let current_fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
                    let diff = (current_fraction - state.scroll_fraction).abs();
                    if diff > SCROLL_SYNC_DEAD_ZONE {
                        state.scroll_fraction = current_fraction;
                        state.scroll_source = ScrollSource::Editor;
                    }
                }
            }
        }
    }
}

pub(crate) fn render_tree_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    selected: &mut Option<std::path::PathBuf>,
    force: Option<bool>,
    depth: usize,
    active_path: Option<&std::path::Path>,
) {
    use katana_core::workspace::TreeEntry;
    match entry {
        TreeEntry::Directory { path, children } => {
            render_directory_entry(ui, path, children, selected, force, depth, active_path);
        }
        TreeEntry::File { path } => {
            render_file_entry(ui, entry, path, selected, depth, active_path);
        }
    }
}

pub(crate) fn indent_prefix(depth: usize) -> String {
    "  ".repeat(depth)
}

pub(crate) fn render_directory_entry(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    children: &[katana_core::workspace::TreeEntry],
    selected: &mut Option<std::path::PathBuf>,
    force: Option<bool>,
    depth: usize,
    active_path: Option<&std::path::Path>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let id = ui.make_persistent_id(format!("dir:{}", path.display()));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    if let Some(open) = force {
        state.set_open(open);
    }
    let is_open = state.is_open();

    let arrow = if is_open { "▼" } else { "▶" };
    let dir_icon = if is_open { "📂" } else { "📁" };
    let prefix = indent_prefix(depth);
    let label_text = format!("{prefix}{arrow} {dir_icon} {name}");
    let file_tree_color = ui.visuals().text_color();
    let resp = ui.add(
        egui::Label::new(egui::RichText::new(label_text).color(file_tree_color))
            .truncate()
            .sense(egui::Sense::click()),
    );
    if resp.clicked() {
        state.set_open(!is_open);
    }
    state.store(ui.ctx());

    if state.is_open() {
        for child in children {
            render_tree_entry(ui, child, selected, force, depth + 1, active_path);
        }
    }
}

pub(crate) fn render_file_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    path: &std::path::Path,
    selected: &mut Option<std::path::PathBuf>,
    depth: usize,
    active_path: Option<&std::path::Path>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let prefix = indent_prefix(depth);
    let icon = if entry.is_markdown() { "📄" } else { "  " };
    let label = format!("{prefix}{icon} {name}");

    let is_active = active_path.is_some_and(|ap| ap == path);

    let text_color = if is_active {
        egui::Color32::WHITE
    } else {
        ui.visuals().text_color()
    };
    let rich = egui::RichText::new(&label).color(text_color);
    let rich = if is_active { rich.strong() } else { rich };

    let resp = ui.add(
        egui::Label::new(rich)
            .truncate()
            .sense(egui::Sense::click()),
    );

    if is_active {
        let full_rect = egui::Rect::from_min_max(
            egui::pos2(ui.min_rect().min.x, resp.rect.min.y),
            egui::pos2(ui.min_rect().max.x, resp.rect.max.y),
        );
        let highlight_color = ui.visuals().selection.bg_fill;
        ui.painter()
            .rect_filled(full_rect, ACTIVE_FILE_HIGHLIGHT_ROUNDING, highlight_color);
    }
    if resp.clicked() && entry.is_markdown() {
        *selected = Some(path.to_path_buf());
    }
}

// ─────────────────────────────────────────────
// macOS Native Menu FFI
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

    #[allow(dead_code)]
    extern "C" {
        pub fn katana_setup_native_menu();
        pub fn katana_poll_menu_action() -> i32;
        pub fn katana_set_app_icon_png(png_data: *const u8, png_len: std::ffi::c_ulong);
        pub fn katana_set_process_name();
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

// ─────────────────────────────────────────────
// eframe::App Implementation (egui Main Rendering Loop)
// ─────────────────────────────────────────────

use crate::shell::{
    KatanaApp, SIDEBAR_COLLAPSED_TOGGLE_WIDTH, SPLIT_PREVIEW_PANEL_DEFAULT_WIDTH,
    SPLIT_PREVIEW_PANEL_MIN_WIDTH,
};

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme colours to egui Visuals (only when the palette changed)
        let theme_colors = self.state.settings.settings().effective_theme_colors();
        if self.cached_theme.as_ref() != Some(&theme_colors) {
            let dark = theme_colors.mode == katana_platform::theme::ThemeMode::Dark;
            ctx.set_visuals(theme_bridge::visuals_from_theme(&theme_colors));
            katana_core::markdown::color_preset::DiagramColorPreset::set_dark_mode(dark);
            self.cached_theme = Some(theme_colors.clone());
            // Re-render diagrams with the new theme colours
            self.pending_action = AppAction::RefreshDiagrams;
        }

        // Apply font size to egui text styles (only when the size changed)
        let font_size = self.state.settings.settings().clamped_font_size();
        if self.cached_font_size != Some(font_size) {
            theme_bridge::apply_font_size(ctx, font_size);
            self.cached_font_size = Some(font_size);
        }

        // Apply font family by rebuilding FontDefinitions (only when family changed)
        let font_family = self.state.settings.settings().font_family.clone();
        if self.cached_font_family.as_deref() != Some(&font_family) {
            theme_bridge::apply_font_family(ctx, &font_family);
            self.cached_font_family = Some(font_family);
        }

        self.poll_download(ctx);

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
                native_menu::TAG_ABOUT => {
                    self.show_about = !self.show_about;
                }
                native_menu::TAG_SETTINGS => {
                    self.pending_action = AppAction::ToggleSettings;
                }
                _ => {}
            }
        }

        let action = self.take_action();
        self.process_action(action);

        // On macOS, the egui menu is hidden because the native menu bar is used.
        #[cfg(not(target_os = "macos"))]
        render_menu_bar(ctx, &mut self.state, &mut self.pending_action);
        render_status_bar(ctx, &self.state);

        // Reflect the file name in the window title
        let ws_root_for_title = self.state.workspace.as_ref().map(|ws| ws.root.clone());
        let title_text = match self.state.active_document() {
            Some(doc) => {
                let rel = relative_full_path(&doc.path, ws_root_for_title.as_deref());
                format!("KatanA — {rel}")
            }
            None => "KatanA".to_string(),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));

        // In-app title bar
        egui::TopBottomPanel::top("app_title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.centered_and_justified(|ui| {
                    let title_color = theme_bridge::rgb_to_color32(theme_colors.title_bar_text);
                    ui.label(egui::RichText::new(&title_text).small().color(title_color));
                });
            });
        });

        // Show the collapse toggle button even when the workspace is hidden.
        if !self.state.show_workspace {
            egui::SidePanel::left("workspace_collapsed")
                .resizable(false)
                .exact_width(SIDEBAR_COLLAPSED_TOGGLE_WIDTH)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if ui
                            .button("›")
                            .on_hover_text(crate::i18n::t("workspace_title"))
                            .clicked()
                        {
                            self.state.show_workspace = true;
                        }
                    });
                });
        } else {
            render_workspace_panel(ctx, &mut self.state, &mut self.pending_action);
        }

        // Tab row + breadcrumbs + view mode row
        egui::TopBottomPanel::top("tab_toolbar").show(ctx, |ui| {
            render_tab_bar(ui, &mut self.state, &mut self.pending_action);
            if let Some(doc) = self.state.active_document() {
                let ws_root = self.state.workspace.as_ref().map(|ws| ws.root.clone());
                let rel = relative_full_path(&doc.path, ws_root.as_deref());
                ui.horizontal(|ui| {
                    let segments: Vec<&str> = rel.split('/').collect();
                    for (i, seg) in segments.iter().enumerate() {
                        if i > 0 {
                            ui.label(egui::RichText::new("›").small());
                        }
                        ui.label(egui::RichText::new(*seg).small());
                    }
                });
                render_view_mode_bar(ui, &mut self.state);
            }
        });

        let mut download_req: Option<DownloadRequest> = None;
        let current_mode = self.state.active_view_mode();
        let is_split = current_mode == ViewMode::Split;

        // Split mode
        if is_split {
            let active_path = self.state.active_document().map(|d| d.path.clone());
            let mut scroll_state = (
                self.state.scroll_fraction,
                self.state.scroll_source,
                self.state.preview_max_scroll,
            );
            let preview_bg = theme_bridge::rgb_to_color32(
                self.state
                    .settings
                    .settings()
                    .effective_theme_colors()
                    .preview_background,
            );
            egui::SidePanel::right("preview_panel")
                .resizable(true)
                .min_width(SPLIT_PREVIEW_PANEL_MIN_WIDTH)
                .default_width(SPLIT_PREVIEW_PANEL_DEFAULT_WIDTH)
                .frame(egui::Frame::side_top_panel(ctx.style().as_ref()).fill(preview_bg))
                .show(ctx, |ui| {
                    if let Some(path) = &active_path {
                        let pane = self.tab_panes.entry(path.clone()).or_default();
                        download_req = render_preview_content(
                            ui,
                            pane,
                            &self.state,
                            &mut self.pending_action,
                            true,
                            &mut scroll_state,
                        );
                    }
                });
            self.state.scroll_fraction = scroll_state.0;
            self.state.scroll_source = scroll_state.1;
            self.state.preview_max_scroll = scroll_state.2;
        }

        egui::CentralPanel::default().show(ctx, |ui| match current_mode {
            ViewMode::CodeOnly | ViewMode::Split => {
                render_editor_content(ui, &mut self.state, &mut self.pending_action, is_split);
            }
            ViewMode::PreviewOnly => {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    0.0,
                    theme_bridge::rgb_to_color32(
                        self.state
                            .settings
                            .settings()
                            .effective_theme_colors()
                            .preview_background,
                    ),
                );
                let active_path = self.state.active_document().map(|d| d.path.clone());
                let mut scroll_state = (0.0_f32, ScrollSource::Neither, 0.0_f32);
                if let Some(path) = active_path {
                    let pane = self.tab_panes.entry(path).or_default();
                    let req = render_preview_content(
                        ui,
                        pane,
                        &self.state,
                        &mut self.pending_action,
                        false,
                        &mut scroll_state,
                    );
                    if req.is_some() {
                        download_req = req;
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(i18n::t("no_document_selected"));
                    });
                }
            }
        });

        if let Some(req) = download_req {
            self.start_download(req);
        }

        // Settings window
        crate::settings_window::render_settings_window(
            ctx,
            &mut self.state.show_settings,
            &mut self.state.active_settings_tab,
            &mut self.state.settings,
            &mut self.settings_preview,
        );

        // About dialog
        if self.show_about {
            render_about_window(ctx, &mut self.show_about, self.about_icon.as_ref());
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
                    self.process_action(AppAction::SelectDocument(path));
                }
            } else {
                unprocessed_commands.push(cmd);
            }
        }

        // Put back the commands we didn't handle
        if !unprocessed_commands.is_empty() {
            ctx.output_mut(|o| o.commands.extend(unprocessed_commands));
        }
    }
}

/// Renders the custom About window with all required OSS project information.
fn render_about_window(ctx: &egui::Context, open: &mut bool, icon: Option<&egui::TextureHandle>) {
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
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
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

            // ── 1. Basic Info ──
            about_section_header(ui, "Basic Info", SECTION_HEADER_SIZE, SECTION_HEADER_BOTTOM);
            about_row(ui, "Version", &format!("v{}", info.version));
            about_row(ui, "Build", info.build);
            about_row(ui, "Copyright", info.copyright);
            ui.add_space(SECTION_SPACING);

            // ── 2. Runtime ──
            about_section_header(ui, "Runtime", SECTION_HEADER_SIZE, SECTION_HEADER_BOTTOM);
            about_row(ui, "Platform", &info.system.os);
            about_row(ui, "Architecture", &info.system.arch);
            about_row(ui, "Rust", &info.system.rustc_version);
            ui.add_space(SECTION_SPACING);

            // ── 3. License ──
            about_section_header(ui, "License", SECTION_HEADER_SIZE, SECTION_HEADER_BOTTOM);
            about_row(ui, "License", info.license);
            ui.add_space(SECTION_SPACING);

            // ── 4-6. Links ──
            about_section_header(ui, "Links", SECTION_HEADER_SIZE, SECTION_HEADER_BOTTOM);
            about_link_row(ui, "Source Code", info.repository);
            about_link_row(ui, "Documentation", info.docs_url);
            about_link_row(ui, "Report Issue", info.issues_url);
            ui.add_space(SECTION_SPACING);

            // ── 7. Support / Sponsor ──
            about_section_header(ui, "Support", SECTION_HEADER_SIZE, SECTION_HEADER_BOTTOM);
            if info.sponsor_url.is_empty() {
                about_row(ui, "Sponsor", "Coming Soon");
            } else {
                about_link_row(ui, "Sponsor", info.sponsor_url);
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

/// Link row: label on the left, clickable short text on the right.
fn about_link_row(ui: &mut egui::Ui, label: &str, url: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.hyperlink_to("↗", url);
        });
    });
}
