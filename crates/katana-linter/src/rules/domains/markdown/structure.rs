use crate::utils::locale_violation;
use crate::Violation;
use std::path::Path;

use super::discovery::MarkdownPair;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownHeading {
    pub level: u8,
    pub line: usize,
}

pub fn extract_markdown_headings(path: &Path) -> Result<Vec<MarkdownHeading>, Vec<Violation>> {
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

pub fn compare_markdown_heading_structure(pair: &MarkdownPair) -> Vec<Violation> {
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
