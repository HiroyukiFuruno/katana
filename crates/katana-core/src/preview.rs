use std::path::Path;

use crate::markdown::diagram::DiagramKind;
use regex::Regex;

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

/// Resolves relative `src` attributes in HTML `<img>` tags to absolute `file://` URIs.
///
/// This is the HTML counterpart of [`resolve_image_paths`], handling raw HTML
/// image tags within `HtmlBlock` sections.
pub fn resolve_html_image_paths(html: &str, md_file_path: &Path) -> String {
    /// Regex capture group index for the `<img ... src="` prefix.
    const CAP_PREFIX: usize = 1;
    /// Regex capture group index for the `src` attribute value.
    const CAP_SRC: usize = 2;
    /// Regex capture group index for the `" ...>` suffix.
    const CAP_SUFFIX: usize = 3;

    let base_dir = md_file_path.parent().unwrap_or(Path::new("."));
    let re = Regex::new(r#"(<img\s[^>]*src\s*=\s*")([^"]+)("[^>]*>)"#).unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let prefix = caps.get(CAP_PREFIX).unwrap().as_str();
        let src = caps.get(CAP_SRC).unwrap().as_str();
        let suffix = caps.get(CAP_SUFFIX).unwrap().as_str();
        if src.starts_with("http://")
            || src.starts_with("https://")
            || src.starts_with("file://")
            || src.starts_with('/')
        {
            format!("{prefix}{src}{suffix}")
        } else {
            let resolved = base_dir.join(src);
            let canonical = resolved.canonicalize().unwrap_or(resolved);
            format!("{prefix}file://{}{suffix}", canonical.display())
        }
    })
    .into_owned()
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
///
/// HTML blocks within the text are handled by pulldown-cmark (via egui_commonmark's
/// `render_html_fn` callback). However, pulldown-cmark only recognises certain tags
/// as block-level HTML starters (p, div, h1-h6, table, etc. per CommonMark spec §4.6).
/// Standalone `<a>` and `<img>` tags are NOT in that list, so they would be treated
/// as inline HTML and rendered as plain text.
///
/// To handle this, we wrap standalone lines that consist entirely of an `<a>` or `<img>`
/// tag in a `<div>` wrapper, converting them into block-level HTML for pulldown-cmark.
fn flush_markdown(sections: &mut Vec<PreviewSection>, acc: &mut String) {
    if acc.is_empty() {
        return;
    }
    let text = std::mem::take(acc);
    let processed = wrap_standalone_inline_html(&text);
    sections.push(PreviewSection::Markdown(processed));
}

/// Wraps standalone lines containing only `<a>` or `<img>` tags in `<div>` blocks.
///
/// A "standalone" line is one where the trimmed content starts with `<a ` or `<img `
/// and ends with the corresponding closing (`</a>` or `>`), with no surrounding
/// Markdown text on adjacent non-blank lines that would make it part of a paragraph.
///
/// This converts inline-level HTML into block-level HTML so that pulldown-cmark
/// emits `Tag::HtmlBlock` events and our `render_html_fn` callback can handle them.
pub fn wrap_standalone_inline_html(text: &str) -> String {
    let inline_re =
        Regex::new(r"^[ \t]*(<a\s[^>]*>.*?</a>|<img\s[^>]*>)[ \t]*$").expect("valid regex");
    // Block-level HTML elements whose children should not be wrapped.
    let open_re =
        Regex::new(r"(?i)^[ \t]*<(p|div|h[1-6]|section|article|header|footer|nav|main|aside)\b")
            .expect("valid regex");
    let close_re =
        Regex::new(r"(?i)</(p|div|h[1-6]|section|article|header|footer|nav|main|aside)>")
            .expect("valid regex");

    let mut result = String::with_capacity(text.len());
    let mut block_depth: usize = 0;

    for line in text.lines() {
        // Track nesting: count opens before closes on this line.
        if open_re.is_match(line) {
            block_depth += 1;
        }
        let is_close = close_re.is_match(line);

        if block_depth == 0 && inline_re.is_match(line) {
            result.push_str(&format!("<div>\n{}\n</div>", line.trim()));
        } else {
            result.push_str(line);
        }
        result.push('\n');

        if is_close && block_depth > 0 {
            block_depth -= 1;
        }
    }

    // Remove trailing newline added by the loop if the original didn't end with one.
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
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
