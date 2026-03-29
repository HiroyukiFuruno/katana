use crate::Violation;
use std::path::Path;

pub fn lint_comment_style(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return vec![];
    };
    let mut violations = Vec::new();
    let mut in_test = false;
    let mut in_allow = false;
    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            in_test = true;
        }
        if in_test {
            continue;
        }
        if let Some(msg) = check_line(line, &mut in_allow) {
            violations.push(build_viol(path, msg, idx));
        }
    }
    violations
}

fn check_line<'a>(line: &'a str, in_allow: &mut bool) -> Option<&'a str> {
    let Some(start) = find_comment_start(line) else {
        *in_allow = false;
        return None;
    };
    let text = &line[start..];
    if text.starts_with("///") || text.starts_with("//!") {
        return None;
    }
    let body = extract_body(text.trim());
    if body.starts_with("WHY:") || body.starts_with("SAFETY:") {
        *in_allow = true;
        return None;
    }
    if *in_allow || is_allowed(body) {
        return None;
    }
    Some(text.trim())
}

fn find_comment_start(line: &str) -> Option<usize> {
    if line.contains("r#\"") {
        return None;
    }
    let mut in_str = false;
    let mut in_char = false;
    let mut escape = false;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if escape {
            escape = false;
            continue;
        }
        match bytes[i] {
            b'\\' => escape = true,
            b'"' if !in_char => in_str = !in_str,
            b'\'' if !in_str => in_char = !in_char,
            b'/' if !in_str && !in_char => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn build_viol(path: &Path, trimmed: &str, line_idx: usize) -> Violation {
    let msg = format!(
        "Comment must start with `// WHY:` or `// SAFETY:`. Found: `{}`",
        truncate(trimmed, 60)
    );
    Violation {
        file: path.to_path_buf(),
        line: line_idx + 1,
        column: 1,
        message: msg,
    }
}

fn extract_body(trimmed: &str) -> &str {
    trimmed.strip_prefix("//").unwrap_or("").trim()
}

fn is_allowed(body: &str) -> bool {
    body.is_empty()
        || body
            .chars()
            .all(|c| matches!(c, '-' | '─' | '═' | '=' | ' ' | '/' | '━'))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_tmp(content: &str) -> (tempfile::TempDir, PathBuf) {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.rs");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        (dir, path)
    }

    #[test]
    fn allows_why_safety() {
        let code = "// WHY: reason\n// SAFETY: ptr ok\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).is_empty());
    }

    #[test]
    fn allows_doc_and_sep() {
        let code = "/// doc\n// ---\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).is_empty());
    }

    #[test]
    fn rejects_invalid() {
        let code = "// invalid\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert_eq!(
            lint_comment_style(&p, &syn::parse_file(code).unwrap()).len(),
            1
        );
    }
}
