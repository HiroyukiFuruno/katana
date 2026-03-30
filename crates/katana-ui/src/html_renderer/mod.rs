
use std::path::Path;

use eframe::egui;
use eframe::egui::text::LayoutJob;

use katana_core::html::{HtmlNode, LinkAction, TextAlign};


fn svg_badge_hosts() -> Vec<&'static str> {
    vec!["img.shields.io"]
}

const LINE_BREAK_SPACING: f32 = 4.0;

const HEADING_H2_SIZE: f32 = 20.0;

const HEADING_H3_SIZE: f32 = 16.0;
const PARAGRAPH_BLOCK_MARGIN_Y: f32 = 5.0;
const HEADING_BLOCK_MARGIN_Y: f32 = 6.0;

const HEADING_LEVEL_1: u8 = 1;
const HEADING_LEVEL_2: u8 = 2;
const HEADING_LEVEL_3: u8 = 3;


pub struct HtmlRenderer<'a> {
    ui: &'a mut egui::Ui,
    _base_dir: &'a Path,
    text_color: Option<egui::Color32>,
    max_image_width: f32,
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(ui: &'a mut egui::Ui, base_dir: &'a Path) -> Self {
        let max_w = ui.available_width();
        Self {
            ui,
            _base_dir: base_dir,
            text_color: None,
            max_image_width: max_w,
        }
    }

    pub fn text_color(mut self, color: egui::Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn max_image_width(mut self, width: f32) -> Self {
        self.max_image_width = width;
        self
    }

    pub fn render(mut self, nodes: &[HtmlNode]) -> Option<LinkAction> {
        self.render_nodes(nodes)
    }


    fn render_nodes(&mut self, nodes: &[HtmlNode]) -> Option<LinkAction> {
        let mut action: Option<LinkAction> = None;
        let mut inline_batch: Vec<&HtmlNode> = Vec::new();

        for (i, node) in nodes.iter().enumerate() {
            if node.is_block() {
                if let Some(a) = self.flush_inline_batch(&inline_batch) {
                    action = Some(a);
                }
                inline_batch.clear();

                if let Some(a) = self.render_block(node) {
                    action = Some(a);
                }

                if i < nodes.len() - 1 {
                    self.ui.add_space(match node {
                        HtmlNode::Heading { .. } => HEADING_BLOCK_MARGIN_Y,
                        HtmlNode::Paragraph { .. } => PARAGRAPH_BLOCK_MARGIN_Y,
                        _ => 0.0,
                    });
                }
            } else {
                inline_batch.push(node);
            }
        }

        if let Some(a) = self.flush_inline_batch(&inline_batch) {
            action = Some(a);
        }

        action
    }

    fn render_block(&mut self, node: &HtmlNode) -> Option<LinkAction> {
        match node {
            HtmlNode::Paragraph { align, children } => match align {
                Some(TextAlign::Center) => {
                    self.render_centered_children(children)
                }
                _ => self.render_nodes(children),
            },
            HtmlNode::Heading {
                level,
                align,
                children,
            } => {
                let text = collect_text(children);
                let mut rt = if *level == HEADING_LEVEL_1 {
                    egui::RichText::new(&text).heading()
                } else if *level == HEADING_LEVEL_2 {
                    egui::RichText::new(&text).strong().size(HEADING_H2_SIZE)
                } else if *level == HEADING_LEVEL_3 {
                    egui::RichText::new(&text).strong().size(HEADING_H3_SIZE)
                } else {
                    egui::RichText::new(&text).strong()
                };
                if let Some(c) = self.text_color {
                    rt = rt.color(c);
                }

                match align {
                    Some(TextAlign::Center) => {
                        let avail_w = self.ui.available_width();
                        self.ui.allocate_ui_with_layout(
                            egui::vec2(avail_w, 0.0),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.set_width(avail_w);
                                ui.label(rt);
                            },
                        );
                    }
                    _ => {
                        self.ui.label(rt);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn batch_is_textual(batch: &[&HtmlNode]) -> bool {
        batch.iter().all(|node| {
            matches!(
                node,
                HtmlNode::Text(_)
                    | HtmlNode::Emphasis(_)
                    | HtmlNode::Strong(_)
                    | HtmlNode::LineBreak
            )
        })
    }

    fn render_text_batch(&mut self, batch: &[&HtmlNode], centered: bool) {
        let mut job = LayoutJob::default();
        job.wrap.max_width = self.ui.available_width();
        job.halign = if centered {
            egui::Align::Center
        } else {
            egui::Align::LEFT
        };

        for node in batch {
            self.append_text_node(&mut job, node, false, false);
        }

        let label = egui::Label::new(job).wrap();
        if centered {
            self.ui
                .add_sized(egui::vec2(self.ui.available_width(), 0.0), label);
        } else {
            self.ui.add(label);
        }
    }

    fn render_centered_children(&mut self, children: &[HtmlNode]) -> Option<LinkAction> {
        let mut action: Option<LinkAction> = None;
        let mut inline_batch: Vec<&HtmlNode> = Vec::new();
        let mut batch_index: usize = 0;

        for node in children {
            if node.is_block() {
                if let Some(a) = self.flush_centered_inline_batch(&inline_batch, batch_index) {
                    action = Some(a);
                }
                batch_index += 1;
                inline_batch.clear();
                if let Some(a) = self.render_block(node) {
                    action = Some(a);
                }
            } else {
                inline_batch.push(node);
            }
        }

        if let Some(a) = self.flush_centered_inline_batch(&inline_batch, batch_index) {
            action = Some(a);
        }

        action
    }

    fn flush_inline_batch(&mut self, batch: &[&HtmlNode]) -> Option<LinkAction> {
        if batch.is_empty() {
            return None;
        }

        if Self::batch_is_textual(batch) {
            self.render_text_batch(batch, false);
            return None;
        }

        if batch.len() == 1 {
            return self.render_inline(batch[0]);
        }

        let mut action = None;
        self.ui.horizontal_wrapped(|ui| {
            for node in batch {
                let mut inner = HtmlRenderer::new_inner(ui, self.text_color, self.max_image_width);
                if let Some(a) = inner.render_inline(node) {
                    action = Some(a);
                }
            }
        });
        action
    }

    fn flush_centered_inline_batch(
        &mut self,
        batch: &[&HtmlNode],
        batch_index: usize,
    ) -> Option<LinkAction> {
        if batch.is_empty() {
            return None;
        }

        if Self::batch_is_textual(batch) {
            self.render_text_batch(batch, true);
            return None;
        }

        if batch.len() == 1 {
            let mut action = None;
            self.ui.vertical_centered(|ui| {
                let mut inner = HtmlRenderer::new_inner(ui, self.text_color, self.max_image_width);
                action = inner.render_inline(batch[0]);
            });
            return action;
        }

        let mut action = None;

        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write_usize(batch_index);
        hasher.write_usize(batch.len());
        let text_content = batch
            .iter()
            .map(|n| collect_text(std::slice::from_ref(*n)))
            .collect::<String>();
        hasher.write(text_content.as_bytes());
        let hash = hasher.finish();

        let id = self.ui.id().with("centered_batch").with(hash);

        let mut memorized = true;
        let bounds = self.ui.available_rect_before_wrap();
        let content_size: egui::Vec2 =
            self.ui.ctx().data(|r| r.get_temp(id)).unwrap_or_else(|| {
                memorized = false;
                bounds.size()
            });

        let centered_rect = egui::Align2::CENTER_TOP.align_size_within_rect(content_size, bounds);

        let layout = egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(false);
        let child_max_rect = egui::Rect::from_min_size(
            centered_rect.min,
            egui::vec2(bounds.width(), bounds.height()),
        );
        let builder = egui::UiBuilder::new()
            .max_rect(child_max_rect)
            .layout(layout);
        let builder = if memorized {
            builder
        } else {
            self.ui.ctx().request_discard("center_inline_batch");
            builder.sizing_pass().invisible()
        };

        let mut child_ui = self.ui.new_child(builder);
        const HORIZONTAL_ITEM_SPACING: f32 = 4.0;
        child_ui.spacing_mut().item_spacing.x = HORIZONTAL_ITEM_SPACING;

        for node in batch {
            let mut inner =
                HtmlRenderer::new_inner(&mut child_ui, self.text_color, self.max_image_width);
            if let Some(a) = inner.render_inline(node) {
                action = Some(a);
            }
        }

        let new_size = child_ui.min_size();
        if new_size != content_size || !memorized {
            self.ui.ctx().data_mut(|w| w.insert_temp(id, new_size));
        }

        let row_height = new_size.y;
        self.ui
            .allocate_space(egui::vec2(bounds.width(), row_height));

        action
    }

    fn render_inline(&mut self, node: &HtmlNode) -> Option<LinkAction> {
        match node {
            HtmlNode::Text(text) => {
                let mut rt = egui::RichText::new(text.as_str());
                if let Some(c) = self.text_color {
                    rt = rt.color(c);
                }
                self.ui.label(rt);
                None
            }
            HtmlNode::Image { src, alt: _ } => {
                let url = ensure_svg_extension(src);
                self.ui.add(
                    egui::Image::new(url)
                        .fit_to_original_size(1.0)
                        .max_width(self.max_image_width),
                );
                None
            }
            HtmlNode::Link { target, children } => {
                let text = collect_text(children);
                let action = target.default_action();
                let color = self.ui.visuals().hyperlink_color;
                let tooltip = target.tooltip_text();

                let has_images = children.iter().any(|c| matches!(c, HtmlNode::Image { .. }));
                if has_images {
                    let mut clicked = false;
                    for child in children {
                        if let HtmlNode::Image { src, alt: _ } = child {
                            let url = ensure_svg_extension(src);
                            let response = self.ui.add(
                                egui::Image::new(url)
                                    .fit_to_original_size(1.0)
                                    .max_width(self.max_image_width)
                                    .sense(egui::Sense::click()),
                            );
                            let response = response
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text(&tooltip);
                            if response.clicked() {
                                clicked = true;
                            }
                        }
                    }
                    if clicked {
                        return Some(action);
                    }
                } else {
                    let rt = egui::RichText::new(&text).underline().color(color);
                    let response = self
                        .ui
                        .add(egui::Label::new(rt).sense(egui::Sense::click()))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(&tooltip);
                    if response.clicked() {
                        return Some(action);
                    }
                }
                None
            }
            HtmlNode::LineBreak => {
                self.ui.add_space(LINE_BREAK_SPACING);
                None
            }
            HtmlNode::Emphasis(children) => {
                let text = collect_text(children);
                let mut rt = egui::RichText::new(&text).italics();
                if let Some(c) = self.text_color {
                    rt = rt.color(c);
                }
                self.ui.label(rt);
                None
            }
            HtmlNode::Strong(children) => {
                let text = collect_text(children);
                let mut rt = egui::RichText::new(&text).strong();
                if let Some(c) = self.text_color {
                    rt = rt.color(c);
                }
                self.ui.label(rt);
                None
            }
            _ => None,
        }
    }

    fn append_text_node(&self, job: &mut LayoutJob, node: &HtmlNode, strong: bool, italics: bool) {
        match node {
            HtmlNode::Text(text) => {
                let mut rich = egui::RichText::new(text.as_str());
                if strong {
                    rich = rich.strong();
                }
                if italics {
                    rich = rich.italics();
                }
                if let Some(color) = self.text_color {
                    rich = rich.color(color);
                }
                rich.append_to(
                    job,
                    self.ui.style().as_ref(),
                    egui::FontSelection::Default,
                    egui::Align::Center,
                );
            }
            HtmlNode::Emphasis(children) => {
                for child in children {
                    self.append_text_node(job, child, strong, true);
                }
            }
            HtmlNode::Strong(children) => {
                for child in children {
                    self.append_text_node(job, child, true, italics);
                }
            }
            HtmlNode::LineBreak => {
                let mut rich = egui::RichText::new("\n");
                if let Some(color) = self.text_color {
                    rich = rich.color(color);
                }
                rich.append_to(
                    job,
                    self.ui.style().as_ref(),
                    egui::FontSelection::Default,
                    egui::Align::Center,
                );
            }
            _ => {}
        }
    }

    fn new_inner(ui: &'a mut egui::Ui, text_color: Option<egui::Color32>, max_w: f32) -> Self {
        Self {
            ui,
            _base_dir: Path::new(""),
            text_color,
            max_image_width: max_w,
        }
    }
}


fn collect_text(nodes: &[HtmlNode]) -> String {
    let mut s = String::new();
    for node in nodes {
        match node {
            HtmlNode::Text(t) => s.push_str(t),
            HtmlNode::Link { children, .. }
            | HtmlNode::Heading { children, .. }
            | HtmlNode::Paragraph { children, .. }
            | HtmlNode::Emphasis(children)
            | HtmlNode::Strong(children) => s.push_str(&collect_text(children)),
            HtmlNode::Image { alt, .. } => s.push_str(alt),
            HtmlNode::LineBreak => s.push('\n'),
        }
    }
    s
}

fn ensure_svg_extension(url: &str) -> String {
    let (path, suffix) = split_url_suffix(url);
    if path.ends_with(".svg") {
        return url.to_string();
    }
    for host in svg_badge_hosts() {
        if url.contains(host) {
            return format!("{path}.svg{suffix}");
        }
    }
    url.to_string()
}

fn split_url_suffix(url: &str) -> (&str, &str) {
    let suffix_start = url
        .find('?')
        .into_iter()
        .chain(url.find('#'))
        .min()
        .unwrap_or(url.len());
    url.split_at(suffix_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_svg_extension_inserts_suffix_before_query_string() {
        let url =
            "https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors";

        let normalized = ensure_svg_extension(url);

        assert_eq!(
            normalized,
            "https://img.shields.io/badge/Sponsor-❤️-ea4aaa.svg?style=for-the-badge&logo=github-sponsors"
        );
    }

    #[test]
    fn ensure_svg_extension_preserves_existing_svg_suffix_before_query_string() {
        let url = "https://img.shields.io/badge/License-MIT-blue.svg?style=flat";

        assert_eq!(ensure_svg_extension(url), url);
    }

    #[test]
    fn heading_with_align_center_is_centered() {
        use eframe::egui;
        use egui_kittest::{
            kittest::{NodeT, Queryable},
            Harness,
        };

        let html = "<h1 align=\"center\">Centered Heading</h1>";
        let parser = katana_core::html::HtmlParser::new(std::path::Path::new("."));
        let nodes = parser.parse(html);

        let mut harness = Harness::builder()
            .with_size(egui::vec2(600.0, 400.0))
            .build_ui(move |ui| {
                ui.set_width(600.0);
                let renderer = HtmlRenderer::new(ui, std::path::Path::new("."));
                renderer.render(&nodes);
            });

        harness.step();

        let label = harness.get_by_label("Centered Heading");
        let bounds = label
            .accesskit_node()
            .raw_bounds()
            .expect("heading must have bounds");

        assert!(
            bounds.x0 > 200.0,
            "Heading with align='center' should be centered, but its x0 is {:.1}",
            bounds.x0
        );
    }
}