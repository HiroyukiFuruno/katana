use crate::Violation;
use std::path::Path;

/// Enforces that all inline comments must start with `// WHY:`.
/// Code should be self-documenting; comments should only explain *why*,
/// never *what*.
///
/// Allowed:
/// - `// WHY: ...` (the only permitted inline comment form)
/// - `// SAFETY: ...` (Rust convention for unsafe blocks)
/// - Doc comments (`///`, `//!`)
/// - Section separators (lines containing only `/`, `-`, `─`, `═`, `=`, spaces)
///
/// Disallowed:
/// - `// TODO: ...`, `// FIXME: ...`, `// HACK: ...`
/// - `// Phase 1: ...`, `// some explanation`
/// - Any other inline comment
pub fn lint_comment_style(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let mut violations = Vec::new();
    let mut in_test_module = false;
    let mut in_allowed_block = false;
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            in_test_module = true;
        }
        if in_test_module {
            continue;
        }
        if !is_inline_comment(trimmed) {
            in_allowed_block = false;
            continue;
        }
        let body = extract_comment_body(trimmed);
        if starts_allowed_block(body) {
            in_allowed_block = true;
            continue;
        }
        if in_allowed_block || is_allowed_comment(body) {
            continue;
        }
        violations.push(build_violation(path, trimmed, line_idx));
    }
    violations
}

fn build_violation(path: &Path, trimmed: &str, line_idx: usize) -> Violation {
    Violation {
        file: path.to_path_buf(),
        line: line_idx + 1,
        column: 1,
        message: format!(
            "Comment must start with `// WHY:` or `// SAFETY:`. Found: `{}`",
            truncate(trimmed, 60)
        ),
    }
}

fn starts_allowed_block(body: &str) -> bool {
    body.starts_with("WHY:") || body.starts_with("SAFETY:")
}

fn is_inline_comment(trimmed: &str) -> bool {
    if !trimmed.starts_with("//") {
        return false;
    }
    // WHY: doc comments (/// and //!) are documentation, not inline comments
    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }
    true
}

fn extract_comment_body(trimmed: &str) -> &str {
    trimmed.strip_prefix("//").unwrap_or("").trim()
}

fn is_allowed_comment(body: &str) -> bool {
    if body.is_empty() {
        return true;
    }
    if body.starts_with("WHY:") || body.starts_with("SAFETY:") {
        return true;
    }
    is_separator_line(body)
}

fn is_separator_line(body: &str) -> bool {
    body.chars()
        .all(|c| matches!(c, '-' | '─' | '═' | '=' | ' ' | '/' | '━'))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_temp_file(content: &str) -> (tempfile::TempDir, PathBuf) {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (dir, file_path)
    }

    #[test]
    fn allows_why_comment() {
        let code = "// WHY: Business rule requires this fallback\nfn foo() {}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_comment_style(&path, &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_safety_comment() {
        let code = "// SAFETY: pointer is guaranteed non-null by caller\nfn foo() {}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_comment_style(&path, &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_doc_comment() {
        let code = "/// Documents the function\nfn foo() {}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_comment_style(&path, &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_separator() {
        let code = "// ─────────────────────────────\nfn foo() {}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_comment_style(&path, &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn rejects_plain_comment() {
        let code = "// this explains what the code does\nfn foo() {}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_comment_style(&path, &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("WHY:"));
    }

    #[test]
    fn rejects_todo_comment() {
        let code = "// TODO: fix this later\nfn foo() {}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_comment_style(&path, &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn skips_test_module() {
        let code = "#[cfg(test)]\nmod tests {\n    // plain comment in test is fine\n}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_comment_style(&path, &syntax);
        assert!(violations.is_empty());
    }
}
