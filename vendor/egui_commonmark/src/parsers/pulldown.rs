use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::ops::Range;

use crate::{CommonMarkCache, CommonMarkOptions};

use egui::{self, Id, Pos2, RichText, TextStyle, Ui};

use crate::List;
use egui_commonmark_backend::elements::*;
use egui_commonmark_backend::misc::*;
use egui_commonmark_backend::pulldown::*;
use pulldown_cmark::{CowStr, HeadingLevel};
use unicode_segmentation::UnicodeSegmentation;

#[allow(dead_code)]
const INLINE_EMOJI_URI_PREFIX: &str = "bytes://katana-inline-emoji-";
#[allow(dead_code)]
const INLINE_EMOJI_FILENAME_SUFFIX: &str = ".png";
#[allow(dead_code)]
const INLINE_EMOJI_MIN_PIXEL_SIZE: u32 = 16;
#[allow(dead_code)]
const INLINE_EMOJI_DISPLAY_SCALE: f32 = 1.0;

/// Newline logic is constructed by the following:
/// All elements try to insert a newline before them (if they are allowed)
/// and end their own line.
struct Newline {
    /// Whether a newline should not be inserted before a widget. This is only for
    /// the first widget.
    should_not_start_newline_forced: bool,
    /// Whether an element should insert a newline before it
    should_start_newline: bool,
    /// Whether an element should end it's own line using a newline
    /// This will have to be set to false in cases such as when blocks are within
    /// a list.
    should_end_newline: bool,
    /// only false when the widget is the last one.
    should_end_newline_forced: bool,
}

impl Default for Newline {
    fn default() -> Self {
        Self {
            should_not_start_newline_forced: true,
            should_start_newline: true,
            should_end_newline: true,
            should_end_newline_forced: true,
        }
    }
}

impl Newline {
    pub fn can_insert_end(&self) -> bool {
        self.should_end_newline && self.should_end_newline_forced
    }

    pub fn can_insert_start(&self) -> bool {
        self.should_start_newline && !self.should_not_start_newline_forced
    }

    pub fn try_insert_start(&self, ui: &mut Ui) {
        if self.can_insert_start() {
            newline(ui);
        }
    }

    pub fn try_insert_end(&self, ui: &mut Ui) {
        if self.can_insert_end() {
            newline(ui);
        }
    }
}

#[derive(Default)]
struct DefinitionList {
    is_first_item: bool,
    is_def_list_def: bool,
}

pub(crate) struct CommonMarkViewerInternal<'a> {
    curr_table: usize,
    curr_heading: usize,
    scroll_to_heading_index: Option<usize>,
    heading_anchors: Option<&'a mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    text_style: Style,
    pending_inline: Vec<RichText>,
    after_inline_widget: bool,
    list: List,
    link: Option<Link>,
    image: Option<Image>,
    line: Newline,
    is_list_item: bool,
    def_list: DefinitionList,
    code_block: Option<CodeBlock>,
    html_block: String,
    table_alignments: Option<Vec<pulldown_cmark::Alignment>>,
    is_blockquote: bool,
    /// True while processing events inside a blockquote. Used to suppress
    /// paragraph-level newlines that would otherwise create large vertical gaps.
    inside_blockquote: bool,
    checkbox_events: Vec<crate::TaskListAction>,
    /// When inside a `<details>` block, holds the summary text.
    /// Used to render a CollapsingHeader across HTML block boundaries.
    details_summary: Option<String>,
    details_id_counter: usize,
    task_list_indices: std::collections::HashSet<usize>,
    current_event_idx: usize,
    active_task_context_menu: Option<(char, std::ops::Range<usize>, bool)>,
    active_char_range: Option<std::ops::Range<usize>>,
    hovered_spans: Option<&'a mut Vec<std::ops::Range<usize>>>,
    active_rects: Vec<(egui::Rect, std::ops::Range<usize>)>,
    block_states: Vec<(f32, std::ops::Range<usize>)>,
    active_bg_color: Option<egui::Color32>,
    hover_bg_color: Option<egui::Color32>,
}

impl<'a> CommonMarkViewerInternal<'a> {
    pub fn new(
        scroll_to_heading_index: Option<usize>,
        heading_anchors: Option<&'a mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        heading_offset: usize,
        active_char_range: Option<std::ops::Range<usize>>,
        hovered_spans: Option<&'a mut Vec<std::ops::Range<usize>>>,
        active_bg_color: Option<egui::Color32>,
        hover_bg_color: Option<egui::Color32>,
    ) -> Self {
        Self {
            curr_table: 0,
            curr_heading: heading_offset,
            scroll_to_heading_index,
            heading_anchors,
            text_style: Style::default(),
            pending_inline: Vec::new(),
            after_inline_widget: false,
            list: List::default(),
            link: None,
            image: None,
            line: Newline::default(),
            is_list_item: false,
            def_list: Default::default(),
            code_block: None,
            html_block: String::new(),
            table_alignments: None,
            is_blockquote: false,
            inside_blockquote: false,
            checkbox_events: Vec::new(),
            details_summary: None,
            details_id_counter: 0,
            task_list_indices: std::collections::HashSet::new(),
            current_event_idx: 0,
            active_task_context_menu: None,
            active_char_range,
            hovered_spans,
            active_rects: Vec::new(),
            block_states: Vec::new(),
            active_bg_color,
            hover_bg_color,
        }
    }
}

fn parser_options_math(is_math_enabled: bool) -> pulldown_cmark::Options {
    if is_math_enabled {
        parser_options() | pulldown_cmark::Options::ENABLE_MATH
    } else {
        parser_options()
    }
}

const BLOCKQUOTE_LEFT_INSET: i8 = 10;

const BLOCKQUOTE_LINE_WIDTH: f32 = 3.0;

fn render_blockquote(ui: &mut Ui, accent: egui::Color32, add_contents: impl FnOnce(&mut Ui)) {
    let content_width = (ui.available_width() - f32::from(BLOCKQUOTE_LEFT_INSET)).max(0.0);
    let start = ui.painter().add(egui::Shape::Noop);
    let response = egui::Frame::NONE
        .inner_margin(egui::Margin {
            left: BLOCKQUOTE_LEFT_INSET,
            ..Default::default()
        })
        .show(ui, |ui| {
            let child_rect = egui::Rect::from_min_size(
                ui.next_widget_position(),
                egui::vec2(content_width, 0.0),
            );
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(child_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                add_contents,
            );
        })
        .response;

    ui.painter().set(
        start,
        egui::epaint::Shape::line_segment(
            [
                egui::pos2(
                    response.rect.left() + BLOCKQUOTE_LINE_WIDTH / 2.0,
                    response.rect.top(),
                ),
                egui::pos2(
                    response.rect.left() + BLOCKQUOTE_LINE_WIDTH / 2.0,
                    response.rect.bottom(),
                ),
            ],
            egui::Stroke::new(BLOCKQUOTE_LINE_WIDTH, accent),
        ),
    );
}

impl<'a> CommonMarkViewerInternal<'a> {
    fn flush_pending_inline(&mut self, ui: &mut Ui, max_width: f32) {
        if self.pending_inline.is_empty() {
            return;
        }

        let pending_inline = std::mem::take(&mut self.pending_inline);
        let remaining_width = (ui.clip_rect().right() - ui.cursor().min.x).max(0.0);
        let max_width = max_width.min(remaining_width);
        let style = ui.style().clone();
        let halign = ui.layout().horizontal_align();
        let mut layout_job = egui::text::LayoutJob::default();
        layout_job.halign = halign;
        layout_job.wrap.max_width = max_width;
        layout_job.wrap.break_anywhere = true;
        for rich_text in &pending_inline {
            rich_text.clone().append_to(
                &mut layout_job,
                &style,
                egui::FontSelection::Default,
                egui::Align::Center,
            );
        }

        // Extract and strip text decorations from LayoutJob sections so that we
        // can draw them ourselves at visually pleasing native Y coordinates.
        // epaint's built-in strikethrough and underline use inaccurate font metrics
        // from ab_glyph for macOS system CJK fonts, causing misalignments or clipping.
        let mut strikethrough_char_flags: Vec<bool> = Vec::new();
        let mut underline_char_flags: Vec<bool> = Vec::new();
        let mut any_strikethrough = false;
        let mut any_underline = false;
        let mut st_stroke = egui::Stroke::NONE;
        let mut ul_stroke = egui::Stroke::NONE;

        {
            let text = &layout_job.text;
            let total_chars = text.chars().count();
            strikethrough_char_flags.resize(total_chars, false);
            underline_char_flags.resize(total_chars, false);

            for section in &layout_job.sections {
                if section.format.strikethrough != egui::Stroke::NONE {
                    any_strikethrough = true;
                    st_stroke = section.format.strikethrough;
                    let start_char = text[..section.byte_range.start].chars().count();
                    let end_char = start_char
                        + text[section.byte_range.start..section.byte_range.end]
                            .chars()
                            .count();
                    for flag in &mut strikethrough_char_flags[start_char..end_char] {
                        *flag = true;
                    }
                }

                if section.format.underline != egui::Stroke::NONE {
                    any_underline = true;
                    ul_stroke = section.format.underline;
                    let start_char = text[..section.byte_range.start].chars().count();
                    let end_char = start_char
                        + text[section.byte_range.start..section.byte_range.end]
                            .chars()
                            .count();
                    for flag in &mut underline_char_flags[start_char..end_char] {
                        *flag = true;
                    }
                }
            }
        }

        // Strip decorations from sections so epaint won't draw them with buggy bounds
        if any_strikethrough {
            for section in &mut layout_job.sections {
                section.format.strikethrough = egui::Stroke::NONE;
            }
        }
        if any_underline {
            for section in &mut layout_job.sections {
                section.format.underline = egui::Stroke::NONE;
            }
        }

        // Lay out and paint the galley manually for decoration overrides,
        // or fall back to Label for the common un-decorated case.
        if any_strikethrough || any_underline {
            // Render text using Label (identical to non-strikethrough path)
            // so that list indentation, wrapping, and positioning are correct.
            let layout_job_for_galley = layout_job.clone();
            let mut response = ui.add(egui::Label::new(layout_job).wrap().halign(halign));

            // Attach context menu to the text response if we are within an active task
            if let Some((_state, ref span, mutable)) = self.active_task_context_menu {
                response = ui.interact(response.rect, response.id.with("hitbox"), if mutable { egui::Sense::click() } else { egui::Sense::hover() });
                
                if mutable {
                    response.context_menu(|ui| {
                        if ui.button("未実施 [ ]").clicked() {
                            self.checkbox_events.push(crate::TaskListAction {
                                span: span.clone(),
                                new_state: ' ',
                            });
                            ui.close();
                        }
                        if ui.button("実施中 [/]").clicked() {
                            self.checkbox_events.push(crate::TaskListAction {
                                span: span.clone(),
                                new_state: '/',
                            });
                            ui.close();
                        }
                        if ui.button("完了 [x]").clicked() {
                            self.checkbox_events.push(crate::TaskListAction {
                                span: span.clone(),
                                new_state: 'x',
                            });
                            ui.close();
                        }
                    });
                }
            }

            // Lay out a separate galley to get per-row glyph positions
            // for drawing the strikethrough overlay.
            let galley = ui.fonts_mut(|f| f.layout_job(layout_job_for_galley));
            let text_pos = response.rect.left_top();

            // Draw overlay lines
            let mut char_idx = 0usize;
            for row in &galley.rows {
                let row_rect = row.rect();
                let mut st_min_x: Option<f32> = None;
                let mut st_max_x: Option<f32> = None;
                let mut ul_min_x: Option<f32> = None;
                let mut ul_max_x: Option<f32> = None;

                for glyph in &row.glyphs {
                    let is_st = char_idx < strikethrough_char_flags.len()
                        && strikethrough_char_flags[char_idx];
                    let is_ul = char_idx < underline_char_flags.len()
                        && underline_char_flags[char_idx];

                    if is_st {
                        let gx = text_pos.x + glyph.pos.x;
                        let gx_end = text_pos.x + glyph.max_x();
                        st_min_x = Some(st_min_x.map_or(gx, |v: f32| v.min(gx)));
                        st_max_x = Some(st_max_x.map_or(gx_end, |v: f32| v.max(gx_end)));
                    } else if let (Some(min_x), Some(max_x)) = (st_min_x.take(), st_max_x.take()) {
                        let y = text_pos.y + row_rect.min.y + row_rect.height() * 0.38;
                        ui.painter().hline(min_x..=max_x, y, st_stroke);
                    }

                    if is_ul {
                        let gx = text_pos.x + glyph.pos.x;
                        let gx_end = text_pos.x + glyph.max_x();
                        ul_min_x = Some(ul_min_x.map_or(gx, |v: f32| v.min(gx)));
                        ul_max_x = Some(ul_max_x.map_or(gx_end, |v: f32| v.max(gx_end)));
                    } else if let (Some(min_x), Some(max_x)) = (ul_min_x.take(), ul_max_x.take()) {
                        let y = text_pos.y + row_rect.min.y + row_rect.height() * 0.70; // Tighten precisely to the visual text baseline
                        ui.painter().hline(min_x..=max_x, y, ul_stroke);
                    }

                    char_idx += 1;
                }
                
                // Flush remaining runs at end of row
                if let (Some(min_x), Some(max_x)) = (st_min_x.take(), st_max_x.take()) {
                    let y = text_pos.y + row_rect.min.y + row_rect.height() * 0.38;
                    ui.painter().hline(min_x..=max_x, y, st_stroke);
                }
                if let (Some(min_x), Some(max_x)) = (ul_min_x.take(), ul_max_x.take()) {
                    let y = text_pos.y + row_rect.min.y + row_rect.height() * 0.70;
                    ui.painter().hline(min_x..=max_x, y, ul_stroke);
                }
            }
        } else {
            let mut response = ui.add(egui::Label::new(layout_job).wrap().halign(halign));
            
            // Attach context menu to the text response if we are within an active task
            if let Some((_state, ref span, mutable)) = self.active_task_context_menu {
                response = ui.interact(response.rect, response.id.with("hitbox"), if mutable { egui::Sense::click() } else { egui::Sense::hover() });
                
                if mutable {
                    response.context_menu(|ui| {
                        if ui.button("未実施 [ ]").clicked() {
                            self.checkbox_events.push(crate::TaskListAction {
                                span: span.clone(),
                                new_state: ' ',
                            });
                            ui.close();
                        }
                        if ui.button("実施中 [/]").clicked() {
                            self.checkbox_events.push(crate::TaskListAction {
                                span: span.clone(),
                                new_state: '/',
                            });
                            ui.close();
                        }
                        if ui.button("完了 [x]").clicked() {
                            self.checkbox_events.push(crate::TaskListAction {
                                span: span.clone(),
                                new_state: 'x',
                            });
                            ui.close();
                        }
                    });
                }
            }
        }
    }

    fn emit_wrapped_followup_chunks(&self, ui: &mut Ui, text: &str) {
        for chunk in text.split_inclusive(char::is_whitespace) {
            if !chunk.is_empty() {
                ui.label(self.text_style.to_richtext(ui, chunk));
            }
        }
    }

    fn push_inline_text(&mut self, text: &str, ui: &mut Ui, max_width: f32) {
        if text.is_empty() {
            return;
        }

        let rich_text = self.text_style.to_richtext(ui, text);
        if self.after_inline_widget && !self.text_style.code && text.contains(char::is_whitespace) {
            self.flush_pending_inline(ui, max_width);
            self.emit_wrapped_followup_chunks(ui, text);
        } else {
            self.pending_inline.push(rich_text);
        }
        self.after_inline_widget = false;
    }

    #[allow(dead_code)]
    fn current_inline_font_size(&self, ui: &Ui) -> f32 {
        let body_size = TextStyle::Body.resolve(ui.style()).size;
        let heading_size = TextStyle::Heading.resolve(ui.style()).size;
        let heading_delta = heading_size - body_size;

        match self.text_style.heading {
            Some(0) => heading_size,
            Some(1) => body_size + heading_delta * 0.835,
            Some(2) => body_size + heading_delta * 0.668,
            Some(3) => body_size + heading_delta * 0.501,
            Some(4) => body_size + heading_delta * 0.334,
            Some(_) => body_size + heading_delta * 0.167,
            None if self.text_style.code => TextStyle::Monospace.resolve(ui.style()).size,
            None => body_size,
        }
    }

    fn try_render_inline_emoji(&mut self, ui: &mut Ui, max_width: f32, grapheme: &str) -> bool {
        #[cfg(target_os = "macos")]
        {
            let pixel_size = self
                .current_inline_font_size(ui)
                .ceil()
                .max(INLINE_EMOJI_MIN_PIXEL_SIZE as f32) as u32;
            let display_size = pixel_size as f32 * INLINE_EMOJI_DISPLAY_SCALE;
            if let Some(bytes) =
                katana_core::emoji::render_apple_color_emoji_png(grapheme, pixel_size)
            {
                self.flush_pending_inline(ui, max_width);
                ui.add(
                    egui::Image::from_bytes(inline_emoji_uri(grapheme, pixel_size), bytes)
                        .fit_to_exact_size(egui::vec2(display_size, display_size)),
                );
                self.after_inline_widget = true;
                return true;
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = ui;
            let _ = max_width;
            let _ = grapheme;
        }

        false
    }

    fn should_flush_before_start_tag(tag: &pulldown_cmark::Tag) -> bool {
        !matches!(
            tag,
            pulldown_cmark::Tag::Emphasis
                | pulldown_cmark::Tag::Strong
                | pulldown_cmark::Tag::Strikethrough
        )
    }

    fn should_flush_before_end_tag(tag: &pulldown_cmark::TagEnd) -> bool {
        !matches!(
            tag,
            pulldown_cmark::TagEnd::Emphasis
                | pulldown_cmark::TagEnd::Strong
                | pulldown_cmark::TagEnd::Strikethrough
                | pulldown_cmark::TagEnd::Link
                | pulldown_cmark::TagEnd::Image
        )
    }

    /// If split Id is provided then split points will be populated
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        text: &str,
        split_points_id: Option<Id>,
    ) -> (egui::InnerResponse<()>, Vec<crate::TaskListAction>) {
        let max_width = options.max_width(ui);
        let layout = egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true);
        let child_rect =
            egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(max_width, 0.0));

        let re = ui.scope_builder(
            egui::UiBuilder::new().max_rect(child_rect).layout(layout),
            |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let height = ui.text_style_height(&TextStyle::Body);
                ui.set_row_height(height);

                let raw_events = pulldown_cmark::Parser::new_ext(
                    text,
                    parser_options_math(options.math_fn.is_some()),
                )
                .into_offset_iter();

                let (extracted_events, task_indices) = extract_custom_task_lists(extract_footnotes(raw_events));
                self.task_list_indices = task_indices;

                let mut events = extracted_events
                    .into_iter()
                    .enumerate()
                    .peekable();

                while let Some((index, (e, src_span))) = events.next() {
                    self.current_event_idx = index;
                    let start_position = ui.next_widget_position();
                    let is_element_end = matches!(e, pulldown_cmark::Event::End(_));
                    let should_add_split_point = self.list.is_inside_a_list() && is_element_end;

                    if events.peek().is_none() {
                        self.line.should_end_newline_forced = false;
                    }

                    self.process_event(ui, &mut events, e, src_span, cache, options, max_width);

                    if let Some(source_id) = split_points_id {
                        if should_add_split_point {
                            let scroll_cache = scroll_cache(cache, &source_id);
                            let end_position = ui.next_widget_position();

                            let split_point_exists = scroll_cache
                                .split_points
                                .iter()
                                .any(|(i, _, _)| *i == index);

                            if !split_point_exists {
                                scroll_cache.split_points.push((
                                    index,
                                    start_position,
                                    end_position,
                                ));
                            }
                        }
                    }

                    if index == 0 {
                        self.line.should_not_start_newline_forced = false;
                    }
                }

                if let Some(source_id) = split_points_id {
                    scroll_cache(cache, &source_id).page_size =
                        Some(ui.next_widget_position().to_vec2());
                }
            },
        );

        (re, std::mem::take(&mut self.checkbox_events))
    }

    pub(crate) fn show_scrollable(
        &mut self,
        source_id: Id,
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        text: &str,
    ) {
        let available_size = ui.available_size();
        let scroll_id = source_id.with("_scroll_area");

        let Some(page_size) = scroll_cache(cache, &source_id).page_size else {
            egui::ScrollArea::vertical()
                .id_salt(scroll_id)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    self.show(ui, cache, options, text, Some(source_id));
                });
            // Prevent repopulating points twice at startup
            scroll_cache(cache, &source_id).available_size = available_size;
            return;
        };

        let raw_events =
            pulldown_cmark::Parser::new_ext(text, parser_options_math(options.math_fn.is_some()))
                .into_offset_iter();
        let events = extract_footnotes(raw_events);

        let num_rows = events.len();

        egui::ScrollArea::vertical()
            .id_salt(scroll_id)
            // Elements have different widths, so the scroll area cannot try to shrink to the
            // content, as that will mean that the scroll bar will move when loading elements
            // with different widths.
            .auto_shrink([false, true])
            .show_viewport(ui, |ui, viewport| {
                ui.set_height(page_size.y);
                let layout = egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true);

                let max_width = options.max_width(ui);
                let child_rect = egui::Rect::from_min_size(
                    ui.next_widget_position(),
                    egui::vec2(max_width, 0.0),
                );
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(child_rect).layout(layout),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let scroll_cache = scroll_cache(cache, &source_id);

                        // finding the first element that's not in the viewport anymore
                        let (first_event_index, _, first_end_position) = scroll_cache
                            .split_points
                            .iter()
                            .filter(|(_, _, end_position)| end_position.y < viewport.min.y)
                            .nth_back(1)
                            .copied()
                            .unwrap_or((0, Pos2::ZERO, Pos2::ZERO));

                        // finding the last element that's just outside the viewport
                        let last_event_index = scroll_cache
                            .split_points
                            .iter()
                            .filter(|(_, start_position, _)| start_position.y > viewport.max.y)
                            .nth(1)
                            .map(|(index, _, _)| *index)
                            .unwrap_or(num_rows);

                        ui.allocate_space(first_end_position.to_vec2());

                        // only rendering the elements that are inside the viewport
                        let mut events = events
                            .into_iter()
                            .enumerate()
                            .skip(first_event_index)
                            .take(last_event_index - first_event_index)
                            .peekable();

                        while let Some((i, (e, src_span))) = events.next() {
                            self.current_event_idx = i;
                            if events.peek().is_none() {
                                self.line.should_end_newline_forced = false;
                            }

                            self.process_event(
                                ui,
                                &mut events,
                                e,
                                src_span,
                                cache,
                                options,
                                max_width,
                            );

                            if i == 0 {
                                self.line.should_not_start_newline_forced = false;
                            }
                        }
                    },
                );
            });

        // Forcing full re-render to repopulate split points for the new size
        let scroll_cache = scroll_cache(cache, &source_id);
        if available_size != scroll_cache.available_size {
            scroll_cache.available_size = available_size;
            scroll_cache.page_size = None;
            scroll_cache.split_points.clear();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_event<'e>(
        &mut self,
        ui: &mut Ui,
        events: &mut Peekable<impl Iterator<Item = EventIteratorItem<'e>>>,
        event: pulldown_cmark::Event,
        src_span: Range<usize>,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        max_width: f32,
    ) {
        // When inside a <details> block, check if the CollapsingHeader was just entered.
        // The first event after setting details_summary triggers the header rendering.
        if let Some(summary) = self.details_summary.take() {
            let id = ui.id().with("_details").with(self.details_id_counter);

            // Collect the body events up to </details>
            let mut body_events = Vec::new();
            body_events.push((0, (event, src_span)));
            body_events.extend(self.collect_until_details_close(events));

            // Flush inline content first, then render accordion in a top_down scope.
            // A top_down scope is required so that ui.add_space() works as VERTICAL spacing;
            // in the outer left_to_right+main_wrap layout add_space is horizontal (4.3 fix).
            self.flush_pending_inline(ui, max_width);
            let cursor = ui.next_widget_position();
            let outer_rect = egui::Rect::from_min_max(
                cursor,
                egui::pos2(
                    cursor.x + max_width,
                    ui.max_rect().max.y.max(cursor.y + 1.0),
                ),
            );
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(outer_rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
                |ui| {
                    const OPTICAL_ACCORDION_TOP_MARGIN: f32 = -24.0;
                    const OPTICAL_DEFAULT_INTERACT_HEIGHT: f32 = 18.0;

                    // 4.3 (Optical): Markdown block parser generates empty \n\n gaps resulting in a massive 
                    // unbalanced top gap before <details> compared to the bottom. 
                    // Explicitly pull the widget back up to optically balance against 8px bottom spacing.
                    ui.add_space(OPTICAL_ACCORDION_TOP_MARGIN); 

                    // 4.4 (Optical): Egui natively generates geometrically centered icons that map incorrectly 
                    // to the optical visual weight of Japanese fonts, plunging to the baseline.
                    // Instead of inflating/deflating interact_size, we use standard spacing and apply an optical shift.
                    ui.spacing_mut().interact_size.y = OPTICAL_DEFAULT_INTERACT_HEIGHT;

                    let id_salt = egui::Id::new(&summary).with(id);
                    let collapsing = egui::CollapsingHeader::new(
                        egui::RichText::new(&summary).strong()
                    )
                    .id_salt(id_salt)
                    .icon(crate::ui_components::centering::AccordionIcon::paint_optically_centered);

                    let _header_res = collapsing.show_unindented(ui, |ui| {
                        let layout = egui::Layout::left_to_right(egui::Align::Center)
                            .with_main_wrap(true);
                        let body_width = ui.available_width();
                        let child_rect = egui::Rect::from_min_size(
                            ui.next_widget_position(),
                            egui::vec2(body_width, 0.0),
                        );
                        ui.scope_builder(
                            egui::UiBuilder::new().max_rect(child_rect).layout(layout),
                            |ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                let height = ui.text_style_height(&egui::TextStyle::Body);
                                ui.set_row_height(height);
                                let mut iter = body_events.into_iter().peekable();
                                while let Some((index, (e, src_span))) = iter.next() {
                                    self.current_event_idx = index;
                                    if iter.peek().is_none() {
                                        self.line.should_end_newline_forced = false;
                                    }
                                    self.process_event(
                                        ui, &mut iter, e, src_span, cache, options, max_width,
                                    );
                                    if index == 0 {
                                        self.line.should_not_start_newline_forced = false;
                                    }
                                }
                            },
                        );
                    });

                    // 4.3: Because `egui_commonmark` inline HTML processing strips some block spacing,
                    // we add a small bottom margin so the next block (like heading 13) isn't crushed.
                    ui.add_space(8.0);
                },
            );
            return;
        }

        if let pulldown_cmark::Event::Html(ref text) = event {
            if text.starts_with("<!-- FOOTNOTE_START_BLOCK:") {
                // Collect ALL remaining footnote blocks (this one + subsequent ones) upfront
                // so we can render them inside a single top_down scope — eliminating any
                // inter-block spacing that the outer left_to_right+main_wrap layout would insert.
                // Each collected entry: (note_name, is_highlighted, should_scroll, frame, body_events)
                let active_highlight_key = egui::Id::new("highlight_footnote_active");
                let scroll_key = egui::Id::new("scroll_to_footnote");

                // Helper to clone an Event<'e> into an owned Event<'static> so we can
                // store it across lifetime boundaries inside the local Vec.
                fn own_event(e: pulldown_cmark::Event<'_>) -> pulldown_cmark::Event<'static> {
                    use pulldown_cmark::Event;
                    match e {
                        Event::Text(s) => Event::Text(s.into_string().into()),
                        Event::Code(s) => Event::Code(s.into_string().into()),
                        Event::Html(s) => Event::Html(s.into_string().into()),
                        Event::InlineHtml(s) => Event::InlineHtml(s.into_string().into()),
                        Event::SoftBreak => Event::SoftBreak,
                        Event::HardBreak => Event::HardBreak,
                        Event::Rule => Event::Rule,
                        Event::Start(tag) => Event::Start(own_tag(tag)),
                        Event::End(tag) => Event::End(tag),
                        _ => Event::SoftBreak, // footnotes won't contain other variants
                    }
                }
                fn own_tag(t: pulldown_cmark::Tag<'_>) -> pulldown_cmark::Tag<'static> {
                    use pulldown_cmark::Tag;
                    match t {
                        Tag::Emphasis => Tag::Emphasis,
                        Tag::Strong => Tag::Strong,
                        Tag::Strikethrough => Tag::Strikethrough,
                        Tag::Paragraph => Tag::Paragraph,
                        Tag::Heading {
                            level,
                            id,
                            classes,
                            attrs,
                        } => Tag::Heading {
                            level,
                            id: id.map(|s| s.into_string().into()),
                            classes: classes
                                .into_iter()
                                .map(|c| c.into_string().into())
                                .collect(),
                            attrs: attrs
                                .into_iter()
                                .map(|(k, v)| {
                                    (k.into_string().into(), v.map(|v| v.into_string().into()))
                                })
                                .collect(),
                        },
                        Tag::Link {
                            link_type,
                            dest_url,
                            title,
                            id,
                        } => Tag::Link {
                            link_type,
                            dest_url: dest_url.into_string().into(),
                            title: title.into_string().into(),
                            id: id.into_string().into(),
                        },
                        Tag::Image {
                            link_type,
                            dest_url,
                            title,
                            id,
                        } => Tag::Image {
                            link_type,
                            dest_url: dest_url.into_string().into(),
                            title: title.into_string().into(),
                            id: id.into_string().into(),
                        },
                        Tag::CodeBlock(kind) => Tag::CodeBlock(match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(s) => {
                                pulldown_cmark::CodeBlockKind::Fenced(s.into_string().into())
                            }
                            pulldown_cmark::CodeBlockKind::Indented => {
                                pulldown_cmark::CodeBlockKind::Indented
                            }
                        }),
                        Tag::List(n) => Tag::List(n),
                        Tag::Item => Tag::Item,
                        Tag::BlockQuote(k) => Tag::BlockQuote(k),
                        Tag::FootnoteDefinition(s) => {
                            Tag::FootnoteDefinition(s.into_string().into())
                        }
                        Tag::Table(alignment) => Tag::Table(alignment),
                        Tag::TableHead => Tag::TableHead,
                        Tag::TableRow => Tag::TableRow,
                        Tag::TableCell => Tag::TableCell,
                        Tag::DefinitionList => Tag::DefinitionList,
                        Tag::DefinitionListTitle => Tag::DefinitionListTitle,
                        Tag::DefinitionListDefinition => Tag::DefinitionListDefinition,
                        Tag::HtmlBlock => Tag::HtmlBlock,
                        Tag::MetadataBlock(k) => Tag::MetadataBlock(k),
                        _ => Tag::Paragraph, // safe fallback for any future variants
                    }
                }

                struct FootnoteBlock {
                    frame: egui::Frame,
                    should_scroll: bool,
                    body_events: Vec<(
                        usize,
                        (pulldown_cmark::Event<'static>, std::ops::Range<usize>),
                    )>,
                }

                let mut all_blocks: Vec<FootnoteBlock> = Vec::new();

                // Process the CURRENT block (triggered by `event`)
                let process_block =
                    |note: &str, events: &mut Peekable<_>, ctx: &egui::Context| -> FootnoteBlock {
                        let is_highlighted = ctx.memory(|m| {
                            m.data
                                .get_temp::<String>(active_highlight_key)
                                .map(|active| active == note)
                                .unwrap_or(false)
                        });
                        let should_scroll = ctx.memory_mut(|m| {
                            if let Some(target) = m.data.get_temp::<String>(scroll_key) {
                                if target == note {
                                    m.data.remove_temp::<String>(scroll_key);
                                    return true;
                                }
                            }
                            false
                        });
                        let frame = if is_highlighted {
                            egui::Frame::NONE
                                .fill(ctx.style().visuals.selection.bg_fill.linear_multiply(0.3))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    ctx.style().visuals.selection.bg_fill,
                                ))
                                .inner_margin(egui::Margin {
                                    left: 6,
                                    right: 6,
                                    top: 1,
                                    bottom: 1,
                                })
                                .corner_radius(4.0)
                        } else {
                            egui::Frame::NONE
                        };
                        let mut body_events = Vec::new();
                        while let Some(&(_, (ref e, _))) = events.peek() {
                            let mut done = false;
                            if let pulldown_cmark::Event::Html(t) = e {
                                if t.starts_with("<!-- FOOTNOTE_END_BLOCK:") && t.contains(note) {
                                    done = true;
                                }
                            }
                            if done {
                                events.next();
                                break;
                            }
                            let (idx, (e, span)) = events.next().unwrap();
                            body_events.push((idx, (own_event(e), span)));
                        }
                        FootnoteBlock {
                            frame,
                            should_scroll,
                            body_events,
                        }
                    };

                let first_note = text
                    .trim_start_matches("<!-- FOOTNOTE_START_BLOCK:")
                    .trim_end_matches(" -->");
                all_blocks.push(process_block(first_note, events, ui.ctx()));

                // Peek ahead and collect any immediately following footnote blocks.
                loop {
                    let next_is_footnote = events.peek().map(|(_, (e, _))| {
                        matches!(e, pulldown_cmark::Event::Html(t) if t.starts_with("<!-- FOOTNOTE_START_BLOCK:"))
                    }).unwrap_or(false);
                    if !next_is_footnote {
                        break;
                    }
                    // Consume the FOOTNOTE_START_BLOCK marker event.
                    let (_, (start_event, _)) = events.next().unwrap();
                    if let pulldown_cmark::Event::Html(t) = start_event {
                        let note = t
                            .trim_start_matches("<!-- FOOTNOTE_START_BLOCK:")
                            .trim_end_matches(" -->");
                        let note_owned = note.to_string();
                        all_blocks.push(process_block(&note_owned, events, ui.ctx()));
                    }
                }

                // Render all footnote blocks in a single top_down scope_builder.
                // Use the remaining panel height (not 0) so egui treats the scope as
                // properly bounded. frame.show inside draws around content min_rect so the
                // frame border is correctly sized to the text (not the full panel height).
                self.flush_pending_inline(ui, max_width);
                let footnote_width = max_width;
                let cursor = ui.next_widget_position();
                let outer_rect = egui::Rect::from_min_max(
                    cursor,
                    egui::pos2(
                        cursor.x + footnote_width,
                        ui.max_rect().max.y.max(cursor.y + 1.0),
                    ),
                );
                let td_layout = egui::Layout::top_down(egui::Align::LEFT);
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(outer_rect)
                        .layout(td_layout),
                    |ui| {
                        // Zero vertical spacing so consecutive frame.show blocks touch (fix 5.10).
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for block in all_blocks {
                            if block.should_scroll {
                                ui.scroll_to_cursor(Some(egui::Align::Center));
                            }
                            block.frame.show(ui, |ui| {
                                ui.set_min_width(footnote_width - 12.0);
                                let inner_layout = egui::Layout::left_to_right(egui::Align::Center)
                                    .with_main_wrap(true);
                                ui.with_layout(inner_layout, |ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    // 5.9: use Small text style for footnotes
                                    let small_size = ui.text_style_height(&egui::TextStyle::Small);
                                    ui.style_mut().override_text_style =
                                        Some(egui::TextStyle::Small);
                                    ui.set_row_height(small_size);
                                    let mut iter = block.body_events.into_iter().peekable();
                                    while let Some((_, (e, src_span))) = iter.next() {
                                        // Skip Paragraph start/end: they call newline(ui) which
                                        // adds ~body_font_height gap inside the frame (fix 5.10).
                                        if matches!(
                                            e,
                                            pulldown_cmark::Event::Start(
                                                pulldown_cmark::Tag::Paragraph
                                            ) | pulldown_cmark::Event::End(
                                                pulldown_cmark::TagEnd::Paragraph
                                            )
                                        ) {
                                            continue;
                                        }
                                        if iter.peek().is_none() {
                                            self.line.should_end_newline_forced = false;
                                        }
                                        self.process_event(
                                            ui, &mut iter, e, src_span, cache, options, max_width,
                                        );
                                        self.def_list_def_wrapping(
                                            &mut iter, max_width, cache, options, ui,
                                        );
                                        self.item_list_wrapping(
                                            &mut iter, max_width, cache, options, ui,
                                        );
                                    }
                                });
                            });
                        }
                    },
                );
                newline(ui);
                return;
            }
        }

        self.event(ui, event, src_span, cache, options, max_width);

        self.def_list_def_wrapping(events, max_width, cache, options, ui);
        self.item_list_wrapping(events, max_width, cache, options, ui);
        self.table(events, cache, options, ui, max_width);
        self.blockquote(events, max_width, cache, options, ui);
    }

    fn collect_until_details_close<'e>(
        &mut self,
        events: &mut Peekable<impl Iterator<Item = EventIteratorItem<'e>>>,
    ) -> Vec<EventIteratorItem<'e>> {
        let mut collected = Vec::new();
        let mut depth = 0i32;
        let mut local_html_block = String::new();

        while let Some((i, (event, src_span))) = events.next() {
            self.current_event_idx = i;
            let mut is_closing = false;

            match &event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::HtmlBlock) => {
                    local_html_block.clear();
                }
                pulldown_cmark::Event::Html(text) => {
                    local_html_block.push_str(text);
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::HtmlBlock) => {
                    let trimmed = local_html_block.trim();
                    if extract_details_summary(trimmed).is_some() {
                        depth += 1;
                    } else if trimmed.contains("</details>") {
                        if depth == 0 {
                            is_closing = true;
                        } else {
                            depth -= 1;
                        }
                    }
                }
                _ => {}
            }

            collected.push((i, (event, src_span)));
            if is_closing {
                break;
            }
        }
        collected
    }

    fn def_list_def_wrapping<'e>(
        &mut self,
        events: &mut Peekable<impl Iterator<Item = EventIteratorItem<'e>>>,
        max_width: f32,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        ui: &mut Ui,
    ) {
        if self.def_list.is_def_list_def {
            self.def_list.is_def_list_def = false;

            let item_events = delayed_events(events, |tag| {
                matches!(tag, pulldown_cmark::TagEnd::DefinitionListDefinition)
            });

            let mut events_iter = item_events.into_iter().enumerate().peekable();

            self.line.try_insert_start(ui);

            // Proccess a single event separately so that we do not insert spaces where we do not
            // want them
            self.line.should_start_newline = false;
            if let Some((_, (e, src_span))) = events_iter.next() {
                self.process_event(ui, &mut events_iter, e, src_span, cache, options, max_width);
            }

            ui.label(" ".repeat(options.indentation_spaces));
            self.line.should_start_newline = true;
            self.line.should_end_newline = false;
            // Required to ensure that the content is aligned with the identation
            ui.horizontal_wrapped(|ui| {
                while let Some((_, (e, src_span))) = events_iter.next() {
                    self.process_event(
                        ui,
                        &mut events_iter,
                        e,
                        src_span,
                        cache,
                        options,
                        max_width,
                    );
                }
            });
            self.line.should_end_newline = true;

            // Only end the definition items line if it is not the last element in the list
            if !matches!(
                events.peek(),
                Some((
                    _,
                    (
                        pulldown_cmark::Event::End(pulldown_cmark::TagEnd::DefinitionList),
                        _
                    )
                ))
            ) {
                self.line.try_insert_end(ui);
            }
        }
    }

    fn item_list_wrapping<'e>(
        &mut self,
        events: &mut impl Iterator<Item = EventIteratorItem<'e>>,
        max_width: f32,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        ui: &mut Ui,
    ) {
        if self.is_list_item {
            self.is_list_item = false;

            let item_events = delayed_events_list_item(events);
            let mut events_iter = item_events.into_iter().enumerate().peekable();

            // Required to ensure that the content of the list item is aligned with
            // the * or - when wrapping
            ui.horizontal_wrapped(|ui| {
                // Inside blockquotes, the bullet was NOT added by start_tag(Item) because
                // that would place it in a separate top_down row.  Draw it here so that
                // bullet and text share the same horizontal_wrapped row.
                if self.inside_blockquote {
                    let is_task_list = self.task_list_indices.contains(&self.current_event_idx);
                    self.list.start_item(ui, options, true, is_task_list);
                }
                while let Some((_, (e, src_span))) = events_iter.next() {
                    self.process_event(
                        ui,
                        &mut events_iter,
                        e,
                        src_span,
                        cache,
                        options,
                        max_width,
                    );
                }
            });
        }
    }

    fn blockquote<'e>(
        &mut self,
        events: &mut Peekable<impl Iterator<Item = EventIteratorItem<'e>>>,
        max_width: f32,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        ui: &mut Ui,
    ) {
        if self.is_blockquote {
            let mut collected_events = delayed_events(events, |tag| {
                matches!(tag, pulldown_cmark::TagEnd::BlockQuote(_))
            });

            // MUST reset before the loop. Otherwise process_event -> blockquote()
            // would re-enter here because is_blockquote is still true.
            self.is_blockquote = false;

            // Set the flag to suppress paragraph-level newlines inside the blockquote.
            let was_inside = self.inside_blockquote;
            self.inside_blockquote = true;

            if let Some(alert) = parse_alerts(&options.alerts, &mut collected_events) {
                egui_commonmark_backend::alert_ui(alert, ui, |ui| {
                    let mut events_iter = collected_events.into_iter().enumerate().peekable();
                    while let Some((_, (e, src_span))) = events_iter.next() {
                        self.process_event(
                            ui,
                            &mut events_iter,
                            e,
                            src_span,
                            cache,
                            options,
                            max_width,
                        );
                    }
                })
            } else {
                render_blockquote(ui, ui.visuals().weak_text_color(), |ui| {
                    // Reduce vertical spacing between inner elements for a compact look.
                    ui.spacing_mut().item_spacing.y = 2.0;
                    self.text_style.quote = true;
                    let mut events_iter = collected_events.into_iter().enumerate().peekable();
                    while let Some((_, (e, src_span))) = events_iter.next() {
                        self.process_event(
                            ui,
                            &mut events_iter,
                            e,
                            src_span,
                            cache,
                            options,
                            max_width,
                        );
                    }
                    self.text_style.quote = false;
                });
            }

            self.inside_blockquote = was_inside;
            // Insert spacing after the blockquote so the next element does not glue to it.
            if !was_inside {
                newline(ui);
            }
        }
    }

    fn apply_alignment<R>(
        ui: &mut egui::Ui,
        alignment: &pulldown_cmark::Alignment,
        min_width: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        match alignment {
            pulldown_cmark::Alignment::Center => {
                // Use top_down(Center) layout which centers child widgets
                // horizontally within the available width — analogous to
                // CSS `margin: 0 auto`.
                let layout = egui::Layout::top_down(egui::Align::Center);
                ui.with_layout(layout, |ui| {
                    if min_width > 0.0 {
                        ui.set_min_width(min_width);
                    }
                    add_contents(ui)
                })
            }
            pulldown_cmark::Alignment::Right => {
                let layout = egui::Layout::right_to_left(egui::Align::Center).with_main_wrap(true);
                ui.with_layout(layout, |ui| {
                    if min_width > 0.0 {
                        ui.set_min_width(min_width);
                    }
                    add_contents(ui)
                })
            }
            _ => {
                // Left or None
                let layout = egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true);
                ui.with_layout(layout, |ui| {
                    if min_width > 0.0 {
                        ui.set_min_width(min_width);
                    }
                    add_contents(ui)
                })
            }
        }
    }

    fn table<'e>(
        &mut self,
        events: &mut Peekable<impl Iterator<Item = EventIteratorItem<'e>>>,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        ui: &mut Ui,
        _max_width: f32,
    ) {
        if let Some(alignments) = self.table_alignments.take() {
            self.line.try_insert_start(ui);

            let id = ui.id().with("_table").with(self.curr_table);
            self.curr_table += 1;

            let table_width_key = id.with("_table_width");
            let prev_table_width: Option<f32> = ui.memory(|mem| mem.data.get_temp(table_width_key));

            let Table { header, rows } = parse_table(events);
            let num_cols = header.len().max(1);

            ui.horizontal(|ui| {
                if let Some(prev_w) = prev_table_width {
                    let available = ui.available_width();
                    if available > prev_w {
                        ui.add_space((available - prev_w) / 2.0);
                    }
                }

                let frame_res = egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::symmetric(5, 5))
                    .show(ui, |ui| {
                        let mut col_boundaries = Vec::new();
                        let mut row_bottoms = Vec::new();
                        let mut header_bottom_y = None;

                        ui.spacing_mut().item_spacing.x = 5.0; // 2.5px padding per side
                        ui.spacing_mut().item_spacing.y = 5.0;

                        let table_width = ui.available_width();
                        ui.set_min_width(table_width);

                        let spacing_total =
                            ui.spacing().item_spacing.x * (num_cols.saturating_sub(1) as f32);
                        let min_col = (table_width - spacing_total) / (num_cols as f32);
                        
                        let header_bg_idx = ui.painter().add(egui::Shape::Noop);

                        let _grid_res = egui::Grid::new(id)
                            .num_columns(num_cols)
                            .striped(true)
                            .min_col_width(min_col.max(0.0))
                            .show(ui, |ui| {
                                // ── Header row ──
                                for (col_idx, col) in header.iter().enumerate() {
                                    let alignment = alignments
                                        .get(col_idx)
                                        .unwrap_or(&pulldown_cmark::Alignment::None);

                                    Self::apply_alignment(ui, alignment, min_col, |ui| {
                                        for (e, src_span) in col {
                                            let tmp_start = std::mem::replace(
                                                &mut self.line.should_start_newline,
                                                false,
                                            );
                                            let tmp_end = std::mem::replace(
                                                &mut self.line.should_end_newline,
                                                false,
                                            );
                                            self.event(
                                                ui,
                                                e.clone(),
                                                src_span.clone(),
                                                cache,
                                                options,
                                                ui.available_width(),
                                            );
                                            self.line.should_start_newline = tmp_start;
                                            self.line.should_end_newline = tmp_end;
                                        }
                                        self.flush_pending_inline(ui, ui.available_width());
                                    });

                                    if col_boundaries.len() < num_cols - 1 && col_idx < num_cols - 1
                                    {
                                        col_boundaries.push(
                                            ui.cursor().min.x - ui.spacing().item_spacing.x / 2.0,
                                        );
                                    }
                                }
                                header_bottom_y = Some(
                                    ui.min_rect().bottom() + ui.spacing().item_spacing.y / 2.0,
                                );
                                ui.end_row();

                                // ── Body rows ──
                                for (row_idx, row_data) in rows.iter().enumerate() {
                                    for col_idx in 0..num_cols {
                                        if let Some(col_data) = row_data.get(col_idx) {
                                            let alignment = alignments
                                                .get(col_idx)
                                                .unwrap_or(&pulldown_cmark::Alignment::None);

                                            Self::apply_alignment(ui, alignment, min_col, |ui| {
                                                for (e, src_span) in col_data {
                                                    let tmp_start = std::mem::replace(
                                                        &mut self.line.should_start_newline,
                                                        false,
                                                    );
                                                    let tmp_end = std::mem::replace(
                                                        &mut self.line.should_end_newline,
                                                        false,
                                                    );
                                                    self.event(
                                                        ui,
                                                        e.clone(),
                                                        src_span.clone(),
                                                        cache,
                                                        options,
                                                        ui.available_width(),
                                                    );
                                                    self.line.should_start_newline = tmp_start;
                                                    self.line.should_end_newline = tmp_end;
                                                }
                                                self.flush_pending_inline(ui, ui.available_width());
                                            });
                                        } else {
                                            ui.label("");
                                        }
                                    }
                                    if row_idx < rows.len() - 1 {
                                        row_bottoms.push(
                                            ui.min_rect().bottom()
                                                + ui.spacing().item_spacing.y / 2.0,
                                        );
                                    }
                                    ui.end_row();
                                }
                            });

                        // Draw vertical and horizontal separators
                        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
                        let visual_rect = ui.min_rect();
                        
                        if let Some(y) = header_bottom_y {
                            let header_bg_rect = egui::Rect::from_min_max(
                                egui::pos2(visual_rect.left(), visual_rect.top()),
                                egui::pos2(visual_rect.right(), y),
                            );
                            const TABLE_HEADER_ALPHA: f32 = 0.3;
                            ui.painter().set(
                                header_bg_idx,
                                egui::Shape::rect_filled(
                                    header_bg_rect,
                                    0.0,
                                    ui.visuals().selection.bg_fill.gamma_multiply(TABLE_HEADER_ALPHA),
                                ),
                            );
                        }

                        for x in col_boundaries {
                            ui.painter().vline(x, visual_rect.y_range(), stroke);
                        }
                        if let (Some(y), false) = (header_bottom_y, rows.is_empty()) {
                            let header_stroke = egui::Stroke::new(
                                1.0,
                                ui.visuals().text_color().gamma_multiply(0.4),
                            );
                            ui.painter().hline(visual_rect.x_range(), y, header_stroke);
                        }
                        for y in row_bottoms {
                            ui.painter().hline(visual_rect.x_range(), y, stroke);
                        }
                    });

                let current_table_width = frame_res.response.rect.width();
                if prev_table_width.map_or(true, |pw| (pw - current_table_width).abs() > 0.1) {
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(table_width_key, current_table_width);
                    });
                    ui.ctx().request_repaint();
                }
            });

            if events.peek().is_none() {
                self.line.should_end_newline_forced = false;
            }

            self.line.try_insert_end(ui);
        }
    }

    fn event(
        &mut self,
        ui: &mut Ui,
        event: pulldown_cmark::Event,
        src_span: Range<usize>,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        max_width: f32,
    ) {
        match event {
            pulldown_cmark::Event::Start(tag) => {
                if Self::should_flush_before_start_tag(&tag) {
                    self.flush_pending_inline(ui, max_width);
                }
                let start_y = ui.next_widget_position().y;
                self.block_states.push((start_y, src_span.clone()));
                self.start_tag(ui, tag, options)
            }
            pulldown_cmark::Event::End(tag) => {
                if Self::should_flush_before_end_tag(&tag) {
                    self.flush_pending_inline(ui, max_width);
                }
                self.end_tag(ui, tag.clone(), cache, options, max_width);
                
                if let Some((start_y, span)) = self.block_states.pop() {
                    let end_y = ui.next_widget_position().y;
                    if end_y > start_y {
                        let rect = egui::Rect::from_min_max(
                            egui::pos2(ui.min_rect().left(), start_y),
                            egui::pos2(ui.min_rect().right(), end_y),
                        );
                        if let Some(active) = &self.active_char_range {
                            if active.start <= span.end && active.end >= span.start {
                                self.active_rects.push((rect, span.clone()));
                                let highlight_color = self.active_bg_color.unwrap_or_else(|| {
                                    if ui.visuals().dark_mode {
                                        egui::Color32::from_white_alpha(15)
                                    } else {
                                        egui::Color32::from_black_alpha(15)
                                    }
                                });
                                ui.painter().rect_filled(rect, 1.0, highlight_color);
                            }
                        }
                        if let Some(hovered) = &mut self.hovered_spans {
                            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                                if rect.contains(pos) {
                                    hovered.push(span.clone());
                                    let hover_color = self.hover_bg_color.unwrap_or_else(|| {
                                        if ui.visuals().dark_mode {
                                            egui::Color32::from_white_alpha(8)
                                        } else {
                                            egui::Color32::from_black_alpha(8)
                                        }
                                    });
                                    ui.painter().rect_filled(rect, 1.0, hover_color);
                                }
                            }
                        }
                    }
                }
            }
            pulldown_cmark::Event::Text(text) => {
                self.event_text(text, ui, max_width);
            }
            pulldown_cmark::Event::Code(text) => {
                self.text_style.code = true;
                self.event_text(text, ui, max_width);
                self.text_style.code = false;
            }
            pulldown_cmark::Event::InlineHtml(text) => {
                let trimmed = text.trim();
                if trimmed.eq_ignore_ascii_case("<u>") {
                    self.text_style.underline = true;
                } else if trimmed.eq_ignore_ascii_case("</u>") {
                    self.text_style.underline = false;
                } else if trimmed.eq_ignore_ascii_case("<mark>") {
                    self.text_style.highlight = true;
                } else if trimmed.eq_ignore_ascii_case("</mark>") {
                    self.text_style.highlight = false;
                } else {
                    self.event_text(text, ui, max_width);
                }
            }

            pulldown_cmark::Event::Html(text) => {
                if let Some(task_state_str) = text.strip_prefix("<!-- KATANA_TASK:") {
                    if let Some(task_state_str) = task_state_str.strip_suffix(" -->") {
                        if !task_state_str.is_empty() {
                            let state_char = task_state_str.chars().next().unwrap();
                            self.after_inline_widget = false;
                            self.flush_pending_inline(ui, max_width);
                            
                            // Activate context menu state for subsequent text events in this list item
                            self.active_task_context_menu = Some((state_char, src_span.clone(), options.mutable));
                            
                            crate::ui_components::task_list::katana_task_box(
                                ui, 
                                state_char, 
                                src_span, 
                                options.mutable, 
                                &mut self.checkbox_events
                            );
                            return;
                        }
                    }
                }

                if options.html_fn.is_some() {
                    self.flush_pending_inline(ui, max_width);
                    self.html_block.push_str(&text);
                } else {
                    self.event_text(text, ui, max_width);
                }
            }
            pulldown_cmark::Event::FootnoteReference(footnote) => {
                self.after_inline_widget = false;
                self.flush_pending_inline(ui, max_width);

                let scroll_back_key = egui::Id::new("scroll_to_footnote_ref");
                let should_scroll_back = ui.memory_mut(|m| {
                    if let Some(target) = m.data.get_temp::<String>(scroll_back_key) {
                        if target == footnote.as_ref() {
                            m.data.remove_temp::<String>(scroll_back_key);
                            return true;
                        }
                    }
                    false
                });

                if should_scroll_back {
                    ui.scroll_to_cursor(Some(egui::Align::Center));
                }

                let num_str = match footnote.as_ref().parse::<u32>() {
                    Ok(n) => format!("[{n}]"),
                    Err(_) => format!("[{}]", footnote.as_ref()),
                };

                let text = RichText::new(num_str)
                    .raised()
                    .small()
                    .color(ui.visuals().hyperlink_color)
                    .underline();
                let mut resp = ui.add(egui::Label::new(text).sense(egui::Sense::click()));
                resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

                if resp.clicked() {
                    let scroll_key = egui::Id::new("scroll_to_footnote");
                    ui.ctx()
                        .memory_mut(|m| m.data.insert_temp(scroll_key, footnote.to_string()));
                    // Use a single "active highlight" key so only one footnote is highlighted
                    // at a time, preventing accumulation on back/forth navigation (fix 5.7).
                    let active_key = egui::Id::new("highlight_footnote_active");
                    ui.ctx()
                        .memory_mut(|m| m.data.insert_temp(active_key, footnote.to_string()));
                }
            }
            pulldown_cmark::Event::SoftBreak => {
                self.after_inline_widget = false;
                self.event_text(CowStr::Borrowed(" "), ui, max_width);
            }
            pulldown_cmark::Event::HardBreak => {
                self.after_inline_widget = false;
                self.flush_pending_inline(ui, max_width);
                newline(ui)
            }
            pulldown_cmark::Event::Rule => {
                self.after_inline_widget = false;
                self.flush_pending_inline(ui, max_width);
                self.line.try_insert_start(ui);
                rule(ui, self.line.can_insert_end());
            }
            pulldown_cmark::Event::TaskListMarker(mut checkbox) => {
                self.after_inline_widget = false;
                self.flush_pending_inline(ui, max_width);
                if options.mutable {
                    let mut is_checked = checkbox;
                    let response = ui.add(egui::Checkbox::without_text(&mut is_checked));
                    
                    if response.clicked() {
                        // For generic checkboxes, we just emit x or space
                        let new_state = if !checkbox { 'x' } else { ' ' };
                        self.checkbox_events.push(crate::TaskListAction {
                            new_state,
                            span: src_span,
                        });
                    }
                } else {
                    ui.add(ImmutableCheckbox::without_text(&mut checkbox));
                }
            }
            pulldown_cmark::Event::InlineMath(tex) => {
                if let Some(math_fn) = options.math_fn {
                    self.after_inline_widget = false;
                    self.flush_pending_inline(ui, max_width);
                    math_fn(ui, &tex, true);
                }
            }
            pulldown_cmark::Event::DisplayMath(tex) => {
                if let Some(math_fn) = options.math_fn {
                    self.after_inline_widget = false;
                    self.flush_pending_inline(ui, max_width);
                    math_fn(ui, &tex, false);
                }
            }
        }
    }

    fn event_text(&mut self, text: CowStr, ui: &mut Ui, max_width: f32) {
        if let Some(image) = &mut self.image {
            self.after_inline_widget = false;
            image.alt_text.push(self.text_style.to_richtext(ui, &text));
        } else if let Some(block) = &mut self.code_block {
            self.after_inline_widget = false;
            block.content.push_str(&text);
        } else if let Some(link) = &mut self.link {
            self.after_inline_widget = false;
            link.text.push(self.text_style.to_richtext(ui, &text));
        } else {
            for segment in split_inline_text_and_emoji(&text) {
                match segment {
                    InlineSegment::Text(text) => self.push_inline_text(text, ui, max_width),
                    InlineSegment::Emoji(grapheme) => {
                        if !self.try_render_inline_emoji(ui, max_width, grapheme) {
                            self.push_inline_text(grapheme, ui, max_width);
                        }
                    }
                }
            }
        }
    }

    fn start_tag(&mut self, ui: &mut Ui, tag: pulldown_cmark::Tag, options: &CommonMarkOptions) {
        match tag {
            pulldown_cmark::Tag::Paragraph => {
                if !self.inside_blockquote {
                    self.line.try_insert_start(ui);
                }
            }
            pulldown_cmark::Tag::Heading { level, .. } => {
                // Headings should always insert a newline even if it is at the start.
                // Whether this is okay in all scenarios is a different question.
                newline(ui);

                if let Some(target_idx) = self.scroll_to_heading_index {
                    if target_idx == self.curr_heading {
                        ui.scroll_to_cursor(Some(egui::Align::TOP));
                    }
                }

                self.text_style.heading = Some(match level {
                    HeadingLevel::H1 => 0,
                    HeadingLevel::H2 => 1,
                    HeadingLevel::H3 => 2,
                    HeadingLevel::H4 => 3,
                    HeadingLevel::H5 => 4,
                    HeadingLevel::H6 => 5,
                });
            }

            // deliberately not using the built in alerts from pulldown-cmark as
            // the markdown itself cannot be localized :( e.g: [!TIP]
            pulldown_cmark::Tag::BlockQuote(_) => {
                self.is_blockquote = true;
            }
            pulldown_cmark::Tag::CodeBlock(c) => {
                match c {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        self.code_block = Some(CodeBlock {
                            lang: Some(lang.to_string()),
                            content: "".to_string(),
                        });
                    }
                    pulldown_cmark::CodeBlockKind::Indented => {
                        self.code_block = Some(CodeBlock {
                            lang: None,
                            content: "".to_string(),
                        });
                    }
                }
                if !self.inside_blockquote {
                    self.line.try_insert_start(ui);
                }
            }

            pulldown_cmark::Tag::List(point) => {
                if !self.inside_blockquote
                    && !self.list.is_inside_a_list()
                    && self.line.can_insert_start()
                {
                    newline(ui);
                }

                if let Some(number) = point {
                    self.list.start_level_with_number(number);
                } else {
                    self.list.start_level_without_number();
                }
                self.line.should_start_newline = false;
                self.line.should_end_newline = false;
            }

            pulldown_cmark::Tag::Item => {
                // Clear any active task context if a new item starts (safety)
                self.active_task_context_menu = None;
                self.is_list_item = true;
                let is_task_list = self.task_list_indices.contains(&self.current_event_idx);

                if !self.inside_blockquote {
                    self.list.start_item(ui, options, false, is_task_list);
                }
            }

            pulldown_cmark::Tag::FootnoteDefinition(note) => {
                self.line.try_insert_start(ui);

                self.line.should_start_newline = false;
                self.line.should_end_newline = false;

                let scroll_key = egui::Id::new("scroll_to_footnote");
                let should_scroll = ui.ctx().memory_mut(|m| {
                    if let Some(target) = m.data.get_temp::<String>(scroll_key) {
                        if target == note.as_ref() {
                            m.data.remove_temp::<String>(scroll_key);
                            return true;
                        }
                    }
                    false
                });

                if should_scroll {
                    ui.scroll_to_cursor(Some(egui::Align::Center));
                }
            }
            pulldown_cmark::Tag::Table(alignments) => {
                self.table_alignments = Some(alignments);
            }
            pulldown_cmark::Tag::TableHead => {}
            pulldown_cmark::Tag::TableRow => {}
            pulldown_cmark::Tag::TableCell => {}
            pulldown_cmark::Tag::Emphasis => {
                self.text_style.emphasis = true;
            }
            pulldown_cmark::Tag::Strong => {
                self.text_style.strong = true;
            }
            pulldown_cmark::Tag::Strikethrough => {
                self.text_style.strikethrough = true;
            }
            pulldown_cmark::Tag::Link { dest_url, .. } => {
                self.link = Some(Link {
                    destination: dest_url.to_string(),
                    text: Vec::new(),
                });
            }
            pulldown_cmark::Tag::Image { dest_url, .. } => {
                self.image = Some(Image::new(&dest_url, options));
            }
            pulldown_cmark::Tag::HtmlBlock => {
                self.line.try_insert_start(ui);
            }
            pulldown_cmark::Tag::MetadataBlock(_) => {}

            pulldown_cmark::Tag::DefinitionList => {
                self.line.try_insert_start(ui);
                self.def_list.is_first_item = true;
            }
            pulldown_cmark::Tag::DefinitionListTitle => {
                // we disable newline as the first title should not insert a newline
                // as we have already done that upon the DefinitionList Tag
                if !self.def_list.is_first_item {
                    self.line.try_insert_start(ui)
                } else {
                    self.def_list.is_first_item = false;
                }
            }
            pulldown_cmark::Tag::DefinitionListDefinition => {
                self.def_list.is_def_list_def = true;
            }
            // Not yet supported
            pulldown_cmark::Tag::Superscript | pulldown_cmark::Tag::Subscript => {}
        }
    }

    fn end_tag(
        &mut self,
        ui: &mut Ui,
        tag: pulldown_cmark::TagEnd,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        max_width: f32,
    ) {
        match tag {
            pulldown_cmark::TagEnd::Paragraph => {
                if !self.inside_blockquote {
                    self.line.try_insert_end(ui);
                }
            }
            pulldown_cmark::TagEnd::Heading { .. } => {
                self.line.try_insert_end(ui);
                self.text_style.heading = None;

                let last_state = self.block_states.last().cloned();
                if let (Some(anchors), Some((start_y, span))) =
                    (&mut self.heading_anchors, last_state)
                {
                    let end_y = ui.next_widget_position().y;
                    let width = ui.available_width();
                    let min_x = ui.min_rect().left();
                    anchors.push((
                        span,
                        egui::Rect::from_min_max(
                            egui::pos2(min_x, start_y),
                            egui::pos2(min_x + width, end_y),
                        ),
                    ));
                }
                self.curr_heading += 1;
            }
            pulldown_cmark::TagEnd::BlockQuote(_) => {}
            pulldown_cmark::TagEnd::CodeBlock => {
                self.end_code_block(ui, cache, options, max_width);
            }

            pulldown_cmark::TagEnd::List(_) => {
                if self.list.is_last_level() {
                    self.line.should_start_newline = true;
                    self.line.should_end_newline = true;
                }

                let insert_nl = !self.inside_blockquote && self.line.can_insert_end();
                self.list.end_level(ui, insert_nl);

                if !self.list.is_inside_a_list() {
                    // Reset all the state and make it ready for the next list that occurs
                    self.list = List::default();
                }
            }
            pulldown_cmark::TagEnd::Item => {
                // Deactivate the task context menu when the item ends
                self.active_task_context_menu = None;
            }
            pulldown_cmark::TagEnd::FootnoteDefinition => {
                self.line.should_start_newline = true;
                self.line.should_end_newline = true;
                self.line.try_insert_end(ui);
            }
            pulldown_cmark::TagEnd::Table => {}
            pulldown_cmark::TagEnd::TableHead => {}
            pulldown_cmark::TagEnd::TableRow => {}
            pulldown_cmark::TagEnd::TableCell => {}
            pulldown_cmark::TagEnd::Emphasis => {
                self.text_style.emphasis = false;
            }
            pulldown_cmark::TagEnd::Strong => {
                self.text_style.strong = false;
            }
            pulldown_cmark::TagEnd::Strikethrough => {
                self.text_style.strikethrough = false;
            }
            pulldown_cmark::TagEnd::Link => {
                if let Some(link) = self.link.take() {
                    let dest = link.destination.clone();
                    if let Some(target) = dest.strip_prefix("scroll-to-footnote-ref:") {
                        // Render ↩ in hyperlink_color (blue) without underline (fix 5.8).
                        let hyperlink_color = ui.visuals().hyperlink_color;
                        let mut layout_job = egui::text::LayoutJob::default();
                        for t in link.text {
                            let section_start = layout_job.text.len();
                            t.append_to(
                                &mut layout_job,
                                ui.style(),
                                egui::FontSelection::Default,
                                egui::Align::LEFT,
                            );
                            for section in layout_job.sections.iter_mut() {
                                if section.byte_range.start >= section_start {
                                    section.format.color = hyperlink_color;
                                    section.format.underline = egui::Stroke::NONE;
                                }
                            }
                        }
                        let resp = ui.link(layout_job);
                        if resp.clicked() {
                            let back_key = egui::Id::new("scroll_to_footnote_ref");
                            ui.ctx()
                                .memory_mut(|m| m.data.insert_temp(back_key, target.to_string()));

                            // Clear the single active-highlight key when returning (fix 5.7)
                            let active_key = egui::Id::new("highlight_footnote_active");
                            ui.ctx()
                                .memory_mut(|m| m.data.remove_temp::<String>(active_key));
                        }
                    } else {
                        link.end(ui, cache);
                    }
                    self.after_inline_widget = true;
                }
            }
            pulldown_cmark::TagEnd::Image => {
                if let Some(image) = self.image.take() {
                    image.end(ui, options);
                    self.after_inline_widget = true;
                }
            }
            pulldown_cmark::TagEnd::HtmlBlock => {
                let block = std::mem::take(&mut self.html_block);
                let trimmed = block.trim();

                // Detect <details><summary>...</summary> opening block
                if let Some(summary) = extract_details_summary(trimmed) {
                    self.details_summary = Some(summary);
                    self.details_id_counter += 1;
                } else if trimmed.contains("</details>") {
                    // Closing block: reset details state
                    self.details_summary = None;
                } else if let Some(html_fn) = options.html_fn {
                    // Regular HTML block — delegate to the callback
                    html_fn(ui, &block);
                    self.line.try_insert_end(ui);
                }
            }

            pulldown_cmark::TagEnd::MetadataBlock(_) => {}

            pulldown_cmark::TagEnd::DefinitionList => self.line.try_insert_end(ui),
            pulldown_cmark::TagEnd::DefinitionListTitle
            | pulldown_cmark::TagEnd::DefinitionListDefinition => {}
            pulldown_cmark::TagEnd::Superscript | pulldown_cmark::TagEnd::Subscript => {}
        }
    }

    fn end_code_block(
        &mut self,
        ui: &mut Ui,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        max_width: f32,
    ) {
        if let Some(block) = self.code_block.take() {
            // Route ```math blocks through render_math_fn so they are treated as
            // display math instead of being rendered as a fenced code block.
            let is_math = block
                .lang
                .as_deref()
                .map(|l| l.eq_ignore_ascii_case("math"))
                .unwrap_or(false);

            if is_math {
                if let Some(math_fn) = options.math_fn {
                    self.line.try_insert_start(ui);
                    math_fn(ui, block.content.trim(), false);
                    if !self.inside_blockquote {
                        self.line.try_insert_end(ui);
                    }
                    return;
                }
            }

            block.end(ui, cache, options, max_width);
            if !self.inside_blockquote {
                self.line.try_insert_end(ui);
            }
        }
    }
}

enum InlineSegment<'a> {
    Text(&'a str),
    Emoji(&'a str),
}

fn split_inline_text_and_emoji(text: &str) -> Vec<InlineSegment<'_>> {
    let mut segments = Vec::new();
    let mut text_start = 0usize;

    for (idx, grapheme) in text.grapheme_indices(true) {
        if is_emoji_grapheme(grapheme) {
            if text_start < idx {
                segments.push(InlineSegment::Text(&text[text_start..idx]));
            }
            segments.push(InlineSegment::Emoji(grapheme));
            text_start = idx + grapheme.len();
        }
    }

    if text_start < text.len() {
        segments.push(InlineSegment::Text(&text[text_start..]));
    }

    if segments.is_empty() {
        segments.push(InlineSegment::Text(text));
    }

    segments
}

fn is_emoji_grapheme(grapheme: &str) -> bool {
    grapheme.chars().any(is_emoji_scalar)
}

fn is_emoji_scalar(ch: char) -> bool {
    matches!(
        ch as u32,
        0x2600..=0x27BF
            | 0x1F000..=0x1FAFF
            | 0x1FC00..=0x1FFFD
    )
}

#[allow(dead_code)]
fn inline_emoji_uri(grapheme: &str, pixel_size: u32) -> String {
    let mut hasher = DefaultHasher::new();
    grapheme.hash(&mut hasher);
    pixel_size.hash(&mut hasher);
    format!(
        "{INLINE_EMOJI_URI_PREFIX}{:016x}{INLINE_EMOJI_FILENAME_SUFFIX}",
        hasher.finish()
    )
}

/// Extracts the summary text from a `<details><summary>...</summary>` HTML block.
///
/// Returns `Some(summary_text)` if the block starts with `<details>` and contains
/// a `<summary>...</summary>` pair. Returns `None` otherwise.
fn extract_details_summary(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    if !lower.starts_with("<details") {
        return None;
    }
    let summary_start = lower.find("<summary>")?;
    let summary_end = lower.find("</summary>")?;
    if summary_end <= summary_start {
        return None;
    }
    let start = summary_start + "<summary>".len();
    Some(html[start..summary_end].trim().to_string())
}

pub(crate) fn extract_custom_task_lists<'e>(
    events: Vec<(pulldown_cmark::Event<'e>, std::ops::Range<usize>)>,
) -> (Vec<(pulldown_cmark::Event<'e>, std::ops::Range<usize>)>, std::collections::HashSet<usize>) {
    let mut out = Vec::with_capacity(events.len());
    let mut task_list_indices = std::collections::HashSet::new();
    let mut i = 0;
    while i < events.len() {
        if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item) = &events[i].0 {
            let item_start_idx = out.len();
            out.push(events[i].clone());

            // Check for native TaskListMarker
            if i + 1 < events.len() {
                if let pulldown_cmark::Event::TaskListMarker(checked) = &events[i + 1].0 {
                    task_list_indices.insert(item_start_idx);
                    let c = if *checked { 'x' } else { ' ' };
                    let span = events[i + 1].1.clone();
                    let marker = format!("<!-- KATANA_TASK:{} -->", c);
                    out.push((pulldown_cmark::Event::Html(marker.into()), span));
                    i += 2;
                    continue;
                }
            }

            // Check for custom task list e.g. [/], [-], [~] as Text
            if i + 3 < events.len() {
                if let (pulldown_cmark::Event::Text(t1), span1) = &events[i + 1] {
                    if let (pulldown_cmark::Event::Text(t2), _) = &events[i + 2] {
                        if let (pulldown_cmark::Event::Text(t3), span3) = &events[i + 3] {
                            if t1.as_ref() == "[" && t3.as_ref() == "]" && t2.as_ref().len() == 1 {
                                task_list_indices.insert(item_start_idx);
                                let c = t2.as_ref().chars().next().unwrap();
                                let merged_span = span1.start..span3.end;
                                let marker = format!("<!-- KATANA_TASK:{} -->", c);
                                out.push((pulldown_cmark::Event::Html(marker.into()), merged_span));

                                i += 4;
                                // Strip leading space from the subsequent text event
                                if i < events.len() {
                                    if let (pulldown_cmark::Event::Text(t4), span4) = &events[i] {
                                        if t4.as_ref().starts_with(' ') {
                                            let mut new_text = t4.as_ref().to_string();
                                            new_text.remove(0); // slice off leading space
                                            let new_span = (span4.start + 1)..span4.end;
                                            if !new_text.is_empty() {
                                                out.push((pulldown_cmark::Event::Text(new_text.into()), new_span));
                                            }
                                            i += 1;
                                            continue;
                                        }
                                    }
                                }
                                continue;
                            }
                        }
                    }
                }
            }
        } else {
            out.push(events[i].clone());
        }
        i += 1;
    }
    (out, task_list_indices)
}

fn extract_footnotes<'e>(
    raw_events: impl Iterator<Item = (pulldown_cmark::Event<'e>, std::ops::Range<usize>)>,
) -> Vec<(pulldown_cmark::Event<'e>, std::ops::Range<usize>)> {
    let mut main_events = Vec::new();
    let mut footnote_events = Vec::new();
    let mut in_footnote = 0;
    let mut current_footnote = String::new();
    let mut is_first_paragraph_of_footnote = false;

    for (event, span) in raw_events {
        match &event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::FootnoteDefinition(note)) => {
                in_footnote += 1;
                current_footnote = note.to_string();
                is_first_paragraph_of_footnote = true;
                let start_html = format!("<!-- FOOTNOTE_START_BLOCK:{} -->", current_footnote);
                footnote_events
                    .push((pulldown_cmark::Event::Html(start_html.into()), span.clone()));
            }
            pulldown_cmark::Event::End(pulldown_cmark::TagEnd::FootnoteDefinition) => {
                let return_url = format!("scroll-to-footnote-ref:{}", current_footnote);
                let pos = footnote_events.iter().rposition(|(e, _)| {
                    matches!(
                        e,
                        pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Paragraph)
                    )
                });

                let s_space = (pulldown_cmark::Event::Text("  ".into()), 0..0);
                let link_start = (
                    pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link {
                        link_type: pulldown_cmark::LinkType::Inline,
                        dest_url: return_url.into(),
                        title: "".into(),
                        id: "".into(),
                    }),
                    0..0,
                );
                let link_text = (pulldown_cmark::Event::Text("↩".into()), 0..0);
                let link_end = (
                    pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Link),
                    0..0,
                );

                if let Some(p) = pos {
                    footnote_events.insert(p, s_space);
                    footnote_events.insert(p + 1, link_start);
                    footnote_events.insert(p + 2, link_text);
                    footnote_events.insert(p + 3, link_end);
                } else {
                    footnote_events.push(s_space);
                    footnote_events.push(link_start);
                    footnote_events.push(link_text);
                    footnote_events.push(link_end);
                }

                let end_html = format!("<!-- FOOTNOTE_END_BLOCK:{} -->", current_footnote);
                footnote_events.push((pulldown_cmark::Event::Html(end_html.into()), 0..0));

                in_footnote -= 1;
            }
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Paragraph) => {
                if in_footnote > 0 {
                    footnote_events.push((event.clone(), span.clone()));
                    if is_first_paragraph_of_footnote {
                        let label = match current_footnote.parse::<u32>() {
                            Ok(n) => format!("{n}. "),
                            Err(_) => format!("{}. ", current_footnote),
                        };
                        footnote_events.push((pulldown_cmark::Event::Text(label.into()), 0..0));
                        is_first_paragraph_of_footnote = false;
                    }
                } else {
                    main_events.push((event.clone(), span.clone()));
                }
            }
            _ => {
                if in_footnote > 0 {
                    footnote_events.push((event.clone(), span.clone()));
                } else {
                    main_events.push((event.clone(), span.clone()));
                }
            }
        }
    }

    if !footnote_events.is_empty() {
        main_events.push((pulldown_cmark::Event::Rule, 0..0));
        main_events.extend(footnote_events);
    }
    main_events
}
