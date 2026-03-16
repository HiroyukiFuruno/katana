use std::path::Path;

use crate::markdown::diagram::DiagramKind;

/// Resolves relative image paths in Markdown source to absolute `file://` URIs.
///
/// Given the path to the Markdown file being previewed, rewrites image references
/// like `![alt](../assets/image.png)` to `![alt](file:///absolute/path/assets/image.png)`.
///
/// Already-absolute paths, URLs (`http://`, `https://`), and `file://` URIs are left unchanged.
pub fn resolve_image_paths(source: &str, md_file_path: &Path) -> String {
    let base_dir = md_file_path.parent().unwrap_or(Path::new("."));
    let mut result = String::with_capacity(source.len());
    let mut remaining = source;

    while let Some(img_start) = remaining.find("![") {
        result.push_str(&remaining[..img_start]);
        remaining = &remaining[img_start..];

        // Find the closing `]` of the alt text.
        let Some(alt_end) = remaining.find("](") else {
            result.push_str("![");
            remaining = &remaining["![".len()..];
            continue;
        };
        let alt_part = &remaining[..alt_end + "](".len()]; // "![alt]("
        let after_paren = &remaining[alt_end + "](".len()..];

        // Find the closing `)`.
        let Some(close) = after_paren.find(')') else {
            result.push_str(alt_part);
            remaining = after_paren;
            continue;
        };
        let raw_path = &after_paren[..close];

        // Skip already-absolute or URL paths.
        if raw_path.starts_with("http://")
            || raw_path.starts_with("https://")
            || raw_path.starts_with("file://")
            || raw_path.starts_with('/')
        {
            result.push_str(alt_part);
            result.push_str(raw_path);
            result.push(')');
        } else {
            let resolved = base_dir.join(raw_path);
            let canonical = resolved.canonicalize().unwrap_or(resolved);
            result.push_str(alt_part);
            result.push_str("file://");
            result.push_str(&canonical.display().to_string());
            result.push(')');
        }
        remaining = &after_paren[close + 1..];
    }
    result.push_str(remaining);
    result
}

/// Strips indentation from code fences that appear inside list items so
/// that `pulldown_cmark` treats them as top-level block elements.
///
/// # Why this is needed
///
/// `egui_commonmark` (v0.22) renders list item content inside
/// `ui.horizontal_wrapped()`, which forces **all** child elements—including
/// code blocks—into a single horizontal line. This is a fundamental
/// limitation of `egui`'s layout system and cannot be fixed by patching the
/// renderer (tested with multiple patch strategies, all failed due to egui
/// not re-allocating width after block elements).
///
/// By removing the leading whitespace from indented code fences,
/// `pulldown_cmark` sees them as top-level code blocks outside the list.
/// The list is split around the code block, which is the correct visual
/// result: the code block appears between list items as a standalone block.
pub fn flatten_list_code_blocks(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = String::with_capacity(source.len());
    let mut in_indented_fence = false;
    let mut fence_indent = 0;

    for line in &lines {
        if in_indented_fence {
            // Strip up to `fence_indent` spaces from the front.
            let stripped = strip_leading_spaces(line, fence_indent);
            let trimmed = stripped.trim_start();
            if trimmed.starts_with("```") {
                // Closing fence — also de-indent, then leave fence mode.
                result.push_str(trimmed);
                result.push('\n');
                in_indented_fence = false;
            } else {
                result.push_str(stripped);
                result.push('\n');
            }
        } else {
            let indent = count_leading_spaces(line);
            let trimmed = line.trim_start();
            if indent >= 2 && trimmed.starts_with("```") {
                // Indented opening fence — de-indent it.
                in_indented_fence = true;
                fence_indent = indent;
                result.push_str(trimmed);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
    }

    // Preserve the original trailing-newline behaviour.
    if !source.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

fn count_leading_spaces(s: &str) -> usize {
    s.bytes().take_while(|b| *b == b' ').count()
}

fn strip_leading_spaces(s: &str, max: usize) -> &str {
    let n = count_leading_spaces(s).min(max);
    &s[n..]
}

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

    loop {
        // Find the next fence: either at the very start of remaining, or after a newline.
        let fence_offset = if remaining.starts_with("```") {
            Some(0)
        } else {
            remaining.find("\n```").map(|pos| pos + 1)
        };
        let Some(offset) = fence_offset else {
            break;
        };

        markdown_acc.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
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
