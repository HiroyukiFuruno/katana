use crate::html::node::{HtmlNode, LinkTarget};
use std::path::Path;

/// Parses plain text that may contain markdown inline syntax.
///
/// Handles:
/// - `![alt](src)` → Image
/// - `[text](url)` → Link
/// - Plain text → Text
pub fn parse_inline_text(text: &str, base_dir: &Path, nodes: &mut Vec<HtmlNode>) {
    let mut remaining = text;

    while !remaining.is_empty() {
        if let Some(pos) = find_next_syntax_pos(remaining) {
            // WHY: Text before the syntax
            if pos > 0 {
                nodes.push(HtmlNode::Text(remaining[..pos].to_string()));
            }

            remaining = process_syntax(&remaining[pos..], base_dir, nodes);
        } else {
            // WHY: No more markdown syntax
            nodes.push(HtmlNode::Text(remaining.to_string()));
            break;
        }
    }
}

fn find_next_syntax_pos(text: &str) -> Option<usize> {
    let img_pos = text.find("![");
    let link_pos = text.find('[').filter(|&pos| {
        // WHY: Exclude the '[' that's part of '!['
        img_pos != Some(pos.saturating_sub(1))
    });

    match (img_pos, link_pos) {
        (Some(i), Some(l)) => Some(i.min(l)),
        (Some(i), None) => Some(i),
        (None, Some(l)) => Some(l),
        (None, None) => None,
    }
}

fn process_syntax<'a>(text: &'a str, base_dir: &Path, nodes: &mut Vec<HtmlNode>) -> &'a str {
    // WHY: Try markdown image: ![alt](src)
    if text.starts_with("![") {
        if let Some((node, consumed)) = try_parse_md_image(text) {
            nodes.push(node);
            return &text[consumed..];
        }
    }

    // WHY: Try markdown link: [text](url)
    if text.starts_with('[') {
        if let Some((node, consumed)) = try_parse_md_link(text, base_dir) {
            nodes.push(node);
            return &text[consumed..];
        }
    }

    // WHY: Not a valid syntax — emit the character and continue
    nodes.push(HtmlNode::Text(text[..1].to_string()));
    &text[1..]
}

/// Tries to parse `![alt](src)` at the start of `s`.
pub fn try_parse_md_image(s: &str) -> Option<(HtmlNode, usize)> {
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
pub fn try_parse_md_link(s: &str, base_dir: &Path) -> Option<(HtmlNode, usize)> {
    let rest = s.strip_prefix('[')?;
    let close_bracket = rest.find("](")?;
    let link_text = &rest[..close_bracket];
    let after = &rest[close_bracket + 2..];
    let close_paren = after.find(')')?;
    let url = &after[..close_paren];
    let total = 1 + close_bracket + 2 + close_paren + 1;
    let target = LinkTarget::resolve(url, base_dir);
    Some((
        HtmlNode::Link {
            target,
            children: vec![HtmlNode::Text(link_text.to_string())],
        },
        total,
    ))
}
