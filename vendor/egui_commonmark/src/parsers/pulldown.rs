use std::collections::hash_map::DefaultHasher;
use std::iter::Peekable;
use std::ops::Range;
use std::hash::{Hash, Hasher};

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

pub(crate) struct CheckboxClickEvent {
    pub(crate) checked: bool,
    pub(crate) span: Range<usize>,
}

pub(crate) struct CommonMarkViewerInternal<'a> {
    curr_table: usize,
    curr_heading: usize,
    curr_heading_start_y: Option<f32>,
    scroll_to_heading_index: Option<usize>,
    populate_heading_rects: Option<&'a mut Vec<egui::Rect>>,
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
    checkbox_events: Vec<CheckboxClickEvent>,
}

impl<'a> CommonMarkViewerInternal<'a> {
    pub fn new(
        scroll_to_heading_index: Option<usize>,
        populate_heading_rects: Option<&'a mut Vec<egui::Rect>>,
        heading_offset: usize,
    ) -> Self {
        Self {
            curr_table: 0,
            curr_heading: heading_offset,
            curr_heading_start_y: None,
            scroll_to_heading_index,
            populate_heading_rects,
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
        let mut layout_job = egui::text::LayoutJob::default();
        layout_job.halign = egui::Align::LEFT;
        layout_job.wrap.max_width = max_width;
        layout_job.wrap.break_anywhere = true;
        for rich_text in &pending_inline {
            rich_text.clone().append_to(
                &mut layout_job,
                &style,
                egui::FontSelection::Default,
                egui::Align::BOTTOM,
            );
        }

        ui.add(egui::Label::new(layout_job).wrap().halign(egui::Align::LEFT));
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
            if let Some(bytes) = katana_core::emoji::render_apple_color_emoji_png(grapheme, pixel_size) {
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

    /// Be aware that this acquires egui::Context internally.
    /// If split Id is provided then split points will be populated
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        options: &CommonMarkOptions,
        text: &str,
        split_points_id: Option<Id>,
    ) -> (egui::InnerResponse<()>, Vec<CheckboxClickEvent>) {
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

            let mut events = pulldown_cmark::Parser::new_ext(
                text,
                parser_options_math(options.math_fn.is_some()),
            )
            .into_offset_iter()
            .enumerate()
            .peekable();

            while let Some((index, (e, src_span))) = events.next() {
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
                            scroll_cache
                                .split_points
                                .push((index, start_position, end_position));
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
        });

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

        let events =
            pulldown_cmark::Parser::new_ext(text, parser_options_math(options.math_fn.is_some()))
                .into_offset_iter()
                .collect::<Vec<_>>();

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
                        if events.peek().is_none() {
                            self.line.should_end_newline_forced = false;
                        }

                        self.process_event(ui, &mut events, e, src_span, cache, options, max_width);

                        if i == 0 {
                            self.line.should_not_start_newline_forced = false;
                        }
                    }
                });
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
        self.event(ui, event, src_span, cache, options, max_width);

        self.def_list_def_wrapping(events, max_width, cache, options, ui);
        self.item_list_wrapping(events, max_width, cache, options, ui);
        self.table(events, cache, options, ui, max_width);
        self.blockquote(events, max_width, cache, options, ui);
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
                    self.list.start_item(ui, options, true);
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
        let layout = match alignment {
            pulldown_cmark::Alignment::None | pulldown_cmark::Alignment::Center => {
                egui::Layout::left_to_right(egui::Align::Center)
                    .with_main_wrap(true)
                    .with_main_align(egui::Align::Center)
            }
            pulldown_cmark::Alignment::Left => {
                egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true)
            }
            pulldown_cmark::Alignment::Right => {
                egui::Layout::right_to_left(egui::Align::Center).with_main_wrap(true)
            }
        };
        ui.with_layout(layout, |ui| {
            if min_width > 0.0 {
                ui.set_min_width(min_width);
            }
            add_contents(ui)
        })
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
            let prev_table_width: Option<f32> =
                ui.memory(|mem| mem.data.get_temp(table_width_key));

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

                        let _grid_res = egui::Grid::new(id)
                            .num_columns(num_cols)
                            .striped(true)
                            .min_col_width(min_col.max(0.0))
                            .show(ui, |ui| {
                            // ── Header row ──
                            for (col_idx, col) in header.iter().enumerate() {
                                let alignment = alignments.get(col_idx).unwrap_or(&pulldown_cmark::Alignment::None);

                                Self::apply_alignment(ui, alignment, min_col, |ui| {
                                    for (e, src_span) in col {
                                        let tmp_start = std::mem::replace(&mut self.line.should_start_newline, false);
                                        let tmp_end = std::mem::replace(&mut self.line.should_end_newline, false);
                                        self.event(ui, e.clone(), src_span.clone(), cache, options, ui.available_width());
                                        self.line.should_start_newline = tmp_start;
                                        self.line.should_end_newline = tmp_end;
                                    }
                                    self.flush_pending_inline(ui, ui.available_width());
                                });

                                if col_boundaries.len() < num_cols - 1 && col_idx < num_cols - 1 {
                                    col_boundaries.push(ui.cursor().min.x - ui.spacing().item_spacing.x / 2.0);
                                }
                            }
                            header_bottom_y = Some(ui.min_rect().bottom() + ui.spacing().item_spacing.y / 2.0);
                            ui.end_row();

                            // ── Body rows ──
                            for (row_idx, row_data) in rows.iter().enumerate() {
                                for col_idx in 0..num_cols {
                                    if let Some(col_data) = row_data.get(col_idx) {
                                        let alignment = alignments.get(col_idx).unwrap_or(&pulldown_cmark::Alignment::None);

                                        Self::apply_alignment(ui, alignment, min_col, |ui| {
                                            for (e, src_span) in col_data {
                                                let tmp_start = std::mem::replace(
                                                    &mut self.line.should_start_newline,
                                                    false,
                                                );
                                                let tmp_end = std::mem::replace(&mut self.line.should_end_newline, false);
                                                self.event(ui, e.clone(), src_span.clone(), cache, options, ui.available_width());
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
                                    row_bottoms.push(ui.min_rect().bottom() + ui.spacing().item_spacing.y / 2.0);
                                }
                                ui.end_row();
                            }
                        });

                    // Draw vertical and horizontal separators
                    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
                    let visual_rect = ui.min_rect();
                    for x in col_boundaries {
                        ui.painter().vline(x, visual_rect.y_range(), stroke);
                    }
                    if let (Some(y), false) = (header_bottom_y, rows.is_empty()) {
                        let header_stroke = egui::Stroke::new(1.0, ui.visuals().text_color().gamma_multiply(0.4));
                        ui.painter().hline(visual_rect.x_range(), y, header_stroke);
                    }
                    for y in row_bottoms {
                        ui.painter().hline(visual_rect.x_range(), y, stroke);
                    }
                });

                let current_table_width = frame_res.response.rect.width();
                if prev_table_width.map_or(true, |pw| (pw - current_table_width).abs() > 0.1) {
                    ui.memory_mut(|mem| { mem.data.insert_temp(table_width_key, current_table_width); });
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
                self.start_tag(ui, tag, options)
            }
            pulldown_cmark::Event::End(tag) => {
                if Self::should_flush_before_end_tag(&tag) {
                    self.flush_pending_inline(ui, max_width);
                }
                self.end_tag(ui, tag, cache, options, max_width)
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
                self.event_text(text, ui, max_width);
            }

            pulldown_cmark::Event::Html(text) => {
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
                footnote_start(ui, &footnote);
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
                    if ui
                        .add(egui::Checkbox::without_text(&mut checkbox))
                        .clicked()
                    {
                        self.checkbox_events.push(CheckboxClickEvent {
                            checked: checkbox,
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
                self.curr_heading_start_y = Some(ui.next_widget_position().y);

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
                if !self.inside_blockquote && !self.list.is_inside_a_list() && self.line.can_insert_start() {
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
                self.is_list_item = true;
                if !self.inside_blockquote {
                    self.list.start_item(ui, options, false);
                }
            }

            pulldown_cmark::Tag::FootnoteDefinition(note) => {
                self.line.try_insert_start(ui);

                self.line.should_start_newline = false;
                self.line.should_end_newline = false;
                footnote(ui, &note);
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

                if let (Some(rects), Some(start_y)) = (&mut self.populate_heading_rects, self.curr_heading_start_y) {
                    let end_y = ui.next_widget_position().y;
                    let width = ui.available_width();
                    let min_x = ui.min_rect().left();
                    rects.push(egui::Rect::from_min_max(
                        egui::pos2(min_x, start_y),
                        egui::pos2(min_x + width, end_y),
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
            pulldown_cmark::TagEnd::Item => {}
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
                    link.end(ui, cache);
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
                if let Some(html_fn) = options.html_fn {
                    html_fn(ui, &self.html_block);
                    self.html_block.clear();
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
