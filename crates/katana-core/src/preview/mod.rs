//! Pre-processing logic for the markdown preview pipeline.
//!
//! Handles diagram fence detection, image resolution, and HTML inline wrapping
//! before passing to the markdown renderer.

pub mod image;
pub mod section;
#[cfg(test)]
mod tests;

pub use image::*;
pub use section::*;

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
    let mut result = String::with_capacity(source.len());
    let mut in_indented_fence = false;
    let mut fence_indent = 0;

    for line in source.lines() {
        if in_indented_fence {
            let stripped = strip_leading_spaces(line, fence_indent);
            if stripped.trim_start().starts_with("```") {
                result.push_str(stripped.trim_start());
                in_indented_fence = false;
            } else {
                result.push_str(stripped);
            }
        } else {
            let indent = count_leading_spaces(line);
            if indent >= 2 && line.trim_start().starts_with("```") {
                in_indented_fence = true;
                fence_indent = indent;
                result.push_str(line.trim_start());
            } else {
                result.push_str(line);
            }
        }
        result.push('\n');
    }

    if !source.ends_with('\n') && result.ends_with('\n') { result.pop(); }
    result
}

fn count_leading_spaces(s: &str) -> usize {
    s.bytes().take_while(|b| *b == b' ').count()
}

fn strip_leading_spaces(s: &str, max: usize) -> &str {
    let n = count_leading_spaces(s).min(max);
    &s[n..]
}
