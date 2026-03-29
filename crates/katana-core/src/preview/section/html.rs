use regex::Regex;

/* WHY: Wraps standalone lines containing only `<a>` or `<img>` tags in `<div>` blocks.
A "standalone" line is one where the trimmed content starts with `<a ` or `<img `
and ends with the corresponding closing (`</a>` or `>`), with no surrounding
Markdown text on adjacent non-blank lines that would make it part of a paragraph.
This converts inline-level HTML into block-level HTML so that pulldown-cmark
emits `Tag::HtmlBlock` events and our `render_html_fn` callback can handle them. */
pub fn wrap_standalone_inline_html(text: &str) -> String {
    #[rustfmt::skip]
    let inline_re = Regex::new(r"^[ \t]*(<a\s[^>]*>.*?</a>|<img\s[^>]*>)[ \t]*$").expect("ok");
    #[rustfmt::skip]
    let open_re = Regex::new(r"(?i)^[ \t]*<(p|div|h[1-6]|section|article|header|footer|nav|main|aside)\b").expect("ok");
    #[rustfmt::skip]
    let close_re = Regex::new(r"(?i)</(p|div|h[1-6]|section|article|header|footer|nav|main|aside)>").expect("ok");

    let mut res = String::with_capacity(text.len());
    let mut d = 0;
    for l in text.lines() {
        if open_re.is_match(l) {
            d += 1;
        }
        if d == 0 && inline_re.is_match(l) {
            res.push_str(&format!("<div>\n{}\n</div>", l.trim()));
        } else {
            res.push_str(l);
        }
        res.push('\n');
        if close_re.is_match(l) && d > 0 {
            d -= 1;
        }
    }
    if !text.ends_with('\n') && res.ends_with('\n') {
        res.pop();
    }
    res
}
