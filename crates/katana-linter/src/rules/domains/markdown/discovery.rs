use ignore::WalkBuilder;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MarkdownPair {
    pub base: PathBuf,
    pub ja: PathBuf,
}

pub fn collect_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .require_git(false)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            files.push(path.to_path_buf());
        }
    }

    files.sort();
    files
}

pub fn markdown_pair_key(path: &Path) -> Option<(String, bool)> {
    let path_str = path.to_string_lossy();
    if let Some(prefix) = path_str.strip_suffix(".ja.md") {
        return Some((prefix.to_string(), true));
    }
    if let Some(prefix) = path_str.strip_suffix("_ja.md") {
        return Some((prefix.to_string(), true));
    }
    path_str
        .strip_suffix(".md")
        .map(|prefix| (prefix.to_string(), false))
}

pub fn collect_markdown_pairs(root: &Path) -> Vec<MarkdownPair> {
    let files = collect_markdown_files(root);
    let mut base_files = BTreeMap::<String, PathBuf>::new();
    let mut ja_files = BTreeMap::<String, PathBuf>::new();

    for file in files {
        if let Some((key, is_ja)) = markdown_pair_key(&file) {
            if is_ja {
                ja_files.insert(key, file);
            } else {
                base_files.insert(key, file);
            }
        }
    }

    let mut pairs = Vec::new();
    for (key, base) in base_files {
        if let Some(ja) = ja_files.remove(&key) {
            pairs.push(MarkdownPair { base, ja });
        }
    }

    pairs
}
