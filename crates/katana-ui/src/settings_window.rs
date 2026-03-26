//! Settings window rendered as an `egui::Window` overlay.
//!
//! The window uses a split layout:
//!   - **Left pane**: Tab bar + settings controls (scrollable)
//!   - **Right pane**: Live markdown preview using `PreviewPane` (scrollable independently)

use crate::app_state::{AppAction, SettingsTab};
use crate::preview_pane::PreviewPane;
use crate::theme_bridge;
use katana_platform::settings::{SettingsService, MAX_FONT_SIZE, MIN_FONT_SIZE};
use katana_platform::theme::{Rgb, ThemeColors, ThemeMode, ThemePreset};
use katana_platform::{PaneOrder, SplitDirection};

// ── Window layout constants ──────────────────────────────────────────

const SETTINGS_WINDOW_DEFAULT_WIDTH: f32 = 900.0;
const SETTINGS_WINDOW_DEFAULT_HEIGHT: f32 = 500.0;

const SETTINGS_SIDE_PANEL_DEFAULT_WIDTH: f32 = 200.0;

const SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH: f32 = 350.0;

const SETTINGS_HEADER_FONT_SIZE: f32 = 14.0;
const SETTINGS_GROUP_SPACING: f32 = 8.0;

// ── Spacing & sizing constants ───────────────────────────────────────

const SECTION_SPACING: f32 = 12.0;
const SUBSECTION_SPACING: f32 = 6.0;
const INNER_MARGIN: f32 = 12.0;
const COLOUR_CHANNEL_MAX: f32 = 255.0;
const FONT_SIZE_STEP: f64 = 1.0;
/// Spacing between layout selectors (split direction / pane order) within the Layout tab.
const LAYOUT_SELECTOR_SPACING: f32 = 4.0;
const PRESET_SWATCH_SIZE: f32 = 14.0;
const COLOR_GRID_LABEL_WIDTH: f32 = 130.0;
const SECTION_HEADER_SIZE: f32 = 14.0;
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
    if !state.show_settings {
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

    let mut open = state.show_settings;
    egui::Window::new(crate::i18n::get().settings.title.clone())
        .open(&mut open)
        .fixed_size(egui::vec2(
            SETTINGS_WINDOW_DEFAULT_WIDTH,
            SETTINGS_WINDOW_DEFAULT_HEIGHT,
        ))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
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
                            state.settings_tree_force_open = Some(true);
                        }
                        if ui
                            .add(egui::Button::image_and_text(
                                crate::Icon::CollapseAll.ui_image(ui, crate::icon::IconSize::Small),
                                "",
                            ))
                            .on_hover_text(crate::i18n::get().action.collapse_all.clone())
                            .clicked()
                        {
                            state.settings_tree_force_open = Some(false);
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
                state.active_settings_tab,
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
                let title = match state.active_settings_tab {
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
                };

                section_header(ui, title);
                ui.add_space(SUBSECTION_SPACING);

                egui::ScrollArea::vertical()
                    .id_salt("settings_form_scroll")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        egui::Frame::NONE.inner_margin(INNER_MARGIN).show(ui, |ui| {
                            match state.active_settings_tab {
                                SettingsTab::Theme => render_theme_tab(ui, &mut state.settings),
                                SettingsTab::Font => render_font_tab(ui, &mut state.settings),
                                SettingsTab::Layout => render_layout_tab(ui, state),
                                SettingsTab::Workspace => render_workspace_tab(ui, state),
                                SettingsTab::Updates => {
                                    if let Some(action) = render_updates_tab(ui, state) {
                                        triggered_action = Some(action);
                                    }
                                }
                            }
                        });
                    });
            });

            // Clear the force open flag after rendering the tree once
            if state.settings_tree_force_open.is_some() {
                state.settings_tree_force_open = None;
            }
        });
    state.show_settings = open;
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
        .unwrap_or_else(|| "外観".to_string());

    let mut appearance_header = egui::CollapsingHeader::new(
        egui::RichText::new(title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
    )
    .default_open(true)
    .id_salt("settings_grp_appearance")
    .icon(egui_commonmark::ui_components::centering::AccordionIcon::paint_optically_centered);

    if let Some(force_open) = state.settings_tree_force_open {
        appearance_header = appearance_header.open(Some(force_open));
    }

    appearance_header.show(ui, |ui| {
        let theme_selected = state.active_settings_tab == SettingsTab::Theme;
        if ui
            .selectable_label(theme_selected, settings_msgs.tab_name("theme"))
            .clicked()
        {
            state.active_settings_tab = SettingsTab::Theme;
        }

        let font_selected = state.active_settings_tab == SettingsTab::Font;
        if ui
            .selectable_label(font_selected, settings_msgs.tab_name("font"))
            .clicked()
        {
            state.active_settings_tab = SettingsTab::Font;
        }

        let layout_selected = state.active_settings_tab == SettingsTab::Layout;
        if ui
            .selectable_label(layout_selected, settings_msgs.tab_name("layout"))
            .clicked()
        {
            state.active_settings_tab = SettingsTab::Layout;
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
        .unwrap_or_else(|| "システム".to_string());

    let mut system_header = egui::CollapsingHeader::new(
        egui::RichText::new(title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
    )
    .default_open(true)
    .id_salt("settings_grp_system")
    .icon(egui_commonmark::ui_components::centering::AccordionIcon::paint_optically_centered);

    if let Some(force_open) = state.settings_tree_force_open {
        system_header = system_header.open(Some(force_open));
    }

    system_header.show(ui, |ui| {
        let workspace_selected = state.active_settings_tab == SettingsTab::Workspace;
        if ui
            .selectable_label(workspace_selected, settings_msgs.tab_name("workspace"))
            .clicked()
        {
            state.active_settings_tab = SettingsTab::Workspace;
        }

        let updates_selected = state.active_settings_tab == SettingsTab::Updates;
        if ui
            .selectable_label(updates_selected, settings_msgs.tab_name("updates"))
            .clicked()
        {
            state.active_settings_tab = SettingsTab::Updates;
        }
    });
}

// ── Section header helper ────────────────────────────────────────────

fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).strong().size(SECTION_HEADER_SIZE));
    ui.add_space(SUBSECTION_SPACING);
}

// ── Theme tab ────────────────────────────────────────────────────────

fn render_theme_tab(ui: &mut egui::Ui, settings: &mut SettingsService) {
    render_theme_preset_selector(ui, settings);
    ui.add_space(SECTION_SPACING);

    egui::CollapsingHeader::new(
        egui::RichText::new(crate::i18n::get().settings.theme.custom_colors.clone())
            .strong()
            .size(SECTION_HEADER_SIZE),
    )
    .default_open(settings.settings().theme.custom_color_overrides.is_some())
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
    let all_presets = ThemePreset::all();
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

    ui.add_space(SUBSECTION_SPACING);

    let toggle_text = if show_more {
        "Show less..."
    } else {
        "Show more..."
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
        let bg_color = theme_bridge::rgb_to_color32(colors.background);
        let accent_color = theme_bridge::rgb_to_color32(colors.accent);

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

            if ui
                .selectable_label(is_selected, preset.display_name())
                .clicked()
                && !is_selected
            {
                settings.settings_mut().theme.preset = (*preset).clone();
                settings.settings_mut().theme.custom_color_overrides = None;
                let _ = settings.save();
            }
        });
    }
}

fn render_custom_color_editor(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let current_colors = settings.settings().effective_theme_colors();

    let color_fields: Vec<(&str, &String, Rgb)> = vec![
        (
            "settings_color_background",
            &crate::i18n::get().settings.color.background,
            current_colors.background,
        ),
        (
            "settings_color_panel_background",
            &crate::i18n::get().settings.color.panel_background,
            current_colors.panel_background,
        ),
        (
            "settings_color_text",
            &crate::i18n::get().settings.color.text,
            current_colors.text,
        ),
        (
            "settings_color_text_secondary",
            &crate::i18n::get().settings.color.text_secondary,
            current_colors.text_secondary,
        ),
        (
            "settings_color_accent",
            &crate::i18n::get().settings.color.accent,
            current_colors.accent,
        ),
        (
            "settings_color_border",
            &crate::i18n::get().settings.color.border,
            current_colors.border,
        ),
        (
            "settings_color_selection",
            &crate::i18n::get().settings.color.selection,
            current_colors.selection,
        ),
        (
            "settings_color_code_background",
            &crate::i18n::get().settings.color.code_background,
            current_colors.code_background,
        ),
        (
            "settings_color_preview_background",
            &crate::i18n::get().settings.color.preview_background,
            current_colors.preview_background,
        ),
    ];

    let mut changed = false;
    let mut new_colors = current_colors.clone();

    egui::Grid::new("color_editor_grid")
        .num_columns(2)
        .spacing(egui::vec2(SECTION_SPACING, SUBSECTION_SPACING))
        .show(ui, |ui| {
            for (key, label, original_rgb) in &color_fields {
                ui.add_sized(
                    egui::vec2(COLOR_GRID_LABEL_WIDTH, 0.0),
                    egui::Label::new(*label),
                );
                let r = f32::from(original_rgb.r) / COLOUR_CHANNEL_MAX;
                let g = f32::from(original_rgb.g) / COLOUR_CHANNEL_MAX;
                let b = f32::from(original_rgb.b) / COLOUR_CHANNEL_MAX;
                let mut color_arr = std::array::from_fn(|i| {
                    if i == 0 {
                        r
                    } else if i == 1 {
                        g
                    } else {
                        b
                    }
                });
                if ui.color_edit_button_rgb(&mut color_arr).changed() {
                    // Colour channel values are clamped to [0.0, 1.0] by the colour picker;
                    // multiplying by 255 always yields a valid u8.
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let new_rgb = Rgb {
                        r: (color_arr[0] * COLOUR_CHANNEL_MAX) as u8,
                        g: (color_arr[1] * COLOUR_CHANNEL_MAX) as u8,
                        b: (color_arr[2] * COLOUR_CHANNEL_MAX) as u8,
                    };
                    apply_color_to_theme(&mut new_colors, key, new_rgb);
                    changed = true;
                }
                ui.end_row();
            }
        });

    if changed {
        settings.settings_mut().theme.custom_color_overrides = Some(new_colors);
        let _ = settings.save();
    }

    ui.add_space(SUBSECTION_SPACING);
    if settings.settings().theme.custom_color_overrides.is_some()
        && ui
            .button(crate::i18n::get().settings.theme.reset_custom.clone())
            .clicked()
    {
        settings.settings_mut().theme.custom_color_overrides = None;
        let _ = settings.save();
    }
}

fn apply_color_to_theme(colors: &mut ThemeColors, field_key: &str, rgb: Rgb) {
    match field_key {
        "settings_color_background" => colors.background = rgb,
        "settings_color_panel_background" => colors.panel_background = rgb,
        "settings_color_text" => colors.text = rgb,
        "settings_color_text_secondary" => colors.text_secondary = rgb,
        "settings_color_accent" => colors.accent = rgb,
        "settings_color_border" => colors.border = rgb,
        "settings_color_selection" => colors.selection = rgb,
        "settings_color_code_background" => colors.code_background = rgb,
        "settings_color_preview_background" => colors.preview_background = rgb,
        _ => {}
    }
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
    ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(
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
    render_toc_toggle(ui, &mut state.settings);
    ui.add_space(SECTION_SPACING);
    render_toc_position_selector(ui, state);
    ui.add_space(SECTION_SPACING);
    render_split_direction_selector(ui, state);
    ui.add_space(LAYOUT_SELECTOR_SPACING);
    render_pane_order_selector(ui, state);
}

fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        const TOGGLE_RADIUS_RATIO: f32 = 0.5;
        let radius = TOGGLE_RADIUS_RATIO * rect.height();
        ui.painter().rect(
            rect,
            radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        const TOGGLE_CIRCLE_RATIO: f32 = 0.75;
        ui.painter().circle(
            center,
            TOGGLE_CIRCLE_RATIO * radius,
            visuals.bg_fill,
            visuals.fg_stroke,
        );
    }

    response
}

fn render_toc_toggle(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let mut toc_visible = settings.settings().layout.toc_visible;
    ui.horizontal(|ui| {
        ui.label(crate::i18n::get().settings.toc_visible.clone());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if toggle_ui(ui, &mut toc_visible).changed() {
                settings.settings_mut().layout.toc_visible = toc_visible;
                let _ = settings.save();
            }
        });
    });
}

fn render_toc_position_selector(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    if !state.settings.settings().layout.toc_visible {
        return;
    }

    use katana_platform::settings::TocPosition;

    ui.label(crate::i18n::get().settings.layout.toc_position.clone());
    ui.horizontal(|ui| {
        let current = state.settings.settings().layout.toc_position;
        if ui
            .selectable_label(
                current == TocPosition::Left,
                crate::i18n::get().settings.layout.left.clone(),
            )
            .clicked()
            && current != TocPosition::Left
        {
            state.settings.settings_mut().layout.toc_position = TocPosition::Left;
            let _ = state.settings.save();
        }
        if ui
            .selectable_label(
                current == TocPosition::Right,
                crate::i18n::get().settings.layout.right.clone(),
            )
            .clicked()
            && current != TocPosition::Right
        {
            state.settings.settings_mut().layout.toc_position = TocPosition::Right;
            let _ = state.settings.save();
        }
    });
}

fn render_split_direction_selector(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    ui.label(crate::i18n::get().settings.layout.split_direction.clone());
    ui.horizontal(|ui| {
        let current = state.settings.settings().layout.split_direction;
        if ui
            .selectable_label(
                current == SplitDirection::Horizontal,
                crate::i18n::get().settings.layout.horizontal.clone(),
            )
            .clicked()
            && current != SplitDirection::Horizontal
        {
            state.settings.settings_mut().layout.split_direction = SplitDirection::Horizontal;
            let _ = state.settings.save();
        }
        if ui
            .selectable_label(
                current == SplitDirection::Vertical,
                crate::i18n::get().settings.layout.vertical.clone(),
            )
            .clicked()
            && current != SplitDirection::Vertical
        {
            state.settings.settings_mut().layout.split_direction = SplitDirection::Vertical;
            let _ = state.settings.save();
        }
    });
}

fn render_pane_order_selector(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    ui.label(crate::i18n::get().settings.layout.pane_order.clone());
    ui.horizontal(|ui| {
        let current = state.settings.settings().layout.pane_order;
        if ui
            .selectable_label(
                current == PaneOrder::EditorFirst,
                crate::i18n::get().settings.layout.editor_first.clone(),
            )
            .clicked()
            && current != PaneOrder::EditorFirst
        {
            state.settings.settings_mut().layout.pane_order = PaneOrder::EditorFirst;
            let _ = state.settings.save();
        }
        if ui
            .selectable_label(
                current == PaneOrder::PreviewFirst,
                crate::i18n::get().settings.layout.preview_first.clone(),
            )
            .clicked()
            && current != PaneOrder::PreviewFirst
        {
            state.settings.settings_mut().layout.pane_order = PaneOrder::PreviewFirst;
            let _ = state.settings.save();
        }
    });
}

// ── Workspace tab ───────────────────────────────────────────────────

fn render_workspace_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    let workspace_msgs = &crate::i18n::get().settings.workspace;
    let settings = &mut state.settings;

    section_header(ui, &workspace_msgs.max_depth);
    let mut max_depth = settings.settings().workspace.max_depth;
    let slider = egui::Slider::new(&mut max_depth, 1..=100);
    if add_styled_slider(ui, slider).changed() {
        settings.settings_mut().workspace.max_depth = max_depth;
        let _ = settings.save();
    }

    ui.add_space(SECTION_SPACING);

    section_header(ui, &workspace_msgs.ignored_directories);
    let mut ignored = settings.settings().workspace.ignored_directories.join(", ");
    if ui.text_edit_singleline(&mut ignored).changed() {
        let list: Vec<String> = ignored
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        settings.settings_mut().workspace.ignored_directories = list;
        let _ = settings.save();
    }
    ui.label(
        egui::RichText::new(&workspace_msgs.ignored_directories_hint)
            .weak()
            .size(HINT_FONT_SIZE),
    );
}

fn render_updates_tab(
    ui: &mut egui::Ui,
    state: &mut crate::app_state::AppState,
) -> Option<AppAction> {
    let update_msgs = &crate::i18n::get().settings.updates;
    let settings = &mut state.settings;

    section_header(ui, &update_msgs.section_title);

    let ver_str = format!("Current version: v{}", env!("CARGO_PKG_VERSION"));
    ui.label(egui::RichText::new(ver_str).weak().size(HINT_FONT_SIZE));

    ui.horizontal(|ui| {
        ui.label(&update_msgs.interval);

        let mut interval = settings.settings().updates.interval;
        use katana_platform::settings::UpdateInterval;
        let mut changed = false;

        egui::ComboBox::from_id_salt("update_interval")
            .selected_text(match interval {
                UpdateInterval::Never => &update_msgs.never,
                UpdateInterval::Daily => &update_msgs.daily,
                UpdateInterval::Weekly => &update_msgs.weekly,
                UpdateInterval::Monthly => &update_msgs.monthly,
            })
            .show_ui(ui, |ui| {
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
