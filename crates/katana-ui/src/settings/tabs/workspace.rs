use crate::settings::*;

pub(crate) fn render_workspace_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    let workspace_msgs = &crate::i18n::get().settings.workspace;
    let settings = &mut state.config.settings;

    section_header(ui, &workspace_msgs.max_depth);
    let mut max_depth = settings.settings().workspace.max_depth;
    let slider = egui::Slider::new(&mut max_depth, 1..=100);
    if add_styled_slider(ui, slider).changed() {
        settings.settings_mut().workspace.max_depth = max_depth;
        let _ = settings.save();
    }

    ui.add_space(SECTION_SPACING);

    section_header(ui, &workspace_msgs.ignored_directories);
    let mut ignored = settings.settings().workspace.ignored_directories.clone();
    if render_string_list_editor(ui, &mut ignored) {
        settings.settings_mut().workspace.ignored_directories = ignored;
        let _ = settings.save();
    }
    ui.label(
        egui::RichText::new(&workspace_msgs.ignored_directories_hint)
            .weak()
            .size(HINT_FONT_SIZE),
    );

    ui.add_space(SECTION_SPACING);

    section_header(ui, &workspace_msgs.visible_extensions);
    let mut extensions = settings.settings().workspace.visible_extensions.clone();
    let mut changed_ext = false;

    ui.horizontal_wrapped(|ui| {
        for ext in &["md", "markdown", "mdx", "txt", "adr", ""] {
            let label_text = if ext.is_empty() {
                workspace_msgs.no_extension_label.as_str()
            } else {
                *ext
            };

            let mut is_enabled = extensions.contains(&ext.to_string());
            if ui
                .add(
                    crate::widgets::LabeledToggle::new(label_text, &mut is_enabled)
                        .alignment(crate::widgets::ToggleAlignment::Attached(8.0)),
                )
                .changed()
            {
                if is_enabled {
                    if ext.is_empty() {
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("show_no_extension_warning"), true)
                        });
                    } else if !extensions.contains(&ext.to_string()) {
                        extensions.push(ext.to_string());
                        changed_ext = true;
                    }
                } else {
                    extensions.retain(|e| e != ext);
                    changed_ext = true;
                }
            }
        }
    });

    if changed_ext {
        settings.settings_mut().workspace.visible_extensions = extensions;
        let _ = settings.save();
    }

    if settings
        .settings()
        .workspace
        .visible_extensions
        .contains(&"".to_string())
    {
        ui.add_space(SECTION_SPACING);

        section_header(ui, &workspace_msgs.extensionless_excludes);
        let mut excludes = settings.settings().workspace.extensionless_excludes.clone();
        if render_string_list_editor(ui, &mut excludes) {
            settings.settings_mut().workspace.extensionless_excludes = excludes;
            let _ = settings.save();
        }
        ui.label(
            egui::RichText::new(&workspace_msgs.extensionless_excludes_hint)
                .weak()
                .size(HINT_FONT_SIZE),
        );
    }

    let modal_id = egui::Id::new("show_no_extension_warning");
    let show_modal = ui.data(|d| d.get_temp::<bool>(modal_id).unwrap_or(false));
    if show_modal {
        let mut close = false;
        let mut confirm = false;
        egui::Window::new(&workspace_msgs.no_extension_warning_title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label(&workspace_msgs.no_extension_warning);
                ui.add_space(SUBSECTION_SPACING);
                ui.horizontal(|ui| {
                    if ui
                        .button(crate::i18n::get().action.cancel.clone())
                        .clicked()
                    {
                        close = true;
                    }
                    if ui
                        .button(crate::i18n::get().action.confirm.clone())
                        .clicked()
                    {
                        confirm = true;
                        close = true;
                    }
                });
            });

        let should_close = close || ui.input(|i| i.key_pressed(egui::Key::Escape));

        if confirm
            && !settings
                .settings()
                .workspace
                .visible_extensions
                .contains(&"".to_string())
        {
            settings
                .settings_mut()
                .workspace
                .visible_extensions
                .push("".to_string());
            let _ = settings.save();
        }

        if should_close {
            ui.data_mut(|d| d.insert_temp(modal_id, false));
        }
    }
}