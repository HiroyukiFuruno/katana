use std::borrow::Cow;

/* WHY: Pre-processes text to intercept `$ E = mc^2 $` and transforms it to `$E = mc^2$` automatically.
To prevent converting text like "Costs $ 5 and $ 10" to math, we use a
heuristic that requires characteristic math symbols or letters inside the block. */
pub fn process_relaxed_math(source: &str) -> Cow<'_, str> {
    let re = regex::Regex::new(r"(?m)(^|[^\\])\$([ \t]+)([^$\n]+?)([ \t]+)\$").unwrap();
    re.replace_all(source, |caps: &regex::Captures| {
        const REGEX_PREFIX_GROUP: usize = 1;
        const REGEX_CONTENT_GROUP: usize = 3;
        let prefix = caps.get(REGEX_PREFIX_GROUP).unwrap().as_str();
        let content = caps.get(REGEX_CONTENT_GROUP).unwrap().as_str();

        // WHY: Heuristic: Does this contain any characteristic math symbols or ascii letters?
        let is_math = content
            .chars()
            .any(|c| c.is_ascii_alphabetic() || "=\\+-*/^_<>()[]{}|".contains(c));

        if is_math {
            // WHY: Strip the spaces inside the delimiters so pulldown-cmark parses it as Math
            format!("{}${}$", prefix, content)
        } else {
            /* WHY: Leave the original text unmodified (spaces intact), so pulldown-cmark
            treats it as plain text and avoids false positive SVG rendering. */
            caps.get(0).unwrap().as_str().to_string()
        }
    })
}
