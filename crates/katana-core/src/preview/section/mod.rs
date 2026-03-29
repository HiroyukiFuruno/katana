mod fence;
mod html;
mod local_image;
mod math;

use crate::markdown::diagram::DiagramKind;
use fence::try_parse_diagram_fence;
pub use html::wrap_standalone_inline_html;
use local_image::extract_standalone_images;
use math::process_relaxed_math;

/// The type of section that makes up a document.
#[derive(Debug, Clone)]
pub enum PreviewSection {
    /// Normal Markdown text.
    Markdown(String),
    /// A diagram fence block.
    Diagram {
        kind: DiagramKind,
        source: String,
        lines: usize,
    },
    /// A standalone local image.
    LocalImage {
        path: String,
        alt: String,
        lines: usize,
    },
}

/// Splits the source text into a list of `PreviewSection`s.
///
/// Detects diagram fences (` ```mermaid` / ` ```plantuml` / ` ```drawio` ),
/// and groups the rest as Markdown sections.
pub fn split_into_sections(source: &str) -> Vec<PreviewSection> {
    let source_cow = process_relaxed_math(source);
    let initial_sections = parse_initial_sections(source_cow.as_ref());
    let temp = extract_standalone_images(initial_sections);
    merge_and_wrap_sections(temp)
}

fn parse_initial_sections(source: &str) -> Vec<PreviewSection> {
    let mut initial_sections = Vec::new();
    let mut markdown_acc = String::new();
    let mut remaining = source;

    while let Some(offset) = if remaining.starts_with("```") { Some(0) } else { remaining.find("\n```").map(|pos| pos + 1) } {
        markdown_acc.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
        if let Some((kind, fence_source, after)) = try_parse_diagram_fence(remaining) {
            if !markdown_acc.is_empty() {
                initial_sections.push(PreviewSection::Markdown(std::mem::take(&mut markdown_acc)));
            }
            let lines = fence_source.chars().filter(|c| *c == '\n').count();
            initial_sections.push(PreviewSection::Diagram { kind, source: fence_source, lines });
            remaining = after;
        } else {
            markdown_acc.push_str("```");
            remaining = &remaining["```".len()..];
        }
    }

    markdown_acc.push_str(remaining);
    if !markdown_acc.is_empty() {
        initial_sections.push(PreviewSection::Markdown(markdown_acc));
    }
    initial_sections
}

fn merge_and_wrap_sections(sections: Vec<PreviewSection>) -> Vec<PreviewSection> {
    let mut merged = Vec::new();
    let mut md_acc = String::new();
    for sec in sections {
        match sec {
            PreviewSection::Markdown(t) => {
                md_acc.push_str(&t);
                md_acc.push('\n');
            }
            other => {
                if !md_acc.is_empty() {
                    let processed = wrap_standalone_inline_html(&md_acc);
                    merged.push(PreviewSection::Markdown(processed));
                    md_acc.clear();
                }
                merged.push(other);
            }
        }
    }
    if !md_acc.is_empty() {
        let processed = wrap_standalone_inline_html(&md_acc);
        merged.push(PreviewSection::Markdown(processed));
    }
    merged
}
