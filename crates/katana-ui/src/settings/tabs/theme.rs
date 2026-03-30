use crate::settings::*;
use crate::theme_bridge;
use katana_platform::settings::SettingsService;
use katana_platform::theme::{Rgb, Rgba, ThemeColors, ThemeMode, ThemePreset};

pub(crate) fn render_theme_tab(ui: &mut egui::Ui, settings: &mut SettingsService) {
    section_header(
        ui,
        crate::i18n::get()
            .settings
            .theme
            .ui_contrast_offset
            .as_str(),
    );
    let mut offset = settings.settings().theme.ui_contrast_offset;
    let original_offset = offset;
    let slider = egui::Slider::new(&mut offset, -100.0..=100.0)
        .step_by(1.0)
        .suffix(" %");
    if add_styled_slider(ui, slider).changed() {
        settings.settings_mut().theme.ui_contrast_offset = offset;
        if offset != original_offset {
            let colors = settings.settings().effective_theme_colors();
            ui.ctx()
                .set_visuals(crate::theme_bridge::visuals_from_theme(&colors));
        }
    }
    ui.add_space(SECTION_SPACING);

    render_theme_preset_selector(ui, settings);
    ui.add_space(SECTION_SPACING);

    ui.add_space(SECTION_SPACING);

    let is_open = settings.settings().theme.custom_color_overrides.is_some();

    crate::widgets::Accordion::new(
        "custom_color_overrides_accordion",
        egui::RichText::new(crate::i18n::get().settings.theme.custom_colors.clone())
            .strong()
            .size(SECTION_HEADER_SIZE),
        |ui| render_custom_color_editor(ui, settings),
    )
    .default_open(is_open)
    .show(ui);
}

pub(crate) fn render_theme_preset_selector(ui: &mut egui::Ui, settings: &mut SettingsService) {
    section_header(ui, &crate::i18n::get().settings.theme.preset);

    let show_more_id = ui.id().with("show_more_themes");
    let mut show_more = ui.data_mut(|d| d.get_temp::<bool>(show_more_id).unwrap_or(false));

    const VISIBLE_PRESET_COUNT: usize = 5;

    ui.label(egui::RichText::new(crate::i18n::get().settings.theme.dark_section.clone()).weak());
    let all_presets = ThemePreset::builtins();
    let mut dark_presets: Vec<&ThemePreset> = all_presets
        .iter()
        .filter(|it| it.colors().mode == ThemeMode::Dark)
        .collect();
    if !show_more {
        dark_presets.truncate(VISIBLE_PRESET_COUNT);
    }
    render_preset_group(ui, settings, &dark_presets);

    ui.add_space(SECTION_SPACING);

    ui.label(egui::RichText::new(crate::i18n::get().settings.theme.light_section.clone()).weak());
    let mut light_presets: Vec<&ThemePreset> = all_presets
        .iter()
        .filter(|it| it.colors().mode == ThemeMode::Light)
        .collect();
    if !show_more {
        light_presets.truncate(VISIBLE_PRESET_COUNT);
    }
    render_preset_group(ui, settings, &light_presets);

    let custom_themes = settings.settings().theme.custom_themes.clone();
    if !custom_themes.is_empty() {
        ui.add_space(SECTION_SPACING);
        ui.label(
            egui::RichText::new(crate::i18n::get().settings.theme.custom_section.clone()).weak(),
        );
        for (idx, custom_theme) in custom_themes.iter().enumerate() {
            let is_selected = settings.settings().theme.custom_color_overrides.as_ref()
                == Some(&custom_theme.colors);
            let bg_color = theme_bridge::rgb_to_color32(custom_theme.colors.system.background);
            let accent_color = theme_bridge::rgb_to_color32(custom_theme.colors.system.accent);

            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(PRESET_SWATCH_SIZE, PRESET_SWATCH_SIZE),
                    egui::Sense::hover(),
                );
                let corner = PRESET_SWATCH_SIZE / SWATCH_CORNER_DIVISOR;
                ui.painter().rect_filled(rect, corner, bg_color);
                ui.painter()
                    .circle_filled(rect.center(), corner, accent_color);

                let response = ui.selectable_label(is_selected, &custom_theme.name);
                if response.clicked() && !is_selected {
                    settings.settings_mut().theme.custom_color_overrides =
                        Some(custom_theme.colors.clone());
                    settings.settings_mut().theme.active_custom_theme =
                        Some(custom_theme.name.clone());
                    let _ = settings.save();
                }

                response.context_menu(|ui| {
                    if ui
                        .button(crate::i18n::get().settings.theme.duplicate.clone())
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("show_save_theme_modal"), true);
                            d.insert_temp(
                                egui::Id::new("custom_theme_name_input"),
                                format!("{} copy", custom_theme.name),
                            );
                            d.insert_temp(
                                egui::Id::new("duplicate_theme_colors"),
                                custom_theme.colors.clone(),
                            );
                        });
                        ui.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::image(
                            crate::Icon::Remove.ui_image(ui, crate::icon::IconSize::Medium),
                        ))
                        .on_hover_text(crate::i18n::get().settings.theme.delete_custom.clone())
                        .clicked()
                    {
                        settings.settings_mut().theme.custom_themes.remove(idx);
                        if is_selected {
                            settings.settings_mut().theme.custom_color_overrides = None;
                            settings.settings_mut().theme.active_custom_theme = None;
                        }
                        let _ = settings.save();
                    }
                });
            });
        }
    }

    ui.add_space(SUBSECTION_SPACING);

    let msgs = &crate::i18n::get().settings.theme;
    let toggle_text = if show_more {
        &msgs.show_less
    } else {
        &msgs.show_more
    };
    if ui.link(toggle_text).clicked() {
        show_more = !show_more;
        ui.data_mut(|d| d.insert_temp(show_more_id, show_more));
    }
}

pub(crate) fn render_preset_group(
    ui: &mut egui::Ui,
    settings: &mut SettingsService,
    presets: &[&ThemePreset],
) {
    for preset in presets {
        let is_selected = settings.settings().theme.preset == **preset;
        let colors = preset.colors();
        let bg_color = theme_bridge::rgb_to_color32(colors.system.background);
        let accent_color = theme_bridge::rgb_to_color32(colors.system.accent);

        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(PRESET_SWATCH_SIZE, PRESET_SWATCH_SIZE),
                egui::Sense::hover(),
            );
            let corner = PRESET_SWATCH_SIZE / SWATCH_CORNER_DIVISOR;
            ui.painter().rect_filled(rect, corner, bg_color);
            ui.painter()
                .circle_filled(rect.center(), corner, accent_color);

            let response = ui.selectable_label(is_selected, preset.display_name());
            if response.clicked() && !is_selected {
                settings.settings_mut().theme.preset = **preset;
                settings.settings_mut().theme.custom_color_overrides = None;
                settings.settings_mut().theme.active_custom_theme = None;
                let _ = settings.save();
            }

            response.context_menu(|ui| {
                if ui
                    .button(crate::i18n::get().settings.theme.duplicate.clone())
                    .clicked()
                {
                    ui.data_mut(|d| {
                        d.insert_temp(egui::Id::new("show_save_theme_modal"), true);
                        d.insert_temp(
                            egui::Id::new("custom_theme_name_input"),
                            format!("{} copy", preset.display_name()),
                        );
                        d.insert_temp(
                            egui::Id::new("duplicate_theme_colors"),
                            preset.colors().clone(),
                        );
                    });
                    ui.close();
                }
            });
        });
    }
}

pub(crate) fn render_custom_color_editor(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let current_colors = settings.settings().effective_theme_colors();
    let color_i18n = &crate::i18n::get().settings.color;

    let mut changed = false;
    let mut new_colors = current_colors.clone();

    struct ColorSettingDef<'a> {
        label: &'a String,
        prop: ColorPropType,
    }

    let system_settings = vec![
        (
            Some(&color_i18n.group_basic),
            vec![
                ColorSettingDef {
                    label: &color_i18n.background,
                    prop: ColorPropType::Rgb(
                        |c| c.system.background,
                        |c, r| c.system.background = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.panel_background,
                    prop: ColorPropType::Rgb(
                        |c| c.system.panel_background,
                        |c, r| c.system.panel_background = r,
                    ),
                },
            ],
        ),
        (
            Some(&color_i18n.group_text),
            vec![
                ColorSettingDef {
                    label: &color_i18n.text,
                    prop: ColorPropType::Rgb(|c| c.system.text, |c, r| c.system.text = r),
                },
                ColorSettingDef {
                    label: &color_i18n.text_secondary,
                    prop: ColorPropType::Rgb(
                        |c| c.system.text_secondary,
                        |c, r| c.system.text_secondary = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.success_text,
                    prop: ColorPropType::Rgb(
                        |c| c.system.success_text,
                        |c, r| c.system.success_text = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.warning_text,
                    prop: ColorPropType::Rgb(
                        |c| c.system.warning_text,
                        |c, r| c.system.warning_text = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.error_text,
                    prop: ColorPropType::Rgb(
                        |c| c.system.error_text,
                        |c, r| c.system.error_text = r,
                    ),
                },
            ],
        ),
        (
            Some(&color_i18n.group_ui_elements),
            vec![
                ColorSettingDef {
                    label: &color_i18n.title_bar_text,
                    prop: ColorPropType::Rgb(
                        |c| c.system.title_bar_text,
                        |c, r| c.system.title_bar_text = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.file_tree_text,
                    prop: ColorPropType::Rgb(
                        |c| c.system.file_tree_text,
                        |c, r| c.system.file_tree_text = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.accent,
                    prop: ColorPropType::Rgb(|c| c.system.accent, |c, r| c.system.accent = r),
                },
                ColorSettingDef {
                    label: &color_i18n.selection,
                    prop: ColorPropType::Rgb(|c| c.system.selection, |c, r| c.system.selection = r),
                },
                ColorSettingDef {
                    label: &color_i18n.border,
                    prop: ColorPropType::Rgb(|c| c.system.border, |c, r| c.system.border = r),
                },
                ColorSettingDef {
                    label: &color_i18n.button_background,
                    prop: ColorPropType::Rgba(
                        |c| c.system.button_background,
                        |c, r| c.system.button_background = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.button_active_background,
                    prop: ColorPropType::Rgba(
                        |c| c.system.button_active_background,
                        |c, r| c.system.button_active_background = r,
                    ),
                },
                ColorSettingDef {
                    label: &color_i18n.active_file_highlight,
                    prop: ColorPropType::Rgba(
                        |c| c.system.active_file_highlight,
                        |c, r| c.system.active_file_highlight = r,
                    ),
                },
            ],
        ),
    ];

    let code_settings = vec![(
        None,
        vec![
            ColorSettingDef {
                label: &color_i18n.code_background,
                prop: ColorPropType::Rgb(|c| c.code.background, |c, r| c.code.background = r),
            },
            ColorSettingDef {
                label: &color_i18n.code_text,
                prop: ColorPropType::Rgb(|c| c.code.text, |c, r| c.code.text = r),
            },
            ColorSettingDef {
                label: &color_i18n.highlight,
                prop: ColorPropType::Rgb(|c| c.code.selection, |c, r| c.code.selection = r),
            },
            ColorSettingDef {
                label: &color_i18n.line_number_text,
                prop: ColorPropType::Rgb(
                    |c| c.code.line_number_text,
                    |c, r| c.code.line_number_text = r,
                ),
            },
            ColorSettingDef {
                label: &color_i18n.line_number_active_text,
                prop: ColorPropType::Rgb(
                    |c| c.code.line_number_active_text,
                    |c, r| c.code.line_number_active_text = r,
                ),
            },
            ColorSettingDef {
                label: &color_i18n.current_line_background,
                prop: ColorPropType::Rgba(
                    |c| c.code.current_line_background,
                    |c, r| c.code.current_line_background = r,
                ),
            },
            ColorSettingDef {
                label: &color_i18n.hover_line_background,
                prop: ColorPropType::Rgba(
                    |c| c.code.hover_line_background,
                    |c, r| c.code.hover_line_background = r,
                ),
            },
        ],
    )];

    let preview_settings = vec![(
        None,
        vec![
            ColorSettingDef {
                label: &color_i18n.preview_background,
                prop: ColorPropType::Rgb(|c| c.preview.background, |c, r| c.preview.background = r),
            },
            ColorSettingDef {
                label: &color_i18n.preview_text,
                prop: ColorPropType::Rgb(|c| c.preview.text, |c, r| c.preview.text = r),
            },
            ColorSettingDef {
                label: &color_i18n.warning_text,
                prop: ColorPropType::Rgb(
                    |c| c.preview.warning_text,
                    |c, r| c.preview.warning_text = r,
                ),
            },
            ColorSettingDef {
                label: &color_i18n.highlight,
                prop: ColorPropType::Rgb(|c| c.preview.selection, |c, r| c.preview.selection = r),
            },
            ColorSettingDef {
                label: &color_i18n.border,
                prop: ColorPropType::Rgb(|c| c.preview.border, |c, r| c.preview.border = r),
            },
            ColorSettingDef {
                label: &color_i18n.hover_line_background,
                prop: ColorPropType::Rgba(
                    |c| c.preview.hover_line_background,
                    |c, r| c.preview.hover_line_background = r,
                ),
            },
        ],
    )];

    let sections = vec![
        (&color_i18n.section_system, system_settings),
        (&color_i18n.section_code, code_settings),
        (&color_i18n.section_preview, preview_settings),
    ];

    for (section_name, grouped_settings) in sections {
        crate::widgets::Accordion::new(
            section_name.clone(),
            egui::RichText::new(section_name.clone())
                .strong()
                .size(SECTION_HEADER_SIZE),
            |ui| {
                ui.add_space(SUBSECTION_SPACING);
                for (group_opt, settings_list) in grouped_settings {
                    ui.add_space(SUBSECTION_SPACING);
                    if let Some(group_name) = group_opt {
                        crate::widgets::Accordion::new(
                            group_name.clone(),
                            group_name.clone(),
                            |ui| {
                                ui.add_space(SUBSECTION_SPACING);
                                for def in settings_list {
                                    changed |=
                                        render_color_row(ui, &mut new_colors, def.label, &def.prop);
                                    ui.add_space(SUBSECTION_SPACING);
                                }
                            },
                        )
                        .default_open(true)
                        .show(ui);
                    } else {
                        for def in settings_list {
                            changed |= render_color_row(ui, &mut new_colors, def.label, &def.prop);
                            ui.add_space(SUBSECTION_SPACING);
                        }
                    }
                }
            },
        )
        .default_open(true)
        .show(ui);
        ui.add_space(SECTION_SPACING);
    }

    if changed {
        settings.settings_mut().theme.custom_color_overrides = Some(new_colors);
        let _ = settings.save();
    }

    ui.add_space(SUBSECTION_SPACING);

    let active_custom = settings.settings().theme.active_custom_theme.clone();

    ui.with_layout(
        egui::Layout::top_down_justified(egui::Align::Center),
        |ui| {
            let limit_reached = settings.settings().theme.custom_themes.len()
                >= katana_platform::settings::MAX_CUSTOM_THEMES;
            ui.add_enabled_ui(!limit_reached, |ui| {
                let save_btn =
                    ui.button(crate::i18n::get().settings.theme.save_custom_theme.clone());
                if save_btn.clicked() {
                    ui.data_mut(|d| d.insert_temp(egui::Id::new("show_save_theme_modal"), true));

                    if let Some(name) = &active_custom {
                        let dup_name = format!("{} copy", name);
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("custom_theme_name_input"), dup_name)
                        });
                    } else {
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("custom_theme_name_input"), String::new())
                        });
                    }
                }
            });

            if settings.settings().theme.custom_color_overrides.is_some() {
                ui.add_space(SUBSECTION_SPACING);
                if ui
                    .button(crate::i18n::get().settings.theme.reset_custom.clone())
                    .clicked()
                {
                    if let Some(name) = &active_custom {
                        if let Some(theme) = settings
                            .settings()
                            .theme
                            .custom_themes
                            .iter()
                            .find(|t| t.name == *name)
                        {
                            settings.settings_mut().theme.custom_color_overrides =
                                Some(theme.colors.clone());
                        } else {
                            settings.settings_mut().theme.custom_color_overrides = None;
                            settings.settings_mut().theme.active_custom_theme = None;
                        }
                    } else {
                        settings.settings_mut().theme.custom_color_overrides = None;
                        settings.settings_mut().theme.active_custom_theme = None;
                    }
                    let _ = settings.save();
                }
            }
        },
    );

    let modal_id = egui::Id::new("show_save_theme_modal");
    let show_modal = ui.data(|d| d.get_temp::<bool>(modal_id).unwrap_or(false));
    if show_modal {
        let mut close = false;
        egui::Window::new(
            crate::i18n::get()
                .settings
                .theme
                .save_custom_theme_title
                .clone(),
        )
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ui.ctx(), |ui| {
            let name_id = egui::Id::new("custom_theme_name_input");
            let mut name = ui.data(|d| d.get_temp::<String>(name_id).unwrap_or_default());

            ui.horizontal(|ui| {
                ui.label(crate::i18n::get().settings.theme.theme_name_label.clone());
                let re = ui.text_edit_singleline(&mut name);
                re.request_focus();
                if re.changed() {
                    ui.data_mut(|d| d.insert_temp(name_id, name.clone()));
                }
            });

            ui.add_space(SUBSECTION_SPACING);
            ui.horizontal(|ui| {
                if ui
                    .button(crate::i18n::get().action.cancel.clone())
                    .clicked()
                {
                    close = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(crate::i18n::get().action.save.clone()).clicked()
                        && !name.is_empty()
                    {
                        let dup_id = egui::Id::new("duplicate_theme_colors");
                        let mut theme_colors = ui
                            .data(|d| d.get_temp::<ThemeColors>(dup_id))
                            .unwrap_or_else(|| settings.settings().effective_theme_colors());
                        theme_colors.name = name.clone();

                        let mut themes = settings.settings().theme.custom_themes.clone();
                        if let Some(existing) = themes.iter_mut().find(|t| t.name == name) {
                            existing.colors = theme_colors.clone();
                        } else {
                            themes.push(katana_platform::settings::CustomTheme {
                                name: name.clone(),
                                colors: theme_colors.clone(),
                            });
                        }
                        settings.settings_mut().theme.custom_themes = themes;
                        settings.settings_mut().theme.custom_color_overrides = Some(theme_colors);
                        settings.settings_mut().theme.active_custom_theme = Some(name.clone());

                        let _ = settings.save();
                        close = true;
                    }
                });
            });

            let should_close = close || ui.input(|i| i.key_pressed(egui::Key::Escape));
            if should_close {
                ui.data_mut(|d: &mut egui::util::IdTypeMap| {
                    d.insert_temp(modal_id, false);
                    d.remove::<String>(egui::Id::new("custom_theme_name_input"));
                    d.remove::<ThemeColors>(egui::Id::new("duplicate_theme_colors"));
                });
            }
        });
    }
}

pub(crate) enum ColorPropType {
    Rgb(fn(&ThemeColors) -> Rgb, fn(&mut ThemeColors, Rgb)),
    Rgba(fn(&ThemeColors) -> Rgba, fn(&mut ThemeColors, Rgba)),
}

pub(crate) fn render_color_row(
    ui: &mut egui::Ui,
    new_colors: &mut ThemeColors,
    label: &str,
    prop: &ColorPropType,
) -> bool {
    let mut changed = false;
    match prop {
        ColorPropType::Rgb(get, apply) => {
            let original_rgb = get(new_colors);
            let mut color = crate::theme_bridge::rgb_to_color32(original_rgb);
            let response = crate::widgets::LabeledColorPicker::new(label)
                .label_width(COLOR_GRID_LABEL_WIDTH)
                .spacing(SECTION_SPACING)
                .show_rgb(ui, &mut color);

            if response.changed() {
                let new_rgb = Rgb {
                    r: color.r(),
                    g: color.g(),
                    b: color.b(),
                };
                apply(new_colors, new_rgb);
                changed = true;
            }
        }
        ColorPropType::Rgba(get, apply) => {
            let original_rgba = get(new_colors);
            let mut color = crate::theme_bridge::rgba_to_color32(original_rgba);
            let response = crate::widgets::LabeledColorPicker::new(label)
                .label_width(COLOR_GRID_LABEL_WIDTH)
                .spacing(SECTION_SPACING)
                .show_rgba(ui, &mut color);

            if response.changed() {
                let [r, g, b, a] = color.to_srgba_unmultiplied();
                let new_rgba = Rgba { r, g, b, a };
                apply(new_colors, new_rgba);
                changed = true;
            }
        }
    }

    changed
}
