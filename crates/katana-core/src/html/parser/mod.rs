/* WHY: Works with `comrak` AST's `HtmlBlock` / `HtmlInline` content,
extracting tag attributes via regex for shallow-nested HTML structures. */

pub mod inline;
pub mod regex;
#[cfg(test)]
mod tests;

use std::path::Path;

use crate::html::node::{HtmlNode, LinkTarget, TextAlign};

pub struct HtmlParser<'a> {
    base_dir: &'a Path,
}

impl<'a> HtmlParser<'a> {
    pub fn new(base_dir: &'a Path) -> Self {
        Self { base_dir }
    }

    pub fn parse(&self, html: &str) -> Vec<HtmlNode> {
        self.parse_fragment(html)
    }

    fn parse_fragment(&self, html: &str) -> Vec<HtmlNode> {
        let mut nodes = Vec::new();
        let mut remaining = html;

        while !remaining.is_empty() {
            if let Some(tag_start) = remaining.find('<') {
                self.parse_inline_before_tag(&mut nodes, remaining, tag_start);
                remaining = self.process_tag_or_skip(&mut nodes, &remaining[tag_start..]);
            } else {
                // WHY: No more tags — parse remaining as inline text
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    inline::parse_inline_text(trimmed, self.base_dir, &mut nodes);
                }
                break;
            }
        }
        nodes
    }

    fn parse_inline_before_tag(
        &self,
        nodes: &mut Vec<HtmlNode>,
        remaining: &str,
        tag_start: usize,
    ) {
        let trimmed = remaining[..tag_start].trim();
        if !trimmed.is_empty() {
            inline::parse_inline_text(trimmed, self.base_dir, nodes);
        }
    }

    fn process_tag_or_skip<'b>(&self, nodes: &mut Vec<HtmlNode>, remaining: &'b str) -> &'b str {
        if let Some((node, consumed)) = self.try_parse_tag(remaining) {
            nodes.push(node);
            &remaining[consumed..]
        } else if let Some(end) = remaining.find('>') {
            &remaining[end + 1..]
        } else {
            ""
        }
    }

    fn try_parse_tag(&self, s: &str) -> Option<(HtmlNode, usize)> {
        self.try_parse_br_or_img(s)
            .or_else(|| self.try_parse_a(s))
            .or_else(|| self.try_parse_paragraph(s))
            .or_else(|| self.try_parse_heading(s))
            .or_else(|| self.try_parse_em_or_strong(s))
    }

    fn try_parse_br_or_img(&self, s: &str) -> Option<(HtmlNode, usize)> {
        if let Some(m) = regex::regex_br().find(s) {
            if m.start() == 0 {
                return Some((HtmlNode::LineBreak, m.end()));
            }
        }
        if let Some(caps) = regex::regex_img().captures(s) {
            if caps.get(0)?.start() == 0 {
                let attrs = caps.get(1)?.as_str();
                let src = regex::extract_attr(attrs, "src").unwrap_or_default();
                let alt = regex::extract_attr(attrs, "alt").unwrap_or_default();
                return Some((HtmlNode::Image { src, alt }, caps.get(0)?.end()));
            }
        }
        None
    }
    fn try_parse_a(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps = regex::regex_a().captures(s)?;
        if caps.get(0)?.start() != 0 {
            return None;
        }
        let href = caps.get(1)?.as_str();
        let inner = caps.get(2)?.as_str();
        let children = self.parse_fragment(inner);
        let target = LinkTarget::resolve(href, self.base_dir);
        Some((HtmlNode::Link { target, children }, caps.get(0)?.end()))
    }

    fn try_parse_paragraph(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps = regex::regex_p().captures(s)?;
        if caps.get(0)?.start() != 0 {
            return None;
        }

        let attrs = caps.get(1)?.as_str();
        let inner = caps.get(2)?.as_str();
        let align = regex::extract_attr(attrs, "align").and_then(|a| match a.as_str() {
            "center" => Some(TextAlign::Center),
            "left" => Some(TextAlign::Left),
            "right" => Some(TextAlign::Right),
            _ => None,
        });
        let children = self.parse_fragment(inner);
        Some((HtmlNode::Paragraph { align, children }, caps.get(0)?.end()))
    }

    fn try_parse_heading(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps = regex::regex_heading().captures(s)?;
        if caps.get(0)?.start() != 0 {
            return None;
        }

        let level: u8 = caps.get(1)?.as_str().parse().ok()?;
        let attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let align = if attrs.contains(r#"align="center""#) {
            Some(TextAlign::Center)
        } else if attrs.contains(r#"align="left""#) {
            Some(TextAlign::Left)
        } else if attrs.contains(r#"align="right""#) {
            Some(TextAlign::Right)
        } else {
            None
        };
        const CAP_HEADING_INNER: usize = 3;
        let inner = caps.get(CAP_HEADING_INNER)?.as_str();
        let children = self.parse_fragment(inner);
        Some((
            HtmlNode::Heading {
                level,
                align,
                children,
            },
            caps.get(0)?.end(),
        ))
    }

    fn try_parse_em_or_strong(&self, s: &str) -> Option<(HtmlNode, usize)> {
        if let Some(caps) = regex::regex_em().captures(s) {
            if caps.get(0)?.start() == 0 {
                let inner = caps.get(1)?.as_str();
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Emphasis(children), caps.get(0)?.end()));
            }
        }
        if let Some(caps) = regex::regex_strong().captures(s) {
            if caps.get(0)?.start() == 0 {
                let inner = caps.get(1)?.as_str();
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Strong(children), caps.get(0)?.end()));
            }
        }
        None
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_try_parse_paragraph() {
        let parser = HtmlParser::new(std::path::Path::new("."));
        let (node, len) = parser
            .try_parse_paragraph(r#"<p align="center">center</p>"#)
            .unwrap();
        assert!(matches!(
            node,
            HtmlNode::Paragraph {
                align: Some(TextAlign::Center),
                ..
            }
        ));
        assert_eq!(len, 28);
    }

    #[test]
    fn test_try_parse_heading_alignments() {
        let parser = HtmlParser::new(std::path::Path::new("."));

        let (n1, _) = parser
            .try_parse_heading(r#"<h1 align="center">center</h1>"#)
            .unwrap();
        if let HtmlNode::Heading { level, align, .. } = n1 {
            assert_eq!(level, 1);
            assert_eq!(align, Some(TextAlign::Center));
        } else {
            panic!("Expected heading");
        }

        let (n2, _) = parser
            .try_parse_heading(r#"<h2 align="left">left</h2>"#)
            .unwrap();
        if let HtmlNode::Heading { align, .. } = n2 {
            assert_eq!(align, Some(TextAlign::Left));
        } else {
            panic!("Expected heading");
        }

        let (n3, _) = parser
            .try_parse_heading(r#"<h3 align="right">right</h3>"#)
            .unwrap();
        if let HtmlNode::Heading { align, .. } = n3 {
            assert_eq!(align, Some(TextAlign::Right));
        } else {
            panic!("Expected heading");
        }
    }

    #[test]
    fn test_try_parse_tag_fallback() {
        let parser = HtmlParser::new(std::path::Path::new("."));
        assert!(parser
            .try_parse_tag(r#"<p align="left">test</p>"#)
            .is_some());
    }
}
