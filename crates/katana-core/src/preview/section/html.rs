use regex::Regex;

/// Wraps standalone lines containing only `<a>` or `<img>` tags in `<div>` blocks.
///
/// A "standalone" line is one where the trimmed content starts with `<a ` or `<img `
/// and ends with the corresponding closing (`</a>` or `>`), with no surrounding
/// Markdown text on adjacent non-blank lines that would make it part of a paragraph.
///
/// This converts inline-level HTML into block-level HTML so that pulldown-cmark
/// emits `Tag::HtmlBlock` events and our `render_html_fn` callback can handle them.
pub fn wrap_standalone_inline_html(text: &str) -> String {
    let inline_re = Regex::new(r"^[ \t]*(<a\s[^>]*>.*?</a>|<img\s[^>]*>)[ \t]*$").expect("valid regex");
    let open_re = Regex::new(r"(?i)^[ \t]*<(p|div|h[1-6]|section|article|header|footer|nav|main|aside)\b").expect("valid regex");
    let close_re = Regex::new(r"(?i)</(p|div|h[1-6]|section|article|header|footer|nav|main|aside)>").expect("valid regex");

    let mut result = String::with_capacity(text.len());
    let mut block_depth: usize = 0;

    for line in text.lines() {
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

    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}
