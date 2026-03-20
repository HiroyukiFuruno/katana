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
    preview_pane::{DownloadRequest, PreviewPane},
};

use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, EDITOR_INITIAL_VISIBLE_ROWS, FILE_TREE_PANEL_DEFAULT_WIDTH,
    FILE_TREE_PANEL_MIN_WIDTH, NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_ITEM_SPACING,
    RECENT_WORKSPACES_SPACING, SCROLL_SYNC_DEAD_ZONE, TAB_INTER_ITEM_SPACING,
    TAB_NAV_BUTTONS_AREA_WIDTH, TAB_TOOLTIP_SHOW_DELAY_SECS,
};
use crate::theme_bridge;
use katana_platform::{PaneOrder, SplitDirection};

pub(crate) fn open_folder_dialog() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_menu_bar(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(crate::i18n::get().menu.file.clone(), |ui| {
                render_file_menu(ui, state, action);
            });
            ui.menu_button(crate::i18n::get().menu.settings.clone(), |ui| {
                render_settings_menu(ui, state, action);
            });
            render_header_right(ui, state);
        });
    });
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_file_menu(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    if ui
        .button(crate::i18n::get().menu.open_workspace.clone())
        .clicked()
    {
        if let Some(path) = open_folder_dialog() {
            *action = AppAction::OpenWorkspace(path);
        }
        ui.close();
    }
    ui.separator();
    if ui
        .add_enabled(
            state.is_dirty(),
            egui::Button::new(crate::i18n::get().menu.save.clone()),
        )
        .clicked()
    {
        *action = AppAction::SaveDocument;
        ui.close();
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_settings_menu(ui: &mut egui::Ui, _state: &AppState, action: &mut AppAction) {
    ui.menu_button(crate::i18n::get().menu.language.clone(), |ui| {
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
            let ready = crate::i18n::get().status.ready.clone();
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

const WORKSPACE_SPINNER_OUTER_MARGIN: f32 = 10.0;
const WORKSPACE_SPINNER_INNER_MARGIN: f32 = 10.0;
const WORKSPACE_SPINNER_TEXT_MARGIN: f32 = 5.0;

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
                ui.heading(crate::i18n::get().workspace.workspace_title.clone());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("‹")
                        .on_hover_text(crate::i18n::get().action.collapse_sidebar.clone())
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
                        .on_hover_text(crate::i18n::get().action.expand_all.clone())
                        .clicked()
                    {
                        if let Some(ws) = &state.workspace {
                            state
                                .expanded_directories
                                .extend(ws.collect_all_directory_paths());
                        }
                    }
                    if ui
                        .small_button("-")
                        .on_hover_text(crate::i18n::get().action.collapse_all.clone())
                        .clicked()
                    {
                        state.force_tree_open = Some(false);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !state.settings.settings().workspace.paths.is_empty() {
                            ui.menu_button("📄", |ui| {
                                for path in state.settings.settings().workspace.paths.iter().rev() {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .button("×")
                                            .on_hover_text(
                                                crate::i18n::get().action.remove_workspace.clone(),
                                            )
                                            .clicked()
                                        {
                                            *action = AppAction::RemoveWorkspace(path.clone());
                                            ui.close();
                                        }
                                        if ui.selectable_label(false, path).clicked() {
                                            *action = AppAction::OpenWorkspace(
                                                std::path::PathBuf::from(path),
                                            );
                                            ui.close();
                                        }
                                    });
                                }
                            })
                            .response
                            .on_hover_text(crate::i18n::get().workspace.recent_workspaces.clone());
                        }
                        if ui
                            .small_button("🔄")
                            .on_hover_text(crate::i18n::get().action.refresh_workspace.clone())
                            .clicked()
                        {
                            *action = AppAction::RefreshWorkspace;
                        }
                        let filter_btn_color = if state.filter_enabled {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        if ui
                            .add(egui::Button::new("\u{25BC}").small().fill(filter_btn_color))
                            .on_hover_text(crate::i18n::get().action.toggle_filter.clone())
                            .clicked()
                        {
                            state.filter_enabled = !state.filter_enabled;
                        }
                        if ui
                            .add(egui::Button::new("🔍").small())
                            .on_hover_text(crate::i18n::get().search.modal_title.clone())
                            .clicked()
                        {
                            state.show_search_modal = true;
                        }
                    });
                });

                if state.filter_enabled {
                    let mut is_valid_regex = true;
                    if !state.filter_query.is_empty() {
                        is_valid_regex = regex::Regex::new(&state.filter_query).is_ok();
                    }
                    ui.horizontal(|ui| {
                        let text_color = if is_valid_regex {
                            ui.visuals().text_color()
                        } else {
                            egui::Color32::RED
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut state.filter_query)
                                .text_color(text_color)
                                .hint_text("Filter (Regex)...")
                                .desired_width(f32::INFINITY),
                        );
                    });
                }
            }
            ui.separator();
            if state.is_loading_workspace {
                ui.add_space(WORKSPACE_SPINNER_OUTER_MARGIN);
                ui.horizontal(|ui| {
                    ui.add_space(WORKSPACE_SPINNER_INNER_MARGIN);
                    ui.spinner();
                    ui.add_space(WORKSPACE_SPINNER_TEXT_MARGIN);
                    ui.label(crate::i18n::get().action.refresh_workspace.clone());
                });
            } else {
                render_workspace_content(ui, state, action);
            }
        });
}

pub(crate) fn render_workspace_content(
    ui: &mut egui::Ui,
    state: &mut AppState,
    action: &mut AppAction,
) {
    if let Some(ws) = &state.workspace {
        let entries = ws.tree.clone();
        if let Some(force) = state.force_tree_open {
            if force {
                state
                    .expanded_directories
                    .extend(ws.collect_all_directory_paths());
            } else {
                state.expanded_directories.clear();
            }
        }
        let active_path = state.active_path().map(|p| p.to_path_buf());

        let ws_root = ws.root.clone();
        if state.filter_enabled && !state.filter_query.is_empty() {
            if let Ok(regex) = regex::Regex::new(&state.filter_query) {
                if state.filter_cache.as_ref().map(|(q, _)| q) != Some(&state.filter_query) {
                    let mut visible = std::collections::HashSet::new();
                    gather_visible_paths(&entries, &regex, &ws_root, &mut visible);
                    state.filter_cache = Some((state.filter_query.clone(), visible));
                }
            } else {
                state.filter_cache = None;
            }
        } else {
            state.filter_cache = None;
        }
        let filter_set = state.filter_cache.as_ref().map(|(_, v)| v);

        egui::ScrollArea::vertical()
            .id_salt("workspace_tree_scroll")
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                let mut ctx = TreeRenderContext {
                    action,
                    depth: 0,
                    active_path: active_path.as_deref(),
                    filter_set,
                    expanded_directories: &mut state.expanded_directories,
                };
                for entry in &entries {
                    render_tree_entry(ui, entry, &mut ctx);
                }
            });
        state.force_tree_open = None;
    } else {
        ui.label(crate::i18n::get().workspace.no_workspace_open.clone());
        ui.add_space(NO_WORKSPACE_BOTTOM_SPACING);
        if ui
            .button(crate::i18n::get().menu.open_workspace.clone())
            .clicked()
        {
            if let Some(path) = open_folder_dialog() {
                *action = AppAction::OpenWorkspace(path);
            }
        }

        let recent_paths = &state.settings.settings().workspace.paths;
        if !recent_paths.is_empty() {
            ui.add_space(RECENT_WORKSPACES_SPACING);
            ui.separator();
            ui.add_space(RECENT_WORKSPACES_SPACING);
            ui.heading(crate::i18n::get().workspace.recent_workspaces.clone());
            ui.add_space(RECENT_WORKSPACES_ITEM_SPACING);
            for path in recent_paths.iter().rev() {
                ui.horizontal(|ui| {
                    if ui
                        .button("×")
                        .on_hover_text(crate::i18n::get().action.remove_workspace.clone())
                        .clicked()
                    {
                        *action = AppAction::RemoveWorkspace(path.clone());
                    }
                    if ui.selectable_label(false, path).clicked() {
                        *action = AppAction::OpenWorkspace(std::path::PathBuf::from(path));
                    }
                });
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
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    let padding = f32::from(PREVIEW_CONTENT_PADDING);
    let outer_rect = ui.available_rect_before_wrap();
    let content_rect = outer_rect.shrink2(egui::vec2(padding, padding));
    ui.allocate_rect(outer_rect, egui::Sense::hover());

    let (fraction, source, prev_max_scroll) = scroll_state;
    let mut scroll_area = egui::ScrollArea::vertical()
        .id_salt("preview_scroll")
        .auto_shrink(std::array::from_fn(|_| false));

    let consuming_editor = scroll_sync && *source == ScrollSource::Editor;
    if consuming_editor {
        scroll_area = scroll_area.vertical_scroll_offset(*fraction * (*prev_max_scroll).max(1.0));
    }

    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(content_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    content_ui.set_clip_rect(content_rect);

    let output = scroll_area.show(&mut content_ui, |ui| {
        let content_width = ui.available_width();
        let child_rect =
            egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(content_width, 0.0));
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(child_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                download_req = preview.show_content(ui);
            },
        );
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

    render_preview_header(ui, state, action);

    download_req
}

pub(crate) fn render_preview_header(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    let button_size = egui::vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y);
    let margin = f32::from(PREVIEW_CONTENT_PADDING);
    let button_rect = egui::Rect::from_min_size(
        egui::pos2(
            ui.max_rect().right() - margin - button_size.x,
            ui.max_rect().top() + margin,
        ),
        button_size,
    );
    let mut overlay_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(button_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    let has_doc = state.active_document().is_some();
    if overlay_ui
        .add_enabled(
            has_doc,
            egui::Button::new("\u{1F504}").min_size(button_size),
        )
        .on_hover_text(crate::i18n::get().preview.refresh_diagrams.clone())
        .clicked()
    {
        *action = AppAction::RefreshDiagrams;
    }
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
            .on_hover_text(crate::i18n::get().tab.nav_prev.clone())
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
            .on_hover_text(crate::i18n::get().tab.nav_next.clone())
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
            let is_split = mode == ViewMode::Split;
            if ui
                .selectable_label(is_split, crate::i18n::get().view_mode.split.clone())
                .clicked()
                && !is_split
            {
                mode = ViewMode::Split;
            }

            ui.selectable_value(
                &mut mode,
                ViewMode::CodeOnly,
                crate::i18n::get().view_mode.code.clone(),
            );
            ui.selectable_value(
                &mut mode,
                ViewMode::PreviewOnly,
                crate::i18n::get().view_mode.preview.clone(),
            );

            // Show split controls only while split mode is active.
            let prev_is_split = prev == ViewMode::Split;
            if is_split && (is_split == prev_is_split) {
                ui.separator();

                // Toggle pane order.
                let current_order = state.active_pane_order();
                let (order_icon, order_tip) = match current_order {
                    katana_platform::PaneOrder::EditorFirst => (
                        "📄|👁",
                        crate::i18n::get().split_toggle.preview_first.clone(),
                    ),
                    katana_platform::PaneOrder::PreviewFirst => {
                        ("👁|📄", crate::i18n::get().split_toggle.editor_first.clone())
                    }
                };
                if ui
                    .add(egui::Button::new(order_icon).sense(egui::Sense::click()))
                    .on_hover_text(order_tip)
                    .clicked()
                {
                    let new_order = match current_order {
                        katana_platform::PaneOrder::EditorFirst => {
                            katana_platform::PaneOrder::PreviewFirst
                        }
                        katana_platform::PaneOrder::PreviewFirst => {
                            katana_platform::PaneOrder::EditorFirst
                        }
                    };
                    state.set_active_pane_order(new_order);
                }

                // Toggle split direction.
                let current_dir = state.active_split_direction();
                let (dir_icon, dir_tip) = match current_dir {
                    katana_platform::SplitDirection::Horizontal => {
                        ("⇕", crate::i18n::get().split_toggle.vertical.clone())
                    }
                    katana_platform::SplitDirection::Vertical => {
                        ("⇔", crate::i18n::get().split_toggle.horizontal.clone())
                    }
                };
                if ui
                    .add(egui::Button::new(dir_icon).sense(egui::Sense::click()))
                    .on_hover_text(dir_tip)
                    .clicked()
                {
                    let new_dir = match current_dir {
                        katana_platform::SplitDirection::Horizontal => {
                            katana_platform::SplitDirection::Vertical
                        }
                        katana_platform::SplitDirection::Vertical => {
                            katana_platform::SplitDirection::Horizontal
                        }
                    };
                    state.set_active_split_direction(new_dir);
                }
            }
        },
    );
    if mode != prev {
        if mode == ViewMode::Split {
            state.ensure_active_split_state();
        }
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

pub(crate) struct TreeRenderContext<'a, 'b> {
    pub action: &'a mut AppAction,
    pub depth: usize,
    pub active_path: Option<&'b std::path::Path>,
    pub filter_set: Option<&'b std::collections::HashSet<std::path::PathBuf>>,
    pub expanded_directories: &'a mut std::collections::HashSet<std::path::PathBuf>,
}

pub(crate) fn render_tree_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    ctx: &mut TreeRenderContext<'_, '_>,
) {
    use katana_core::workspace::TreeEntry;
    let entry_path = match entry {
        TreeEntry::Directory { path, .. } => path,
        TreeEntry::File { path } => path,
    };
    if let Some(fs) = ctx.filter_set {
        if !fs.contains(entry_path) {
            return;
        }
    }
    match entry {
        TreeEntry::Directory { path, children } => {
            render_directory_entry(ui, path, children, ctx);
        }
        TreeEntry::File { path } => {
            render_file_entry(ui, entry, path, ctx);
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
    ctx: &mut TreeRenderContext<'_, '_>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let id = ui.make_persistent_id(format!("dir:{}", path.display()));

    // Check programmatic state for expansion
    let is_open = ctx.expanded_directories.contains(path);

    // Sync egui animation state with programmatic state
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, is_open);
    state.set_open(is_open);

    let arrow = if is_open { "▼" } else { "▶" };
    let dir_icon = if is_open { "📂" } else { "📁" };
    let prefix = indent_prefix(ctx.depth);
    let label_text = format!("{prefix}{arrow} {dir_icon} {name}");
    let file_tree_color = ui.visuals().text_color();

    // Full-width clickable label for better clickability and testability
    let resp = ui.add_sized(
        egui::vec2(ui.available_width(), 22.0),
        egui::Label::new(egui::RichText::new(label_text).color(file_tree_color))
            .sense(egui::Sense::click())
            .truncate(),
    );

    if resp.hovered() {
        ui.painter()
            .rect_filled(resp.rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
    }

    // Directory level Meta Info on Hover
    let path_str = path.display().to_string();
    let meta_text = format!(
        "{}\n{}",
        crate::i18n::tf("Path", &[("path", path_str.as_str())]),
        if let Ok(metadata) = std::fs::metadata(path) {
            let mod_time = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("Size: {} B\nModified (Unix): {}", metadata.len(), mod_time)
        } else {
            "Metadata unavailable".to_string()
        }
    );
    let resp = resp.on_hover_ui(|ui| {
        ui.label(meta_text);
    });

    // "Open All" Context Menu for directories
    resp.context_menu(|ui| {
        if ui
            .button(crate::i18n::get().menu.open_all.clone())
            .clicked()
        {
            let mut to_open = Vec::new();
            let mut to_expand = Vec::new();
            for child in children {
                child.collect_all_markdown_file_paths(&mut to_open);
                child.collect_all_directory_paths(&mut to_expand);
            }
            // Also expand the current directory
            ctx.expanded_directories.insert(path.to_path_buf());
            // And all subdirectories
            ctx.expanded_directories.extend(to_expand);

            if !to_open.is_empty() {
                *ctx.action = crate::app_state::AppAction::OpenMultipleDocuments(to_open);
            }
            ui.close();
        }
    });

    if resp.clicked() {
        let new_state = !is_open;
        state.set_open(new_state);
        if new_state {
            ctx.expanded_directories.insert(path.to_path_buf());
        } else {
            ctx.expanded_directories.remove(path);
        }
    }
    state.store(ui.ctx());

    if state.is_open() {
        let prev_depth = ctx.depth;
        ctx.depth += 1;
        for child in children {
            render_tree_entry(ui, child, ctx);
        }
        ctx.depth = prev_depth;
    }
}

fn gather_visible_paths(
    entries: &[katana_core::workspace::TreeEntry],
    regex: &regex::Regex,
    ws_root: &std::path::Path,
    visible: &mut std::collections::HashSet<std::path::PathBuf>,
) -> bool {
    let mut any_visible = false;
    for entry in entries {
        match entry {
            katana_core::workspace::TreeEntry::File { path } => {
                let rel = crate::shell_logic::relative_full_path(path, Some(ws_root));
                if regex.is_match(&rel) {
                    visible.insert(path.clone());
                    any_visible = true;
                }
            }
            katana_core::workspace::TreeEntry::Directory { path, children } => {
                if gather_visible_paths(children, regex, ws_root, visible) {
                    visible.insert(path.clone());
                    any_visible = true;
                }
            }
        }
    }
    any_visible
}

pub(crate) fn render_file_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    path: &std::path::Path,
    ctx: &mut TreeRenderContext<'_, '_>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let prefix = indent_prefix(ctx.depth);
    let icon = if entry.is_markdown() { "📄" } else { "  " };
    let label = format!("{prefix}{icon} {name}");

    let is_active = ctx.active_path.is_some_and(|ap| ap == path);

    let text_color = if is_active {
        ui.visuals().widgets.active.fg_stroke.color
    } else {
        ui.visuals().text_color()
    };
    let rich = egui::RichText::new(&label).color(text_color);
    let rich = if is_active { rich.strong() } else { rich };

    let row_height = ui.spacing().interact_size.y;
    let desired_size = egui::vec2(ui.available_width(), row_height);
    let (full_rect, row_resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if is_active {
        let highlight_color = ui.visuals().selection.bg_fill;
        ui.painter()
            .rect_filled(full_rect, ACTIVE_FILE_HIGHLIGHT_ROUNDING, highlight_color);
    }
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(full_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child_ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
    let label_resp = child_ui.add(
        egui::Label::new(rich)
            .truncate()
            .sense(egui::Sense::click()),
    );
    let resp = row_resp.union(label_resp);

    if resp.clicked() && entry.is_markdown() {
        *ctx.action = crate::app_state::AppAction::SelectDocument(path.to_path_buf());
    }
}

// ─────────────────────────────────────────────
// Split layout helpers
// ─────────────────────────────────────────────

/// Renders the split view and returns a pending download request if any.
///
/// Supports both `Horizontal` (left/right) and `Vertical` (top/bottom) splits,
/// with configurable pane order via `PaneOrder`.
fn render_split_mode(
    ctx: &egui::Context,
    app: &mut KatanaApp,
    split_dir: SplitDirection,
    pane_order: PaneOrder,
) -> Option<DownloadRequest> {
    match split_dir {
        SplitDirection::Horizontal => render_horizontal_split(ctx, app, pane_order),
        SplitDirection::Vertical => render_vertical_split(ctx, app, pane_order),
    }
}

/// Renders a left/right 50-50 split using SidePanel + CentralPanel.
///
/// The panel width is calculated as half of the available central area so that
/// both panes always occupy an equal share regardless of window size.
fn render_horizontal_split(
    ctx: &egui::Context,
    app: &mut KatanaApp,
    pane_order: PaneOrder,
) -> Option<DownloadRequest> {
    let available_width = ctx.available_rect().width();
    let half_width = (available_width * SPLIT_HALF_RATIO).max(SPLIT_PREVIEW_PANEL_MIN_WIDTH);
    let preview_bg = theme_bridge::rgb_to_color32(
        app.state
            .settings
            .settings()
            .effective_theme_colors()
            .preview_background,
    );
    let active_path = app.state.active_document().map(|d| d.path.clone());
    let mut scroll_state = (
        app.state.scroll_fraction,
        app.state.scroll_source,
        app.state.preview_max_scroll,
    );
    let mut download_req = None;
    let panel_id = match pane_order {
        PaneOrder::EditorFirst => preview_panel_id(active_path.as_deref(), "preview_panel_h_right"),
        PaneOrder::PreviewFirst => preview_panel_id(active_path.as_deref(), "preview_panel_h_left"),
    };

    // EditorFirst: preview panel on the right; PreviewFirst: preview panel on the left.
    let panel_side = match pane_order {
        PaneOrder::EditorFirst => egui::SidePanel::right(panel_id),
        PaneOrder::PreviewFirst => egui::SidePanel::left(panel_id),
    };

    panel_side
        .resizable(true)
        .min_width(SPLIT_PREVIEW_PANEL_MIN_WIDTH)
        .default_width(half_width)
        .frame(egui::Frame::NONE.fill(preview_bg))
        .show(ctx, |ui| {
            if let Some(path) = &active_path {
                let pane =
                    crate::shell::KatanaApp::get_preview_pane(&mut app.tab_previews, path.clone());
                download_req = render_preview_content(
                    ui,
                    pane,
                    &app.state,
                    &mut app.pending_action,
                    true,
                    &mut scroll_state,
                );
            }
        });

    // Sync preview's scroll state to app.state BEFORE the editor renders.
    // The editor reads/writes app.state directly; writing back after the
    // editor would overwrite the editor's consumption of scroll signals.
    app.state.scroll_fraction = scroll_state.0;
    app.state.scroll_source = scroll_state.1;
    app.state.preview_max_scroll = scroll_state.2;

    egui::CentralPanel::default().show(ctx, |ui| {
        render_editor_content(ui, &mut app.state, &mut app.pending_action, true);
    });

    download_req
}

/// Renders a top/bottom 50-50 split using TopBottomPanel + CentralPanel.
fn render_vertical_split(
    ctx: &egui::Context,
    app: &mut KatanaApp,
    pane_order: PaneOrder,
) -> Option<DownloadRequest> {
    let available_height = ctx.available_rect().height();
    let half_height = available_height * SPLIT_HALF_RATIO;
    let preview_bg = theme_bridge::rgb_to_color32(
        app.state
            .settings
            .settings()
            .effective_theme_colors()
            .preview_background,
    );
    let active_path = app.state.active_document().map(|d| d.path.clone());
    let mut scroll_state = (
        app.state.scroll_fraction,
        app.state.scroll_source,
        app.state.preview_max_scroll,
    );
    let mut download_req = None;
    let panel_id = match pane_order {
        PaneOrder::EditorFirst => {
            preview_panel_id(active_path.as_deref(), "preview_panel_v_bottom")
        }
        PaneOrder::PreviewFirst => preview_panel_id(active_path.as_deref(), "preview_panel_v_top"),
    };

    // EditorFirst: editor on top, preview on bottom; PreviewFirst: reversed.
    let show_preview_top = pane_order == PaneOrder::PreviewFirst;

    if show_preview_top {
        egui::TopBottomPanel::top(panel_id)
            .resizable(true)
            .default_height(half_height)
            .max_height(available_height * SPLIT_PANEL_MAX_RATIO)
            .frame(egui::Frame::NONE.fill(preview_bg))
            .show(ctx, |ui| {
                if let Some(path) = &active_path {
                    let pane = crate::shell::KatanaApp::get_preview_pane(
                        &mut app.tab_previews,
                        path.clone(),
                    );
                    download_req = render_preview_content(
                        ui,
                        pane,
                        &app.state,
                        &mut app.pending_action,
                        true,
                        &mut scroll_state,
                    );
                }
            });

        // Sync preview's scroll state to app.state BEFORE the editor renders.
        // The editor reads from app.state directly, so it must see the preview's
        // latest scroll_source/fraction. After the editor runs, its writes to
        // app.state are preserved (no subsequent overwrite).
        app.state.scroll_fraction = scroll_state.0;
        app.state.scroll_source = scroll_state.1;
        app.state.preview_max_scroll = scroll_state.2;

        egui::CentralPanel::default().show(ctx, |ui| {
            render_editor_content(ui, &mut app.state, &mut app.pending_action, true);
        });
    } else {
        egui::TopBottomPanel::bottom(panel_id)
            .resizable(true)
            .default_height(half_height)
            .max_height(available_height * SPLIT_PANEL_MAX_RATIO)
            .frame(egui::Frame::NONE.fill(preview_bg))
            .show(ctx, |ui| {
                if let Some(path) = &active_path {
                    let pane = crate::shell::KatanaApp::get_preview_pane(
                        &mut app.tab_previews,
                        path.clone(),
                    );
                    download_req = render_preview_content(
                        ui,
                        pane,
                        &app.state,
                        &mut app.pending_action,
                        true,
                        &mut scroll_state,
                    );
                }
            });

        app.state.scroll_fraction = scroll_state.0;
        app.state.scroll_source = scroll_state.1;
        app.state.preview_max_scroll = scroll_state.2;

        egui::CentralPanel::default().show(ctx, |ui| {
            render_editor_content(ui, &mut app.state, &mut app.pending_action, true);
        });
    }

    download_req
}

/// Renders the preview-only mode into the given ui.
fn render_preview_only(ui: &mut egui::Ui, app: &mut KatanaApp) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0.0,
        theme_bridge::rgb_to_color32(
            app.state
                .settings
                .settings()
                .effective_theme_colors()
                .preview_background,
        ),
    );
    let active_path = app.state.active_document().map(|d| d.path.clone());
    let mut scroll_state = (0.0_f32, ScrollSource::Neither, 0.0_f32);
    if let Some(path) = active_path {
        let pane = crate::shell::KatanaApp::get_preview_pane(&mut app.tab_previews, path);
        render_preview_content(
            ui,
            pane,
            &app.state,
            &mut app.pending_action,
            false,
            &mut scroll_state,
        );
    } else {
        ui.centered_and_justified(|ui| {
            ui.label(crate::i18n::get().workspace.no_document_selected.clone());
        });
    }
}

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

    // These FFI symbols are linked from Objective-C (macos_menu.m) and called
    // only at runtime; the Rust compiler cannot see the call sites.
    #[allow(dead_code)]
    extern "C" {
        pub fn katana_setup_native_menu();
        pub fn katana_poll_menu_action() -> i32;
        pub fn katana_set_app_icon_png(png_data: *const u8, png_len: std::ffi::c_ulong);
        pub fn katana_set_process_name();
        pub fn katana_update_menu_strings(
            file: *const std::ffi::c_char,
            open_workspace: *const std::ffi::c_char,
            save: *const std::ffi::c_char,
            settings: *const std::ffi::c_char,
            preferences: *const std::ffi::c_char,
            language: *const std::ffi::c_char,
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
unsafe fn native_update_menu_strings(
    file: &str,
    open_workspace: &str,
    save: &str,
    settings: &str,
    preferences: &str,
    language: &str,
) {
    let f = std::ffi::CString::new(file).unwrap_or_default();
    let ow = std::ffi::CString::new(open_workspace).unwrap_or_default();
    let s = std::ffi::CString::new(save).unwrap_or_default();
    let st = std::ffi::CString::new(settings).unwrap_or_default();
    let p = std::ffi::CString::new(preferences).unwrap_or_default();
    let l = std::ffi::CString::new(language).unwrap_or_default();
    native_menu::katana_update_menu_strings(
        f.as_ptr(),
        ow.as_ptr(),
        s.as_ptr(),
        st.as_ptr(),
        p.as_ptr(),
        l.as_ptr(),
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
        );
    }
}

#[cfg(any(not(target_os = "macos"), test))]
pub fn update_native_menu_strings_from_i18n() {}

// ─────────────────────────────────────────────
// eframe::App Implementation (egui Main Rendering Loop)
// ─────────────────────────────────────────────

use crate::shell::{KatanaApp, SIDEBAR_COLLAPSED_TOGGLE_WIDTH, SPLIT_PREVIEW_PANEL_MIN_WIDTH};

// Half-panel ratio for responsive 50/50 split.
const SPLIT_HALF_RATIO: f32 = 0.5;
/// Maximum ratio for TopBottomPanel in vertical split.
/// Prevents preview from consuming more than 70% of the available height,
/// guaranteeing the editor retains at least 30% for scrolling.
const SPLIT_PANEL_MAX_RATIO: f32 = 0.7;
const PREVIEW_CONTENT_PADDING: i8 = 12;

fn preview_panel_id(path: Option<&std::path::Path>, base: &'static str) -> egui::Id {
    match path {
        Some(path) => egui::Id::new((base, path)),
        None => egui::Id::new(base),
    }
}

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme colours to egui Visuals (only when the palette changed)
        let theme_colors = self.state.settings.settings().effective_theme_colors();
        if self.cached_theme.as_ref() != Some(&theme_colors) {
            let dark = theme_colors.mode == katana_platform::theme::ThemeMode::Dark;
            ctx.set_visuals(theme_bridge::visuals_from_theme(&theme_colors));
            katana_core::markdown::color_preset::DiagramColorPreset::set_dark_mode(dark);
            self.cached_theme = Some(theme_colors.clone());
            // Re-render diagrams with the new theme colours.
            // Only set if no action is already pending (e.g. OpenWorkspace from startup restore).
            if matches!(self.pending_action, AppAction::None) {
                self.pending_action = AppAction::RefreshDiagrams;
            }
        }

        // Apply font size to egui text styles (only when the size changed)
        let font_size = self.state.settings.settings().clamped_font_size();
        if self.cached_font_size != Some(font_size) {
            theme_bridge::apply_font_size(ctx, font_size);
            self.cached_font_size = Some(font_size);
        }

        // Apply font family by rebuilding FontDefinitions (only when family changed)
        let font_family = self.state.settings.settings().font.family.clone();
        if self.cached_font_family.as_deref() != Some(&font_family) {
            theme_bridge::apply_font_family(ctx, &font_family);
            self.cached_font_family = Some(font_family);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::P,
            ))
        }) {
            self.state.show_search_modal = true;
            // The query will persist across invocations as per standard fuzzy finders
        }

        self.poll_download(ctx);
        self.poll_workspace_load(ctx);

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
                            .on_hover_text(crate::i18n::get().workspace.workspace_title.clone())
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

        if is_split {
            let split_dir = self.state.active_split_direction();
            let pane_order = self.state.active_pane_order();
            download_req = render_split_mode(ctx, self, split_dir, pane_order);
        }

        if !is_split {
            egui::CentralPanel::default().show(ctx, |ui| match current_mode {
                ViewMode::CodeOnly => {
                    render_editor_content(ui, &mut self.state, &mut self.pending_action, false);
                }
                ViewMode::PreviewOnly => {
                    render_preview_only(ui, self);
                }
                ViewMode::Split => {}
            });
        }

        if let Some(req) = download_req {
            self.start_download(req);
        }

        // Settings window
        crate::settings_window::render_settings_window(
            ctx,
            &mut self.state,
            &mut self.settings_preview,
        );

        if self.state.show_search_modal {
            render_search_modal(ctx, &mut self.state, &mut self.pending_action);
        }

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

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{self, pos2, Rect};
    use katana_core::{document::Document, workspace::TreeEntry};
    use std::path::{Path, PathBuf};

    const PREVIEW_CONTENT_PADDING: f32 = 12.0;

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
            .open_documents
            .push(Document::new(path, "# Heading\n\nBody"));
        state.active_doc_idx = Some(0);
        state
    }

    fn app_with_preview_doc(path: &Path, markdown: &str) -> KatanaApp {
        let mut app = KatanaApp::new(state_with_active_doc(path));
        if let Some(doc) = app.state.active_document_mut() {
            doc.buffer = markdown.to_string();
        }
        let mut pane = PreviewPane::default();
        let cache = app.state.cache.clone();
        let concurrency = app
            .state
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

    #[test]
    fn preview_header_leaves_height_for_preview_body() {
        let ctx = egui::Context::default();
        let state = state_with_active_doc(std::path::Path::new("/tmp/preview.md"));
        let mut action = AppAction::None;
        let mut before_height = 0.0;
        let mut remaining_height = 0.0;

        let _ = ctx.run(test_input(egui::vec2(800.0, 600.0)), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                before_height = ui.available_height();
                render_preview_header(ui, &state, &mut action);
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
        let ctx = egui::Context::default();
        let path = std::path::PathBuf::from("/tmp/CHANGELOG.md");
        let entry = TreeEntry::File { path: path.clone() };
        let mut action = AppAction::None;
        let mut expanded_directories = std::collections::HashSet::new();

        let output = ctx.run(test_input(egui::vec2(320.0, 200.0)), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut render_ctx = TreeRenderContext {
                    action: &mut action,
                    force: None,
                    depth: 0,
                    active_path: Some(path.as_path()),
                    filter_set: None,
                    expanded_directories: &mut expanded_directories,
                };
                render_file_entry(ui, &entry, &path, &mut render_ctx);
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
        let ctx = egui::Context::default();
        let path = PathBuf::from("/tmp/padding.md");
        let mut app = app_with_preview_doc(&path, "# PaddingHeading\n\nBody");
        let output = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(path.as_path()), "preview_panel_h_right"),
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
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/active.md");
        let stale = PathBuf::from("/tmp/stale.md");
        let mut app = app_with_preview_doc(&active, "Body");

        ctx.data_mut(|data| {
            data.insert_persisted(
                preview_panel_id(Some(stale.as_path()), "preview_panel_h_right"),
                egui::containers::panel::PanelState {
                    rect: Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(240.0, 800.0)),
                },
            );
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_h_right"),
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
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/active.md");
        let mut app = app_with_preview_doc(&active, "# Title\n\nBody");

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_h_right"),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_h_right"),
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
        let ctx = egui::Context::default();
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
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_h_right"),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_h_right"),
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
        let ctx = egui::Context::default();
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
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_h_right"),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_h_right"),
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
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/active.md");
        let stale = PathBuf::from("/tmp/stale.md");
        let mut app = app_with_preview_doc(&active, "Body");

        ctx.data_mut(|data| {
            data.insert_persisted(
                preview_panel_id(Some(stale.as_path()), "preview_panel_v_bottom"),
                egui::containers::panel::PanelState {
                    rect: Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1200.0, 180.0)),
                },
            );
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            render_vertical_split(ctx, &mut app, PaneOrder::EditorFirst);
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_v_bottom"),
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
        let ctx = egui::Context::default();
        let path = PathBuf::from("/tmp/long-line.md");
        let long_line = "あ".repeat(240);
        let mut app = app_with_preview_doc(&path, &long_line);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(path.as_path()), "preview_panel_h_right"),
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
        let ctx = egui::Context::default();
        let path = PathBuf::from("/tmp/long-inline-code.md");
        let inline_code = format!("`{}`", "あ".repeat(240));
        let mut app = app_with_preview_doc(&path, &inline_code);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(path.as_path()), "preview_panel_h_right"),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let text_shape = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains(&"あ".repeat(60)) =>
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
        let ctx = egui::Context::default();
        let path = PathBuf::from("/tmp/blockquote-strong.md");
        let markdown = concat!(
            "> **Note:** On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple. ",
            "Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n"
        );
        let mut app = app_with_preview_doc(&path, markdown);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(path.as_path()), "preview_panel_h_right"),
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
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/vsplit_scroll.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);
        let total_height = 800.0_f32;

        // Run 3 frames for layout stabilization
        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, total_height)), |ctx| {
                render_vertical_split(ctx, &mut app, PaneOrder::EditorFirst);
            });
        }

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            preview_panel_id(Some(active.as_path()), "preview_panel_v_bottom"),
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
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/vsplit_sync_e2p.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        // Stabilize layout (5 frames)
        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_vertical_split(ctx, &mut app, PaneOrder::EditorFirst);
            });
        }

        // Simulate editor scroll by setting scroll state
        app.state.scroll_fraction = 0.5;
        app.state.scroll_source = ScrollSource::Editor;

        // Run 3 frames for sync to propagate
        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_vertical_split(ctx, &mut app, PaneOrder::EditorFirst);
            });
        }

        // After sync, scroll_source must settle to Neither.
        // If it bounces to Preview, the sync is creating an oscillation loop.
        assert_eq!(
            app.state.scroll_source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither after consumption. \
             Got {:?}, fraction={:.4}",
            app.state.scroll_source,
            app.state.scroll_fraction,
        );
    }

    /// Same editor→preview sync test for horizontal split — expected to PASS.
    #[test]
    fn horizontal_split_editor_to_preview_scroll_sync() {
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/hsplit_sync_e2p.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
            });
        }

        app.state.scroll_fraction = 0.5;
        app.state.scroll_source = ScrollSource::Editor;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_horizontal_split(ctx, &mut app, PaneOrder::EditorFirst);
            });
        }

        assert_eq!(
            app.state.scroll_source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither in horizontal split. \
             Got {:?}, fraction={:.4}",
            app.state.scroll_source,
            app.state.scroll_fraction,
        );
    }

    /// Scenario 5: After swapping order (PreviewFirst), the same
    /// editor→preview sync must work in vertical split.
    #[test]
    fn vertical_split_editor_to_preview_scroll_sync_after_swap() {
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/vsplit_sync_swap.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        // Use PreviewFirst (swapped order)
        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_vertical_split(ctx, &mut app, PaneOrder::PreviewFirst);
            });
        }

        app.state.scroll_fraction = 0.5;
        app.state.scroll_source = ScrollSource::Editor;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_vertical_split(ctx, &mut app, PaneOrder::PreviewFirst);
            });
        }

        assert_eq!(
            app.state.scroll_source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither after order swap. \
             Got {:?}, fraction={:.4}",
            app.state.scroll_source,
            app.state.scroll_fraction,
        );
    }

    /// Verify preview→editor sync direction also works in vertical split.
    /// Set scroll_source=Preview and verify it transitions to Neither.
    #[test]
    fn vertical_split_preview_to_editor_scroll_sync() {
        let ctx = egui::Context::default();
        let active = PathBuf::from("/tmp/vsplit_sync_p2e.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_vertical_split(ctx, &mut app, PaneOrder::EditorFirst);
            });
        }

        // Simulate preview scroll
        app.state.scroll_fraction = 0.5;
        app.state.scroll_source = ScrollSource::Preview;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                render_vertical_split(ctx, &mut app, PaneOrder::EditorFirst);
            });
        }

        assert_eq!(
            app.state.scroll_source,
            ScrollSource::Neither,
            "Preview→Editor sync must settle to Neither in vertical split. \
             Got {:?}, fraction={:.4}",
            app.state.scroll_source,
            app.state.scroll_fraction,
        );
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

const SEARCH_MODAL_WIDTH: f32 = 500.0;
const SEARCH_MODAL_HEIGHT: f32 = 400.0;

pub(crate) fn render_search_modal(
    ctx: &egui::Context,
    state: &mut AppState,
    action: &mut AppAction,
) {
    let mut is_open = state.show_search_modal;
    egui::Window::new(crate::i18n::get().search.modal_title.clone())
        .open(&mut is_open)
        .collapsible(false)
        .resizable(true)
        .default_size(egui::vec2(SEARCH_MODAL_WIDTH, SEARCH_MODAL_HEIGHT))
        .show(ctx, |ui| {
            // Focus on the text edit automatically
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text(crate::i18n::get().search.query_hint.clone())
                    .desired_width(f32::INFINITY),
            );
            response.request_focus();

            let mut include_regexes = Vec::new();
            let mut include_valid = true;
            if !state.search_include_pattern.is_empty() {
                for pat in state.search_include_pattern.split(',') {
                    let pat = pat.trim();
                    if !pat.is_empty() {
                        match regex::Regex::new(pat) {
                            Ok(re) => include_regexes.push(re),
                            Err(_) => include_valid = false,
                        }
                    }
                }
            }

            let mut exclude_regexes = Vec::new();
            let mut exclude_valid = true;
            if !state.search_exclude_pattern.is_empty() {
                for pat in state.search_exclude_pattern.split(',') {
                    let pat = pat.trim();
                    if !pat.is_empty() {
                        match regex::Regex::new(pat) {
                            Ok(re) => exclude_regexes.push(re),
                            Err(_) => exclude_valid = false,
                        }
                    }
                }
            }

            let include_color = if include_valid {
                ui.visuals().text_color()
            } else {
                egui::Color32::RED
            };

            let exclude_color = if exclude_valid {
                ui.visuals().text_color()
            } else {
                egui::Color32::RED
            };

            ui.add(
                egui::TextEdit::singleline(&mut state.search_include_pattern)
                    .hint_text(crate::i18n::get().search.include_pattern_hint.clone())
                    .text_color(include_color)
                    .desired_width(f32::INFINITY),
            );

            ui.add(
                egui::TextEdit::singleline(&mut state.search_exclude_pattern)
                    .hint_text(crate::i18n::get().search.exclude_pattern_hint.clone())
                    .text_color(exclude_color)
                    .desired_width(f32::INFINITY),
            );

            let current_params = (
                state.search_query.clone(),
                state.search_include_pattern.clone(),
                state.search_exclude_pattern.clone(),
            );

            if state.last_search_params.as_ref() != Some(&current_params) {
                state.last_search_params = Some(current_params);

                let query = state.search_query.to_lowercase();
                if query.is_empty() && include_regexes.is_empty() && exclude_regexes.is_empty() {
                    state.search_results.clear();
                } else if let Some(ws) = &state.workspace {
                    let mut results = Vec::new();
                    let ws_root = ws.root.clone();

                    fn collect_matches(
                        entries: &[katana_core::workspace::TreeEntry],
                        query: &str,
                        include_regexes: &[regex::Regex],
                        exclude_regexes: &[regex::Regex],
                        ws_root: &std::path::Path,
                        results: &mut Vec<std::path::PathBuf>,
                    ) {
                        if results.len() >= 100 {
                            return;
                        }
                        for entry in entries {
                            match entry {
                                katana_core::workspace::TreeEntry::File { path } => {
                                    let rel =
                                        crate::shell_logic::relative_full_path(path, Some(ws_root));

                                    // 1. Exclude check (priority)
                                    let mut is_excluded = false;
                                    for re in exclude_regexes {
                                        if re.is_match(&rel) {
                                            is_excluded = true;
                                            break;
                                        }
                                    }
                                    if is_excluded {
                                        continue;
                                    }

                                    // 2. Query check
                                    let mut matches_query = true;
                                    if !query.is_empty() {
                                        matches_query = rel.to_lowercase().contains(query);
                                    }

                                    // 3. Include check
                                    let mut matches_include = true;
                                    if !include_regexes.is_empty() {
                                        matches_include = false;
                                        for re in include_regexes {
                                            if re.is_match(&rel) {
                                                matches_include = true;
                                                break;
                                            }
                                        }
                                    }

                                    if matches_query && matches_include {
                                        results.push(path.clone());
                                        if results.len() >= 100 {
                                            return;
                                        }
                                    }
                                }
                                katana_core::workspace::TreeEntry::Directory {
                                    children, ..
                                } => {
                                    collect_matches(
                                        children,
                                        query,
                                        include_regexes,
                                        exclude_regexes,
                                        ws_root,
                                        results,
                                    );
                                }
                            }
                        }
                    }

                    collect_matches(
                        &ws.tree,
                        &query,
                        &include_regexes,
                        &exclude_regexes,
                        &ws_root,
                        &mut results,
                    );
                    state.search_results = results;
                }
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if state.search_results.is_empty() && !state.search_query.is_empty() {
                        ui.label(crate::i18n::get().search.no_results.clone());
                    } else {
                        let ws_root = state.workspace.as_ref().map(|ws| ws.root.clone());
                        for path in &state.search_results {
                            let rel =
                                crate::shell_logic::relative_full_path(path, ws_root.as_deref());
                            if ui.selectable_label(false, rel).clicked() && path.exists() {
                                *action = AppAction::SelectDocument(path.clone());
                                // Close the modal by updating state directly
                                state.show_search_modal = false;
                            }
                        }
                    }
                });
        });

    if !is_open {
        state.show_search_modal = false;
    }
}
