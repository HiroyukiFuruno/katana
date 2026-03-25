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

const INVISIBLE_LABEL_SIZE: f32 = 0.1;
/// Splash screen animation repaint interval (~30fps).
const SPLASH_REPAINT_INTERVAL_MS: u64 = 32;

fn invisible_label(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(INVISIBLE_LABEL_SIZE)
        .color(egui::Color32::TRANSPARENT)
}

use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, EDITOR_INITIAL_VISIBLE_ROWS, FILE_TREE_PANEL_DEFAULT_WIDTH,
    FILE_TREE_PANEL_MIN_WIDTH, NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_ITEM_SPACING,
    RECENT_WORKSPACES_SPACING, SCROLL_SYNC_DEAD_ZONE, TAB_INTER_ITEM_SPACING,
    TAB_NAV_BUTTONS_AREA_WIDTH, TAB_TOOLTIP_SHOW_DELAY_SECS, TREE_LABEL_HOFFSET, TREE_ROW_HEIGHT,
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
            ui.menu_button(crate::i18n::get().menu.help.clone(), |ui| {
                render_help_menu(ui, action);
            });
            render_header_right(ui, state);
        });
    });
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_help_menu(ui: &mut egui::Ui, action: &mut AppAction) {
    if ui.button(crate::i18n::get().menu.about.clone()).clicked() {
        *action = AppAction::ToggleAbout;
        ui.close();
    }
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

    let has_doc = state.active_document().is_some();
    ui.add_enabled_ui(has_doc, |ui| {
        ui.menu_button(crate::i18n::get().menu.export.clone(), |ui| {
            if ui
                .button(crate::i18n::get().menu.export_html.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Html);
                ui.close();
            }
            if ui
                .button(crate::i18n::get().menu.export_pdf.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Pdf);
                ui.close();
            }
            if ui
                .button(crate::i18n::get().menu.export_png.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Png);
                ui.close();
            }
            if ui
                .button(crate::i18n::get().menu.export_jpg.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Jpg);
                ui.close();
            }
        });
    });
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

pub(crate) fn render_status_bar(
    ctx: &egui::Context,
    state: &AppState,
    export_filenames: &[String],
) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let (msg, kind) = if let Some((msg, kind)) = &state.status_message {
                (msg.as_str(), Some(kind))
            } else {
                (crate::i18n::get().status.ready.as_str(), None)
            };

            let (color, icon) = match kind {
                Some(crate::app_state::StatusType::Error) => {
                    (egui::Color32::RED, Some(crate::Icon::Error))
                }
                Some(crate::app_state::StatusType::Warning) => {
                    (ui.visuals().warn_fg_color, Some(crate::Icon::Warning))
                }
                Some(crate::app_state::StatusType::Success) => (
                    egui::Color32::from_rgb(0, STATUS_SUCCESS_GREEN, 0),
                    Some(crate::Icon::Success),
                ),
                Some(crate::app_state::StatusType::Info) => {
                    (ui.visuals().text_color(), Some(crate::Icon::Info))
                }
                _ => (ui.visuals().text_color(), None),
            };

            ui.add_space(STATUS_BAR_ICON_SPACING);
            if let Some(i) = icon {
                ui.add(egui::Image::new(i.uri()).tint(color));
                ui.add_space(2.0);
            }
            ui.colored_label(color, msg);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !export_filenames.is_empty() {
                    let total = export_filenames.len();
                    ui.spinner();
                    for (i, filename) in export_filenames.iter().enumerate() {
                        let numbered = crate::i18n::tf(
                            &crate::i18n::get().export.exporting,
                            &[("filename", &format!("({}/{}) {}", i + 1, total, filename))],
                        );
                        ui.label(numbered);
                    }
                }
                const DIRTY_DOT_MAX_HEIGHT: f32 = 10.0;
                if state.is_dirty() {
                    ui.add(
                        egui::Image::new(crate::Icon::Dot.uri())
                            .tint(ui.visuals().text_color())
                            .max_height(DIRTY_DOT_MAX_HEIGHT),
                    );
                }
            });
        });
    });
}

const WORKSPACE_SPINNER_OUTER_MARGIN: f32 = 10.0;
const WORKSPACE_SPINNER_INNER_MARGIN: f32 = 10.0;
const WORKSPACE_SPINNER_TEXT_MARGIN: f32 = 5.0;
/// Green channel value for the success status bar color.
const STATUS_SUCCESS_GREEN: u8 = 200;
/// Spacing before the icon in the status bar.
const STATUS_BAR_ICON_SPACING: f32 = 4.0;

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
                        .add(egui::Button::image(
                            crate::icon::Icon::ChevronLeft
                                .ui_image(ui, crate::icon::IconSize::Small),
                        ))
                        .on_hover_text(crate::i18n::get().action.collapse_sidebar.clone())
                        .clicked()
                    {
                        state.show_workspace = false;
                    }
                });
            });
            if state.workspace.is_some() {
                ui.horizontal(|ui| {
                    let btn_resp = ui
                        .add(egui::Button::image(
                            crate::Icon::ExpandAll.ui_image(ui, crate::icon::IconSize::Small),
                        ))
                        .on_hover_text(crate::i18n::get().action.expand_all.clone());
                    btn_resp.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "+")
                    });
                    if btn_resp.clicked() {
                        if let Some(ws) = &state.workspace {
                            state
                                .expanded_directories
                                .extend(ws.collect_all_directory_paths());
                        }
                    }
                    let btn_resp = ui
                        .add(egui::Button::image(
                            crate::Icon::CollapseAll.ui_image(ui, crate::icon::IconSize::Small),
                        ))
                        .on_hover_text(crate::i18n::get().action.collapse_all.clone());
                    btn_resp.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "-")
                    });
                    if btn_resp.clicked() {
                        state.force_tree_open = Some(false);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let icon_bg = if ui.visuals().dark_mode {
                            egui::Color32::TRANSPARENT
                        } else {
                            egui::Color32::from_gray(LIGHT_MODE_ICON_BG) // Always gray for icons in light mode
                        };

                        if !state.settings.settings().workspace.paths.is_empty() {
                            let ws_history_img =
                                crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Small);
                            ui.scope(|ui| {
                                ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;
                                ui.menu_image_button(ws_history_img, |ui| {
                                    for path in
                                        state.settings.settings().workspace.paths.iter().rev()
                                    {
                                        ui.horizontal(|ui| {
                                            if ui
                                                .add(egui::Button::image_and_text(
                                                    crate::Icon::Remove
                                                        .ui_image(ui, crate::icon::IconSize::Small),
                                                    invisible_label("x"),
                                                ))
                                                .on_hover_text(
                                                    crate::i18n::get()
                                                        .action
                                                        .remove_workspace
                                                        .clone(),
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
                                .on_hover_text(
                                    crate::i18n::get().workspace.recent_workspaces.clone(),
                                );
                            });
                        }

                        if ui
                            .add(
                                egui::Button::image_and_text(
                                    crate::Icon::Refresh.ui_image(ui, crate::icon::IconSize::Small),
                                    invisible_label("🔄"),
                                )
                                .fill(icon_bg),
                            )
                            .on_hover_text(crate::i18n::get().action.refresh_workspace.clone())
                            .clicked()
                        {
                            *action = AppAction::RefreshWorkspace;
                        }

                        let filter_btn_color = if state.filter_enabled {
                            if ui.visuals().dark_mode {
                                ui.visuals().selection.bg_fill
                            } else {
                                egui::Color32::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
                                // Darker gray when active in light mode
                            }
                        } else {
                            icon_bg
                        };

                        if ui
                            .add(
                                egui::Button::image_and_text(
                                    crate::Icon::Filter.ui_image(ui, crate::icon::IconSize::Small),
                                    invisible_label("\u{2207}"),
                                )
                                .fill(filter_btn_color),
                            )
                            .on_hover_text(crate::i18n::get().action.toggle_filter.clone())
                            .clicked()
                        {
                            state.filter_enabled = !state.filter_enabled;
                        }

                        if ui
                            .add(
                                egui::Button::image_and_text(
                                    crate::Icon::Search.ui_image(ui, crate::icon::IconSize::Small),
                                    invisible_label("🔍"),
                                )
                                .fill(icon_bg),
                            )
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
            let is_negated = state.filter_query.starts_with('!');
            let query_str = if is_negated {
                &state.filter_query[1..]
            } else {
                &state.filter_query
            };

            if let Ok(regex) = regex::Regex::new(query_str) {
                if state.filter_cache.as_ref().map(|(q, _)| q) != Some(&state.filter_query) {
                    let mut visible = std::collections::HashSet::new();
                    gather_visible_paths(&entries, &regex, is_negated, &ws_root, &mut visible);
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
    state: &mut AppState,
    action: &mut AppAction,
    scroll_sync: bool,
    scroll_state: &mut (f32, ScrollSource, f32),
) -> Option<DownloadRequest> {
    let mut download_req = None;
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    let outer_rect = ui.available_rect_before_wrap();
    ui.allocate_rect(outer_rect, egui::Sense::hover());

    let (fraction, source, prev_max_scroll) = scroll_state;
    let mut scroll_area = egui::ScrollArea::vertical()
        .id_salt("preview_scroll")
        .auto_shrink(std::array::from_fn(|_| false));

    let mut target_scroll_offset = *fraction * (*prev_max_scroll).max(1.0);
    let consuming_editor = scroll_sync && *source == ScrollSource::Editor;
    if consuming_editor {
        if let Some(doc) = state.active_document() {
            let _buffer = &doc.buffer;
            let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
            let editor_y = *fraction * state.editor_max_scroll.max(1.0);

            let mut points = Vec::new();
            points.push((0.0, 0.0));
            for (span, rect) in &preview.heading_anchors {
                let e_y = span.start as f32 * row_height;
                let p_y = (rect.min.y - preview.content_top_y).max(0.0);
                points.push((e_y, p_y));
            }
            points.push((
                state.editor_max_scroll.max(1.0),
                (*prev_max_scroll).max(1.0),
            ));

            let mut mapped_y = 0.0;
            for i in 0..points.len() - 1 {
                let (e_y1, p_y1) = points[i];
                let (e_y2, p_y2) = points[i + 1];
                if editor_y >= e_y1 && editor_y <= e_y2 {
                    if e_y2 > e_y1 {
                        let t = (editor_y - e_y1) / (e_y2 - e_y1);
                        mapped_y = p_y1 + t * (p_y2 - p_y1);
                    } else {
                        mapped_y = p_y1;
                    }
                    break;
                }
            }
            if editor_y > points.last().unwrap().0 {
                mapped_y = points.last().unwrap().1;
            }
            target_scroll_offset = mapped_y;
        }

        scroll_area = scroll_area.vertical_scroll_offset(target_scroll_offset);
    }

    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(outer_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    content_ui.set_clip_rect(outer_rect);

    let output = scroll_area.show(&mut content_ui, |ui| {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(
                PREVIEW_CONTENT_PADDING,
                PREVIEW_CONTENT_PADDING,
            ))
            .show(ui, |ui| {
                let content_width = ui.available_width();
                let child_rect = egui::Rect::from_min_size(
                    ui.next_widget_position(),
                    egui::vec2(content_width, 0.0),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(child_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        const PREVIEW_PANE_TOP_BOTTOM_PADDING: f32 = 4.0; // 0.25rem padding
                        ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                        let mut hovered_lines = Vec::new();
                        let (req, actions) = preview.show_content(
                            ui,
                            state.active_editor_line,
                            Some(&mut hovered_lines),
                        );
                        if scroll_sync && *source != ScrollSource::Preview {
                            state.hovered_preview_lines = hovered_lines.clone();
                        }

                        if ui.rect_contains_pointer(ui.min_rect())
                            && ui.input(|i| i.pointer.primary_clicked())
                        {
                            if let Some(hovered) = hovered_lines.first() {
                                state.scroll_to_line = Some(hovered.start);
                            }
                        }
                        download_req = req;
                        if let Some((global_index, new_state)) = actions.into_iter().next() {
                            *action = AppAction::ToggleTaskList {
                                global_index,
                                new_state,
                            };
                        }
                        ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                    },
                );
            });
    });

    if scroll_sync {
        let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
        *prev_max_scroll = max_scroll;

        if consuming_editor {
            *source = ScrollSource::Neither;
            if max_scroll > 0.0 {
                // If mapped via piecewise, we should inversely map the actual offset back to fraction to stay stable.
                // However, holding the fraction is generally safer.
                // We leave fraction alone when consuming editor input.
            }
        } else {
            if max_scroll > 0.0 {
                let preview_y = output.state.offset.y;
                let mut editor_target_y = preview_y;

                if let Some(doc) = state.active_document() {
                    let _buffer = &doc.buffer;
                    let row_height = ui.text_style_height(&egui::TextStyle::Monospace);

                    let mut points = Vec::new();
                    points.push((0.0, 0.0));
                    for (span, rect) in &preview.heading_anchors {
                        let e_y = span.start as f32 * row_height;
                        let p_y = (rect.min.y - preview.content_top_y).max(0.0);
                        points.push((e_y, p_y));
                    }
                    points.push((state.editor_max_scroll.max(1.0), max_scroll));

                    let mut mapped_y = 0.0;
                    for i in 0..points.len() - 1 {
                        let (e_y1, p_y1) = points[i];
                        let (e_y2, p_y2) = points[i + 1];
                        if preview_y >= p_y1 && preview_y <= p_y2 {
                            if p_y2 > p_y1 {
                                let t = (preview_y - p_y1) / (p_y2 - p_y1);
                                mapped_y = e_y1 + t * (e_y2 - e_y1);
                            } else {
                                mapped_y = e_y1;
                            }
                            break;
                        }
                    }
                    if preview_y > points.last().unwrap().1 {
                        mapped_y = points.last().unwrap().0;
                    }
                    editor_target_y = mapped_y;
                }

                let current_fraction =
                    (editor_target_y / state.editor_max_scroll.max(1.0)).clamp(0.0, 1.0);
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
    let spacing = ui.spacing().item_spacing.x;
    let mut button_count = 2.0; // Refresh + Export
    if state.settings.settings().layout.toc_visible {
        button_count += 1.0;
    }
    let total_width = (button_size.x * button_count) + (spacing * (button_count - 1.0));

    let button_rect = egui::Rect::from_min_size(
        egui::pos2(
            ui.max_rect().right() - margin - total_width,
            ui.max_rect().top() + margin,
        ),
        egui::vec2(total_width, button_size.y),
    );
    let mut overlay_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(button_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    let has_doc = state.active_document().is_some();

    let icon_bg = if ui.visuals().dark_mode {
        egui::Color32::TRANSPARENT
    } else {
        egui::Color32::from_gray(LIGHT_MODE_ICON_BG)
    };

    if overlay_ui
        .add_enabled(
            has_doc,
            egui::Button::image_and_text(
                crate::Icon::Refresh.ui_image(ui, crate::icon::IconSize::Medium),
                invisible_label("🔄"),
            )
            .min_size(button_size)
            .fill(icon_bg),
        )
        .on_hover_text(crate::i18n::get().preview.refresh_diagrams.clone())
        .clicked()
    {
        *action = AppAction::RefreshDiagrams;
    }

    let export_img =
        egui::Image::new(crate::icon::Icon::Export.uri()).tint(overlay_ui.visuals().text_color());
    overlay_ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;
        ui.menu_image_button(export_img, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            if ui
                .button(crate::i18n::get().menu.export_html.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Html);
                ui.close();
            }
            if ui
                .button(crate::i18n::get().menu.export_pdf.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Pdf);
                ui.close();
            }
            if ui
                .button(crate::i18n::get().menu.export_png.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Png);
                ui.close();
            }
            if ui
                .button(crate::i18n::get().menu.export_jpg.clone())
                .clicked()
            {
                *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Jpg);
                ui.close();
            }
        });
    });

    if state.settings.settings().layout.toc_visible {
        let toc_bg = if state.show_toc {
            if ui.visuals().dark_mode {
                ui.visuals().selection.bg_fill
            } else {
                egui::Color32::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
            }
        } else {
            icon_bg
        };
        if overlay_ui
            .add_enabled(
                has_doc,
                egui::Button::image_and_text(
                    crate::Icon::Toc.ui_image(ui, crate::icon::IconSize::Medium),
                    invisible_label("toggle_toc"),
                )
                .min_size(button_size)
                .fill(toc_bg),
            )
            .on_hover_text(crate::i18n::get().action.toggle_toc.clone())
            .clicked()
        {
            *action = AppAction::ToggleToc;
        }
    }
}

/// Tab bar: Displays tabs of open documents side-by-side.
pub(crate) fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState, action: &mut AppAction) {
    const MAX_TAB_WIDTH: f32 = 200.0;
    const PINNED_TAB_MAX_WIDTH: f32 = 60.0;

    let mut close_idx: Option<usize> = None;
    let mut tab_action: Option<AppAction> = None;
    let mut dragged_source: Option<(usize, egui::Pos2)> = None;
    let mut tab_rects: Vec<(usize, egui::Rect)> = Vec::new();

    let ws_root = state.workspace.as_ref().map(|ws| ws.root.clone());
    let doc_count = state.open_documents.len();

    ui.style_mut().interaction.tooltip_delay = TAB_TOOLTIP_SHOW_DELAY_SECS;

    ui.horizontal(|ui| {
        let nav_button_width = TAB_NAV_BUTTONS_AREA_WIDTH;
        let scroll_width = ui.available_width() - nav_button_width;

        let should_scroll = ui.memory_mut(|mem| {
            mem.data
                .get_temp::<bool>(egui::Id::new("scroll_tab_req"))
                .unwrap_or(false)
        });

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
                        let title = if doc.is_pinned {
                            format!("📌 {filename}{dirty_suffix}")
                        } else {
                            format!("{filename}{dirty_suffix}")
                        };
                        let tooltip_path = relative_full_path(&doc.path, ws_root.as_deref());

                        let resp = ui
                            .push_id(format!("tab_{idx}"), |ui| {
                                ui.set_max_width(if doc.is_pinned {
                                    PINNED_TAB_MAX_WIDTH
                                } else {
                                    MAX_TAB_WIDTH
                                });
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                let inner_resp = ui.selectable_label(is_active, &title);
                                ui.interact(
                                    inner_resp.rect,
                                    inner_resp.id,
                                    egui::Sense::click_and_drag(),
                                )
                            })
                            .inner;

                        tab_rects.push((idx, resp.rect));
                        // Task 3.7: Only scroll when navigated via left/right buttons.
                        if is_active && should_scroll {
                            resp.scroll_to_me(Some(egui::Align::Center));
                        }
                        if resp.drag_stopped() {
                            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                                dragged_source = Some((idx, pos));
                            }
                        }

                        let clicked = resp.clicked();
                        let resp = resp.on_hover_text(&tooltip_path);

                        resp.context_menu(|ui| {
                            let i18n = crate::i18n::get();

                            if ui.button(&i18n.tab.close).clicked() {
                                tab_action = Some(AppAction::CloseDocument(idx));
                                ui.close();
                            }
                            if ui.button(&i18n.tab.close_others).clicked() {
                                tab_action = Some(AppAction::CloseOtherDocuments(idx));
                                ui.close();
                            }
                            if ui.button(&i18n.tab.close_all).clicked() {
                                tab_action = Some(AppAction::CloseAllDocuments);
                                ui.close();
                            }
                            if ui.button(&i18n.tab.close_right).clicked() {
                                tab_action = Some(AppAction::CloseDocumentsToRight(idx));
                                ui.close();
                            }
                            if ui.button(&i18n.tab.close_left).clicked() {
                                tab_action = Some(AppAction::CloseDocumentsToLeft(idx));
                                ui.close();
                            }
                            ui.separator();
                            let pin_label = if doc.is_pinned {
                                &i18n.tab.unpin
                            } else {
                                &i18n.tab.pin
                            };
                            if ui.button(pin_label).clicked() {
                                tab_action = Some(AppAction::TogglePinDocument(idx));
                                ui.close();
                            }
                            if !state.recently_closed_tabs.is_empty() {
                                ui.separator();
                                if ui.button(&i18n.tab.restore_closed).clicked() {
                                    tab_action = Some(AppAction::RestoreClosedDocument);
                                    ui.close();
                                }
                            }
                        });

                        if clicked && !is_active {
                            tab_action = Some(AppAction::SelectDocument(doc.path.clone()));
                        }

                        if ui
                            .add(egui::Button::image_and_text(
                                crate::Icon::Close.ui_image(ui, crate::icon::IconSize::Small),
                                invisible_label("x"),
                            ))
                            .clicked()
                        {
                            close_idx = Some(idx);
                        }
                        ui.add_space(TAB_INTER_ITEM_SPACING);
                    }
                });
            });

        if should_scroll {
            ui.memory_mut(|mem| {
                mem.data
                    .remove_temp::<bool>(egui::Id::new("scroll_tab_req"));
            });
        }

        ui.separator();

        let nav_enabled = doc_count > 1;
        let icon_bg = if ui.visuals().dark_mode {
            egui::Color32::TRANSPARENT
        } else {
            egui::Color32::from_gray(LIGHT_MODE_ICON_BG)
        };

        if ui
            .add_enabled(
                nav_enabled,
                egui::Button::image_and_text(
                    crate::Icon::TriangleLeft.ui_image(ui, crate::icon::IconSize::Small),
                    invisible_label("◀"),
                )
                .fill(icon_bg),
            )
            .on_hover_text(crate::i18n::get().tab.nav_prev.clone())
            .clicked()
        {
            if let Some(idx) = state.active_doc_idx {
                let new_idx = crate::shell_logic::prev_tab_index(idx, doc_count);
                tab_action = Some(AppAction::SelectDocument(
                    state.open_documents[new_idx].path.clone(),
                ));
                ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
            }
        }
        if ui
            .add_enabled(
                nav_enabled,
                egui::Button::image_and_text(
                    crate::Icon::TriangleRight.ui_image(ui, crate::icon::IconSize::Small),
                    invisible_label("▶"),
                )
                .fill(icon_bg),
            )
            .on_hover_text(crate::i18n::get().tab.nav_next.clone())
            .clicked()
        {
            if let Some(idx) = state.active_doc_idx {
                let new_idx = crate::shell_logic::next_tab_index(idx, doc_count);
                tab_action = Some(AppAction::SelectDocument(
                    state.open_documents[new_idx].path.clone(),
                ));
                ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
            }
        }
    });

    if let Some((src_idx, drop_pos)) = dragged_source {
        for (target_idx, rect) in &tab_rects {
            if rect.contains(drop_pos) && src_idx != *target_idx {
                tab_action = Some(AppAction::ReorderDocument {
                    from: src_idx,
                    to: *target_idx,
                });
                break;
            }
        }
    }

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

pub(crate) fn render_view_mode_bar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    pending_action: &mut AppAction,
) {
    let mut mode = state.active_view_mode();
    let prev = mode;
    let bar_height = ui.spacing().interact_size.y;
    let available_width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(available_width, bar_height),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            // Render non-intrusive Update Available Badge
            if state.update_available.is_some() && !state.checking_for_updates {
                const COLOR_SUCCESS_G: u8 = 200;
                let badge_str = format!("✨ {}", crate::i18n::get().update.update_available);
                let badge_text = egui::RichText::new(badge_str)
                    .color(egui::Color32::from_rgb(0, COLOR_SUCCESS_G, 100))
                    .strong();

                if ui
                    .add(egui::Button::new(badge_text).sense(egui::Sense::click()))
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    *pending_action = AppAction::CheckForUpdates;
                }
                ui.separator();
            }

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
                let (order_text, order_tip) = match current_order {
                    katana_platform::PaneOrder::EditorFirst => (
                        "📄|👁",
                        crate::i18n::get().split_toggle.preview_first.clone(),
                    ),
                    katana_platform::PaneOrder::PreviewFirst => {
                        ("👁|📄", crate::i18n::get().split_toggle.editor_first.clone())
                    }
                };
                if ui
                    .add(egui::Button::new(order_text).sense(egui::Sense::click()))
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
                    katana_platform::SplitDirection::Horizontal => (
                        crate::icon::Icon::SplitHorizontal,
                        crate::i18n::get().split_toggle.vertical.clone(),
                    ),
                    katana_platform::SplitDirection::Vertical => (
                        crate::icon::Icon::SplitVertical,
                        crate::i18n::get().split_toggle.horizontal.clone(),
                    ),
                };
                let icon_size = crate::icon::IconSize::Medium;
                let resp = ui
                    .add(egui::Button::image(
                        dir_icon.image(icon_size).tint(ui.visuals().text_color()),
                    ))
                    .on_hover_text(dir_tip);

                resp.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        true,
                        "Toggle Split Direction",
                    )
                });

                if resp.clicked() {
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
            ui.horizontal_top(|ui| {
                const LINE_NUMBER_MARGIN: f32 = 40.0;
                const LINE_NUMBER_PAD_RIGHT: f32 = 8.0;
                let left_margin = LINE_NUMBER_MARGIN;
                let (ln_rect, _) =
                    ui.allocate_exact_size(egui::vec2(left_margin, 0.0), egui::Sense::hover());
                let text_output = egui::TextEdit::multiline(&mut buffer)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(EDITOR_INITIAL_VISIBLE_ROWS)
                    .margin(egui::Margin {
                        left: 0,
                        right: LINE_NUMBER_MARGIN as i8,
                        top: 0,
                        bottom: 0,
                    })
                    .frame(false)
                    .show(ui);
                let response = text_output.response;
                let galley = text_output.galley;

                if response.clicked() {
                    if let Some(c) = text_output.cursor_range {
                        let char_idx = c.primary.index;
                        let line = galley
                            .text()
                            .chars()
                            .take(char_idx)
                            .filter(|&ch| ch == '\n')
                            .count();
                        state.scroll_to_line = Some(line);
                    }
                }

                let mut current_cursor_y = None;
                if let Some(c) = text_output.cursor_range {
                    let char_idx = c.primary.index;
                    let paragraph = galley
                        .text()
                        .chars()
                        .take(char_idx)
                        .filter(|&ch| ch == '\n')
                        .count();
                    state.active_editor_line = Some(paragraph);

                    let cursor_rect = galley.pos_from_cursor(c.primary);
                    current_cursor_y = Some(cursor_rect.min.y);

                    let min_y = cursor_rect.min.y;
                    let max_y = cursor_rect.max.y;

                    let highlight_rect = egui::Rect::from_min_max(
                        egui::pos2(ln_rect.min.x, response.rect.min.y + min_y),
                        egui::pos2(response.rect.max.x, response.rect.min.y + max_y),
                    );

                    const HIGHLIGHT_ALPHA: u8 = 15;
                    let highlight_color = if ui.visuals().dark_mode {
                        egui::Color32::from_white_alpha(HIGHLIGHT_ALPHA)
                    } else {
                        egui::Color32::from_black_alpha(HIGHLIGHT_ALPHA)
                    };
                    ui.painter()
                        .rect_filled(highlight_rect, 1.0, highlight_color);
                } else {
                    state.active_editor_line = None;
                }

                // Hover highlights from preview pane
                const HOVER_HIGHLIGHT_ALPHA: u8 = 10;
                let hover_color = if ui.visuals().dark_mode {
                    egui::Color32::from_white_alpha(HOVER_HIGHLIGHT_ALPHA)
                } else {
                    egui::Color32::from_black_alpha(HOVER_HIGHLIGHT_ALPHA)
                };

                for line_range in &state.hovered_preview_lines {
                    let mut current_line = 0;
                    let mut start_char = None;
                    let mut end_char = None;

                    for (char_idx, c) in buffer.chars().enumerate() {
                        if current_line == line_range.start && start_char.is_none() {
                            start_char = Some(char_idx);
                        }
                        if current_line == line_range.end + 1 {
                            end_char = Some(char_idx.saturating_sub(1));
                            break;
                        }
                        if c == '\n' {
                            current_line += 1;
                        }
                    }
                    if start_char.is_some() && end_char.is_none() {
                        end_char = Some(buffer.chars().count().saturating_sub(1));
                    }

                    if let (Some(start_idx), Some(end_idx)) = (start_char, end_char) {
                        let cursor_start = egui::text::CCursor {
                            index: start_idx,
                            prefer_next_row: false,
                        };
                        // Ensure we don't highlight beyond the actual characters
                        let cursor_end = egui::text::CCursor {
                            index: end_idx.saturating_sub(1),
                            prefer_next_row: false,
                        };

                        let pos_start = galley.pos_from_cursor(cursor_start);
                        let pos_end = galley.pos_from_cursor(cursor_end);

                        let highlight_rect = egui::Rect::from_min_max(
                            egui::pos2(ln_rect.min.x, response.rect.min.y + pos_start.min.y),
                            egui::pos2(response.rect.max.x, response.rect.min.y + pos_end.max.y),
                        );
                        ui.painter().rect_filled(highlight_rect, 1.0, hover_color);
                    }
                }

                // Draw line numbers
                let clip_rect = ui.clip_rect().expand(100.0);
                let mut p = 0;
                let mut is_start_of_para = true;

                for row in &galley.rows {
                    let top_y = row.rect().min.y;
                    let y = response.rect.min.y + top_y;
                    let is_visible = is_start_of_para
                        && y <= clip_rect.max.y
                        && (y + row.rect().height()) >= clip_rect.min.y;

                    if is_visible {
                        let is_current = current_cursor_y == Some(top_y);
                        let text = format!("{}", p + 1);
                        let color = if is_current {
                            ui.visuals().text_color()
                        } else {
                            ui.visuals().weak_text_color()
                        };
                        let font_id = egui::TextStyle::Monospace.resolve(ui.style());

                        let label_rect = egui::Rect::from_min_size(
                            egui::pos2(ln_rect.min.x, y),
                            egui::vec2(left_margin - LINE_NUMBER_PAD_RIGHT, row.rect().height()),
                        );
                        let mut text_rt = egui::RichText::new(text).color(color).font(font_id);
                        if is_current {
                            text_rt = text_rt.strong();
                        }

                        let label_for_measuring =
                            egui::Label::new(text_rt.clone()).selectable(false);
                        // align right
                        let galley_ln = label_for_measuring.layout_in_ui(ui);
                        let offset_x = (label_rect.width() - galley_ln.1.rect.width()).max(0.0);
                        let tight_rect = egui::Rect::from_min_size(
                            label_rect.min + egui::vec2(offset_x, 0.0),
                            galley_ln.1.rect.size(),
                        );

                        let resp = ui.interact(label_rect, ui.id().with(p), egui::Sense::click());
                        if resp.clicked() {
                            state.scroll_to_line = Some(p);
                        }
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        ui.put(tight_rect, egui::Label::new(text_rt).selectable(false));
                    }

                    is_start_of_para = row.ends_with_newline;
                    if row.ends_with_newline {
                        p += 1;
                    }
                }

                if response.changed() {
                    *action = AppAction::UpdateBuffer(buffer.clone());
                }

                if let Some(target_line) = state.scroll_to_line.take() {
                    let mut current_line = 0;
                    let mut target_char = None;
                    for (char_idx, c) in buffer.chars().enumerate() {
                        if current_line == target_line && target_char.is_none() {
                            target_char = Some(char_idx);
                            break;
                        }
                        if c == '\n' {
                            current_line += 1;
                        }
                    }
                    if let Some(idx) = target_char {
                        let cursor = egui::text::CCursor {
                            index: idx,
                            prefer_next_row: false,
                        };
                        let pos = galley.pos_from_cursor(cursor);
                        let rect = egui::Rect::from_min_max(
                            egui::pos2(response.rect.min.x, response.rect.min.y + pos.min.y),
                            egui::pos2(response.rect.max.x, response.rect.min.y + pos.max.y),
                        );
                        ui.scroll_to_rect(rect, Some(egui::Align::Center));
                    }
                }

                response
            })
            .inner
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
    let file_tree_color = ui.visuals().text_color();
    let (rect, mut resp) = ui.allocate_at_least(
        egui::vec2(ui.available_width(), TREE_ROW_HEIGHT),
        egui::Sense::click(),
    );
    resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

    let accessible_label = format!("dir {}", name);
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &accessible_label)
    });

    if resp.clicked() {
        if is_open {
            ctx.expanded_directories.remove(path);
        } else {
            ctx.expanded_directories.insert(path.to_path_buf());
        }
    }

    if ui.is_rect_visible(rect) {
        if ui.rect_contains_pointer(rect) && ui.is_enabled() {
            ui.painter()
                .rect_filled(rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
        }

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.add_space(TREE_LABEL_HOFFSET);
        let prefix = indent_prefix(ctx.depth);
        let arrow_icon = if is_open {
            crate::icon::Icon::PanDown
        } else {
            crate::icon::Icon::PanRight
        };
        let folder_icon = if is_open {
            crate::icon::Icon::FolderOpen
        } else {
            crate::icon::Icon::FolderClosed
        };

        child_ui.add(egui::Label::new(prefix).selectable(false));

        // Add spacing equivalent to item_spacing.x = 2.0 manually or rely on default layout
        child_ui.add(
            arrow_icon
                .image(crate::icon::IconSize::Small)
                .tint(file_tree_color),
        );
        child_ui.add(
            folder_icon
                .image(crate::icon::IconSize::Medium)
                .tint(file_tree_color),
        );
        child_ui.add(
            egui::Label::new(egui::RichText::new(name).color(file_tree_color))
                .selectable(false)
                .truncate(),
        );
        // We do not union because resp already covers the whole area and the
        // children don't have interactive senses that would steal hover/clicks.
    }

    // Context Menu for directories
    resp.context_menu(|ui| {
        if ui
            .button(crate::i18n::get().action.recursive_expand.clone())
            .clicked()
        {
            let mut to_expand = Vec::new();
            for child in children {
                child.collect_all_directory_paths(&mut to_expand);
            }
            ctx.expanded_directories.insert(path.to_path_buf()); // Expand self too
            ctx.expanded_directories.extend(to_expand);
            ui.close();
        }
        if ui
            .button(crate::i18n::get().action.recursive_open_all.clone())
            .clicked()
        {
            let mut to_open = Vec::new();
            for child in children {
                child.collect_all_markdown_file_paths(&mut to_open);
            }
            if !to_open.is_empty() {
                *ctx.action = crate::app_state::AppAction::OpenMultipleDocuments(to_open);
            }
            ui.close();
        }

        ui.separator();

        // Meta info moved to context menu click action
        if ui
            .button(crate::i18n::get().action.show_meta_info.clone())
            .clicked()
        {
            *ctx.action = crate::app_state::AppAction::ShowMetaInfo(path.to_path_buf());
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
    is_negated: bool,
    ws_root: &std::path::Path,
    visible: &mut std::collections::HashSet<std::path::PathBuf>,
) -> bool {
    let mut any_visible = false;
    for entry in entries {
        match entry {
            katana_core::workspace::TreeEntry::File { path } => {
                let rel = crate::shell_logic::relative_full_path(path, Some(ws_root));
                let is_match = regex.is_match(&rel);
                let should_show = if is_negated { !is_match } else { is_match };

                if should_show {
                    visible.insert(path.clone());
                    any_visible = true;
                }
            }
            katana_core::workspace::TreeEntry::Directory { path, children } => {
                if gather_visible_paths(children, regex, is_negated, ws_root, visible) {
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
    // Prepend two spaces to align with the directory's arrow and its following space ("▶ ").

    // Accessibility label without leading indentation (used for widget_info / test queries).
    let accessible_label = format!("file {}", name);

    let is_active = ctx.active_path.is_some_and(|ap| ap == path);

    let text_color = if is_active {
        ui.visuals().widgets.active.fg_stroke.color
    } else {
        ui.visuals().text_color()
    };
    let (full_rect, mut resp) = ui.allocate_at_least(
        egui::vec2(ui.available_width(), TREE_ROW_HEIGHT),
        egui::Sense::click(),
    );
    resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

    if ui.is_rect_visible(full_rect) {
        if is_active {
            let highlight_color = ui.visuals().selection.bg_fill;
            ui.painter()
                .rect_filled(full_rect, ACTIVE_FILE_HIGHLIGHT_ROUNDING, highlight_color);
        } else if ui.rect_contains_pointer(full_rect) && ui.is_enabled() {
            ui.painter()
                .rect_filled(full_rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
        }

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(full_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.add_space(TREE_LABEL_HOFFSET);

        let prefix_string = indent_prefix(ctx.depth);
        child_ui.add(
            egui::Label::new(egui::RichText::new(prefix_string).color(text_color))
                .selectable(false),
        );

        // Allocate empty space exactly matching the directory arrow icon's size
        child_ui.allocate_response(
            egui::vec2(crate::icon::IconSize::Small.to_vec2().x, 0.0),
            egui::Sense::hover(),
        );

        if entry.is_markdown() {
            child_ui.add(
                crate::icon::Icon::Document
                    .image(crate::icon::IconSize::Medium)
                    .tint(text_color),
            );
        } else {
            // For non-markdown, we might want a raw file icon, but for now we fallback to invisible space to match old behavior
            child_ui.allocate_response(
                egui::vec2(crate::icon::IconSize::Medium.to_vec2().x, 0.0),
                egui::Sense::hover(),
            );
        };

        let mut rich = egui::RichText::new(name).color(text_color);
        if is_active {
            rich = rich.strong();
        }
        let resp_label = child_ui.add(egui::Label::new(rich).truncate().selectable(false));

        resp_label.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &accessible_label)
        });
    }

    resp.context_menu(|ui| {
        if ui
            .button(crate::i18n::get().action.show_meta_info.clone())
            .clicked()
        {
            *ctx.action = crate::app_state::AppAction::ShowMetaInfo(path.to_path_buf());
            ui.close();
        }
    });

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
                    &mut app.state,
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

    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
        .show(ctx, |ui| {
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
                        &mut app.state,
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

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
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
                        &mut app.state,
                        &mut app.pending_action,
                        true,
                        &mut scroll_state,
                    );
                }
            });

        app.state.scroll_fraction = scroll_state.0;
        app.state.scroll_source = scroll_state.1;
        app.state.preview_max_scroll = scroll_state.2;

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
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
            &mut app.state,
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
    pub const TAG_CHECK_UPDATES: i32 = 15;

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

fn invalidate_preview_image_cache(ctx: &egui::Context, action: &AppAction) {
    if matches!(action, AppAction::RefreshDiagrams) {
        crate::icon::IconRegistry::install(ctx);
    }
}

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

                                            egui::ComboBox::from_id_salt("terms_lang_select")
                                                .selected_text(current_name)
                                                .width(Self::TERMS_LANG_SELECT_WIDTH)
                                                .show_ui(ui, |ui| {
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

        // Pre-calculate splash state to prevent flickering of the background UI.
        // If the splash is fully opaque (the first 1.5s), we skip drawing the panels.
        let splash_opacity = self
            .splash_start
            .map(|s| crate::shell_logic::calculate_splash_opacity(s.elapsed().as_secs_f32()))
            .unwrap_or(0.0);
        let splash_is_opaque = self.splash_start.is_some() && splash_opacity >= 1.0;

        // Apply theme colours to egui Visuals (only when the palette changed)
        let theme_colors = self.state.settings.settings().effective_theme_colors();
        if self.cached_theme.as_ref() != Some(&theme_colors) {
            let dark = theme_colors.mode == katana_platform::theme::ThemeMode::Dark;
            ctx.set_visuals(theme_bridge::visuals_from_theme(&theme_colors));
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
                egui::Modifiers {
                    command: true,
                    shift: true,
                    ..Default::default()
                },
                egui::Key::T,
            ))
        }) && !self.state.recently_closed_tabs.is_empty()
        {
            self.pending_action = AppAction::RestoreClosedDocument;
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

        // Process pending document loads (1 per frame to avoid UI freeze)
        if let Some(path) = self.pending_document_loads.pop_front() {
            self.handle_select_document(path, false);
            ctx.request_repaint();
        }

        self.poll_update_install(ctx);
        self.poll_update_check(ctx);
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
                native_menu::TAG_SETTINGS => {
                    self.pending_action = AppAction::ToggleSettings;
                }
                _ => {}
            }
        }

        let action = self.take_action();
        invalidate_preview_image_cache(ctx, &action);
        self.process_action(ctx, action);

        if !splash_is_opaque {
            // Terms of Service check (Blocking UI)
            let terms_ver = crate::about_info::APP_VERSION.to_string();
            let accepted_ver = self
                .state
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
            #[cfg(not(target_os = "macos"))]
            render_menu_bar(ctx, &mut self.state, &mut self.pending_action);
            let export_filenames: Vec<String> = self
                .export_tasks
                .iter()
                .map(|t| t.filename.clone())
                .collect();
            render_status_bar(ctx, &self.state, &export_filenames);

            // Reflect the file name in the window title
            let ws_root_for_title = self.state.workspace.as_ref().map(|ws| ws.root.clone());
            let title_text = match self.state.active_document() {
                Some(doc) => {
                    let rel = relative_full_path(&doc.path, ws_root_for_title.as_deref());
                    format!("KatanA — {rel}")
                }
                None => "KatanA".to_string(),
            };
            if self.state.last_window_title != title_text {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));
                self.state.last_window_title = title_text.clone();
            }

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
                                .add(egui::Button::image(
                                    crate::Icon::ChevronRight
                                        .ui_image(ui, crate::icon::IconSize::Medium),
                                ))
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
                                const CHEVRON_ICON_SIZE: f32 = 10.0;
                                ui.add(
                                    egui::Image::new(crate::Icon::ChevronRight.uri())
                                        .tint(ui.visuals().text_color())
                                        .max_height(CHEVRON_ICON_SIZE),
                                );
                            }
                            ui.label(egui::RichText::new(*seg).small());
                        }
                    });
                    render_view_mode_bar(ui, &mut self.state, &mut self.pending_action);
                }
            });

            let mut download_req: Option<DownloadRequest> = None;
            let current_mode = self.state.active_view_mode();
            let is_split = current_mode == ViewMode::Split;

            if self.state.show_toc && self.state.settings.settings().layout.toc_visible {
                if let Some(doc) = self.state.active_document() {
                    if let Some(preview) = self.tab_previews.iter_mut().find(|p| p.path == doc.path)
                    {
                        render_toc_panel(ctx, &mut preview.pane, &self.state);
                    }
                }
            }

            if is_split {
                let split_dir = self.state.active_split_direction();
                let pane_order = self.state.active_pane_order();
                download_req = render_split_mode(ctx, self, split_dir, pane_order);
            }

            if !is_split {
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                    .show(ctx, |ui| match current_mode {
                        ViewMode::CodeOnly => {
                            render_editor_content(
                                ui,
                                &mut self.state,
                                &mut self.pending_action,
                                false,
                            );
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
        }

        // Settings window
        if let Some(settings_action) = crate::settings_window::render_settings_window(
            ctx,
            &mut self.state,
            &mut self.settings_preview,
        ) {
            self.pending_action = settings_action;
        }

        if self.state.show_search_modal {
            render_search_modal(ctx, &mut self.state, &mut self.pending_action);
        }

        // About dialog
        if self.show_about {
            render_about_window(ctx, &mut self.show_about, self.about_icon.as_ref());
        }

        // Meta info dialog
        if let Some(path) = self.show_meta_info_for.clone() {
            let mut is_open = true;
            render_meta_info_window(ctx, &mut is_open, &path);
            if !is_open {
                self.show_meta_info_for = None;
            }
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
                            egui::Color32::from_rgb(SPLASH_BG_DARK, SPLASH_BG_DARK, SPLASH_BG_DARK)
                        } else {
                            egui::Color32::from_rgb(
                                SPLASH_BG_LIGHT,
                                SPLASH_BG_LIGHT,
                                SPLASH_BG_LIGHT,
                            )
                        };
                        let fill_color = bg_color.gamma_multiply(opacity);
                        ui.painter().rect_filled(content_rect, 1.0, fill_color);

                        let text_color = if is_dark {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
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
                                    ui.visuals_mut().selection.bg_fill = egui::Color32::from_rgb(
                                        SPLASH_PROGRESS_BG_LIGHT,
                                        SPLASH_PROGRESS_BG_LIGHT,
                                        SPLASH_PROGRESS_BG_LIGHT,
                                    )
                                    .gamma_multiply(opacity);
                                } else {
                                    ui.visuals_mut().selection.bg_fill = egui::Color32::from_rgb(
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
    use eframe::egui::{self, pos2, Rect};
    use eframe::App as _;
    use egui::load::{BytesLoadResult, BytesLoader, LoadError};
    use katana_core::{document::Document, workspace::TreeEntry};
    use std::path::{Path, PathBuf};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    const PREVIEW_CONTENT_PADDING: f32 = 12.0;

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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
        let ctx = test_context();
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
            about_link_row(ui, &i18n_about.source_code, info.repository);
            about_link_row(ui, &i18n_about.documentation, info.docs_url);
            about_link_row(ui, &i18n_about.report_issue, info.issues_url);
            ui.add_space(SECTION_SPACING);

            // ── 7. Support / Sponsor ──
            about_section_header(
                ui,
                &i18n_about.support,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            if info.sponsor_url.is_empty() {
                about_row(ui, &i18n_about.sponsor, &i18n_about.coming_soon);
            } else {
                about_link_row(ui, &i18n_about.sponsor, info.sponsor_url);
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
                    crate::shell_logic::collect_matches(
                        &ws.tree,
                        &query,
                        &include_regexes,
                        &exclude_regexes,
                        &ws.root,
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

const TOC_PANEL_DEFAULT_WIDTH: f32 = 200.0;
const TOC_PANEL_MARGIN: f32 = 8.0;
const TOC_HEADING_VISIBILITY_THRESHOLD: f32 = 40.0;
const TOC_INDENT_PER_LEVEL: f32 = 12.0;

const LIGHT_MODE_ICON_BG: u8 = 235;
const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 200;

pub(crate) fn render_toc_panel(
    ctx: &egui::Context,
    preview: &mut crate::preview_pane::PreviewPane,
    state: &crate::app_state::AppState,
) {
    use katana_platform::settings::TocPosition;
    let position = state.settings.settings().layout.toc_position;

    let panel = match position {
        TocPosition::Left => egui::SidePanel::left("toc_panel"),
        TocPosition::Right => egui::SidePanel::right("toc_panel"),
    };

    let frame = egui::Frame::side_top_panel(&ctx.style()).inner_margin(TOC_PANEL_MARGIN);

    panel
        .frame(frame)
        .resizable(true)
        .default_width(TOC_PANEL_DEFAULT_WIDTH)
        .show(ctx, |ui| {
            ui.heading(crate::i18n::get().toc.title.clone());
            ui.separator();

            // Prevent text from wrapping or pushing the SidePanel width. Text will truncate with `...`
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    if preview.outline_items.is_empty() {
                        ui.label(
                            egui::RichText::new(crate::i18n::get().toc.empty.clone())
                                .weak()
                                .italics(),
                        );
                    } else {
                        let mut active_index = 0;
                        if let Some(visible_rect) = preview.visible_rect {
                            let threshold = visible_rect.min.y + TOC_HEADING_VISIBILITY_THRESHOLD;
                            for (i, (_, rect)) in preview.heading_anchors.iter().enumerate() {
                                if rect.min.y <= threshold {
                                    active_index = i;
                                } else {
                                    break;
                                }
                            }
                        }

                        let mut next_scroll = None;
                        for (i, item) in preview.outline_items.iter().enumerate() {
                            let indent = (item.level as f32 - 1.0) * TOC_INDENT_PER_LEVEL;
                            ui.horizontal(|ui| {
                                ui.add_space(indent);
                                let is_active = i == active_index;
                                let mut text = egui::RichText::new(&item.text);
                                if is_active {
                                    text = text
                                        .strong()
                                        .color(ui.visuals().widgets.active.text_color());
                                }
                                if ui.selectable_label(is_active, text).clicked() {
                                    next_scroll = Some(item.index);
                                }
                            });
                        }
                        if next_scroll.is_some() {
                            preview.scroll_request = next_scroll;
                        }
                    }
                });
        });
}

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

    let msgs = &crate::i18n::get().update;

    // Phase-aware modals (Downloading / Installing / ReadyToRelaunch)
    match &state.update_phase {
        Some(UpdatePhase::Downloading) => {
            Modal::new("katana_update_progress", &msgs.title).show_body_only(ctx, |ui| {
                ui.add(egui::Spinner::new());
                ui.add_space(SPACING_MEDIUM);
                ui.label(&msgs.downloading);
            });
            return;
        }
        Some(UpdatePhase::Installing) => {
            Modal::new("katana_update_progress", &msgs.title).show_body_only(ctx, |ui| {
                ui.add(egui::Spinner::new());
                ui.add_space(SPACING_MEDIUM);
                ui.label(&msgs.installing);
            });
            return;
        }
        Some(UpdatePhase::ReadyToRelaunch) => {
            let action = Modal::new("katana_update_relaunch", &msgs.title).show(
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
    if state.checking_for_updates {
        // Checking spinner — no footer, no close button
        Modal::new("katana_update_dialog_v6", &msgs.title).show_body_only(ctx, |ui| {
            ui.add(egui::Spinner::new());
            ui.add_space(SPACING_MEDIUM);
            ui.label(msgs.checking_for_updates.clone());
        });
    } else if let Some(err) = &state.update_check_error {
        // Error state — OK button to close
        let close = {
            let err = err.clone();
            Modal::new("katana_update_dialog_v6", &msgs.title).show(
                ctx,
                |ui| {
                    ui.colored_label(egui::Color32::RED, msgs.failed_to_check.clone());
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
    } else if let Some(latest) = &state.update_available {
        // Update available — Install/Skip/Later buttons
        let tag = latest.tag_name.clone();
        let body_text = latest.body.clone();
        let desc = msgs
            .update_available_desc
            .replace("{version}", tag.as_str());
        let action = Modal::new("katana_update_dialog_v6", &msgs.title).show(
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
            *open = false;
        }
    } else {
        // Up to date — OK button to close
        let close = Modal::new("katana_update_dialog_v6", &msgs.title).show(
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
