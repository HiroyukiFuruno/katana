use crate::settings::*;
use katana_platform::settings::SettingsService;
use katana_platform::{PaneOrder, SplitDirection};

pub(crate) fn render_layout_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    let title = crate::i18n::get().settings.tab_name("layout");
    section_header(ui, &title);
    render_toc_toggle(ui, &mut state.config.settings);
    ui.add_space(SECTION_SPACING);
    render_toc_position_selector(ui, state);
    ui.add_space(SECTION_SPACING);
    render_split_direction_selector(ui, state);
    ui.add_space(LAYOUT_SELECTOR_SPACING);
    render_pane_order_selector(ui, state);
}

pub(crate) fn render_toc_toggle(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let mut toc_visible = settings.settings().layout.toc_visible;
    if ui
        .add(
            crate::widgets::LabeledToggle::new(
                &crate::i18n::get().settings.toc_visible,
                &mut toc_visible,
            )
            .position(crate::widgets::TogglePosition::Right)
            .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
        )
        .changed()
    {
        settings.settings_mut().layout.toc_visible = toc_visible;
        let _ = settings.save();
    }
}

pub(crate) fn render_toc_position_selector(
    ui: &mut egui::Ui,
    state: &mut crate::app_state::AppState,
) {
    if !state.config.settings.settings().layout.toc_visible {
        return;
    }

    use katana_platform::settings::TocPosition;

    ui.label(crate::i18n::get().settings.layout.toc_position.clone());
    ui.horizontal(|ui| {
        let current = state.config.settings.settings().layout.toc_position;
        if ui
            .selectable_label(
                current == TocPosition::Left,
                crate::i18n::get().settings.layout.left.clone(),
            )
            .clicked()
            && current != TocPosition::Left
        {
            state.config.settings.settings_mut().layout.toc_position = TocPosition::Left;
            let _ = state.config.settings.save();
        }
        if ui
            .selectable_label(
                current == TocPosition::Right,
                crate::i18n::get().settings.layout.right.clone(),
            )
            .clicked()
            && current != TocPosition::Right
        {
            state.config.settings.settings_mut().layout.toc_position = TocPosition::Right;
            let _ = state.config.settings.save();
        }
    });
}

pub(crate) fn render_split_direction_selector(
    ui: &mut egui::Ui,
    state: &mut crate::app_state::AppState,
) {
    ui.label(crate::i18n::get().settings.layout.split_direction.clone());
    ui.horizontal(|ui| {
        let current = state.config.settings.settings().layout.split_direction;
        if ui
            .selectable_label(
                current == SplitDirection::Horizontal,
                crate::i18n::get().settings.layout.horizontal.clone(),
            )
            .clicked()
            && current != SplitDirection::Horizontal
        {
            state.config.settings.settings_mut().layout.split_direction =
                SplitDirection::Horizontal;
            let _ = state.config.settings.save();
        }
        if ui
            .selectable_label(
                current == SplitDirection::Vertical,
                crate::i18n::get().settings.layout.vertical.clone(),
            )
            .clicked()
            && current != SplitDirection::Vertical
        {
            state.config.settings.settings_mut().layout.split_direction = SplitDirection::Vertical;
            let _ = state.config.settings.save();
        }
    });
}

pub(crate) fn render_pane_order_selector(
    ui: &mut egui::Ui,
    state: &mut crate::app_state::AppState,
) {
    ui.label(crate::i18n::get().settings.layout.pane_order.clone());
    ui.horizontal(|ui| {
        let current = state.config.settings.settings().layout.pane_order;
        if ui
            .selectable_label(
                current == PaneOrder::EditorFirst,
                crate::i18n::get().settings.layout.editor_first.clone(),
            )
            .clicked()
            && current != PaneOrder::EditorFirst
        {
            state.config.settings.settings_mut().layout.pane_order = PaneOrder::EditorFirst;
            let _ = state.config.settings.save();
        }
        if ui
            .selectable_label(
                current == PaneOrder::PreviewFirst,
                crate::i18n::get().settings.layout.preview_first.clone(),
            )
            .clicked()
            && current != PaneOrder::PreviewFirst
        {
            state.config.settings.settings_mut().layout.pane_order = PaneOrder::PreviewFirst;
            let _ = state.config.settings.save();
        }
    });
}


pub(crate) fn render_string_list_editor(ui: &mut egui::Ui, list: &mut Vec<String>) -> bool {
    let mut changed = false;
    let mut to_remove = None;

    ui.vertical(|ui| {
        for (i, item) in list.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                ui.horizontal(|ui| {
                    let response = ui.text_edit_singleline(item);
                    if response.changed() {
                        changed = true;
                    }
                    if ui.button("-").clicked() {
                        to_remove = Some(i);
                    }
                });
            });
        }

        if let Some(i) = to_remove {
            list.remove(i);
            changed = true;
        }

        if ui.button("+").clicked() {
            list.push(String::new());
            changed = true;
        }
    });

    changed
}