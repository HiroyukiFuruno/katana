//! Settings window rendered as an `egui::Window` overlay.
//!
//! The window uses a split layout:
//!   - **Left pane**: Tab bar + settings controls (scrollable)
//!   - **Right pane**: Live markdown preview using `PreviewPane` (scrollable independently)

use crate::app_state::{AppAction, SettingsTab};
use crate::preview_pane::PreviewPane;
use crate::theme_bridge;
use crate::widgets::StyledComboBox;
use katana_platform::settings::{SettingsService, MAX_FONT_SIZE, MIN_FONT_SIZE};
use katana_platform::theme::{Rgb, Rgba, ThemeColors, ThemeMode, ThemePreset};
use katana_platform::{PaneOrder, SplitDirection};

// ── Window layout constants ──────────────────────────────────────────

const SETTINGS_WINDOW_DEFAULT_WIDTH: f32 = 1000.0;
const SETTINGS_WINDOW_DEFAULT_HEIGHT: f32 = 500.0;

const SETTINGS_SIDE_PANEL_DEFAULT_WIDTH: f32 = 200.0;

const SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH: f32 = 350.0;

const SETTINGS_HEADER_FONT_SIZE: f32 = 14.0;
const SETTINGS_GROUP_SPACING: f32 = 8.0;
const SETTINGS_TOGGLE_SPACING: f32 = 8.0;

const AUTO_SAVE_INTERVAL_MIN: f64 = 0.0;
const AUTO_SAVE_INTERVAL_MAX: f64 = 300.0;
const AUTO_SAVE_INTERVAL_STEP: f64 = 0.1;

// ── Spacing & sizing constants ───────────────────────────────────────

const SECTION_SPACING: f32 = 12.0;
const SUBSECTION_SPACING: f32 = 6.0;
const INNER_MARGIN: f32 = 12.0;

const FONT_SIZE_STEP: f64 = 1.0;
/// Spacing between layout selectors (split direction / pane order) within the Layout tab.
const LAYOUT_SELECTOR_SPACING: f32 = 4.0;
const PRESET_SWATCH_SIZE: f32 = 14.0;
const COLOR_GRID_LABEL_WIDTH: f32 = 130.0;
const SECTION_HEADER_SIZE: f32 = 14.0;
const SECTION_HEADER_MARGIN: f32 = 4.0;
const SWATCH_CORNER_DIVISOR: f32 = 4.0;
const FONT_FAMILY_COMBOBOX_WIDTH: f32 = 200.0;
/// Maximum height of the scrollable font list inside the search popup.
const FONT_DROPDOWN_MAX_HEIGHT: f32 = 200.0;
/// Opacity for the inactive slider rail (0–255). Provides visible contrast on both light and dark themes.
const SLIDER_RAIL_OPACITY: u8 = 80;
/// Border width for the slider handle and rail for visibility on all themes.
const SLIDER_BORDER_WIDTH: f32 = 1.0;
/// Font size for hint text in the settings window.
const HINT_FONT_SIZE: f32 = 10.0;

// ── Sample markdown for settings preview ─────────────────────────────

const SAMPLE_MARKDOWN: &str = r#"# Heading 1

## Heading 2

Normal paragraph text with **bold**, *italic*, and `inline code`.

- List item 1
- List item 2
  - Nested item

> Blockquote text goes here.

```rust
fn main() {
    println!("Hello, KatanA!");
}
```

| Column A | Column B |
|----------|----------|
| Cell 1   | Cell 2   |

---

Secondary text and [a link](https://example.com) for reference.
"#;

// ── Public rendering entry-point ─────────────────────────────────────

/// Render the settings window. Returns an action to dispatch if triggered from settings.
pub(crate) fn render_settings_window(
    ctx: &egui::Context,
    state: &mut crate::app_state::AppState,
    preview_pane: &mut PreviewPane,
) -> Option<AppAction> {
    if !state.layout.show_settings {
        return None;
    }

    let mut triggered_action: Option<AppAction> = None;

    // Ensure preview pane has content loaded
    if preview_pane.sections.is_empty() {
        preview_pane.update_markdown_sections(
            SAMPLE_MARKDOWN,
            std::path::Path::new("/settings-preview.md"),
        );
    }

    let mut open = state.layout.show_settings;
    egui::Window::new(crate::i18n::get().settings.title.clone())
        .open(&mut open)
        .fixed_size(egui::vec2(
            SETTINGS_WINDOW_DEFAULT_WIDTH,
            SETTINGS_WINDOW_DEFAULT_HEIGHT,
        ))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_min_width(SETTINGS_WINDOW_DEFAULT_WIDTH);
            ui.set_min_height(SETTINGS_WINDOW_DEFAULT_HEIGHT);

            egui::SidePanel::left("settings_left_panel")
                .resizable(false)
                .min_width(SETTINGS_SIDE_PANEL_DEFAULT_WIDTH)
                .max_width(SETTINGS_SIDE_PANEL_DEFAULT_WIDTH)
                .show_inside(ui, |ui| {
                    // Expand All / Collapse All toolbar
                    ui.horizontal(|ui| {
                        const TAB_SPACING: f32 = 4.0;
                        ui.add_space(TAB_SPACING);
                        if ui
                            .add(egui::Button::image_and_text(
                                crate::Icon::ExpandAll.ui_image(ui, crate::icon::IconSize::Small),
                                "",
                            ))
                            .on_hover_text(crate::i18n::get().action.expand_all.clone())
                            .clicked()
                        {
                            state.config.settings_tree_force_open = Some(true);
                        }
                        if ui
                            .add(egui::Button::image_and_text(
                                crate::Icon::CollapseAll.ui_image(ui, crate::icon::IconSize::Small),
                                "",
                            ))
                            .on_hover_text(crate::i18n::get().action.collapse_all.clone())
                            .clicked()
                        {
                            state.config.settings_tree_force_open = Some(false);
                        }
                    });
                    const TAB_SPACING: f32 = 4.0;
                    ui.add_space(TAB_SPACING);
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("settings_nav_scroll")
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            render_settings_tree(ui, state);
                        });
                });

            let show_preview = matches!(
                state.config.active_settings_tab,
                SettingsTab::Theme | SettingsTab::Font | SettingsTab::Layout
            );

            if show_preview {
                egui::SidePanel::right("settings_right_panel")
                    .resizable(false)
                    .min_width(SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH)
                    .max_width(SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH)
                    .show_inside(ui, |ui| {
                        section_header(ui, &crate::i18n::get().settings.preview.title);
                        preview_pane.show(ui);
                    });
            }

            egui::CentralPanel::default().show_inside(ui, |ui| {
                let tab_messages = &crate::i18n::get().settings.tabs;
                let title = match state.config.active_settings_tab {
                    SettingsTab::Theme => tab_messages
                        .iter()
                        .find(|t| t.key == "theme")
                        .map(|t| t.name.as_str())
                        .unwrap_or("Theme"),
                    SettingsTab::Font => tab_messages
                        .iter()
                        .find(|t| t.key == "font")
                        .map(|t| t.name.as_str())
                        .unwrap_or("Font"),
                    SettingsTab::Layout => tab_messages
                        .iter()
                        .find(|t| t.key == "layout")
                        .map(|t| t.name.as_str())
                        .unwrap_or("Layout"),
                    SettingsTab::Workspace => tab_messages
                        .iter()
                        .find(|t| t.key == "workspace")
                        .map(|t| t.name.as_str())
                        .unwrap_or("Workspace"),
                    SettingsTab::Updates => tab_messages
                        .iter()
                        .find(|t| t.key == "updates")
                        .map(|t| t.name.as_str())
                        .unwrap_or("Updates"),
                    SettingsTab::Behavior => tab_messages
                        .iter()
                        .find(|t| t.key == "behavior")
                        .map(|t| t.name.as_str())
                        .unwrap_or("Behavior"),
                };

                section_header(ui, title);

                egui::ScrollArea::vertical()
                    .id_salt("settings_form_scroll")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        egui::Frame::NONE.inner_margin(INNER_MARGIN).show(ui, |ui| {
                            match state.config.active_settings_tab {
                                SettingsTab::Theme => {
                                    render_theme_tab(ui, &mut state.config.settings)
                                }
                                SettingsTab::Font => {
                                    render_font_tab(ui, &mut state.config.settings)
                                }
                                SettingsTab::Layout => render_layout_tab(ui, state),
                                SettingsTab::Workspace => render_workspace_tab(ui, state),
                                SettingsTab::Updates => {
                                    if let Some(action) = render_updates_tab(ui, state) {
                                        triggered_action = Some(action);
                                    }
                                }
                                SettingsTab::Behavior => {
                                    if let Some(action) = render_behavior_tab(ui, state) {
                                        triggered_action = Some(action);
                                    }
                                }
                            }
                        });
                    });
            });

            // Clear the force open flag after rendering the tree once
            if state.config.settings_tree_force_open.is_some() {
                state.config.settings_tree_force_open = None;
            }
        });
    state.layout.show_settings = open;
    triggered_action
}
// ── Tree Navigation ──────────────────────────────────────────────────

fn render_settings_tree(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    let settings_msgs = &crate::i18n::get().settings;

    // Group 1: Appearance (Theme, Font, Layout)
    let appearance_key = "group_appearance";
    // Fallback to English if "group_appearance" key is not found
    let title = settings_msgs
        .tabs
        .iter()
        .find(|t| t.key == appearance_key)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Appearance".to_string());

    let mut appearance_header = egui::CollapsingHeader::new(
        egui::RichText::new(title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
    )
    .default_open(true)
    .id_salt("settings_grp_appearance")
    .icon(egui_commonmark::ui_components::centering::AccordionIcon::paint_optically_centered);

    if let Some(force_open) = state.config.settings_tree_force_open {
        appearance_header = appearance_header.open(Some(force_open));
    }

    appearance_header.show(ui, |ui| {
        let theme_selected = state.config.active_settings_tab == SettingsTab::Theme;
        if ui
            .selectable_label(theme_selected, settings_msgs.tab_name("theme"))
            .clicked()
        {
            state.config.active_settings_tab = SettingsTab::Theme;
        }

        let font_selected = state.config.active_settings_tab == SettingsTab::Font;
        if ui
            .selectable_label(font_selected, settings_msgs.tab_name("font"))
            .clicked()
        {
            state.config.active_settings_tab = SettingsTab::Font;
        }

        let layout_selected = state.config.active_settings_tab == SettingsTab::Layout;
        if ui
            .selectable_label(layout_selected, settings_msgs.tab_name("layout"))
            .clicked()
        {
            state.config.active_settings_tab = SettingsTab::Layout;
        }
    });

    ui.add_space(SETTINGS_GROUP_SPACING);

    // Group 2: System/Behavior (Workspace)
    let system_key = "group_system";
    let title = settings_msgs
        .tabs
        .iter()
        .find(|t| t.key == system_key)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "System".to_string());

    let mut system_header = egui::CollapsingHeader::new(
        egui::RichText::new(title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
    )
    .default_open(true)
    .id_salt("settings_grp_system")
    .icon(egui_commonmark::ui_components::centering::AccordionIcon::paint_optically_centered);

    if let Some(force_open) = state.config.settings_tree_force_open {
        system_header = system_header.open(Some(force_open));
    }

    system_header.show(ui, |ui| {
        let workspace_selected = state.config.active_settings_tab == SettingsTab::Workspace;
        if ui
            .selectable_label(workspace_selected, settings_msgs.tab_name("workspace"))
            .clicked()
        {
            state.config.active_settings_tab = SettingsTab::Workspace;
        }

        let updates_selected = state.config.active_settings_tab == SettingsTab::Updates;
        if ui
            .selectable_label(updates_selected, settings_msgs.tab_name("updates"))
            .clicked()
        {
            state.config.active_settings_tab = SettingsTab::Updates;
        }

        let behavior_selected = state.config.active_settings_tab == SettingsTab::Behavior;
        if ui
            .selectable_label(behavior_selected, settings_msgs.tab_name("behavior"))
            .clicked()
        {
            state.config.active_settings_tab = SettingsTab::Behavior;
        }
    });
}

// ── Section header helper ────────────────────────────────────────────

fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(SECTION_HEADER_MARGIN);
    ui.label(egui::RichText::new(text).size(SECTION_HEADER_SIZE).strong());
    ui.add_space(SECTION_HEADER_MARGIN);
    ui.separator();
    ui.add_space(SUBSECTION_SPACING);
}

// ── Theme tab ────────────────────────────────────────────────────────

fn render_theme_tab(ui: &mut egui::Ui, settings: &mut SettingsService) {
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
            // Apply the new effective colors dynamically
            let colors = settings.settings().effective_theme_colors();
            ui.ctx()
                .set_visuals(crate::theme_bridge::visuals_from_theme(&colors));
        }
    }
    ui.add_space(SECTION_SPACING);

    render_theme_preset_selector(ui, settings);
    ui.add_space(SECTION_SPACING);

    ui.add_space(SECTION_SPACING);

    egui::CollapsingHeader::new(
        egui::RichText::new(crate::i18n::get().settings.theme.custom_colors.clone())
            .strong()
            .size(SECTION_HEADER_SIZE),
    )
    .default_open(settings.settings().theme.custom_color_overrides.is_some())
    .icon(egui_commonmark::ui_components::centering::AccordionIcon::paint_optically_centered)
    .show(ui, |ui| {
        render_custom_color_editor(ui, settings);
    });
}

fn render_theme_preset_selector(ui: &mut egui::Ui, settings: &mut SettingsService) {
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

    // Render Custom Themes if any
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

fn render_preset_group(
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
            // Colour swatch
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

fn render_custom_color_editor(ui: &mut egui::Ui, settings: &mut SettingsService) {
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
        egui::CollapsingHeader::new(
            egui::RichText::new(section_name.clone())
                .strong()
                .size(SECTION_HEADER_SIZE),
        )
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(SUBSECTION_SPACING);
            for (group_opt, settings_list) in grouped_settings {
                ui.add_space(SUBSECTION_SPACING);
                if let Some(group_name) = group_opt {
                    egui::CollapsingHeader::new(group_name.clone())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add_space(SUBSECTION_SPACING);
                            for def in settings_list {
                                changed |=
                                    render_color_row(ui, &mut new_colors, def.label, &def.prop);
                                ui.add_space(SUBSECTION_SPACING);
                            }
                        });
                } else {
                    for def in settings_list {
                        changed |= render_color_row(ui, &mut new_colors, def.label, &def.prop);
                        ui.add_space(SUBSECTION_SPACING);
                    }
                }
            }
        });
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

/// Renders a single colour picker row inside a Grid, returning `true` if changed.
pub(crate) enum ColorPropType {
    Rgb(fn(&ThemeColors) -> Rgb, fn(&mut ThemeColors, Rgb)),
    Rgba(fn(&ThemeColors) -> Rgba, fn(&mut ThemeColors, Rgba)),
}

fn render_color_row(
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

// ── Font tab ──────────────────────────────────────────────
fn render_font_family_selector(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let current = settings.settings().font.family.clone();
    let os_fonts = katana_platform::os_fonts::OsFontScanner::cached_fonts();

    // Persistent state IDs
    let open_id = egui::Id::new("font_selector_open");
    let search_id = egui::Id::new("font_search_query");

    let is_open: bool = ui.data(|d| d.get_temp(open_id)).unwrap_or(false);
    let mut query: String = ui
        .data(|d| d.get_temp::<String>(search_id))
        .unwrap_or_default();

    // ── Trigger button (shows current value, opens popup on click) ────────
    let button_resp =
        ui.add(egui::Button::new(&current).min_size(egui::vec2(FONT_FAMILY_COMBOBOX_WIDTH, 0.0)));
    if button_resp.clicked() {
        let new_state = !is_open;
        ui.data_mut(|d| d.insert_temp(open_id, new_state));
        if new_state {
            // Clear search when opening.
            ui.data_mut(|d| d.insert_temp(search_id, String::new()));
            query = String::new();
        }
    }

    // ── Popup with inline search field + filtered list ────────────────────
    egui::Popup::from_response(&button_resp)
        .open(is_open)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(FONT_FAMILY_COMBOBOX_WIDTH);

            // Search field at the top of the popup.
            let search_resp = ui.text_edit_singleline(&mut query);
            if search_resp.changed() {
                ui.data_mut(|d: &mut egui::util::IdTypeMap| {
                    d.insert_temp(search_id, query.clone())
                });
            }
            // Auto-focus whenever popup is open.
            search_resp.request_focus();

            ui.separator();

            let query_lower = query.to_lowercase();
            let mut selected: Option<String> = None;
            let mut close = false;

            egui::ScrollArea::vertical()
                .max_height(FONT_DROPDOWN_MAX_HEIGHT)
                .show(ui, |ui| {
                    let defaults = vec!["Proportional", "Monospace"];
                    for family in defaults {
                        if query_lower.is_empty() || family.to_lowercase().contains(&query_lower) {
                            let is_current = current == family;
                            if ui.selectable_label(is_current, family).clicked() {
                                selected = Some(family.to_string());
                                close = true;
                            }
                        }
                    }
                    ui.separator();
                    for (name, _) in os_fonts.iter() {
                        let name: &String = name;
                        if query_lower.is_empty() || name.to_lowercase().contains(&query_lower) {
                            let is_current = current == *name;
                            if ui.selectable_label(is_current, name.as_str()).clicked() {
                                selected = Some(name.clone());
                                close = true;
                            }
                        }
                    }
                });

            if let Some(new_font) = selected {
                settings.settings_mut().font.family = new_font;
                let _ = settings.save();
            }
            let should_close = close || ui.input(|i| i.key_pressed(egui::Key::Escape));
            if should_close {
                ui.data_mut(|d: &mut egui::util::IdTypeMap| {
                    d.insert_temp(open_id, false);
                    d.insert_temp(search_id, String::new());
                });
            }
        });
}

// ── Common UI Components ─────────────────────────────────────────────

pub(crate) fn add_styled_slider<'a>(ui: &mut egui::Ui, slider: egui::Slider<'a>) -> egui::Response {
    let selection_color = ui.visuals().selection.bg_fill;
    let saved_active_bg = ui.visuals().widgets.active.bg_fill;
    let saved_hovered_bg = ui.visuals().widgets.hovered.bg_fill;
    let saved_inactive_bg = ui.visuals().widgets.inactive.bg_fill;

    ui.visuals_mut().widgets.active.bg_fill = selection_color;
    ui.visuals_mut().widgets.hovered.bg_fill = selection_color;
    // Semi-transparent selection color for the unfilled portion of the rail.
    ui.visuals_mut().widgets.inactive.bg_fill = crate::theme_bridge::from_rgba_unmultiplied(
        selection_color.r(),
        selection_color.g(),
        selection_color.b(),
        SLIDER_RAIL_OPACITY,
    );

    // Add visible border to the slider handle/rail on all themes.
    let border_stroke = egui::Stroke::new(SLIDER_BORDER_WIDTH, selection_color);
    let saved_active_stroke = ui.visuals().widgets.active.bg_stroke;
    let saved_hovered_stroke = ui.visuals().widgets.hovered.bg_stroke;
    let saved_inactive_stroke = ui.visuals().widgets.inactive.bg_stroke;
    ui.visuals_mut().widgets.active.bg_stroke = border_stroke;
    ui.visuals_mut().widgets.hovered.bg_stroke = border_stroke;
    ui.visuals_mut().widgets.inactive.bg_stroke = border_stroke;

    let response = ui.add(slider);

    // Restore original visuals.
    ui.visuals_mut().widgets.active.bg_fill = saved_active_bg;
    ui.visuals_mut().widgets.hovered.bg_fill = saved_hovered_bg;
    ui.visuals_mut().widgets.inactive.bg_fill = saved_inactive_bg;
    ui.visuals_mut().widgets.active.bg_stroke = saved_active_stroke;
    ui.visuals_mut().widgets.hovered.bg_stroke = saved_hovered_stroke;
    ui.visuals_mut().widgets.inactive.bg_stroke = saved_inactive_stroke;

    response
}

// ── Font tab ─────────────────────────────────────────────────────────

fn render_font_tab(ui: &mut egui::Ui, settings: &mut SettingsService) {
    section_header(ui, &crate::i18n::get().settings.font.size);
    render_font_size_slider(ui, settings);
    ui.add_space(SECTION_SPACING);
    section_header(ui, &crate::i18n::get().settings.font.family);
    render_font_family_selector(ui, settings);
}

fn render_font_size_slider(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let mut size = settings.settings().clamped_font_size();
    let slider = egui::Slider::new(&mut size, MIN_FONT_SIZE..=MAX_FONT_SIZE)
        .step_by(FONT_SIZE_STEP)
        .suffix(" px");

    if add_styled_slider(ui, slider)
        .on_hover_text(crate::i18n::get().settings.font.size_slider_hint.clone())
        .changed()
    {
        settings.settings_mut().set_font_size(size);
        let _ = settings.save();
    }
}

// ── Layout tab ───────────────────────────────────────────────────────

fn render_layout_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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

fn render_toc_toggle(ui: &mut egui::Ui, settings: &mut SettingsService) {
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

fn render_toc_position_selector(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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

fn render_split_direction_selector(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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

fn render_pane_order_selector(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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

// ── Workspace tab ───────────────────────────────────────────────────

fn render_string_list_editor(ui: &mut egui::Ui, list: &mut Vec<String>) -> bool {
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

    // Option: The empty strings remain in the UI, enabling UX.
    // They are filtered implicitly in filesystem scanning.
    changed
}

fn render_workspace_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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
    ui.horizontal_wrapped(|ui| {
        let mut extensions = settings.settings().workspace.visible_extensions.clone();
        let mut changed_ext = false;

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
                        .position(crate::widgets::TogglePosition::Left)
                        .alignment(crate::widgets::ToggleAlignment::Attached(
                            SETTINGS_TOGGLE_SPACING,
                        )),
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
            ui.add_space(SETTINGS_TOGGLE_SPACING);
        }

        if changed_ext {
            settings.settings_mut().workspace.visible_extensions = extensions;
            let _ = settings.save();
        }
    });

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

fn render_updates_tab(
    ui: &mut egui::Ui,
    state: &mut crate::app_state::AppState,
) -> Option<AppAction> {
    let update_msgs = &crate::i18n::get().settings.updates;
    let settings = &mut state.config.settings;

    section_header(ui, &update_msgs.section_title);

    let ver_str = format!("Current version: v{}", env!("CARGO_PKG_VERSION"));
    ui.label(egui::RichText::new(ver_str).weak().size(HINT_FONT_SIZE));

    ui.horizontal(|ui| {
        ui.label(&update_msgs.interval);

        let mut interval = settings.settings().updates.interval;
        use katana_platform::settings::UpdateInterval;
        let mut changed = false;

        StyledComboBox::new(
            "update_interval",
            match interval {
                UpdateInterval::Never => update_msgs.never.as_str(),
                UpdateInterval::Daily => update_msgs.daily.as_str(),
                UpdateInterval::Weekly => update_msgs.weekly.as_str(),
                UpdateInterval::Monthly => update_msgs.monthly.as_str(),
            },
        )
        .show(ui, |ui| {
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Never, &update_msgs.never)
                .changed();
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Daily, &update_msgs.daily)
                .changed();
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Weekly, &update_msgs.weekly)
                .changed();
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Monthly, &update_msgs.monthly)
                .changed();
        });

        if changed {
            settings.settings_mut().updates.interval = interval;
            let _ = settings.save();
        }
    });

    ui.add_space(SUBSECTION_SPACING);

    if ui.button(&update_msgs.check_now).clicked() {
        return Some(AppAction::CheckForUpdates);
    }
    None
}

fn render_behavior_tab(
    ui: &mut egui::Ui,
    state: &mut crate::app_state::AppState,
) -> Option<AppAction> {
    let behavior_msgs = &crate::i18n::get().settings.behavior;
    let settings = &mut state.config.settings;

    // A1: Confirm before closing unsaved tabs
    let mut confirm = settings.settings().behavior.confirm_close_dirty_tab;
    if ui
        .add(
            crate::widgets::LabeledToggle::new(
                &behavior_msgs.confirm_close_dirty_tab,
                &mut confirm,
            )
            .position(crate::widgets::TogglePosition::Right)
            .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
        )
        .changed()
    {
        settings.settings_mut().behavior.confirm_close_dirty_tab = confirm;
        let _ = settings.save();
    }

    ui.add_space(SUBSECTION_SPACING);

    // B1: Scroll sync (persistent setting)
    let mut scroll_sync = settings.settings().behavior.scroll_sync_enabled;
    if ui
        .add(
            crate::widgets::LabeledToggle::new(&behavior_msgs.scroll_sync, &mut scroll_sync)
                .position(crate::widgets::TogglePosition::Right)
                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
        )
        .changed()
    {
        settings.settings_mut().behavior.scroll_sync_enabled = scroll_sync;
        let _ = settings.save();
    }

    ui.add_space(SUBSECTION_SPACING);

    ui.add_space(SUBSECTION_SPACING);

    // E1: Auto-save toggle
    let mut enabled = settings.settings().behavior.auto_save;
    if ui
        .add(
            crate::widgets::LabeledToggle::new(&behavior_msgs.auto_save, &mut enabled)
                .position(crate::widgets::TogglePosition::Right)
                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
        )
        .changed()
    {
        settings.settings_mut().behavior.auto_save = enabled;
        let _ = settings.save();
    }

    if enabled {
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let interval = settings.settings().behavior.auto_save_interval_secs;
        ui.label(&behavior_msgs.auto_save_interval);

        let original_width = ui.spacing().slider_width;
        const SETTINGS_SLIDER_WIDTH: f32 = 300.0;
        ui.spacing_mut().slider_width = SETTINGS_SLIDER_WIDTH;

        ui.horizontal(|ui| {
            let mut display_val = interval;

            let slider = egui::Slider::new(
                &mut display_val,
                AUTO_SAVE_INTERVAL_MIN..=AUTO_SAVE_INTERVAL_MAX,
            )
            .show_value(false) // Text is displayed separately
            .step_by(AUTO_SAVE_INTERVAL_STEP)
            .min_decimals(1)
            .max_decimals(1)
            .logarithmic(true)
            .clamping(egui::SliderClamping::Always);

            let slider_response = add_styled_slider(ui, slider);

            // Place text portion as a separate component alongside
            let drag_response = ui.add(
                egui::DragValue::new(&mut display_val)
                    .speed(AUTO_SAVE_INTERVAL_STEP)
                    .suffix("s")
                    .max_decimals(1)
                    .range(AUTO_SAVE_INTERVAL_MIN..=AUTO_SAVE_INTERVAL_MAX),
            );

            if slider_response.changed() || drag_response.changed() {
                // Save when DragValue or Slider changes
                settings.settings_mut().behavior.auto_save_interval_secs = display_val;
                let _ = settings.save();
            }
        });

        ui.spacing_mut().slider_width = original_width;
    }

    ui.add_space(SUBSECTION_SPACING);

    if ui.button(&behavior_msgs.clear_http_cache).clicked() {
        return Some(AppAction::ClearAllCaches);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_save_interval_slider_config_invariants() {
        // Strict UT for the 0.1 step requirement.
        assert_eq!(
            AUTO_SAVE_INTERVAL_STEP, 0.1,
            "The auto-save slider MUST increment/decrement by exactly 0.1 seconds \
             to satisfy the UI precision requirements."
        );
        assert_eq!(
            AUTO_SAVE_INTERVAL_MIN, 0.0,
            "The minimum auto-save interval MUST be 0.0 (off or immediate)."
        );
        assert_eq!(
            AUTO_SAVE_INTERVAL_MAX, 300.0,
            "The maximum auto-save interval MUST be 300.0 seconds."
        );
    }
}
