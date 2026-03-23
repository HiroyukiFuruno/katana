use std::path::Path;

use crate::markdown::diagram::DiagramKind;
use regex::Regex;

/// Resolves relative image paths in Markdown source to absolute `file://` URIs.
///
/// Given the path to the Markdown file being previewed, rewrites image references
/// like `![alt](../assets/image.png)` to `![alt](file:///absolute/path/assets/image.png)`.
///
/// Already-absolute paths, URLs (`http://`, `https://`), and `file://` URIs are left unchanged.
pub fn resolve_image_paths(source: &str, md_file_path: &Path) -> (String, Vec<std::path::PathBuf>) {
    use comrak::nodes::NodeValue;
    use comrak::{parse_document, Arena, Options};

    let arena = Arena::new();
    let root = parse_document(&arena, source, &Options::default());

    let mut line_offsets = vec![0];
    for (i, c) in source.char_indices() {
        if c == '\n' {
            line_offsets.push(i + 1);
        }
    }

    // Collect all Image AST nodes' source positions
    let mut replacements = Vec::new();
    for node in root.descendants() {
        if let NodeValue::Image(ref img) = node.data.borrow().value {
            let pos = node.data.borrow().sourcepos;
            let start_line_idx = pos.start.line.saturating_sub(1);
            let start_col_offset = pos.start.column.saturating_sub(1);
            let end_line_idx = pos.end.line.saturating_sub(1);
            let end_col_offset = pos.end.column.saturating_sub(1);

            let start_byte = line_offsets.get(start_line_idx).unwrap_or(&0) + start_col_offset;
            let end_byte = line_offsets.get(end_line_idx).unwrap_or(&0) + end_col_offset;

            if start_byte < source.len() && end_byte <= source.len() && start_byte <= end_byte {
                let node_str = &source[start_byte..end_byte];
                // We know node_str is something like `![alt](url)` or `![alt][ref]`
                // Let's find the URL portion. This is complex if `alt` texts contain parens.
                // However, we know `img.url` is exactly the URL string from the AST.
                let url_str = &img.url;
                if !url_str.is_empty() {
                    // Find the exact occurrence of `url_str` inside `node_str` searching from the end
                    if let Some(url_idx) = node_str.rfind(url_str.as_str()) {
                        replacements.push((
                            start_byte + url_idx,
                            start_byte + url_idx + url_str.len(),
                            url_str.to_string(),
                        ));
                    }
                }
            }
        }
    }

    // Sort replacements by start byte in reverse order to safely perform string replacements from the end
    replacements.sort_by_key(|&(start, _, _)| std::cmp::Reverse(start));

    let base_dir = md_file_path.parent().unwrap_or(Path::new("."));
    let mut result = source.to_string();

    let mut extracted_paths = Vec::new();

    for (start, end, raw_path) in replacements {
        if raw_path.starts_with("http://")
            || raw_path.starts_with("https://")
            || raw_path.starts_with("file://")
            || raw_path.starts_with("data:")
            || raw_path.starts_with('/')
        {
            continue;
        }

        let resolved = base_dir.join(&raw_path);
        let canonical = resolved.canonicalize().unwrap_or(resolved);
        let absolute_url = format!("file://{}", canonical.display());

        extracted_paths.push(canonical);

        result.replace_range(start..end, &absolute_url);
    }

    (result, extracted_paths)
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
            || src.starts_with("data:")
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
    /// A standalone local image.
    LocalImage { path: String, alt: String },
}

/// Splits the source text into a list of `PreviewSection`s.
///
/// Detects diagram fences (` ```mermaid` / ` ```plantuml` / ` ```drawio` ),
/// and groups the rest as Markdown sections.
pub fn split_into_sections(source: &str) -> Vec<PreviewSection> {
    let mut initial_sections = Vec::new();
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
                // Do not wrap here, we will wrap in the final merge pass
                if !markdown_acc.is_empty() {
                    initial_sections
                        .push(PreviewSection::Markdown(std::mem::take(&mut markdown_acc)));
                }
                initial_sections.push(PreviewSection::Diagram {
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
    if !markdown_acc.is_empty() {
        initial_sections.push(PreviewSection::Markdown(std::mem::take(&mut markdown_acc)));
    }

    // Extract standalone images from the markdown blocks
    let img_re = Regex::new(r"(?m)^[ \t]*!\[([^\]]*)\]\(([^\)]+)\)[ \t]*$").unwrap();
    let mut temp = Vec::new();

    for sec in initial_sections {
        match sec {
            PreviewSection::Markdown(text) => {
                let mut last_end = 0;
                for cap in img_re.captures_iter(&text) {
                    let m = cap.get(0).unwrap();
                    let alt = cap.get(1).unwrap().as_str().to_string();
                    let url = cap.get(2).unwrap().as_str().to_string();

                    let before = &text[last_end..m.start()];
                    if !before.trim().is_empty() {
                        temp.push(PreviewSection::Markdown(before.to_string()));
                    }

                    temp.push(PreviewSection::LocalImage { path: url, alt });
                    last_end = m.end();
                }

                let after = &text[last_end..];
                if !after.trim().is_empty() {
                    temp.push(PreviewSection::Markdown(after.to_string()));
                }
            }
            other => temp.push(other),
        }
    }

    // Merge adjacent text sections and apply HTML wrapper
    let mut merged = Vec::new();
    let mut md_acc = String::new();
    for sec in temp {
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

#[cfg(test)]
mod sourcepos_tests {
    use comrak::nodes::NodeValue;
    use comrak::{parse_document, Arena, Options};

    #[test]
    fn test_sourcepos_bytes() {
        let arena = Arena::new();
        //                   0         1         2
        //                   0123456789012345678901234567
        let src = "Hello\nThis is an ![alt](test.png) text\n";
        let doc = parse_document(&arena, src, &Options::default());
        for node in doc.descendants() {
            if let NodeValue::Image(_) = node.data.borrow().value {
                let pos = node.data.borrow().sourcepos;
                let lines: Vec<&str> = src.lines().collect();
                let line = lines[pos.start.line - 1];
                let extracted = &line[pos.start.column - 1..pos.end.column];
                assert_eq!(extracted, "![alt](test.png)");
            }
        }
    }
}

#[cfg(test)]
mod split_tests {
    use super::*;

    #[test]
    fn test_split_with_mixed_diagram_and_image() {
        let md = "```mermaid\ngraph TD\nA-->B\n```\n![alt](url)\nText";
        let sections = split_into_sections(md);
        assert_eq!(sections.len(), 3);
        assert!(matches!(
            sections[0],
            PreviewSection::Diagram {
                kind: crate::preview::DiagramKind::Mermaid,
                ..
            }
        ));
        assert!(matches!(sections[1], PreviewSection::LocalImage { .. }));
        assert!(matches!(sections[2], PreviewSection::Markdown(_)));
    }
}
