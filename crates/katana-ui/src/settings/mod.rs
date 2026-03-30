
use crate::app_state::{AppAction, SettingsTab};
use crate::preview_pane::PreviewPane;


pub(crate) const SETTINGS_WINDOW_DEFAULT_WIDTH: f32 = 1000.0;
pub(crate) const SETTINGS_WINDOW_DEFAULT_HEIGHT: f32 = 500.0;

pub(crate) const SETTINGS_SIDE_PANEL_DEFAULT_WIDTH: f32 = 200.0;

pub(crate) const SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH: f32 = 350.0;

pub(crate) const SETTINGS_HEADER_FONT_SIZE: f32 = 14.0;
pub(crate) const SETTINGS_GROUP_SPACING: f32 = 8.0;
pub(crate) const SETTINGS_TOGGLE_SPACING: f32 = 8.0;

pub(crate) const AUTO_SAVE_INTERVAL_MIN: f64 = 0.0;
pub(crate) const AUTO_SAVE_INTERVAL_MAX: f64 = 300.0;
pub(crate) const AUTO_SAVE_INTERVAL_STEP: f64 = 0.1;


pub(crate) const SECTION_SPACING: f32 = 12.0;
pub(crate) const SUBSECTION_SPACING: f32 = 6.0;
pub(crate) const INNER_MARGIN: f32 = 12.0;

pub(crate) const FONT_SIZE_STEP: f64 = 1.0;
pub(crate) const LAYOUT_SELECTOR_SPACING: f32 = 4.0;
pub(crate) const PRESET_SWATCH_SIZE: f32 = 14.0;
pub(crate) const COLOR_GRID_LABEL_WIDTH: f32 = 130.0;
pub(crate) const SECTION_HEADER_SIZE: f32 = 14.0;
pub(crate) const SECTION_HEADER_MARGIN: f32 = 4.0;
pub(crate) const SWATCH_CORNER_DIVISOR: f32 = 4.0;
pub(crate) const FONT_FAMILY_COMBOBOX_WIDTH: f32 = 200.0;
pub(crate) const FONT_DROPDOWN_MAX_HEIGHT: f32 = 200.0;
pub(crate) const SLIDER_RAIL_OPACITY: u8 = 80;
pub(crate) const SLIDER_BORDER_WIDTH: f32 = 1.0;
pub(crate) const HINT_FONT_SIZE: f32 = 10.0;


pub(crate) const SAMPLE_MARKDOWN: &str = concat!(
    r#"# Heading 1

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

"#,
    "Secondary text and [a link](https://example.com) for reference.\n"
);


pub mod tabs;
pub(crate) use self::tabs::behavior::*;
pub(crate) use self::tabs::font::*;
pub(crate) use self::tabs::layout::*;
pub(crate) use self::tabs::theme::*;
pub(crate) use self::tabs::updates::*;
pub(crate) use self::tabs::workspace::*;

pub(crate) struct SettingsWindow<'a> {
    pub state: &'a mut crate::app_state::AppState,
    pub preview_pane: &'a mut PreviewPane,
}

impl<'a> SettingsWindow<'a> {
    pub fn new(
        state: &'a mut crate::app_state::AppState,
        preview_pane: &'a mut PreviewPane,
    ) -> Self {
        Self {
            state,
            preview_pane,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> Option<AppAction> {
        let state = self.state;
        let preview_pane = self.preview_pane;

        if !state.layout.show_settings {
            return None;
        }

        let mut triggered_action: Option<AppAction> = None;

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
                        ui.horizontal(|ui| {
                            const TAB_SPACING: f32 = 4.0;
                            ui.add_space(TAB_SPACING);
                            if ui
                                .add(egui::Button::image_and_text(
                                    crate::Icon::ExpandAll
                                        .ui_image(ui, crate::icon::IconSize::Small),
                                    "",
                                ))
                                .on_hover_text(crate::i18n::get().action.expand_all.clone())
                                .clicked()
                            {
                                state.config.settings_tree_force_open = Some(true);
                            }
                            if ui
                                .add(egui::Button::image_and_text(
                                    crate::Icon::CollapseAll
                                        .ui_image(ui, crate::icon::IconSize::Small),
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
                            egui::Frame::NONE
                                .inner_margin(INNER_MARGIN)
                                .show(ui, |ui| match state.config.active_settings_tab {
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
                                });
                        });
                });

                if state.config.settings_tree_force_open.is_some() {
                    state.config.settings_tree_force_open = None;
                }
            });
        state.layout.show_settings = open;
        triggered_action
    }
}

pub(crate) fn render_settings_tree(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    let settings_msgs = &crate::i18n::get().settings;

    let appearance_key = "group_appearance";
    let title = settings_msgs
        .tabs
        .iter()
        .find(|t| t.key == appearance_key)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Appearance".to_string());

    crate::widgets::Accordion::new(
        "settings_grp_appearance",
        egui::RichText::new(title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
        |ui| {
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
        },
    )
    .default_open(true)
    .open(state.config.settings_tree_force_open)
    .show(ui);

    ui.add_space(SETTINGS_GROUP_SPACING);

    let system_key = "group_system";
    let title = settings_msgs
        .tabs
        .iter()
        .find(|t| t.key == system_key)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "System".to_string());

    crate::widgets::Accordion::new(
        "settings_grp_system",
        egui::RichText::new(title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
        |ui| {
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
        },
    )
    .default_open(true)
    .open(state.config.settings_tree_force_open)
    .show(ui);
}


pub(crate) fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(SECTION_HEADER_MARGIN);
    ui.label(egui::RichText::new(text).size(SECTION_HEADER_SIZE).strong());
    ui.add_space(SECTION_HEADER_MARGIN);
    ui.separator();
    ui.add_space(SUBSECTION_SPACING);
}


pub(crate) fn add_styled_slider<'a>(ui: &mut egui::Ui, slider: egui::Slider<'a>) -> egui::Response {
    let selection_color = ui.visuals().selection.bg_fill;
    let saved_active_bg = ui.visuals().widgets.active.bg_fill;
    let saved_hovered_bg = ui.visuals().widgets.hovered.bg_fill;
    let saved_inactive_bg = ui.visuals().widgets.inactive.bg_fill;

    ui.visuals_mut().widgets.active.bg_fill = selection_color;
    ui.visuals_mut().widgets.hovered.bg_fill = selection_color;
    ui.visuals_mut().widgets.inactive.bg_fill = crate::theme_bridge::from_rgba_unmultiplied(
        selection_color.r(),
        selection_color.g(),
        selection_color.b(),
        SLIDER_RAIL_OPACITY,
    );

    let border_stroke = egui::Stroke::new(SLIDER_BORDER_WIDTH, selection_color);
    let saved_active_stroke = ui.visuals().widgets.active.bg_stroke;
    let saved_hovered_stroke = ui.visuals().widgets.hovered.bg_stroke;
    let saved_inactive_stroke = ui.visuals().widgets.inactive.bg_stroke;
    ui.visuals_mut().widgets.active.bg_stroke = border_stroke;
    ui.visuals_mut().widgets.hovered.bg_stroke = border_stroke;
    ui.visuals_mut().widgets.inactive.bg_stroke = border_stroke;

    let response = ui.add(slider);

    ui.visuals_mut().widgets.active.bg_fill = saved_active_bg;
    ui.visuals_mut().widgets.hovered.bg_fill = saved_hovered_bg;
    ui.visuals_mut().widgets.inactive.bg_fill = saved_inactive_bg;
    ui.visuals_mut().widgets.active.bg_stroke = saved_active_stroke;
    ui.visuals_mut().widgets.hovered.bg_stroke = saved_hovered_stroke;
    ui.visuals_mut().widgets.inactive.bg_stroke = saved_inactive_stroke;

    response
}
