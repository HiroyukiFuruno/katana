#![allow(unused_imports, dead_code)]
use crate::app_state::{AppAction, AppState, ScrollSource, ViewMode};
use crate::i18n;
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, EDITOR_INITIAL_VISIBLE_ROWS, FILE_TREE_PANEL_DEFAULT_WIDTH,
    FILE_TREE_PANEL_MIN_WIDTH, NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_ITEM_SPACING,
    RECENT_WORKSPACES_SPACING, SCROLL_SYNC_DEAD_ZONE, TAB_DROP_ANIMATION_TIME,
    TAB_DROP_INDICATOR_WIDTH, TAB_INTER_ITEM_SPACING, TAB_NAV_BUTTONS_AREA_WIDTH,
    TAB_TOOLTIP_SHOW_DELAY_SECS, TREE_LABEL_HOFFSET, TREE_ROW_HEIGHT,
};
use crate::shell_ui::{
    indent_prefix, invisible_label, open_folder_dialog, relative_full_path, TreeRenderContext,
    LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG, PREVIEW_CONTENT_PADDING, SEARCH_MODAL_HEIGHT,
    SEARCH_MODAL_WIDTH, STATUS_BAR_ICON_SPACING, STATUS_SUCCESS_GREEN,
    TOC_HEADING_VISIBILITY_THRESHOLD, TOC_INDENT_PER_LEVEL, TOC_PANEL_DEFAULT_WIDTH,
    TOC_PANEL_MARGIN, WORKSPACE_SPINNER_INNER_MARGIN, WORKSPACE_SPINNER_OUTER_MARGIN,
    WORKSPACE_SPINNER_TEXT_MARGIN,
};
use crate::theme_bridge;
use crate::Icon;
use eframe::egui;
use katana_core::workspace::{TreeEntry, Workspace};
use std::path::{Path, PathBuf};

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
                        state.layout.show_workspace = false;
                    }
                });
            });
            if state.workspace.data.is_some() {
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
                        if let Some(ws) = &state.workspace.data {
                            state
                                .workspace
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
                        state.workspace.force_tree_open = Some(false);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let icon_bg = if ui.visuals().dark_mode {
                            crate::theme_bridge::TRANSPARENT
                        } else {
                            crate::theme_bridge::from_gray(LIGHT_MODE_ICON_BG) // Always gray for icons in light mode
                        };

                        if !state.config.settings.settings().workspace.paths.is_empty() {
                            let ws_history_img =
                                crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Small);
                            ui.scope(|ui| {
                                ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;
                                ui.menu_image_button(ws_history_img, |ui| {
                                    for path in state
                                        .config
                                        .settings
                                        .settings()
                                        .workspace
                                        .paths
                                        .iter()
                                        .rev()
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

                        let filter_btn_color = if state.search.filter_enabled {
                            if ui.visuals().dark_mode {
                                ui.visuals().selection.bg_fill
                            } else {
                                crate::theme_bridge::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
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
                            state.search.filter_enabled = !state.search.filter_enabled;
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
                            state.layout.show_search_modal = true;
                        }
                    });
                });

                if state.search.filter_enabled {
                    let mut is_valid_regex = true;
                    if !state.search.filter_query.is_empty() {
                        is_valid_regex = regex::Regex::new(&state.search.filter_query).is_ok();
                    }
                    ui.horizontal(|ui| {
                        let text_color = if is_valid_regex {
                            ui.visuals().text_color()
                        } else {
                            ui.ctx()
                                .data(|d| {
                                    d.get_temp::<katana_platform::theme::ThemeColors>(
                                        egui::Id::new("katana_theme_colors"),
                                    )
                                })
                                .map_or(crate::theme_bridge::WHITE, |tc| {
                                    crate::theme_bridge::rgb_to_color32(tc.system.error_text)
                                })
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut state.search.filter_query)
                                .text_color(text_color)
                                .hint_text("Filter (Regex)...")
                                .desired_width(f32::INFINITY),
                        );
                    });
                }
            }
            ui.separator();
            if state.workspace.is_loading {
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
    if let Some(ws) = &state.workspace.data {
        let entries = ws.tree.clone();
        if let Some(force) = state.workspace.force_tree_open {
            if force {
                state
                    .workspace
                    .expanded_directories
                    .extend(ws.collect_all_directory_paths());
            } else {
                state.workspace.expanded_directories.clear();
            }
        }
        let active_path = state.active_path().map(|p| p.to_path_buf());

        let ws_root = ws.root.clone();
        if state.search.filter_enabled && !state.search.filter_query.is_empty() {
            let is_negated = state.search.filter_query.starts_with('!');
            let query_str = if is_negated {
                &state.search.filter_query[1..]
            } else {
                &state.search.filter_query
            };

            if let Ok(regex) = regex::Regex::new(query_str) {
                if state.search.filter_cache.as_ref().map(|(q, _)| q)
                    != Some(&state.search.filter_query)
                {
                    let mut visible = std::collections::HashSet::new();
                    crate::views::panels::tree::gather_visible_paths(
                        &entries,
                        &regex,
                        is_negated,
                        &ws_root,
                        &mut visible,
                    );
                    state.search.filter_cache = Some((state.search.filter_query.clone(), visible));
                }
            } else {
                state.search.filter_cache = None;
            }
        } else {
            state.search.filter_cache = None;
        }
        let filter_set = state.search.filter_cache.as_ref().map(|(_, v)| v);

        egui::ScrollArea::vertical()
            .id_salt("workspace_tree_scroll")
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                let mut ctx = TreeRenderContext {
                    action,
                    depth: 0,
                    active_path: active_path.as_deref(),
                    filter_set,
                    expanded_directories: &mut state.workspace.expanded_directories,
                    disable_context_menu: false,
                };
                for entry in &entries {
                    render_tree_entry(ui, entry, &mut ctx);
                }
            });
        state.workspace.force_tree_open = None;
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

        let recent_paths = &state.config.settings.settings().workspace.paths;
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
    if !ctx.disable_context_menu {
        resp.context_menu(|ui| {
            crate::views::panels::tree::render_tree_context_menu(
                ui,
                path,
                true,
                Some(children),
                None,
                ctx,
            );
        });
    }

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

    if !ctx.disable_context_menu {
        resp.context_menu(|ui| {
            crate::views::panels::tree::render_tree_context_menu(
                ui,
                path,
                false,
                None,
                Some(entry),
                ctx,
            );
        });
    }

    if resp.clicked() {
        *ctx.action = crate::app_state::AppAction::SelectDocument(path.to_path_buf());
    }
}

pub(crate) fn render_breadcrumb_menu(
    ui: &mut egui::Ui,
    entries: &[katana_core::workspace::TreeEntry],
    action: &mut crate::app_state::AppAction,
) {
    for entry in entries {
        match entry {
            katana_core::workspace::TreeEntry::Directory { path, children } => {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                ui.menu_button(name, |ui| {
                    render_breadcrumb_menu(ui, children, action);
                });
            }
            katana_core::workspace::TreeEntry::File { path } => {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                if ui.button(name).clicked() {
                    *action = crate::app_state::AppAction::SelectDocument(path.clone());
                    ui.close();
                }
            }
        }
    }
}
