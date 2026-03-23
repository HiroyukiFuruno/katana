use egui::{self, NumExt, RichText, Sense, TextBuffer, TextStyle, Ui, Vec2, epaint};

#[inline]
pub fn rule(ui: &mut Ui, end_line: bool) {
    ui.add(egui::Separator::default().horizontal());
    // This does not add a new line, but instead ends the separator
    if end_line {
        newline(ui);
    }
}

#[inline]
pub fn soft_break(ui: &mut Ui) {
    ui.label(" ");
}

#[inline]
pub fn newline(ui: &mut Ui) {
    ui.label("\n");
}

pub fn bullet_point(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().circle_filled(
        rect.center(),
        rect.height() / 6.0,
        ui.visuals().strong_text_color(),
    );
}

pub fn bullet_point_hollow(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().circle(
        rect.center(),
        rect.height() / 6.0,
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(0.6, ui.visuals().strong_text_color()),
    );
}

pub fn number_point(ui: &mut Ui, number: &str) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().text(
        rect.right_top() + egui::vec2(0.0, 3.0),
        egui::Align2::RIGHT_TOP,
        format!("{number}."),
        TextStyle::Body.resolve(ui.style()),
        ui.visuals().strong_text_color(),
    );
}

#[inline]
pub fn footnote_start(ui: &mut Ui, note: &str) {
    ui.label(RichText::new(note).raised().strong().small());
}

pub fn footnote(ui: &mut Ui, text: &str) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().text(
        rect.right_top() + egui::vec2(0.0, 3.0),
        egui::Align2::RIGHT_TOP,
        format!("{text}."),
        TextStyle::Small.resolve(ui.style()),
        ui.visuals().strong_text_color(),
    );
}

fn height_body(ui: &Ui) -> f32 {
    ui.text_style_height(&TextStyle::Body)
}

fn width_body_space(ui: &Ui) -> f32 {
    let id = TextStyle::Body.resolve(ui.style());
    ui.fonts_mut(|f| f.glyph_width(&id, ' '))
}

/// Enhanced/specialized version of egui's code blocks. This one features copy button and borders
pub fn code_block<'t>(
    ui: &mut Ui,
    max_width: f32,
    text: &str,
    layouter: &'t mut dyn FnMut(&Ui, &dyn TextBuffer, f32) -> std::sync::Arc<egui::Galley>,
) {
    let mut text = text.strip_suffix('\n').unwrap_or(text);

    // To manually add background color to the code block, we imitate what
    // TextEdit does internally
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);

    // We use a `TextEdit` to make the text selectable.
    // Note that we take a `&mut` to a non-`mut` `&str`, which is
    // the how to tell `egui` that the text is not editable.
    let output = egui::TextEdit::multiline(&mut text)
        .layouter(layouter)
        .desired_width(max_width)
        // prevent trailing lines
        .desired_rows(1)
        .show(ui);

    // Background color + frame (This is lost when TextEdit it not editable)
    let frame_rect = output.response.rect;
    ui.painter().set(
        where_to_put_background,
        epaint::RectShape::new(
            frame_rect,
            ui.style().noninteractive().corner_radius,
            ui.visuals().extreme_bg_color,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Outside,
        ),
    );

    if !text.is_empty() {
        // Copy icon
        let spacing = &ui.style().spacing;
        let icon_size = egui::vec2(20.0, 20.0);
        let position = egui::pos2(
            frame_rect.right_top().x - icon_size.x - spacing.button_padding.x - 8.0,
            frame_rect.right_top().y + spacing.button_padding.y + 5.0,
        );
        let icon_rect = egui::Rect::from_min_size(position, icon_size);
        // Check if we should show ✓ instead of copy icon
        let persistent_id = ui.make_persistent_id(output.response.id);
        let copied_icon = ui.memory_mut(|m| *m.data.get_temp_mut_or_default::<bool>(persistent_id));
        // Draw copy icon using egui painter API directly
        // instead of SVG to bypass image loader pipeline issues.
        let copy_btn_response = ui.allocate_rect(icon_rect, egui::Sense::click());
        let icon_color = if copy_btn_response.hovered() {
            ui.visuals().strong_text_color()
        } else {
            ui.visuals().weak_text_color()
        };
        let painter = ui.painter();
        let stroke = egui::Stroke::new(1.2, icon_color);
        if copied_icon {
            // Checkmark icon
            let p1 = icon_rect.min + egui::vec2(4.0, 11.0);
            let p2 = icon_rect.min + egui::vec2(8.0, 15.0);
            let p3 = icon_rect.min + egui::vec2(16.0, 5.0);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
        } else {
            // Two overlapping rectangles (copy icon)
            let back = egui::Rect::from_min_size(
                icon_rect.min + egui::vec2(2.0, 2.0),
                egui::vec2(10.0, 12.0),
            );
            let front = egui::Rect::from_min_size(
                icon_rect.min + egui::vec2(6.0, 0.0),
                egui::vec2(10.0, 12.0),
            );
            painter.rect_stroke(back, 1.0, stroke, egui::StrokeKind::Outside);
            painter.rect_filled(front, 1.0, ui.visuals().extreme_bg_color);
            painter.rect_stroke(front, 1.0, stroke, egui::StrokeKind::Outside);
        }
        let copy_button = copy_btn_response
        .on_hover_cursor(
            ui.visuals()
                .interact_cursor
                .unwrap_or(egui::CursorIcon::Default),
        );

    // Update icon state in persistent memory
    if copied_icon && !copy_button.hovered() {
        ui.memory_mut(|m| *m.data.get_temp_mut_or_default(persistent_id) = false);
    }
    if !copied_icon && copy_button.clicked() {
        ui.memory_mut(|m| *m.data.get_temp_mut_or_default(persistent_id) = true);
    }

    if copy_button.clicked() {
        use egui::TextBuffer as _;
        let copy_text = if let Some(cursor) = output.cursor_range {
            let selected_chars = cursor.as_sorted_char_range();
            let selected_text = text.char_range(selected_chars);
            if selected_text.is_empty() {
                text.to_owned()
            } else {
                selected_text.to_owned()
            }
        } else {
            text.to_owned()
        };
        ui.ctx().copy_text(copy_text);
    }
    }
}
// Stripped down version of egui's Checkbox. The only difference is that this
// creates a noninteractive checkbox. ui.add_enabled could have been used instead,
// but it makes the checkbox too grey.
pub struct ImmutableCheckbox<'a> {
    checked: &'a mut bool,
}

impl<'a> ImmutableCheckbox<'a> {
    pub fn without_text(checked: &'a mut bool) -> Self {
        ImmutableCheckbox { checked }
    }
}

impl egui::Widget for ImmutableCheckbox<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let ImmutableCheckbox { checked } = self;

        let spacing = &ui.spacing();
        let icon_width = spacing.icon_width;

        let mut desired_size = egui::vec2(icon_width, 0.0);
        desired_size = desired_size.at_least(Vec2::splat(spacing.interact_size.y));
        desired_size.y = desired_size.y.max(icon_width);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().visuals.noninteractive();
            let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
            ui.painter().add(epaint::RectShape::new(
                big_icon_rect.expand(visuals.expansion),
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            ));

            if *checked {
                // Check mark:
                ui.painter().add(egui::Shape::line(
                    vec![
                        egui::pos2(small_icon_rect.left(), small_icon_rect.center().y),
                        egui::pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
                        egui::pos2(small_icon_rect.right(), small_icon_rect.top()),
                    ],
                    visuals.fg_stroke,
                ));
            }
        }

        response
    }
}

pub fn blockquote(ui: &mut Ui, accent: egui::Color32, add_contents: impl FnOnce(&mut Ui)) {
    let start = ui.painter().add(egui::Shape::Noop);
    let response = egui::Frame::new()
        // offset the frame so that we can use the space for the horizontal line and other stuff
        // By not using a separator we have better control
        .outer_margin(egui::Margin {
            left: 10,
            ..Default::default()
        })
        .show(ui, add_contents)
        .response;

    // FIXME: Add some rounding

    ui.painter().set(
        start,
        egui::epaint::Shape::line_segment(
            [
                egui::pos2(response.rect.left_top().x, response.rect.left_top().y + 5.0),
                egui::pos2(
                    response.rect.left_bottom().x,
                    response.rect.left_bottom().y - 5.0,
                ),
            ],
            egui::Stroke::new(3.0, accent),
        ),
    );
}
