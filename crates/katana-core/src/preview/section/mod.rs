mod fence;
mod html;
mod local_image;
mod math;

use crate::markdown::diagram::DiagramKind;
use fence::try_parse_diagram_fence;
pub use html::wrap_standalone_inline_html;
use local_image::extract_standalone_images;
use math::process_relaxed_math;

#[derive(Debug, Clone)]
pub enum PreviewSection {
    Markdown(String),
    Diagram {
        kind: DiagramKind,
        source: String,
        lines: usize,
    },
    LocalImage {
        path: String,
        alt: String,
        lines: usize,
    },
}

pub fn split_into_sections(source: &str) -> Vec<PreviewSection> {
    let source_cow = process_relaxed_math(source);
    let initial_sections = parse_initial_sections(source_cow.as_ref());
    let temp = extract_standalone_images(initial_sections);
    merge_and_wrap_sections(temp)
}

fn parse_initial_sections(source: &str) -> Vec<PreviewSection> {
    let mut secs = Vec::new();
    let mut acc = String::new();
    let mut rem = source;

    while let Some(o) = if rem.starts_with("```") {
        Some(0)
    } else {
        rem.find("\n```").map(|p| p + 1)
    } {
        acc.push_str(&rem[..o]);
        rem = &rem[o..];
        if let Some((kind, fence, after)) = try_parse_diagram_fence(rem) {
            if !acc.is_empty() {
                secs.push(PreviewSection::Markdown(std::mem::take(&mut acc)));
            }
            #[rustfmt::skip]
            let lines = fence.chars().filter(|c| *c == '\n').count();
            #[rustfmt::skip]
            secs.push(PreviewSection::Diagram { kind, source: fence, lines });
            rem = after;
        } else {
            acc.push_str("```");
            rem = &rem["```".len()..];
        }
    }
    acc.push_str(rem);
    if !acc.is_empty() {
        secs.push(PreviewSection::Markdown(acc));
    }
    secs
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
