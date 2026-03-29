use regex::Regex;
use std::path::Path;

/// Resolves relative image paths in Markdown source to absolute `file://` URIs.
///
/// Given the path to the Markdown file being previewed, rewrites image references
/// like `![alt](../assets/image.png)` to `![alt](file:///absolute/path/assets/image.png)`.
///
/// Already-absolute paths, URLs (`http://`, `https://`), and `file://` URIs are left unchanged.
pub fn resolve_image_paths(source: &str, md_file_path: &Path) -> (String, Vec<std::path::PathBuf>) {
    use comrak::{parse_document, Arena, Options};

    let arena = Arena::new();
    let root = parse_document(&arena, source, &Options::default());
    let mut replacements = find_image_replacements(root, source);
    replacements.sort_by_key(|&(start, _, _)| std::cmp::Reverse(start));

    let base_dir = md_file_path.parent().unwrap_or(Path::new("."));
    let mut result = source.to_string();
    let mut extracted_paths = Vec::new();

    for (start, end, raw_path) in replacements {
        if is_absolute_url(&raw_path) { continue; }
        let resolved = base_dir.join(&raw_path);
        let canonical = resolved.canonicalize().unwrap_or(resolved);
        extracted_paths.push(canonical.clone());
        result.replace_range(start..end, &format!("file://{}", canonical.display()));
    }
    (result, extracted_paths)
}

fn is_absolute_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("file://")
        || url.starts_with("data:")
        || url.starts_with('/')
}

fn find_image_replacements<'a>(root: &'a comrak::nodes::AstNode<'a>, source: &str) -> Vec<(usize, usize, String)> {
    let mut offsets = vec![0];
    for (i, c) in source.char_indices() {
        if c == '\n' { offsets.push(i + 1); }
    }
    let mut replacements = Vec::new();
    for node in root.descendants() {
        process_image_node(node, source, &offsets, &mut replacements);
    }
    replacements
}

fn process_image_node(
    node: &comrak::nodes::AstNode<'_>,
    source: &str,
    offsets: &[usize],
    replacements: &mut Vec<(usize, usize, String)>,
) {
    let comrak::nodes::NodeValue::Image(ref img) = node.data.borrow().value else { return; };
    let pos = node.data.borrow().sourcepos;
    let start = offsets.get(pos.start.line.saturating_sub(1)).unwrap_or(&0) + pos.start.column.saturating_sub(1);
    let end = offsets.get(pos.end.line.saturating_sub(1)).unwrap_or(&0) + pos.end.column.saturating_sub(1);

    if start > end || end > source.len() || img.url.is_empty() { return; }

    if let Some(idx) = source[start..end].rfind(img.url.as_str()) {
        replacements.push((start + idx, start + idx + img.url.len(), img.url.to_string()));
    }
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
        if is_absolute_url(src) {
            format!("{prefix}{src}{suffix}")
        } else {
            let resolved = base_dir.join(src);
            let canonical = resolved.canonicalize().unwrap_or(resolved);
            format!("{prefix}file://{}{suffix}", canonical.display())
        }
    })
    .to_string()
}
