//! Settings window rendered as an `egui::Window` overlay.
//!
//! The window uses a split layout:
//!   - **Left pane**: Tab bar + settings controls (scrollable)
//!   - **Right pane**: Live markdown preview using `PreviewPane` (scrollable independently)

use crate::app_state::SettingsTab;
use crate::i18n;
use crate::preview_pane::PreviewPane;
use crate::theme_bridge;
use katana_platform::settings::{SettingsService, MAX_FONT_SIZE, MIN_FONT_SIZE};
use katana_platform::theme::{Rgb, ThemeColors, ThemeMode, ThemePreset};

// ── Window layout constants ──────────────────────────────────────────

const SETTINGS_WINDOW_DEFAULT_WIDTH: f32 = 720.0;
const SETTINGS_WINDOW_DEFAULT_HEIGHT: f32 = 520.0;

/// Fraction of the window width allocated to the settings (left) pane.
const SETTINGS_PANE_RATIO: f32 = 0.45;

// ── Spacing & sizing constants ───────────────────────────────────────

const SECTION_SPACING: f32 = 12.0;
const SUBSECTION_SPACING: f32 = 6.0;
const INNER_MARGIN: f32 = 12.0;
const COLOUR_CHANNEL_MAX: f32 = 255.0;
const FONT_SIZE_STEP: f64 = 1.0;
const TAB_UNDERLINE_HEIGHT: f32 = 2.0;
const TAB_BUTTON_PADDING_X: f32 = 16.0;
const PRESET_SWATCH_SIZE: f32 = 14.0;
const COLOR_GRID_LABEL_WIDTH: f32 = 130.0;
const SECTION_HEADER_SIZE: f32 = 14.0;
const SWATCH_CORNER_DIVISOR: f32 = 4.0;
const FONT_FAMILY_COMBOBOX_WIDTH: f32 = 200.0;

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

/// Render the settings window.
pub(crate) fn render_settings_window(
    ctx: &egui::Context,
    show: &mut bool,
    active_tab: &mut SettingsTab,
    settings: &mut SettingsService,
    preview_pane: &mut PreviewPane,
) {
    if !*show {
        return;
    }

    // Ensure preview pane has content loaded
    if preview_pane.sections.is_empty() {
        preview_pane.update_markdown_sections(
            SAMPLE_MARKDOWN,
            std::path::Path::new("/settings-preview.md"),
        );
    }

    let mut open = *show;
    egui::Window::new(i18n::t("settings_title"))
        .open(&mut open)
        .default_size(egui::vec2(
            SETTINGS_WINDOW_DEFAULT_WIDTH,
            SETTINGS_WINDOW_DEFAULT_HEIGHT,
        ))
        .collapsible(false)
        .resizable(true)
        .show(ctx, |ui| {
            let total_width = ui.available_width();
            let left_width = total_width * SETTINGS_PANE_RATIO;
            let available_height = ui.available_height();

            ui.horizontal(|ui| {
                // ── Left pane: settings controls ──
                ui.vertical(|ui| {
                    ui.set_width(left_width);
                    ui.set_min_height(available_height);

                    render_tab_bar(ui, active_tab);
                    ui.add_space(SUBSECTION_SPACING);

                    egui::ScrollArea::vertical()
                        .id_salt("settings_left_scroll")
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            egui::Frame::NONE.inner_margin(INNER_MARGIN).show(ui, |ui| {
                                match *active_tab {
                                    SettingsTab::Theme => render_theme_tab(ui, settings),
                                    SettingsTab::Font => render_font_tab(ui, settings),
                                    SettingsTab::Layout => render_layout_tab(ui, settings),
                                }
                            });
                        });
                });

                // ── Vertical divider ──
                ui.add(egui::Separator::default().vertical());

                // ── Right pane: live markdown preview ──
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(available_height);

                    section_header(ui, "settings_preview_title");
                    preview_pane.show(ui);
                });
            });
        });
    *show = open;
}

// ── Tab bar (styled underline tabs) ──────────────────────────────────

fn render_tab_bar(ui: &mut egui::Ui, active_tab: &mut SettingsTab) {
    let tabs = [
        (SettingsTab::Theme, "settings_tab_theme"),
        (SettingsTab::Font, "settings_tab_font"),
        (SettingsTab::Layout, "settings_tab_layout"),
    ];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let tab_width = ui.available_width() / tabs.len() as f32;

        for (tab, key) in &tabs {
            let label = i18n::t(key);
            let is_active = *active_tab == *tab;

            let text = if is_active {
                egui::RichText::new(label).strong()
            } else {
                egui::RichText::new(label)
            };

            let button = egui::Button::new(text).frame(false);
            let tab_height = ui.text_style_height(&egui::TextStyle::Body) + SUBSECTION_SPACING;
            let resp = ui.add_sized(egui::vec2(tab_width, tab_height), button);

            // Active tab underline
            if is_active {
                let rect = resp.rect;
                let accent = ui.visuals().selection.bg_fill;
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(
                            rect.min.x + TAB_BUTTON_PADDING_X,
                            rect.max.y - TAB_UNDERLINE_HEIGHT,
                        ),
                        egui::vec2(
                            rect.width() - TAB_BUTTON_PADDING_X * 2.0,
                            TAB_UNDERLINE_HEIGHT,
                        ),
                    ),
                    TAB_UNDERLINE_HEIGHT / 2.0,
                    accent,
                );
            }

            if resp.clicked() {
                *active_tab = *tab;
            }
        }
    });
}

// ── Section header helper ────────────────────────────────────────────

fn section_header(ui: &mut egui::Ui, key: &str) {
    ui.label(
        egui::RichText::new(i18n::t(key))
            .strong()
            .size(SECTION_HEADER_SIZE),
    );
    ui.add_space(SUBSECTION_SPACING);
}

// ── Theme tab ────────────────────────────────────────────────────────

fn render_theme_tab(ui: &mut egui::Ui, settings: &mut SettingsService) {
    render_theme_preset_selector(ui, settings);
    ui.add_space(SECTION_SPACING);

    egui::CollapsingHeader::new(
        egui::RichText::new(i18n::t("settings_theme_custom_colors"))
            .strong()
            .size(SECTION_HEADER_SIZE),
    )
    .default_open(settings.settings().custom_color_overrides.is_some())
    .show(ui, |ui| {
        render_custom_color_editor(ui, settings);
    });
}

fn render_theme_preset_selector(ui: &mut egui::Ui, settings: &mut SettingsService) {
    section_header(ui, "settings_theme_preset");

    ui.label(egui::RichText::new(i18n::t("settings_theme_dark_section")).weak());
    let dark_presets: Vec<&ThemePreset> = ThemePreset::all()
        .iter()
        .filter(|it| it.colors().mode == ThemeMode::Dark)
        .collect();
    render_preset_group(ui, settings, &dark_presets);

    ui.add_space(SECTION_SPACING);

    ui.label(egui::RichText::new(i18n::t("settings_theme_light_section")).weak());
    let light_presets: Vec<&ThemePreset> = ThemePreset::all()
        .iter()
        .filter(|it| it.colors().mode == ThemeMode::Light)
        .collect();
    render_preset_group(ui, settings, &light_presets);
}

fn render_preset_group(
    ui: &mut egui::Ui,
    settings: &mut SettingsService,
    presets: &[&ThemePreset],
) {
    for preset in presets {
        let is_selected = settings.settings().selected_preset == **preset;
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
                settings.settings_mut().selected_preset = (*preset).clone();
                settings.settings_mut().custom_color_overrides = None;
                let _ = settings.save();
            }
        });
    }
}

fn render_custom_color_editor(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let current_colors = settings.settings().effective_theme_colors();

    let color_fields: Vec<(&str, Rgb)> = vec![
        ("settings_color_background", current_colors.background),
        (
            "settings_color_panel_background",
            current_colors.panel_background,
        ),
        ("settings_color_text", current_colors.text),
        (
            "settings_color_text_secondary",
            current_colors.text_secondary,
        ),
        ("settings_color_accent", current_colors.accent),
        ("settings_color_border", current_colors.border),
        ("settings_color_selection", current_colors.selection),
        (
            "settings_color_code_background",
            current_colors.code_background,
        ),
        (
            "settings_color_preview_background",
            current_colors.preview_background,
        ),
    ];

    let mut changed = false;
    let mut new_colors = current_colors.clone();

    egui::Grid::new("color_editor_grid")
        .num_columns(2)
        .spacing([SECTION_SPACING, SUBSECTION_SPACING])
        .show(ui, |ui| {
            for (key, original_rgb) in &color_fields {
                ui.add_sized(
                    egui::vec2(COLOR_GRID_LABEL_WIDTH, 0.0),
                    egui::Label::new(i18n::t(key)),
                );
                let mut color_arr = [
                    f32::from(original_rgb.r) / COLOUR_CHANNEL_MAX,
                    f32::from(original_rgb.g) / COLOUR_CHANNEL_MAX,
                    f32::from(original_rgb.b) / COLOUR_CHANNEL_MAX,
                ];
                if ui.color_edit_button_rgb(&mut color_arr).changed() {
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
        settings.settings_mut().custom_color_overrides = Some(new_colors);
        let _ = settings.save();
    }

    ui.add_space(SUBSECTION_SPACING);
    if settings.settings().custom_color_overrides.is_some()
        && ui.button(i18n::t("settings_theme_reset_custom")).clicked()
    {
        settings.settings_mut().custom_color_overrides = None;
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

// ── Font tab ─────────────────────────────────────────────────────────

fn render_font_tab(ui: &mut egui::Ui, settings: &mut SettingsService) {
    section_header(ui, "settings_font_size");
    render_font_size_slider(ui, settings);
    ui.add_space(SECTION_SPACING);
    section_header(ui, "settings_font_family");
    render_font_family_selector(ui, settings);
}

fn render_font_size_slider(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let mut size = settings.settings().clamped_font_size();
    let slider = egui::Slider::new(&mut size, MIN_FONT_SIZE..=MAX_FONT_SIZE)
        .step_by(FONT_SIZE_STEP)
        .suffix(" px");
    if ui.add(slider).changed() {
        settings.settings_mut().set_font_size(size);
        let _ = settings.save();
    }
}

fn render_font_family_selector(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let mut current = settings.settings().font_family.clone();
    let os_fonts = katana_platform::os_fonts::OsFontScanner::cached_fonts();

    egui::ComboBox::from_id_salt("font_family_selector")
        .selected_text(&current)
        .width(FONT_FAMILY_COMBOBOX_WIDTH) // Provide enough width for long font names
        .show_ui(ui, |ui| {
            let defaults = ["Proportional", "Monospace"];
            for family in defaults {
                if ui
                    .selectable_value(&mut current, family.to_string(), family)
                    .changed()
                {
                    settings.settings_mut().font_family = family.to_string();
                    let _ = settings.save();
                }
            }
            ui.separator();
            for (name, _) in os_fonts {
                if ui
                    .selectable_value(&mut current, name.to_string(), name)
                    .changed()
                {
                    settings.settings_mut().font_family = name.to_string();
                    let _ = settings.save();
                }
            }
        });
}

// ── Layout tab ───────────────────────────────────────────────────────

fn render_layout_tab(ui: &mut egui::Ui, settings: &mut SettingsService) {
    section_header(ui, "settings_tab_layout");
    render_toc_toggle(ui, settings);
}

fn render_toc_toggle(ui: &mut egui::Ui, settings: &mut SettingsService) {
    let mut toc_visible = settings.settings().toc_visible;
    if ui
        .checkbox(&mut toc_visible, i18n::t("settings_toc_visible"))
        .changed()
    {
        settings.settings_mut().toc_visible = toc_visible;
        let _ = settings.save();
    }
}
