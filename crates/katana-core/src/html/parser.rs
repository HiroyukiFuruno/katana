//! HTML parser that converts HTML fragments into `HtmlNode` trees.
//!
//! Works with `comrak` AST's `HtmlBlock` / `HtmlInline` content,
//! extracting tag attributes via regex for shallow-nested HTML structures.

use std::path::Path;

use regex::Regex;

use super::node::{HtmlNode, LinkTarget, TextAlign};

/// Parser that converts HTML strings into structured `HtmlNode` trees.
///
/// Holds the base directory for resolving relative link paths.
pub struct HtmlParser<'a> {
    base_dir: &'a Path,
}

impl<'a> HtmlParser<'a> {
    /// Creates a new parser with the given base directory for link resolution.
    pub fn new(base_dir: &'a Path) -> Self {
        Self { base_dir }
    }

    /// Parses an HTML fragment into a list of `HtmlNode`s.
    ///
    /// The input is typically the content of a `<p align="center">...</p>` block
    /// or similar HTML extracted from a Markdown document.
    pub fn parse(&self, html: &str) -> Vec<HtmlNode> {
        self.parse_fragment(html)
    }

    /// Recursively parses an HTML fragment into nodes.
    fn parse_fragment(&self, html: &str) -> Vec<HtmlNode> {
        let mut nodes = Vec::new();
        let mut remaining = html;

        while !remaining.is_empty() {
            if let Some(tag_start) = remaining.find('<') {
                // Text before the tag
                let before = &remaining[..tag_start];
                let trimmed = before.trim();
                if !trimmed.is_empty() {
                    self.parse_inline_text(trimmed, &mut nodes);
                }

                remaining = &remaining[tag_start..];

                // Try to parse a known tag
                if let Some((node, consumed)) = self.try_parse_tag(remaining) {
                    nodes.push(node);
                    remaining = &remaining[consumed..];
                } else {
                    // Skip unknown/malformed tag — find the end '>'
                    if let Some(end) = remaining.find('>') {
                        remaining = &remaining[end + 1..];
                    } else {
                        break;
                    }
                }
            } else {
                // No more tags — parse remaining as inline text
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    self.parse_inline_text(trimmed, &mut nodes);
                }
                break;
            }
        }

        nodes
    }

    /// Tries to parse a known HTML tag at the beginning of `s`.
    /// Returns `(HtmlNode, bytes_consumed)`.
    fn try_parse_tag(&self, s: &str) -> Option<(HtmlNode, usize)> {
        // <br> / <br/>
        if let Some(m) = regex_br().find(s) {
            if m.start() == 0 {
                return Some((HtmlNode::LineBreak, m.end()));
            }
        }

        // <img ...>
        if let Some(caps) = regex_img().captures(s) {
            if caps.get(0)?.start() == 0 {
                let attrs = caps.get(1)?.as_str();
                let src = extract_attr(attrs, "src").unwrap_or_default();
                let alt = extract_attr(attrs, "alt").unwrap_or_default();
                return Some((HtmlNode::Image { src, alt }, caps.get(0)?.end()));
            }
        }

        // <a href="...">...</a>
        if let Some(caps) = regex_a().captures(s) {
            if caps.get(0)?.start() == 0 {
                let href = caps.get(1)?.as_str();
                let inner = caps.get(2)?.as_str();
                let children = self.parse_fragment(inner);
                let target = LinkTarget::resolve(href, self.base_dir);
                return Some((HtmlNode::Link { target, children }, caps.get(0)?.end()));
            }
        }

        // <p align="...">...</p>
        if let Some(caps) = regex_p().captures(s) {
            if caps.get(0)?.start() == 0 {
                let attrs = caps.get(1)?.as_str();
                let inner = caps.get(2)?.as_str();
                let align = extract_attr(attrs, "align").and_then(|a| match a.as_str() {
                    "center" => Some(TextAlign::Center),
                    "left" => Some(TextAlign::Left),
                    "right" => Some(TextAlign::Right),
                    _ => None,
                });
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Paragraph { align, children }, caps.get(0)?.end()));
            }
        }

        // <h1>...<h6>
        if let Some(caps) = regex_heading().captures(s) {
            if caps.get(0)?.start() == 0 {
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
                return Some((
                    HtmlNode::Heading {
                        level,
                        align,
                        children,
                    },
                    caps.get(0)?.end(),
                ));
            }
        }

        // <em>...</em>
        if let Some(caps) = regex_em().captures(s) {
            if caps.get(0)?.start() == 0 {
                let inner = caps.get(1)?.as_str();
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Emphasis(children), caps.get(0)?.end()));
            }
        }

        // <strong>...</strong>
        if let Some(caps) = regex_strong().captures(s) {
            if caps.get(0)?.start() == 0 {
                let inner = caps.get(1)?.as_str();
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Strong(children), caps.get(0)?.end()));
            }
        }

        None
    }

    /// Parses plain text that may contain markdown inline syntax.
    ///
    /// Handles:
    /// - `![alt](src)` → Image
    /// - `[text](url)` → Link
    /// - Plain text → Text
    fn parse_inline_text(&self, text: &str, nodes: &mut Vec<HtmlNode>) {
        let mut remaining = text;

        while !remaining.is_empty() {
            // Look for markdown image or link
            let img_pos = remaining.find("![");
            let link_pos = remaining.find('[').filter(|&pos| {
                // Exclude the '[' that's part of '!['
                img_pos != Some(pos.saturating_sub(1))
            });

            let next_syntax = match (img_pos, link_pos) {
                (Some(i), Some(l)) => Some(i.min(l)),
                (Some(i), None) => Some(i),
                (None, Some(l)) => Some(l),
                (None, None) => None,
            };

            if let Some(pos) = next_syntax {
                // Text before the syntax
                if pos > 0 {
                    nodes.push(HtmlNode::Text(remaining[..pos].to_string()));
                }

                remaining = &remaining[pos..];

                // Try markdown image: ![alt](src)
                if remaining.starts_with("![") {
                    if let Some((node, consumed)) = self.try_parse_md_image(remaining) {
                        nodes.push(node);
                        remaining = &remaining[consumed..];
                        continue;
                    }
                }

                // Try markdown link: [text](url)
                if remaining.starts_with('[') {
                    if let Some((node, consumed)) = self.try_parse_md_link(remaining) {
                        nodes.push(node);
                        remaining = &remaining[consumed..];
                        continue;
                    }
                }

                // Not a valid syntax — emit the character and continue
                nodes.push(HtmlNode::Text(remaining[..1].to_string()));
                remaining = &remaining[1..];
            } else {
                // No more markdown syntax
                nodes.push(HtmlNode::Text(remaining.to_string()));
                break;
            }
        }
    }

    /// Tries to parse `![alt](src)` at the start of `s`.
    fn try_parse_md_image(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let rest = s.strip_prefix("![")?;
        let close_bracket = rest.find("](")?;
        let alt = &rest[..close_bracket];
        let after = &rest[close_bracket + 2..];
        let close_paren = after.find(')')?;
        let src = &after[..close_paren];
        if src.is_empty() {
            return None;
        }
        let total = 2 + close_bracket + 2 + close_paren + 1;
        Some((
            HtmlNode::Image {
                src: src.to_string(),
                alt: alt.to_string(),
            },
            total,
        ))
    }

    /// Tries to parse `[text](url)` at the start of `s`.
    fn try_parse_md_link(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let rest = s.strip_prefix('[')?;
        let close_bracket = rest.find("](")?;
        let link_text = &rest[..close_bracket];
        let after = &rest[close_bracket + 2..];
        let close_paren = after.find(')')?;
        let url = &after[..close_paren];
        let total = 1 + close_bracket + 2 + close_paren + 1;
        let target = LinkTarget::resolve(url, self.base_dir);
        Some((
            HtmlNode::Link {
                target,
                children: vec![HtmlNode::Text(link_text.to_string())],
            },
            total,
        ))
    }
}

// ───────────────────── Regex helpers (compiled once) ─────────────────────

fn regex_br() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?is)<br\s*/?>"#).unwrap());
    &RE
}

fn regex_img() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?is)<img\s+([^>]+)>"#).unwrap());
    &RE
}

fn regex_a() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(?is)<a\s+[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#).unwrap());
    &RE
}

fn regex_p() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(?is)<p\s+([^>]*)>(.*?)</p>"#).unwrap());
    &RE
}

fn regex_heading() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(?is)<h([1-6])([^>]*)>(.*?)</h[1-6]>"#).unwrap());
    &RE
}

fn regex_em() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?is)<em>(.*?)</em>"#).unwrap());
    &RE
}

fn regex_strong() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(?is)<strong>(.*?)</strong>"#).unwrap());
    &RE
}

/// Extracts an attribute value from an HTML tag's attribute string.
fn extract_attr(attrs: &str, attr_name: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"(?is){}\s*=\s*"([^"]+)""#, attr_name)).ok()?;
    re.captures(attrs)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

// ───────────────────── Tests ─────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn parser() -> HtmlParser<'static> {
        HtmlParser::new(Path::new("/project"))
    }

    // ── Basic tag parsing ──

    #[test]
    fn parse_img_tag() {
        let nodes = parser().parse(r#"<img src="icon.png" alt="icon">"#);
        assert_eq!(
            nodes,
            vec![HtmlNode::Image {
                src: "icon.png".into(),
                alt: "icon".into(),
            }]
        );
    }

    #[test]
    fn parse_br_tag() {
        let nodes = parser().parse("<br>");
        assert_eq!(nodes, vec![HtmlNode::LineBreak]);
    }

    #[test]
    fn parse_br_self_closing() {
        let nodes = parser().parse("<br/>");
        assert_eq!(nodes, vec![HtmlNode::LineBreak]);
    }

    #[test]
    fn parse_link_with_image_badge() {
        let html = r#"<a href="LICENSE"><img src="badge.svg" alt="License"></a>"#;
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Link {
                target: LinkTarget::InternalFile(PathBuf::from("/project/LICENSE")),
                children: vec![HtmlNode::Image {
                    src: "badge.svg".into(),
                    alt: "License".into(),
                }],
            }]
        );
    }

    #[test]
    fn parse_centered_paragraph() {
        let html = r#"<p align="center">hello</p>"#;
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Paragraph {
                align: Some(TextAlign::Center),
                children: vec![HtmlNode::Text("hello".into())],
            }]
        );
    }

    #[test]
    fn parse_heading() {
        let html = "<h2>Section Title</h2>";
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Heading {
                level: 2,
                align: None,
                children: vec![HtmlNode::Text("Section Title".into())],
            }]
        );
    }

    #[test]
    fn parse_centered_heading() {
        let html = r#"<h1 align="center">Title</h1>"#;
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Heading {
                level: 1,
                align: Some(TextAlign::Center),
                children: vec![HtmlNode::Text("Title".into())],
            }]
        );
    }

    #[test]
    fn parse_emphasis() {
        let html = "<em>italic</em>";
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Emphasis(vec![HtmlNode::Text("italic".into()),])]
        );
    }

    #[test]
    fn parse_strong() {
        let html = "<strong>bold</strong>";
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Strong(vec![HtmlNode::Text("bold".into()),])]
        );
    }

    #[test]
    fn parse_left_heading() {
        let html = r#"<h3 align="left">Left</h3>"#;
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Heading {
                level: 3,
                align: Some(TextAlign::Left),
                children: vec![HtmlNode::Text("Left".into())],
            }]
        );
    }

    #[test]
    fn parse_right_heading() {
        let html = r#"<h6 align="right">Right</h6>"#;
        let nodes = parser().parse(html);
        assert_eq!(
            nodes,
            vec![HtmlNode::Heading {
                level: 6,
                align: Some(TextAlign::Right),
                children: vec![HtmlNode::Text("Right".into())],
            }]
        );
    }

    // ── Markdown inline syntax ──

    #[test]
    fn parse_md_image() {
        let nodes = parser().parse("![alt text](image.png)");
        assert_eq!(
            nodes,
            vec![HtmlNode::Image {
                src: "image.png".into(),
                alt: "alt text".into(),
            }]
        );
    }

    #[test]
    fn parse_md_link() {
        let nodes = parser().parse("[日本語](README.ja.md)");
        assert_eq!(
            nodes,
            vec![HtmlNode::Link {
                target: LinkTarget::InternalFile(PathBuf::from("/project/README.ja.md")),
                children: vec![HtmlNode::Text("日本語".into())],
            }]
        );
    }

    #[test]
    fn parse_md_external_link() {
        let nodes = parser().parse("[GitHub](https://github.com)");
        assert_eq!(
            nodes,
            vec![HtmlNode::Link {
                target: LinkTarget::External("https://github.com".into()),
                children: vec![HtmlNode::Text("GitHub".into())],
            }]
        );
    }

    #[test]
    fn parse_text_with_md_link() {
        let nodes = parser().parse("English | [日本語](README.ja.md)");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], HtmlNode::Text("English | ".into()));
        assert_eq!(
            nodes[1],
            HtmlNode::Link {
                target: LinkTarget::InternalFile(PathBuf::from("/project/README.ja.md")),
                children: vec![HtmlNode::Text("日本語".into())],
            }
        );
    }

    // ── Complex / nested ──

    #[test]
    fn parse_shields_badge_pattern() {
        let html = r#"<a href="https://github.com/org/repo/actions"><img src="https://img.shields.io/github/actions/workflow/status/org/repo/ci.yml?label=CI" alt="CI"></a>"#;
        let nodes = parser().parse(html);
        assert_eq!(nodes.len(), 1);
        let HtmlNode::Link { target, children } = &nodes[0] else {
            panic!("Expected Link node");
        };
        assert_eq!(
            *target,
            LinkTarget::External("https://github.com/org/repo/actions".into())
        );
        assert_eq!(children.len(), 1);
        let HtmlNode::Image { src, alt } = &children[0] else {
            panic!("Expected Image inside Link");
        };
        assert!(src.contains("img.shields.io"));
        assert_eq!(alt, "CI");
    }

    #[test]
    fn parse_centered_paragraph_with_badges() {
        let html = r#"<p align="center">
  <a href="LICENSE"><img src="badge1.svg" alt="License"></a>
  <a href="actions"><img src="badge2.svg" alt="CI"></a>
</p>"#;
        let nodes = parser().parse(html);
        assert_eq!(nodes.len(), 1);
        let HtmlNode::Paragraph { align, children } = &nodes[0] else {
            panic!("Expected Paragraph");
        };
        assert_eq!(*align, Some(TextAlign::Center));
        let links: Vec<_> = children
            .iter()
            .filter(|n| matches!(n, HtmlNode::Link { .. }))
            .collect();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn parse_readme_badge_block_structure() {
        let html = r#"<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml"><img src="https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/HiroyukiFuruno/katana/releases/latest"><img src="https://img.shields.io/github/v/release/HiroyukiFuruno/katana" alt="Latest Release"></a>
  <img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="Platform: macOS">
</p>"#;
        let nodes = parser().parse(html);
        assert_eq!(nodes.len(), 1, "Should produce 1 Paragraph node");

        let HtmlNode::Paragraph { align, children } = &nodes[0] else {
            panic!("Expected Paragraph");
        };
        assert_eq!(*align, Some(TextAlign::Center));
        assert_eq!(
            children.len(),
            4,
            "Should have 4 children (3 links + 1 image)"
        );

        // All children must be inline
        for (i, c) in children.iter().enumerate() {
            assert!(c.is_inline(), "Child {i} should be inline, got {:?}", c);
        }
    }

    // ── Edge case coverage ──

    #[test]
    fn parse_unknown_tag_is_skipped() {
        let nodes = parser().parse("<div>content</div>");
        // Unknown tag is skipped; content text is parsed
        assert!(!nodes.is_empty());
    }

    #[test]
    fn parse_malformed_tag_without_closing_bracket() {
        // Malformed tag without '>' should not loop forever
        let nodes = parser().parse("<unclosed");
        assert!(nodes.is_empty());
    }

    #[test]
    fn parse_text_before_known_tag() {
        let nodes = parser().parse("hello <br>");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], HtmlNode::Text("hello".into()));
        assert_eq!(nodes[1], HtmlNode::LineBreak);
    }

    #[test]
    fn parse_paragraph_left_align() {
        let html = r#"<p align="left">text</p>"#;
        let nodes = parser().parse(html);
        let HtmlNode::Paragraph { align, .. } = &nodes[0] else {
            panic!("Expected Paragraph");
        };
        assert_eq!(*align, Some(TextAlign::Left));
    }

    #[test]
    fn parse_paragraph_right_align() {
        let html = r#"<p align="right">text</p>"#;
        let nodes = parser().parse(html);
        let HtmlNode::Paragraph { align, .. } = &nodes[0] else {
            panic!("Expected Paragraph");
        };
        assert_eq!(*align, Some(TextAlign::Right));
    }

    #[test]
    fn parse_paragraph_unknown_align_is_none() {
        let html = r#"<p align="justify">text</p>"#;
        let nodes = parser().parse(html);
        let HtmlNode::Paragraph { align, .. } = &nodes[0] else {
            panic!("Expected Paragraph");
        };
        assert_eq!(*align, None);
    }

    #[test]
    fn parse_md_image_with_empty_src_returns_text() {
        // ![alt]() has empty src → should NOT create Image node
        let nodes = parser().parse("![alt]()");
        // Falls through to text output
        assert!(nodes.iter().all(|n| !matches!(n, HtmlNode::Image { .. })));
    }

    #[test]
    fn parse_mixed_md_image_and_link() {
        // Link before image → both `[` and `![` are found → covers min(img_pos, link_pos)
        let nodes = parser().parse("[click](https://example.com) ![icon](a.png)");
        let images: Vec<_> = nodes
            .iter()
            .filter(|n| matches!(n, HtmlNode::Image { .. }))
            .collect();
        let links: Vec<_> = nodes
            .iter()
            .filter(|n| matches!(n, HtmlNode::Link { .. }))
            .collect();
        assert_eq!(images.len(), 1, "Should have one image");
        assert_eq!(links.len(), 1, "Should have one link");
    }

    #[test]
    fn parse_bracket_not_followed_by_paren_emits_text() {
        // Lone '[' that isn't a valid link → should emit as text character
        let nodes = parser().parse("[not a link");
        assert!(!nodes.is_empty());
        // Should contain text, not a link
        assert!(nodes.iter().all(|n| !matches!(n, HtmlNode::Link { .. })));
    }
}
