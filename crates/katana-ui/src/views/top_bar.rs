use crate::app_state::{AppAction, AppState, ViewMode};
use crate::shell::{
    TAB_DROP_ANIMATION_TIME, TAB_DROP_INDICATOR_WIDTH, TAB_INTER_ITEM_SPACING,
    TAB_NAV_BUTTONS_AREA_WIDTH, TAB_TOOLTIP_SHOW_DELAY_SECS,
};
use crate::shell_ui::{
    invisible_label, relative_full_path, LIGHT_MODE_ICON_BG, STATUS_BAR_ICON_SPACING,
    STATUS_SUCCESS_GREEN,
};
use eframe::egui;

pub(crate) struct StatusBar<'a> {
    pub state: &'a AppState,
    pub export_filenames: &'a [String],
}

impl<'a> StatusBar<'a> {
    pub fn new(state: &'a AppState, export_filenames: &'a [String]) -> Self {
        Self {
            state,
            export_filenames,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        let state = self.state;
        let export_filenames = self.export_filenames;
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (msg, kind) = if let Some((msg, kind)) = &state.layout.status_message {
                    (msg.as_str(), Some(kind))
                } else {
                    (crate::i18n::get().status.ready.as_str(), None)
                };

                let (color, icon) = match kind {
                    Some(crate::app_state::StatusType::Error) => (
                        ui.ctx()
                            .data(|d| {
                                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                    "katana_theme_colors",
                                ))
                            })
                            .map_or(crate::theme_bridge::WHITE, |tc| {
                                crate::theme_bridge::rgb_to_color32(tc.system.error_text)
                            }),
                        Some(crate::Icon::Error),
                    ),
                    Some(crate::app_state::StatusType::Warning) => {
                        (ui.visuals().warn_fg_color, Some(crate::Icon::Warning))
                    }
                    Some(crate::app_state::StatusType::Success) => (
                        crate::theme_bridge::from_rgb(0, STATUS_SUCCESS_GREEN, 0),
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
}

pub(crate) struct TabBar<'a> {
    pub state: &'a mut AppState,
    pub action: &'a mut AppAction,
}

impl<'a> TabBar<'a> {
    pub fn new(state: &'a mut AppState, action: &'a mut AppAction) -> Self {
        Self { state, action }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let state = self.state;
        let action = self.action;
        const MAX_TAB_WIDTH: f32 = 200.0;
        const PINNED_TAB_MAX_WIDTH: f32 = 60.0;

        let mut close_idx: Option<usize> = None;
        let mut tab_action: Option<AppAction> = None;
        let mut dragged_source: Option<(usize, f32)> = None;
        let mut tab_rects: Vec<(usize, egui::Rect)> = Vec::new();

        let ws_root = state.workspace.data.as_ref().map(|ws| ws.root.clone());
        let doc_count = state.document.open_documents.len();

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
                    let mut current_hovered_drop_x = None;
                    let mut dragging_ghost_info = None;

                    ui.horizontal(|ui| {
                        for (idx, doc) in state.document.open_documents.iter().enumerate() {
                            let is_active = state.document.active_doc_idx == Some(idx);
                            let original_filename = doc.file_name().unwrap_or("untitled");
                            let is_changelog =
                                doc.path.to_string_lossy().starts_with("Katana://ChangeLog");

                            let filename = if is_changelog {
                                original_filename.to_string()
                            } else if original_filename.starts_with("CHANGELOG_v")
                                && original_filename.ends_with(".md")
                            {
                                let ver = original_filename
                                    .trim_start_matches("CHANGELOG_v")
                                    .trim_end_matches(".md");
                                format!("📄 {} {}", crate::i18n::get().menu.release_notes, ver)
                            } else {
                                original_filename.to_string()
                            };
                            let dirty_suffix = if doc.is_dirty { " *" } else { "" };
                            let title = if doc.is_pinned {
                                format!("📌 {filename}{dirty_suffix}")
                            } else {
                                format!("{filename}{dirty_suffix}")
                            };
                            let tooltip_path = relative_full_path(&doc.path, ws_root.as_deref());

                            let (title_resp, close_resp) = ui
                                .push_id(format!("tab_{idx}"), |ui| {
                                    ui.set_max_width(if doc.is_pinned {
                                        PINNED_TAB_MAX_WIDTH
                                    } else {
                                        MAX_TAB_WIDTH
                                    });
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                                    let t_resp = if is_changelog {
                                        ui.add(
                                            egui::Button::image_and_text(
                                                crate::Icon::Info
                                                    .ui_image(ui, crate::icon::IconSize::Small),
                                                &title,
                                            )
                                            .selected(is_active),
                                        )
                                    } else {
                                        ui.add(egui::Button::selectable(is_active, &title))
                                    };

                                    let c_resp = ui.add(egui::Button::image_and_text(
                                        crate::Icon::Close
                                            .ui_image(ui, crate::icon::IconSize::Small),
                                        invisible_label("x"),
                                    ));
                                    (t_resp, c_resp)
                                })
                                .inner;

                            let full_tab_rect = title_resp.rect.union(close_resp.rect);
                            tab_rects.push((idx, full_tab_rect));

                            let tab_interact = ui.interact(
                                title_resp.rect,
                                egui::Id::new("tab_interact").with(idx),
                                egui::Sense::click_and_drag(),
                            );

                            let mut clicked_tab = tab_interact.clicked();
                            if close_resp.clicked() {
                                close_idx = Some(idx);
                                clicked_tab = false;
                            }

                            let is_being_dragged = ui.ctx().is_being_dragged(tab_interact.id);
                            if is_being_dragged {
                                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let press_origin = ui
                                        .input(|i| i.pointer.press_origin())
                                        .unwrap_or(pointer_pos);
                                    let drag_offset = pointer_pos - press_origin;
                                    let ghost_rect = full_tab_rect.translate(drag_offset);

                                    // Save the ghost center X for the exact moment of drop!
                                    ui.memory_mut(|mem| {
                                        mem.data.insert_temp(
                                            egui::Id::new("drag_ghost_x").with(idx),
                                            ghost_rect.center().x,
                                        )
                                    });

                                    // Auto-scroll the horizontal scroll area to ensure the dragged tab is visible
                                    ui.scroll_to_rect(ghost_rect, None);

                                    // 1. Render ghost tab following the cursor
                                    egui::Area::new(egui::Id::new("tab_ghost").with(idx))
                                        .fixed_pos(ghost_rect.min)
                                        .order(egui::Order::Tooltip)
                                        .show(ui.ctx(), |ui| {
                                            ui.set_max_width(if doc.is_pinned {
                                                PINNED_TAB_MAX_WIDTH
                                            } else {
                                                MAX_TAB_WIDTH
                                            });
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Truncate);

                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                if is_changelog {
                                                    let btn = egui::Button::image_and_text(
                                                        crate::Icon::Info.ui_image(
                                                            ui,
                                                            crate::icon::IconSize::Medium,
                                                        ),
                                                        &title,
                                                    )
                                                    .selected(is_active);
                                                    ui.add(btn);
                                                } else {
                                                    ui.add(egui::Button::selectable(
                                                        is_active, &title,
                                                    ));
                                                }
                                                ui.add(egui::Button::image_and_text(
                                                    crate::Icon::Close
                                                        .ui_image(ui, crate::icon::IconSize::Small),
                                                    invisible_label("x"),
                                                ));
                                            });
                                        });

                                    dragging_ghost_info =
                                        Some((ghost_rect, full_tab_rect.y_range()));
                                }
                            }

                            // Task 3.7: Only scroll when navigated via left/right buttons.
                            if is_active && should_scroll {
                                tab_interact.scroll_to_me(Some(egui::Align::Center));
                            }

                            // Retrieve the saved ghost_x purely from memory because pointer input might be cleared
                            if tab_interact.drag_stopped() {
                                if let Some(ghost_x) = ui.memory(|mem| {
                                    mem.data
                                        .get_temp::<f32>(egui::Id::new("drag_ghost_x").with(idx))
                                }) {
                                    dragged_source = Some((idx, ghost_x));
                                }
                            }

                            let tab_interact = tab_interact.on_hover_text(&tooltip_path);

                            tab_interact.context_menu(|ui| {
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
                                if !state.document.recently_closed_tabs.is_empty() {
                                    ui.separator();
                                    if ui.button(&i18n.tab.restore_closed).clicked() {
                                        tab_action = Some(AppAction::RestoreClosedDocument);
                                        ui.close();
                                    }
                                }
                            });

                            if clicked_tab && !is_active {
                                tab_action = Some(AppAction::SelectDocument(doc.path.clone()));
                            }

                            ui.add_space(TAB_INTER_ITEM_SPACING);
                        }

                        let mut drop_points = Vec::new();
                        if !tab_rects.is_empty() {
                            for i in 0..tab_rects.len() {
                                if i == 0 {
                                    drop_points.push((0, tab_rects[i].1.left()));
                                } else {
                                    let prev_right = tab_rects[i - 1].1.right();
                                    let current_left = tab_rects[i].1.left();
                                    drop_points.push((i, (prev_right + current_left) / 2.0));
                                }
                            }
                            drop_points
                                .push((tab_rects.len(), tab_rects.last().unwrap().1.right()));
                        }

                        if let Some((ghost_rect, y_range)) = dragging_ghost_info {
                            let mut best_dist = f32::MAX;
                            let mut best_x = None;

                            for (_insert_idx, x) in &drop_points {
                                let dist = (ghost_rect.center().x - x).abs();
                                if dist < best_dist {
                                    best_dist = dist;
                                    best_x = Some(*x);
                                }
                            }
                            if let Some(x) = best_x {
                                current_hovered_drop_x = Some((x, y_range));
                            }
                        }

                        // Draw animated (lerp) insertion marker
                        if let Some((target_x, y_range)) = current_hovered_drop_x {
                            let indicator_id = egui::Id::new("tab_drop_indicator");
                            let animated_x = ui.ctx().animate_value_with_time(
                                indicator_id,
                                target_x,
                                TAB_DROP_ANIMATION_TIME,
                            );

                            let stroke = egui::Stroke::new(
                                TAB_DROP_INDICATOR_WIDTH,
                                ui.visuals().selection.bg_fill,
                            );
                            ui.painter().vline(animated_x, y_range, stroke);
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
                crate::theme_bridge::TRANSPARENT
            } else {
                crate::theme_bridge::from_gray(LIGHT_MODE_ICON_BG)
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
                if let Some(idx) = state.document.active_doc_idx {
                    let new_idx = crate::shell_logic::prev_tab_index(idx, doc_count);
                    tab_action = Some(AppAction::SelectDocument(
                        state.document.open_documents[new_idx].path.clone(),
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
                if let Some(idx) = state.document.active_doc_idx {
                    let new_idx = crate::shell_logic::next_tab_index(idx, doc_count);
                    tab_action = Some(AppAction::SelectDocument(
                        state.document.open_documents[new_idx].path.clone(),
                    ));
                    ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
                }
            }
        });

        if let Some((src_idx, ghost_center_x)) = dragged_source {
            let mut drop_points = Vec::new();
            if !tab_rects.is_empty() {
                for i in 0..tab_rects.len() {
                    if i == 0 {
                        drop_points.push((0, tab_rects[i].1.left()));
                    } else {
                        let prev_right = tab_rects[i - 1].1.right();
                        let current_left = tab_rects[i].1.left();
                        drop_points.push((i, (prev_right + current_left) / 2.0));
                    }
                }
                drop_points.push((tab_rects.len(), tab_rects.last().unwrap().1.right()));
            }

            let mut best_dist = f32::MAX;
            let mut best_insert_idx = None;

            for (insert_idx, x) in drop_points {
                let dist = (ghost_center_x - x).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_insert_idx = Some(insert_idx);
                }
            }

            if let Some(to) = best_insert_idx {
                if src_idx != to && src_idx + 1 != to {
                    tab_action = Some(AppAction::ReorderDocument { from: src_idx, to });
                }
            }
        }

        if let Some(action_val) = tab_action {
            *action = action_val;
        } else if let Some(idx) = close_idx {
            *action = AppAction::CloseDocument(idx);
        }
    }
}

pub(crate) struct ViewModeBar<'a> {
    pub state: &'a mut AppState,
    pub pending_action: &'a mut AppAction,
}

impl<'a> ViewModeBar<'a> {
    pub fn new(state: &'a mut AppState, pending_action: &'a mut AppAction) -> Self {
        Self {
            state,
            pending_action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let state = self.state;
        let pending_action = self.pending_action;
        let mut mode = state.active_view_mode();
        let prev = mode;
        let bar_height = ui.spacing().interact_size.y;
        let available_width = ui.available_width();
        ui.allocate_ui_with_layout(
            egui::vec2(available_width, bar_height),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                // Render non-intrusive Update Available Badge
                if state.update.available.is_some() && !state.update.checking {
                    const COLOR_SUCCESS_G: u8 = 200;
                    let badge_str = format!("✨ {}", crate::i18n::get().update.update_available);
                    let badge_text = egui::RichText::new(badge_str)
                        .color(crate::theme_bridge::from_rgb(0, COLOR_SUCCESS_G, 100))
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

                // Omit View Mode controls for virtual tabs like ChangeLog since they are native documents
                let is_changelog = state.active_document().is_some_and(|doc| {
                    doc.path.to_string_lossy().starts_with("Katana://ChangeLog")
                });

                let prev_is_split = prev == ViewMode::Split;
                let is_split = mode == ViewMode::Split;

                if !is_changelog {
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
                }

                // Show split controls only while split mode is active.
                if !is_changelog && is_split && (is_split == prev_is_split) {
                    ui.separator();

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
                    let resp_dir = ui
                        .add(egui::Button::image(
                            dir_icon.image(icon_size).tint(ui.visuals().text_color()),
                        ))
                        .on_hover_text(dir_tip);

                    resp_dir.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            "Toggle Split Direction",
                        )
                    });

                    if resp_dir.clicked() {
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

                    ui.separator();

                    let mut is_on = state.scroll.sync_override.unwrap_or(
                        state
                            .config
                            .settings
                            .settings()
                            .behavior
                            .scroll_sync_enabled,
                    );

                    const TOGGLE_LABEL_SPACING: f32 = 8.0;

                    let resp = ui.add(
                        crate::widgets::LabeledToggle::new(
                            crate::i18n::get().settings.behavior.scroll_sync.clone(),
                            &mut is_on,
                        )
                        .position(crate::widgets::TogglePosition::Right)
                        .alignment(
                            crate::widgets::ToggleAlignment::Attached(TOGGLE_LABEL_SPACING),
                        ),
                    );

                    if resp.clicked() {
                        state.scroll.sync_override = Some(is_on);
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
}
