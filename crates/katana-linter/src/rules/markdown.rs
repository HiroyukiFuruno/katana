use crate::utils::locale_violation;
use crate::Violation;
use ignore::WalkBuilder;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownHeading {
    level: u8,
    line: usize,
}

#[derive(Debug, Clone)]
struct MarkdownPair {
    base: PathBuf,
    ja: PathBuf,
}

pub fn lint_markdown_heading_pairs(root: &Path) -> Vec<Violation> {
    let mut violations = Vec::new();
    for pair in collect_markdown_pairs(root) {
        violations.extend(compare_markdown_heading_structure(&pair));
    }
    violations
}

fn collect_markdown_files(root: &Path) -> Vec<PathBuf> {
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

fn markdown_pair_key(path: &Path) -> Option<(String, bool)> {
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

fn collect_markdown_pairs(root: &Path) -> Vec<MarkdownPair> {
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

fn extract_markdown_headings(path: &Path) -> Result<Vec<MarkdownHeading>, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Markdown file read error: {err}"),
        }]
    })?;

    let mut in_fence = false;
    let mut headings = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let hashes = trimmed.chars().take_while(|char| *char == '#').count();
        if !(1..=6).contains(&hashes) {
            continue;
        }

        let rest = &trimmed[hashes..];
        if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
            continue;
        }

        headings.push(MarkdownHeading {
            level: hashes as u8,
            line: index + 1,
        });
    }

    Ok(headings)
}

fn compare_markdown_heading_structure(pair: &MarkdownPair) -> Vec<Violation> {
    let base_headings = match extract_markdown_headings(&pair.base) {
        Ok(headings) => headings,
        Err(violations) => return violations,
    };
    let ja_headings = match extract_markdown_headings(&pair.ja) {
        Ok(headings) => headings,
        Err(violations) => return violations,
    };

    let mut violations = Vec::new();

    if base_headings.len() != ja_headings.len() {
        violations.push(locale_violation(
            &pair.ja,
            format!(
                "Markdown heading count mismatch between `{}` and `{}`: {} vs {}.",
                pair.base.display(),
                pair.ja.display(),
                base_headings.len(),
                ja_headings.len()
            ),
        ));
    }

    for (index, (base_heading, ja_heading)) in
        base_headings.iter().zip(ja_headings.iter()).enumerate()
    {
        if base_heading.level != ja_heading.level {
            violations.push(Violation {
                file: pair.ja.clone(),
                line: ja_heading.line,
                column: 1,
                message: format!(
                    "Markdown heading level mismatch at heading {} compared with `{}`: expected H{}, found H{}.",
                    index + 1,
                    pair.base.display(),
                    base_heading.level,
                    ja_heading.level
                ),
            });
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_markdown_headings_ignores_code_blocks() {
        let tmp = tempfile::NamedTempFile::with_suffix(".md").unwrap();
        std::fs::write(tmp.path(), "# H1\n```\n# Not a heading\n```\n## H2").unwrap();

        let headings = extract_markdown_headings(tmp.path()).unwrap();
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[1].level, 2);
    }

    #[test]
    fn compare_markdown_heading_structure_detects_mismatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base_path = tmp.path().join("base.md");
        let ja_path = tmp.path().join("base.ja.md");

        std::fs::write(&base_path, "# H1\n## H2").unwrap();
        std::fs::write(&ja_path, "# H1\n### H3").unwrap();

        let pair = MarkdownPair {
            base: base_path,
            ja: ja_path,
        };
        let violations = compare_markdown_heading_structure(&pair);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected H2, found H3"));
    }
}
