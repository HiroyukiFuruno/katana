//! Document section model for preview.
//!
//! Splits Markdown source into "normal text" and "diagram blocks",
//! allowing the UI layer to render each section independently.

use crate::markdown::diagram::DiagramKind;

/// The type of section that makes up a document.
#[derive(Debug, Clone)]
pub enum PreviewSection {
    /// Normal Markdown text.
    Markdown(String),
    /// A diagram fence block.
    Diagram { kind: DiagramKind, source: String },
}

/// Splits the source text into a list of `PreviewSection`s.
///
/// Detects diagram fences (` ```mermaid` / ` ```plantuml` / ` ```drawio` ),
/// and groups the rest as Markdown sections.
pub fn split_into_sections(source: &str) -> Vec<PreviewSection> {
    let mut sections = Vec::new();
    let mut markdown_acc = String::new();
    let mut remaining = source;

    while let Some(fence_pos) = remaining.find("\n```") {
        markdown_acc.push_str(&remaining[..fence_pos + 1]);
        remaining = &remaining[fence_pos + 1..];
        match try_parse_diagram_fence(remaining) {
            Some((kind, fence_source, after)) => {
                flush_markdown(&mut sections, &mut markdown_acc);
                sections.push(PreviewSection::Diagram {
                    kind,
                    source: fence_source,
                });
                remaining = after;
            }
            None => {
                // If not a diagram, treat as plain Markdown.
                markdown_acc.push_str("```");
                remaining = &remaining["```".len()..];
            }
        }
    }

    markdown_acc.push_str(remaining);
    flush_markdown(&mut sections, &mut markdown_acc);
    sections
}

/// If the accumulated Markdown text is not empty, add it to the sections.
fn flush_markdown(sections: &mut Vec<PreviewSection>, acc: &mut String) {
    if !acc.is_empty() {
        sections.push(PreviewSection::Markdown(std::mem::take(acc)));
    }
}

/// If the start is a diagram fence, returns `(kind, source, after)`.
fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
    let body = s.strip_prefix("```")?;
    let info_end = body.find('\n')?;
    let info = body[..info_end].trim();
    let kind = DiagramKind::from_info(info)?;
    let after_info = &body[info_end + 1..];
    let close = after_info.find("\n```")?;
    let source = after_info[..close].to_string();
    let rest_start = close + "\n```".len();
    let after = after_info[rest_start..]
        .strip_prefix('\n')
        .unwrap_or(&after_info[rest_start..]);
    Some((kind, source, after))
}
