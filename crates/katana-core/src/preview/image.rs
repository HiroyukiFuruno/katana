use regex::Regex;
use std::path::Path;

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
        if is_absolute_url(&raw_path) {
            continue;
        }
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

fn find_image_replacements<'a>(
    root: &'a comrak::nodes::AstNode<'a>,
    source: &str,
) -> Vec<(usize, usize, String)> {
    let mut offsets = vec![0];
    for (i, c) in source.char_indices() {
        if c == '\n' {
            offsets.push(i + 1);
        }
    }
    let mut replacements = Vec::new();
    for node in root.descendants() {
        process_image_node(node, source, &offsets, &mut replacements);
    }
    replacements
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_resolve_image_paths() {
        let source = "![local](../img.png) ![abs](file:///img.png)";
        let file_path = PathBuf::from("/docs/doc.md");
        let (resolved, extracted) = resolve_image_paths(source, &file_path);
        assert!(
            resolved.contains("file:///docs/../img.png")
                || resolved.contains("file:///docs/img.png")
        ); // Depends on canonicalization
        assert!(resolved.contains("file:///img.png"));
        assert!(extracted.len() >= 1);
    }

    #[test]
    fn test_find_image_replacements() {
        use comrak::{parse_document, Arena, Options};
        let arena = Arena::new();
        let source = "![test](test.png)\n![test2](http://example.com/test.png)";
        let root = parse_document(&arena, source, &Options::default());
        let replacements = find_image_replacements(root, source);
        assert_eq!(replacements.len(), 2);
    }
}

fn process_image_node(
    node: &comrak::nodes::AstNode<'_>,
    source: &str,
    offsets: &[usize],
    replacements: &mut Vec<(usize, usize, String)>,
) {
    let comrak::nodes::NodeValue::Image(ref img) = node.data.borrow().value else {
        return;
    };
    let pos = node.data.borrow().sourcepos;
    let start = offsets.get(pos.start.line.saturating_sub(1)).unwrap_or(&0)
        + pos.start.column.saturating_sub(1);
    let end = offsets.get(pos.end.line.saturating_sub(1)).unwrap_or(&0)
        + pos.end.column.saturating_sub(1);

    if start > end || end > source.len() || img.url.is_empty() {
        return;
    }

    if let Some(idx) = source[start..end].rfind(img.url.as_str()) {
        replacements.push((
            start + idx,
            start + idx + img.url.len(),
            img.url.to_string(),
        ));
    }
}

pub fn resolve_html_image_paths(html: &str, md_file_path: &Path) -> String {
    const CAP_PREFIX: usize = 1;
    const CAP_SRC: usize = 2;
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
